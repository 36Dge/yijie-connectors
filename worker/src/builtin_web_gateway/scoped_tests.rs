use super::*;
use axum::{
    Json, Router,
    extract::{Path, State as HttpState},
    routing::{get, post},
};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Default)]
struct Peer {
    scope: Uuid,
    sends: AtomicUsize,
    reads: AtomicUsize,
    active: AtomicUsize,
    peak: AtomicUsize,
    entered: Notify,
    release: Notify,
}

async fn peer() -> (
    WebGateway,
    Value,
    Arc<Peer>,
    CancellationToken,
    tokio::task::JoinHandle<()>,
) {
    let state = Arc::new(Peer {
        scope: Uuid::new_v4(),
        ..Default::default()
    });
    async fn admit(HttpState(s): HttpState<Arc<Peer>>, Json(q): Json<Value>) -> Json<Value> {
        Json(
            json!({"root_thread_id":q["thread_id"],"root_turn_id":q["turn_id"],"operation_id":q["operation_id"],"scope_id":s.scope,"deadline_unix_ms":now_ms()+45000}),
        )
    }
    fn response(id: &str, relation: &str) -> Value {
        json!({"call_id":id,"receipt":{"operation_id":id,"version":2,"relation":relation,"dispatch":"responded","outcome":"empty","availability":"available"},"result":{"results":[],"dropped_results":0,"truncated":false}})
    }
    async fn search(HttpState(s): HttpState<Arc<Peer>>, Json(q): Json<Value>) -> Json<Value> {
        s.sends.fetch_add(1, SeqCst);
        let active = s.active.fetch_add(1, SeqCst) + 1;
        s.peak.fetch_max(active, SeqCst);
        let query = q["request"]["arguments"]["query"].as_str().unwrap();
        s.entered.notify_one();
        if query == "held" {
            s.release.notified().await
        }
        if query.starts_with("slow") {
            tokio::time::sleep(Duration::from_millis(5500)).await
        }
        s.active.fetch_sub(1, SeqCst);
        Json(response(
            q["request"]["operation_id"].as_str().unwrap(),
            "first",
        ))
    }
    async fn read(
        HttpState(s): HttpState<Arc<Peer>>,
        Path((_scope, id)): Path<(String, String)>,
    ) -> Json<Value> {
        s.reads.fetch_add(1, SeqCst);
        Json(response(&id, "reused"))
    }
    let router = Router::new()
        .route("/admit", post(admit))
        .route("/v2/web-retrieval/searches", post(search))
        .route("/v2/web-retrieval/searches/{scope}/{id}", get(read))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let install = json!({"api_origin":origin,"admission_url":format!("{origin}/admit"),"admission_bearer":"a".repeat(64),"control":{"binding_epoch":Uuid::new_v4(),"process_epoch":Uuid::new_v4(),"delegation":{"binding_id":Uuid::new_v4(),"authorization_revision":1,"bearer":"b".repeat(64)}}});
    let gateway = WebGateway::default();
    gateway.0.lock().unwrap().install = Some(install.clone());
    let shutdown = CancellationToken::new();
    let token = shutdown.clone();
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(token.cancelled_owned())
            .await
            .unwrap()
    });
    (gateway, install, state, shutdown, task)
}
async fn run(g: &WebGateway, i: &Value, query: &str, token: &CancellationToken) -> Value {
    let mut meta = Feedback::default();
    let req =
        serde_json::from_value(json!({"name":"kimi_web_search_pro","arguments":{"query":query}}))
            .unwrap();
    let body = g
        .execute(
            i,
            &Uuid::now_v7().to_string(),
            ("thread", "turn"),
            req,
            token,
            &mut meta,
        )
        .await;
    serde_json::to_value(gateway_receipt(body, meta)).unwrap()
}
fn fact(receipt: &Value) -> &Value {
    &receipt["_meta"]["ai.yijie/deep-search-receipt"]
}

#[tokio::test]
async fn scoped_waiter_cancel_keeps_joined_owner_and_reads_are_free() {
    let (g, i, s, shutdown, task) = peer().await;
    let first = CancellationToken::new();
    let owner = tokio::spawn({
        let (g, i, t) = (g.clone(), i.clone(), first.clone());
        async move { run(&g, &i, "held", &t).await }
    });
    s.entered.notified().await;
    let follower = tokio::spawn({
        let (g, i) = (g.clone(), i.clone());
        async move { run(&g, &i, "held", &CancellationToken::new()).await }
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while !g
            .0
            .lock()
            .unwrap()
            .scoped
            .values()
            .any(|f| f.waiter_count() == 2)
        {
            tokio::task::yield_now().await
        }
    })
    .await
    .unwrap();
    first.cancel();
    assert_eq!(
        owner.await.unwrap()["structuredContent"]["error"]["code"],
        "cancelled"
    );
    s.release.notify_one();
    let joined = follower.await.unwrap();
    assert_eq!(fact(&joined)["relation"], "joined");
    let reused = run(&g, &i, "held", &CancellationToken::new()).await;
    assert_eq!(fact(&reused)["relation"], "reused");
    assert_eq!(fact(&reused)["operation_id"], fact(&joined)["operation_id"]);
    assert_eq!(s.sends.load(SeqCst), 1);
    assert_eq!(s.reads.load(SeqCst), 2);
    shutdown.cancel();
    task.await.unwrap();
}

