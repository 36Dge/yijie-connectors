use super::{WebGateway, error};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::sync::watch;

// Store operation handles only. Product authorization and result retention
// remain with the existing operation service.
type Outcome = Option<Result<(), Value>>;
pub(super) type Searches = HashMap<Key, (String, watch::Receiver<Outcome>)>;

#[derive(Clone, Hash, PartialEq, Eq)]
pub(super) struct Key {
    caller: (String, String),
    root: (String, String),
    binding_epoch: String,
    binding: String,
    authorization_revision: u64,
    tool: String,
    arguments: [u8; 32],
}

impl Key {
    pub(super) fn new(
        install: &Value,
        caller: (&str, &str),
        receipt: &Value,
        tool: &str,
        arguments: &Value,
    ) -> Self {
        let mut sorted = arguments.clone();
        sorted.sort_all_objects();
        let grant = &install["control"]["delegation"];
        Self {
            caller: (caller.0.into(), caller.1.into()),
            root: (
                receipt["root_thread_id"].as_str().unwrap().into(),
                receipt["root_turn_id"].as_str().unwrap().into(),
            ),
            binding_epoch: install["control"]["binding_epoch"].as_str().unwrap().into(),
            binding: grant["binding_id"].as_str().unwrap().into(),
            authorization_revision: grant["authorization_revision"].as_u64().unwrap(),
            tool: tool.to_owned(),
            arguments: Sha256::digest(sorted.to_string().as_bytes()).into(),
        }
    }

    pub(super) fn check(&self, state: &super::State) -> Result<(), &'static str> {
        if state.stopped.contains(&self.caller) || state.stopped.contains(&self.root) {
            return Err("cancelled");
        }
        if !state.install.as_ref().is_some_and(|i| {
            let grant = &i["control"]["delegation"];
            i["control"]["binding_epoch"] == self.binding_epoch
                && grant["binding_id"] == self.binding
                && grant["authorization_revision"] == self.authorization_revision
        }) {
            return Err("not_entitled");
        }
        Ok(())
    }
    pub(super) fn belongs_to(&self, thread: &str, turn: &str) -> bool {
        [&self.caller, &self.root]
            .into_iter()
            .any(|(t, u)| t == thread && u == turn)
    }
}

pub(super) enum Search {
    First(Leader),
    Existing(String, watch::Receiver<Outcome>),
}

impl WebGateway {
    pub(super) fn begin_search(&self, key: Key, id: &str) -> Result<Search, Value> {
        let mut state = self.0.lock().expect("web state");
        key.check(&state).map_err(error)?;
        if let Some((original, done)) = state.searches.get(&key) {
            return Ok(Search::Existing(original.clone(), done.clone()));
        }
        // Retain active-turn IDs through terminal/revocation. Evicting a
        // successful ID here would turn a repeat into another paid operation.
        if state.searches.len() >= 1024 {
            return Err(error("busy"));
        }
        let (complete, done) = watch::channel(None);
        state.searches.insert(key.clone(), (id.into(), done));
        Ok(Search::First(Leader {
            owner: self.clone(),
            key,
            id: id.into(),
            complete,
        }))
    }
}

pub(super) struct Leader {
    owner: WebGateway,
    key: Key,
    id: String,
    complete: watch::Sender<Outcome>,
}

impl Leader {
    pub(super) fn finish(self, value: &Value) {
        let code = value.pointer("/error/code").and_then(Value::as_str);
        let mut state = self.owner.0.lock().expect("web state");
        self.complete.send_replace(Some(match code {
            None => Ok(()),
            Some(_) => Err(value.clone()),
        }));
        if let Some(code) = code
            && matches!(
                code,
                "busy"
                    | "not_entitled"
                    | "quota_exceeded"
                    | "session_expired"
                    | "invalid_arguments"
            )
            && state
                .searches
                .get(&self.key)
                .is_some_and(|(id, _)| id == &self.id)
        {
            state.searches.remove(&self.key);
        }
    }
}

impl Drop for Leader {
    fn drop(&mut self) {
        // Dropping an in-flight MCP future cannot establish whether the
        // supplier received it. Wake followers without starting another call.
        if self.complete.borrow().is_none() {
            self.complete
                .send_replace(Some(Err(error("outcome_unknown"))));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn deep_search_joins_in_flight_and_keeps_only_confirmed_results() {
        let gateway = WebGateway::default();
        let install = json!({"control":{"binding_epoch":"epoch","delegation":{"binding_id":"binding","authorization_revision":1}}});
        gateway.0.lock().unwrap().install = Some(install.clone());
        let receipt = json!({"root_thread_id":"root","root_turn_id":"turn"});
        let key = |turn, args: Value| {
            Key::new(
                &install,
                ("root", turn),
                &receipt,
                "kimi_web_search_pro",
                &args,
            )
        };
        let args = json!({"query":"西湖游船","limit":5,"sites":["a.example","b.example"]});
        let same = json!({"sites":["a.example","b.example"],"limit":5,"query":"西湖游船"});
        let Search::First(first) = gateway
            .begin_search(key("turn", args.clone()), "1")
            .unwrap()
        else {
            panic!("expected original operation")
        };
        let Search::Existing(id, mut follower) =
            gateway.begin_search(key("turn", same), "2").unwrap()
        else {
            panic!("expected in-flight join")
        };
        assert_eq!(id, "1");
        assert!(follower.borrow().is_none());
        first.finish(&json!({"results":[]}));
        follower.changed().await.unwrap();
        assert_eq!(*follower.borrow(), Some(Ok(())));
        assert!(
            matches!(gateway.begin_search(key("turn", args.clone()), "3").unwrap(), Search::Existing(id, _) if id == "1")
        );

        for (turn, changed) in [
            (
                "turn",
                json!({"query":"西湖游船票价","limit":5,"sites":["a.example","b.example"]}),
            ),
            (
                "turn",
                json!({"query":"西湖游船","limit":4,"sites":["a.example","b.example"]}),
            ),
            (
                "turn",
                json!({"query":"西湖游船","sites":["a.example","b.example"]}),
            ),
            (
                "turn",
                json!({"query":"西湖游船","limit":5,"sites":["b.example","a.example"]}),
            ),
            (
                "turn",
                json!({"query":"西湖游船","limit":5,"sites":["a.example","b.example"],"time_window":{"start":"2026-11-18"}}),
            ),
            ("fresh-user-turn", args.clone()),
        ] {
            let Search::First(new) = gateway
                .begin_search(key(turn, changed.clone()), "4")
                .unwrap()
            else {
                panic!("distinct request coalesced")
            };
            new.finish(&error("busy"));
            let Search::First(retry) = gateway.begin_search(key(turn, changed), "5").unwrap()
            else {
                panic!("busy was recorded as a success")
            };
            retry.finish(&error("outcome_unknown"));
        }
        let Search::Existing(_, uncertain) = gateway
            .begin_search(key("fresh-user-turn", args.clone()), "6")
            .unwrap()
        else {
            panic!("ambiguous operation repeated")
        };
        assert_eq!(*uncertain.borrow(), Some(Err(error("outcome_unknown"))));
        let revoked = key("turn", args);
        gateway.0.lock().unwrap().install = None;
        assert!(matches!(gateway.begin_search(revoked, "7"), Err(v) if v == error("not_entitled")));
    }
}
