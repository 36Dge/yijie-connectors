# Kimi 网页检索：FEAT-162 本地候选

## 2026-10-29 Basic/Pro 实施候选（最新）

当前范围以元仓22/24号为准。一个服务新增第三工具 kimi_web_search_pro；Basic 默认仍 10s 上游/至多15s本机，显式 include_content=true 和 Pro 为上游30s、本机完整35s，Fetch不变。新模式仅在新 Host 成对声明 0.3.0/60000 的 lease 下可见/准入；旧 lease 只见冻结两工具。

新模式 D 受原 call、lease/scope、prepare+60s 保守 MCP 下界限制，A=D−35s−2s。A只关闭待批准状态，已批准者仍受 D；发送前剩余不足35s时零HTTP。前置模型/排队时间会缩短批准窗口，可能安全拒绝，真实时延尚未验收。

`kimi_web_content.rs` 逐项过滤无效页并保留供应商顺序；原始空列表成功、全无效 invalid_response，合法必要来源无法交付 response_too_large。每页 available/unavailable/truncated 与 dropped_results 原始减最终计数保持；只裁 UTF-8 文本、不裁 URL，完整双份 MCP 封包≤65536 bytes。原 Basic/Fetch 归一化独立保留。Pro只透传用户给的 sites/time_window，不增筛选或重试。

本轮24项 Kimi focused tests、格式/Clippy及 canonical worker 构建证据在元仓24号；真实供应商/模型请求为0。正规 worker 输出到新隔离 generation，不覆盖运行中的/固定 Runtime 制品，不启动该 worker。下面2026-10-24记录及真实旧两工具结果属于历史，不能代表新 Pro/正文通过。


2026-10-24。实施范围以元仓 `docs/features/FEAT-162-kimi-web-retrieval/00-feature-brief.md` 及后续 `05-model-key-reuse-plan.md` 为准。目录 revision 11 实际 61 项，管理 reader 容量 62；新服务 `kimi-web` 包含两个稳定 MCP 工具。尚未发布，不代表真实账号或付费工具验收完成。

## 执行与凭据

`worker/src/kimi_web.rs` 是本机 worker 拥有的 REST adapter，经既有 Broker/Gateway 暴露 `kimi_web_search` 与 `kimi_web_fetch`，不启动远程 MCP 会话或额外运行时。两个 descriptor、输入、输出、错误文案和预算全部消费 Contracts 独立 `web-retrieval/1` 生成材料。工具结果仍属于 MCP Item，不写入原生搜索 sources。

用户于2026-10-24明确授权复用已配置的Kimi模型Key。本轮替代早期独立Keyring表单：Host将其已加载的同一模型Key快照通过单次匿名FD3管道交给worker，版本前缀/16KiB上限/末尾换行及EOF均校验；worker在线程启动前读取并关闭，不把Key放argv/env、控制JSON、公开IPC、日志或第二份持久存储。Native默认安装并复用既有Enable恢复链完成本地准备，用户无需配置或启用连接器。缺失来源返回not_configured并提示先完成统一模型配置；正常重开重新载入当前快照。明确停用/卸载的选择保留，不删除模型Key；其他服务继续使用既有Keyring。

Kimi Probe仅本地加载固定canonical工具定义和当前凭据快照并注册精确scope/binding，不发送任何HTTP，包括/models、Search、Fetch或模型请求。connected/qualified/executionAvailable表示当前本地适配器已准备好接受受审调用，不表示供应商鉴权、业务权限、余额或网页读取已验证。真实调用仍需任务选用及单次审批。

Search 固定 POST `/v1/tools/search`，只发送 `text_query`、`limit`（默认 5，范围 1–20）、`timeout_seconds=10`、`include_content=false`。Fetch 固定 POST `/v1/tools/fetch`，只发送已审查的原始 `url`。本机不会直接访问目标网页、跟随链接、读取登录态、执行网页脚本或下载图片。只支持正文 Markdown，不提供原始 HTML 或可控 JavaScript 渲染。

## 准入、容量与失败

只有当前显式选集、安装 generation、有效租约和每次双决策批准才能进入执行。工具为 read，但仍按次审批并提示可能计费。冻结参数及 schema digest 绑定到同一调用；一次 HTTP 许可在实际 POST 前消费。停用/卸载撤销准入并等待既有 owner 正常结束，未完成清理不得假报成功。

单次 HTTP 总预算最多 15 秒并受租约剩余时间限制；无 redirect、后台刷新、结果缓存、自动链接抓取或第二次模型请求。业务错误、超时、取消和未知结果不自动重试。发送后取消不承诺免计费，原请求在有限预算内正常结束，结果不再投递为成功。

