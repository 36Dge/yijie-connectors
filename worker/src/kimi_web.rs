//! Owned Kimi REST adapter. The MCP Gateway still owns selection and approval.
//! No target-page request, model credential lookup, redirect, or retry occurs here.
use crate::{
    broker_generated as broker_wire,
    credentials::HeaderCredential,
    daily::{HttpPermit, Permit},
    tushare_oauth::{Diagnostic, Event, Failure, Stage},
    web_retrieval_generated as wire,
};
use codex_exec_server::{HttpHeader, HttpRedirectPolicy, HttpRequestParams};
use rmcp::model::{CallToolResult, Tool};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    future::Future,
    net::Ipv4Addr,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
use wire::ErrorCode;

const API_ORIGIN: &str = "https://api.moonshot.cn";

pub fn descriptors() -> Vec<Tool> {
    serde_json::from_str(wire::TOOLS_JSON).expect("canonical web retrieval descriptors")
}

fn public_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    !ip.is_private()
        && !ip.is_loopback()
        && !ip.is_link_local()
        && !ip.is_unspecified()
        && !ip.is_broadcast()
        && !ip.is_multicast()
        && octets[0] != 0
        && octets[0] < 240
        && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
}

// This is a parameter constraint, not a local crawler or DNS lookup.
fn public_url(value: &str) -> bool {
    if value.chars().any(char::is_control) || value.chars().count() > 4096 {
        return false;
    }
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return false;
    }
    match url.host() {
        Some(url::Host::Ipv4(ip)) => public_ipv4(ip),
        Some(url::Host::Ipv6(ip)) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_unique_local()
                && !ip.is_unicast_link_local()
                && ip.to_ipv4_mapped().is_none_or(public_ipv4)
        }
        Some(url::Host::Domain(host)) => {
            let host = host.trim_end_matches('.');
            !host.is_empty()
                && host != "localhost"
                && !host.ends_with(".localhost")
                && !host.ends_with(".local")
                && !host.ends_with(".internal")
        }
        None => false,
    }
}

enum Arguments {
    Search(wire::SearchArguments),
    Fetch(wire::FetchArguments),
}
impl Arguments {
    fn parse(name: &str, args: &Map<String, Value>) -> Result<Self, ErrorCode> {
        let value = Value::Object(args.clone());
        match name {
            wire::SEARCH_TOOL_NAME => serde_json::from_value(value)
                .map(Self::Search)
                .map_err(|_| ErrorCode::InvalidArguments),
            wire::FETCH_TOOL_NAME => {
                let args: wire::FetchArguments =
                    serde_json::from_value(value).map_err(|_| ErrorCode::InvalidArguments)?;
                if !public_url(&args.url) {
                    return Err(ErrorCode::InvalidArguments);
                }
                Ok(Self::Fetch(args))
            }
            _ => Err(ErrorCode::InvalidArguments),
        }
    }
    fn path(&self) -> &'static str {
        match self {
            Self::Search(_) => wire::SEARCH_PATH,
            Self::Fetch(_) => wire::FETCH_PATH,
        }
    }
    fn body(&self) -> Value {
        match self {
            Self::Search(args) => json!({"text_query": args.query, "limit": args.effective_limit(),
                "timeout_seconds": wire::SEARCH_TIMEOUT_SECONDS, "include_content": false}),
            Self::Fetch(args) => json!({"url": args.url}),
        }
    }
}

struct HttpReply {
    status: u16,
    body: Vec<u8>,
}
type HttpFuture<'a> = Pin<Box<dyn Future<Output = Result<HttpReply, ErrorCode>> + Send + 'a>>;
trait Transport: Send + Sync {
    fn request(&self, request: HttpRequestParams) -> HttpFuture<'_>;
}
struct StreamingHttp;
fn append_chunk(bytes: &mut Vec<u8>, chunk: &[u8]) -> Result<(), ErrorCode> {
    if bytes.len().saturating_add(chunk.len()) > wire::MAX_RESPONSE_BYTES {
        return Err(ErrorCode::ResponseTooLarge);
    }
    bytes.extend_from_slice(chunk);
    Ok(())
}
impl Transport for StreamingHttp {
    fn request(&self, request: HttpRequestParams) -> HttpFuture<'_> {
        Box::pin(async move {
            let started = Instant::now();
            let budget =
                Duration::from_millis(request.timeout_ms.unwrap_or(wire::HTTP_TIMEOUT_MS as u64));
            let failed = || {
                if started.elapsed() >= budget {
                    ErrorCode::Timeout
                } else {
                    ErrorCode::Unavailable
                }
            };
            // The general fixed HttpClient retains reqwest's protocol retry
            // default. This adapter explicitly disables it for paid requests.
            let client = reqwest::Client::builder()
                .retry(reqwest::retry::never())
                .redirect(reqwest::redirect::Policy::none())
                .timeout(budget)
                .connect_timeout(budget)
                .build()
                .map_err(|_| ErrorCode::Unavailable)?;
            let method = reqwest::Method::from_bytes(request.method.as_bytes())
                .map_err(|_| ErrorCode::InvalidArguments)?;
            let mut outgoing = client.request(method, &request.url);
            for header in request.headers {
                outgoing = outgoing.header(header.name.as_str(), header.value.as_str());
            }
            if let Some(body) = request.body {
                outgoing = outgoing.body(body.0);
            }
            let mut response = outgoing.send().await.map_err(|_| failed())?;
            let status = response.status().as_u16();
            let mut body = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|_| failed())? {
                append_chunk(&mut body, &chunk)?;
            }
            Ok(HttpReply { status, body })
        })
    }
}