#[tokio::test]
async fn scoped_four_requests_keep_two_slots_and_budgeted_queue() {
    let (g, i, s, shutdown, task) = peer().await;
    let t = CancellationToken::new();
    let pending = tokio::spawn({
        let (g, i, t) = (g.clone(), i.clone(), t.clone());
        async move { tokio::join!(run(&g, &i, "slow-a", &t), run(&g, &i, "slow-b", &t)) }
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while s.active.load(SeqCst) != 2 {
            tokio::task::yield_now().await
        }
    })
    .await
    .unwrap();
    let (third, fourth) = tokio::join!(run(&g, &i, "quick-c", &t), run(&g, &i, "quick-d", &t));
    let (first, second) = pending.await.unwrap();
    for v in [first, second, third, fourth] {
        assert_eq!(fact(&v)["dispatch"], "responded")
    }
    assert_eq!(s.sends.load(SeqCst), 4);
    assert_eq!(s.peak.load(SeqCst), 2);
    let one = g.2.clone().acquire_owned().await.unwrap();
    let two = g.2.clone().acquire_owned().await.unwrap();
    let queued = CancellationToken::new();
    let waiting = tokio::spawn({
        let (g, i, t) = (g.clone(), i.clone(), queued.clone());
        async move { run(&g, &i, "cancelled in queue", &t).await }
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while g.0.lock().unwrap().scoped.len() < 5 {
            tokio::task::yield_now().await
        }
    })
    .await
    .unwrap();
    queued.cancel();
    let cancelled = waiting.await.unwrap();
    assert_eq!(fact(&cancelled)["dispatch"], "not_dispatched");
    assert_eq!(cancelled["structuredContent"]["error"]["code"], "cancelled");
    let busy = run(&g, &i, "blocked", &t).await;
    assert_eq!(fact(&busy)["dispatch"], "not_dispatched");
    assert_eq!(fact(&busy)["error_code"], "busy");
    drop((one, two));
    let retry = run(&g, &i, "blocked", &t).await;
    assert_eq!(fact(&retry)["relation"], "first");
    assert_eq!(s.sends.load(SeqCst), 5);
    shutdown.cancel();
    task.await.unwrap();
}

// A recorded provider response never contacts that provider. Explicit local
// product-API/PostgreSQL fixtures supply the authority and the three run sets.
#[tokio::test]
#[ignore = "requires a recorded local run and an isolated product API"]
async fn recorded_scoped_calls_from_file() {
    use std::path::PathBuf;
    let dir = PathBuf::from(std::env::var("SCOPED_REPLAY_DIRECTORY").unwrap());
    let name = std::env::var("SCOPED_REPLAY_SET").unwrap();
    let origin = std::env::var("SCOPED_REPLAY_ORIGIN").unwrap();
    assert_eq!(
        url::Url::parse(&origin).unwrap().host_str(),
        Some("127.0.0.1")
    );
    let data: Value =
        serde_json::from_slice(&std::fs::read(dir.join("recorded.json")).unwrap()).unwrap();
    let g = WebGateway::default();
    let i = json!({"api_origin":origin,"admission_url":format!("{origin}/admit"),"admission_bearer":"a".repeat(64),"control":{"binding_epoch":Uuid::new_v4(),"process_epoch":Uuid::new_v4(),"delegation":{"binding_id":Uuid::new_v4(),"authorization_revision":1,"bearer":std::env::var("SCOPED_REPLAY_GRANT").unwrap()}}});
    g.0.lock().unwrap().install = Some(i.clone());
    let mut output = Vec::new();
    for row in data[&name].as_array().unwrap() {
        let req =
            serde_json::from_value(json!({"name":"kimi_web_search_pro","arguments":row["args"]}))
                .unwrap();
        // Reproduce the recorded saturated slots locally for the five previous
        // not-dispatched rows only. The separate capacity test exercises 8 s.
        let (one, two) = if row["body"]["error"]["code"] == "busy" {
            (
                Some(g.2.clone().acquire_owned().await.unwrap()),
                Some(g.2.clone().acquire_owned().await.unwrap()),
            )
        } else {
            (None, None)
        };
        let mut feedback = Feedback::default();
        let t = CancellationToken::new();
        let body = g
            .execute(
                &i,
                &Uuid::now_v7().to_string(),
                ("thread", &name),
                req,
                &t,
                &mut feedback,
            )
            .await;
        drop((one, two));
        let r = serde_json::to_value(gateway_receipt(body, feedback)).unwrap();
        assert!(fact(&r)["version"] == 2);
        if row["body"].get("error").is_none() {
            assert_eq!(
                r["structuredContent"]["results"].as_array().map(Vec::len),
                row["body"]["results"].as_array().map(Vec::len)
            );
        } else {
            assert_eq!(
                r["structuredContent"]["error"]["code"],
                row["body"]["error"]["code"]
            );
        }
        output.push(json!({"id":row["logical"],"type":"mcpToolCall","status":"completed","server":"yijie_web","tool":"kimi_web_search_pro","arguments":row["args"],"result":r}));
    }
    std::fs::write(
        dir.join(format!("mcp-{name}.json")),
        serde_json::to_vec(&output).unwrap(),
    )
    .unwrap();
}
