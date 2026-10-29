//! Ordinary in-process responses only: no account, Keyring, listener, or network.
use super::*;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

struct OrdinaryHttp {
    requests: Mutex<Vec<HttpRequestParams>>,
    calls: Arc<AtomicUsize>,
    status: u16,
    response: Value,
    cancel_after_send: Option<CancellationToken>,
}
impl Transport for OrdinaryHttp {
    fn request(&self, request: HttpRequestParams) -> HttpFuture<'_> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(cancel) = &self.cancel_after_send {
                cancel.cancel();
            }
            Ok(HttpReply {
                status: self.status,
                body: serde_json::to_vec(&self.response).unwrap(),
            })
        })
    }
}
fn setup(response: Value) -> (Arc<Backend>, Arc<OrdinaryHttp>) {
    let http = Arc::new(OrdinaryHttp {
        requests: Default::default(),
        calls: Arc::new(AtomicUsize::new(0)),
        status: 200,
        response,
        cancel_after_send: None,
    });
    let credential = serde_json::from_value(json!({"serviceId": wire::SERVICE_ID, "name":"Authorization", "value":"Bearer ordinary-kimi-test-value"})).unwrap();
    (Backend::new(credential, http.clone()), http)
}
pub(crate) fn local_backend() -> (Arc<Backend>, Arc<AtomicUsize>) {
    let (backend, http) = setup(
        json!({"search_results":[{"title":"公开文档", "url":"https://example.com/docs", "snippet":"普通摘要"}]}),
    );
    (backend, http.calls.clone())
}
fn permit() -> Permit {
    Permit {
        tool: String::new(),
        arguments: Value::Null,
        request: CancellationToken::new(),
        lease: CancellationToken::new(),
        backend: CancellationToken::new(),
        deadline: Instant::now() + Duration::from_secs(2),
    }
}
fn args(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}
fn value(result: &CallToolResult) -> &Value {
    result.structured_content.as_ref().unwrap()
}
fn consistent(result: &CallToolResult) {
    assert!(fits(result));
    assert_eq!(result.content.len(), 1);
    let text = result.content[0].raw.as_text().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&text.text).unwrap(),
        *value(result)
    );
}

#[test]
fn canonical_descriptors_and_normal_inputs_match_frozen_contract() {
    let tools = descriptors();
    assert_eq!(
        serde_json::to_value(&tools).unwrap(),
        serde_json::from_str::<Value>(wire::TOOLS_JSON).unwrap()
    );
    for tool in tools {
        assert_eq!(
            tool.annotations.as_ref().unwrap().read_only_hint,
            Some(true)
        );
        assert_eq!(
            tool.annotations.as_ref().unwrap().idempotent_hint,
            Some(false)
        );
        assert!(tool.output_schema.is_some());
    }
    let input = Arguments::parse(
        wire::SEARCH_TOOL_NAME,
        &args(json!({"query":"公开产品文档"})),
    )
    .unwrap();
    assert_eq!(
        input.body(),
        json!({"text_query":"公开产品文档","limit":5,"timeout_seconds":10,"include_content":false})
    );
    let input = Arguments::parse(
        wire::FETCH_TOOL_NAME,
        &args(json!({"url":"https://example.com/docs?a=1"})),
    )
    .unwrap();
    assert_eq!(input.body(), json!({"url":"https://example.com/docs?a=1"}));
    assert!(
        Arguments::parse(
            wire::SEARCH_TOOL_NAME,
            &args(json!({"query":" ","limit":5}))
        )
        .is_err()
    );
    assert!(
        Arguments::parse(
            wire::SEARCH_TOOL_NAME,
            &args(json!({"query":"文档","limit":21}))
        )
        .is_err()
    );
}

#[test]
fn ordinary_search_empty_optional_fields_and_source_order_are_preserved() {
    let result = normalize_search(&json!({"search_results":[]}), 5).unwrap();
    assert_eq!(value(&result), &json!({"results":[],"truncated":false}));
    consistent(&result);
    let result = normalize_search(
        &json!({"search_results":[{"url":"https://example.com/first"},
        {"title":"Second","url":"https://example.com/second","snippet":"Second text"}]}),
        1,
    )
    .unwrap();
    assert_eq!(
        value(&result),
        &json!({"results":[{"title":"","url":"https://example.com/first","snippet":""}],"truncated":true})
    );
    consistent(&result);
}

