//! OAuth endpoints are learned only through the selected service's HTTPS
//! protected-resource discovery chain. No endpoint is accepted from the UI.
use crate::tushare_oauth::Failure;
use rmcp::transport::auth::AuthorizationMetadata;
use serde_json::Value;
use std::{collections::HashSet, sync::Mutex};
use url::Url;

#[derive(Default)]
struct State {
    issuer: Option<String>,
    issuers: Vec<String>,
    resource_metadata: Option<String>,
    metadata: Option<AuthorizationMetadata>,
}
pub struct DiscoveryPolicy {
    resource: String,
    state: Mutex<State>,
}
fn https(value: &str) -> Option<Url> {
    let url = Url::parse(value).ok()?;
    let host = url.host_str()?;
    (url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && host.contains('.')
        && host.parse::<std::net::IpAddr>().is_err()
        && !host.ends_with(".localhost")
        && !host.ends_with(".local"))
    .then_some(url)
}
fn metadata_urls(base: &str, kind: &str) -> Vec<String> {
    let Some(url) = https(base) else {
        return vec![];
    };
    let origin = url.origin().ascii_serialization();
    let path = url.path().trim_end_matches('/');
    let mut urls = vec![
        format!("{origin}/.well-known/{kind}{path}"),
        format!("{origin}/.well-known/{kind}"),
        format!("{origin}{path}/.well-known/{kind}"),
    ];
    if kind == "oauth-protected-resource" {
        urls.push(format!("{origin}/.well-known/{kind}{}", url.path()));
    }
    urls
}
impl DiscoveryPolicy {
    pub fn new(resource: String) -> Result<Self, Failure> {
        https(&resource).ok_or(Failure::InvalidMetadata)?;
        Ok(Self {
            resource,
            state: Mutex::new(State::default()),
        })
    }
    pub fn google_calendar() -> Result<Self, Failure> {
        let policy = Self::new(crate::google_calendar::RESOURCE.into())?;
        {
            let mut state = policy.state.lock().expect("OAuth discovery");
            state.issuer = Some(crate::google_calendar::ISSUER.into());
            state.metadata = Some(crate::google_calendar::metadata());
        }
        Ok(policy)
    }
    pub fn permits(&self, method: &str, url: &str) -> bool {
        if https(url).is_none() {
            return false;
        }
        if url == self.resource {
            return matches!(method, "GET" | "POST" | "DELETE");
        }
        let state = self.state.lock().expect("OAuth discovery");
        if method == "GET" {
            if metadata_urls(&self.resource, "oauth-protected-resource")
                .iter()
                .any(|v| v == url)
            {
                return true;
            }
            if state.resource_metadata.as_deref() == Some(url) {
                return true;
            }
            let bases: Vec<&str> = if let Some(issuer) = state.issuer.as_deref() {
                vec![issuer]
            } else if !state.issuers.is_empty() {
                state.issuers.iter().map(String::as_str).collect()
            } else {
                vec![&self.resource]
            };
            return bases
                .into_iter()
                .flat_map(|base| {
                    ["oauth-authorization-server", "openid-configuration"]
                        .into_iter()
                        .flat_map(move |kind| metadata_urls(base, kind))
                })
                .any(|v| v == url);
        }
        method == "POST"
            && state.metadata.as_ref().is_some_and(|m| {
                url == m.token_endpoint || m.registration_endpoint.as_deref() == Some(url)
            })
    }
    // Only the selected resource's own HTTPS response may name a custom
    // protected-resource metadata URL. Store one exact same-origin URL.
    pub fn observe_challenge(
        &self,
        endpoint: &str,
        status: u16,
        header: &str,
    ) -> Result<(), Failure> {
        if endpoint != self.resource || status != 401 {
            return Ok(());
        }
        if header.len() > 8192 {
            return Err(Failure::InvalidMetadata);
        }
        let lower = header.to_ascii_lowercase();
        let Some(offset) = lower.find("resource_metadata=") else {
            return Ok(());
        };
        let tail = &header[offset + "resource_metadata=".len()..];
        let value = if let Some(tail) = tail.strip_prefix('"') {
            tail.split_once('"').map(|(value, _)| value)
        } else {
            tail.split([',', ' ']).next()
        }
        .ok_or(Failure::InvalidMetadata)?;
        let base = https(&self.resource).ok_or(Failure::InvalidMetadata)?;
        let url = base.join(value).map_err(|_| Failure::InvalidMetadata)?;
        let parsed = https(url.as_str()).ok_or(Failure::InvalidMetadata)?;
        if parsed.origin() != base.origin() {
            return Err(Failure::InvalidMetadata);
        }
        let mut state = self.state.lock().expect("OAuth discovery");
        if state
            .resource_metadata
            .as_deref()
            .is_some_and(|old| old != url.as_str())
        {
            return Err(Failure::InvalidMetadata);
        }
        state.resource_metadata = Some(url.to_string());
        Ok(())
    }
    pub fn observe(&self, url: &str, body: &[u8]) -> Result<(), Failure> {
        if url == self.resource {
            return Ok(());
        }
        let Ok(value) = serde_json::from_slice::<Value>(body) else {
            return Ok(());
        };
        if metadata_urls(&self.resource, "oauth-protected-resource")
            .iter()
            .any(|v| v == url)
            || self
                .state
                .lock()
                .expect("OAuth discovery")
                .resource_metadata
                .as_deref()
                == Some(url)
        {
            let resource = value
                .get("resource")
                .and_then(Value::as_str)
                .ok_or(Failure::InvalidMetadata)?;
            // The resource may identify a base URL containing the MCP endpoint.
            let expected = https(&self.resource).ok_or(Failure::InvalidMetadata)?;
            let advertised = https(resource).ok_or(Failure::InvalidMetadata)?;
            if expected.origin() != advertised.origin()
                || !(expected.path() == advertised.path()
                    || expected
                        .path()
                        .starts_with(&format!("{}/", advertised.path().trim_end_matches('/'))))
            {
                return Err(Failure::InvalidMetadata);
            }
            let mut issuers = Vec::new();
            if let Some(issuer) = value.get("authorization_server").and_then(Value::as_str) {
                issuers.push(issuer.to_string());
            }
            if let Some(list) = value.get("authorization_servers").and_then(Value::as_array) {
                for issuer in list {
                    issuers.push(issuer.as_str().ok_or(Failure::InvalidMetadata)?.to_string());
                }
            }
            if issuers.is_empty() || issuers.len() > 8 {
                return Err(Failure::InvalidMetadata);
            }
            for issuer in &issuers {
                if https(issuer)
                    .ok_or(Failure::InvalidMetadata)?
                    .query()
                    .is_some()
                {
                    return Err(Failure::InvalidMetadata);
                }
            }
            let mut state = self.state.lock().expect("OAuth discovery");
            if !state.issuers.is_empty() && state.issuers != issuers {
                return Err(Failure::InvalidMetadata);
            }
            state.issuers = issuers;
        } else if let Ok(metadata) = serde_json::from_value::<AuthorizationMetadata>(value) {
            self.validate_metadata(&metadata)?;
        }
        Ok(())
    }
    pub fn validate_metadata(&self, metadata: &AuthorizationMetadata) -> Result<(), Failure> {
        let issuer = metadata.issuer.as_deref().ok_or(Failure::InvalidMetadata)?;
        let google = self.resource == crate::google_calendar::RESOURCE;
        if google
            && (issuer != crate::google_calendar::ISSUER
                || metadata.authorization_endpoint != crate::google_calendar::AUTHORIZE
                || metadata.token_endpoint != crate::google_calendar::TOKEN
                || metadata.registration_endpoint.is_some())
        {
            return Err(Failure::InvalidMetadata);
        }
        let issuer_url = https(issuer).ok_or(Failure::InvalidMetadata)?;
        let mut state = self.state.lock().expect("OAuth discovery");
        if let Some(expected) = &state.issuer {
            if issuer != expected {
                return Err(Failure::InvalidMetadata);
            }
        } else if !state.issuers.is_empty() {
            if !state.issuers.iter().any(|candidate| candidate == issuer) {
                return Err(Failure::InvalidMetadata);
            }
        } else if issuer_url.origin()
            != https(&self.resource)
                .ok_or(Failure::InvalidMetadata)?
                .origin()
        {
            return Err(Failure::InvalidMetadata);
        }
        for endpoint in [
            Some(metadata.authorization_endpoint.as_str()),
            Some(metadata.token_endpoint.as_str()),
            metadata.registration_endpoint.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            let url = https(endpoint).ok_or(Failure::InvalidMetadata)?;
            if url.query().is_some() {
                return Err(Failure::InvalidMetadata);
            }
        }
        if metadata
            .code_challenge_methods_supported
            .as_ref()
            .is_some_and(|values| !values.iter().any(|v| v == "S256"))
            || metadata
                .response_types_supported
                .as_ref()
                .is_some_and(|values| !values.iter().any(|v| v == "code"))
        {
            return Err(Failure::InvalidMetadata);
        }
        if !google
            && metadata
                .additional_fields
                .get("token_endpoint_auth_methods_supported")
                .and_then(Value::as_array)
                .is_some_and(|v| !v.iter().any(|v| v == "none"))
        {
            return Err(Failure::InvalidMetadata);
        }
        state.issuer = Some(issuer.into());
        state.metadata = Some(metadata.clone());
        Ok(())
    }
    pub fn validate_authorization_url(&self, value: &str, callback: &str) -> Result<(), Failure> {
        let url = https(value).ok_or(Failure::InvalidMetadata)?;
        let state = self.state.lock().expect("OAuth discovery");
        let metadata = state.metadata.as_ref().ok_or(Failure::InvalidMetadata)?;
        let mut base = url.clone();
        base.set_query(None);
        if base.as_str() != metadata.authorization_endpoint {
            return Err(Failure::InvalidMetadata);
        }
        let mut seen = HashSet::new();
        let mut pairs = std::collections::HashMap::new();
        for (key, value) in url.query_pairs() {
            if !seen.insert(key.to_string()) {
                return Err(Failure::InvalidMetadata);
            }
            pairs.insert(key.to_string(), value.to_string());
        }
        for (key, value) in [
            ("redirect_uri", callback),
            ("response_type", "code"),
            ("code_challenge_method", "S256"),
            ("resource", &self.resource),
        ] {
            if pairs.get(key).map(String::as_str) != Some(value) {
                return Err(Failure::InvalidMetadata);
            }
        }
        if ["state", "client_id", "code_challenge"]
            .iter()
            .any(|key| pairs.get(*key).is_none_or(String::is_empty))
        {
            return Err(Failure::InvalidMetadata);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_discovery_supports_path_issuer_and_exact_registered_endpoints() {
        let p = DiscoveryPolicy::new("https://service.example/mcp".into()).unwrap();
        p.observe("https://service.example/.well-known/oauth-protected-resource/mcp",br#"{"resource":"https://service.example/mcp","authorization_servers":["https://auth.example/realm"]}"#).unwrap();
        assert!(p.permits(
            "GET",
            "https://auth.example/.well-known/oauth-authorization-server/realm"
        ));
        let metadata:AuthorizationMetadata=serde_json::from_value(serde_json::json!({"issuer":"https://auth.example/realm","authorization_endpoint":"https://auth.example/authorize","token_endpoint":"https://auth.example/token","registration_endpoint":"https://auth.example/register","response_types_supported":["code"],"code_challenge_methods_supported":["S256"]})).unwrap();
        p.validate_metadata(&metadata).unwrap();
        assert!(p.permits("POST", "https://auth.example/token"));
    }
    #[test]
    fn normal_custom_resource_metadata_and_multiple_issuers_follow_one_selected_chain() {
        let p = DiscoveryPolicy::new("https://service.example/mcp".into()).unwrap();
        p.observe_challenge(
            "https://service.example/mcp",
            401,
            r#"Bearer resource_metadata="https://service.example/oauth-resource""#,
        )
        .unwrap();
        assert!(p.permits("GET", "https://service.example/oauth-resource"));
        p.observe("https://service.example/oauth-resource", br#"{"resource":"https://service.example/mcp","authorization_servers":["https://auth.example/primary","https://auth.example/secondary"]}"#).unwrap();
        assert!(p.permits(
            "GET",
            "https://auth.example/.well-known/oauth-authorization-server/secondary"
        ));
        let metadata = serde_json::from_value(serde_json::json!({"issuer":"https://auth.example/secondary","authorization_endpoint":"https://auth.example/authorize","token_endpoint":"https://auth.example/token"})).unwrap();
        p.validate_metadata(&metadata).unwrap();
        assert!(p.permits("POST", "https://auth.example/token"));
        assert!(!p.permits(
            "GET",
            "https://auth.example/.well-known/oauth-authorization-server/primary"
        ));
    }
}
