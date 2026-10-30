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
    #[serde(rename = "include_content")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub include_content: Option<bool>,
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
    #[serde(rename = "include_content")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub include_content: Option<bool>,
}
impl TryFrom<SearchArgumentsWire> for SearchArguments {
    type Error = &'static str;
    fn try_from(w: SearchArgumentsWire) -> Result<Self, Self::Error> {
        let v = Self {
            query: w.query,
            limit: w.limit,
            include_content: w.include_content,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "truncated")]
    Truncated,
}
pub type ChunkScore = serde_json::Number;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "SearchChunkWire")]
pub struct SearchChunk {
    pub text: String,
    pub score: ChunkScore,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchChunkWire {
    pub text: String,
    pub score: ChunkScore,
}
impl TryFrom<SearchChunkWire> for SearchChunk {
    type Error = &'static str;
    fn try_from(w: SearchChunkWire) -> Result<Self, Self::Error> {
        let v = Self {
            text: w.text,
            score: w.score,
        };
        v.validate()?;
        Ok(v)
    }
}
impl SearchChunk {
    pub fn validate(&self) -> Result<(), &'static str> {
        {
            let value_text = &self.text;
            if value_text.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        {
            let value_score = &self.score;
            if !value_score.as_f64().is_some_and(f64::is_finite) {
                return Err("nonfinite score");
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for SearchChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchChunk([redacted])")
    }
}
pub type DateBound = String;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "TimeWindowWire")]
pub struct TimeWindow {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub start: Option<DateBound>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub end: Option<DateBound>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TimeWindowWire {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub start: Option<DateBound>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub end: Option<DateBound>,
}
impl TryFrom<TimeWindowWire> for TimeWindow {
    type Error = &'static str;
    fn try_from(w: TimeWindowWire) -> Result<Self, Self::Error> {
        let v = Self {
            start: w.start,
            end: w.end,
        };
        v.validate()?;
        Ok(v)
    }
}
impl TimeWindow {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.start
            .as_ref()
            .map(|value_start| -> Result<(), &'static str> {
                date_value(value_start).ok_or("invalid calendar date")?;
                Ok(())
            })
            .transpose()?;
        self.end
            .as_ref()
            .map(|value_end| -> Result<(), &'static str> {
                date_value(value_end).ok_or("invalid calendar date")?;
                Ok(())
            })
            .transpose()?;
        let start = self.start.as_deref().and_then(date_value);
        let end = self.end.as_deref().and_then(date_value);
        if start.zip(end).is_some_and(|(s, e)| s > e) {
            return Err("date range reversed");
        }
        Ok(())
    }
}
impl std::fmt::Debug for TimeWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TimeWindow([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProArgumentsWire")]
pub struct ProArguments {
    pub query: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub limit: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub sites: Option<Vec<String>>,
    #[serde(rename = "time_window")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub time_window: Option<TimeWindow>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProArgumentsWire {
    pub query: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub limit: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub sites: Option<Vec<String>>,
    #[serde(rename = "time_window")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_non_null"
    )]
    pub time_window: Option<TimeWindow>,
}
impl TryFrom<ProArgumentsWire> for ProArguments {
    type Error = &'static str;
    fn try_from(w: ProArgumentsWire) -> Result<Self, Self::Error> {
        let v = Self {
            query: w.query,
            limit: w.limit,
            sites: w.sites,
            time_window: w.time_window,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProArguments {
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
        self.sites
            .as_ref()
            .map(|value_sites| -> Result<(), &'static str> {
                if value_sites.len() > 5 {
                    return Err("invalid broker collection");
                }
                for item in value_sites.iter() {
                    if item.chars().count() < 1 {
                        return Err("invalid broker text");
                    }
                    if item.chars().count() > 4096 {
                        return Err("invalid broker text");
                    }
                    if item
                        .chars()
                        .any(|c| c.is_whitespace() || c == '(' || c == ')')
                    {
                        return Err("invalid site");
                    }
                }
                Ok(())
            })
            .transpose()?;
        self.time_window
            .as_ref()
            .map(|value_time_window| -> Result<(), &'static str> {
                value_time_window.validate()?;
                Ok(())
            })
            .transpose()?;
        Ok(())
    }
}
impl std::fmt::Debug for ProArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProArguments([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ContentResultWire")]
pub struct ContentResult {
    pub title: String,
    pub url: PublicUrl,
    pub snippet: String,
    pub text: String,
    #[serde(rename = "content_status")]
    pub content_status: ContentStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContentResultWire {
    pub title: String,
    pub url: PublicUrl,
    pub snippet: String,
    pub text: String,
    #[serde(rename = "content_status")]
    pub content_status: ContentStatus,
}
impl TryFrom<ContentResultWire> for ContentResult {
    type Error = &'static str;
    fn try_from(w: ContentResultWire) -> Result<Self, Self::Error> {
        let v = Self {
            title: w.title,
            url: w.url,
            snippet: w.snippet,
            text: w.text,
            content_status: w.content_status,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ContentResult {
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
        {
            let value_text = &self.text;
            if value_text.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        let available = !self.text.trim().is_empty();
        if (self.content_status == ContentStatus::Available && !available)
            || (self.content_status == ContentStatus::Unavailable && available)
        {
            return Err("content status disagrees with body");
        }
        Ok(())
    }
}
impl std::fmt::Debug for ContentResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContentResult([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProResultWire")]
pub struct ProResult {
    pub title: String,
    pub url: PublicUrl,
    pub snippet: String,
    #[serde(rename = "site_name")]
    pub site_name: String,
    pub date: String,
    pub authority: String,
    pub chunks: Vec<SearchChunk>,
    #[serde(rename = "content_status")]
    pub content_status: ContentStatus,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProResultWire {
    pub title: String,
    pub url: PublicUrl,
    pub snippet: String,
    #[serde(rename = "site_name")]
    pub site_name: String,
    pub date: String,
    pub authority: String,
    pub chunks: Vec<SearchChunk>,
    #[serde(rename = "content_status")]
    pub content_status: ContentStatus,
}
impl TryFrom<ProResultWire> for ProResult {
    type Error = &'static str;
    fn try_from(w: ProResultWire) -> Result<Self, Self::Error> {
        let v = Self {
            title: w.title,
            url: w.url,
            snippet: w.snippet,
            site_name: w.site_name,
            date: w.date,
            authority: w.authority,
            chunks: w.chunks,
            content_status: w.content_status,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProResult {
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
        {
            let value_site_name = &self.site_name;
            if value_site_name.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        {
            let value_date = &self.date;
            if value_date.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        {
            let value_authority = &self.authority;
            if value_authority.chars().count() > 65536 {
                return Err("invalid broker text");
            }
        }
        {
            let value_chunks = &self.chunks;
            for item in value_chunks.iter() {
                item.validate()?;
            }
        }
        let available = self.chunks.iter().any(|c| !c.text.trim().is_empty());
        if (self.content_status == ContentStatus::Available && !available)
            || (self.content_status == ContentStatus::Unavailable && available)
        {
            return Err("content status disagrees with chunks");
        }
        Ok(())
    }
}
impl std::fmt::Debug for ProResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProResult([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ContentSuccessWire")]
pub struct ContentSuccess {
    pub results: Vec<ContentResult>,
    pub truncated: bool,
    #[serde(rename = "dropped_results")]
    pub dropped_results: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContentSuccessWire {
    pub results: Vec<ContentResult>,
    pub truncated: bool,
    #[serde(rename = "dropped_results")]
    pub dropped_results: i64,
}
impl TryFrom<ContentSuccessWire> for ContentSuccess {
    type Error = &'static str;
    fn try_from(w: ContentSuccessWire) -> Result<Self, Self::Error> {
        let v = Self {
            results: w.results,
            truncated: w.truncated,
            dropped_results: w.dropped_results,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ContentSuccess {
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
        {
            let value_dropped_results = &self.dropped_results;
            if *value_dropped_results < 0 {
                return Err("invalid broker number");
            }
        }
        if (self.dropped_results > 0
            || self
                .results
                .iter()
                .any(|r| r.content_status == ContentStatus::Truncated))
            && !self.truncated
        {
            return Err("missing top-level truncation");
        }
        Ok(())
    }
}
impl std::fmt::Debug for ContentSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContentSuccess([redacted])")
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ProSuccessWire")]
pub struct ProSuccess {
    pub results: Vec<ProResult>,
    pub truncated: bool,
    #[serde(rename = "dropped_results")]
    pub dropped_results: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProSuccessWire {
    pub results: Vec<ProResult>,
    pub truncated: bool,
    #[serde(rename = "dropped_results")]
    pub dropped_results: i64,
}
impl TryFrom<ProSuccessWire> for ProSuccess {
    type Error = &'static str;
    fn try_from(w: ProSuccessWire) -> Result<Self, Self::Error> {
        let v = Self {
            results: w.results,
            truncated: w.truncated,
            dropped_results: w.dropped_results,
        };
        v.validate()?;
        Ok(v)
    }
}
impl ProSuccess {
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
        {
            let value_dropped_results = &self.dropped_results;
            if *value_dropped_results < 0 {
                return Err("invalid broker number");
            }
        }
        if (self.dropped_results > 0
            || self
                .results
                .iter()
                .any(|r| r.content_status == ContentStatus::Truncated))
            && !self.truncated
        {
            return Err("missing top-level truncation");
        }
        Ok(())
    }
}
impl std::fmt::Debug for ProSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProSuccess([redacted])")
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
fn date_value(s: &str) -> Option<(u32, u32, u32)> {
    let p = s.split('-').collect::<Vec<_>>();
    if !(1..=3).contains(&p.len())
        || p[0].len() != 4
        || p.iter().skip(1).any(|v| v.len() != 2)
        || p.iter().any(|v| !v.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    let y = p[0].parse::<u32>().ok()?;
    let m = p.get(1).unwrap_or(&"01").parse::<u32>().ok()?;
    let d = p.get(2).unwrap_or(&"01").parse::<u32>().ok()?;
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => return None,
    };
    if y == 0 || d == 0 || d > days {
        return None;
    }
    Some((y, m, d))
}
impl ProArguments {
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_SEARCH_LIMIT as i64)
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
    ContentSuccess(ContentSuccess),
    ErrorOutput(ErrorOutput),
}
impl SearchOutput {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::SearchSuccess(value) => value.validate(),
            Self::ContentSuccess(value) => value.validate(),
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProOutput {
    ProSuccess(ProSuccess),
    ErrorOutput(ErrorOutput),
}
impl ProOutput {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::ProSuccess(value) => value.validate(),
            Self::ErrorOutput(value) => value.validate(),
        }
    }
}
pub const CONTENT_BUDGET_EXPIRED_MESSAGE: &str =
    "执行时间不足，请重新发起。本次网页检索未进入审批，外部请求未发送；不会自动重试。";
pub const SERVICE_ID: &str = "kimi-web";
pub const SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"$id\":\"https://schemas.yijie.ai/web-retrieval/v1\",\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"SearchArguments\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5},\"include_content\":{\"type\":\"boolean\",\"default\":false}}},\"FetchArguments\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"SearchOutput\":{\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ContentSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}]},\"FetchOutput\":{\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}]},\"ContentStatus\":{\"type\":\"string\",\"enum\":[\"available\",\"unavailable\",\"truncated\"]},\"ChunkScore\":{\"type\":\"number\",\"description\":\"Finite query-relative score. No cross-query trust threshold.\"},\"SearchChunk\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"text\",\"score\"],\"properties\":{\"text\":{\"type\":\"string\",\"maxLength\":65536},\"score\":{\"$ref\":\"#/$defs/ChunkScore\"}}},\"DateBound\":{\"anyOf\":[{\"type\":\"string\",\"pattern\":\"^(?!0000)[0-9]{4}$\"},{\"type\":\"string\",\"pattern\":\"^(?!0000)[0-9]{4}-(0[1-9]|1[0-2])$\"},{\"type\":\"string\",\"format\":\"date\",\"pattern\":\"^(?!0000)[0-9]{4}-[0-9]{2}-[0-9]{2}$\"}]},\"TimeWindow\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[],\"properties\":{\"start\":{\"$ref\":\"#/$defs/DateBound\"},\"end\":{\"$ref\":\"#/$defs/DateBound\"}}},\"ProArguments\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5},\"sites\":{\"type\":\"array\",\"maxItems\":5,\"items\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"pattern\":\"^[^\\\\s()]+$\"}},\"time_window\":{\"$ref\":\"#/$defs/TimeWindow\"}}},\"ContentResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\",\"text\",\"content_status\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536},\"text\":{\"type\":\"string\",\"maxLength\":65536},\"content_status\":{\"$ref\":\"#/$defs/ContentStatus\"}},\"allOf\":[{\"if\":{\"properties\":{\"content_status\":{\"const\":\"available\"}}},\"then\":{\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}},{\"if\":{\"properties\":{\"content_status\":{\"const\":\"unavailable\"}}},\"then\":{\"not\":{\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}]},\"ProResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\",\"site_name\",\"date\",\"authority\",\"chunks\",\"content_status\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536},\"site_name\":{\"type\":\"string\",\"maxLength\":65536},\"date\":{\"type\":\"string\",\"maxLength\":65536},\"authority\":{\"type\":\"string\",\"maxLength\":65536},\"chunks\":{\"type\":\"array\",\"items\":{\"$ref\":\"#/$defs/SearchChunk\"}},\"content_status\":{\"$ref\":\"#/$defs/ContentStatus\"}},\"allOf\":[{\"if\":{\"properties\":{\"content_status\":{\"const\":\"available\"}}},\"then\":{\"properties\":{\"chunks\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"required\":[\"text\"],\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}}},{\"if\":{\"properties\":{\"content_status\":{\"const\":\"unavailable\"}}},\"then\":{\"not\":{\"properties\":{\"chunks\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"required\":[\"text\"],\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}}}}]},\"ContentSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\",\"dropped_results\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/ContentResult\"}},\"truncated\":{\"type\":\"boolean\"},\"dropped_results\":{\"type\":\"integer\",\"minimum\":0}},\"allOf\":[{\"if\":{\"anyOf\":[{\"properties\":{\"dropped_results\":{\"type\":\"integer\",\"minimum\":1}}},{\"properties\":{\"results\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"properties\":{\"content_status\":{\"const\":\"truncated\"}},\"required\":[\"content_status\"]}}}}]},\"then\":{\"properties\":{\"truncated\":{\"const\":true}}}}]},\"ProSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\",\"dropped_results\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/ProResult\"}},\"truncated\":{\"type\":\"boolean\"},\"dropped_results\":{\"type\":\"integer\",\"minimum\":0}},\"allOf\":[{\"if\":{\"anyOf\":[{\"properties\":{\"dropped_results\":{\"type\":\"integer\",\"minimum\":1}}},{\"properties\":{\"results\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"properties\":{\"content_status\":{\"const\":\"truncated\"}},\"required\":[\"content_status\"]}}}}]},\"then\":{\"properties\":{\"truncated\":{\"const\":true}}}}]},\"ProOutput\":{\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/ProSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}]}}}";
pub const TOOLS_JSON:&str="[{\"name\":\"kimi_web_search\",\"description\":\"用户只要网页标题、摘要和链接列表，未限定站点/日期且不要求问答或分析时，直接使用本Basic工具，不使用Pro。用于展示网页标题、摘要、链接列表或找官网，默认include_content=false；用户明确采集正文供自行加工时才用true。选中本连接器的普通联网问答、事实核查、研究或总结直接用kimi_web_search_pro，不先Basic试探，也不为同一信息需求重复原生搜索。不能使用站点/日期过滤；明确只用Basic却要求过滤时说明冲突。只读按次计费，单次批准，不自动重试。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5},\"include_content\":{\"type\":\"boolean\",\"default\":false}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ContentSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ContentSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\",\"dropped_results\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/ContentResult\"}},\"truncated\":{\"type\":\"boolean\"},\"dropped_results\":{\"type\":\"integer\",\"minimum\":0}},\"allOf\":[{\"if\":{\"anyOf\":[{\"properties\":{\"dropped_results\":{\"type\":\"integer\",\"minimum\":1}}},{\"properties\":{\"results\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"properties\":{\"content_status\":{\"const\":\"truncated\"}},\"required\":[\"content_status\"]}}}}]},\"then\":{\"properties\":{\"truncated\":{\"const\":true}}}}]},\"ContentResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\",\"text\",\"content_status\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536},\"text\":{\"type\":\"string\",\"maxLength\":65536},\"content_status\":{\"$ref\":\"#/$defs/ContentStatus\"}},\"allOf\":[{\"if\":{\"properties\":{\"content_status\":{\"const\":\"available\"}}},\"then\":{\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}},{\"if\":{\"properties\":{\"content_status\":{\"const\":\"unavailable\"}}},\"then\":{\"not\":{\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}]},\"ContentStatus\":{\"type\":\"string\",\"enum\":[\"available\",\"unavailable\",\"truncated\"]},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}},{\"name\":\"kimi_web_search_pro\",\"description\":\"先区分交付物：仅列网页标题、摘要、链接列表、未限定站点/日期且不做问答或分析的任务，不使用本工具，只使用kimi_web_search（Basic摘要）。除此之外，选中本连接器时，普通联网问答、事实核查、研究、总结和RAG材料直接使用Pro，即使没有站点或时间过滤；不要先Basic或并行原生搜索。仅按用户明确要求设置sites(最多5，OR)及time_window；不得擅自增加过滤。返回关联真实URL的正文chunks，缺正文或截断须说明，score仅同次查询相对相关性。仅在还需某页当前正文时对结果URL另提Fetch，不声称历史快照。只读按次计费，单次批准，失败不自动重试/换接口。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5},\"sites\":{\"type\":\"array\",\"maxItems\":5,\"items\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"pattern\":\"^[^\\\\s()]+$\"}},\"time_window\":{\"$ref\":\"#/$defs/TimeWindow\"}},\"$defs\":{\"TimeWindow\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[],\"properties\":{\"start\":{\"$ref\":\"#/$defs/DateBound\"},\"end\":{\"$ref\":\"#/$defs/DateBound\"}}},\"DateBound\":{\"anyOf\":[{\"type\":\"string\",\"pattern\":\"^(?!0000)[0-9]{4}$\"},{\"type\":\"string\",\"pattern\":\"^(?!0000)[0-9]{4}-(0[1-9]|1[0-2])$\"},{\"type\":\"string\",\"format\":\"date\",\"pattern\":\"^(?!0000)[0-9]{4}-[0-9]{2}-[0-9]{2}$\"}]}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/ProSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"ProSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\",\"dropped_results\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/ProResult\"}},\"truncated\":{\"type\":\"boolean\"},\"dropped_results\":{\"type\":\"integer\",\"minimum\":0}},\"allOf\":[{\"if\":{\"anyOf\":[{\"properties\":{\"dropped_results\":{\"type\":\"integer\",\"minimum\":1}}},{\"properties\":{\"results\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"properties\":{\"content_status\":{\"const\":\"truncated\"}},\"required\":[\"content_status\"]}}}}]},\"then\":{\"properties\":{\"truncated\":{\"const\":true}}}}]},\"ProResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\",\"site_name\",\"date\",\"authority\",\"chunks\",\"content_status\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536},\"site_name\":{\"type\":\"string\",\"maxLength\":65536},\"date\":{\"type\":\"string\",\"maxLength\":65536},\"authority\":{\"type\":\"string\",\"maxLength\":65536},\"chunks\":{\"type\":\"array\",\"items\":{\"$ref\":\"#/$defs/SearchChunk\"}},\"content_status\":{\"$ref\":\"#/$defs/ContentStatus\"}},\"allOf\":[{\"if\":{\"properties\":{\"content_status\":{\"const\":\"available\"}}},\"then\":{\"properties\":{\"chunks\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"required\":[\"text\"],\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}}},{\"if\":{\"properties\":{\"content_status\":{\"const\":\"unavailable\"}}},\"then\":{\"not\":{\"properties\":{\"chunks\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"required\":[\"text\"],\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}}}}]},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"SearchChunk\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"text\",\"score\"],\"properties\":{\"text\":{\"type\":\"string\",\"maxLength\":65536},\"score\":{\"$ref\":\"#/$defs/ChunkScore\"}}},\"ChunkScore\":{\"type\":\"number\",\"description\":\"Finite query-relative score. No cross-query trust threshold.\"},\"ContentStatus\":{\"type\":\"string\",\"enum\":[\"available\",\"unavailable\",\"truncated\"]},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}},{\"name\":\"kimi_web_fetch\",\"description\":\"读取用户给定且明确要求阅读的公开URL，或在用户要求核对当前正文时读取先前搜索结果的同一URL，不为已知URL先搜索。URL仅为举例/线索不等于读取意图。返回提取的Markdown，可截断，不保证完整网页/历史快照，不提供登录态或JS控制。只读按次计费，另行单次批准，失败不自动重试。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}},\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}}]";
pub const LEGACY_TOOLS_JSON:&str="[{\"name\":\"kimi_web_search\",\"description\":\"按查询词获取网页标题、URL和摘要的结构化列表。需要结构化搜索结果时使用；一般联网回答可使用原生搜索。只读但按次计费，每次需要批准，不自动重试。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}},{\"name\":\"kimi_web_fetch\",\"description\":\"读取指定公开HTTP(S)网页的标题、URL和Markdown正文。用于需要指定URL具体正文的任务；不提供原始HTML、登录态访问或JavaScript渲染控制。只读但按次计费，每次需要批准，不自动重试。\",\"inputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}},\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"}}},\"outputSchema\":{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}},\"annotations\":{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}}]";
pub const SEARCH_TOOL_NAME: &str = "kimi_web_search";
pub const SEARCH_DESCRIPTION:&str="用户只要网页标题、摘要和链接列表，未限定站点/日期且不要求问答或分析时，直接使用本Basic工具，不使用Pro。用于展示网页标题、摘要、链接列表或找官网，默认include_content=false；用户明确采集正文供自行加工时才用true。选中本连接器的普通联网问答、事实核查、研究或总结直接用kimi_web_search_pro，不先Basic试探，也不为同一信息需求重复原生搜索。不能使用站点/日期过滤；明确只用Basic却要求过滤时说明冲突。只读按次计费，单次批准，不自动重试。";
pub const SEARCH_INPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5},\"include_content\":{\"type\":\"boolean\",\"default\":false}}}";
pub const SEARCH_OUTPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/SearchSuccess\"},{\"$ref\":\"#/$defs/ContentSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"SearchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/SearchResult\"}},\"truncated\":{\"type\":\"boolean\"}}},\"SearchResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ContentSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\",\"dropped_results\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/ContentResult\"}},\"truncated\":{\"type\":\"boolean\"},\"dropped_results\":{\"type\":\"integer\",\"minimum\":0}},\"allOf\":[{\"if\":{\"anyOf\":[{\"properties\":{\"dropped_results\":{\"type\":\"integer\",\"minimum\":1}}},{\"properties\":{\"results\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"properties\":{\"content_status\":{\"const\":\"truncated\"}},\"required\":[\"content_status\"]}}}}]},\"then\":{\"properties\":{\"truncated\":{\"const\":true}}}}]},\"ContentResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\",\"text\",\"content_status\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536},\"text\":{\"type\":\"string\",\"maxLength\":65536},\"content_status\":{\"$ref\":\"#/$defs/ContentStatus\"}},\"allOf\":[{\"if\":{\"properties\":{\"content_status\":{\"const\":\"available\"}}},\"then\":{\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}},{\"if\":{\"properties\":{\"content_status\":{\"const\":\"unavailable\"}}},\"then\":{\"not\":{\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}]},\"ContentStatus\":{\"type\":\"string\",\"enum\":[\"available\",\"unavailable\",\"truncated\"]},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}}";
pub const SEARCH_PATH: &str = "/v1/tools/search";
pub const SEARCH_ANNOTATIONS_JSON:&str="{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}";
pub const PRO_TOOL_NAME: &str = "kimi_web_search_pro";
pub const PRO_DESCRIPTION:&str="先区分交付物：仅列网页标题、摘要、链接列表、未限定站点/日期且不做问答或分析的任务，不使用本工具，只使用kimi_web_search（Basic摘要）。除此之外，选中本连接器时，普通联网问答、事实核查、研究、总结和RAG材料直接使用Pro，即使没有站点或时间过滤；不要先Basic或并行原生搜索。仅按用户明确要求设置sites(最多5，OR)及time_window；不得擅自增加过滤。返回关联真实URL的正文chunks，缺正文或截断须说明，score仅同次查询相对相关性。仅在还需某页当前正文时对结果URL另提Fetch，不声称历史快照。只读按次计费，单次批准，失败不自动重试/换接口。";
pub const PRO_INPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"query\"],\"properties\":{\"query\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":2048,\"pattern\":\"\\\\S\"},\"limit\":{\"type\":\"integer\",\"minimum\":1,\"maximum\":20,\"default\":5},\"sites\":{\"type\":\"array\",\"maxItems\":5,\"items\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"pattern\":\"^[^\\\\s()]+$\"}},\"time_window\":{\"$ref\":\"#/$defs/TimeWindow\"}},\"$defs\":{\"TimeWindow\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[],\"properties\":{\"start\":{\"$ref\":\"#/$defs/DateBound\"},\"end\":{\"$ref\":\"#/$defs/DateBound\"}}},\"DateBound\":{\"anyOf\":[{\"type\":\"string\",\"pattern\":\"^(?!0000)[0-9]{4}$\"},{\"type\":\"string\",\"pattern\":\"^(?!0000)[0-9]{4}-(0[1-9]|1[0-2])$\"},{\"type\":\"string\",\"format\":\"date\",\"pattern\":\"^(?!0000)[0-9]{4}-[0-9]{2}-[0-9]{2}$\"}]}}}";
pub const PRO_OUTPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/ProSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"ProSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"results\",\"truncated\",\"dropped_results\"],\"properties\":{\"results\":{\"type\":\"array\",\"maxItems\":20,\"items\":{\"$ref\":\"#/$defs/ProResult\"}},\"truncated\":{\"type\":\"boolean\"},\"dropped_results\":{\"type\":\"integer\",\"minimum\":0}},\"allOf\":[{\"if\":{\"anyOf\":[{\"properties\":{\"dropped_results\":{\"type\":\"integer\",\"minimum\":1}}},{\"properties\":{\"results\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"properties\":{\"content_status\":{\"const\":\"truncated\"}},\"required\":[\"content_status\"]}}}}]},\"then\":{\"properties\":{\"truncated\":{\"const\":true}}}}]},\"ProResult\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"title\",\"url\",\"snippet\",\"site_name\",\"date\",\"authority\",\"chunks\",\"content_status\"],\"properties\":{\"title\":{\"type\":\"string\",\"maxLength\":65536},\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"snippet\":{\"type\":\"string\",\"maxLength\":65536},\"site_name\":{\"type\":\"string\",\"maxLength\":65536},\"date\":{\"type\":\"string\",\"maxLength\":65536},\"authority\":{\"type\":\"string\",\"maxLength\":65536},\"chunks\":{\"type\":\"array\",\"items\":{\"$ref\":\"#/$defs/SearchChunk\"}},\"content_status\":{\"$ref\":\"#/$defs/ContentStatus\"}},\"allOf\":[{\"if\":{\"properties\":{\"content_status\":{\"const\":\"available\"}}},\"then\":{\"properties\":{\"chunks\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"required\":[\"text\"],\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}}},{\"if\":{\"properties\":{\"content_status\":{\"const\":\"unavailable\"}}},\"then\":{\"not\":{\"properties\":{\"chunks\":{\"type\":\"array\",\"contains\":{\"type\":\"object\",\"required\":[\"text\"],\"properties\":{\"text\":{\"type\":\"string\",\"pattern\":\"\\\\S\"}}}}}}}}]},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"SearchChunk\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"text\",\"score\"],\"properties\":{\"text\":{\"type\":\"string\",\"maxLength\":65536},\"score\":{\"$ref\":\"#/$defs/ChunkScore\"}}},\"ChunkScore\":{\"type\":\"number\",\"description\":\"Finite query-relative score. No cross-query trust threshold.\"},\"ContentStatus\":{\"type\":\"string\",\"enum\":[\"available\",\"unavailable\",\"truncated\"]},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}}";
pub const PRO_PATH: &str = "/v1/tools/search_pro";
pub const PRO_ANNOTATIONS_JSON:&str="{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}";
pub const FETCH_TOOL_NAME: &str = "kimi_web_fetch";
pub const FETCH_DESCRIPTION:&str="读取用户给定且明确要求阅读的公开URL，或在用户要求核对当前正文时读取先前搜索结果的同一URL，不为已知URL先搜索。URL仅为举例/线索不等于读取意图。返回提取的Markdown，可截断，不保证完整网页/历史快照，不提供登录态或JS控制。只读按次计费，另行单次批准，失败不自动重试。";
pub const FETCH_INPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"}},\"$defs\":{\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"}}}";
pub const FETCH_OUTPUT_SCHEMA_JSON:&str="{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\",\"type\":\"object\",\"oneOf\":[{\"$ref\":\"#/$defs/FetchSuccess\"},{\"$ref\":\"#/$defs/ErrorOutput\"}],\"$defs\":{\"FetchSuccess\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"url\",\"title\",\"markdown\",\"truncated\"],\"properties\":{\"url\":{\"$ref\":\"#/$defs/PublicUrl\"},\"title\":{\"type\":\"string\",\"maxLength\":65536},\"markdown\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":65536,\"pattern\":\"\\\\S\"},\"truncated\":{\"type\":\"boolean\"}}},\"PublicUrl\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":4096,\"format\":\"uri\",\"pattern\":\"^https?://\",\"description\":\"Public HTTP(S) URL. Connector additionally parses authority, rejects userinfo/control characters and local/private literal destinations; no local request to this target is made.\"},\"ErrorOutput\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"error\"],\"properties\":{\"error\":{\"$ref\":\"#/$defs/ToolError\"}}},\"ToolError\":{\"type\":\"object\",\"additionalProperties\":false,\"required\":[\"code\",\"message\"],\"properties\":{\"code\":{\"$ref\":\"#/$defs/ErrorCode\"},\"message\":{\"type\":\"string\",\"minLength\":1,\"maxLength\":512}},\"allOf\":[{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_arguments\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"检索参数无效，请检查查询词、数量或网页地址。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"authentication_failed\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi API Key 无效或已失效，请重新配置。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"permission_denied\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"服务拒绝了此请求，请检查账户权限或目标网页是否允许读取。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"rate_limited\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"Kimi 请求频率或并发受限，请确认后重新发起调用。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"timeout\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索超时，未自动重试；已发送的请求可能计费。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"unavailable\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务暂不可用，未自动重试。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"no_content\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"目标网页未返回可读取的正文。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"response_too_large\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索结果超过可处理的大小限制。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"invalid_response\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"网页检索服务返回了无法验证的结果。\"}},\"required\":[\"message\"]}},{\"if\":{\"properties\":{\"code\":{\"const\":\"cancelled\"}},\"required\":[\"code\"]},\"then\":{\"properties\":{\"message\":{\"const\":\"本次网页检索已取消；已发送的请求可能计费。\"}},\"required\":[\"message\"]}}]},\"ErrorCode\":{\"type\":\"string\",\"enum\":[\"invalid_arguments\",\"authentication_failed\",\"permission_denied\",\"rate_limited\",\"timeout\",\"unavailable\",\"no_content\",\"response_too_large\",\"invalid_response\",\"cancelled\"]}}}";
pub const FETCH_PATH: &str = "/v1/tools/fetch";
pub const FETCH_ANNOTATIONS_JSON:&str="{\"readOnlyHint\":true,\"destructiveHint\":false,\"idempotentHint\":false,\"openWorldHint\":true}";
pub const DEFAULT_SEARCH_LIMIT: usize = 5;
pub const MAX_RESULT_BYTES: usize = 65536;
pub const MAX_RESPONSE_BYTES: usize = 1048576;
pub const HTTP_TIMEOUT_MS: usize = 15000;
pub const SEARCH_TIMEOUT_SECONDS: usize = 10;
pub const CONTENT_HTTP_TIMEOUT_MS: usize = 35000;
pub const CONTENT_SEARCH_TIMEOUT_SECONDS: usize = 30;
pub const APPROVAL_DELIVERY_MARGIN_MS: usize = 2000;
pub const MCP_TOOL_TIMEOUT_MS: usize = 60000;