#[test]
fn ordinary_long_markdown_and_snippets_fit_the_entire_mcp_envelope() {
    let text = "正常中文正文，带引号\"与换行\n".repeat(9000);
    let result = normalize_fetch(
        &json!({"url":"https://example.com/docs","title":"公开文档","markdown":text}),
    )
    .unwrap();
    consistent(&result);
    assert_eq!(value(&result)["truncated"], true);
    assert!(text.starts_with(value(&result)["markdown"].as_str().unwrap()));
    let source = (0..20).map(|i| json!({"title":format!("Article {i}"),"url":format!("https://example.com/{i}"),"snippet":text})).collect::<Vec<_>>();
    let result = normalize_search(&json!({"search_results":source}), 20).unwrap();
    consistent(&result);
    assert_eq!(value(&result)["truncated"], true);
    assert_eq!(value(&result)["results"][0]["url"], "https://example.com/0");
    assert_eq!(
        normalize_fetch(&json!({"url":"https://example.com/docs","title":"","markdown":"  \n"}))
            .unwrap_err(),
        ErrorCode::NoContent
    );
}

#[test]
fn ordinary_stream_budget_and_status_mapping_do_not_infer_success() {
    let mut bytes = Vec::new();
    append_chunk(&mut bytes, &vec![b'a'; wire::MAX_RESPONSE_BYTES]).unwrap();
    assert_eq!(
        append_chunk(&mut bytes, b"a"),
        Err(ErrorCode::ResponseTooLarge)
    );
    assert_eq!(bytes.len(), wire::MAX_RESPONSE_BYTES);
    for (status, code) in [
        (401, ErrorCode::AuthenticationFailed),
        (403, ErrorCode::PermissionDenied),
        (429, ErrorCode::RateLimited),
        (408, ErrorCode::Timeout),
        (500, ErrorCode::Unavailable),
    ] {
        assert_eq!(status_error(status, false), Err(code));
        let error = error_result(code);
        consistent(&error);
        assert_eq!(error.is_error, Some(true));
        serde_json::from_value::<wire::ErrorOutput>(value(&error).clone())
            .unwrap()
            .validate()
            .unwrap();
    }
    assert_eq!(status_error(404, true), Err(ErrorCode::NoContent));
    assert_eq!(status_error(404, false), Err(ErrorCode::Unavailable));
}

#[tokio::test]
async fn normal_local_preparation_never_requests_models_or_business_tools() {
    let (backend, http) = setup(json!({"object":"list","data":[]}));
    backend.prepare().unwrap();
    assert!(http.requests.lock().unwrap().is_empty());
    backend.close().await;
    assert!(matches!(backend.prepare(), Err(Failure::Cancelled)));
}

#[tokio::test]
async fn normal_approved_search_uses_fixed_rest_once_and_closes() {
    let (backend, http) = setup(
        json!({"search_results":[{"title":"Guide","url":"https://example.com/guide","snippet":"Public guide"}]}),
    );
    let arguments = args(json!({"query":"public guide","limit":3}));
    let review = backend.review(wire::SEARCH_TOOL_NAME, &arguments).unwrap();
    assert_eq!(review.risk, "read");
    assert!(review.summary.contains("按次计费"));
    assert!(review.schema_digest.is_some());
    let result = backend
        .execute(
            wire::SEARCH_TOOL_NAME,
            arguments,
            permit(),
            Diagnostic::default(),
        )
        .await;
    consistent(&result);
    assert_eq!(result.is_error, Some(false));
    let requests = http.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "https://api.moonshot.cn/v1/tools/search");
    assert_eq!(requests[0].method, "POST");
    assert!(requests[0].timeout_ms.unwrap() <= 2000);
    let body: Value = serde_json::from_slice(&requests[0].body.as_ref().unwrap().0).unwrap();
    assert_eq!(
        body,
        json!({"text_query":"public guide","limit":3,"timeout_seconds":10,"include_content":false})
    );
    drop(requests);
    backend.close().await;
    assert!(!backend.available());
}

