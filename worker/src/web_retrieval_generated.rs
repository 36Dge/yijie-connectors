// Generated from web-retrieval canonical source; DO NOT EDIT.
use serde::{Deserialize, Serialize};

fn optional_non_null<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
pub type PublicUrl = String;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SearchArgumentsWire")]
pub struct SearchArguments {
    pub query: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub limit: Option<i64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchArgumentsWire {
    pub query: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub limit: Option<i64>,
}
impl TryFrom<SearchArgumentsWire> for SearchArguments {
    type Error = &'static str;
    fn try_from(w: SearchArgumentsWire) -> Result<Self, Self::Error> {
        let v = Self {
            query: w.query,
            limit: w.limit,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SearchArguments {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_query = &self.query;
            if value_query.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_query.chars().count() > 2048 {
                return Err("invalid broker text");
            }
            if value_query.trim().is_empty() {
                return Err("web retrieval text is blank");
            }
        }
        self.limit
            .as_ref()
            .map(|value_limit| -> Result<(), &'static str> {
                if *value_limit < 1 {
                    return Err("invalid broker number");
                }
                if *value_limit > 20 {
                    return Err("invalid broker number");
                }
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for SearchArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchArguments([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "FetchArgumentsWire")]
pub struct FetchArguments {
    pub url: PublicUrl,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FetchArgumentsWire {
    pub url: PublicUrl,
}
impl TryFrom<FetchArgumentsWire> for FetchArguments {
    type Error = &'static str;
    fn try_from(w: FetchArgumentsWire) -> Result<Self, Self::Error> {
        let v = Self { url: w.url };
        v.validate()?;
        Ok(v)
    }
}
impl FetchArguments {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_url = &self.url;
            if value_url.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_url.chars().count() > 4096 {
                return Err("invalid broker text");
            }
            if !(value_url.starts_with("http://") || value_url.starts_with("https://")) {
                return Err("web retrieval URL requires HTTP(S)");
            }
            let parsed = url::Url::parse(value_url).map_err(|_| "invalid web retrieval URL")?;
            if parsed.host_str().is_none() {
                return Err("web retrieval URL requires host");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for FetchArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FetchArguments([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SearchResultWire")]
pub struct SearchResult {
    pub title: String,
    pub url: PublicUrl,
    pub snippet: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchResultWire {
    pub title: String,
    pub url: PublicUrl,
    pub snippet: String,
}
impl TryFrom<SearchResultWire> for SearchResult {
    type Error = &'static str;
    fn try_from(w: SearchResultWire) -> Result<Self, Self::Error> {
        let v = Self {
            title: w.title,
            url: w.url,
            snippet: w.snippet,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SearchResult {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_title = &self.title;
            if value_title.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        {
            let value_url = &self.url;
            if value_url.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_url.chars().count() > 4096 {
                return Err("invalid broker text");
            }
            if !(value_url.starts_with("http://") || value_url.starts_with("https://")) {
                return Err("web retrieval URL requires HTTP(S)");
            }
            let parsed = url::Url::parse(value_url).map_err(|_| "invalid web retrieval URL")?;
            if parsed.host_str().is_none() {
                return Err("web retrieval URL requires host");
            }
        }
        {
            let value_snippet = &self.snippet;
            if value_snippet.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchResult([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SearchSuccessWire")]
pub struct SearchSuccess {
    pub results: Vec<SearchResult>,
    pub truncated: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchSuccessWire {
    pub results: Vec<SearchResult>,
    pub truncated: bool,
}
impl TryFrom<SearchSuccessWire> for SearchSuccess {
    type Error = &'static str;
    fn try_from(w: SearchSuccessWire) -> Result<Self, Self::Error> {
        let v = Self {
            results: w.results,
            truncated: w.truncated,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SearchSuccess {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_results = &self.results;
            if value_results.len() > 20 {
                return Err("invalid broker collection");
            }
            for item in value_results.iter() {
                item.validate()?;
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SearchSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchSuccess([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "FetchSuccessWire")]
pub struct FetchSuccess {
    pub url: PublicUrl,
    pub title: String,
    pub markdown: String,
    pub truncated: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FetchSuccessWire {
    pub url: PublicUrl,
    pub title: String,
    pub markdown: String,
    pub truncated: bool,
}
impl TryFrom<FetchSuccessWire> for FetchSuccess {
    type Error = &'static str;
    fn try_from(w: FetchSuccessWire) -> Result<Self, Self::Error> {
        let v = Self {
            url: w.url,
            title: w.title,
            markdown: w.markdown,
            truncated: w.truncated,
        };
        v.validate()?;
        Ok(v)
    }
}
impl FetchSuccess {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_url = &self.url;
            if value_url.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_url.chars().count() > 4096 {
                return Err("invalid broker text");
            }
            if !(value_url.starts_with("http://") || value_url.starts_with("https://")) {
                return Err("web retrieval URL requires HTTP(S)");
            }
            let parsed = url::Url::parse(value_url).map_err(|_| "invalid web retrieval URL")?;
            if parsed.host_str().is_none() {
                return Err("web retrieval URL requires host");
            }
        }
        {
            let value_title = &self.title;
            if value_title.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        {
            let value_markdown = &self.markdown;
            if value_markdown.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_markdown.chars().count() > 65536 {
                return Err("invalid broker text");
            }
            if value_markdown.trim().is_empty() {
                return Err("web retrieval text is blank");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for FetchSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FetchSuccess([redacted])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "invalid_arguments")]
    InvalidArguments,
    #[serde(rename = "authentication_failed")]
    AuthenticationFailed,
    #[serde(rename = "permission_denied")]
    PermissionDenied,
    #[serde(rename = "rate_limited")]
    RateLimited,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "no_content")]
    NoContent,
    #[serde(rename = "response_too_large")]
    ResponseTooLarge,
    #[serde(rename = "invalid_response")]
    InvalidResponse,
    #[serde(rename = "cancelled")]
    Cancelled,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ToolErrorWire")]
pub struct ToolError {
    pub code: ErrorCode,
    pub message: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ToolErrorWire {
    pub code: ErrorCode,
    pub message: String,
}
impl TryFrom<ToolErrorWire> for ToolError {
    type Error = &'static str;
    fn try_from(w: ToolErrorWire) -> Result<Self, Self::Error> {
        let v = Self {
            code: w.code,
            message: w.message,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ToolError {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_message = &self.message;
            if value_message.chars().count() < 1 {
                return Err("invalid broker text");
            }
            if value_message.chars().count() > 512 {
                return Err("invalid broker text");
            }
        }
        if self.message != self.code.safe_message() {
            return Err("web retrieval error message mismatch");
        }
        Ok(())
    }
}
impl std::fmt::Debug for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ToolError([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ErrorOutputWire")]
pub struct ErrorOutput {
    pub error: ToolError,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ErrorOutputWire {
    pub error: ToolError,
}
impl TryFrom<ErrorOutputWire> for ErrorOutput {
    type Error = &'static str;
    fn try_from(w: ErrorOutputWire) -> Result<Self, Self::Error> {
        let v = Self { error: w.error };
        v.validate()?;
        Ok(v)
    }
}
impl ErrorOutput {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_error = &self.error;
            value_error.validate()?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ErrorOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ErrorOutput([redacted])")
    }
}
impl ErrorCode {
    pub fn safe_message(self) -> &'static str {
        match self {
            Self::InvalidArguments => "检索参数无效，请检查查询词、数量或网页地址。",
            Self::AuthenticationFailed => "Kimi API Key 无效或已失效，请重新配置。",
            Self::PermissionDenied => "服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。",
            Self::RateLimited => "Kimi 请求频率或并发受限，请确认后重新发起调用。",
            Self::Timeout => "本次网页检索超时，未自动重试；已发送的请求可能计费。",
            Self::Unavailable => "网页检索服务暂不可用，未自动重试。",
            Self::NoContent => "目标网页未返回可读取的正文。",
            Self::ResponseTooLarge => "网页检索结果超过可处理的大小限制。",
            Self::InvalidResponse => "网页检索服务返回了无法验证的结果。",
            Self::Cancelled => "本次网页检索已取消；已发送的请求可能计费。",
        }
    }
}
impl ErrorOutput {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            error: ToolError {
                code,
                message: code.safe_message().to_owned(),
            },
        }
    }
}
impl SearchArguments {
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_SEARCH_LIMIT as i64)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchOutput {
    SearchSuccess(SearchSuccess),
    ErrorOutput(ErrorOutput),
}
impl SearchOutput {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::SearchSuccess(value) => value.validate(),
            Self::ErrorOutput(value) => value.validate(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FetchOutput {
    FetchSuccess(FetchSuccess),
    ErrorOutput(ErrorOutput),
}
impl FetchOutput {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::FetchSuccess(value) => value.validate(),
            Self::ErrorOutput(value) => value.validate(),
        }
    }
}
pub const SERVICE_ID: &str = "kimi-web";
pub const SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"$id\":\"https://schemas.yijie.ai/web-retrieval/v1\",\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"SearchArguments\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5}}},\"FetchArguments\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"SearchOutput\":{\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}]},\"FetchOutput\":{\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}]}}}";
pub const TOOLS_JSON:&str="[{\"name\":\"kimi_web_search\",\"description\":\"按查询词获取网页标题、URL和摘要的结构化列表。需要结构化搜索结果时使用；一般联网回答可使用原生搜索。只读但按次计费，每次需要批准，不自动重试。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}},{\"name\":\"kimi_web_fetch\",\"description\":\"读取指定公开HTTP(S)网页的标题、URL和Markdown正文。用于需要指定URL具体正文的任务；不提供原始HTML、登录态访问或JavaScript渲染控制。只读但按次计费，每次需要批准，不自动重试。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}},\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}}]";
pub const SEARCH_TOOL_NAME: &str = "kimi_web_search";
pub const SEARCH_DESCRIPTION:&str="按查询词获取网页标题、URL和摘要的结构化列表。需要结构化搜索结果时使用；一般联网回答可使用原生搜索。只读但按次计费，每次需要批准，不自动重试。";
pub const SEARCH_INPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5}}}";
pub const SEARCH_OUTPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}}";
pub const SEARCH_PATH: &str = "/v1/tools/search";
pub const SEARCH_ANNOTATIONS_JSON:&str="{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}";
pub const FETCH_TOOL_NAME: &str = "kimi_web_fetch";
pub const FETCH_DESCRIPTION:&str="读取指定公开HTTP(S)网页的标题、URL和Markdown正文。用于需要指定URL具体正文的任务；不提供原始HTML、登录态访问或JavaScript渲染控制。只读但按次计费，每次需要批准，不自动重试。";
pub const FETCH_INPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}},\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"}}}";
pub const FETCH_OUTPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}}";
pub const FETCH_PATH: &str = "/v1/tools/fetch";
pub const FETCH_ANNOTATIONS_JSON:&str="{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}";
pub const DEFAULT_SEARCH_LIMIT: usize = 5;
pub const MAX_RESULT_BYTES: usize = 65536;
pub const MAX_RESPONSE_BYTES: usize = 1048576;
pub const HTTP_TIMEOUT_MS: usize = 15000;
pub const SEARCH_TIMEOUT_SECONDS: usize = 10;
