//! Official rmcp streamable HTTP server. Owner control and MCP data traffic
//! have separate transports; neither HTTP metadata nor the bearer creates a grant.
use crate::broker::{self, BrokerHandle, ProviderMode};
use crate::broker_generated as wire;
use crate::tushare_oauth::{Diagnostic, Event, Stage};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use rmcp::{
    RoleServer, ServerHandler,
    model::*,
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use std::io::{self, BufRead, Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct Auth(Arc<String>);
async fn authorize(State(auth): State<Auth>, mut request: Request<Body>, next: Next) -> Response {
    // No browser origin is needed for the exclusively owned Runtime. Never
    // echo an auth header, original request, query, or library error.
    let authorized = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|value| value.strip_prefix("Bearer ") == Some(auth.0.as_str()));
    if !authorized || request.headers().contains_key("origin") {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let body = std::mem::replace(request.body_mut(), Body::empty());
    let bytes = match to_bytes(body, wire::MAX_FRAME_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    *request.body_mut() = Body::from(bytes);
    next.run(request).await
}
#[derive(Clone)]
struct Gateway {
    broker: BrokerHandle,
}
fn mcp_error() -> ErrorData {
    ErrorData::invalid_request("Market connector admission unavailable", None)
}
// This classification is used only after consume_execution failed under the
// same Broker lock. Consumed or missing facts never prove non-execution.
fn admission_failure(state: Option<wire::CallState>) -> (&'static str, Event) {
    match state {
        Some(wire::CallState::Rejected) => {
            (wire::GATEWAY_ERROR_OWNER_REJECTED_MESSAGE, Event::Rejected)
        }
        Some(wire::CallState::Cancelled | wire::CallState::Revoked | wire::CallState::Expired) => (
            wire::GATEWAY_ERROR_ADMISSION_STOPPED_MESSAGE,
            Event::PolicyRejected,
        ),
        _ => (
            wire::GATEWAY_ERROR_EXECUTION_UNVERIFIED_MESSAGE,
            Event::Unverified,
        ),
    }
}
fn lease_ref(context: &RequestContext<RoleServer>) -> Result<String, ErrorData> {
    let parts = context
        .extensions
        .get::<axum::http::request::Parts>()
        .ok_or_else(mcp_error)?;
    let path = parts.uri.path();
    let reference = path.strip_prefix("/mcp/").ok_or_else(mcp_error)?;
    if reference.contains('/')
        || parts.uri.query().is_some()
        || uuid::Uuid::parse_str(reference).is_err()
    {
        return Err(mcp_error());
    }
    Ok(reference.into())
}
fn lookup_tool() -> Tool {
    let schema = serde_json::json!({"type":"object","properties":{"query":{"type":"string","minLength":1,"maxLength":256}},"required":["query"],"additionalProperties":false});
    Tool::new(
        "lookup",
        "Read a public in-process synthetic value for local qualification.",
        schema.as_object().expect("fixed schema").clone(),
    )
    // These facts apply only to this fixed in-process synthetic lookup.
    // Product mode has no qualified tool and never advertises this schema.
    .with_annotations(
        ToolAnnotations::new()
            .read_only(true)
            .destructive(false)
            .idempotent(true)
            .open_world(false),
    )
}
impl ServerHandler for Gateway {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("yijie-market-gateway", "0.1.0"))
    }
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let reference = lease_ref(&context)?;
        let tool = self
            .broker
            .state
            .lock()
            .expect("broker lock")
            .visible_tools(&reference)
            .map_err(|_| mcp_error())?;
        Ok(ListToolsResult {
            tools: match tool {
                Some(tools) => tools,
                None => vec![lookup_tool()],
            },
            ..Default::default()
        })
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let reference = lease_ref(&context)?;
        // rmcp 1.8 service dispatch moves params._meta into RequestContext
        // (service.rs:1168); reading params.meta here would discard real facts.
        let metadata = &context.meta;
        let thread = metadata
            .0
            .get("threadId")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(mcp_error)?;
        let turn = metadata
            .0
            .get("x-codex-turn-metadata")
            .and_then(|v| v.get("turn_id"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(mcp_error)?;
        let call = wait_for_bound_call(
            &self.broker,
            &context.ct,
            &reference,
            thread,
            turn,
            &request,
        )
        .await?;
        let diagnostic = Diagnostic::for_operation(&call.identity.call_ref);
        diagnostic.emit(Stage::CallApproval, Event::Started, None, None);
        let metadata = wire::ElicitationMetadata {
            yijie_market_call_ref: call.identity.call_ref.clone(),
            yijie_kind: "gateway_call_approval".into(),
        };
        let params = CreateElicitationRequestParams::FormElicitationParams {
            meta: Some(Meta(
                serde_json::to_value(metadata)
                    .expect("generated metadata")
                    .as_object()
                    .expect("metadata object")
                    .clone(),
            )),
            message: call.review.summary.clone(),
            requested_schema: serde_json::from_value(
                serde_json::json!({"type":"object","properties":{}}),
            )
            .expect("fixed elicitation schema"),
        };
        let callback = context.peer.create_elicitation(params);
        tokio::pin!(callback);
        let lease_end = tokio::time::sleep(Duration::from_millis(call.remaining_ttl_ms as u64));
        tokio::pin!(lease_end);
        let accepted = loop {
            let changed = self.broker.changed.notified();
            let current = self
                .broker
                .state
                .lock()
                .expect("broker lock")
                .pending_call_view(&call.identity.call_ref);
            if current.is_none_or(|state| {
                matches!(
                    state,
                    wire::CallState::Revoked
                        | wire::CallState::Expired
                        | wire::CallState::Cancelled
                        | wire::CallState::Rejected
                )
            }) {
                break false;
            }
            tokio::select! {
                biased;
                _=context.ct.cancelled()=>break false,
                _=&mut lease_end=>break false,
                response=&mut callback=>{
                    match response {
                        Ok(value) => {break value.action == ElicitationAction::Accept;},
                        Err(_) => break false,
                    }
                },
                _=changed=>{},
            }
        };
        let admission = {
            let mut state = self.broker.state.lock().expect("broker lock");
            let result = state.consume_execution(&call.identity.call_ref, accepted, &context.ct);
            let after = state.pending_call_view(&call.identity.call_ref);
            result.map_err(|_| admission_failure(after))
        };
        let execution = match admission {
            Ok(execution) => {
                diagnostic.emit(Stage::CallAdmission, Event::Consumed, None, None);
                execution
            }
            Err((message, event)) => {
                diagnostic.emit(Stage::CallAdmission, event, None, None);
                return Ok(CallToolResult::error(vec![Content::text(message)]));
            }
        };
        let result = match execution {
            broker::Execution::Synthetic(value) => Ok(CallToolResult::structured(value)),
            broker::Execution::Daily {
                backend,
                arguments,
                lease,
                deadline,
            } => backend
                .execute(arguments, context.ct.clone(), lease, deadline, diagnostic)
                .await
                .map(CallToolResult::structured),
            broker::Execution::Generic {
                backend,
                tool,
                arguments,
                lease,
                deadline,
            } => {
                let permit = crate::daily::Permit {
                    tool: String::new(),
                    arguments: serde_json::Value::Null,
                    request: context.ct.clone(),
                    lease,
                    deadline,
                    backend: CancellationToken::new(),
                };
                backend.execute(&tool, arguments, permit, diagnostic).await
            }
        };
        match result {
            Ok(value) => Ok(value),
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                wire::GATEWAY_ERROR_EXECUTION_UNVERIFIED_MESSAGE,
            )])),
        }
    }
}

