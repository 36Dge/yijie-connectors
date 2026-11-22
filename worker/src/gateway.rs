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
fn missing_binding() -> ErrorData {
    ErrorData::invalid_request(
        "市场工具调用缺少有效的租约、会话或轮次标识，外部请求未发送。请停止本次尝试。",
        None,
    )
}
fn terminal_lease_error(code: wire::ErrorCode) -> Option<(&'static str, &'static str)> {
    match code {
        wire::ErrorCode::LeaseExpired | wire::ErrorCode::ScopeExpired => Some((
            wire::GATEWAY_ERROR_LEASE_EXPIRED_CODE,
            wire::GATEWAY_ERROR_LEASE_EXPIRED_MESSAGE,
        )),
        wire::ErrorCode::LeaseRevoked | wire::ErrorCode::Stopping => Some((
            wire::GATEWAY_ERROR_LEASE_REVOKED_CODE,
            wire::GATEWAY_ERROR_LEASE_REVOKED_MESSAGE,
        )),
        _ => None,
    }
}
fn terminal_detail(code: &str) -> serde_json::Value {
    serde_json::json!({"code":code,"retryable":false,"retryScope":"market_turn","outcome":"not_sent"})
}
fn terminal_result(
    (reason, message): (&str, &str),
    receipt: Option<wire::GatewayCallReceipt>,
) -> CallToolResult {
    let mut result = CallToolResult::error(vec![Content::text(format!("[{reason}] {message}"))]);
    result.structured_content = Some(terminal_detail(reason));
    // Attach the trusted stop only after the normal supplier-meta scrub.
    let mut result = with_call_receipt(result, receipt);
    result.meta.get_or_insert_with(Default::default).0.insert(
        wire::GATEWAY_RESULT_TURN_STOP_KEY.to_owned(),
        serde_json::json!(reason),
    );
    result
}
fn registration_error(code: wire::ErrorCode) -> ErrorData {
    let message = match code {
        wire::ErrorCode::InvalidRequest => "市场工具参数未通过校验，请检查工具定义。",
        wire::ErrorCode::NotQualified => "当前连接器选集或工具未就绪，外部请求未发送。",
        wire::ErrorCode::TurnMismatch | wire::ErrorCode::NotFound | wire::ErrorCode::NotBound => {
            "当前市场调用没有匹配的轮次授权，外部请求未发送。请停止本轮市场工具调用。"
        }
        wire::ErrorCode::CapacityExceeded => {
            "本轮市场连接器调用容量已用尽，外部请求未发送。请停止本轮市场工具调用。"
        }
        _ => "本地市场网关无法登记本次调用，外部请求未发送。",
    };
    ErrorData::invalid_request(
        message,
        Some(serde_json::json!({"code":code,"outcome":"not_sent"})),
    )
}
// This classification is used only after consume_execution failed under the
// same Broker lock. Consumed or missing facts never prove non-execution.
fn admission_failure(state: Option<wire::CallState>) -> (&'static str, Event) {
    match state {
        Some(wire::CallState::Rejected) => {
            (wire::GATEWAY_ERROR_OWNER_REJECTED_MESSAGE, Event::Rejected)
        }
        Some(wire::CallState::Expired) => (
            wire::GATEWAY_ERROR_APPROVAL_EXPIRED_MESSAGE,
            Event::PolicyRejected,
        ),
        Some(wire::CallState::Cancelled | wire::CallState::Revoked) => (
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
        .ok_or_else(missing_binding)?;
    let path = parts.uri.path();
    let reference = path.strip_prefix("/mcp/").ok_or_else(missing_binding)?;
    if reference.contains('/')
        || parts.uri.query().is_some()
        || uuid::Uuid::parse_str(reference).is_err()
    {
        return Err(missing_binding());
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
            .visible_tools(&reference);
        Ok(ListToolsResult {
            tools: match tool {
                Ok(Some(tools)) => tools,
                Ok(None) => vec![lookup_tool()],
                Err(code) if terminal_lease_error(code).is_some() => Vec::new(),
                Err(code) => return Err(registration_error(code)),
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
            .ok_or_else(missing_binding)?;
        let turn = metadata
            .0
            .get("x-codex-turn-metadata")
            .and_then(|v| v.get("turn_id"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(missing_binding)?;
        let call = wait_for_bound_call(
            &self.broker,
            &context.ct,
            &reference,
            thread,
            turn,
            &request,
        )
        .await;
        let call = match call {
            Ok(Some(call)) => call,
            Ok(None) => return Ok(content_budget_expired()),
            Err(code) => {
                return match terminal_lease_error(code) {
                    Some(facts) => Ok(terminal_result(facts, None)),
                    None => Err(registration_error(code)),
                };
            }
        };
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
        let (admission, receipt) = {
            let mut state = self.broker.state.lock().expect("broker lock");
            let result = state.consume_execution(&call.identity.call_ref, accepted, &context.ct);
            let after = state.pending_call_view(&call.identity.call_ref);
            let receipt = state.result_receipt(&call.identity.call_ref);
            let admission = result.map_err(|code| {
                if matches!(
                    after,
                    Some(
                        wire::CallState::Expired
                            | wire::CallState::Revoked
                            | wire::CallState::Cancelled
                    )
                ) && let Some(facts) = terminal_lease_error(code)
                {
                    return (
                        terminal_result(facts, receipt.clone()),
                        Event::PolicyRejected,
                    );
                }
                let (message, event) = admission_failure(after);
                (
                    with_call_receipt(
                        CallToolResult::error(vec![Content::text(message)]),
                        receipt.clone(),
                    ),
                    event,
                )
            });
            (admission, receipt)
        };
        let execution = match admission {
            Ok(execution) => {
                diagnostic.emit(Stage::CallAdmission, Event::Consumed, None, None);
                execution
            }
            Err((error, event)) => {
                diagnostic.emit(Stage::CallAdmission, event, None, None);
                return Ok(error);
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
            broker::Execution::Kimi {
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
                Ok(backend.execute(&tool, arguments, permit, diagnostic).await)
            }
        };
        let result = match result {
            Ok(value) => value,
            Err(_) => CallToolResult::error(vec![Content::text(
                wire::GATEWAY_ERROR_EXECUTION_UNVERIFIED_MESSAGE,
            )]),
        };
        Ok(with_call_receipt(result, receipt))
    }
}

/// Only this owned Gateway writes correlation. Supplier metadata cannot become
/// a receipt. Native status/result remain authoritative and no-result paths do
/// not invent an Item. Code Mode may expose this non-secret metadata.
fn with_call_receipt(
    mut result: CallToolResult,
    receipt: Option<wire::GatewayCallReceipt>,
) -> CallToolResult {
    result.meta = None;
    if let Some(receipt) = receipt.filter(|value| value.validate().is_ok())
        && let Ok(bytes) = serde_json::to_vec(&receipt)
        && bytes.len() <= wire::GATEWAY_RESULT_MAX_METADATA_BYTES
        && let Ok(value) = serde_json::to_value(receipt)
    {
        result.meta = Some(Meta(serde_json::Map::from_iter([(
            wire::GATEWAY_RESULT_KEY.into(),
            value,
        )])));
    }
    if serde_json::to_vec(&result)
        .is_ok_and(|bytes| bytes.len() <= wire::GATEWAY_RESULT_MAX_RESULT_ENVELOPE_BYTES)
    {
        result
    } else {
        // Preserve the small receipt even if a future adapter exceeds its
        // declared output budget; never turn an unknown outcome into success.
        let mut bounded = CallToolResult::error(vec![Content::text(
            wire::GATEWAY_ERROR_EXECUTION_UNVERIFIED_MESSAGE,
        )]);
        bounded.meta = result.meta;
        bounded
    }
}

// A request may arrive before turn/start's response. Cancellation must end
// that wait, and only an owner bind_turn may permit registration.
fn content_budget_expired() -> CallToolResult {
    CallToolResult::error(vec![Content::text(
        crate::web_retrieval_generated::CONTENT_BUDGET_EXPIRED_MESSAGE,
    )])
}

pub(crate) async fn wait_for_bound_call(
    broker: &BrokerHandle,
    cancellation: &CancellationToken,
    reference: &str,
    thread: &str,
    turn: &str,
    request: &CallToolRequestParams,
) -> Result<Option<wire::PendingCall>, wire::ErrorCode> {
    let wait = async {
        loop {
            let changed = broker.changed.notified();
            let result = {
                let mut state = broker.state.lock().expect("broker lock");
                if cancellation.is_cancelled() {
                    return Err(wire::ErrorCode::ApprovalInvalid);
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
                    return Err(wire::ErrorCode::ApprovalInvalid);
                }
                result
            };
            match result {
                Ok(call) => return Ok(Some(call)),
                // Only the new content-mode registration can return this code;
                // it has established expiration before creating any approval.
                Err(wire::ErrorCode::CallExpired) => return Ok(None),
                Err(wire::ErrorCode::NotBound) => tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(wire::ErrorCode::ApprovalInvalid),
                    _ = changed => {},
                },
                Err(code) => return Err(code),
            }
        }
    };
    tokio::time::timeout(broker::UNBOUND_WAIT, wait)
        .await
        .map_err(|_| wire::ErrorCode::NotBound)?
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
    // Keep the dedicated qualification binary consistent with its existing
    // externalCallsEnabled=false policy. Share the implementation, not the
    // product-only Web route and owner control surface.
    let web = (matches!(mode, ProviderMode::Product) && !crate::local_acceptance::enabled())
        .then(crate::builtin_web_gateway::WebGateway::default);
    let service = StreamableHttpService::new(
        move || Ok(gateway.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_cancellation_token(shutdown.clone())
            .with_allowed_hosts([address.to_string()]),
    );
    let mut router = Router::new().route_service("/mcp/{capability_ref}", service);
    if let Some(web) = web.as_ref() {
        let web_factory = web.clone();
        let web_service = StreamableHttpService::new(
            move || Ok(web_factory.clone()),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default()
                .with_cancellation_token(shutdown.clone())
                .with_allowed_hosts([address.to_string()]),
        );
        router = router.route_service("/builtin-web", web_service);
    }
    let router = router.layer(middleware::from_fn_with_state(
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
        control_loop(&broker, &mut providers, web.as_ref())
    };
    // EOF, shutdown and malformed framing all close admission first.
    broker.stop();
    if let Some(web) = web.as_ref() {
        web.stop();
    }
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
    web: Option<&crate::builtin_web_gateway::WebGateway>,
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
            web.and_then(|web| web.control(&line))
                .or_else(|| providers.handle(&line))
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
    fn receipt() -> wire::GatewayCallReceipt {
        wire::GatewayCallReceipt {
            schema_version: 1,
            call_ref: uuid::Uuid::new_v4().to_string(),
            service_id: "sorftime".into(),
            reference: crate::generated::SelectionRef {
                installation_id: uuid::Uuid::new_v4().to_string(),
                revision: 1,
                generation: 1,
            },
            native_thread_id: "ordinary-thread".into(),
            native_turn_id: "ordinary-turn".into(),
            turn_operation_id: uuid::Uuid::new_v4().to_string(),
            selection_digest: "a".repeat(64),
            tool_name: "ordinary_lookup".into(),
            args_digest: "b".repeat(64),
            args_encoding: "worker-json-v1".into(),
            decision_receipt: Some(wire::GatewayDecisionReceipt {
                approval_id: uuid::Uuid::new_v4().to_string(),
                decision_id: uuid::Uuid::new_v4().to_string(),
                decision: wire::Decision::ApproveOnce,
            }),
            admission_consumed: true,
        }
    }
    #[test]
    fn result_receipt_preserves_native_outcome_and_only_owned_metadata() {
        let receipt = receipt();
        for mut original in [
            CallToolResult::structured(serde_json::json!({"rows":[1]})),
            CallToolResult::error(vec![Content::text("ordinary failure")]),
        ] {
            original.meta = Some(Meta(serde_json::Map::from_iter([(
                "supplier_page".into(),
                serde_json::json!(1),
            )])));
            let result = with_call_receipt(original.clone(), Some(receipt.clone()));
            assert_eq!(result.is_error, original.is_error);
            assert_eq!(result.content, original.content);
            assert_eq!(result.structured_content, original.structured_content);
            let meta = result.meta.unwrap();
            assert_eq!(meta.0.len(), 1);
            assert_eq!(
                meta.0[wire::GATEWAY_RESULT_KEY],
                serde_json::to_value(&receipt).unwrap()
            );
        }
    }
    #[test]
    fn result_receipt_denied_and_expired_calls_do_not_claim_execution() {
        let mut receipt = receipt();
        receipt.admission_consumed = false;
        receipt.decision_receipt.as_mut().unwrap().decision = wire::Decision::Reject;
        for value in [
            Some(receipt.clone()),
            Some(wire::GatewayCallReceipt {
                decision_receipt: None,
                ..receipt
            }),
            None,
        ] {
            let present = value.is_some();
            let result = with_call_receipt(
                CallToolResult::error(vec![Content::text("not admitted")]),
                value,
            );
            assert_eq!(result.is_error, Some(true));
            assert_eq!(result.meta.is_some(), present);
            if let Some(meta) = result.meta {
                assert_eq!(meta.0[wire::GATEWAY_RESULT_KEY]["admissionConsumed"], false);
            }
        }
    }
    #[test]
    fn result_receipt_budget_retains_link_without_claiming_large_output_success() {
        let receipt = receipt();
        let result = with_call_receipt(
            CallToolResult::success(vec![Content::text(
                "x".repeat(wire::GATEWAY_RESULT_MAX_RESULT_ENVELOPE_BYTES),
            )]),
            Some(receipt),
        );
        assert_eq!(result.is_error, Some(true));
        assert!(result.meta.is_some());
        assert!(
            serde_json::to_vec(&result).unwrap().len()
                < wire::GATEWAY_RESULT_MAX_RESULT_ENVELOPE_BYTES
        );
    }
    #[test]
    fn kimi_expired_budget_is_an_explicit_tool_error() {
        let result = content_budget_expired();
        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result.content[0].as_text().unwrap().text,
            crate::web_retrieval_generated::CONTENT_BUDGET_EXPIRED_MESSAGE
        );
    }
    #[test]
    fn admission_outcomes_do_not_confuse_rejection_with_unknown_execution() {
        assert_eq!(
            admission_failure(Some(wire::CallState::Rejected)).0,
            wire::GATEWAY_ERROR_OWNER_REJECTED_MESSAGE
        );
        for state in [wire::CallState::Cancelled, wire::CallState::Revoked] {
            assert_eq!(
                admission_failure(Some(state)).0,
                wire::GATEWAY_ERROR_ADMISSION_STOPPED_MESSAGE
            );
        }
        assert_eq!(
            admission_failure(Some(wire::CallState::Expired)).0,
            wire::GATEWAY_ERROR_APPROVAL_EXPIRED_MESSAGE
        );
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
    #[test]
    fn terminal_market_scope_is_visible_without_a_fabricated_call_receipt() {
        for code in [wire::ErrorCode::LeaseExpired, wire::ErrorCode::LeaseRevoked] {
            let facts = terminal_lease_error(code).unwrap();
            let result = terminal_result(facts, None);
            assert_eq!(result.is_error, Some(true));
            assert_eq!(result.structured_content.unwrap()["retryable"], false);
            let metadata = result.meta.unwrap();
            assert_eq!(metadata.0.len(), 1);
            assert_eq!(metadata.0[wire::GATEWAY_RESULT_TURN_STOP_KEY], facts.0);
            assert!(
                result.content[0]
                    .as_text()
                    .unwrap()
                    .text
                    .contains("停止调用所有市场工具")
            );
        }
        for code in [
            wire::ErrorCode::CallExpired,
            wire::ErrorCode::ApprovalInvalid,
            wire::ErrorCode::InvalidRequest,
        ] {
            assert!(terminal_lease_error(code).is_none());
        }
    }
}
