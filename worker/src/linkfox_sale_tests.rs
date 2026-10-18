//! Ordinary in-process service, never an online LinkFox qualification.
use super::*;
use rmcp::{ServerHandler, ServiceExt};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};
fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/linkfox-sale-request.json")).unwrap()
}
#[derive(Clone)]
struct SaleService(Arc<AtomicUsize>);
fn proxy() -> Tool {
    Tool::new("proxy_call", "Ordinary local sales-video protocol", serde_json::json!({
        "type":"object","properties":{"toolName":{"type":"string"},"arguments":{"type":"object"}},
        "required":["toolName","arguments"],"additionalProperties":false
    }).as_object().unwrap().clone())
}
impl ServerHandler for SaleService {
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
        let args = Value::Object(request.arguments.unwrap());
        let index = self.0.fetch_add(1, Ordering::SeqCst);
        let data = match index {
            0 => {
                assert_eq!(args, fixture());
                assert_eq!(args["arguments"]["voice"], true);
                assert_eq!(args["arguments"]["promptOptimizer"], false);
                assert_eq!(args["arguments"]["videoTime"], 15);
                assert!(args["arguments"].get("isPro").is_none());
                let prompt = args["arguments"]["prompt"].as_str().unwrap();
                assert!(prompt.contains("It holds 350 millilitres."));
                assert!(prompt.contains("[10-15s]"));
                assert!(!prompt.contains("stainless"));
                serde_json::json!({"taskId":"sale-task"})
            }
            1 | 2 => {
                assert_eq!(
                    args,
                    serde_json::json!({"toolName":"aigc/taskQuery","arguments":{"taskId":"sale-task"}})
                );
                if index == 1 {
                    serde_json::json!({"taskId":"sale-task","status":"PROCESSING"})
                } else {
                    serde_json::json!({"taskId":"sale-task","status":"SUCCESS","resultList":[{"type":"video","url":"https://media.example.com/result.mp4"}]})
                }
            }
            _ => unreachable!(),
        };
        Ok(CallToolResult::structured(data))
    }
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
        let handler = SaleService(self.calls.clone());
        self.tasks.lock().unwrap().push(tokio::spawn(async move {
            let service = handler.serve(server).await.unwrap();
            let _ = service.waiting().await;
        }));
        Box::pin(async move { Ok(client) })
    }
}
#[tokio::test]
async fn feat158_linkfox_sale_preserves_selected_request_voice_and_original_task() {
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
                rmcp::model::Implementation::new("ordinary-sale-flow", "1"),
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
    // Three explicit ordinary protocol requests; no production polling loop.
    let query = serde_json::json!({"toolName":"aigc/taskQuery","arguments":{"taskId":"sale-task"}});
    for (index, args) in [fixture(), query.clone(), query].into_iter().enumerate() {
        let args = args.as_object().unwrap().clone();
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
        let data = result.structured_content.unwrap();
        assert_eq!(data["taskId"], "sale-task");
        if index == 1 {
            assert_eq!(data["status"], "PROCESSING");
            assert!(data.get("resultList").is_none());
        }
        if index == 2 {
            assert_eq!(data["status"], "SUCCESS");
            assert_eq!(data["resultList"][0]["type"], "video");
        }
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
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}