// A request may arrive before turn/start's response. Cancellation must end
// that wait, and only an owner bind_turn may permit registration.
pub(crate) async fn wait_for_bound_call(
    broker: &BrokerHandle,
    cancellation: &CancellationToken,
    reference: &str,
    thread: &str,
    turn: &str,
    request: &CallToolRequestParams,
) -> Result<wire::PendingCall, ErrorData> {
    let wait = async {
        loop {
            let changed = broker.changed.notified();
            let result = {
                let mut state = broker.state.lock().expect("broker lock");
                if cancellation.is_cancelled() {
                    return Err(mcp_error());
                }
                let result = state.register_call(
                    reference,
                    thread,
                    turn,
                    &request.name,
                    request.arguments.clone(),
                );
                if let Ok(call) = &result
                    && cancellation.is_cancelled()
                {
                    let _ = state.consume_execution(&call.identity.call_ref, false, cancellation);
                    return Err(mcp_error());
                }
                result
            };
            match result {
                Ok(call) => return Ok(call),
                Err(wire::ErrorCode::NotBound) => tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(mcp_error()),
                    _ = changed => {},
                },
                Err(_) => return Err(mcp_error()),
            }
        }
    };
    tokio::time::timeout(broker::UNBOUND_WAIT, wait)
        .await
        .map_err(|_| mcp_error())?
}

