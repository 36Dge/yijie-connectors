//! Builtin web tools share the existing worker and official rmcp transport.
//! Tool discovery is independent of marketplace selections and user API keys.
use crate::{
    builtin_web_executor::{error, now_ms, validate},
    builtin_web_generated as wire,
};
use rmcp::{RoleServer, ServerHandler, model::*, service::RequestContext};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;
mod scoped;
#[cfg(test)]
mod scoped_tests;
mod search_reuse;
#[derive(Default)]
struct Feedback {
    legacy_reused: bool,
    fact: Option<Value>,
}
use search_reuse::{Key, Search, Searches};

struct Call {
    thread: String,
    turn: String,
    root: String,
    root_turn: String,
    cancel: CancellationToken,
}
#[derive(Default)]
struct State {
    install: Option<Value>,
    calls: HashMap<String, Call>,
    stopped: HashSet<(String, String)>,
    searches: Searches,
    scoped: HashMap<Key, scoped::Flight>,
}
#[derive(Clone)]
pub struct WebGateway(
    Arc<Mutex<State>>,
    Arc<tokio::sync::Notify>,
    Arc<tokio::sync::Semaphore>,
);
impl Default for WebGateway {
    fn default() -> Self {
        Self(
            Arc::new(Mutex::new(State::default())),
            Arc::new(tokio::sync::Notify::new()),
            Arc::new(tokio::sync::Semaphore::new(wire::SUBJECT_CONCURRENCY)),
        )
    }
}
struct ActiveCall {
    owner: WebGateway,
    id: String,
}
impl Drop for ActiveCall {
    fn drop(&mut self) {
        if let Some(call) = self
            .owner
            .0
            .lock()
            .expect("web state")
            .calls
            .remove(&self.id)
        {
            call.cancel.cancel()
        }
    }
}
fn result(value: Value) -> CallToolResult {
    let failed = value.get("error").is_some();
    let mut r = CallToolResult::structured(value);
    r.is_error = Some(failed);
    if serde_json::to_vec(&r).map_or(true, |v| v.len() > 65536) {
        let mut bounded = CallToolResult::structured(error("response_too_large"));
        bounded.is_error = Some(true);
        return bounded;
    }
    r
}
fn origin_valid(value: &Value) -> bool {
    url::Url::parse(value.as_str().unwrap_or("")).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none()
            && u.query().is_none()
            && u.fragment().is_none()
            && matches!(u.path(), "" | "/")
    })
}
impl WebGateway {
    async fn dispatch_slot(
        &self,
        cancel: &CancellationToken,
    ) -> Result<tokio::sync::OwnedSemaphorePermit, &'static str> {
        // Transport backpressure only: Codex still owns tool choice, approvals,
        // parallel execution and cancellation. No API operation exists yet.
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err("cancelled"),
            slot = tokio::time::timeout(
                Duration::from_secs(wire::GATEWAY_QUEUE_WAIT_SECONDS),
                self.2.clone().acquire_owned(),
            ) => slot.map_err(|_| "busy")?.map_err(|_| "busy"),
        }
    }
    pub fn stop(&self) {
        let mut s = self.0.lock().expect("web state");
        for call in s.calls.values() {
            call.cancel.cancel()
        }
        s.install = None;
        s.scoped.clear();
        s.searches.clear();
    }
    pub fn control(&self, bytes: &[u8]) -> Option<Value> {
        let value: Value = serde_json::from_slice(bytes).ok()?;
        let method = value["method"].as_str()?;
        let name = match method {
            "web_worker_install" => "WebWorkerInstall",
            "web_capability_clear" => "WebCapabilityClear",
            "web_turn_revoke" => "WebTurnRevoke",
            "web_call_cancel" => "WebCallCancel",
            _ => return None,
        };
        let control = if method == "web_worker_install" {
            &value["control"]
        } else {
            &value
        };
        let epoch = control["process_epoch"].clone();
        let ack = |state: &str| json!({"state":state,"process_epoch":epoch});
        if !validate(name, &value) {
            return Some(ack("unavailable"));
        }
        let mut s = self.0.lock().expect("web state");
        if let Some(current) = &s.install
            && current["control"]["process_epoch"] != epoch
        {
            return Some(ack("unavailable"));
        }
        match method {
            "web_worker_install" => {
                let local = value["profile"] == "local_demo"
                    && value["api_origin"] == wire::LOCAL_API_ORIGIN;
                if (!origin_valid(&value["api_origin"]) && !local)
                    || !url::Url::parse(value["admission_url"].as_str().unwrap_or("")).is_ok_and(
                        |u| {
                            u.scheme() == "http"
                                && u.host_str() == Some("127.0.0.1")
                                && u.port().is_some()
                                && u.username().is_empty()
                                && u.password().is_none()
                                && u.query().is_none()
                                && u.fragment().is_none()
                        },
                    )
                {
                    return Some(ack("unavailable"));
                }
                let deadline = control["delegation"]["expires_at_unix_ms"]
                    .as_u64()
                    .unwrap_or(0);
                if deadline <= now_ms() || deadline > now_ms() + 300000 {
                    return Some(ack("unavailable"));
                }
                if let Some(current) = &s.install {
                    if current["control"]["binding_epoch"] != control["binding_epoch"]
                        || current["control"]["delegation"]["binding_id"]
                            != control["delegation"]["binding_id"]
                    {
                        return Some(ack("unavailable"));
                    }
                    if current["control"]["delegation"]["credential_revision"].as_u64()
                        >= control["delegation"]["credential_revision"].as_u64()
                    {
                        return Some(ack("already_applied"));
                    }
                    if current["control"]["delegation"]["authorization_revision"]
                        != control["delegation"]["authorization_revision"]
                    {
                        s.scoped.clear();
                        s.searches.clear();
                    }
                }
                s.install = Some(value.clone());
            }
            "web_capability_clear" => {
                if s.install
                    .as_ref()
                    .is_some_and(|i| i["control"]["binding_epoch"] != control["binding_epoch"])
                {
                    return Some(ack("unavailable"));
                }
                for call in s.calls.values() {
                    call.cancel.cancel()
                }
                s.install = None;
                s.stopped.clear();
                s.scoped.clear();
                s.searches.clear();
            }
            "web_turn_revoke" => {
                let thread = control["thread_id"].as_str().unwrap_or("");
                let turn = control["turn_id"].as_str().unwrap_or("");
                s.stopped.insert((thread.into(), turn.into()));
                s.searches.retain(|key, _| !key.belongs_to(thread, turn));
                s.scoped.retain(|key, _| !key.belongs_to(thread, turn));
                for call in s.calls.values() {
                    if (call.thread == thread && call.turn == turn)
                        || (call.root == thread && call.root_turn == turn)
                    {
                        call.cancel.cancel()
                    }
                }
                if s.stopped.len() > 1024 {
                    for call in s.calls.values() {
                        call.cancel.cancel()
                    }
                    s.install = None;
                    s.scoped.clear();
                    s.searches.clear();
                }
            }
            "web_call_cancel" => {
                if let Some(call) = s.calls.get(control["operation_id"].as_str().unwrap_or("")) {
                    call.cancel.cancel()
                }
            }
            _ => {}
        }
        self.1.notify_waiters();
        Some(ack("applied"))
    }
    async fn invoke(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
        feedback: &mut Feedback,
    ) -> Value {
        let thread = context
            .meta
            .0
            .get("threadId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let turn = context
            .meta
            .0
            .get("x-codex-turn-metadata")
            .and_then(|v| v.get("turn_id"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if thread.is_empty() || turn.is_empty() {
            return error("not_entitled");
        }
        let id = uuid::Uuid::now_v7().to_string();
        let cancellation = context.ct.child_token();
        let install = {
            let mut s = self.0.lock().expect("web state");
            let Some(config) = s.install.clone() else {
                return error("session_expired");
            };
            if s.calls.len() >= 64 || s.stopped.contains(&(thread.clone(), turn.clone())) {
                return error("busy");
            }
            if config["control"]["delegation"]["expires_at_unix_ms"]
                .as_u64()
                .unwrap_or(0)
                <= now_ms()
            {
                return error("session_expired");
            }
            s.calls.insert(
                id.clone(),
                Call {
                    thread: thread.clone(),
                    turn: turn.clone(),
                    root: String::new(),
                    root_turn: String::new(),
                    cancel: cancellation.clone(),
                },
            );
            config
        };
        let _owned = ActiveCall {
            owner: self.clone(),
            id: id.clone(),
        };
        self.execute(
            &install,
            &id,
            (&thread, &turn),
            request,
            &cancellation,
            feedback,
        )
        .await
    }
    async fn execute(
        &self,
        install: &Value,
        id: &str,
        caller: (&str, &str),
        request: CallToolRequestParams,
        cancel: &CancellationToken,
        feedback: &mut Feedback,
    ) -> Value {
        let (thread, turn) = caller;
        let mut operation = json!({"operation_id":id,"tool":request.name,"arguments":request.arguments.unwrap_or_default()});
        if !validate("OperationRequest", &operation) {
            return error("invalid_arguments");
        }
        let Ok(client) = reqwest::Client::builder()
            .no_proxy()
            .retry(reqwest::retry::never())
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(45))
            .build()
        else {
            return error("upstream_unavailable");
        };
        let probe = json!({"process_epoch":install["control"]["process_epoch"],"binding_epoch":install["control"]["binding_epoch"],"operation_id":id,"thread_id":thread,"turn_id":turn});
        let deep = operation["tool"] == "kimi_web_search_pro";
        let admission = admit(
            &client,
            install,
            &probe,
            deep,
            cancel,
            Duration::from_secs(5),
        )
        .await;
        let Ok((200, receipt)) = admission else {
            return error("not_entitled");
        };
        let scoped = deep && validate("WebAdmissionReceiptV2", &receipt);
        if (!scoped && !validate("WebAdmissionReceipt", &receipt)) || receipt["operation_id"] != id
        {
            return error("not_entitled");
        }
        let root = receipt["root_thread_id"].as_str().unwrap_or("").to_owned();
        let root_turn = receipt["root_turn_id"].as_str().unwrap_or("").to_owned();
        {
            let mut s = self.0.lock().expect("web state");
            if s.stopped.contains(&(root.clone(), root_turn.clone())) {
                cancel.cancel()
            };
            if let Some(call) = s.calls.get_mut(id) {
                call.root = root;
                call.root_turn = root_turn;
            }
        }
        if cancel.is_cancelled() || receipt["deadline_unix_ms"].as_u64().unwrap_or(0) <= now_ms() {
            return error("cancelled");
        }
        let deadline = receipt["deadline_unix_ms"].as_u64().unwrap_or(0);
        if scoped {
            let key = Key::new(install, caller, &receipt, &operation["arguments"]);
            let r = self
                .scoped_search(
                    key,
                    scoped::Request {
                        client,
                        install: install.clone(),
                        operation,
                        admission: receipt,
                        probe,
                    },
                    cancel,
                )
                .await;
            feedback.fact = Some(r.fact);
            return r.body;
        }
        let mut leader = None;
        let mut replay = false;
        if operation["tool"] == "kimi_web_search_pro" {
            let key = Key::new(install, caller, &receipt, &operation["arguments"]);
            match self.begin_search(key, id) {
                Err(failure) => return failure,
                Ok(Search::First(first)) => leader = Some(first),
                Ok(Search::Existing(original, mut done)) => {
                    let wait = Duration::from_millis(deadline.saturating_sub(now_ms()));
                    let completed = tokio::select! {
                        biased;
                        _ = cancel.cancelled() => return error("cancelled"),
                        result = tokio::time::timeout(wait, done.wait_for(Option::is_some)) => match result {
                            Ok(Ok(value)) => value.clone().expect("completed search"),
                            Ok(Err(_)) => return error("outcome_unknown"),
                            Err(_) => return error("timeout"),
                        },
                    };
                    if let Err(failure) = completed {
                        return failure;
                    }
                    operation["operation_id"] = original.into();
                    replay = true;
                }
            }
        }
        let value = self
            .submit_operation(&client, install, &operation, deadline, cancel, replay)
            .await;
        if let Some(leader) = leader {
            leader.finish(&value);
        }
        feedback.legacy_reused = replay && value.get("error").is_none();
        value
    }

    async fn submit_operation(
        &self,
        client: &reqwest::Client,
        install: &Value,
        operation: &Value,
        deadline: u64,
        cancel: &CancellationToken,
        replay: bool,
    ) -> Value {
        // A completed operation is reauthorized by the API using its original
        // ID. Reading it consumes neither a supplier slot nor another quota unit.
        let _slot = if replay {
            None
        } else {
            match self.dispatch_slot(cancel).await {
                Ok(slot) => Some(slot),
                Err(code) => return error(code),
            }
        };
        if cancel.is_cancelled() || deadline <= now_ms() {
            return error("cancelled");
        }
        let url = format!(
            "{}/v1/web-retrieval/operations",
            install["api_origin"]
                .as_str()
                .unwrap_or("")
                .trim_end_matches('/')
        );
        let token = install["control"]["delegation"]["bearer"]
            .as_str()
            .unwrap_or("");
        let mut response = post(
            client,
            &url,
            token,
            operation,
            cancel,
            Duration::from_secs(45),
        )
        .await;
        // Only a validated product 401 explicitly proving no dispatch may
        // reuse the same operation once, after Native installs a newer grant.
        if matches!(&response,Ok((401,reply)) if validate("APIError",reply) && reply["attempt_dispatched"]==false && reply["error"]["code"]=="session_expired")
        {
            let changed = self.1.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let renewed = || {
                self.0
                    .lock()
                    .expect("web state")
                    .install
                    .clone()
                    .filter(|v| {
                        v["control"]["binding_epoch"] == install["control"]["binding_epoch"]
                            && v["control"]["delegation"]["binding_id"]
                                == install["control"]["delegation"]["binding_id"]
                            && (!replay
                                || v["control"]["delegation"]["authorization_revision"]
                                    == install["control"]["delegation"]["authorization_revision"])
                            && v["control"]["delegation"]["bearer"]
                                != install["control"]["delegation"]["bearer"]
                    })
            };
            let mut fresh = renewed();
            if fresh.is_none() {
                tokio::select! {biased;_=cancel.cancelled()=>{},_=tokio::time::timeout(Duration::from_secs(5),changed)=>{fresh=renewed();}}
            }
            if let Some(fresh) = fresh {
                response = post(
                    client,
                    &url,
                    fresh["control"]["delegation"]["bearer"]
                        .as_str()
                        .unwrap_or(""),
                    operation,
                    cancel,
                    Duration::from_secs(45),
                )
                .await;
            }
        }
        match response {
            Ok((200, reply)) if validate("ExecutionResult", &reply) => {
                if cancel.is_cancelled() {
                    return error("cancelled");
                }
                if reply["operation"]["operation_id"] != operation["operation_id"] {
                    return error("invalid_response");
                }
                if let Some(body) = reply.get("result") {
                    if operation["tool"] == "kimi_web_search_pro" && body.get("error").is_none() {
                        if reply["operation"]["state"] != "completed"
                            || reply["operation"]["dispatch"] != "completed"
                        {
                            return error("outcome_unknown");
                        }
                        if !validate("ProSuccess", body) {
                            return error("invalid_response");
                        }
                    }
                    body.clone()
                } else if reply["operation"]["result_availability"] == "result_unavailable" {
                    error("result_unavailable")
                } else if reply["operation"]["state"] == "cancelled" {
                    error("cancelled")
                } else {
                    error("outcome_unknown")
                }
            }
            Ok((_, reply)) if validate("APIError", &reply) => json!({"error":reply["error"]}),
            _ => {
                if cancel.is_cancelled() {
                    error("cancelled")
                } else {
                    error("outcome_unknown")
                }
            }
        }
    }
}
async fn post(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    body: &Value,
    cancel: &CancellationToken,
    budget: Duration,
) -> Result<(u16, Value), ()> {
    exchange(
        client
            .post(url)
            .bearer_auth(token)
            .header("content-type", "application/json")
            .body(body.to_string()),
        cancel,
        budget,
    )
    .await
}
async fn admit(
    client: &reqwest::Client,
    install: &Value,
    probe: &Value,
    v2: bool,
    cancel: &CancellationToken,
    budget: Duration,
) -> Result<(u16, Value), ()> {
    let mut request = client
        .post(install["admission_url"].as_str().unwrap_or(""))
        .bearer_auth(install["admission_bearer"].as_str().unwrap_or(""))
        .header("content-type", "application/json")
        .body(probe.to_string());
    if v2 {
        request = request.header("Accept", "application/vnd.yijie.web-admission.v2+json");
    }
    exchange(request, cancel, budget).await
}
async fn exchange(
    request: reqwest::RequestBuilder,
    cancel: &CancellationToken,
    budget: Duration,
) -> Result<(u16, Value), ()> {
    if budget.is_zero() {
        return Err(());
    }
    tokio::select! {biased;_=cancel.cancelled()=>Err(()),r=async{
     let mut resp=request.timeout(budget).send().await.map_err(|_|())?;let status=resp.status().as_u16();let mut bytes=Vec::new();
     while let Some(chunk)=resp.chunk().await.map_err(|_|())?{if bytes.len()+chunk.len()>69632{return Err(())}bytes.extend_from_slice(&chunk);}
     Ok((status,serde_json::from_slice(&bytes).map_err(|_|())?))
    }=>r}
}
impl ServerHandler for WebGateway {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("yijie-web", "0.1.0"))
            .with_instructions(wire::SERVER_INSTRUCTIONS)
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let s = self.0.lock().expect("web state");
        let ready = s.install.as_ref().is_some_and(|i| {
            i["control"]["delegation"]["expires_at_unix_ms"]
                .as_u64()
                .unwrap_or(0)
                > now_ms()
        });
        Ok(ListToolsResult {
            tools: if ready {
                serde_json::from_str(wire::TOOLS_JSON)
                    .map_err(|_| ErrorData::internal_error("web tools unavailable", None))?
            } else {
                vec![]
            },
            ..Default::default()
        })
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut feedback = Feedback::default();
        let body = self.invoke(request, context, &mut feedback).await;
        Ok(gateway_receipt(body, feedback))
    }
}