独立代码审查发现固定库通用 `ReqwestHttpClient` 保留 reqwest 协议重试默认值。因此经元仓方案第 11 节记录的最小修正，本 adapter 直接使用既有锁中的 `reqwest=0.12.28`，显式 `retry::never()` 与 `redirect::Policy::none()`；不升级外部依赖，不修改 Runtime 源。worker 根包增加直接依赖后，外部依赖 identity/closure 必须保持一致。

响应按流读取，原始数据最大 1 MiB；超限明确报错，不截断 JSON 后解析。成功结果同时包含相同的 JSON 文本和 `structuredContent`；整个序列化 MCP 包最大 65536 字节。Fetch 只在 UTF-8 边界缩短正文，Search 缩短摘要或移除列表尾部，并显式 `truncated=true`；不剪短 URL 形成其他链接。必要元数据无法容纳时返回错误。空 Search 是 `results=[]` 成功，空白 Fetch 正文是 `no_content`。

供应商任意错误正文不透传；错误只使用 canonical 闭集代码和本地中文消息。结果用生成 validator 和 SecretGuard 验证，URL 是惰性文本。诊断只记录固定阶段、状态码及安全错误，不记完整 query、目标 URL、网页正文或密钥。

## 验证入口与限制

```bash
make worker-test-kimi
make worker-lint
go test -race ./catalog ./internal/app
make worker-build
```

`contract-check` 及 canonical build 同时检查 `web-retrieval` 和已有三个家族；生成材料只通过 Contracts 同步，不手改。worker build 仍按内容摘要保存新制品，不覆盖固定 Codex Runtime 或已有内容地址制品。现有 `worker/source.lock.json` 用户改动不由本功能覆写。

本次新增测试只用普通合成 query、公开示例 URL、正文、状态码、进程内受控响应和正常用户批准/拒绝/取消；没有外部 API、真实 Keyring、模型或付费请求。定向回归包括原一次许可、正常取消、Tushare 双批准与凭据表单单次提交。未运行未筛选的全套测试、攻击注入、进程故障或权限破坏；这些不属于本次验证证据。

2026-10-24 本地执行记录：

| 检查 | 结果 |
| --- | --- |
| `make worker-test-kimi`（最终 `--offline --locked`） | 13 passed，80 filtered out；包括一次真实本地 loopback HTTP 成功、计数 1、正常 graceful shutdown |
| `make worker-lint` | 四族源同步检查、固定 Runtime 库来源检查、rustfmt、Clippy `-D warnings` 通过 |
| `go test -race ./catalog ./internal/app` | 两包通过 |
| 五项命名既有测试 | 一次许可、发送前正常取消、Tushare 双批准、普通凭据单次提交、61 项 registry 一致性通过 |
| Cargo 依赖比较 | 876 个外部 package 的完整记录（版本、来源、checksum、dependencies）不变；正规 Cargo 解析仅为 worker 根包新增 `reqwest 0.12.28` 直接依赖 |
| `git diff --check` | 通过 |

首轮 Clippy 的布尔表达式简化提示已修复；独立审查发现的默认 HTTP 协议重试风险也已修复并复跑上述最终 focused tests 与 Clippy。canonical worker/App 构建与整体验证由元仓交付记录追踪，不把这里的 unit-test binary 当成交付制品。

最新真实验收已完成：复用既有Kimi模型Key，产品worker/Broker/Gateway正常拒绝后0请求，Search一次返回3条标题URL摘要，Fetch一次返回751字符Markdown，两项均HTTP200且未截断，正常EOF退出。模型调用0、供应商重试0，按官方单价核算¥0.02，未查账户账单；本轮额度已用完。自动准备的实际App及重开见元仓09，真实工具证据见13；模型驱动的Native UI审批、工具结果历史及完整D4仍NOT RUN，不扩称全部验收完成。

官方接口依据：

- https://platform.kimi.com/docs/api/tools-search
- https://platform.kimi.com/docs/api/tools-fetch
- https://platform.kimi.com/docs/api/list-models

2026-10-30 构建接口修订（semantic，工具/公共 wire 不变）：`build-market-worker.sh --check-only [--output-root ...]` 只检查输出目录、固定源码/工具链及四家族来源，成功后退出，不编译/发布。Desktop 在全部来源检查完成后再调用原标准 build。新 primary generation 通过 `YIJIE_WORKER_BUILD_DIR` 指定独立中间缓存；脚本将其映射到稳定 Cargo `CARGO_BUILD_BUILD_DIR`，而 `YIJIE_WORKER_TARGET_DIR` 和内容寻址 worker 输出仍逐代独立。旧调用方式保持。缓存策略/容量/互斥由 Desktop primary owner 管理，缓存不替代来源、manifest 和哈希校验；08/09 和历史产物不迁移。本轮 `--check-only` 与现有正规 worker 的本机 Host 集成通过，未重编 worker、构建 App 或调用供应商。
