//! Non-OAuth secrets enter through an owned, one-shot loopback form, and are
//! stored by the fixed Codex KeyringStore. No secret crosses Native/Host IPC.
use crate::{
    provider_registry,
    tushare_oauth::{Failure, Observation, Phase},
};
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use codex_keyring_store::{DefaultKeyringStore, KeyringStore};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const STORE: &str = "Yijie Market Credentials";
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeaderCredential {
    service_id: String,
    pub name: String,
    pub value: String,
}
impl HeaderCredential {
    pub fn query_name(&self) -> Option<&str> {
        provider_registry::get(&self.service_id).and_then(|spec| spec.credential_query.as_deref())
    }
    pub fn resource_url(&self, resource: &str) -> Result<String, Failure> {
        let spec = provider_registry::get(&self.service_id).ok_or(Failure::InvalidMetadata)?;
        if spec.resource.as_deref() != Some(resource) {
            return Err(Failure::InvalidMetadata);
        }
        validate(&self.service_id, &self.name, &self.value)?;
        let mut url = url::Url::parse(resource).map_err(|_| Failure::InvalidMetadata)?;
        if let Some(name) = self.query_name() {
            url.query_pairs_mut().append_pair(name, &self.value);
        }
        Ok(url.into())
    }
}
pub fn supports(service: &str) -> bool {
    provider_registry::get(service).is_some_and(|s| {
        s.transport == "http" && matches!(s.auth_mode.as_str(), "api_key" | "provider_credentials")
    })
}
fn validate(service: &str, name: &str, value: &str) -> Result<(), Failure> {
    if !supports(service)
        || name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || value.is_empty()
        || value.len() > 4096
        || value.chars().any(|c| c.is_control())
    {
        return Err(Failure::InvalidMetadata);
    }
    let lower = name.to_ascii_lowercase();
    if [
        "host",
        "connection",
        "content-length",
        "content-type",
        "transfer-encoding",
        "upgrade",
        "cookie",
        "origin",
        "referer",
        "user-agent",
        "mcp-session-id",
        "mcp-protocol-version",
    ]
    .contains(&lower.as_str())
        || lower.starts_with("sec-")
        || lower.starts_with("proxy-")
    {
        return Err(Failure::InvalidMetadata);
    }
    if provider_registry::get(service)
        .and_then(|s| s.credential_header.as_ref().or(s.credential_query.as_ref()))
        .is_some_and(|expected| !name.eq_ignore_ascii_case(expected))
    {
        return Err(Failure::InvalidMetadata);
    }
    Ok(())
}
pub fn load(alias: &str, service: &str) -> Result<Option<HeaderCredential>, Failure> {
    crate::tushare_oauth::validate_library_home()?;
    let value = DefaultKeyringStore
        .load(STORE, alias)
        .map_err(|_| Failure::VaultUnavailable)?;
    value
        .map(|value| {
            let credential: HeaderCredential =
                serde_json::from_str(&value).map_err(|_| Failure::VaultUnavailable)?;
            if credential.service_id != service {
                return Err(Failure::VaultUnavailable);
            }
            validate(service, &credential.name, &credential.value)?;
            Ok(credential)
        })
        .transpose()
}
pub fn forget(alias: &str) -> Result<(), Failure> {
    crate::tushare_oauth::validate_library_home()?;
    DefaultKeyringStore
        .delete(STORE, alias)
        .map_err(|_| Failure::VaultUnavailable)?;
    Ok(())
}
fn save(alias: &str, value: &HeaderCredential) -> Result<(), Failure> {
    crate::tushare_oauth::validate_library_home()?;
    validate(&value.service_id, &value.name, &value.value)?;
    let encoded = serde_json::to_string(value).map_err(|_| Failure::VaultUnavailable)?;
    DefaultKeyringStore
        .save(STORE, alias, &encoded)
        .map_err(|_| Failure::VaultUnavailable)
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
enum CredentialInput {
    Header(HeaderCredential),
    Google(crate::google_calendar::ClientConfig),
}
impl CredentialInput {
    fn save(&self, alias: &str) -> Result<(), Failure> {
        match self {
            Self::Header(c) => save(alias, c),
            Self::Google(c) => crate::google_calendar::save(alias, c),
        }
    }
    fn forget(&self, alias: &str) -> Result<(), Failure> {
        match self {
            Self::Header(_) => forget(alias),
            Self::Google(_) => crate::google_calendar::forget(alias),
        }
    }
}
#[derive(Clone)]
struct FormState {
    service: String,
    origin: String,
    host: String,
    path: String,
    nonce: String,
    deadline: Instant,
    cancellation: CancellationToken,
    sender: Arc<Mutex<Option<oneshot::Sender<CredentialInput>>>>,
    view: Arc<Mutex<Observation>>,
}
fn reply(status: StatusCode, text: String) -> Response {
    let mut response = (status, Html(text)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    headers.insert("content-security-policy","default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'".parse().unwrap());
    headers.insert("referrer-policy", "no-referrer".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    response
}
fn current(state: &FormState, headers: &HeaderMap) -> bool {
    !state.cancellation.is_cancelled()
        && Instant::now() < state.deadline
        && headers.get(header::HOST).and_then(|v| v.to_str().ok()) == Some(state.host.as_str())
}
async fn page(State(state): State<FormState>, headers: HeaderMap) -> Response {
    if !current(&state, &headers) || state.sender.lock().expect("configuration").is_none() {
        return reply(StatusCode::GONE, "配置窗口已失效，请返回易界。".into());
    }
    if state.service == crate::google_calendar::SERVICE {
        return reply(
            StatusCode::OK,
            format!(
                r#"<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>易界 Google 日历配置</title><style>body{{font:16px system-ui;max-width:600px;margin:8vh auto;padding:24px}}label{{display:block;margin:20px 0}}input:not([type=checkbox]),button{{width:100%;padding:10px;box-sizing:border-box}}button{{background:#d3f36b;border:1px solid #333;border-radius:8px}}</style><h1>配置 Google 日历</h1><p>需要加入 Google Workspace 开发者预览，并在 Google Cloud 项目启用 Calendar API 与 Calendar MCP API。</p><p>创建 Web OAuth 应用，并将授权重定向 URI 设置为：<code>{callback}</code></p><form method="post" action="{path}" autocomplete="off"><input type="hidden" name="nonce" value="{nonce}"><label>OAuth Client ID<input name="client_id" maxlength="256" required autocomplete="off"></label><label>OAuth Client Secret<input name="client_secret" type="password" maxlength="4096" required autocomplete="off"></label><label><input name="allow_event_updates" type="checkbox" value="yes">申请创建和修改日程权限（每次仍需批准；删除操作暂不开放）</label><button>保存并准备 Google 授权</button></form><p>应用配置与令牌仅保存在本机钥匙串。填写后继续前往 Google 完成账号授权。</p></html>"#,
                callback = crate::google_calendar::CALLBACK,
                path = state.path,
                nonce = state.nonce
            ),
        );
    }
    let spec = provider_registry::get(&state.service).expect("fixed registry");
    let name = spec
        .credential_header
        .as_deref()
        .or(spec.credential_query.as_deref())
        .unwrap_or("");
    let fixed = spec.credential_header.is_some() || spec.credential_query.is_some();
    let locked = if fixed { "readonly" } else { "" };
    let guidance = if state.service == "thinkingdata" {
        format!(
            r#"<p>在 Agentic Engine 的 MCP 详情中，使用“添加到客户端”或配置导出，取得与以下地址对应的远程 HTTP 配置：<code>{}</code>。按导出配置填写认证 Header 名称与完整值，选择“直接使用凭据值”。</p><p><a href="https://docs.thinkingai.cn/zh/manual/mcp" target="_blank" rel="noopener noreferrer">查看官方接入说明</a> · <a href="https://docs.thinkingai.cn/zh/manual/mcp_integration" target="_blank" rel="noopener noreferrer">认证与配置手册</a></p><p>可用工具和数据范围取决于当前 AE 账号的项目权限；保存后仍需完成连接检查。</p>"#,
            escape(spec.resource.as_deref().unwrap_or_default())
        )
    } else {
        String::new()
    };
    reply(
        StatusCode::OK,
        format!(
            r#"<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>易界连接器安全配置</title><style>body{{font:16px system-ui;max-width:600px;margin:10vh auto;padding:24px;color:#242424;background:#fff}}label{{display:block;margin:20px 0}}input,select,button{{box-sizing:border-box;width:100%;padding:12px;margin-top:8px;font:inherit}}button{{background:#d3f36b;border:1px solid #333;border-radius:8px}}small{{display:block;line-height:1.6}}</style><h1>配置 {title}</h1><p>凭据直接交给本机易界连接器并保存到系统钥匙串，不进入对话或模型。</p>{guidance}<form method="post" action="{path}" autocomplete="off"><input type="hidden" name="nonce" value="{nonce}"><label>{field_label}<input name="name" value="{name}" maxlength="128" required {locked} autocomplete="off"></label><small>请按服务方提供的接入文档填写。固定参数由易界提供，请直接填写凭据值。</small><label>凭据格式<select name="scheme"><option value="raw">直接使用凭据值</option>{bearer_option}</select></label><label>凭据<input name="value" type="password" maxlength="4096" required autocomplete="off"></label><button type="submit">保存到系统钥匙串</button></form><p>此窗口只保存配置，不代表连接或工具调用已成功。返回易界查看检查结果。</p></html>"#,
            title = escape(&spec.display_name),
            path = state.path,
            nonce = state.nonce,
            name = escape(name),
            field_label = if spec.credential_query.is_some() {
                "认证参数名称"
            } else {
                "认证 Header 名称"
            },
            bearer_option = if fixed {
                ""
            } else {
                r#"<option value="bearer">Bearer Token</option>"#
            },
        ),
    )
}
async fn submit(
    State(state): State<FormState>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    if !current(&state, &headers)
        || headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(state.origin.as_str())
        || form.get("nonce") != Some(&state.nonce)
    {
        return reply(StatusCode::FORBIDDEN, "配置请求已失效。".into());
    }
    let credential = if state.service == crate::google_calendar::SERVICE {
        let (Some(client_id), Some(client_secret)) =
            (form.get("client_id"), form.get("client_secret"))
        else {
            return reply(StatusCode::BAD_REQUEST, "请填写完整应用配置。".into());
        };
        let config = crate::google_calendar::ClientConfig {
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            allow_event_updates: form.get("allow_event_updates").is_some_and(|v| v == "yes"),
        };
        if config.validate().is_err() {
            return reply(StatusCode::BAD_REQUEST, "应用配置格式不符合要求。".into());
        }
        CredentialInput::Google(config)
    } else {
        let (Some(name), Some(value), Some(scheme)) =
            (form.get("name"), form.get("value"), form.get("scheme"))
        else {
            return reply(StatusCode::BAD_REQUEST, "请填写完整配置。".into());
        };
        let value = match scheme.as_str() {
            "raw" => value.clone(),
            "bearer" => format!("Bearer {value}"),
            _ => return reply(StatusCode::BAD_REQUEST, "凭据格式不可用。".into()),
        };
        if validate(&state.service, name, &value).is_err() {
            return reply(
                StatusCode::BAD_REQUEST,
                "Header或凭据格式不符合要求，请返回填写。".into(),
            );
        }
        CredentialInput::Header(HeaderCredential {
            service_id: state.service.clone(),
            name: name.clone(),
            value,
        })
    };
    let result = state
        .sender
        .lock()
        .expect("configuration")
        .take()
        .is_some_and(|tx| tx.send(credential).is_ok());
    if result && state.service == crate::google_calendar::SERVICE {
        reply(
            StatusCode::OK,
            format!(
                r#"<h1>应用配置已接收</h1><p>请继续完成 Google 账号授权。</p><a href="{}/continue">继续 Google 授权</a>"#,
                state.path
            ),
        )
    } else if result {
        reply(
            StatusCode::OK,
            "已接收配置，请返回易界确认钥匙串保存和连接检查结果。此页面可关闭。".into(),
        )
    } else {
        reply(StatusCode::GONE, "配置已处理或取消，请返回易界。".into())
    }
}
async fn continue_google(State(state): State<FormState>, headers: HeaderMap) -> Response {
    if !current(&state, &headers) || state.service != crate::google_calendar::SERVICE {
        return reply(StatusCode::GONE, "授权已失效，请返回易界。".into());
    }
    let view = state.view.lock().expect("configuration");
    if let Some(value) = view
        .authorization_url
        .as_ref()
        .filter(|_| view.phase == Phase::AwaitingUser)
        && let Ok(url) = url::Url::parse(value)
        && url.origin().ascii_serialization() == crate::google_calendar::ISSUER
        && url.path() == "/o/oauth2/v2/auth"
    {
        let mut response = reply(StatusCode::SEE_OTHER, "正在前往 Google 授权。".into());
        if let Ok(location) = value.parse() {
            response.headers_mut().insert(header::LOCATION, location);
            return response;
        }
    }
    reply(
        StatusCode::OK,
        format!(
            r#"<p>正在准备授权；若易界已显示错误，请返回易界处理。</p><a href="{}/continue">继续 Google 授权</a>"#,
            state.path
        ),
    )
}
pub async fn configure(
    alias: &str,
    service: &str,
    deadline: Instant,
    view: &Arc<Mutex<Observation>>,
    cancel: &CancellationToken,
) -> Result<(), Failure> {
    if !supports(service) && service != crate::google_calendar::SERVICE {
        return Err(Failure::InvalidMetadata);
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|_| Failure::Unavailable)?;
    let host = listener
        .local_addr()
        .map_err(|_| Failure::Unavailable)?
        .to_string();
    let origin = format!("http://{host}");
    let path = format!("/credential/{}", Uuid::new_v4());
    let (tx, rx) = oneshot::channel();
    let state = FormState {
        service: service.into(),
        host,
        origin: origin.clone(),
        path: path.clone(),
        nonce: Uuid::new_v4().to_string(),
        deadline,
        cancellation: cancel.clone(),
        sender: Arc::new(Mutex::new(Some(tx))),
        view: view.clone(),
    };
    let app = Router::new()
        .route(&path, get(page).post(submit))
        .route(&format!("{path}/continue"), get(continue_google))
        .layer(DefaultBodyLimit::max(32768))
        .with_state(state);
    let stop = CancellationToken::new();
    let stopping = stop.clone();
    let mut server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(stopping.cancelled_owned())
            .await
    });
    let result = async {
        {
            let mut state=view.lock().expect("configuration");
            if cancel.is_cancelled(){return Err(Failure::Cancelled);}
            state.phase=Phase::AwaitingUser;
            state.authorization_url=Some(format!("{origin}{path}"));
        }
        let credential=tokio::select!{biased;_=cancel.cancelled()=>return Err(Failure::Cancelled),_=tokio::time::sleep_until(deadline.into())=>return Err(Failure::Expired),value=rx=>value.map_err(|_|Failure::Cancelled)?};
        {
            let mut state=view.lock().expect("configuration");
            if cancel.is_cancelled()||state.phase!=Phase::AwaitingUser{return Err(Failure::Cancelled);}
            if Instant::now()>=deadline{return Err(Failure::Expired);}
            credential.save(alias)?;
            if Instant::now()>=deadline{credential.forget(alias).map_err(|_|Failure::CleanupPending)?;return Err(Failure::Expired);}
            state.phase=if service==crate::google_calendar::SERVICE { Phase::Starting } else { Phase::Succeeded };
            state.authorization_url=None;state.failure=None;
        }
        if service==crate::google_calendar::SERVICE {
            crate::tushare_oauth::authorize_google_configured(alias,deadline,view,cancel).await?;
        }
        Ok(())
    }.await;
    stop.cancel();
    loop {
        match tokio::time::timeout(Duration::from_secs(5), &mut server).await {
            Ok(Ok(Ok(()))) => break,
            Ok(_) => return Err(Failure::CleanupPending),
            Err(_) => {
                let mut state = view.lock().expect("configuration");
                state.failure = Some(Failure::CleanupPending);
                state.authorization_url = None;
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn documented_ftshare_header_and_explicit_provider_header_are_distinct() {
        assert!(validate("FTShare", "FTSHARE_API_KEY", "ordinary-local-value").is_ok());
        assert!(validate("FTShare", "Authorization", "ordinary-local-value").is_err());
        assert!(validate("thinkingdata", "X-Api-Key", "ordinary-local-value").is_ok());
        assert!(validate("zerone", "api_key", "ordinary-local-value").is_ok());
    }
    fn form_state(ttl: Duration) -> (FormState, oneshot::Receiver<CredentialInput>, HeaderMap) {
        let (tx, rx) = oneshot::channel();
        let state = FormState {
            service: "FTShare".into(),
            origin: "http://127.0.0.1:12345".into(),
            host: "127.0.0.1:12345".into(),
            path: format!("/credential/{}", Uuid::new_v4()),
            nonce: Uuid::new_v4().to_string(),
            deadline: Instant::now() + ttl,
            cancellation: CancellationToken::new(),
            sender: Arc::new(Mutex::new(Some(tx))),
            view: Arc::new(Mutex::new(Observation {
                phase: Phase::Starting,
                authorization_url: None,
                failure: None,
            })),
        };
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, state.host.parse().unwrap());
        headers.insert(header::ORIGIN, state.origin.parse().unwrap());
        (state, rx, headers)
    }
    fn ordinary_form(state: &FormState) -> HashMap<String, String> {
        [
            ("nonce", state.nonce.as_str()),
            ("name", "FTSHARE_API_KEY"),
            ("scheme", "raw"),
            ("value", "ordinary-local-value"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect()
    }
    #[tokio::test]
    async fn normal_configuration_page_submits_once_without_echoing_credentials() {
        let (state, rx, headers) = form_state(Duration::from_secs(60));
        let response = page(State(state.clone()), headers.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let response = submit(
            State(state.clone()),
            headers.clone(),
            Form(ordinary_form(&state)),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        assert!(!String::from_utf8_lossy(&body).contains("ordinary-local-value"));
        let CredentialInput::Header(received) = rx.await.unwrap() else {
            panic!("expected header")
        };
        assert_eq!(received.service_id, "FTShare");
        assert_eq!(received.name, "FTSHARE_API_KEY");
        assert_eq!(received.value, "ordinary-local-value");
        assert_eq!(
            submit(
                State(state.clone()),
                headers.clone(),
                Form(ordinary_form(&state))
            )
            .await
            .status(),
            StatusCode::GONE
        );
        assert_eq!(page(State(state), headers).await.status(), StatusCode::GONE);
    }
    #[tokio::test]
    async fn agentic_configuration_uses_explicit_exported_header_without_inferred_scheme() {
        let (mut state, rx, headers) = form_state(Duration::from_secs(60));
        state.service = "thinkingdata".into();
        let response = page(State(state.clone()), headers.clone()).await;
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        let page_text = String::from_utf8(body.to_vec()).unwrap();
        assert!(page_text.contains("https://docs.thinkingai.cn/zh/manual/mcp"));
        assert!(page_text.contains("https://ta-sdk-service.thinkingdata.cn/mcp"));
        assert!(page_text.contains("name=\"name\" value=\"\""));
        let mut form = ordinary_form(&state);
        // An ordinary example supplied through the form, not a claimed vendor header.
        form.insert("name".into(), "X-Example-Token".into());
        let response = submit(State(state), headers, Form(form)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let CredentialInput::Header(received) = rx.await.unwrap() else {
            panic!("expected header")
        };
        assert_eq!(received.name, "X-Example-Token");
        assert_eq!(received.value, "ordinary-local-value");
        assert_eq!(
            received
                .resource_url("https://ta-sdk-service.thinkingdata.cn/mcp")
                .unwrap(),
            "https://ta-sdk-service.thinkingdata.cn/mcp"
        );
        assert!(received.query_name().is_none());
    }
    #[tokio::test]
    async fn normal_expiration_and_user_cancel_close_configuration_admission() {
        for ttl in [Duration::ZERO, Duration::from_secs(60)] {
            let (state, mut rx, headers) = form_state(ttl);
            if !ttl.is_zero() {
                state.cancellation.cancel();
            }
            assert_eq!(
                page(State(state.clone()), headers.clone()).await.status(),
                StatusCode::GONE
            );
            assert_eq!(
                submit(State(state.clone()), headers, Form(ordinary_form(&state)))
                    .await
                    .status(),
                StatusCode::FORBIDDEN
            );
            assert!(matches!(
                rx.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
        }
    }
    #[tokio::test]
    async fn normal_google_configuration_has_explicit_scope_choice_and_owned_continuation() {
        let (mut state, rx, headers) = form_state(Duration::from_secs(60));
        state.service = crate::google_calendar::SERVICE.into();
        let response = page(State(state.clone()), headers.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        let html = String::from_utf8(
            axum::body::to_bytes(response.into_body(), 65536)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains(crate::google_calendar::CALLBACK));
        let form = [
            ("nonce", state.nonce.as_str()),
            ("client_id", "ordinary.apps.googleusercontent.com"),
            ("client_secret", "ordinary-local-value"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        let response = submit(State(state.clone()), headers.clone(), Form(form)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let CredentialInput::Google(config) = rx.await.unwrap() else {
            panic!("expected OAuth config")
        };
        assert!(!config.allow_event_updates);
        // Saving a client setting alone has not authorized any Google account.
        assert_ne!(state.view.lock().unwrap().phase, Phase::Succeeded);
        let authorize = format!(
            "{}?state=ordinary-local-state",
            crate::google_calendar::AUTHORIZE
        );
        {
            let mut view = state.view.lock().unwrap();
            view.phase = Phase::AwaitingUser;
            view.authorization_url = Some(authorize.clone());
        }
        let response = continue_google(State(state), headers).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], authorize);
    }
    #[test]
    fn documented_zerone_query_is_encoded_only_for_its_fixed_resource() {
        let config = HeaderCredential {
            service_id: "zerone".into(),
            name: "api_key".into(),
            value: "ordinary-local-value".into(),
        };
        let source = provider_registry::get("zerone")
            .unwrap()
            .resource
            .as_deref()
            .unwrap();
        assert_eq!(source, "https://ai.zerone.com.cn/mcp/pe");
        let url = url::Url::parse(&config.resource_url(source).unwrap()).unwrap();
        assert_eq!(
            url.query_pairs().collect::<Vec<_>>(),
            vec![("api_key".into(), "ordinary-local-value".into())]
        );
        assert!(!source.contains("ordinary-local-value"));
        assert_eq!(config.query_name(), Some("api_key"));
    }
}
