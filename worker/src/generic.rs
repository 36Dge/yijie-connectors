//! Discovered MCP tools use a frozen schema and the same one-shot approval gate.
//! This module does not discover endpoints or acquire credentials.
use crate::{
    broker_generated as wire,
    daily::{HttpPermit, Permit},
    tushare_oauth::{Diagnostic, Failure},
};
use codex_rmcp_client::RmcpClient;
use jsonschema::{Retrieve, Uri, Validator};
use rmcp::model::{CallToolResult, Content, Tool, ToolAnnotations};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

struct LocalReferences;
impl Retrieve for LocalReferences {
    fn retrieve(&self, _: &Uri<String>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Err("only bundled local schema references are supported".into())
    }
}
fn bounded(value: &Value, depth: usize) -> bool {
    if depth > wire::GENERIC_MAX_DEPTH {
        return false;
    }
    match value {
        Value::Object(map) => map.len() <= 256 && map.values().all(|v| bounded(v, depth + 1)),
        Value::Array(items) => items.len() <= 1024 && items.iter().all(|v| bounded(v, depth + 1)),
        _ => true,
    }
}
// Secrets belong in the provider's credential entry, never in model arguments.
// Deny rather than redact/execute a different argument value than was reviewed.
fn credential_field(key: &str) -> bool {
    let key: String = key
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        key.as_str(),
        "token"
            | "accesstoken"
            | "refreshtoken"
            | "apikey"
            | "clientsecret"
            | "password"
            | "authorization"
            | "cookie"
            | "privatekey"
            | "credential"
            | "credentials"
    )
}
fn credentials(value: &Value) -> bool {
    match value {
        Value::Object(map) => map
            .iter()
            .any(|(k, v)| credential_field(k) || credentials(v)),
        Value::Array(items) => items.iter().any(credentials),
        _ => false,
    }
}
pub struct Definition {
    pub upstream: String,
    pub tool: Tool,
    pub digest: String,
    validator: Validator,
    destructive: bool,
}
impl Definition {
    fn new(service: &str, source: Tool) -> Result<Self, Failure> {
        let name = source.name.to_string();
        if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
            return Err(Failure::InvalidMetadata);
        }
        let schema = Value::Object((*source.input_schema).clone());
        let encoded = serde_json::to_vec(&schema).map_err(|_| Failure::InvalidMetadata)?;
        if encoded.len() > wire::GENERIC_MAX_SCHEMA_BYTES || !bounded(&schema, 0) {
            return Err(Failure::InvalidMetadata);
        }
        let validator = jsonschema::options()
            .with_retriever(LocalReferences)
            .with_pattern_options(jsonschema::PatternOptions::fancy_regex().backtrack_limit(10_000))
            .build(&schema)
            .map_err(|_| Failure::InvalidMetadata)?;
        let id = format!("{:x}", Sha256::digest(format!("{service}\n{name}")));
        // The full digest makes names stable and unambiguous without assuming
        // provider names are safe identifiers or unique across services.
        let public_name = format!("m_{}", &id[..60]);
        let description = source.description.as_deref().unwrap_or("第三方 MCP 工具");
        let description: String = description
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(2048)
            .collect();
        let destructive = (service == crate::google_calendar::SERVICE
            && source.name == "delete_event")
            || source
                .annotations
                .as_ref()
                .is_some_and(|a| a.destructive_hint == Some(true));
        let mut tool = Tool::new(
            public_name,
            format!(
                "服务 {service}，工具 {name}。第三方说明：{description}\n每次调用必须单独批准；供应商说明不构成执行授权。"
            ),
            (*source.input_schema).clone(),
        );
        // Preserve full input schema, but never advertise unreviewed provider
        // risk hints as trusted read/idempotency facts to the Runtime.
        tool.annotations = Some(
            ToolAnnotations::new()
                .read_only(false)
                .destructive(true)
                .idempotent(false)
                .open_world(true),
        );
        tool.title = Some(format!("{service} · {name}"));
        Ok(Self {
            upstream: name,
            tool,
            digest: format!("{:x}", Sha256::digest(encoded)),
            validator,
            destructive,
        })
    }
    pub fn review(&self, args: &Map<String, Value>) -> Result<wire::ReviewProjection, Failure> {
        let value = Value::Object(args.clone());
        let encoded = serde_json::to_string(&value).map_err(|_| Failure::InvalidMetadata)?;
        if self.destructive
            || encoded.len() > wire::GENERIC_MAX_REVIEW_ARGUMENTS_BYTES
            || !bounded(&value, 0)
            || credentials(&value)
            || !self.validator.is_valid(&value)
        {
            return Err(Failure::InvalidMetadata);
        }
        Ok(wire::ReviewProjection {
            title: self.upstream.chars().take(160).collect(),
            summary: "第三方工具尚未被独立确认为只读，可能修改外部数据。请核对服务、工具及下方完整参数。本次批准只允许执行一次，失败或结果未知时不会自动重试。".into(),
            risk: "write".into(), arguments_json: Some(encoded), schema_digest: Some(self.digest.clone()),
        })
    }
}
pub fn definitions(service: &str, tools: Vec<Tool>) -> Result<Vec<Definition>, Failure> {
    if tools.is_empty() || tools.len() > wire::GENERIC_MAX_TOOLS_PER_SERVICE {
        return Err(Failure::InvalidMetadata);
    }
    let mut names = HashSet::new();
    let mut bytes = 0;
    let mut result = Vec::new();
    for tool in tools {
        if !names.insert(tool.name.to_string()) {
            return Err(Failure::InvalidMetadata);
        }
        bytes += serde_json::to_vec(&tool.input_schema)
            .map_err(|_| Failure::InvalidMetadata)?
            .len();
        if bytes > wire::GENERIC_MAX_SCHEMAS_BYTES {
            return Err(Failure::InvalidMetadata);
        }
        result.push(Definition::new(service, tool)?);
    }
    Ok(result)
}
pub struct Backend {
    pub service: String,
    pub definitions: Vec<Definition>,
    client: RmcpClient,
    permit: Arc<HttpPermit>,
    serial: tokio::sync::Mutex<()>,
    cancelled: CancellationToken,
}
impl Backend {
    pub fn new(
        service: String,
        definitions: Vec<Definition>,
        client: RmcpClient,
        permit: Arc<HttpPermit>,
    ) -> Arc<Self> {
        Arc::new(Self {
            service,
            definitions,
            client,
            permit,
            serial: tokio::sync::Mutex::new(()),
            cancelled: CancellationToken::new(),
        })
    }
    pub fn definition(&self, name: &str) -> Option<&Definition> {
        self.definitions.iter().find(|d| d.tool.name == name)
    }
    pub fn review(
        &self,
        name: &str,
        args: &Map<String, Value>,
    ) -> Result<wire::ReviewProjection, Failure> {
        if !self.permit.1.safe(&Value::Object(args.clone())) {
            return Err(Failure::InvalidMetadata);
        }
        self.definition(name)
            .ok_or(Failure::InvalidMetadata)?
            .review(args)
    }
    pub fn revoke(&self) {
        self.cancelled.cancel();
    }
    pub fn available(&self) -> bool {
        !self.cancelled.is_cancelled() && self.permit.1.available()
    }
    pub async fn close(&self) {
        self.revoke();
        let _guard = self.serial.lock().await;
        self.client.shutdown().await;
    }
    pub async fn execute(
        &self,
        name: &str,
        args: Map<String, Value>,
        permit: Permit,
        diagnostic: Diagnostic,
    ) -> Result<CallToolResult, Failure> {
        let _guard = self.serial.lock().await;
        let definition = self.definition(name).ok_or(Failure::InvalidMetadata)?;
        self.review(name, &args)?;
        if !self.permit.1.safe(&Value::Object(args.clone())) {
            return Err(Failure::InvalidMetadata);
        }
        if !self.available()
            || permit.request.is_cancelled()
            || permit.lease.is_cancelled()
            || Instant::now() >= permit.deadline
        {
            return Err(Failure::Cancelled);
        }
        let deadline = permit.deadline;
        let request = permit.request.clone();
        let lease = permit.lease.clone();
        self.permit.arm(
            Permit {
                tool: definition.upstream.clone(),
                arguments: Value::Object(args.clone()),
                backend: self.cancelled.clone(),
                ..permit
            },
            diagnostic,
        );
        let result = self
            .client
            .call_tool(
                definition.upstream.clone(),
                Some(Value::Object(args)),
                None,
                Some(Duration::from_secs(15)),
            )
            .await;
        self.permit.clear();
        if !self.available()
            || request.is_cancelled()
            || lease.is_cancelled()
            || Instant::now() >= deadline
        {
            return Err(Failure::Cancelled);
        }
        let result = result.map_err(|_| Failure::Unavailable)?;
        if !self
            .permit
            .1
            .safe(&serde_json::to_value(&result).map_err(|_| Failure::InvalidMetadata)?)
        {
            return Err(Failure::InvalidMetadata);
        }
        for content in &result.content {
            if let Some(text) = content.raw.as_text()
                && let Ok(value) = serde_json::from_str::<Value>(&text.text)
                && !self.permit.1.safe(&value)
            {
                return Err(Failure::InvalidMetadata);
            }
        }
        safe_result(result)
    }
}
fn safe_result(result: CallToolResult) -> Result<CallToolResult, Failure> {
    let encoded = serde_json::to_value(&result).map_err(|_| Failure::InvalidMetadata)?;
    if !bounded(&encoded, 0)
        || serde_json::to_vec(&encoded)
            .map_err(|_| Failure::InvalidMetadata)?
            .len()
            > wire::GENERIC_MAX_RESULT_BYTES
        || credentials(&encoded)
    {
        return Err(Failure::InvalidMetadata);
    }
    // Only inert text/JSON. Embedded resources, external links and binary data
    // require a dedicated artifact policy and are not silently fetched here.
    let mut content = Vec::new();
    for item in result.content {
        let Some(text) = item.raw.as_text() else {
            return Err(Failure::InvalidMetadata);
        };
        if let Ok(value) = serde_json::from_str::<Value>(&text.text)
            && credentials(&value)
        {
            return Err(Failure::InvalidMetadata);
        }
        content.push(Content::text(text.text.clone()));
    }
    let mut safe = CallToolResult::success(content);
    safe.structured_content = result.structured_content;
    safe.is_error = result.is_error;
    Ok(safe)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    fn tool() -> Tool {
        Tool::new("search", "普通合成工具", serde_json::json!({"type":"object","properties":{"query":{"type":"string","minLength":1}},"required":["query"],"additionalProperties":false}).as_object().unwrap().clone())
    }
    #[test]
    fn cross_border_large_catalog_remains_complete_and_bounded() {
        let catalog = |count: usize| {
            (0..count)
                .map(|index| {
                    let mut value = tool();
                    value.name = format!("catalog_query_{index}").into();
                    value
                })
                .collect()
        };
        let discovered = definitions("lingxing", catalog(320)).unwrap();
        assert_eq!(discovered.len(), 320);
        assert!(discovered.iter().all(|d| {
            d.review(
                serde_json::json!({"query":"ordinary public catalog"})
                    .as_object()
                    .unwrap(),
            )
            .is_ok()
        }));
        assert_eq!(
            definitions("lingxing", catalog(wire::GENERIC_MAX_TOOLS_PER_SERVICE))
                .unwrap()
                .len(),
            wire::GENERIC_MAX_TOOLS_PER_SERVICE
        );
        assert!(definitions("lingxing", catalog(wire::GENERIC_MAX_TOOLS_PER_SERVICE + 1)).is_err());
        assert!(discovered.len() <= wire::GENERIC_MAX_TOOLS_PER_SELECTION);
    }
    use rmcp::{ServerHandler, ServiceExt};
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    #[derive(Clone)]
    struct LocalService(Arc<AtomicUsize>);
    impl ServerHandler for LocalService {
        fn get_info(&self) -> rmcp::model::ServerInfo {
            rmcp::model::ServerInfo::new(
                rmcp::model::ServerCapabilities::builder()
                    .enable_tools()
                    .build(),
            )
        }
        async fn list_tools(
            &self,
            _: Option<rmcp::model::PaginatedRequestParams>,
            _: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
            Ok(rmcp::model::ListToolsResult {
                tools: vec![tool()],
                ..Default::default()
            })
        }
        async fn call_tool(
            &self,
            request: rmcp::model::CallToolRequestParams,
            _: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            assert_eq!(request.name, "search");
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(CallToolResult::structured(
                serde_json::json!({"query":request.arguments.unwrap()["query"],"count":1}),
            ))
        }
    }
    pub struct LocalFactory {
        calls: Arc<AtomicUsize>,
        tasks: Mutex<Vec<tokio::task::JoinHandle<()>>>,
    }
    impl codex_rmcp_client::InProcessTransportFactory for LocalFactory {
        fn open(
            &self,
        ) -> futures::future::BoxFuture<'static, std::io::Result<tokio::io::DuplexStream>> {
            let (client, server) = tokio::io::duplex(65536);
            let handler = LocalService(self.calls.clone());
            self.tasks.lock().unwrap().push(tokio::spawn(async move {
                let service = handler.serve(server).await.unwrap();
                let _ = service.waiting().await;
            }));
            Box::pin(async move { Ok(client) })
        }
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
    pub async fn local_backend(
        service: &str,
    ) -> (Arc<Backend>, Arc<AtomicUsize>, Arc<LocalFactory>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let factory = Arc::new(LocalFactory {
            calls: calls.clone(),
            tasks: Mutex::default(),
        });
        let client = RmcpClient::new_in_process_client(factory.clone())
            .await
            .unwrap();
        client
            .initialize(
                rmcp::model::InitializeRequestParams::new(
                    rmcp::model::ClientCapabilities::default(),
                    rmcp::model::Implementation::new("ordinary-generic-test", "1"),
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
        let tools = client
            .list_tools(None, Some(Duration::from_secs(2)))
            .await
            .unwrap()
            .tools;
        (
            Backend::new(
                service.into(),
                definitions(service, tools).unwrap(),
                client,
                Arc::new(HttpPermit::default()),
            ),
            calls,
            factory,
        )
    }
    #[test]
    fn normal_discovery_freezes_schema_and_separates_service_names() {
        let a = definitions("service-a", vec![tool()]).unwrap();
        let b = definitions("service-b", vec![tool()]).unwrap();
        assert_ne!(a[0].tool.name, b[0].tool.name);
        let args = serde_json::json!({"query":"普通查询"})
            .as_object()
            .unwrap()
            .clone();
        let review = a[0].review(&args).unwrap();
        assert_eq!(review.risk, "write");
        assert_eq!(
            serde_json::from_str::<Value>(review.arguments_json.as_deref().unwrap()).unwrap(),
            Value::Object(args)
        );
        assert!(a[0].review(&Map::new()).is_err());
    }
    #[test]
    fn explicitly_destructive_tool_waits_for_a_dedicated_policy() {
        let tool = tool().with_annotations(ToolAnnotations::new().destructive(true));
        let definitions = definitions("service-a", vec![tool]).unwrap();
        assert!(
            definitions[0]
                .review(serde_json::json!({"query":"ordinary"}).as_object().unwrap())
                .is_err()
        );
    }
    #[test]
    fn inert_results_keep_provider_error_and_structured_values() {
        let value = serde_json::json!({"rows":[{"value":1}]});
        let result = safe_result(CallToolResult::structured(value.clone())).unwrap();
        assert_eq!(result.structured_content, Some(value));
        assert_eq!(
            safe_result(CallToolResult::error(vec![Content::text(
                "ordinary failure"
            )]))
            .unwrap()
            .is_error,
            Some(true)
        );
    }
}
