//! The reviewed Tushare daily adapter. It never trusts provider descriptions or
//! annotations, and never forwards supplier text, links or arbitrary metadata.
use crate::{
    provider_generated as wire,
    tushare_oauth::{Diagnostic, Event, Failure, Stage},
};
use codex_rmcp_client::RmcpClient;
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

pub(crate) struct Permit {
    pub tool: String,
    pub arguments: Value,
    pub request: CancellationToken,
    pub lease: CancellationToken,
    pub backend: CancellationToken,
    pub deadline: Instant,
}
#[derive(Default)]
struct PermitSlot {
    permit: Option<Permit>,
    diagnostic: Diagnostic,
}
#[derive(Default)]
pub struct HttpPermit(Mutex<PermitSlot>, pub crate::secret_guard::SecretGuard);
impl HttpPermit {
    pub fn diagnostic(&self) -> Diagnostic {
        self.0.lock().expect("daily permit").diagnostic.clone()
    }
    pub(crate) fn arm(&self, permit: Permit, diagnostic: Diagnostic) {
        *self.0.lock().expect("daily permit") = PermitSlot {
            permit: Some(permit),
            diagnostic,
        };
    }
    pub(crate) fn clear(&self) {
        self.0.lock().expect("daily permit").permit = None;
    }
    // Called at the actual HTTP boundary. Taking the permit before sending
    // forbids an SDK reconnect/401 retry from resending the business request.
    pub fn take(&self, message: &Value) -> bool {
        if message.get("method").and_then(Value::as_str) != Some("tools/call") {
            return false;
        }
        let mut slot = self.0.lock().expect("daily permit");
        let Some(permit) = slot.permit.as_ref() else {
            slot.diagnostic
                .emit(Stage::DailyHttpPermit, Event::PolicyRejected, None, None);
            return false;
        };
        if permit.request.is_cancelled()
            || permit.lease.is_cancelled()
            || permit.backend.is_cancelled()
            || Instant::now() >= permit.deadline
            || message.pointer("/params/name").and_then(Value::as_str) != Some(permit.tool.as_str())
            || message.pointer("/params/arguments") != Some(&permit.arguments)
        {
            slot.diagnostic
                .emit(Stage::DailyHttpPermit, Event::PolicyRejected, None, None);
            return false;
        }
        slot.permit = None;
        slot.diagnostic
            .emit(Stage::DailyHttpPermit, Event::Consumed, None, None);
        true
    }
}
pub struct Backend {
    client: RmcpClient,
    permit: Arc<HttpPermit>,
    serial: tokio::sync::Mutex<()>,
    cancelled: CancellationToken,
}
impl Backend {
    pub fn new(client: RmcpClient, permit: Arc<HttpPermit>) -> Arc<Self> {
        Arc::new(Self {
            client,
            permit,
            serial: tokio::sync::Mutex::new(()),
            cancelled: CancellationToken::new(),
        })
    }
    pub fn revoke(&self) {
        self.cancelled.cancel();
    }
    pub fn available(&self) -> bool {
        !self.cancelled.is_cancelled()
    }
    pub async fn close(&self) {
        self.revoke();
        // A pending call owns its refresh/persistence lifecycle. Await that
        // normal completion before shutdown and any later Keyring deletion.
        let _serial = self.serial.lock().await;
        self.client.shutdown().await;
    }
    pub async fn execute(
        &self,
        arguments: wire::TushareDailyArguments,
        request: CancellationToken,
        lease: CancellationToken,
        deadline: Instant,
        diagnostic: Diagnostic,
    ) -> Result<Value, Failure> {
        diagnostic.emit(Stage::DailyExecute, Event::Started, None, None);
        arguments.validate().map_err(|_| Failure::InvalidMetadata)?;
        let _serial = self.serial.lock().await;
        if !self.available()
            || request.is_cancelled()
            || lease.is_cancelled()
            || Instant::now() >= deadline
        {
            diagnostic.emit(Stage::DailyExecute, Event::Cancelled, None, None);
            return Err(Failure::Cancelled);
        }
        let args = serde_json::to_value(&arguments).map_err(|_| Failure::InvalidMetadata)?;
        self.permit.arm(
            Permit {
                tool: wire::TUSHARE_DAILY_UPSTREAM_TOOL_NAME.into(),
                arguments: args.clone(),
                request: request.clone(),
                lease: lease.clone(),
                backend: self.cancelled.clone(),
                deadline,
            },
            diagnostic.clone(),
        );
        // Do not drop this future on cancellation: it owns the fixed client's
        // refresh/writeback. The HTTP gate checks cancellation before send;
        // after send the bounded original result is awaited, never retried.
        let result = self
            .client
            .call_tool(
                wire::TUSHARE_DAILY_UPSTREAM_TOOL_NAME.into(),
                Some(args),
                None,
                Some(Duration::from_secs(15)),
            )
            .await;
        self.permit.clear();
        diagnostic.emit(
            Stage::DailyClientResult,
            if result.is_ok() {
                Event::Succeeded
            } else {
                Event::TransportUnavailable
            },
            None,
            None,
        );
        if !self.available()
            || request.is_cancelled()
            || lease.is_cancelled()
            || Instant::now() >= deadline
        {
            diagnostic.emit(Stage::DailyExecute, Event::Cancelled, None, None);
            return Err(Failure::Cancelled);
        }
        let result = result.map_err(|_| Failure::Unavailable)?;
        let normalized = sanitize_result_with_diagnostic(&result, &arguments, &diagnostic);
        diagnostic.emit(
            Stage::DailyNormalize,
            if normalized.is_ok() {
                Event::Succeeded
            } else {
                Event::Failed
            },
            None,
            normalized.as_ref().err().copied(),
        );
        normalized
    }
}

