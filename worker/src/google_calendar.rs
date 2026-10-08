//! Official Google Calendar remote MCP: pre-registered web OAuth client.
//! Client settings stay inside Connectors/Keyring; no token files or child runtime.
use crate::tushare_oauth::Failure;
use codex_keyring_store::{DefaultKeyringStore, KeyringStore};
use rmcp::transport::auth::{
    AuthorizationManager, AuthorizationMetadata, AuthorizationSession, OAuthClientConfig,
};
use serde::{Deserialize, Serialize};
pub const SERVICE: &str = "google-calendar";
pub const RESOURCE: &str = "https://calendarmcp.googleapis.com/mcp/v1";
pub const ISSUER: &str = "https://accounts.google.com";
pub const AUTHORIZE: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN: &str = "https://oauth2.googleapis.com/token";
pub const CALLBACK_BIND: &str = "127.0.0.1:18757";
pub const CALLBACK_PATH: &str = "/oauth/callback/google-calendar";
pub const CALLBACK: &str = "http://127.0.0.1:18757/oauth/callback/google-calendar";
const STORE: &str = "Yijie Market OAuth Clients";
pub const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/calendar.calendarlist.readonly",
    "https://www.googleapis.com/auth/calendar.events.freebusy",
    "https://www.googleapis.com/auth/calendar.events.readonly",
];
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub allow_event_updates: bool,
}
impl ClientConfig {
    pub fn scopes(&self) -> Vec<&'static str> {
        if self.allow_event_updates {
            vec![
                SCOPES[0],
                SCOPES[1],
                "https://www.googleapis.com/auth/calendar.events",
            ]
        } else {
            SCOPES.to_vec()
        }
    }
    pub fn validate(&self) -> Result<(), Failure> {
        if !self.client_id.ends_with(".apps.googleusercontent.com")
            || self.client_id.len() > 256
            || self
                .client_id
                .bytes()
                .any(|b| !b.is_ascii_alphanumeric() && b != b'-' && b != b'.')
            || self.client_secret.is_empty()
            || self.client_secret.len() > 4096
            || self.client_secret.chars().any(char::is_control)
        {
            return Err(Failure::InvalidMetadata);
        }
        Ok(())
    }
}
pub fn load(alias: &str) -> Result<Option<ClientConfig>, Failure> {
    crate::tushare_oauth::validate_library_home()?;
    DefaultKeyringStore
        .load(STORE, alias)
        .map_err(|_| Failure::VaultUnavailable)?
        .map(|text| {
            let config: ClientConfig =
                serde_json::from_str(&text).map_err(|_| Failure::VaultUnavailable)?;
            config.validate()?;
            Ok(config)
        })
        .transpose()
}
pub fn save(alias: &str, config: &ClientConfig) -> Result<(), Failure> {
    crate::tushare_oauth::validate_library_home()?;
    config.validate()?;
    DefaultKeyringStore
        .save(
            STORE,
            alias,
            &serde_json::to_string(config).map_err(|_| Failure::VaultUnavailable)?,
        )
        .map_err(|_| Failure::VaultUnavailable)
}
pub fn forget(alias: &str) -> Result<(), Failure> {
    crate::tushare_oauth::validate_library_home()?;
    DefaultKeyringStore
        .delete(STORE, alias)
        .map_err(|_| Failure::VaultUnavailable)?;
    Ok(())
}
pub fn metadata() -> AuthorizationMetadata {
    serde_json::from_value(serde_json::json!({"issuer":ISSUER,"authorization_endpoint":AUTHORIZE,"token_endpoint":TOKEN,"response_types_supported":["code"],"grant_types_supported":["authorization_code","refresh_token"],"code_challenge_methods_supported":["S256"],"token_endpoint_auth_methods_supported":["client_secret_post"],"scopes_supported":SCOPES})).expect("fixed Google OAuth metadata")
}
pub fn tools_for_scopes(
    config: &ClientConfig,
    tools: Vec<rmcp::model::Tool>,
) -> Vec<rmcp::model::Tool> {
    tools
        .into_iter()
        .filter(|tool| {
            config.allow_event_updates
                || matches!(
                    tool.name.as_ref(),
                    "get_event" | "list_calendars" | "list_events" | "suggest_time"
                )
        })
        .collect()
}
pub async fn session(
    mut manager: AuthorizationManager,
    config: &ClientConfig,
    callback: &str,
) -> Result<AuthorizationSession, Failure> {
    config.validate()?;
    let scopes = config.scopes();
    manager
        .configure_client(
            OAuthClientConfig::new(config.client_id.clone(), callback)
                .with_client_secret(config.client_secret.clone())
                .with_scopes(scopes.iter().map(|s| s.to_string()).collect())
                .with_application_type("web"),
        )
        .map_err(|_| Failure::InvalidMetadata)?;
    let mut url = url::Url::parse(
        &manager
            .get_authorization_url(&scopes)
            .await
            .map_err(|_| Failure::Unavailable)?,
    )
    .map_err(|_| Failure::InvalidMetadata)?;
    url.query_pairs_mut()
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");
    Ok(AuthorizationSession::for_scope_upgrade(
        manager,
        url.into(),
        callback,
    ))
}
// Codex's token vault stores the client id and token response, not confidential
// client configuration. Supply that configuration only at Google's token URL.
pub fn token_body(config: &ClientConfig, bytes: &[u8]) -> Result<Vec<u8>, Failure> {
    config.validate()?;
    let mut pairs: Vec<(String, String)> =
        url::form_urlencoded::parse(bytes).into_owned().collect();
    let find = |key: &str| {
        pairs
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect::<Vec<_>>()
    };
    if find("client_id") != vec![config.client_id.as_str()]
        || !matches!(
            find("grant_type").as_slice(),
            ["authorization_code"] | ["refresh_token"]
        )
    {
        return Err(Failure::InvalidMetadata);
    }
    let secrets = find("client_secret");
    if !secrets.is_empty() && secrets != vec![config.client_secret.as_str()] {
        return Err(Failure::InvalidMetadata);
    }
    if secrets.is_empty() {
        pairs.push(("client_secret".into(), config.client_secret.clone()));
    }
    Ok(url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish()
        .into_bytes())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_refresh_uses_private_registered_client_once() {
        let c = ClientConfig {
            client_id: "ordinary.apps.googleusercontent.com".into(),
            client_secret: "ordinary-local-value".into(),
            allow_event_updates: false,
        };
        let first=token_body(&c,b"client_id=ordinary.apps.googleusercontent.com&grant_type=refresh_token&refresh_token=ordinary-local-refresh").unwrap();
        let second = token_body(&c, &first).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            url::form_urlencoded::parse(&second)
                .filter(|(k, _)| k == "client_secret")
                .count(),
            1
        );
        assert_eq!(metadata().token_endpoint, TOKEN);
    }
    #[tokio::test]
    async fn normal_registered_client_session_uses_pkce_and_explicit_read_scopes_without_io() {
        let mut manager = AuthorizationManager::new(RESOURCE).await.unwrap();
        manager.set_metadata(metadata());
        let config = ClientConfig {
            client_id: "ordinary.apps.googleusercontent.com".into(),
            client_secret: "ordinary-local-value".into(),
            allow_event_updates: false,
        };
        let session = session(manager, &config, CALLBACK).await.unwrap();
        let url = url::Url::parse(session.get_authorization_url()).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["redirect_uri"], CALLBACK);
        assert_eq!(pairs["code_challenge_method"], "S256");
        assert_eq!(pairs["access_type"], "offline");
        assert_eq!(pairs["scope"], SCOPES.join(" "));
        assert!(!pairs.contains_key("client_secret"));
        assert!(!pairs["state"].is_empty());
        let editing = ClientConfig {
            allow_event_updates: true,
            ..config
        };
        assert!(
            editing
                .scopes()
                .contains(&"https://www.googleapis.com/auth/calendar.events")
        );
    }
    struct LocalTokenEndpoint(std::sync::atomic::AtomicUsize);
    impl rmcp::transport::auth::OAuthHttpClient for LocalTokenEndpoint {
        fn execute(
            &self,
            request: rmcp::transport::auth::OAuthHttpRequest,
        ) -> rmcp::transport::auth::OAuthHttpClientFuture<'_> {
            Box::pin(async move {
                assert_eq!(request.request.uri().to_string(), TOKEN);
                let body: std::collections::HashMap<_, _> =
                    url::form_urlencoded::parse(request.request.body())
                        .into_owned()
                        .collect();
                assert_eq!(body["client_id"], "ordinary.apps.googleusercontent.com");
                assert_eq!(body["client_secret"], "ordinary-local-value");
                assert_eq!(body["redirect_uri"], CALLBACK);
                assert_eq!(body["grant_type"], "authorization_code");
                assert!(!body["code_verifier"].is_empty());
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(oauth2::http::Response::builder().status(200).header("content-type","application/json").body(br#"{"access_token":"ordinary-local-access","refresh_token":"ordinary-local-refresh","token_type":"Bearer","expires_in":3600}"#.to_vec()).unwrap())
            })
        }
    }
    #[tokio::test]
    async fn registered_oauth_normal_callback_uses_rmcp_state_and_token_exchange_without_network() {
        let endpoint =
            std::sync::Arc::new(LocalTokenEndpoint(std::sync::atomic::AtomicUsize::new(0)));
        let mut manager =
            AuthorizationManager::new_with_oauth_http_client(RESOURCE, endpoint.clone())
                .await
                .unwrap();
        manager.set_metadata(metadata());
        let config = ClientConfig {
            client_id: "ordinary.apps.googleusercontent.com".into(),
            client_secret: "ordinary-local-value".into(),
            allow_event_updates: false,
        };
        let session = session(manager, &config, CALLBACK).await.unwrap();
        let parsed = url::Url::parse(session.get_authorization_url()).unwrap();
        let state = parsed
            .query_pairs()
            .find(|(k, _)| k == "state")
            .unwrap()
            .1
            .into_owned();
        session
            .handle_callback_with_issuer("ordinary-local-code", &state, None)
            .await
            .unwrap();
        assert_eq!(endpoint.0.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(session.get_credentials().await.unwrap().1.is_some());
        let tools = ["list_events", "create_event", "delete_event"]
            .into_iter()
            .map(|name| rmcp::model::Tool::new(name, "普通本地工具", serde_json::Map::new()))
            .collect();
        let allowed = tools_for_scopes(&config, tools);
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0].name, "list_events");
    }
}
