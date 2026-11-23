//! Discovered MCP tools use a frozen schema and the same one-shot approval gate.
//! This module does not discover endpoints or acquire credentials.
use crate::{
    broker_generated as wire,
    daily::{HttpPermit, Permit},
    tushare_oauth::{Diagnostic, Failure},
};
use codex_rmcp_client::RmcpClient;
use jsonschema::{Retrieve, Uri, Validator};
use rmcp::model::{CallToolResult, Content, Tool};
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
fn bounded(value: &Value, remaining_depth: usize) -> bool {
    let child_fits = |child: &Value| {
        remaining_depth
            .checked_sub(1)
            .is_some_and(|budget| bounded(child, budget))
    };
    match value {
        Value::Object(map) => map.len() <= 256 && map.values().all(child_fits),
        Value::Array(items) => items.len() <= 1024 && items.iter().all(child_fits),
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
// Sorftime Amazon query schemas from FEAT-144 native discovery, operation 8.
// Match the actual upstream method and the complete schema before offering native consent.
fn reviewed_sorftime_query(service: &str, name: &str, schema: &Value) -> bool {
    if service != "sorftime" {
        return false;
    }
    let expected = match name {
        "category_name_search" => {
            "af32a2bbe17bf185d958e11ec12e3bd703e284a8a5aa11a11c4df77fbdd273af"
        }
        "category_report" => "16974b1998a4a4ef1b048afeeea1ee640d44fc2bc3c497af8f410e920d152f0b",
        "category_report_from_history" => {
            "c271dab50d9ab39e152377ade217254ead0f3d856f1a67031ce84bb77a076361"
        }
        "category_search_from_product_name" => {
            "e5b4c2751503f06890983db5afdca254dd4712052f75f97c5e5ecedc22322081"
        }
        "category_search_from_top_node" => {
            "a53fe575c93e38bb8a330041bffafaade3018653ea8b3229c0c281d3cbd2d0b5"
        }
        "category_tree" => "c3d105fa4996047c24a2d7b67858cf667263598f1d3e83a92cdbb85bd2109291",
        "category_trend" => "4df490b9491a2143e7b166b637faf5afbb946cbbb0707eb04dd1d8f91c8b1eea",
        "product_detail" => "8fba19a74826bb3fcd9b3a98a6ee74137568194a330cadbdf77c584bc5500f77",
        "product_search" => "d17fb37f0a94847b073735527f9e0feda5b79bcc5dd249ecefe33ae63635b5a4",
        "product_search_from_history" => {
            "1e6f2903d4104036286febc6c2df20e311630b1918edbe13c2ecce72bc810110"
        }
        "product_search_from_name" => {
            "837f514577a3096556de9df37becf0241970c3c271e8fc9bc34fc55a7cf541f1"
        }
        _ => return false,
    };
    let mut canonical = schema.clone();
    canonical.sort_all_objects();
    serde_json::to_vec(&canonical)
        .is_ok_and(|bytes| format!("{:x}", Sha256::digest(bytes)) == expected)
}

pub struct Definition {
    pub upstream: String,
    pub tool: Tool,
    pub digest: String,
    validator: Validator,
    destructive: bool,
    reviewed_query: bool,
    local_sample: bool,
}
impl Definition {
    fn new(service: &str, source: &Tool, encoded: &[u8]) -> Result<Self, &'static str> {
        let name = source.name.to_string();
        if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
            return Err("tool_name");
        }
        let schema = Value::Object((*source.input_schema).clone());
        if encoded.len() > wire::GENERIC_MAX_SCHEMA_BYTES {
            return Err("schema_bytes");
        }
        if !bounded(&schema, wire::GENERIC_MAX_SCHEMA_DEPTH) {
            return Err("schema_structure");
        }
        let validator = jsonschema::options()
            .with_retriever(LocalReferences)
            .with_pattern_options(jsonschema::PatternOptions::fancy_regex().backtrack_limit(10_000))
            .build(&schema)
            .map_err(|_| "schema_compile")?;
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
        let local_sample = crate::local_acceptance::enabled()
            && service == crate::local_acceptance::service_id()
            && name == "local_sample"
            && schema == crate::local_acceptance::profile()["tool"]["inputSchema"];
        let reviewed_query = reviewed_sorftime_query(service, &name, &schema);
        let mut tool = Tool::new(
            public_name,
            format!(
                "服务 {service}，工具 {name}。第三方说明：{description}\n执行时核对当前权限、调用范围和实际参数。"
            ),
            (*source.input_schema).clone(),
        );
        // Native MCP policy consumes the provider's original optional hints.
        // A missing hint stays missing. They are never an execution grant.
        tool.annotations = source.annotations.clone();
        tool.title = Some(format!("{service} · {name}"));
        Ok(Self {
            upstream: name,
            tool,
            digest: format!("{:x}", Sha256::digest(encoded)),
            validator,
            destructive,
            reviewed_query,
            local_sample,
        })
    }
    pub fn review(&self, args: &Map<String, Value>) -> Result<wire::ReviewProjection, Failure> {
        let value = Value::Object(args.clone());
        let encoded = serde_json::to_string(&value).map_err(|_| Failure::InvalidMetadata)?;
        if encoded.len() > wire::GENERIC_MAX_REVIEW_ARGUMENTS_BYTES
            || !bounded(&value, wire::GENERIC_MAX_DEPTH)
            || credentials(&value)
            || !self.validator.is_valid(&value)
        {
            return Err(Failure::InvalidMetadata);
        }
        let reviewed_read = self.local_sample
            || self.reviewed_query
                && args
                    .get("amz_site")
                    .and_then(Value::as_str)
                    .is_some_and(|site| {
                        site.len() == 2 && site.bytes().all(|b| b.is_ascii_uppercase())
                    })
                && self
                    .tool
                    .input_schema
                    .get("properties")
                    .and_then(Value::as_object)
                    .is_some_and(|fields| args.keys().all(|key| fields.contains_key(key)));
        let read = !self.destructive
            && (reviewed_read
                || self
                    .tool
                    .annotations
                    .as_ref()
                    .is_some_and(|a| a.read_only_hint == Some(true)));
        Ok(wire::ReviewProjection {
            title: self.upstream.chars().take(160).collect(),
            summary: if self.local_sample {
                "读取当前本地连接器的验收样本。"
            } else if reviewed_read {
                "查询所选 Amazon 站点的公开商品或类目数据。"
            } else if read {
                "服务将此工具标为只读。"
            } else if self.destructive {
                "服务将此工具标为可能造成不可逆更改。"
            } else {
                "此工具可能修改外部数据。"
            }
            .into(),
            risk: if read { "read" } else { "write" }.into(),
            arguments_json: Some(encoded),
            schema_digest: Some(self.digest.clone()),
        })
    }
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogIssue {
    reason: &'static str,
    tool_index: Option<usize>,
    tool_name_sha256: Option<String>,
    schema_bytes: usize,
    schema_depth: usize,
}

fn schema_depth(value: &Value) -> usize {
    let children = match value {
        Value::Object(fields) => fields.values().map(schema_depth).max(),
        Value::Array(items) => items.iter().map(schema_depth).max(),
        _ => None,
    };
    children.map_or(0, |depth| depth + 1)
}

pub fn definitions(service: &str, tools: Vec<Tool>) -> Result<Vec<Definition>, Vec<CatalogIssue>> {
    if tools.is_empty() || tools.len() > wire::GENERIC_MAX_TOOLS_PER_SERVICE {
        return Err(vec![CatalogIssue {
            reason: "tool_count",
            tool_index: None,
            tool_name_sha256: None,
            schema_bytes: 0,
            schema_depth: 0,
        }]);
    }
    let mut names = HashSet::new();
    let mut bytes = 0;
    let mut result = Vec::new();
    let mut issues = Vec::new();
    for (index, tool) in tools.iter().enumerate() {
        let issue = |reason, schema_bytes| {
            let depth = tool
                .input_schema
                .values()
                .map(schema_depth)
                .max()
                .map_or(0, |d| d + 1);
            let reason = match reason {
                "schema_structure" if depth > wire::GENERIC_MAX_SCHEMA_DEPTH => "schema_depth",
                "schema_structure" => "schema_width",
                other => other,
            };
            CatalogIssue {
                reason,
                tool_index: Some(index),
                tool_name_sha256: Some(format!("{:x}", Sha256::digest(tool.name.as_bytes()))),
                schema_bytes,
                schema_depth: depth,
            }
        };
        if !names.insert(tool.name.to_string()) {
            issues.push(issue("duplicate_tool_name", 0));
            continue;
        }
        let encoded = serde_json::to_vec(&tool.input_schema)
            .map_err(|_| vec![issue("schema_encoding", 0)])?;
        bytes += encoded.len();
        if bytes > wire::GENERIC_MAX_SCHEMAS_BYTES {
            issues.push(issue("catalog_bytes", encoded.len()));
            break;
        }
        match Definition::new(service, tool, &encoded) {
            Ok(definition) => result.push(definition),
            Err(reason) => issues.push(issue(reason, encoded.len())),
        }
    }
    if issues.is_empty() {
        Ok(result)
    } else {
        Err(issues)
    }
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
    if !bounded(&encoded, wire::GENERIC_MAX_DEPTH)
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
    use rmcp::model::ToolAnnotations;
    fn tool() -> Tool {
        Tool::new("search", "普通合成工具", serde_json::json!({"type":"object","properties":{"query":{"type":"string","minLength":1}},"required":["query"],"additionalProperties":false}).as_object().unwrap().clone())
    }
    #[test]
    fn catalog_diagnostics_identify_the_definition_without_exporting_supplier_text() {
        let issues = definitions("ordinary-service", vec![tool(), tool()])
            .err()
            .expect("two entries share the same upstream name");
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.reason, "duplicate_tool_name");
        assert_eq!(issue.tool_index, Some(1));
        assert_eq!(
            issue.tool_name_sha256,
            Some(format!("{:x}", Sha256::digest("search")))
        );
        let report = serde_json::to_string(&issue).unwrap();
        assert!(!report.contains("search"));
        assert!(!report.contains("普通合成工具"));
    }
    #[test]
    fn nested_template_schema_qualifies_without_relaxing_call_data_limits() {
        // Each 'properties' in a JSON Schema adds syntax that is absent from
        // the actual payload. This ordinary 9-field path has a depth-20 schema.
        let mut schema = serde_json::json!({"enum":["draft"]});
        let mut value = serde_json::json!("draft");
        for field in [
            "label", "option", "question", "section", "page", "form", "model", "state", "template",
        ] {
            schema = serde_json::json!({"type":"object","properties":{(field):schema},
                "required":[field],"additionalProperties":false});
            value = serde_json::json!({(field):value});
        }
        assert_eq!(schema_depth(&schema), 20);
        let edit = Tool::new(
            "template_preview",
            "Preview template changes",
            schema.as_object().unwrap().clone(),
        );
        let catalog = definitions("template-service", vec![tool(), edit]).unwrap();
        assert_eq!(catalog.len(), 2);
        assert_eq!(
            catalog[1].tool.input_schema.as_ref(),
            schema.as_object().unwrap()
        );
        assert!(catalog[1].review(value.as_object().unwrap()).is_ok());
        assert!(catalog[1].review(&Map::new()).is_err());

        // A wide-open input schema still cannot waive the invocation data limit.
        let open = Tool::new(
            "summarize",
            "Summarize a hierarchy",
            serde_json::json!({"type":"object"})
                .as_object()
                .unwrap()
                .clone(),
        );
        let catalog = definitions("another-service", vec![open]).unwrap();
        let mut tree = serde_json::json!("leaf");
        for _ in 0..wire::GENERIC_MAX_DEPTH {
            tree = serde_json::json!({"child":tree});
        }
        assert!(catalog[0].review(tree.as_object().unwrap()).is_ok());
        tree = serde_json::json!({"child":tree});
        assert!(catalog[0].review(tree.as_object().unwrap()).is_err());
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
    struct LocalService(Arc<AtomicUsize>, Tool);
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
                tools: vec![self.1.clone()],
                ..Default::default()
            })
        }
        async fn call_tool(
            &self,
            request: rmcp::model::CallToolRequestParams,
            _: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            assert_eq!(request.name, self.1.name);
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(CallToolResult::structured(
                serde_json::json!({"query":request.arguments.unwrap().get("query"),"count":1}),
            ))
        }
    }
    pub struct LocalFactory {
        tool: Tool,
        calls: Arc<AtomicUsize>,
        tasks: Mutex<Vec<tokio::task::JoinHandle<()>>>,
    }
    impl codex_rmcp_client::InProcessTransportFactory for LocalFactory {
        fn open(
            &self,
        ) -> futures::future::BoxFuture<'static, std::io::Result<tokio::io::DuplexStream>> {
            let (client, server) = tokio::io::duplex(65536);
            let handler = LocalService(self.calls.clone(), self.tool.clone());
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
        local_backend_for(service, tool()).await
    }
    pub async fn local_sorftime_query() -> (Arc<Backend>, Arc<AtomicUsize>, Arc<LocalFactory>) {
        let schema: Map<String, Value> = serde_json::from_str(include_str!(
            "../fixtures/sorftime-product-search.schema.json"
        ))
        .unwrap();
        local_backend_for(
            "sorftime",
            Tool::new("product_search", "本地公开查询样本", schema),
        )
        .await
    }
    async fn local_backend_for(
        service: &str,
        tool: Tool,
    ) -> (Arc<Backend>, Arc<AtomicUsize>, Arc<LocalFactory>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let factory = Arc::new(LocalFactory {
            tool,
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
    fn provider_annotations_and_review_are_available_to_native_policy() {
        let annotation = ToolAnnotations::new().destructive(true);
        let definitions = definitions(
            "service-a",
            vec![tool().with_annotations(annotation.clone())],
        )
        .unwrap();
        assert_eq!(definitions[0].tool.annotations.as_ref(), Some(&annotation));
        let review = definitions[0]
            .review(serde_json::json!({"query":"ordinary"}).as_object().unwrap())
            .unwrap();
        assert_eq!(review.risk, "write");
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

#[cfg(test)]
#[path = "linkfox_flow_tests.rs"]
mod linkfox_flow_tests;

#[cfg(test)]
#[path = "linkfox_video_tests.rs"]
mod linkfox_video_tests;

#[cfg(test)]
#[path = "linkfox_sourcing_tests.rs"]
mod linkfox_sourcing_tests;

#[cfg(test)]
#[path = "linkfox_sale_tests.rs"]
mod linkfox_sale_tests;
