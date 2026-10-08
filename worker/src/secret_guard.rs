//! In-memory values already present in owned transport credentials. Never log
//! or serialize this set, and never evict a secret while a client remains live.
use serde_json::Value;
use std::sync::Mutex;
#[derive(Default)]
struct State {
    values: Vec<String>,
    exhausted: bool,
}
#[derive(Default)]
pub struct SecretGuard(Mutex<State>);
impl SecretGuard {
    pub fn remember(&self, value: &str) {
        if value.is_empty() {
            return;
        }
        let mut state = self.0.lock().expect("credential boundary");
        for value in [Some(value), value.strip_prefix("Bearer ")]
            .into_iter()
            .flatten()
        {
            if !state.values.iter().any(|v| v == value) {
                if state.values.len() >= 32 {
                    state.exhausted = true;
                    return;
                }
                state.values.push(value.into());
            }
        }
    }
    pub fn available(&self) -> bool {
        !self.0.lock().expect("credential boundary").exhausted
    }
    pub fn safe(&self, value: &Value) -> bool {
        fn check(value: &Value, secrets: &[String]) -> bool {
            match value {
                Value::String(text) => !secrets.iter().any(|s| text.contains(s)),
                Value::Object(fields) => fields
                    .iter()
                    .all(|(k, v)| !secrets.iter().any(|s| k.contains(s)) && check(v, secrets)),
                Value::Array(items) => items.iter().all(|v| check(v, secrets)),
                _ => true,
            }
        }
        let state = self.0.lock().expect("credential boundary");
        !state.exhausted && check(value, &state.values)
    }
    pub fn remember_tokens(&self, value: &Value) {
        if let Some(object) = value.as_object() {
            for key in ["access_token", "refresh_token", "id_token", "client_secret"] {
                if let Some(value) = object.get(key).and_then(Value::as_str) {
                    self.remember(value);
                }
            }
        }
    }
}
