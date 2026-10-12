//! Ordinary local protocol checks for the observed public notification response.
//! No product endpoint override, credentials, external request or executable shim.
use super::*;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};

#[test]
fn shopify_shape_diagnostic_contains_only_closed_method_and_header_facts() {
    let response = HttpRequestResponse {
        status: 200,
        headers: vec![
            HttpHeader {
                name: "Content-Type".into(),
                value: "application/json; charset=utf-8".into(),
            },
            HttpHeader {
                name: "Content-Length".into(),
                value: "4".into(),
            },
            HttpHeader {
                name: "Server".into(),
                value: "ordinary-local-server".into(),
            },
        ],
        body: b"null".to_vec().into(),
    };
    assert_eq!(
        shopify_response_shape(ShopifyMetadataMethod::Initialized, true, &response),
        serde_json::json!({
            "method":"initialized","ackCandidate":true,"contentLengthCount":1,"contentLength":4,
            "contentTypeCount":1,"json":true,"eventStream":false,"transferEncoding":false
        })
    );
}

#[test]
fn exact_shopify_notification_is_the_only_bridge_input() {
    let client = TushareHttpClient {
        target: Target::for_service(crate::shopify::SERVICE).unwrap(),
        permit: None,
        credential: None,
        metadata_diagnostic: Diagnostic::default(),
    };
    let notification = serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"});
    let request = |body| HttpRequestParams {
        method: "POST".into(),
        url: client.target.resource.clone(),
        headers: vec![],
        body: Some(serde_json::to_vec(&body).unwrap().into()),
        timeout_ms: Some(15_000),
        redirect_policy: HttpRedirectPolicy::Stop,
        request_id: Uuid::new_v4().to_string(),
        stream_response: true,
    };
    let admitted = client.admitted(request(notification.clone())).unwrap();
    assert!(client.is_shopify_initialized_notification(&admitted));
    assert!(!TushareHttpClient::default().is_shopify_initialized_notification(&admitted));
    for method in ["initialize", "tools/list", "ping"] {
        let admitted = client
            .admitted(request(
                serde_json::json!({"jsonrpc":"2.0","id":0,"method":method}),
            ))
            .unwrap();
        assert!(!client.is_shopify_initialized_notification(&admitted));
    }
    let mut session_get = request(notification);
    session_get.method = "GET".into();
    session_get.body = None;
    assert!(!client.is_shopify_initialized_notification(&session_get));
}

#[derive(Clone)]
struct LocalAckClient {
    bridge: bool,
    statuses: Arc<Mutex<Vec<(u16, u16)>>>,
}
impl HttpClient for LocalAckClient {
    fn http_request(
        &self,
        request: HttpRequestParams,
    ) -> Pin<Box<dyn Future<Output = Result<HttpRequestResponse, ExecServerError>> + Send + '_>>
    {
        Box::pin(async move { ReqwestHttpClient.http_request(request).await })
    }
    fn http_request_stream(
        &self,
        request: HttpRequestParams,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<(HttpRequestResponse, HttpResponseBodyStream), ExecServerError>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let notification = request.body.as_ref().is_some_and(|body| {
                serde_json::from_slice::<serde_json::Value>(&body.0).ok()
                    == Some(
                        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                    )
            });
            let result = ReqwestHttpClient.http_request_stream(request).await?;
            let original_status = result.0.status;
            // The product classifier is separately checked against the fixed
            // registry above. This loopback harness exercises only the bridge
            // and actual SDK parser, without altering any product target rule.
            let result = if self.bridge && notification {
                normalize_shopify_initialized_ack(result).await?
            } else {
                result
            };
            if notification {
                self.statuses
                    .lock()
                    .unwrap()
                    .push((original_status, result.0.status));
            }
            Ok(result)
        })
    }
}