pub struct Backend {
    credential: HeaderCredential,
    http: Arc<dyn Transport>,
    permit: Arc<HttpPermit>,
    serial: tokio::sync::Mutex<()>,
    cancelled: CancellationToken,
}
impl Backend {
    pub fn load() -> Result<Arc<Self>, Failure> {
        let credential = crate::model_credentials::kimi().ok_or(Failure::NotAuthorized)?;
        Ok(Self::new(credential, Arc::new(StreamingHttp)))
    }
    fn new(credential: HeaderCredential, http: Arc<dyn Transport>) -> Arc<Self> {
        let permit = Arc::new(HttpPermit::default());
        permit.1.remember(&credential.value);
        Arc::new(Self {
            credential,
            http,
            permit,
            serial: tokio::sync::Mutex::new(()),
            cancelled: CancellationToken::new(),
        })
    }
    pub fn available(&self) -> bool {
        !self.cancelled.is_cancelled() && self.permit.1.available()
    }
    pub fn revoke(&self) {
        self.cancelled.cancel();
    }
    pub async fn close(&self) {
        self.revoke();
        // The bounded original HTTP operation retains its owner until completion.
        let _guard = self.serial.lock().await;
    }
    pub fn contains(&self, tool: &str) -> bool {
        matches!(tool, wire::SEARCH_TOOL_NAME | wire::FETCH_TOOL_NAME)
    }
    pub fn review(
        &self,
        tool: &str,
        args: &Map<String, Value>,
    ) -> Result<broker_wire::ReviewProjection, ErrorCode> {
        let arguments = Value::Object(args.clone());
        if !self.permit.1.safe(&arguments) {
            return Err(ErrorCode::InvalidArguments);
        }
        let parsed = Arguments::parse(tool, args)?;
        let (title, summary) = match parsed {
            Arguments::Search(args) => ("Kimi 网页搜索", format!("只读搜索，最多返回 {} 条标题、链接和摘要。查询词将发给 Kimi，服务按次计费；本次批准仅允许一次请求，不自动重试。", args.effective_limit())),
            Arguments::Fetch(_) => ("Kimi 网页读取", "只读获取指定公开网页的 Markdown 正文。网页地址将发给 Kimi，服务按次计费；本次批准仅允许一次请求，不自动重试，不承诺 JavaScript 渲染。".into()),
        };
        let definition = descriptors()
            .into_iter()
            .find(|d| d.name == tool)
            .ok_or(ErrorCode::InvalidArguments)?;
        let bytes = serde_json::to_vec(&definition).map_err(|_| ErrorCode::InvalidArguments)?;
        let encoded = serde_json::to_string(&arguments).map_err(|_| ErrorCode::InvalidArguments)?;
        if encoded.len() > broker_wire::GENERIC_MAX_REVIEW_ARGUMENTS_BYTES {
            return Err(ErrorCode::InvalidArguments);
        }
        Ok(broker_wire::ReviewProjection {
            title: title.into(),
            summary,
            risk: "read".into(),
            arguments_json: Some(encoded),
            schema_digest: Some(format!("{:x}", Sha256::digest(bytes))),
        })
    }
    fn request(
        &self,
        url: String,
        body: Option<Value>,
        deadline: Instant,
    ) -> Result<HttpRequestParams, ErrorCode> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let timeout = remaining
            .min(Duration::from_millis(wire::HTTP_TIMEOUT_MS as u64))
            .as_millis() as u64;
        if timeout == 0 {
            return Err(ErrorCode::Timeout);
        }
        let method = if body.is_some() { "POST" } else { "GET" };
        if !(method == "POST"
            && [
                format!("{API_ORIGIN}{}", wire::SEARCH_PATH),
                format!("{API_ORIGIN}{}", wire::FETCH_PATH),
            ]
            .contains(&url))
        {
            return Err(ErrorCode::InvalidArguments);
        }
        Ok(HttpRequestParams {
            method: method.into(),
            url,
            headers: vec![
                HttpHeader {
                    name: "Authorization".into(),
                    value: self.credential.value.clone(),
                },
                HttpHeader {
                    name: "Content-Type".into(),
                    value: "application/json".into(),
                },
                HttpHeader {
                    name: "Accept".into(),
                    value: "application/json".into(),
                },
            ],
            body: body.map(|v| serde_json::to_vec(&v).expect("JSON request").into()),
            timeout_ms: Some(timeout),
            redirect_policy: HttpRedirectPolicy::Stop,
            request_id: uuid::Uuid::new_v4().to_string(),
            stream_response: true,
        })
    }
    async fn send(
        &self,
        mut request: HttpRequestParams,
        deadline: Instant,
        diagnostic: &Diagnostic,
    ) -> Result<HttpReply, ErrorCode> {
        let budget = Duration::from_millis(request.timeout_ms.expect("bounded HTTP request"));
        let deadline = deadline.min(Instant::now() + budget);
        let timeout = deadline
            .saturating_duration_since(Instant::now())
            .as_millis() as u64;
        if timeout == 0 {
            return Err(ErrorCode::Timeout);
        }
        request.timeout_ms = Some(timeout);
        diagnostic.emit(Stage::WebHttpSend, Event::Request, None, None);
        let result = tokio::time::timeout_at(deadline.into(), self.http.request(request))
            .await
            .map_err(|_| ErrorCode::Timeout)?;
        match &result {
            Ok(reply) => diagnostic.emit(
                Stage::WebHttpResponse,
                Event::Response,
                Some(reply.status),
                None,
            ),
            Err(_) => diagnostic.emit(
                Stage::WebHttpResponse,
                Event::TransportUnavailable,
                None,
                None,
            ),
        }
        result
    }
    pub fn prepare(&self) -> Result<(), Failure> {
        // REST invocation support is local; credentials/entitlement are checked
        // only by the user's actual approved Search/Fetch request.
        if !self.available() {
            return Err(Failure::Cancelled);
        }
        if descriptors().len() != 2 {
            return Err(Failure::InvalidMetadata);
        }
        Ok(())
    }
    pub async fn execute(
        &self,
        tool: &str,
        args: Map<String, Value>,
        mut permit: Permit,
        diagnostic: Diagnostic,
    ) -> CallToolResult {
        let result = self
            .execute_inner(tool, args, &mut permit, &diagnostic)
            .await;
        result.unwrap_or_else(error_result)
    }
    async fn execute_inner(
        &self,
        tool: &str,
        args: Map<String, Value>,
        permit: &mut Permit,
        diagnostic: &Diagnostic,
    ) -> Result<CallToolResult, ErrorCode> {
        let _guard = self.serial.lock().await;
        self.review(tool, &args)?;
        if !self.available() || permit.request.is_cancelled() || permit.lease.is_cancelled() {
            return Err(ErrorCode::Cancelled);
        }
        if Instant::now() >= permit.deadline {
            return Err(ErrorCode::Timeout);
        }
        let arguments = Arguments::parse(tool, &args)?;
        let request = self.request(
            format!("{API_ORIGIN}{}", arguments.path()),
            Some(arguments.body()),
            permit.deadline,
        )?;
        let reviewed = Value::Object(args);
        self.permit.arm(
            Permit {
                tool: tool.into(),
                arguments: reviewed.clone(),
                request: permit.request.clone(),
                lease: permit.lease.clone(),
                backend: self.cancelled.clone(),
                deadline: permit.deadline,
            },
            diagnostic.clone(),
        );
        if !self.permit.take_call(tool, &reviewed) {
            self.permit.clear();
            return Err(ErrorCode::Cancelled);
        }
        // Consume before the sole send. No retries or alternate endpoint follows.
        let response = self.send(request, permit.deadline, diagnostic).await;
        if !self.available() || permit.request.is_cancelled() || permit.lease.is_cancelled() {
            return Err(ErrorCode::Cancelled);
        }
        if Instant::now() >= permit.deadline {
            return Err(ErrorCode::Timeout);
        }
        let reply = response?;
        status_error(reply.status, matches!(arguments, Arguments::Fetch(_)))?;
        let value: Value =
            serde_json::from_slice(&reply.body).map_err(|_| ErrorCode::InvalidResponse)?;
        if !self.permit.1.safe(&value) {
            return Err(ErrorCode::InvalidResponse);
        }
        let result = match arguments {
            Arguments::Search(args) => normalize_search(&value, args.effective_limit() as usize)?,
            Arguments::Fetch(_) => normalize_fetch(&value)?,
        };
        if !self
            .permit
            .1
            .safe(&serde_json::to_value(&result).map_err(|_| ErrorCode::InvalidResponse)?)
        {
            return Err(ErrorCode::InvalidResponse);
        }
        diagnostic.emit(Stage::WebNormalize, Event::Succeeded, None, None);
        Ok(result)
    }
}