fn gateway_receipt(body: Value, feedback: Feedback) -> CallToolResult {
    let mut receipt = result(body);
    if let Some(fact) = feedback.fact {
        let mut meta = Meta::new();
        if matches!(fact["relation"].as_str(), Some("joined" | "reused")) {
            meta.insert("yijie/search-reused".into(), true.into());
        }
        meta.insert("ai.yijie/deep-search-receipt".into(), fact);
        receipt.meta = Some(meta);
        if serde_json::to_vec(&receipt).map_or(true, |v| v.len() > 65536) {
            let mut meta = receipt.meta.take();
            if let Some(fact) = meta
                .as_mut()
                .and_then(|m| m.get_mut("ai.yijie/deep-search-receipt"))
            {
                fact["availability"] = json!("unavailable");
            }
            receipt = result(error("response_too_large"));
            receipt.meta = meta;
        }
    } else if feedback.legacy_reused && receipt.is_error != Some(true) {
        let mut meta = Meta::new();
        meta.insert("yijie/search-reused".into(), Value::Bool(true));
        receipt.meta = Some(meta);
        if serde_json::to_vec(&receipt).map_or(true, |v| v.len() > 65536) {
            receipt.meta = None;
        }
    }
    receipt
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn builtin_dispatch_wait_reuses_capacity_and_honors_native_cancellation() {
        let gateway = WebGateway::default();
        let cancel = CancellationToken::new();
        let first = gateway.dispatch_slot(&cancel).await.unwrap();
        let second = gateway.dispatch_slot(&cancel).await.unwrap();
        let mut waiting = Box::pin(gateway.dispatch_slot(&cancel));
        assert!(futures::poll!(waiting.as_mut()).is_pending());
        drop(first);
        let third = waiting.await.unwrap();
        let queued_cancel = CancellationToken::new();
        let mut cancelled = Box::pin(gateway.dispatch_slot(&queued_cancel));
        assert!(futures::poll!(cancelled.as_mut()).is_pending());
        queued_cancel.cancel();
        assert!(matches!(cancelled.await, Err("cancelled")));
        drop((second, third));
        assert_eq!(gateway.2.available_permits(), wire::SUBJECT_CONCURRENCY);
    }
    #[tokio::test]
    async fn builtin_dispatch_wait_is_bounded_without_retrying() {
        let gateway = WebGateway::default();
        let cancel = CancellationToken::new();
        let _first = gateway.dispatch_slot(&cancel).await.unwrap();
        let _second = gateway.dispatch_slot(&cancel).await.unwrap();
        assert!(matches!(gateway.dispatch_slot(&cancel).await, Err("busy")));
        assert_eq!(gateway.2.available_permits(), 0);
    }
    #[test]
    fn builtin_rotation_and_root_stop_share_worker_control_without_selection() {
        let gateway = WebGateway::default();
        let process = uuid::Uuid::new_v4().to_string();
        let binding = uuid::Uuid::new_v4().to_string();
        let mut request = json!({"method":"web_worker_install","control":{"method":"web_capability_install","process_epoch":process,"binding_epoch":binding,"delegation":{"binding_id":uuid::Uuid::new_v4().to_string(),"grant_id":uuid::Uuid::new_v4().to_string(),"bearer":"a".repeat(64),"expires_at_unix_ms":now_ms()+299000,"authorization_revision":1,"credential_revision":1,"audience":"builtin-web-retrieval/1"}},"api_origin":"https://api.example","admission_url":"http://127.0.0.1:18889/internal/web-retrieval/admit","admission_bearer":"b".repeat(64)});
        assert_eq!(
            gateway
                .control(&serde_json::to_vec(&request).unwrap())
                .unwrap()["state"],
            "applied"
        );
        let cancel = CancellationToken::new();
        gateway.0.lock().unwrap().calls.insert(
            "operation".into(),
            Call {
                thread: "child".into(),
                turn: "child-turn".into(),
                root: "root".into(),
                root_turn: "root-turn".into(),
                cancel: cancel.clone(),
            },
        );
        request["control"]["delegation"]["credential_revision"] = json!(2);
        request["control"]["delegation"]["grant_id"] = json!(uuid::Uuid::new_v4().to_string());
        request["control"]["delegation"]["bearer"] = json!("c".repeat(64));
        assert_eq!(
            gateway
                .control(&serde_json::to_vec(&request).unwrap())
                .unwrap()["state"],
            "applied"
        );
        assert!(!cancel.is_cancelled());
        let revoke = json!({"method":"web_turn_revoke","process_epoch":process,"thread_id":"root","turn_id":"root-turn"});
        assert_eq!(
            gateway
                .control(&serde_json::to_vec(&revoke).unwrap())
                .unwrap()["state"],
            "applied"
        );
        assert!(cancel.is_cancelled());
        let clear = json!({"method":"web_capability_clear","process_epoch":process,"binding_epoch":binding});
        assert_eq!(
            gateway
                .control(&serde_json::to_vec(&clear).unwrap())
                .unwrap()["state"],
            "applied"
        );
        assert!(gateway.0.lock().unwrap().install.is_none());
    }
}
