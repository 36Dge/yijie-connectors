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
    Tool::new("proxy_call", "Ordinary synthetic asynchronous video service", serde_json::json!({
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
            "aigc/textGenAsync",
            "aigc/textTaskQuery",
            "aigc/textGenAsync",
            "aigc/textTaskQuery",
            "aigc/multiImageVideoGenAsync",
            "aigc/taskQuery",
            "aigc/taskQuery",
        ];
        assert_eq!(args["toolName"], expected[index]);
        let data = match index {
            0 => {
                assert_eq!(
                    args["arguments"]["imageUrls"][0],
                    "https://media.example.com/reference.mp4"
                );
                serde_json::json!({"taskId":"reference-task"})
            }
            1 => {
                assert_eq!(args["arguments"]["taskId"], "reference-task");
                serde_json::json!({"taskId":"reference-task","status":"SUCCESS","content":"Camera moves. Audio unknown."})
            }
            2 => {
                assert_eq!(
                    args["arguments"]["imageUrls"][0],
                    "https://media.example.com/product.jpg"
                );
                serde_json::json!({"taskId":"product-task"})
            }
            3 => {
                assert_eq!(args["arguments"]["taskId"], "product-task");
                serde_json::json!({"taskId":"product-task","status":"SUCCESS","content":"Blue organizer. Dimensions unknown."})
            }
            4 => {
                assert_eq!(args["arguments"]["imageUrls"], serde_json::json!([]));
                serde_json::json!({"taskId":"prompt-task"})
            }
            5 => {
                assert_eq!(args["arguments"]["taskId"], "prompt-task");
                serde_json::json!({"taskId":"prompt-task","status":"SUCCESS","content":"[0-5s] Blue organizer.\n[5-15s] Camera moves slowly."})
            }
            6 => {
                assert_eq!(
                    args["arguments"]["prompt"],
                    "[0-5s] Blue organizer.\n[5-15s] Camera moves slowly."
                );
                assert_eq!(
                    args["arguments"]["imageList"],
                    serde_json::json!([
                        "https://media.example.com/product.jpg",
                        "https://media.example.com/detail.jpg"
                    ])
                );
                assert_eq!(args["arguments"]["videoType"], "SEED_FAST");
                assert_eq!(args["arguments"]["videoTime"], 15);
                assert_eq!(args["arguments"]["voice"], false);
                assert!(args["arguments"].get("isPro").is_none());
                serde_json::json!({"taskId":"video-task"})
            }
            7 => {
                assert_eq!(args["arguments"]["taskId"], "video-task");
                serde_json::json!({"taskId":"video-task","status":"PROCESSING"})
            }
            8 => {
                assert_eq!(args["arguments"]["taskId"], "video-task");
                serde_json::json!({"taskId":"video-task","status":"SUCCESS","resultList":[{"type":"video","url":"https://media.example.com/result.mp4"}]})
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
async fn feat158_linkfox_video_flow_keeps_four_tasks_and_exact_media_arguments() {
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
                rmcp::model::Implementation::new("ordinary-video-flow", "1"),
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
        "aigc/textGenAsync",
        "aigc/textTaskQuery",
        "aigc/textGenAsync",
        "aigc/textTaskQuery",
        "aigc/multiImageVideoGenAsync",
        "aigc/taskQuery",
        "aigc/taskQuery",
    ]
    .iter()
    .enumerate()
    {
        // Each invocation is a separate requested operation; this local protocol
        // fixture is not a production loop, automatic continuation or live service.
        let business = match index {
            0 => {
                serde_json::json!({"prompt":"Analyze reference","imageUrls":["https://media.example.com/reference.mp4"],"model":"LF_GEM_2","thinkingLevel":"low"})
            }
            1 => serde_json::json!({"taskId":"reference-task"}),
            2 => {
                serde_json::json!({"prompt":"Analyze product","imageUrls":["https://media.example.com/product.jpg"],"model":"LF_GEM_2","thinkingLevel":"low"})
            }
            3 => serde_json::json!({"taskId":"product-task"}),
            4 => {
                serde_json::json!({"prompt":"Replace product with grounded facts","imageUrls":[],"model":"LF_GEM_2","thinkingLevel":"low"})
            }
            5 => serde_json::json!({"taskId":"prompt-task"}),
            6 => {
                serde_json::json!({"prompt":prompt,"imageList":["https://media.example.com/product.jpg","https://media.example.com/detail.jpg"],"videoType":"SEED_FAST","videoTime":15,"aspectRatio":"9:16","resolution":"720p","voice":false,"promptOptimizer":false})
            }
            _ => serde_json::json!({"taskId":"video-task"}),
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
        if index == 7 {
            assert_eq!(data["status"], "PROCESSING");
            assert!(data.get("content").is_none());
        }
        if index == 5 {
            prompt = data["content"].as_str().unwrap().to_owned();
            assert!(prompt.contains('\n'));
        }
        if index == 8 {
            assert_eq!(
                data["resultList"][0]["url"],
                "https://media.example.com/result.mp4"
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
    assert_eq!(calls.load(Ordering::SeqCst), 9);
}