#[tokio::test]
async fn ordinary_fetch_and_user_cancellation_keep_one_request_limit() {
    let (backend, http) = setup(
        json!({"url":"https://example.com/guide","title":"Guide","markdown":"# Guide\nPublic text"}),
    );
    let cancelled = permit();
    cancelled.request.cancel();
    let result = backend
        .execute(
            wire::FETCH_TOOL_NAME,
            args(json!({"url":"https://example.com/guide"})),
            cancelled,
            Diagnostic::default(),
        )
        .await;
    assert_eq!(value(&result)["error"]["code"], "cancelled");
    assert_eq!(http.calls.load(Ordering::SeqCst), 0);
    let result = backend
        .execute(
            wire::FETCH_TOOL_NAME,
            args(json!({"url":"https://example.com/guide"})),
            permit(),
            Diagnostic::default(),
        )
        .await;
    consistent(&result);
    assert_eq!(result.is_error, Some(false));
    assert_eq!(http.calls.load(Ordering::SeqCst), 1);
    let requests = http.requests.lock().unwrap();
    assert_eq!(requests[0].url, "https://api.moonshot.cn/v1/tools/fetch");
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[0].body.as_ref().unwrap().0).unwrap(),
        json!({"url":"https://example.com/guide"})
    );
    drop(requests);
    backend.close().await;
}

#[tokio::test]
async fn ordinary_sent_cancellation_discards_result_without_resending() {
    let permitted = permit();
    let cancel = permitted.request.clone();
    let http = Arc::new(OrdinaryHttp {
        requests: Default::default(),
        calls: Arc::new(AtomicUsize::new(0)),
        status: 200,
        response: json!({"search_results":[]}),
        cancel_after_send: Some(cancel),
    });
    let credential=serde_json::from_value(json!({"serviceId":wire::SERVICE_ID,"name":"Authorization","value":"Bearer ordinary-kimi-test-value"})).unwrap();
    let backend = Backend::new(credential, http.clone());
    let result = backend
        .execute(
            wire::SEARCH_TOOL_NAME,
            args(json!({"query":"public guide"})),
            permitted,
            Diagnostic::default(),
        )
        .await;
    assert_eq!(value(&result)["error"]["code"], "cancelled");
    assert_eq!(http.calls.load(Ordering::SeqCst), 1);
    backend.close().await;
}

#[tokio::test]
async fn ordinary_rate_limit_is_returned_without_another_request() {
    let http = Arc::new(OrdinaryHttp {
        requests: Default::default(),
        calls: Arc::new(AtomicUsize::new(0)),
        status: 429,
        response: json!({"message":"ordinary rate limit"}),
        cancel_after_send: None,
    });
    let credential = serde_json::from_value(json!({"serviceId":wire::SERVICE_ID,"name":"Authorization","value":"Bearer ordinary-kimi-test-value"})).unwrap();
    let backend = Backend::new(credential, http.clone());
    let result = backend
        .execute(
            wire::SEARCH_TOOL_NAME,
            args(json!({"query":"public guide"})),
            permit(),
            Diagnostic::default(),
        )
        .await;
    assert_eq!(value(&result)["error"]["code"], "rate_limited");
    assert_eq!(http.calls.load(Ordering::SeqCst), 1);
    assert!(
        !serde_json::to_string(&result)
            .unwrap()
            .contains("ordinary rate limit")
    );
    backend.close().await;
}