/// Runs only when the binary entrypoint explicitly selects Broker mode. The
/// qualification binary supplies its mode at compile-time, never via an env flag.
pub fn run(mode: ProviderMode) -> io::Result<()> {
    let capability = std::env::var(broker::CAPABILITY_ENV)
        .map_err(|_| io::Error::other("internal data capability unavailable"))?;
    if capability.len() != 64
        || !capability
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(io::Error::other("internal data capability unavailable"));
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    let listener = runtime.block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))?;
    let address = listener.local_addr()?;
    let broker = BrokerHandle::new(mode, format!("http://127.0.0.1:{}/mcp", address.port()));
    let shutdown = CancellationToken::new();
    let gateway = Gateway {
        broker: broker.clone(),
    };
    let service = StreamableHttpService::new(
        move || Ok(gateway.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_cancellation_token(shutdown.clone())
            .with_allowed_hosts([address.to_string()]),
    );
    let router = Router::new()
        .route_service("/mcp/{capability_ref}", service)
        .layer(middleware::from_fn_with_state(
            Auth(Arc::new(capability)),
            authorize,
        ));
    let stopped = shutdown.clone();
    let server = runtime.spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(stopped.cancelled_owned())
            .await
    });
    let mut providers =
        crate::providers::Providers::new(broker.clone(), matches!(mode, ProviderMode::Product));
    let outcome = {
        let _runtime_context = runtime.enter();
        control_loop(&broker, &mut providers)
    };
    // EOF, shutdown and malformed framing all close admission first.
    broker.stop();
    shutdown.cancel();
    // Host has its own bounded wait and retains STOP_PENDING. This process
    // retains all join owners after EOF instead of exiting and losing a live
    // callback fact when a graceful operation has not yet finished.
    runtime.block_on(async {
        while providers.close().await.is_err() {
            tokio::task::yield_now().await;
        }
    });
    match runtime.block_on(server) {
        Ok(Ok(())) => outcome,
        _ => Err(io::Error::other("Gateway normal cleanup incomplete")),
    }
}
fn control_loop(
    broker: &BrokerHandle,
    providers: &mut crate::providers::Providers,
) -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let count = input
            .by_ref()
            .take(wire::MAX_FRAME_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)?;
        if count == 0 {
            return Ok(());
        }
        let too_large = line.len() > wire::MAX_FRAME_BYTES;
        let response = if too_large {
            broker::safe_error(None, wire::ErrorCode::InvalidRequest)
        } else {
            providers
                .handle(&line)
                .or_else(|| providers.broker_initialize_error(&line))
                .unwrap_or_else(|| broker.handle(&line))
        };
        serde_json::to_writer(&mut output, &response)?;
        output.write_all(b"\n")?;
        output.flush()?;
        if too_large || broker.state.lock().expect("broker lock").is_stopped() {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod outcome_tests {
    use super::*;
    #[test]
    fn admission_outcomes_do_not_confuse_rejection_with_unknown_execution() {
        assert_eq!(
            admission_failure(Some(wire::CallState::Rejected)).0,
            wire::GATEWAY_ERROR_OWNER_REJECTED_MESSAGE
        );
        for state in [
            wire::CallState::Cancelled,
            wire::CallState::Expired,
            wire::CallState::Revoked,
        ] {
            assert_eq!(
                admission_failure(Some(state)).0,
                wire::GATEWAY_ERROR_ADMISSION_STOPPED_MESSAGE
            );
        }
        for state in [
            None,
            Some(wire::CallState::Consumed),
            Some(wire::CallState::Pending),
            Some(wire::CallState::Approved),
        ] {
            assert_eq!(
                admission_failure(state).0,
                wire::GATEWAY_ERROR_EXECUTION_UNVERIFIED_MESSAGE
            );
        }
    }
}