pub fn descriptor() -> Tool {
    let schema: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(crate::provider_generated::TUSHARE_DAILY_INPUT_SCHEMA_JSON)
            .expect("generated daily schema");
    let output: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(crate::provider_generated::TUSHARE_DAILY_OUTPUT_SCHEMA_JSON)
            .expect("generated daily output schema");
    let mut tool=Tool::new(
        crate::provider_generated::TUSHARE_DAILY_TOOL_NAME,
        "查询单个 A 股代码、单个交易日的公开未复权日线行情，最多一行。vol 单位为手，amount 单位为千元。每次执行需要单独批准。",
        schema,
    )
    .with_annotations(
        ToolAnnotations::new()
            .read_only(true)
            .destructive(false)
            .idempotent(true)
            .open_world(true),
    );
    tool.output_schema = Some(Arc::new(output));
    tool
}
pub fn qualified(tools: &[rmcp::model::Tool]) -> bool {
    let mut daily = tools
        .iter()
        .filter(|tool| tool.name == wire::TUSHARE_DAILY_UPSTREAM_TOOL_NAME);
    let Some(tool) = daily.next() else {
        return false;
    };
    if daily.next().is_some() {
        return false;
    }
    let Ok(bytes) = serde_json::to_vec(&tool.input_schema) else {
        return false;
    };
    format!("{:x}", Sha256::digest(bytes)) == wire::TUSHARE_DAILY_UPSTREAM_INPUT_SCHEMA_SHA256
        && reviewed_input_shape(&Value::Object((*tool.input_schema).clone()))
}
fn reviewed_input_shape(schema: &Value) -> bool {
    let Some(root) = schema.as_object() else {
        return false;
    };
    if root.get("type").and_then(Value::as_str) != Some("object")
        || root.keys().any(|key| {
            !matches!(
                key.as_str(),
                "type"
                    | "properties"
                    | "required"
                    | "additionalProperties"
                    | "description"
                    | "title"
                    | "$schema"
            )
        })
    {
        return false;
    }
    if root
        .get("additionalProperties")
        .is_some_and(|value| !value.is_boolean())
    {
        return false;
    }
    if root.get("required").is_some_and(|value| {
        value.as_array().is_none_or(|items| {
            items
                .iter()
                .any(|v| !matches!(v.as_str(), Some("ts_code" | "trade_date")))
        })
    }) {
        return false;
    }
    let Some(properties) = root.get("properties").and_then(Value::as_object) else {
        return false;
    };
    ["ts_code", "trade_date"].iter().all(|name| {
        properties
            .get(*name)
            .is_some_and(|property| string_branch(property, 0) == Some(true))
    })
}
// Validate every union branch, including ones not selected for forwarding.
// Annotation text is ignored; every unreviewed constraint rejects qualification.
fn string_branch(schema: &Value, depth: usize) -> Option<bool> {
    if depth > 8 {
        return None;
    }
    let root = schema.as_object()?;
    if root.keys().any(|key| {
        !matches!(
            key.as_str(),
            "type" | "description" | "title" | "default" | "examples" | "anyOf" | "oneOf"
        )
    }) {
        return None;
    }
    if let Some(kind) = root.get("type") {
        if root.contains_key("anyOf") || root.contains_key("oneOf") {
            return None;
        }
        return match kind.as_str() {
            Some("string") => Some(true),
            Some("integer") => Some(false),
            _ => None,
        };
    }
    let (branches, exclusive) = match (root.get("anyOf"), root.get("oneOf")) {
        (Some(v), None) => (v, false),
        (None, Some(v)) => (v, true),
        _ => return None,
    };
    let branches = branches.as_array()?;
    if branches.is_empty() || branches.len() > 8 {
        return None;
    }
    let accepted = branches
        .iter()
        .map(|branch| string_branch(branch, depth + 1))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .filter(|v| *v)
        .count();
    Some(if exclusive {
        accepted == 1
    } else {
        accepted > 0
    })
}