#[tokio::test]
async fn fixed_sdk_observed_null_ack_and_standard_acks() {
    async fn rpc(
        State((style, methods)): State<(&'static str, Arc<Mutex<Vec<String>>>)>,
        Json(body): Json<serde_json::Value>,
    ) -> axum::response::Response {
        let method = body["method"].as_str().unwrap();
        methods.lock().unwrap().push(method.into());
        let result = match method {
            "initialize" => serde_json::json!({
                "protocolVersion":body["params"]["protocolVersion"],
                "capabilities":{"tools":{}},
                "serverInfo":{"name":"ordinary-local-metadata","version":"1"}
            }),
            "notifications/initialized" => {
                return match style {
                    "observed-null" => {
                        ([("content-type", "application/json")], "null").into_response()
                    }
                    "observed-chunked-null" => axum::response::Response::builder()
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from_stream(futures::stream::iter([
                            Ok::<_, std::convert::Infallible>(axum::body::Bytes::from_static(
                                b"nu",
                            )),
                            Ok::<_, std::convert::Infallible>(axum::body::Bytes::from_static(
                                b"ll",
                            )),
                        ])))
                        .unwrap(),
                    "standard-204" => StatusCode::NO_CONTENT.into_response(),
                    "valid-json" => Json(serde_json::json!({"jsonrpc":"2.0","id":0,"result":{}}))
                        .into_response(),
                    _ => unreachable!(),
                };
            }
            "tools/list" => serde_json::json!({"tools":[]}),
            _ => panic!("only ordinary metadata methods expected"),
        };
        Json(serde_json::json!({"jsonrpc":"2.0","id":body["id"],"result":result})).into_response()
    }
    for (style, bridge, expected_ready, expected_status) in [
        ("observed-null", false, false, (200, 200)),
        ("observed-null", true, true, (200, 204)),
        ("observed-chunked-null", false, false, (200, 200)),
        ("observed-chunked-null", true, true, (200, 204)),
        ("standard-204", true, true, (204, 204)),
        ("valid-json", true, true, (200, 200)),
    ] {
        let methods: Arc<Mutex<Vec<String>>> = Arc::default();
        let statuses = Arc::default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let resource = format!("http://{}/mcp", listener.local_addr().unwrap());
        let stop = CancellationToken::new();
        let stopped = stop.clone();
        let app = Router::new()
            .route("/mcp", post(rpc))
            .with_state((style, methods.clone()));
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(stopped.cancelled_owned())
                .await
                .unwrap();
        });
        let client = codex_rmcp_client::RmcpClient::new_streamable_http_client(
            "ordinary-local-ack-check",
            &resource,
            None,
            // Like the existing local metadata test, an explicit synthetic
            // loopback header prevents any Keyring/OAuth lookup in the SDK.
            // It is never used in the product's keyless Shopify path.
            Some(HashMap::from([(
                "Authorization".into(),
                "Bearer ordinary-loopback-only".into(),
            )])),
            None,
            OAuthCredentialsStoreMode::Keyring,
            AuthKeyringBackendKind::Direct,
            Arc::new(LocalAckClient {
                bridge,
                statuses: Arc::clone(&statuses),
            }),
            None,
        )
        .await
        .unwrap();
        let initialized = client
            .initialize(
                rmcp::model::InitializeRequestParams::new(
                    rmcp::model::ClientCapabilities::default(),
                    rmcp::model::Implementation::new("ordinary-local-ack-check", "1"),
                ),
                Some(Duration::from_secs(5)),
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
            .await;
        assert_eq!(
            initialized.is_ok(),
            expected_ready,
            "{style}, bridge={bridge}"
        );
        if expected_ready {
            assert!(
                client
                    .list_tools(None, Some(Duration::from_secs(2)))
                    .await
                    .unwrap()
                    .tools
                    .is_empty()
            );
        }
        client.shutdown().await;
        drop(client);
        stop.cancel();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        let statuses = statuses.lock().unwrap();
        assert!(!statuses.is_empty());
        assert!(statuses.iter().all(|status| *status == expected_status));
        let methods = methods.lock().unwrap();
        assert_eq!(
            methods
                .iter()
                .filter(|method| method.as_str() == "tools/list")
                .count(),
            usize::from(expected_ready)
        );
        assert!(methods.iter().all(|method| {
            ["initialize", "notifications/initialized", "tools/list"].contains(&method.as_str())
        }));
    }
}
