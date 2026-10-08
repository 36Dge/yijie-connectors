//! Tushare OAuth uses the fixed official rmcp authorization state machine and
//! Codex's Keyring-only storage. We own only the callback/cancellation lifetime.
//! No platform credential is serialized onto the owner control channel.
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use codex_config::types::{AuthKeyringBackendKind, OAuthCredentialsStoreMode};
use codex_exec_server::{
    ExecServerError, HttpClient, HttpHeader, HttpRedirectPolicy, HttpRequestParams,
    HttpRequestResponse, HttpResponseBodyStream, ReqwestHttpClient,
};
use codex_rmcp_client::{McpAuthState, StoredOAuthTokens, WrappedOAuthTokenResponse};
use oauth2::TokenResponse;
use rmcp::transport::auth::{
    AuthorizationManager, AuthorizationMetadata, AuthorizationSession, OAuthHttpClient,
    OAuthHttpClientError, OAuthHttpClientFuture, OAuthHttpRequest,
};
use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::oneshot, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use url::Url;
use uuid::Uuid;

pub const TUSHARE_RESOURCE: &str = "https://api.tushare.pro/mcp/";
pub const TUSHARE_ISSUER: &str = "https://tushare.pro";
pub const TUSHARE_SCOPE: &str = "mcp:tools";
const MAX_OAUTH_BYTES: usize = 1024 * 1024;
const MAX_FLOW_TTL: Duration = Duration::from_secs(300);
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    Cancelled,
    Expired,
    Unavailable,
    VaultUnavailable,
    InvalidMetadata,
    NotAuthorized,
    CleanupPending,
}
#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    KeyringPreflight,
    CallbackBind,
    ManagerNew,
    Discovery,
    ResourceChallenge,
    ResourceMetadata,
    AuthorizationMetadata,
    MetadataValidation,
    Registration,
    AuthorizationUrlValidation,
    AwaitingCallback,
    TokenExchange,
    KeyringCommit,
    MetadataInitialize,
    MetadataToolsList,
    MetadataArtifact,
    Cleanup,
    UnrecognizedEndpoint,
    CallApproval,
    CallAdmission,
    DailyExecute,
    DailyHttpAdmission,
    DailyHttpPermit,
    DailyHttpSend,
    DailyHttpResponse,
    DailyClientResult,
    DailyNormalize,
    DailyResultEnvelope,
    DailyResultRows,
    DailyResultIdentity,
    DailyResultNumeric,
}
#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Started,
    Succeeded,
    Failed,
    Request,
    Response,
    TransportUnavailable,
    ResponseUnavailable,
    PolicyRejected,
    RedirectRejected,
    ResponseLimit,
    Cancelled,
    Expired,
    Rejected,
    Stopped,
    Consumed,
    Unverified,
    ProviderError,
    Structured,
    TextJson,
    TextInvalid,
    ContentUnsupported,
    Records,
    DataWrapper,
    FieldsItems,
    SingleRecord,
    UnknownShape,
}
struct DiagnosticFile {
    file: std::fs::File,
    count: usize,
}
#[derive(Clone, Default)]
pub struct Diagnostic(Option<Arc<Mutex<DiagnosticFile>>>);
impl Diagnostic {
    pub fn for_operation(operation: &str) -> Self {
        use std::os::unix::fs::OpenOptionsExt;
        let create = || -> Option<DiagnosticFile> {
            let id = Uuid::parse_str(operation).ok()?;
            if id.is_nil() || id.to_string() != operation {
                return None;
            }
            let directory = validate_library_home()
                .ok()?
                .join("diagnostics")
                .join("provider");
            std::fs::create_dir_all(&directory).ok()?;
            // One worker owns this home. Keep new diagnostic allocation bounded
            // across normal Host restarts without deleting historical evidence.
            static CREATION: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
            let _creation = CREATION.get_or_init(|| Mutex::new(())).lock().ok()?;
            let directory_info = std::fs::symlink_metadata(&directory).ok()?;
            if !directory_info.is_dir() || directory_info.file_type().is_symlink() {
                return None;
            }
            let mut count = 0usize;
            let mut total = 0u64;
            for entry in std::fs::read_dir(&directory).ok()? {
                count += 1;
                total = total.saturating_add(entry.ok()?.metadata().ok()?.len());
                if count >= 128 || total >= 8 * 1024 * 1024 {
                    return None;
                }
            }
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(directory.join(format!("{operation}.jsonl")))
                .ok()?;
            Some(DiagnosticFile { file, count: 0 })
        };
        Self(create().map(|file| Arc::new(Mutex::new(file))))
    }
    pub fn emit(
        &self,
        stage: Stage,
        event: Event,
        http_status: Option<u16>,
        failure: Option<Failure>,
    ) {
        use std::io::Write;
        let Some(file) = &self.0 else {
            return;
        };
        let Ok(mut file) = file.lock() else {
            return;
        };
        if file.count >= 128 {
            return;
        }
        // Every field is an enum or numeric observation. Never accept upstream
        // text, URL, headers, scope, alias, identifiers, callback codes or tokens.
        let record = serde_json::json!({"schemaVersion":1,"unixMs":SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|v|v.as_millis() as u64),"stage":stage,"event":event,"httpStatus":http_status,"safeFailure":failure});
        if let Ok(mut bytes) = serde_json::to_vec(&record) {
            bytes.push(b'\n');
            if bytes.len() > 512 {
                return;
            }
            if file.file.write_all(&bytes).is_ok() {
                file.count += 1;
            }
        }
    }
    fn result<T>(&self, stage: Stage, result: Result<T, Failure>) -> Result<T, Failure> {
        self.emit(
            stage,
            if result.is_ok() {
                Event::Succeeded
            } else {
                Event::Failed
            },
            None,
            result.as_ref().err().copied(),
        );
        result
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Starting,
    AwaitingUser,
    Succeeded,
    Failed,
    Cancelled,
}
#[derive(Clone)]
pub struct Observation {
    pub phase: Phase,
    pub authorization_url: Option<String>,
    pub failure: Option<Failure>,
}
impl Observation {
    fn failed(failure: Failure) -> Self {
        Self {
            phase: if failure == Failure::Cancelled {
                Phase::Cancelled
            } else {
                Phase::Failed
            },
            authorization_url: None,
            failure: Some(failure),
        }
    }
}

