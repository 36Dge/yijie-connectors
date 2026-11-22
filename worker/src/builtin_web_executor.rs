//! One server-owned request, using the existing Kimi adapter and one-shot HTTP
//! permit. No model, discovery, user vault, connector selection or approval UI.
use crate::{builtin_web_generated as wire, daily::Permit, kimi_web, tushare_oauth::Diagnostic};
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Read, Write},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio_util::sync::CancellationToken;

pub(crate) fn validate(name: &str, value: &Value) -> bool {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    static VALIDATORS: OnceLock<Mutex<HashMap<String, Arc<jsonschema::Validator>>>> =
        OnceLock::new();
    let compiled = {
        let mut validators = VALIDATORS
            .get_or_init(Default::default)
            .lock()
            .expect("web schema cache");
        if let Some(c) = validators.get(name) {
            Arc::clone(c)
        } else {
            let schema = SCHEMA.get_or_init(|| {
                serde_json::from_str(wire::SCHEMA_JSON).expect("generated web schema")
            });
            if schema["$defs"].get(name).is_none() {
                return false;
            }
            let mut schema = schema.clone();
            schema["$ref"] = json!(format!("#/$defs/{name}"));
            let Ok(v) = jsonschema::options()
                .should_validate_formats(true)
                .build(&schema)
            else {
                return false;
            };
            let c = Arc::new(v);
            validators.insert(name.to_owned(), Arc::clone(&c));
            c
        }
    };
    compiled.is_valid(value)
}
pub(crate) fn error(code: &str) -> Value {
    let messages: Value =
        serde_json::from_str(wire::ERROR_MESSAGES_JSON).expect("generated errors");
    json!({"error":{"code":code,"message":messages[code]}})
}
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn execute(
    value: Value,
    backend: Arc<kimi_web::Backend>,
    cancellation: CancellationToken,
    diagnostic: Diagnostic,
) -> Value {
    if !validate("ExecutorWorkerRequest", &value) {
        return error("invalid_arguments");
    }
    let execute = &value["execute"];
    let receipt = &value["receipt"];
    let request = &execute["request"];
    let deadline = execute["deadline_unix_ms"].as_u64().unwrap_or(0);
    if receipt["operation_id"] != request["operation_id"]
        || receipt["parameter_digest"] != execute["parameter_digest"]
        || receipt["deadline_unix_ms"] != execute["deadline_unix_ms"]
        || deadline <= now_ms()
        || deadline > now_ms() + 45000
    {
        return error("operation_conflict");
    }
    let Some(tool) = request["tool"].as_str() else {
        return error("invalid_arguments");
    };
    let Some(arguments) = request["arguments"].as_object() else {
        return error("invalid_arguments");
    };
    if backend.review(tool, arguments).is_err() {
        return error("invalid_arguments");
    }
    let permit = Permit {
        tool: tool.into(),
        arguments: Value::Object(arguments.clone()),
        request: cancellation.clone(),
        lease: cancellation.clone(),
        backend: cancellation.clone(),
        deadline: Instant::now() + Duration::from_millis(deadline.saturating_sub(now_ms())),
    };
    let result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => { return error("cancelled") },
        result = backend.retrieve(tool, arguments.clone(), permit, &diagnostic) => result,
    };
    use crate::web_retrieval_generated::ErrorCode;
    use kimi_web::RetrievalFailure;
    let output = match result {
        Ok(result) => result
            .structured_content
            .unwrap_or_else(|| error("invalid_response")),
        Err(failure) => error(match failure {
            RetrievalFailure::UrlBlocked => "url_blocked",
            RetrievalFailure::Tool(code) => match code {
                ErrorCode::InvalidArguments => "invalid_arguments",
                ErrorCode::Timeout => "timeout",
                ErrorCode::Cancelled => "cancelled",
                ErrorCode::NoContent => "no_content",
                ErrorCode::ResponseTooLarge => "response_too_large",
                ErrorCode::InvalidResponse => "invalid_response",
                // Provider authentication/permissions do not describe the product session.
                ErrorCode::AuthenticationFailed
                | ErrorCode::PermissionDenied
                | ErrorCode::RateLimited
                | ErrorCode::Unavailable => "upstream_unavailable",
            },
        }),
    };
    if !validate("ToolOutput", &output) {
        diagnostic.validation_issue("web_contract", "result");
        return error("invalid_response");
    }
    output
}