fn rows(value: &Value, diagnostic: &Diagnostic) -> Result<Vec<Map<String, Value>>, Failure> {
    // Accept ordinary structured MCP records or Tushare's fields/items table.
    // Recognized wrappers are data-only; no supplier messages are projected.
    if let Some(values) = value.as_array() {
        diagnostic.emit(Stage::DailyResultRows, Event::Records, None, None);
        if values.len() > wire::TUSHARE_DAILY_MAX_ROWS {
            diagnostic.emit(Stage::DailyResultRows, Event::ResponseLimit, None, None);
            return Err(Failure::InvalidMetadata);
        }
        return values
            .iter()
            .map(|v| v.as_object().cloned().ok_or(Failure::InvalidMetadata))
            .collect();
    }
    let obj = value.as_object().ok_or_else(|| {
        diagnostic.emit(Stage::DailyResultRows, Event::UnknownShape, None, None);
        Failure::InvalidMetadata
    })?;
    if let Some(code) = obj.get("code")
        && code.as_i64() != Some(0)
    {
        diagnostic.emit(Stage::DailyResultRows, Event::ProviderError, None, None);
        return Err(Failure::Unavailable);
    }
    if let Some(data) = obj.get("data") {
        diagnostic.emit(Stage::DailyResultRows, Event::DataWrapper, None, None);
        return rows(data, diagnostic);
    }
    if let (Some(fields), Some(items)) = (
        obj.get("fields").and_then(Value::as_array),
        obj.get("items").and_then(Value::as_array),
    ) {
        diagnostic.emit(Stage::DailyResultRows, Event::FieldsItems, None, None);
        if fields.len() > 64 || items.len() > wire::TUSHARE_DAILY_MAX_ROWS {
            return Err(Failure::InvalidMetadata);
        }
        let names: Vec<_> = fields
            .iter()
            .map(|v| v.as_str().ok_or(Failure::InvalidMetadata))
            .collect::<Result<_, _>>()?;
        let mut unique = std::collections::HashSet::new();
        if names.iter().any(|name| !unique.insert(*name)) {
            return Err(Failure::InvalidMetadata);
        }
        return items
            .iter()
            .map(|row| {
                let row = row
                    .as_array()
                    .filter(|v| v.len() == names.len())
                    .ok_or(Failure::InvalidMetadata)?;
                Ok(names
                    .iter()
                    .zip(row)
                    .map(|(name, value)| ((*name).into(), value.clone()))
                    .collect())
            })
            .collect();
    }
    if obj.contains_key("ts_code") && obj.contains_key("trade_date") {
        diagnostic.emit(Stage::DailyResultRows, Event::SingleRecord, None, None);
        return Ok(vec![obj.clone()]);
    }
    diagnostic.emit(Stage::DailyResultRows, Event::UnknownShape, None, None);
    Err(Failure::InvalidMetadata)
}
#[cfg(test)]
pub fn sanitize_result(
    result: &rmcp::model::CallToolResult,
    arguments: &wire::TushareDailyArguments,
) -> Result<Value, Failure> {
    sanitize_result_with_diagnostic(result, arguments, &Diagnostic::default())
}
fn sanitize_result_with_diagnostic(
    result: &rmcp::model::CallToolResult,
    arguments: &wire::TushareDailyArguments,
    diagnostic: &Diagnostic,
) -> Result<Value, Failure> {
    if result.is_error == Some(true) {
        diagnostic.emit(Stage::DailyResultEnvelope, Event::ProviderError, None, None);
        return Err(Failure::Unavailable);
    }
    let value = if let Some(value) = &result.structured_content {
        diagnostic.emit(Stage::DailyResultEnvelope, Event::Structured, None, None);
        value.clone()
    } else {
        if result.content.len() != 1 {
            diagnostic.emit(
                Stage::DailyResultEnvelope,
                Event::ContentUnsupported,
                None,
                None,
            );
            return Err(Failure::InvalidMetadata);
        }
        let text = result.content[0].as_text().ok_or_else(|| {
            diagnostic.emit(
                Stage::DailyResultEnvelope,
                Event::ContentUnsupported,
                None,
                None,
            );
            Failure::InvalidMetadata
        })?;
        if text.text.len() > 64 * 1024 {
            diagnostic.emit(Stage::DailyResultEnvelope, Event::ResponseLimit, None, None);
            return Err(Failure::InvalidMetadata);
        }
        let value = serde_json::from_str(&text.text).map_err(|_| {
            diagnostic.emit(Stage::DailyResultEnvelope, Event::TextInvalid, None, None);
            Failure::InvalidMetadata
        })?;
        diagnostic.emit(Stage::DailyResultEnvelope, Event::TextJson, None, None);
        value
    };
    if serde_json::to_vec(&value)
        .map_err(|_| Failure::InvalidMetadata)?
        .len()
        > 64 * 1024
    {
        diagnostic.emit(Stage::DailyResultEnvelope, Event::ResponseLimit, None, None);
        return Err(Failure::InvalidMetadata);
    }
    let mut projected = Vec::new();
    for row in rows(&value, diagnostic)? {
        if row.get("ts_code").and_then(Value::as_str) != Some(arguments.ts_code.as_str())
            || row.get("trade_date").and_then(Value::as_str) != Some(arguments.trade_date.as_str())
        {
            diagnostic.emit(Stage::DailyResultIdentity, Event::Failed, None, None);
            return Err(Failure::InvalidMetadata);
        }
        let mut output = Map::new();
        for name in wire::TUSHARE_DAILY_RESULT_IDENTITY_FIELDS {
            output.insert(name.into(), row[name].clone());
        }
        for name in wire::TUSHARE_DAILY_RESULT_NUMERIC_FIELDS {
            if let Some(value) = row.get(name) {
                if !value.as_f64().is_some_and(f64::is_finite) {
                    diagnostic.emit(Stage::DailyResultNumeric, Event::Failed, None, None);
                    return Err(Failure::InvalidMetadata);
                }
                output.insert(name.into(), value.clone());
            }
        }
        if wire::TUSHARE_DAILY_RESULT_REQUIRED_NUMERIC_FIELDS
            .iter()
            .any(|name| !output.contains_key(*name))
        {
            diagnostic.emit(Stage::DailyResultNumeric, Event::Failed, None, None);
            return Err(Failure::InvalidMetadata);
        }
        projected.push(Value::Object(output));
    }
    let mut result = Map::new();
    result.insert(
        wire::TUSHARE_DAILY_RESULT_ROWS_FIELD.into(),
        Value::Array(projected),
    );
    Ok(Value::Object(result))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use rmcp::{ServerHandler, ServiceExt};
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Clone)]
    struct LocalDaily(Arc<AtomicUsize>);
    impl ServerHandler for LocalDaily {
        fn get_info(&self) -> rmcp::model::ServerInfo {
            rmcp::model::ServerInfo::new(
                rmcp::model::ServerCapabilities::builder()
                    .enable_tools()
                    .build(),
            )
        }
        async fn call_tool(
            &self,
            request: rmcp::model::CallToolRequestParams,
            _: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
            assert_eq!(request.name, "daily");
            let args = request.arguments.unwrap();
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(rmcp::model::CallToolResult::structured(
                serde_json::json!([{"ts_code":args["ts_code"],"trade_date":args["trade_date"],"close":10.5}]),
            ))
        }
    }
    pub struct LocalFactory {
        calls: Arc<AtomicUsize>,
        tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    }
    impl codex_rmcp_client::InProcessTransportFactory for LocalFactory {
        fn open(
            &self,
        ) -> futures::future::BoxFuture<'static, std::io::Result<tokio::io::DuplexStream>> {
            let (client, server) = tokio::io::duplex(65536);
            let handler = LocalDaily(self.calls.clone());
            let task = tokio::spawn(async move {
                let service = handler.serve(server).await.unwrap();
                let _ = service.waiting().await;
            });
            self.tasks.lock().unwrap().push(task);
            Box::pin(async move { Ok(client) })
        }
    }
    pub async fn local_backend() -> (Arc<Backend>, Arc<AtomicUsize>, Arc<LocalFactory>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let factory = Arc::new(LocalFactory {
            calls: calls.clone(),
            tasks: Default::default(),
        });
        let client = RmcpClient::new_in_process_client(factory.clone())
            .await
            .unwrap();
        client
            .initialize(
                rmcp::model::InitializeRequestParams::new(
                    rmcp::model::ClientCapabilities::default(),
                    rmcp::model::Implementation::new("ordinary-local-daily", "1"),
                ),
                Some(Duration::from_secs(2)),
                Box::new(|_, _| {
                    Box::pin(async {
                        Ok(codex_rmcp_client::ElicitationResponse {
                            action: codex_rmcp_client::ElicitationAction::Decline,
                            content: None,
                            meta: None,
                        })
                    })
                }),
            )
            .await
            .unwrap();
        (
            Backend::new(client, Arc::new(HttpPermit::default())),
            calls,
            factory,
        )
    }
    impl LocalFactory {
        pub async fn join(&self) {
            let tasks = std::mem::take(&mut *self.tasks.lock().unwrap());
            for mut task in tasks {
                tokio::time::timeout(Duration::from_secs(2), &mut task)
                    .await
                    .unwrap()
                    .unwrap();
            }
        }
    }
    fn args() -> wire::TushareDailyArguments {
        wire::TushareDailyArguments {
            ts_code: "000001.SZ".into(),
            trade_date: "20260105".into(),
        }
    }
    #[test]
    fn projects_only_public_fields_and_one_row() {
        let result = rmcp::model::CallToolResult::structured(
            serde_json::json!({"code":0,"data":{"fields":["ts_code","trade_date","close","provider_note"],"items":[["000001.SZ","20260105",10.5,"ordinary note"]]}}),
        );
        assert_eq!(
            sanitize_result(&result, &args()).unwrap(),
            serde_json::json!({"rows":[{"ts_code":"000001.SZ","trade_date":"20260105","close":10.5}]})
        );
        let empty = rmcp::model::CallToolResult::structured(serde_json::json!({"data":[]}));
        assert_eq!(
            sanitize_result(&empty, &args()).unwrap(),
            serde_json::json!({"rows":[]})
        );
    }
    #[test]
    fn reviews_full_forwarded_union_branches_not_supplier_annotations() {
        let schema = serde_json::json!({"type":"object","properties":{"ts_code":{"anyOf":[{"type":"integer"},{"type":"string"}]},"trade_date":{"oneOf":[{"type":"string"},{"type":"integer"}]}},"required":[]});
        assert!(reviewed_input_shape(&schema));
        let mut bounded = schema.clone();
        bounded["properties"]["ts_code"]["anyOf"][1]["maxLength"] = serde_json::json!(9);
        assert!(!reviewed_input_shape(&bounded));
        let mut extra_required = schema;
        extra_required["required"] = serde_json::json!(["fields"]);
        assert!(!reviewed_input_shape(&extra_required));
    }
    #[test]
    fn ordinary_multirow_response_requires_different_qualification() {
        let row = serde_json::json!({"ts_code":"000001.SZ","trade_date":"20260105","close":10.5});
        let result = rmcp::model::CallToolResult::structured(serde_json::json!([row.clone(), row]));
        assert_eq!(
            sanitize_result(&result, &args()),
            Err(Failure::InvalidMetadata)
        );
    }
    #[test]
    fn identity_only_is_not_a_price_observation_or_an_empty_result() {
        let result = rmcp::model::CallToolResult::structured(
            serde_json::json!([{"ts_code":"000001.SZ","trade_date":"20260105"}]),
        );
        assert_eq!(
            sanitize_result(&result, &args()),
            Err(Failure::InvalidMetadata)
        );
    }
    #[test]
    fn approved_http_permit_is_consumed_at_most_once() {
        let gate = HttpPermit::default();
        let arguments = serde_json::to_value(args()).unwrap();
        gate.0.lock().unwrap().permit = Some(Permit {
            tool: wire::TUSHARE_DAILY_UPSTREAM_TOOL_NAME.into(),
            arguments: arguments.clone(),
            request: CancellationToken::new(),
            lease: CancellationToken::new(),
            backend: CancellationToken::new(),
            deadline: Instant::now() + Duration::from_secs(1),
        });
        let request = serde_json::json!({"method":"tools/call","params":{"name":"daily","arguments":arguments}});
        assert!(gate.take(&request));
        assert!(!gate.take(&request));
    }
    #[test]
    fn normal_cancel_before_send_spends_no_http_permit() {
        let gate = HttpPermit::default();
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let arguments = serde_json::to_value(args()).unwrap();
        gate.0.lock().unwrap().permit = Some(Permit {
            tool: wire::TUSHARE_DAILY_UPSTREAM_TOOL_NAME.into(),
            arguments: arguments.clone(),
            request: cancelled,
            lease: CancellationToken::new(),
            backend: CancellationToken::new(),
            deadline: Instant::now() + Duration::from_secs(1),
        });
        assert!(!gate.take(&serde_json::json!({"params":{"name":"daily","arguments":arguments}})));
    }
    #[tokio::test]
    async fn fixed_client_reads_ordinary_inprocess_result_and_closes_normally() {
        let (backend, calls, factory) = local_backend().await;
        let value = backend
            .execute(
                args(),
                CancellationToken::new(),
                CancellationToken::new(),
                Instant::now() + Duration::from_secs(2),
                Diagnostic::default(),
            )
            .await
            .unwrap();
        assert_eq!(value["rows"][0]["close"], 10.5);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        backend.close().await;
        factory.join().await;
    }
}