/// Pure internal target, never deserialized from control/renderer input.
#[derive(Clone)]
struct Target {
    resource: String,
    issuer: String,
    policy: Option<Arc<crate::oauth_policy::DiscoveryPolicy>>,
    google_client: Option<Arc<crate::google_calendar::ClientConfig>>,
}
impl Target {
    fn official() -> Self {
        Self {
            resource: TUSHARE_RESOURCE.into(),
            issuer: TUSHARE_ISSUER.into(),
            policy: None,
            google_client: None,
        }
    }
    fn for_service(service: &str) -> Result<Self, Failure> {
        if service == "tushareMcp" {
            return Ok(Self::official());
        }
        if service == crate::google_calendar::SERVICE {
            return Ok(Self {
                resource: crate::google_calendar::RESOURCE.into(),
                issuer: crate::google_calendar::ISSUER.into(),
                policy: Some(Arc::new(
                    crate::oauth_policy::DiscoveryPolicy::google_calendar()?,
                )),
                google_client: None,
            });
        }
        let spec = crate::provider_registry::get(service)
            .filter(|s| s.auth_mode == "oauth" || crate::credentials::supports(service))
            .ok_or(Failure::InvalidMetadata)?;
        let resource = spec.resource.clone().ok_or(Failure::InvalidMetadata)?;
        Ok(Self {
            policy: if spec.auth_mode == "oauth" {
                Some(Arc::new(crate::oauth_policy::DiscoveryPolicy::new(
                    resource.clone(),
                )?))
            } else {
                None
            },
            resource,
            issuer: String::new(),
            google_client: None,
        })
    }
    fn observe_challenges(
        &self,
        endpoint: &str,
        response: &HttpRequestResponse,
    ) -> Result<(), Failure> {
        if let Some(policy) = &self.policy {
            for header in &response.headers {
                if header.name.eq_ignore_ascii_case("www-authenticate") {
                    policy.observe_challenge(endpoint, response.status, &header.value)?;
                }
            }
        }
        Ok(())
    }
    fn observe(&self, url: &str, bytes: &[u8]) -> Result<(), Failure> {
        match &self.policy {
            Some(policy) => policy.observe(url, bytes),
            None => Ok(()),
        }
    }
    fn permits(&self, method: &str, url: &str) -> bool {
        if let Some(policy) = &self.policy {
            return policy.permits(method, url);
        }

        let Ok(parsed) = Url::parse(url) else {
            return false;
        };
        if !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return false;
        }
        if url == self.resource {
            return matches!(method, "GET" | "POST" | "DELETE");
        }
        let resource_origin = Url::parse(&self.resource)
            .expect("fixed target")
            .origin()
            .ascii_serialization();
        if method == "GET"
            && [
                format!("{resource_origin}/.well-known/oauth-protected-resource"),
                format!("{resource_origin}/.well-known/oauth-protected-resource/mcp/"),
                format!("{resource_origin}/.well-known/oauth-protected-resource/mcp"),
                format!("{}/.well-known/oauth-authorization-server", self.issuer),
            ]
            .contains(&url.to_string())
        {
            return true;
        }
        method == "POST"
            && [
                format!("{}/oauth/register", self.issuer),
                format!("{}/oauth/token", self.issuer),
            ]
            .contains(&url.to_string())
    }
    fn validate_metadata(&self, value: &AuthorizationMetadata) -> Result<(), Failure> {
        if let Some(policy) = &self.policy {
            return policy.validate_metadata(value);
        }
        if value.issuer.as_deref() != Some(&self.issuer)
            || value.authorization_endpoint != format!("{}/oauth/authorize", self.issuer)
            || value.token_endpoint != format!("{}/oauth/token", self.issuer)
            || value.registration_endpoint.as_deref()
                != Some(format!("{}/oauth/register", self.issuer).as_str())
            || !value
                .scopes_supported
                .as_ref()
                .is_some_and(|v| v.iter().any(|s| s == TUSHARE_SCOPE))
            || !value
                .code_challenge_methods_supported
                .as_ref()
                .is_some_and(|v| v.iter().any(|s| s == "S256"))
            || !value
                .response_types_supported
                .as_ref()
                .is_some_and(|v| v.iter().any(|s| s == "code"))
            || !value
                .additional_fields
                .get("token_endpoint_auth_methods_supported")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|v| v.iter().any(|s| s == "none"))
        {
            return Err(Failure::InvalidMetadata);
        }
        Ok(())
    }
    fn validate_authorization_url(&self, url: &str, callback: &str) -> Result<(), Failure> {
        if let Some(policy) = &self.policy {
            return policy.validate_authorization_url(url, callback);
        }
        let parsed = Url::parse(url).map_err(|_| Failure::InvalidMetadata)?;
        if parsed.origin().ascii_serialization() != self.issuer
            || parsed.path() != "/oauth/authorize"
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.fragment().is_some()
        {
            return Err(Failure::InvalidMetadata);
        }
        let pairs: HashMap<_, _> = parsed
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        for (key, expected) in [
            ("redirect_uri", callback),
            ("response_type", "code"),
            ("scope", TUSHARE_SCOPE),
            ("code_challenge_method", "S256"),
            ("resource", &self.resource),
        ] {
            if pairs.get(key).map(String::as_str) != Some(expected) {
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

/// A narrow OAuth HTTP adapter. Protocol/PKCE/token exchange remain rmcp-owned.
/// All responses are bounded; errors never format upstream bodies or URLs.
struct OAuthHttp {
    target: Target,
    http: Arc<dyn HttpClient>,
    cancellation: CancellationToken,
    diagnostic: Diagnostic,
}
impl OAuthHttpClient for OAuthHttp {
    fn execute(&self, request: OAuthHttpRequest) -> OAuthHttpClientFuture<'_> {
        Box::pin(async move {
            let (parts, body) = request.request.into_parts();
            let endpoint = parts.uri.to_string();
            let stage = if endpoint == self.target.resource {
                Stage::ResourceChallenge
            } else if endpoint.contains("/.well-known/oauth-protected-resource") {
                Stage::ResourceMetadata
            } else if endpoint.contains("/.well-known/oauth-authorization-server") {
                Stage::AuthorizationMetadata
            } else if endpoint == format!("{}/oauth/register", self.target.issuer) {
                Stage::Registration
            } else if endpoint == format!("{}/oauth/token", self.target.issuer) {
                Stage::TokenExchange
            } else {
                Stage::UnrecognizedEndpoint
            };
            if self.cancellation.is_cancelled()
                || !self
                    .target
                    .permits(parts.method.as_str(), &parts.uri.to_string())
                || body.len() > MAX_OAUTH_BYTES
            {
                self.diagnostic
                    .emit(stage, Event::PolicyRejected, None, None);
                return Err(OAuthHttpClientError::new("OAuth operation unavailable"));
            }
            let headers = parts
                .headers
                .iter()
                .map(|(name, value)| {
                    Ok(HttpHeader {
                        name: name.to_string(),
                        value: value
                            .to_str()
                            .map_err(|_| OAuthHttpClientError::new("OAuth headers unavailable"))?
                            .into(),
                    })
                })
                .collect::<Result<Vec<_>, OAuthHttpClientError>>()?;
            let operation = async {
                self.diagnostic.emit(stage, Event::Request, None, None);
                let (response, mut stream) = self
                    .http
                    .http_request_stream(HttpRequestParams {
                        method: parts.method.to_string(),
                        url: parts.uri.to_string(),
                        headers,
                        body: (!body.is_empty()).then_some(body.into()),
                        timeout_ms: Some(15_000),
                        redirect_policy: HttpRedirectPolicy::Stop,
                        request_id: Uuid::new_v4().to_string(),
                        stream_response: true,
                    })
                    .await
                    .map_err(|_| {
                        self.diagnostic
                            .emit(stage, Event::TransportUnavailable, None, None);
                        OAuthHttpClientError::new("OAuth transport unavailable")
                    })?;
                self.diagnostic
                    .emit(stage, Event::Response, Some(response.status), None);
                // Registered Tushare endpoints do not require redirects. Reject
                // any redirect so credentials cannot follow a new destination.
                if (300..400).contains(&response.status) {
                    self.diagnostic.emit(
                        stage,
                        Event::RedirectRejected,
                        Some(response.status),
                        None,
                    );
                    return Err(OAuthHttpClientError::new(
                        "OAuth redirect requires qualification",
                    ));
                }
                self.target
                    .observe_challenges(&endpoint, &response)
                    .map_err(|_| OAuthHttpClientError::new("OAuth discovery unavailable"))?;
                let mut bytes = Vec::new();
                while let Some(chunk) = stream.recv().await.map_err(|_| {
                    self.diagnostic
                        .emit(stage, Event::ResponseUnavailable, None, None);
                    OAuthHttpClientError::new("OAuth response unavailable")
                })? {
                    if bytes.len().saturating_add(chunk.len()) > MAX_OAUTH_BYTES {
                        self.diagnostic
                            .emit(stage, Event::ResponseLimit, None, None);
                        return Err(OAuthHttpClientError::new("OAuth response exceeds limit"));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                if (200..300).contains(&response.status) && parts.method == "GET" {
                    self.target
                        .observe(&endpoint, &bytes)
                        .map_err(|_| OAuthHttpClientError::new("OAuth discovery unavailable"))?;
                }
                let mut result = oauth2::http::Response::builder().status(response.status);
                for header in response.headers {
                    result = result.header(header.name, header.value);
                }
                result
                    .body(bytes)
                    .map_err(|_| OAuthHttpClientError::new("OAuth response unavailable"))
            };
            tokio::select! { biased; _=self.cancellation.cancelled()=>Err(OAuthHttpClientError::new("OAuth operation cancelled")), value=operation=>value }
        })
    }
}

// A second check at the actual HTTP boundary prevents the fixed client's
// documented Keyring-read-error fallback from sending anonymous MCP POSTs.
// Public unauthenticated GET is retained only for OAuth challenge discovery.
pub struct TushareHttpClient {
    credential: Option<crate::credentials::HeaderCredential>,
    target: Target,
    permit: Option<Arc<crate::daily::HttpPermit>>,
}
impl Default for TushareHttpClient {
    fn default() -> Self {
        Self {
            target: Target::official(),
            permit: None,
            credential: None,
        }
    }
}
impl TushareHttpClient {
    fn call_diagnostic(&self, request: &HttpRequestParams) -> Diagnostic {
        if request.url == self.target.resource
            && request.method == "POST"
            && request
                .body
                .as_ref()
                .and_then(|body| serde_json::from_slice::<serde_json::Value>(&body.0).ok())
                .is_some_and(|value| {
                    value.get("method").and_then(serde_json::Value::as_str) == Some("tools/call")
                })
        {
            return self
                .permit
                .as_ref()
                .map(|permit| permit.diagnostic())
                .unwrap_or_default();
        }
        Diagnostic::default()
    }
    fn admitted(
        &self,
        mut request: HttpRequestParams,
    ) -> Result<HttpRequestParams, ExecServerError> {
        let safe =
            || ExecServerError::HttpRequest("Tushare transport admission unavailable".into());
        if !self.target.permits(&request.method, &request.url) {
            return Err(safe());
        }
        if let Some(credential) = &self.credential
            && (request.url != self.target.resource
                || (credential.query_name().is_none()
                    && !request.headers.iter().any(|h| {
                        h.name.eq_ignore_ascii_case(&credential.name) && h.value == credential.value
                    })))
        {
            return Err(safe());
        }
        if self.credential.is_none()
            && request.url == self.target.resource
            && request.method != "GET"
        {
            let authorized = request.headers.iter().any(|h| {
                h.name.eq_ignore_ascii_case("authorization")
                    && h.value
                        .strip_prefix("Bearer ")
                        .is_some_and(|token| !token.is_empty())
            });
            if !authorized {
                return Err(safe());
            }
        }
        if self.credential.is_none()
            && request.url == self.target.resource
            && request.method == "GET"
        {
            let session = request
                .headers
                .iter()
                .any(|h| h.name.eq_ignore_ascii_case("mcp-session-id"));
            let authorized = request.headers.iter().any(|h| {
                h.name.eq_ignore_ascii_case("authorization")
                    && h.value
                        .strip_prefix("Bearer ")
                        .is_some_and(|token| !token.is_empty())
            });
            if session && !authorized {
                return Err(safe());
            }
        }
        // Metadata clients have no permit. The daily client receives exactly
        // one permit after Broker approval, checked at the actual send boundary.
        if request.url == self.target.resource && request.method == "POST" {
            let message = request
                .body
                .as_ref()
                .and_then(|body| serde_json::from_slice::<serde_json::Value>(&body.0).ok());
            let method = message
                .as_ref()
                .and_then(|v| v.get("method"))
                .and_then(serde_json::Value::as_str);
            if method == Some("tools/call") {
                if !self
                    .permit
                    .as_ref()
                    .is_some_and(|permit| permit.take(message.as_ref().expect("method")))
                {
                    return Err(safe());
                }
            } else if !method.is_some_and(|method| {
                matches!(
                    method,
                    "initialize"
                        | "notifications/initialized"
                        | "tools/list"
                        | "ping"
                        | "notifications/cancelled"
                )
            }) {
                return Err(safe());
            }
        }
        if let Some(permit) = &self.permit {
            for header in &request.headers {
                if header.name.eq_ignore_ascii_case("authorization")
                    || self
                        .credential
                        .as_ref()
                        .is_some_and(|c| header.name.eq_ignore_ascii_case(&c.name))
                {
                    permit.1.remember(&header.value);
                }
            }
            if request.url != self.target.resource
                && let Some(body) = &request.body
            {
                for (key, value) in url::form_urlencoded::parse(&body.0) {
                    if matches!(key.as_ref(), "refresh_token" | "client_secret") {
                        permit.1.remember(&value);
                    }
                }
            }
        }
        if let Some(config) = &self.target.google_client {
            if let Some(permit) = &self.permit {
                permit.1.remember(&config.client_secret);
            }
            if request.method == "POST" && request.url == crate::google_calendar::TOKEN {
                let body = request.body.as_ref().ok_or_else(safe)?;
                request
                    .headers
                    .retain(|header| !header.name.eq_ignore_ascii_case("content-length"));
                request.body = Some(
                    crate::google_calendar::token_body(config, &body.0)
                        .map_err(|_| safe())?
                        .into(),
                );
            }
        }
        if let Some(credential) = &self.credential
            && credential.query_name().is_some()
        {
            request.url = credential
                .resource_url(&self.target.resource)
                .map_err(|_| safe())?;
            if let Some(permit) = &self.permit {
                permit.1.remember(&credential.value);
                permit.1.remember(&request.url);
                let encoded = url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("value", &credential.value)
                    .finish();
                permit
                    .1
                    .remember(encoded.strip_prefix("value=").ok_or_else(safe)?);
            }
        }
        request.redirect_policy = HttpRedirectPolicy::Stop;
        request.timeout_ms = Some(request.timeout_ms.unwrap_or(15_000).min(15_000));
        Ok(request)
    }
}
impl HttpClient for TushareHttpClient {
    fn http_request(
        &self,
        request: HttpRequestParams,
    ) -> Pin<Box<dyn Future<Output = Result<HttpRequestResponse, ExecServerError>> + Send + '_>>
    {
        Box::pin(async move {
            let diagnostic = self.call_diagnostic(&request);
            let request = self.admitted(request).inspect_err(|_| {
                diagnostic.emit(Stage::DailyHttpAdmission, Event::PolicyRejected, None, None);
            })?;
            diagnostic.emit(Stage::DailyHttpSend, Event::Request, None, None);
            let endpoint = request.url.clone();
            let method = request.method.clone();
            let response = ReqwestHttpClient.http_request(request).await.map_err(|_| {
                diagnostic.emit(
                    Stage::DailyHttpResponse,
                    Event::TransportUnavailable,
                    None,
                    None,
                );
                ExecServerError::HttpRequest("Tushare transport unavailable".into())
            })?;
            diagnostic.emit(
                Stage::DailyHttpResponse,
                Event::Response,
                Some(response.status),
                None,
            );
            if endpoint != self.target.resource
                && (200..300).contains(&response.status)
                && let (Some(permit), Ok(value)) = (
                    &self.permit,
                    serde_json::from_slice::<serde_json::Value>(&response.body.0),
                )
            {
                permit.1.remember_tokens(&value);
            }
            if method == "GET" && (200..300).contains(&response.status) {
                self.target
                    .observe(&endpoint, &response.body.0)
                    .map_err(|_| {
                        ExecServerError::HttpRequest("OAuth discovery unavailable".into())
                    })?;
            }
            Ok(response)
        })
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
            let diagnostic = self.call_diagnostic(&request);
            let request = self.admitted(request).inspect_err(|_| {
                diagnostic.emit(Stage::DailyHttpAdmission, Event::PolicyRejected, None, None);
            })?;
            diagnostic.emit(Stage::DailyHttpSend, Event::Request, None, None);
            let endpoint = request.url.clone();
            let result = ReqwestHttpClient
                .http_request_stream(request)
                .await
                .map_err(|_| {
                    diagnostic.emit(
                        Stage::DailyHttpResponse,
                        Event::TransportUnavailable,
                        None,
                        None,
                    );
                    ExecServerError::HttpRequest("Tushare transport unavailable".into())
                })?;
            diagnostic.emit(
                Stage::DailyHttpResponse,
                Event::Response,
                Some(result.0.status),
                None,
            );
            self.target
                .observe_challenges(&endpoint, &result.0)
                .map_err(|_| ExecServerError::HttpRequest("OAuth discovery unavailable".into()))?;
            // The fixed public stream type has no wrapper constructor. Body/
            // protocol failures remain visible as the safe client-result class.
            Ok(result)
        })
    }
}

pub async fn connect_for_metadata(
    alias: &str,
    permit: Option<Arc<crate::daily::HttpPermit>>,
) -> Result<codex_rmcp_client::RmcpClient, Failure> {
    connect_for_service(alias, "tushareMcp", permit).await
}
pub async fn connect_for_service(
    alias: &str,
    service: &str,
    permit: Option<Arc<crate::daily::HttpPermit>>,
) -> Result<codex_rmcp_client::RmcpClient, Failure> {
    let mut target = Target::for_service(service)?;
    if service == crate::google_calendar::SERVICE {
        target.google_client = Some(Arc::new(
            crate::google_calendar::load(alias)?.ok_or(Failure::NotAuthorized)?,
        ));
    }
    if !keyring_readiness_for(alias, service).await? {
        return Err(Failure::NotAuthorized);
    }
    let credential = if crate::credentials::supports(service) {
        crate::credentials::load(alias, service)?
    } else {
        None
    };
    let headers = credential
        .as_ref()
        .filter(|credential| credential.query_name().is_none())
        .map(|c| HashMap::from([(c.name.clone(), c.value.clone())]));
    let client = codex_rmcp_client::RmcpClient::new_streamable_http_client(
        alias,
        &target.resource.clone(),
        None,
        headers,
        None,
        OAuthCredentialsStoreMode::Keyring,
        AuthKeyringBackendKind::Direct,
        Arc::new(TushareHttpClient {
            permit,
            target,
            credential,
        }),
        None,
    )
    .await
    .map_err(|_| Failure::Unavailable)?;
    let outcome = client
        .initialize(
            rmcp::model::InitializeRequestParams::new(
                rmcp::model::ClientCapabilities::default(),
                rmcp::model::Implementation::new("yijie-market-provider", "0.2.0"),
            ),
            Some(Duration::from_secs(15)),
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
    if outcome.is_err() {
        client.shutdown().await;
        return Err(Failure::Unavailable);
    }
    Ok(client)
}

/// Storage is only substituted by an in-process memory implementation in tests.
/// Production exclusively calls fixed Codex Keyring functions and never File/Auto.
trait Vault: Send + Sync {
    fn save(&self, alias: &str, tokens: &StoredOAuthTokens) -> Result<(), Failure>;
    fn remove(&self, alias: &str) -> Result<(), Failure>;
}
struct KeyringVault {
    resource: String,
}
impl Vault for KeyringVault {
    fn remove(&self, alias: &str) -> Result<(), Failure> {
        forget_keyring_resource(alias, &self.resource)
    }
    fn save(&self, alias: &str, tokens: &StoredOAuthTokens) -> Result<(), Failure> {
        validate_library_home()?;
        codex_rmcp_client::save_oauth_tokens(
            alias,
            tokens,
            OAuthCredentialsStoreMode::Keyring,
            AuthKeyringBackendKind::Direct,
        )
        .map_err(|_| Failure::VaultUnavailable)
    }
}
pub fn validate_library_home() -> Result<PathBuf, Failure> {
    let home = std::env::var_os("YIJIE_MARKET_WORKER_HOME")
        .map(PathBuf::from)
        .ok_or(Failure::VaultUnavailable)?;
    if !home.is_absolute()
        || home.file_name() != Some(std::ffi::OsStr::new("library-home"))
        || home.parent().and_then(Path::file_name) != Some(std::ffi::OsStr::new("market-worker"))
        || std::env::var_os("CODEX_HOME").map(PathBuf::from).as_ref() != Some(&home)
        || !home.is_dir()
    {
        return Err(Failure::VaultUnavailable);
    }
    Ok(home)
}
pub fn credential_alias(
    owner: &str,
    tenant: &str,
    installation: &str,
    generation: i64,
    credential_ref: &str,
) -> String {
    let scope = serde_json::to_string(&(owner, tenant)).expect("scope identity");
    let connection = serde_json::to_string(&(installation, generation, credential_ref))
        .expect("connection identity");
    crate::native_library::account_alias(&scope, &connection)
}
pub async fn keyring_readiness(alias: &str) -> Result<bool, Failure> {
    keyring_readiness_for(alias, "tushareMcp").await
}
pub async fn keyring_readiness_for(alias: &str, service: &str) -> Result<bool, Failure> {
    if service == crate::google_calendar::SERVICE && crate::google_calendar::load(alias)?.is_none()
    {
        return Ok(false);
    }
    if crate::credentials::supports(service) {
        return Ok(crate::credentials::load(alias, service)?.is_some());
    }
    let target = Target::for_service(service)?;
    validate_library_home()?;
    let state = codex_rmcp_client::determine_streamable_http_auth_status_with_http_client(
        alias,
        &target.resource.clone(),
        None,
        None,
        None,
        OAuthCredentialsStoreMode::Keyring,
        AuthKeyringBackendKind::Direct,
        Arc::new(TushareHttpClient {
            target,
            credential: None,
            permit: None,
        }),
    )
    .await
    .map_err(|_| Failure::VaultUnavailable)?;
    Ok(matches!(state, McpAuthState::OAuth))
}
pub fn forget_keyring(alias: &str) -> Result<(), Failure> {
    forget_keyring_resource(alias, TUSHARE_RESOURCE)
}
pub fn forget_for_service(alias: &str, service: &str) -> Result<(), Failure> {
    if service == crate::google_calendar::SERVICE {
        crate::google_calendar::forget(alias)?;
    }
    if crate::credentials::supports(service) {
        return crate::credentials::forget(alias);
    }
    forget_keyring_resource(alias, &Target::for_service(service)?.resource)
}
fn forget_keyring_resource(alias: &str, resource: &str) -> Result<(), Failure> {
    validate_library_home()?;
    codex_rmcp_client::delete_oauth_tokens(
        alias,
        resource,
        OAuthCredentialsStoreMode::Keyring,
        AuthKeyringBackendKind::Direct,
    )
    .map_err(|_| Failure::VaultUnavailable)?;
    Ok(())
}

struct Callback {
    code: String,
    state: String,
    issuer: Option<String>,
}
type CallbackReply = oneshot::Sender<Result<Callback, Failure>>;
#[derive(Clone)]
struct CallbackSender(Arc<Mutex<Option<CallbackReply>>>);
async fn callback(
    State(sender): State<CallbackSender>,
    Query(values): Query<HashMap<String, String>>,
) -> (StatusCode, &'static str) {
    let decoded = match (values.get("code"), values.get("state")) {
        (Some(code), Some(state))
            if !code.is_empty()
                && code.len() <= 4096
                && !state.is_empty()
                && state.len() <= 1024 =>
        {
            Ok(Callback {
                code: code.clone(),
                state: state.clone(),
                issuer: values.get("iss").cloned(),
            })
        }
        _ => Err(Failure::NotAuthorized),
    };
    let sent = sender
        .0
        .lock()
        .expect("callback owner")
        .take()
        .is_some_and(|sender| sender.send(decoded).is_ok());
    if sent {
        (
            StatusCode::OK,
            "Authorization response received. You can return to Yijie.",
        )
    } else {
        (StatusCode::GONE, "Authorization is no longer pending.")
    }
}

#[derive(Clone)]
pub struct AuthControl {
    observation: Arc<Mutex<Observation>>,
    cancellation: CancellationToken,
}
pub struct AuthRun {
    control: AuthControl,
    task: Option<JoinHandle<()>>,
}
impl AuthControl {
    pub fn observe(&self) -> Observation {
        self.observation.lock().expect("auth state").clone()
    }
    pub fn cancel(&self) -> Observation {
        let mut state = self.observation.lock().expect("auth state");
        if matches!(state.phase, Phase::Starting | Phase::AwaitingUser) {
            self.cancellation.cancel();
            *state = Observation::failed(Failure::Cancelled);
        }
        state.clone()
    }
}
impl AuthRun {
    pub fn start(
        alias: String,
        remaining_scope: Duration,
        diagnostic: Diagnostic,
    ) -> Result<Self, Failure> {
        Self::start_for_service(alias, "tushareMcp", remaining_scope, diagnostic)
    }
    pub fn start_for_service(
        alias: String,
        service: &str,
        remaining_scope: Duration,
        diagnostic: Diagnostic,
    ) -> Result<Self, Failure> {
        validate_library_home()?;
        if crate::credentials::supports(service) {
            return Self::start_configuration(alias, service.into(), remaining_scope);
        }
        let mut target = Target::for_service(service)?;
        if service == crate::google_calendar::SERVICE {
            let Some(config) = crate::google_calendar::load(&alias)? else {
                return Self::start_configuration(alias, service.into(), remaining_scope);
            };
            target.google_client = Some(Arc::new(config));
        }
        let vault = Arc::new(KeyringVault {
            resource: target.resource.clone(),
        });
        Self::start_with(
            alias,
            remaining_scope,
            target,
            Arc::new(ReqwestHttpClient),
            vault,
            diagnostic,
        )
    }
    fn start_configuration(
        alias: String,
        service: String,
        remaining: Duration,
    ) -> Result<Self, Failure> {
        let ttl = remaining.min(MAX_FLOW_TTL);
        if ttl.is_zero() {
            return Err(Failure::Expired);
        }
        let observation = Arc::new(Mutex::new(Observation {
            phase: Phase::Starting,
            authorization_url: None,
            failure: None,
        }));
        let cancellation = CancellationToken::new();
        let view = observation.clone();
        let cancelled = cancellation.clone();
        let task = tokio::spawn(async move {
            if let Err(failure) = crate::credentials::configure(
                &alias,
                &service,
                Instant::now() + ttl,
                &view,
                &cancelled,
            )
            .await
            {
                let mut state = view.lock().expect("configuration");
                if !matches!(state.phase, Phase::Succeeded | Phase::Cancelled) {
                    *state = Observation::failed(failure);
                }
            }
        });
        Ok(Self {
            control: AuthControl {
                observation,
                cancellation,
            },
            task: Some(task),
        })
    }
    fn start_with(
        alias: String,
        remaining_scope: Duration,
        target: Target,
        http: Arc<dyn HttpClient>,
        vault: Arc<dyn Vault>,
        diagnostic: Diagnostic,
    ) -> Result<Self, Failure> {
        let ttl = remaining_scope.min(MAX_FLOW_TTL);
        if ttl.is_zero() {
            return Err(Failure::Expired);
        }
        let deadline = Instant::now() + ttl;
        let observation = Arc::new(Mutex::new(Observation {
            phase: Phase::Starting,
            authorization_url: None,
            failure: None,
        }));
        let cancellation = CancellationToken::new();
        let view = observation.clone();
        let cancelled = cancellation.clone();
        let task = tokio::spawn(async move {
            let result = run_flow(
                &alias,
                deadline,
                target,
                http,
                vault,
                &view,
                &cancelled,
                &diagnostic,
            )
            .await;
            if let Err(error) = result {
                let mut state = view.lock().expect("auth state");
                if !matches!(state.phase, Phase::Succeeded | Phase::Cancelled) {
                    *state = Observation::failed(error);
                }
            }
        });
        Ok(Self {
            control: AuthControl {
                observation,
                cancellation,
            },
            task: Some(task),
        })
    }
    pub fn control(&self) -> AuthControl {
        self.control.clone()
    }
    pub fn observe(&self) -> Observation {
        self.control.observe()
    }
    pub fn cancel(&self) -> Observation {
        self.control.cancel()
    }
    pub async fn close(&mut self) -> Result<(), Failure> {
        self.cancel();
        let Some(mut task) = self.task.take() else {
            return Ok(());
        };
        match tokio::time::timeout(Duration::from_secs(5), &mut task).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(Failure::Unavailable),
            Err(_) => {
                self.task = Some(task);
                Err(Failure::CleanupPending)
            }
        }
    }
}
pub(crate) async fn authorize_google_configured(
    alias: &str,
    deadline: Instant,
    view: &Arc<Mutex<Observation>>,
    cancel: &CancellationToken,
) -> Result<(), Failure> {
    let mut target = Target::for_service(crate::google_calendar::SERVICE)?;
    target.google_client = Some(Arc::new(
        crate::google_calendar::load(alias)?.ok_or(Failure::NotAuthorized)?,
    ));
    let vault = Arc::new(KeyringVault {
        resource: target.resource.clone(),
    });
    run_flow(
        alias,
        deadline,
        target,
        Arc::new(ReqwestHttpClient),
        vault,
        view,
        cancel,
        &Diagnostic::default(),
    )
    .await
}
#[allow(clippy::too_many_arguments)]
async fn run_flow(
    alias: &str,
    deadline: Instant,
    target: Target,
    http: Arc<dyn HttpClient>,
    vault: Arc<dyn Vault>,
    view: &Arc<Mutex<Observation>>,
    cancelled: &CancellationToken,
    diagnostic: &Diagnostic,
) -> Result<(), Failure> {
    diagnostic.emit(Stage::CallbackBind, Event::Started, None, None);
    let google = target.resource == crate::google_calendar::RESOURCE;
    let listener = tokio::net::TcpListener::bind(if google {
        crate::google_calendar::CALLBACK_BIND
    } else {
        "127.0.0.1:0"
    })
    .await
    .map_err(|_| Failure::Unavailable)?;
    diagnostic.emit(Stage::CallbackBind, Event::Succeeded, None, None);
    let callback_path = if google {
        crate::google_calendar::CALLBACK_PATH.into()
    } else {
        format!("/oauth/callback/{}", Uuid::new_v4())
    };
    let callback_url = format!(
        "http://{}{}",
        listener.local_addr().map_err(|_| Failure::Unavailable)?,
        callback_path
    );
    let (tx, rx) = oneshot::channel();
    let server_stop = CancellationToken::new();
    let stop = server_stop.clone();
    let router = Router::new()
        .route(&callback_path, get(callback))
        .with_state(CallbackSender(Arc::new(Mutex::new(Some(tx)))));
    let mut server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(stop.cancelled_owned())
            .await
    });
    let flow = async {
        let oauth_http = Arc::new(OAuthHttp {
            target: target.clone(),
            http,
            cancellation: cancelled.clone(),
            diagnostic: diagnostic.clone(),
        });
        diagnostic.emit(Stage::ManagerNew, Event::Started, None, None);
        let mut manager = diagnostic.result(
            Stage::ManagerNew,
            AuthorizationManager::new_with_oauth_http_client(&target.resource, oauth_http)
                .await
                .map_err(|_| Failure::Unavailable),
        )?;
        diagnostic.emit(Stage::Discovery, Event::Started, None, None);
        let metadata = if google {
            crate::google_calendar::metadata()
        } else {
            diagnostic.result(
                Stage::Discovery,
                manager
                    .discover_metadata()
                    .await
                    .map_err(|_| Failure::Unavailable),
            )?
        };
        diagnostic.result(
            Stage::MetadataValidation,
            target.validate_metadata(&metadata),
        )?;
        manager.set_metadata(metadata);
        diagnostic.emit(Stage::Registration, Event::Started, None, None);
        let session = if google {
            let config = target
                .google_client
                .as_ref()
                .ok_or(Failure::NotAuthorized)?;
            crate::google_calendar::session(manager, config, &callback_url).await?
        } else {
            diagnostic.result(
                Stage::Registration,
                AuthorizationSession::new(
                    manager,
                    &if target.policy.is_some() {
                        vec![]
                    } else {
                        vec![TUSHARE_SCOPE]
                    },
                    &callback_url,
                    Some("Yijie Market"),
                    None,
                )
                .await
                .map_err(|_| Failure::Unavailable),
            )?
        };
        diagnostic.result(
            Stage::AuthorizationUrlValidation,
            target.validate_authorization_url(session.get_authorization_url(), &callback_url),
        )?;
        {
            let mut state = view.lock().expect("auth state");
            if cancelled.is_cancelled() || !matches!(state.phase, Phase::Starting) {
                return Err(Failure::Cancelled);
            }
            state.phase = Phase::AwaitingUser;
            state.authorization_url = Some(session.get_authorization_url().into());
        }
        diagnostic.emit(Stage::AwaitingCallback, Event::Started, None, None);
        let callback = rx.await.map_err(|_| Failure::Unavailable)??;
        diagnostic.emit(Stage::TokenExchange, Event::Started, None, None);
        session
            .handle_callback_with_issuer(
                &callback.code,
                &callback.state,
                callback.issuer.as_deref(),
            )
            .await
            .map_err(|error| {
                // rmcp preserves callback validation as typed auth errors but
                // folds token transport/server failures into TokenExchangeFailed.
                // Those failures do not establish that the user denied access.
                let failure = match error {
                    rmcp::transport::auth::AuthError::TokenExchangeFailed(_)
                    | rmcp::transport::auth::AuthError::HttpError(_)
                    | rmcp::transport::auth::AuthError::InternalError(_) => Failure::Unavailable,
                    _ => Failure::NotAuthorized,
                };
                diagnostic.emit(Stage::TokenExchange, Event::Failed, None, Some(failure));
                failure
            })?;
        diagnostic.emit(Stage::TokenExchange, Event::Succeeded, None, None);
        let (client_id, tokens) = session
            .get_credentials()
            .await
            .map_err(|_| Failure::NotAuthorized)?;
        let tokens = tokens.ok_or(Failure::NotAuthorized)?;
        if client_id.is_empty() || tokens.access_token().secret().is_empty() {
            return Err(Failure::NotAuthorized);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Failure::Unavailable)?
            .as_millis() as u64;
        let expires_at = tokens
            .expires_in()
            .map(|ttl| now.saturating_add(ttl.as_millis() as u64));
        let stored = StoredOAuthTokens {
            server_name: alias.into(),
            url: target.resource.clone(),
            client_id,
            token_response: WrappedOAuthTokenResponse(tokens),
            expires_at,
        };
        // Cancellation and Keyring commit share one state lock. An accepted
        // cancel cannot race a late callback into saving/re-enabling credentials.
        let mut state = view.lock().expect("auth state");
        if cancelled.is_cancelled() || !matches!(state.phase, Phase::AwaitingUser) {
            return Err(Failure::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(Failure::Expired);
        }
        diagnostic.emit(Stage::KeyringCommit, Event::Started, None, None);
        diagnostic.result(Stage::KeyringCommit, vault.save(alias, &stored))?;
        if Instant::now() >= deadline {
            vault.remove(alias).map_err(|_| Failure::CleanupPending)?;
            return Err(Failure::Expired);
        }
        state.phase = Phase::Succeeded;
        state.authorization_url = None;
        state.failure = None;
        Ok(())
    };
    let result = tokio::select! {biased;_=cancelled.cancelled()=>Err(Failure::Cancelled),_=tokio::time::sleep_until(deadline.into())=>Err(Failure::Expired),result=flow=>result};
    diagnostic.emit(
        Stage::Cleanup,
        Event::Started,
        None,
        result.as_ref().err().copied(),
    );
    server_stop.cancel();
    loop {
        match tokio::time::timeout(Duration::from_secs(5), &mut server).await {
            Ok(Ok(Ok(()))) => {
                diagnostic.emit(Stage::Cleanup, Event::Succeeded, None, None);
                return result;
            }
            Ok(_) => return Err(Failure::Unavailable),
            Err(_) => {
                // Retain the actual callback owner until its graceful shutdown
                // joins. A pending cleanup must never detach a live listener.
                let mut state = view.lock().expect("auth state");
                if state.phase != Phase::Succeeded {
                    *state = Observation::failed(Failure::CleanupPending);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, extract::Form, routing::post};
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[test]
    fn diagnostic_is_bounded_to_enum_and_numeric_fields() {
        let path =
            std::env::temp_dir().join(format!("yijie-normal-diagnostic-{}.jsonl", Uuid::new_v4()));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let diagnostic = Diagnostic(Some(Arc::new(Mutex::new(DiagnosticFile {
            file,
            count: 0,
        }))));
        for _ in 0..140 {
            diagnostic.emit(Stage::Registration, Event::Response, Some(201), None);
        }
        drop(diagnostic);
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(text.lines().count(), 128);
        for line in text.lines() {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(value.as_object().unwrap().len(), 6);
            assert_eq!(value["stage"], "registration");
            assert_eq!(value["event"], "response");
            assert_eq!(value["httpStatus"], 201);
            assert!(line.len() < 512);
        }
    }
    #[test]
    fn metadata_client_requires_owned_authorization_and_never_calls_tools() {
        let request = |method: &str, authorized: bool| HttpRequestParams {
            method: "POST".into(),
            url: TUSHARE_RESOURCE.into(),
            headers: if authorized {
                vec![HttpHeader {
                    name: "authorization".into(),
                    value: "Bearer ordinary-local-test".into(),
                }]
            } else {
                vec![]
            },
            body: Some(
                serde_json::to_vec(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":method}))
                    .unwrap()
                    .into(),
            ),
            timeout_ms: Some(60_000),
            redirect_policy: HttpRedirectPolicy::Stop,
            request_id: Uuid::new_v4().to_string(),
            stream_response: false,
        };
        assert!(
            TushareHttpClient::default()
                .admitted(request("tools/list", false))
                .is_err()
        );
        assert!(
            TushareHttpClient::default()
                .admitted(request("tools/call", true))
                .is_err()
        );
        let admitted = TushareHttpClient::default()
            .admitted(request("tools/list", true))
            .unwrap();
        assert_eq!(admitted.timeout_ms, Some(15_000));
    }
    #[test]
    fn normal_zerone_query_is_injected_after_single_call_admission() {
        let credential:crate::credentials::HeaderCredential=serde_json::from_value(serde_json::json!({"serviceId":"zerone","name":"api_key","value":"ordinary-local-value"})).unwrap();
        let permit = Arc::new(crate::daily::HttpPermit::default());
        let client = TushareHttpClient {
            credential: Some(credential),
            target: Target::for_service("zerone").unwrap(),
            permit: Some(permit.clone()),
        };
        let args = serde_json::json!({"name":"普通合成企业"});
        permit.arm(
            crate::daily::Permit {
                tool: "entity_profile".into(),
                arguments: args.clone(),
                request: CancellationToken::new(),
                lease: CancellationToken::new(),
                backend: CancellationToken::new(),
                deadline: Instant::now() + Duration::from_secs(30),
            },
            Diagnostic::default(),
        );
        let request = || {
            HttpRequestParams {method:"POST".into(),url:client.target.resource.clone(),headers:vec![],body:Some(serde_json::to_vec(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"entity_profile","arguments":args}})).unwrap().into()),timeout_ms:None,redirect_policy:HttpRedirectPolicy::Stop,request_id:Uuid::new_v4().to_string(),stream_response:true}
        };
        let admitted = client.admitted(request()).unwrap();
        assert_eq!(
            admitted.url,
            "https://ai.zerone.com.cn/mcp/pe?api_key=ordinary-local-value"
        );
        assert!(admitted.headers.is_empty());
        assert_eq!(client.target.resource, "https://ai.zerone.com.cn/mcp/pe");
        assert_eq!(admitted.timeout_ms, Some(15_000));
        assert!(client.admitted(request()).is_err());
    }
    #[derive(Default)]
    struct MemoryVault {
        saved: Mutex<Option<StoredOAuthTokens>>,
        writes: AtomicUsize,
    }
    impl Vault for MemoryVault {
        fn remove(&self, _alias: &str) -> Result<(), Failure> {
            *self.saved.lock().unwrap() = None;
            Ok(())
        }
        fn save(&self, _alias: &str, tokens: &StoredOAuthTokens) -> Result<(), Failure> {
            *self.saved.lock().unwrap() = Some(tokens.clone());
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    #[derive(Clone)]
    struct LocalProvider {
        base: String,
        registrations: Arc<AtomicUsize>,
        exchanges: Arc<AtomicUsize>,
    }
    async fn challenge(
        State(provider): State<LocalProvider>,
    ) -> (StatusCode, [(String, String); 1], Json<serde_json::Value>) {
        (
            StatusCode::UNAUTHORIZED,
            [(
                "www-authenticate".into(),
                format!(
                    "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource/mcp/\"",
                    provider.base
                ),
            )],
            Json(serde_json::json!({"error":"missing_token"})),
        )
    }
    async fn resource_metadata(State(provider): State<LocalProvider>) -> Json<serde_json::Value> {
        Json(
            serde_json::json!({"resource":format!("{}/mcp/",provider.base),"authorization_servers":[provider.base]}),
        )
    }
    async fn auth_metadata(State(provider): State<LocalProvider>) -> Json<serde_json::Value> {
        Json(
            serde_json::json!({"issuer":provider.base,"authorization_endpoint":format!("{}/oauth/authorize",provider.base),"token_endpoint":format!("{}/oauth/token",provider.base),"registration_endpoint":format!("{}/oauth/register",provider.base),"response_types_supported":["code"],"grant_types_supported":["authorization_code","refresh_token"],"token_endpoint_auth_methods_supported":["none"],"code_challenge_methods_supported":["S256"],"scopes_supported":[TUSHARE_SCOPE]}),
        )
    }
    async fn registration(
        State(provider): State<LocalProvider>,
        Json(request): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        provider.registrations.fetch_add(1, Ordering::SeqCst);
        Json(
            serde_json::json!({"client_id":"ordinary-local-client","redirect_uris":request["redirect_uris"],"grant_types":["authorization_code","refresh_token"],"response_types":["code"],"token_endpoint_auth_method":"none"}),
        )
    }
    async fn exchange(
        State(provider): State<LocalProvider>,
        Form(request): Form<HashMap<String, String>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        provider.exchanges.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            request.get("grant_type").map(String::as_str),
            Some("authorization_code")
        );
        assert!(request.get("code_verifier").is_some_and(|v| !v.is_empty()));
        assert!(
            request
                .get("code")
                .is_some_and(|v| v == "ordinary-local-code")
        );
        (
            StatusCode::OK,
            Json(
                serde_json::json!({"access_token":"ordinary-local-access","refresh_token":"ordinary-local-refresh","token_type":"Bearer","expires_in":3600,"scope":TUSHARE_SCOPE}),
            ),
        )
    }
    async fn local_provider() -> (
        LocalProvider,
        Target,
        CancellationToken,
        JoinHandle<std::io::Result<()>>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let provider = LocalProvider {
            base: base.clone(),
            registrations: Arc::new(AtomicUsize::new(0)),
            exchanges: Arc::new(AtomicUsize::new(0)),
        };
        let app = Router::new()
            .route("/mcp/", get(challenge))
            .route(
                "/.well-known/oauth-protected-resource/mcp/",
                get(resource_metadata),
            )
            .route(
                "/.well-known/oauth-protected-resource/mcp",
                get(resource_metadata),
            )
            .route(
                "/.well-known/oauth-protected-resource",
                get(resource_metadata),
            )
            .route(
                "/.well-known/oauth-authorization-server",
                get(auth_metadata),
            )
            .route("/oauth/register", post(registration))
            .route("/oauth/token", post(exchange))
            .with_state(provider.clone());
        let stop = CancellationToken::new();
        let done = stop.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(done.cancelled_owned())
                .await
        });
        (
            provider,
            Target {
                resource: format!("{base}/mcp/"),
                issuer: base,
                policy: None,
                google_client: None,
            },
            stop,
            task,
        )
    }
    async fn wait_phase(run: &AuthRun, expected: Phase) -> Observation {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let view = run.observe();
                if view.phase == expected {
                    return view;
                }
                assert!(
                    !matches!(view.phase, Phase::Failed),
                    "ordinary OAuth operation failed: {:?}",
                    view.failure
                );
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("normal OAuth phase should finish")
    }
    async fn provider_close(stop: CancellationToken, task: JoinHandle<std::io::Result<()>>) {
        stop.cancel();
        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
    #[tokio::test]
    async fn ordinary_oauth_pkce_callback_saves_only_inside_owned_vault() {
        let (provider, target, stop, task) = local_provider().await;
        let vault = Arc::new(MemoryVault::default());
        let mut run = AuthRun::start_with(
            "ordinary-local-namespace".into(),
            Duration::from_secs(30),
            target,
            Arc::new(ReqwestHttpClient),
            vault.clone(),
            Diagnostic::default(),
        )
        .unwrap();
        let ready = wait_phase(&run, Phase::AwaitingUser).await;
        let authorize = Url::parse(ready.authorization_url.as_ref().unwrap()).unwrap();
        let pairs: HashMap<_, _> = authorize
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert_eq!(
            pairs.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        let mut callback = Url::parse(pairs.get("redirect_uri").unwrap()).unwrap();
        callback
            .query_pairs_mut()
            .append_pair("code", "ordinary-local-code")
            .append_pair("state", pairs.get("state").unwrap())
            .append_pair("iss", &provider.base);
        let response = ReqwestHttpClient
            .http_request(HttpRequestParams {
                method: "GET".into(),
                url: callback.to_string(),
                headers: vec![],
                body: None,
                timeout_ms: Some(3000),
                redirect_policy: HttpRedirectPolicy::Stop,
                request_id: Uuid::new_v4().to_string(),
                stream_response: false,
            })
            .await
            .unwrap();
        assert_eq!(response.status, 200);
        let completed = wait_phase(&run, Phase::Succeeded).await;
        assert!(completed.authorization_url.is_none());
        assert_eq!(vault.writes.load(Ordering::SeqCst), 1);
        assert_eq!(provider.registrations.load(Ordering::SeqCst), 1);
        assert_eq!(provider.exchanges.load(Ordering::SeqCst), 1);
        {
            let stored = vault.saved.lock().unwrap();
            let tokens = stored.as_ref().unwrap();
            assert!(tokens.token_response.0.access_token().secret() == "ordinary-local-access");
            assert!(tokens.expires_at.is_some());
        }
        run.close().await.unwrap();
        provider_close(stop, task).await;
    }
    #[tokio::test]
    async fn ordinary_cancel_stops_callback_and_never_commits_tokens() {
        let (provider, target, stop, task) = local_provider().await;
        let vault = Arc::new(MemoryVault::default());
        let mut run = AuthRun::start_with(
            "ordinary-local-namespace".into(),
            Duration::from_secs(30),
            target,
            Arc::new(ReqwestHttpClient),
            vault.clone(),
            Diagnostic::default(),
        )
        .unwrap();
        wait_phase(&run, Phase::AwaitingUser).await;
        assert_eq!(run.cancel().phase, Phase::Cancelled);
        run.close().await.unwrap();
        assert_eq!(vault.writes.load(Ordering::SeqCst), 0);
        assert_eq!(provider.exchanges.load(Ordering::SeqCst), 0);
        provider_close(stop, task).await;
    }
    #[tokio::test]
    async fn ordinary_owner_close_before_start_does_not_register_or_save() {
        let (provider, target, stop, task) = local_provider().await;
        let vault = Arc::new(MemoryVault::default());
        let mut run = AuthRun::start_with(
            "ordinary-local-namespace".into(),
            Duration::from_secs(30),
            target,
            Arc::new(ReqwestHttpClient),
            vault.clone(),
            Diagnostic::default(),
        )
        .unwrap();
        run.close().await.unwrap();
        assert_eq!(vault.writes.load(Ordering::SeqCst), 0);
        assert_eq!(provider.registrations.load(Ordering::SeqCst), 0);
        assert_eq!(provider.exchanges.load(Ordering::SeqCst), 0);
        provider_close(stop, task).await;
    }
    #[test]
    fn persistent_alias_excludes_runtime_epochs_and_preserves_installation_isolation() {
        let alias = credential_alias("owner-a", "tenant-a", "installation-a", 1, "cref-a");
        assert_eq!(
            alias,
            credential_alias("owner-a", "tenant-a", "installation-a", 1, "cref-a")
        );
        assert_ne!(
            alias,
            credential_alias("owner-a", "tenant-a", "installation-a", 2, "cref-a")
        );
        assert_ne!(
            alias,
            credential_alias("owner-a", "tenant-b", "installation-a", 1, "cref-a")
        );
        assert!(!alias.contains("owner-a"));
    }
}