pub fn run(with_facts: bool) -> io::Result<()> {
    let backend = kimi_web::Backend::load()
        .map_err(|_| io::Error::other("server web credential unavailable"))?;
    let cancelled = CancellationToken::new();
    let reader_cancel = cancelled.clone();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    // Exactly one reader owns stdin. Keeping it open retains the admitted call;
    // normal EOF is the cancellation protocol. Never signal or replace a worker.
    let reader = std::thread::spawn(move || {
        let mut input = io::stdin().lock();
        let mut line = Vec::new();
        let read = input.by_ref().take(32769).read_until(b'\n', &mut line);
        let value = if read.is_ok() && line.len() <= 32768 {
            serde_json::from_slice::<Value>(&line).ok()
        } else {
            None
        };
        let _ = sender.send(value);
        let mut byte = [0u8; 1];
        let _ = input.read(&mut byte); // EOF or any second frame closes admission.
        reader_cancel.cancel();
    });
    let value = receiver.recv().ok().flatten();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let diagnostic = Diagnostic::service_stderr();
    let mut output = match value {
        Some(value) => runtime.block_on(execute(value, backend, cancelled, diagnostic.clone())),
        None => error("invalid_arguments"),
    };
    if with_facts {
        let code = output.pointer("/error/code").and_then(Value::as_str);
        let outcome = if code.is_some() {
            "failed"
        } else if output["truncated"] == true || output["dropped_results"].as_u64().unwrap_or(0) > 0
        {
            "partial"
        } else if output["results"].as_array().is_some_and(Vec::is_empty) {
            "empty"
        } else {
            "results"
        };
        let mut facts = json!({"transport":diagnostic.web_transport(),"outcome":outcome});
        if let Some(code) = code {
            facts["error_code"] = json!(code);
        }
        output = json!({"facts":facts,"result":output});
    }
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &output)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    drop(stdout);
    // The owner closes stdin after receiving the result. This handshake keeps
    // shutdown ownership explicit, including when a provider request was cancelled.
    reader
        .join()
        .map_err(|_| io::Error::other("web reader unavailable"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn worker_request(tool: &str, arguments: Value) -> Value {
        let deadline = now_ms() + 45_000;
        let id = "0199aaaa-aaaa-7aaa-8aaa-aaaaaaaaaaaa";
        let digest = "a".repeat(64);
        json!({
            "execute": {
                "binding_id": "11111111-1111-4111-8111-111111111111",
                "request": {"operation_id": id, "tool": tool, "arguments": arguments},
                "parameter_digest": digest,
                "deadline_unix_ms": deadline
            },
            "receipt": {
                "operation_id": id,
                "claim_id": "22222222-2222-4222-8222-222222222222",
                "executor_instance": "33333333-3333-4333-8333-333333333333",
                "parameter_digest": digest,
                "deadline_unix_ms": deadline
            }
        })
    }

    #[tokio::test]
    async fn server_executor_reuses_the_existing_adapter_once() {
        let (backend, http) = kimi_web::tests::local_backend();
        let request = worker_request("kimi_web_search", json!({"query":"public documentation"}));
        let result = execute(
            request,
            backend,
            CancellationToken::new(),
            Diagnostic::service_stderr(),
        )
        .await;
        assert!(validate("ToolOutput", &result));
        assert_eq!(http.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn fetch_url_refusal_requires_the_provider_type_and_status() {
        use crate::web_retrieval_generated::{FETCH_TOOL_NAME, SEARCH_TOOL_NAME};
        let url = json!({"url":"https://example.org/article"});
        let policy_refusal = json!({"error":{"type":"security_risk","message":"provider detail"}});
        for (tool, args, status, body, expected) in [
            (
                FETCH_TOOL_NAME,
                url.clone(),
                403,
                policy_refusal.clone(),
                "url_blocked",
            ),
            (
                FETCH_TOOL_NAME,
                url.clone(),
                403,
                Value::Null,
                "upstream_unavailable",
            ),
            (
                FETCH_TOOL_NAME,
                url.clone(),
                403,
                json!({"error":{"type":"permission_denied"}}),
                "upstream_unavailable",
            ),
            (
                FETCH_TOOL_NAME,
                url.clone(),
                502,
                policy_refusal.clone(),
                "upstream_unavailable",
            ),
            (
                FETCH_TOOL_NAME,
                url,
                404,
                json!({"error":{"type":"markdown_not_found"}}),
                "no_content",
            ),
            (
                SEARCH_TOOL_NAME,
                json!({"query":"public documentation"}),
                403,
                policy_refusal,
                "upstream_unavailable",
            ),
        ] {
            let (backend, calls) = kimi_web::tests::provider_reply(status, body);
            let result = execute(
                worker_request(tool, args),
                backend,
                CancellationToken::new(),
                Diagnostic::service_stderr(),
            )
            .await;
            assert_eq!(result, error(expected));
            assert!(validate("ToolOutput", &result));
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
    }
}
