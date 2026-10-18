//! Normal in-process MCP service exercises the production generic backend.
//! No account, HTTP business call, executable substitution or live quality claim.
use super::*;
use rmcp::{ServerHandler, ServiceExt};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
struct Service(Arc<AtomicUsize>);
fn proxy() -> Tool {
    Tool::new("proxy_call", "Ordinary synthetic sourcing data service", serde_json::json!({
        "type":"object","properties":{"toolName":{"type":"string"},"arguments":{"type":"object"}},
        "required":["toolName","arguments"],"additionalProperties":false
    }).as_object().unwrap().clone())
}
impl ServerHandler for Service {
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
            tools: vec![proxy()],
            ..Default::default()
        })
    }
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        assert_eq!(request.name, "proxy_call");
        let args = request.arguments.unwrap();
        let index = self.0.fetch_add(1, Ordering::SeqCst);
        let (method, business, result) = cases().remove(index);
        assert_eq!(args["toolName"], method);
        assert_eq!(args["arguments"], business);
        Ok(CallToolResult::structured(result))
    }
}
fn cases() -> Vec<(&'static str, Value, Value)> {
    vec![
        (
            "keepa/productRequest",
            serde_json::json!({"asin":"B0TEST0001","domain":"1","history":0}),
            serde_json::json!({"errcode":200,"products":[{"asin":"B0TEST0001","currency":"USD","price":30}]}),
        ),
        (
            "keepa/productSeries",
            serde_json::json!({"asin":"B0TEST0001","domain":"1","days":90}),
            serde_json::json!({"asin":"B0TEST0001","buyboxPrice":[{"time":"2026-10-01","value":30}]}),
        ),
        (
            "dld/productSearch",
            serde_json::json!({"keyWord":"收纳架","cycle":"30","searchType":1,"pageIndex":1,"pageSize":20}),
            serde_json::json!({"errcode":200,"products":[{"offerId":"16880001","asin":"16880001","currency":"CNY","price":36,"quantityBegin":100}]}),
        ),
        (
            "sif/asinSummary",
            serde_json::json!({"searchValue":"B0TEST0001","country":"US","last7d":true,"pageNum":1,"pageSize":20}),
            serde_json::json!({"code":"200","data":[{"asin":"B0TEST0001"}]}),
        ),
        (
            "jiimore/getNicheInfoByKeyword",
            serde_json::json!({"keyword":"organizer","countryCode":"US","page":1,"pageSize":20}),
            serde_json::json!({"data":[{"nicheId":"niche-one","nicheTitle":"organizer","acos":0.2,"returnRateAnnual":0.1}]}),
        ),
    ]
}
struct Factory {
    calls: Arc<AtomicUsize>,
    tasks: Mutex<Vec<tokio::task::JoinHandle<()>>>,
}
impl codex_rmcp_client::InProcessTransportFactory for Factory {
    fn open(
        &self,
    ) -> futures::future::BoxFuture<'static, std::io::Result<tokio::io::DuplexStream>> {
        let (client, server) = tokio::io::duplex(65536);
        let handler = Service(self.calls.clone());
        self.tasks.lock().unwrap().push(tokio::spawn(async move {
            let service = handler.serve(server).await.unwrap();
            let _ = service.waiting().await;
        }));
        Box::pin(async move { Ok(client) })
    }
}
#[tokio::test]
async fn feat158_linkfox_sourcing_flow_preserves_target_and_supplier_identity() {
    let calls = Arc::new(AtomicUsize::new(0));
    let factory = Arc::new(Factory {
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
                rmcp::model::Implementation::new("ordinary-sourcing-flow", "1"),
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
    let backend = Backend::new(
        "linkfox".into(),
        definitions("linkfox", tools).unwrap(),
        client,
        Arc::new(HttpPermit::default()),
    );
    let name = backend.definitions[0].tool.name.to_string();
    for (index, (method, business, expected)) in cases().into_iter().enumerate() {
        // Separate explicit operations. No automatic query expansion or external service.
        let args = serde_json::json!({"toolName":method,"arguments":business})
            .as_object()
            .unwrap()
            .clone();
        assert_eq!(backend.review(&name, &args).unwrap().risk, "write");
        assert_eq!(calls.load(Ordering::SeqCst), index);
        let result = backend
            .execute(
                &name,
                args,
                Permit {
                    tool: String::new(),
                    arguments: Value::Null,
                    request: CancellationToken::new(),
                    lease: CancellationToken::new(),
                    backend: CancellationToken::new(),
                    deadline: Instant::now() + Duration::from_secs(5),
                },
                Diagnostic::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.structured_content.unwrap(), expected);
        assert_eq!(calls.load(Ordering::SeqCst), index + 1);
    }
    backend.close().await;
    let tasks = std::mem::take(&mut *factory.tasks.lock().unwrap());
    for task in tasks {
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 5);
}
