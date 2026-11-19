//! A local flight only coordinates waiters. The product operation remains the
//! durable authority, including after a worker rebind or result eviction.
use super::*;
use tokio::sync::{oneshot, watch};

pub(super) struct Flight {
    owner: String,
    cancel: CancellationToken,
    waiters: usize,
    submitted: bool,
    done: watch::Receiver<Option<Value>>,
}
#[cfg(test)]
impl Flight {
    pub(super) fn waiter_count(&self) -> usize {
        self.waiters
    }
}
impl Drop for Flight {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
struct Subscription {
    gateway: WebGateway,
    key: Key,
    owner: String,
}
impl Subscription {
    fn cancelled(&self, relation: &str) -> Reply {
        let state = self.gateway.0.lock().expect("web state");
        let mut reply = Reply::failure("cancelled", "uncertain", None);
        if let Some(flight) = state
            .scoped
            .get(&self.key)
            .filter(|f| f.owner == self.owner)
        {
            if flight.waiters == 1 {
                flight.cancel.cancel();
                if !flight.submitted {
                    reply = Reply::failure("cancelled", "not_dispatched", None);
                }
            }
            if let Some(original) = flight
                .done
                .borrow()
                .as_ref()
                .and_then(|v| v.get("operation_id"))
            {
                reply.fact["operation_id"] = original.clone();
            }
        }
        reply.fact["relation"] = json!(relation);
        reply
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        let mut s = self.gateway.0.lock().expect("web state");
        if let Some(f) = s.scoped.get_mut(&self.key)
            && f.owner == self.owner
        {
            f.waiters -= 1;
            if f.waiters == 0 && f.done.borrow().is_none() {
                f.cancel.cancel();
            }
        }
    }
}
#[derive(Clone)]
pub(super) struct Request {
    pub client: reqwest::Client,
    pub install: Value,
    pub operation: Value,
    pub admission: Value,
    pub probe: Value,
}
pub(super) struct Reply {
    pub body: Value,
    pub fact: Value,
}
impl Reply {
    fn failure(code: &str, dispatch: &str, id: Option<&str>) -> Self {
        let mut fact = json!({"version":2,"relation":"first","dispatch":dispatch,"outcome":if dispatch=="not_dispatched"{"failed"}else{"unknown"},"availability":"available","error_code":code});
        if let Some(id) = id {
            fact["operation_id"] = json!(id)
        }
        Self {
            body: error(code),
            fact,
        }
    }
    fn accepts(wire: &Value, call: &str, reference: Option<&str>) -> bool {
        if !validate("DeepSearchResponse", wire) || wire["call_id"] != call {
            return false;
        }
        let fact = &wire["receipt"];
        let Some(id) = fact["operation_id"].as_str() else {
            return false;
        };
        if (fact["relation"] == "first" && id != call)
            || (reference.is_some() && fact["relation"] != "reused")
        {
            return false;
        }
        match wire.get("result") {
            None => fact["availability"] != "available",
            Some(body) if body.get("error").is_some() => {
                fact["availability"] == "available" && validate("AppErrorOutput", body)
            }
            Some(body) => {
                if fact["availability"] != "available" || !validate("ProSuccess", body) {
                    return false;
                }
                let outcome = if body["truncated"] == true || body["dropped_results"] != 0 {
                    "partial"
                } else if body["results"].as_array().is_some_and(Vec::is_empty) {
                    "empty"
                } else {
                    "results"
                };
                fact["outcome"] == outcome
            }
        }
    }