fn status_error(status: u16, fetch: bool) -> Result<(), ErrorCode> {
    match status {
        200..=299 => Ok(()),
        401 => Err(ErrorCode::AuthenticationFailed),
        403 => Err(ErrorCode::PermissionDenied),
        429 => Err(ErrorCode::RateLimited),
        408 | 504 => Err(ErrorCode::Timeout),
        404 if fetch => Err(ErrorCode::NoContent),
        400 | 422 => Err(ErrorCode::InvalidArguments),
        _ => Err(ErrorCode::Unavailable),
    }
}
fn error_result(code: ErrorCode) -> CallToolResult {
    let value = wire::ErrorOutput::new(code);
    value.validate().expect("canonical error");
    CallToolResult::structured_error(serde_json::to_value(value).expect("canonical error"))
}
fn fits(result: &CallToolResult) -> bool {
    serde_json::to_vec(result).is_ok_and(|bytes| bytes.len() <= wire::MAX_RESULT_BYTES)
}
fn text(value: &Value, key: &str, optional: bool) -> Result<String, ErrorCode> {
    match value.get(key) {
        Some(Value::String(text)) => Ok(text.clone()),
        None | Some(Value::Null) if optional => Ok(String::new()),
        _ => Err(ErrorCode::InvalidResponse),
    }
}
fn shrink(value: &mut String) {
    let mut boundary = value.len() / 2;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
}
fn normalize_search(value: &Value, limit: usize) -> Result<CallToolResult, ErrorCode> {
    let source = value
        .get("search_results")
        .and_then(Value::as_array)
        .ok_or(ErrorCode::InvalidResponse)?;
    let mut output = wire::SearchSuccess {
        results: Vec::new(),
        truncated: source.len() > limit,
    };
    for item in source.iter().take(limit) {
        let url = text(item, "url", false)?;
        if !public_url(&url) {
            return Err(ErrorCode::InvalidResponse);
        }
        let title = text(item, "title", true)?;
        let mut snippet = text(item, "snippet", true)?;
        while snippet.chars().count() > 65536 {
            shrink(&mut snippet);
            output.truncated = true;
        }
        output.results.push(wire::SearchResult {
            title,
            url,
            snippet,
        });
    }
    loop {
        let result = CallToolResult::structured(
            serde_json::to_value(&output).map_err(|_| ErrorCode::InvalidResponse)?,
        );
        if fits(&result) {
            output.validate().map_err(|_| ErrorCode::InvalidResponse)?;
            return Ok(result);
        }
        output.truncated = true;
        if let Some(longest) = output
            .results
            .iter_mut()
            .filter(|r| !r.snippet.is_empty())
            .max_by_key(|r| r.snippet.len())
        {
            shrink(&mut longest.snippet);
        } else if output.results.len() > 1 {
            output.results.pop();
        } else {
            return Err(ErrorCode::ResponseTooLarge);
        }
    }
}
fn normalize_fetch(value: &Value) -> Result<CallToolResult, ErrorCode> {
    let mut output = wire::FetchSuccess {
        url: text(value, "url", false)?,
        title: text(value, "title", true)?,
        markdown: text(value, "markdown", false)?,
        truncated: false,
    };
    if !public_url(&output.url) {
        return Err(ErrorCode::InvalidResponse);
    }
    if output.markdown.trim().is_empty() {
        return Err(ErrorCode::NoContent);
    }
    loop {
        let result = CallToolResult::structured(
            serde_json::to_value(&output).map_err(|_| ErrorCode::InvalidResponse)?,
        );
        if fits(&result) {
            output.validate().map_err(|_| ErrorCode::InvalidResponse)?;
            return Ok(result);
        }
        output.truncated = true;
        shrink(&mut output.markdown);
        if output.markdown.trim().is_empty() {
            return Err(ErrorCode::ResponseTooLarge);
        }
    }
}

#[cfg(test)]
#[path = "kimi_web_tests.rs"]
pub(crate) mod tests;
