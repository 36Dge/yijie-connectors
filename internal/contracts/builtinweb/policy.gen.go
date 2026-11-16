// Generated from builtin-web-retrieval/1. DO NOT EDIT.
package builtinweb

import _ "embed"

const ServerName = "yijie_web"
const ServerInstructions = "Built-in read-only web retrieval needs no connector selection. Use returned results to decide follow-up retrieval: continue only for a specific missing part of the requested deliverable, without adding verification goals. Once the requested items or facts are supported, finish within the user's full output limit. For an unfiltered source list or material collection, call kimi_web_search (Basic). Titles, URLs and short descriptions for later reading are a collection deliverable, not ordinary Q&A or a researched answer. Provider-hosted pre-search does not replace this Basic step; reuse a sufficient collection already returned by Basic for the same need. With include_content=false, enough relevant titles, URLs and snippets satisfy a metadata-only list. Do not run Pro or Fetch merely to verify those links or improve that completed list. Site/date filters or researched conclusions instead require Pro. Ordinary questions normally use native web_search; an explicit URL-reading request uses kimi_web_fetch. For a researched answer or comparison needing body evidence, or a search with explicit site/date restrictions, call kimi_web_search_pro before answering and pass requested sites/time_window filters. An unfiltered list of sources for later reading belongs to Basic. Provider-hosted pre-search may occur independently, but does not replace this Pro step or its filters. Reuse sufficient Pro evidence already returned for the same information need; do not repeat a successful search just to satisfy this rule. Read an explicitly requested public HTTP(S) URL, or a returned research source whose body is still needed after consuming Pro. A URL in a source list or used only as an example is not itself a reading request. This tool accepts only url, with no pagination or adjustable content limit. A successful truncated=true result is usable partial evidence: do not refetch it to obtain a missing tail. Changing only #fragment is not pagination; reuse the returned page body. If required facts remain missing, use a focused Pro query with sites or state the gap. An explicit refresh is a new need for the same page. In Code Mode, inspect or emit structuredContent rather than both structuredContent and content. Preserve error/truncation status and never claim missing content was retrieved."
const ApprovalMode = "auto"
const ToolTimeoutSeconds = 60
const StartupTimeoutSeconds = 10
const SubjectConcurrency = 2
const APIIdentity = "spiffe://yijie/services/api"
const ExecutorIdentity = "spiffe://yijie/services/web-executor"
const LocalAPIOrigin = "http://127.0.0.1:18890"
const LocalCredentialEnvironment = "YIJIE_WEB_LOCAL_CREDENTIAL_FILE"
const LocalEpochHeader = "X-Yijie-Run-Epoch"
const LocalOwnerID = "12500000-0000-4000-8000-000000000001"
const LocalTenantID = "12500000-0000-4000-8000-100000000001"
const RetiredServiceID = "kimi-web"
const CredentialPipeEnvironmentKey = "YIJIE_MARKET_KIMI_MODEL_KEY_FD"
const CredentialPipePreamble = "yijie-kimi-model-key-v1\n"
const CredentialPipeTerminator = "\n"
const CredentialPipeMaxKeyBytes = 16384
const MaxResultBytes = 65536

var ToolNames = [...]string{"kimi_web_search", "kimi_web_search_pro", "kimi_web_fetch"}
var ErrorMessages = map[string]string{"invalid_arguments": "网页检索参数无效。", "session_expired": "产品会话已过期，请恢复易界会话。", "not_entitled": "当前账户没有网页检索资格。", "quota_exceeded": "网页检索额度已用尽。", "busy": "网页检索暂时繁忙。", "upstream_unavailable": "网页检索服务暂不可用。", "timeout": "网页检索请求超时，未自动重试。", "no_content": "网页未返回可用正文。", "response_too_large": "网页检索结果超过限制。", "invalid_response": "网页检索服务返回了无效结果。", "cancelled": "网页检索已取消。", "outcome_unknown": "请求可能已发送，结果暂时未知。", "operation_conflict": "操作标识与原请求不一致。", "operation_expired": "操作标识已超过新准入期限。", "result_unavailable": "原操作结果正文不可恢复，请勿自动重发。"}

type ExpertPolicy struct{ ExpertID, Version, RoleSHA256, Policy string }

var ExpertPolicies = [...]ExpertPolicy{{"data-analyst-pro", "0.1.0", "cc86509bd4f6289f60ea8677cc5e813b2547d99aabbae14309c22a7d8ac4ca78", "disabled"}, {"data-analyst-pro", "0.1.1", "34a44b6580e5b284e925dbf2719c9f73d30b4ef74f354269437a3438a1bbe877", "disabled"}, {"data-analyst-pro", "0.1.2", "ca15eb3ce9578c52f32b0aeaedae786e54a5a7737a34ec1b7bbe9f9b8f4bd910", "disabled"}}

//go:embed tools.json
var Tools []byte

//go:embed wire.schema.json
var Schema []byte