    fn from_wire(wire: Value) -> Self {
        let fact = wire["receipt"].clone();
        let body =
            wire.get("result").cloned().unwrap_or_else(|| {
                error(fact["error_code"].as_str().unwrap_or(
                    if fact["availability"] == "unavailable" {
                        "result_unavailable"
                    } else {
                        "outcome_unknown"
                    },
                ))
            });
        Self { body, fact }
    }
}
impl Request {
    fn deadline(&self) -> u64 {
        self.admission["deadline_unix_ms"].as_u64().unwrap_or(0)
    }
    fn remaining(&self) -> Duration {
        Duration::from_millis(self.deadline().saturating_sub(now_ms()))
    }
    fn id(&self) -> &str {
        self.operation["operation_id"]
            .as_str()
            .expect("validated id")
    }
    fn scope(&self) -> &str {
        self.admission["scope_id"]
            .as_str()
            .expect("validated scope")
    }
    fn origin(&self) -> &str {
        self.install["api_origin"]
            .as_str()
            .expect("installed origin")
            .trim_end_matches('/')
    }
    fn grant(&self, gateway: &WebGateway) -> Option<Value> {
        let state = gateway.0.lock().expect("web state");
        let installed = state.install.as_ref()?;
        let initial = &self.install["control"];
        let current = &installed["control"];
        let original = &initial["delegation"];
        let grant = &current["delegation"];
        (current["process_epoch"] == initial["process_epoch"]
            && current["binding_epoch"] == initial["binding_epoch"]
            && grant["binding_id"] == original["binding_id"]
            && grant["authorization_revision"] == original["authorization_revision"])
            .then(|| grant.clone())
    }
    async fn call(
        &self,
        gateway: &WebGateway,
        reference: Option<&str>,
        cancel: &CancellationToken,
    ) -> Reply {
        let mut refreshed = false;
        loop {
            let Some(grant) = self.grant(gateway) else {
                return Reply::failure("not_entitled", "uncertain", reference);
            };
            let token = grant["bearer"].as_str().unwrap_or("");
            let id = reference.unwrap_or(self.id());
            if cancel.is_cancelled() || self.remaining().is_zero() {
                return Reply::failure("cancelled", "uncertain", Some(id));
            }
            let req=match reference {
                Some(op)=>self.client.get(format!("{}/v2/web-retrieval/searches/{}/{}",self.origin(),self.scope(),op)).bearer_auth(token),
                None=>self.client.post(format!("{}/v2/web-retrieval/searches",self.origin())).bearer_auth(token)
                    .header("content-type","application/json").body(json!({"scope_id":self.scope(),"request":self.operation,"deadline_unix_ms":self.deadline()}).to_string())
            };
            match exchange(req, cancel, self.remaining()).await {
                Ok((200, result)) if Reply::accepts(&result, id, reference) => {
                    return Reply::from_wire(result);
                }
                Ok((status, problem)) if validate("APIError", &problem) => {
                    let code = problem["error"]["code"]
                        .as_str()
                        .unwrap_or("outcome_unknown");
                    if status == 401
                        && !refreshed
                        && code == "session_expired"
                        && problem["attempt_dispatched"] == false
                    {
                        refreshed = true;
                        let notice = gateway.1.notified();
                        tokio::pin!(notice);
                        notice.as_mut().enable();
                        if self
                            .grant(gateway)
                            .is_some_and(|v| v["bearer"] != grant["bearer"])
                        {
                            continue;
                        }
                        tokio::select! {biased;_=cancel.cancelled()=>{},_=tokio::time::timeout(self.remaining().min(Duration::from_secs(5)),notice)=>{}}
                        if self
                            .grant(gateway)
                            .is_some_and(|v| v["bearer"] != grant["bearer"])
                        {
                            continue;
                        }
                    }
                    let before = reference.is_none() && problem["attempt_dispatched"] == false;
                    return Reply::failure(
                        code,
                        if before {
                            "not_dispatched"
                        } else {
                            "uncertain"
                        },
                        if before { None } else { reference },
                    );
                }
                _ => return Reply::failure("outcome_unknown", "uncertain", reference),
            }
        }
    }
    async fn material(&self, g: &WebGateway, mut r: Reply, cancel: &CancellationToken) -> Reply {
        let relation = r.fact["relation"].clone();
        while r.fact["outcome"] == "pending" {
            let Some(id) = r.fact["operation_id"].as_str() else {
                return Reply::failure("invalid_response", "uncertain", None);
            };
            if self.remaining().is_zero() {
                return Reply::failure("timeout", "uncertain", Some(id));
            }
            tokio::select! {biased;_=cancel.cancelled()=>return Reply::failure("cancelled","uncertain",Some(id)),_=tokio::time::sleep(self.remaining().min(Duration::from_millis(150)))=>{}}
            r = self.call(g, Some(id), cancel).await;
        }
        r.fact["relation"] = relation;
        r
    }
    async fn dispatch(&self, g: &WebGateway, key: &Key, cancel: &CancellationToken) -> Reply {
        let reserve = Duration::from_millis(
            crate::web_retrieval_generated::CONTENT_HTTP_TIMEOUT_MS as u64
                + wire::DEEP_SEARCH_SETTLEMENT_RESERVE_MS,
        );
        let wait = self
            .remaining()
            .saturating_sub(reserve)
            .min(Duration::from_secs(wire::DEEP_SEARCH_QUEUE_MAX_SECONDS));
        if wait.is_zero() {
            return Reply::failure("busy", "not_dispatched", None);
        }
        let slot = tokio::select! {biased;_=cancel.cancelled()=>return Reply::failure("cancelled","not_dispatched",None),s=tokio::time::timeout(wait,g.2.clone().acquire_owned())=>s};
        let Ok(Ok(slot)) = slot else {
            return Reply::failure("busy", "not_dispatched", None);
        };
        // Waiting never extends the original admission deadline. Verify the
        // native owner again immediately before creating a product operation.
        let fresh = admit(
            &self.client,
            &self.install,
            &self.probe,
            true,
            cancel,
            self.remaining().min(Duration::from_secs(5)),
        )
        .await;
        let Ok((200, admission)) = fresh else {
            return Reply::failure(
                if cancel.is_cancelled() {
                    "cancelled"
                } else {
                    "not_entitled"
                },
                "not_dispatched",
                None,
            );
        };
        if !validate("WebAdmissionReceiptV2", &admission)
            || admission["scope_id"] != self.scope()
            || admission["operation_id"] != self.id()
        {
            return Reply::failure("not_entitled", "not_dispatched", None);
        }
        let mut admitted = self.clone();
        admitted.admission["deadline_unix_ms"] = json!(
            self.deadline()
                .min(admission["deadline_unix_ms"].as_u64().unwrap_or(0))
        );
        if admitted.remaining() < reserve {
            return Reply::failure("busy", "not_dispatched", None);
        }
        {
            let mut state = g.0.lock().expect("web state");
            let Some(flight) = state.scoped.get_mut(key).filter(|f| f.owner == self.id()) else {
                return Reply::failure("cancelled", "not_dispatched", None);
            };
            if cancel.is_cancelled() {
                return Reply::failure("cancelled", "not_dispatched", None);
            }
            flight.submitted = true;
        }
        let r = admitted.call(g, None, cancel).await;
        drop(slot);
        admitted.material(g, r, cancel).await
    }
}
impl WebGateway {
    pub(super) async fn scoped_search(
        &self,
        key: Key,
        request: Request,
        cancel: &CancellationToken,
    ) -> Reply {
        let mut original = None;
        let (mut done, relation, owner) = {
            let mut s = self.0.lock().expect("web state");
            if let Err(f) = key.check(&s) {
                return Reply::failure(f, "not_dispatched", None);
            }
            if let Some(f) = s.scoped.get_mut(&key) {
                f.waiters += 1;
                (
                    f.done.clone(),
                    if f.done.borrow().is_some() {
                        "reused"
                    } else {
                        "joined"
                    },
                    f.owner.clone(),
                )
            } else {
                if s.scoped.len() >= 1024 {
                    return Reply::failure("busy", "not_dispatched", None);
                }
                let token = CancellationToken::new();
                let (tx, rx) = watch::channel(None);
                let (send, receive) = oneshot::channel();
                original = Some(receive);
                let owner = request.id().to_owned();
                s.scoped.insert(
                    key.clone(),
                    Flight {
                        owner: owner.clone(),
                        cancel: token.clone(),
                        waiters: 1,
                        submitted: false,
                        done: rx.clone(),
                    },
                );
                let gateway = self.clone();
                let key = key.clone();
                let req = request.clone();
                tokio::spawn(async move {
                    let result = req.dispatch(&gateway, &key, &token).await;
                    let mut state = gateway.0.lock().expect("web state");
                    tx.send_replace(Some(result.fact.clone()));
                    if result.fact["dispatch"] == "not_dispatched"
                        && (result.fact.get("operation_id").is_none()
                            || matches!(
                                result.fact["error_code"].as_str(),
                                Some("upstream_unavailable" | "timeout" | "busy")
                            ))
                        && state.scoped.get(&key).is_some_and(|v| v.owner == req.id())
                    {
                        state.scoped.remove(&key);
                    }
                    let _ = send.send(result);
                });
                (rx, "first", owner)
            }
        };
        let subscription = Subscription {
            gateway: self.clone(),
            key,
            owner: owner.clone(),
        };
        if let Some(rx) = original {
            return tokio::select! {biased;_=cancel.cancelled()=>subscription.cancelled(relation),v=rx=>v.unwrap_or_else(|_|Reply::failure("outcome_unknown","uncertain",Some(request.id())))};
        }
        let mut fact = tokio::select! {biased;_=cancel.cancelled()=>return subscription.cancelled(relation),result=tokio::time::timeout(request.remaining(),done.wait_for(Option::is_some))=>match result{
            Ok(Ok(value))=>value.clone().expect("completed operation"),_=>return Reply::failure("outcome_unknown","uncertain",None)
        }};
        fact["relation"] = json!(relation);
        // An interrupted POST can be identified through the caller's own
        // handle. Only an authorized product read establishes the original ID.
        let id = fact["operation_id"]
            .as_str()
            .or_else(|| (fact["dispatch"] == "uncertain").then_some(owner.as_str()));
        let Some(id) = id else {
            return Reply {
                body: error(fact["error_code"].as_str().unwrap_or("outcome_unknown")),
                fact,
            };
        };
        // A follower always performs its own authorized read. A missing or
        // unavailable result never falls back to POST.
        let result = request.call(self, Some(id), cancel).await;
        let mut r = request.material(self, result, cancel).await;
        r.fact["relation"] = json!(relation);
        r
    }
}