#[tokio::test]
async fn ordinary_http_client_streams_once_and_server_closes_gracefully() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let route = axum::Router::new().route(
        "/ordinary",
        axum::routing::post(move || {
            let observed = observed.clone();
            async move {
                observed.fetch_add(1, Ordering::SeqCst);
                axum::Json(json!({"search_results":[]}))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let close = CancellationToken::new();
    let closing = close.clone();
    let owner = tokio::spawn(async move {
        axum::serve(listener, route)
            .with_graceful_shutdown(closing.cancelled_owned())
            .await
    });
    // Only this transport-level test uses an owned loopback listener. Production
    // Backend::request admits the three fixed Kimi HTTPS endpoints exclusively.
    let result = StreamingHttp
        .request(HttpRequestParams {
            method: "POST".into(),
            url: format!("http://{address}/ordinary"),
            headers: vec![],
            body: Some(b"{}".to_vec().into()),
            timeout_ms: Some(2000),
            redirect_policy: HttpRedirectPolicy::Stop,
            request_id: uuid::Uuid::new_v4().to_string(),
            stream_response: true,
        })
        .await;
    close.cancel();
    owner.await.unwrap().unwrap();
    let reply = result.unwrap();
    assert_eq!(reply.status, 200);
    assert_eq!(
        serde_json::from_slice::<Value>(&reply.body).unwrap(),
        json!({"search_results":[]})
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn kimi_content_keeps_partial_sources_and_distinguishes_absence_from_clipping() {
    let source = json!({"search_results":[
        {"title":"No body","url":"https://example.org/empty","snippet":"summary"},
        {"title":"Body","url":"https://example.org/body","snippet":"summary","text":"正文".repeat(40000)},
        {"title":"Missing source","url":"","text":"unusable"}
    ]});
    let result = content::normalize(&source, 5, false).unwrap();
    consistent(&result);
    assert_eq!(value(&result)["dropped_results"], 1);
    assert_eq!(
        value(&result)["results"][0]["content_status"],
        "unavailable"
    );
    assert_eq!(value(&result)["results"][1]["content_status"], "truncated");
    assert_eq!(
        value(&result)["results"][1]["url"],
        "https://example.org/body"
    );
    assert!(content::normalize(&json!({"search_results":[{"url":""}]}), 5, true).is_err());
    let empty = content::normalize(&json!({"search_results":[]}), 5, true).unwrap();
    assert_eq!(
        value(&empty),
        &json!({"results":[],"truncated":false,"dropped_results":0})
    );
}

#[test]
fn kimi_pro_constraints_and_mixed_page_types_remain_local() {
    let input=Arguments::parse(wire::PRO_TOOL_NAME,&args(json!({"query":"docs","sites":["example.org"],"time_window":{"start":"2024-02-29","end":"2026-10"}}))).unwrap();
    assert_eq!(input.path(), "/v1/tools/search_pro");
    assert_eq!(input.body()["timeout_seconds"], 30);
    assert_eq!(input.body()["sites"], json!(["example.org"]));
    assert!(
        Arguments::parse(
            wire::PRO_TOOL_NAME,
            &args(json!({"query":"docs","time_window":{"start":"2025-02-29"}}))
        )
        .is_err()
    );
    let result = content::normalize(
        &json!({"search_results":[
            {"url":"https://example.org/","chunks":[{"text":"ordinary content","score":0.5}]},
            {"url":"https://example.org/null","chunks":null}
        ]}),
        5,
        true,
    )
    .unwrap();
    consistent(&result);
    assert_eq!(value(&result)["results"][0]["content_status"], "available");
    assert_eq!(value(&result)["dropped_results"], 1);
    assert_eq!(value(&result)["truncated"], true);
}

#[tokio::test]
async fn kimi_full_budget_rejects_before_http_but_legacy_keeps_remaining_time() {
    let (backend, http) = setup(json!({"search_results":[]}));
    let rejected = backend
        .execute(
            wire::PRO_TOOL_NAME,
            args(json!({"query":"docs"})),
            permit(),
            Diagnostic::default(),
        )
        .await;
    assert_eq!(value(&rejected)["error"]["code"], "timeout");
    assert_eq!(http.calls.load(Ordering::SeqCst), 0);
    let legacy = backend
        .execute(
            wire::SEARCH_TOOL_NAME,
            args(json!({"query":"docs"})),
            permit(),
            Diagnostic::default(),
        )
        .await;
    assert_eq!(legacy.is_error, Some(false));
    assert_eq!(http.calls.load(Ordering::SeqCst), 1);
    backend.close().await;
}

#[tokio::test]
async fn kimi_pro_and_body_send_one_canonical_request_with_a_long_budget() {
    for (tool, arguments, response) in [
        (
            wire::PRO_TOOL_NAME,
            json!({"query":"docs"}),
            json!({"search_results":[{"url":"https://example.org/","chunks":[{"text":"body","score":1.2}]}]}),
        ),
        (
            wire::SEARCH_TOOL_NAME,
            json!({"query":"docs","include_content":true}),
            json!({"search_results":[{"url":"https://example.org/","text":"body"}]}),
        ),
    ] {
        let (backend, http) = setup(response);
        let mut admitted = permit();
        admitted.deadline = Instant::now() + Duration::from_secs(40);
        let result = backend
            .execute(tool, args(arguments), admitted, Diagnostic::default())
            .await;
        assert_ne!(result.is_error, Some(true));
        consistent(&result);
        assert_eq!(value(&result)["results"][0]["content_status"], "available");
        assert_eq!(http.calls.load(Ordering::SeqCst), 1);
        backend.close().await;
    }
}
