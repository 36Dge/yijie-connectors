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
    Tool::new("proxy_call", "Ordinary synthetic asynchronous image service", serde_json::json!({
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
        let expected = [
            "aigc/textGenAsync",
            "aigc/textTaskQuery",
            "aigc/textTaskQuery",
            "aigc/imageGenAsync",
            "aigc/taskQuery",
        ];
        assert_eq!(args["toolName"], expected[index]);
        if [1, 2].contains(&index) {
            assert_eq!(args["arguments"]["taskId"], "ordinary-text-task");
        }
        if index == 4 {
            assert_eq!(args["arguments"]["taskId"], "ordinary-image-task");
        }
        let data = match index {
            0 => serde_json::json!({"taskId":"ordinary-text-task"}),
            1 => serde_json::json!({"taskId":"ordinary-text-task","status":"PROCESSING"}),
            2 => {
                serde_json::json!({"taskId":"ordinary-text-task","status":"SUCCESS","content":"Preserve the reference garment.\nNeutral background."})
            }
            3 => {
                assert_eq!(args["arguments"]["outputNum"], 1);
                serde_json::json!({"taskId":"ordinary-image-task"})
            }
            4 => {
                serde_json::json!({"taskId":"ordinary-image-task","status":"SUCCESS","resultList":[{"type":"image","url":"https://images.example.com/generated.png"}]})
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
        let handler = Service(self.calls.clone());
        self.tasks.lock().unwrap().push(tokio::spawn(async move {
            let service = handler.serve(server).await.unwrap();
            let _ = service.waiting().await;
        }));
        Box::pin(async move { Ok(client) })
    }
}
#[tokio::test]
async fn feat158_linkfox_async_text_and_image_flow_preserves_ids_without_automatic_polling() {
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
                rmcp::model::Implementation::new("ordinary-image-flow", "1"),
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
    let mut prompt = String::new();
    for (index, method) in [
        "aigc/textGenAsync",
        "aigc/textTaskQuery",
        "aigc/textTaskQuery",
        "aigc/imageGenAsync",
        "aigc/taskQuery",
    ]
    .iter()
    .enumerate()
    {
        // Each loop iteration represents a separately requested, approved operation.
        // No production polling, automatic continuation or creation retry is added.
        let business = match index {
            0 => {
                serde_json::json!({"prompt":"ordinary authorized image brief","imageUrls":["https://images.example.com/garment.png"],"model":"LF_GEM_2"})
            }
            1 | 2 => serde_json::json!({"taskId":"ordinary-text-task"}),
            3 => {
                serde_json::json!({"prompt":prompt,"imageUrls":["https://images.example.com/garment.png"],"outputNum":1})
            }
            _ => serde_json::json!({"taskId":"ordinary-image-task"}),
        };
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
        let data = result.structured_content.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), index + 1);
        if index == 1 {
            assert_eq!(data["status"], "PROCESSING");
            assert!(data.get("content").is_none());
        }
        if index == 2 {
            prompt = data["content"].as_str().unwrap().to_owned();
            assert!(prompt.contains('\n'));
        }
        if index == 4 {
            assert_eq!(
                data["resultList"][0]["url"],
                "https://images.example.com/generated.png"
            );
        }
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
