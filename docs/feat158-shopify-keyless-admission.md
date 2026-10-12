# FEAT-158 Shopify metadata admission correction

2026-10-11, local candidate only. `contract-impact = semantic`: this corrects the
transport behavior for the existing fixed `shopify` registry entry whose
`authMode` is already `none`. No schema, OAuth identity, registry URL, tool
definition, Runtime source, or account credential is added or changed.

The shared HTTP client previously required a Bearer header whenever no explicit
header credential existed. Shopify passes the keyless readiness check and has no
credential; consequently its ordinary `initialize` was rejected locally before
HTTP. An established GET stream also incorrectly required Bearer authentication.

The exception now requires the fixed Shopify registry identity, `authMode=none`,
HTTP transport, and the exact registry resource URL. Its target only permits
GET/POST/DELETE against that resource, with no OAuth discovery or token endpoint.
POST still accepts only the existing metadata methods or a `tools/call` with the
existing exact one-shot permit. Redirect blocking, bounded timeout and the real
worker User-Agent remain. OAuth and header credential services retain their
authentication checks; `credential=None` alone never grants an exception.

Verification uses ordinary in-memory metadata requests, declared OAuth/API-key
header modes and pending/approved one-shot permission state. It does not start a
server, contact Shopify, call a real tool, use real credentials, or alter file
permissions. The canonical build creates a new content-addressed worker and
preserves the previous digest directory. Checks and final artifact digests are
recorded in `.local/feat158-phase6-shopify-checks.json`; real UI metadata discovery
and expert selection acceptance are owned by the main task and are not claimed
by these tests.

## Targeted notification compatibility correction

The later targeted public metadata observation established the actual remaining
failure: initialization returns a valid JSON-RPC result, then
`notifications/initialized` returns HTTP 200, `application/json`,
`Content-Length: 4`, no Transfer-Encoding, and exactly the JSON literal `null`.
The fixed SDK cannot deserialize `null` as `ServerJsonRpcMessage`; the normal
notification acknowledgement is an empty accepted response. The earlier six
HTTP 200 events therefore were not successful MCP qualification. Source evidence
is recorded by the main task in FEAT-158 `phase7-targeted-ledger.json`.

The worker now bridges only this observed acknowledgement, after the normal
admission check, for the exact fixed Shopify endpoint and exact JSON-RPC
`notifications/initialized` notification. Only HTTP 200 with one JSON media-type
header, one Content-Length equal to 4, and no Transfer-Encoding is inspected.
It reads at most four body bytes and requires normal EOF and literal `null`
before returning an empty 204 internally. The actual HTTP 200 is recorded before
adaptation. Other responses retain their original unconsumed stream; body or
transport errors remain errors. Request parsing, tools/call permission, headers,
timeouts, cancellation, authentication and fixed Runtime source are unchanged.
This is a narrow semantic provider adapter correction, not a new wire or schema.

Seven focused tests pass: two new tests cover exact registry/method selection and
the actual fixed SDK with an ordinary loopback server; the latter reproduces the
original null-response failure and checks the bridge plus the SDK's existing
accepted forms, empty 204 and valid JSON 200. These SDK compatibility forms are
not both normative MCP acknowledgements; MCP specifies empty 202 for an accepted
notification. Five existing normal admission/permission/HTTP checks also pass.
Servers use graceful shutdown; no fault injection or external request occurs.
Scoped formatting, Clippy and the canonical worker build are recorded separately
in `.local/feat158-phase7-shopify-checks.json`. These local protocol checks do not
claim that the updated product has completed live Shopify tools discovery or
expert selection. No additional real metadata, model or UI run is performed by
this patch.
# 2026-10-13 组合验收诊断增量

随后真实诊断确认：三次完成通知均为HTTP200 JSON、无Content-Length且有Transfer-Encoding，旧兼容均未命中；单组2请求的只读传输观察确认完整4字节`null`且Transfer-Encoding为chunked。根因是已知null响应采用不同合法HTTP framing，而旧代码把Content-Length:4作为必要条件。

兼容增量仍只作用于固定Shopify URL的精确无id/params完成通知：除原有定长形式外，接受无长度头且单个chunked头的JSON形式；仍有最多4字节和完整EOF约束，只有精确null可内部转为204。内部已消费204移除实体/分块头，诊断保留真实200。其他方法、SSE、标准202/204和其他已知长度JSON继续原路径；该固定通知的chunked JSON若不是完整null则拒绝，不概括为通用JSON修复。不改SDK/Runtime源码、业务许可或审批。

正常本地分块`nu`/`ll`回归复现固定SDK失败，兼容后初始化与tools/list成功；定长null、现有204和定长有效JSON保持。真实产品复验结果独立记录于元仓phase9，不以本地回归冒充通过。

独立专家候选再次出现6次HTTP200后initialize失败，未进入tools/list。原日常客户端的成功事实保留；不能用旧日志唯一归因。此增量仅在既有有界、owner-only诊断中追加固定Shopify元数据的方法枚举、精确通知识别布尔值、响应类型布尔分类、长度数字/计数及Transfer-Encoding存在性，并记录原兼容步骤是否执行。

不记录URL、原始headers、响应正文、身份、session或凭据；不增加外部请求，不改变传输/兼容/权限逻辑。私有诊断为兼容追加字段，公共wire/schema不变。实际失败形态确认前不放宽响应规则；诊断不等于修复或验收通过。


## 2026-10-13 最终 Git 审计

本次只提交本说明、`worker/src/tushare_oauth.rs` 及 `worker/src/shopify_ack_tests.rs`；`.local` 原始诊断留在本机。phase9 和 phase11 的真实成功证据已在元仓交付包登记；上文失败与当时未执行范围为历史，不撤销。当前 worker 43 项构建输入与日常客户端 phase12 回执逐字节一致，本轮复用已通过的正常协议、格式、Clippy 与构建证据，无额外 Shopify 请求或模型调用。源码推送不扩大已资格化工具或产品部署范围。
