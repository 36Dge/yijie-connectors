# FEAT-157 私有 Broker 控制与本地 Gateway

2026-10-07。以下第一阶段验证制品和run记录保留历史原义；当前OAuth网络能力及v2产品manifest见 [Tushare受管候选](tushare/oauth-implementation.md)。本地 source-first candidate；整体 FEAT-157 contract-impact 按 breaking 治理。独立新增 `market-broker-control/1` 不修改旧管理 family 的55个定义和9个 Native 命令，也不修改固定 Runtime。没有不可变发布 pin，不能用于合并/公开激活/D4。

## 进程与数据面

Host 直接拥有标准构建的 `yijie-mcp-worker --broker-control-stdio`。私有 stdin/stdout 是闭合、64 KiB 上限的 JSONL 控制流，8个动作和 DTO 全部从 Contracts `compatibility/market-broker-control` 派生。默认不传参数仍是旧16 KiB `auth_status` 模式。新增模式不经过 Go 纯转发服务。

Broker 使用固定依赖闭包中的官方 `rmcp = 1.8.0` streamable HTTP server，只绑定 `127.0.0.1:0`。初始化返回 `http://127.0.0.1:<port>/mcp`，每个租期的数据路径是 `.../mcp/<capabilityRef>`。capabilityRef 仅定位，不是身份凭证。每个 HTTP 请求还要求专属 `YIJIE_MARKET_RUNTIME_CAPABILITY` 环境能力；它不进入控制 wire、生成 DTO、日志或回执。Host 将此能力仅注入该 Runtime generation，并管理工具环境隔离；本模块不负责替 Host 授权 Native scope。带浏览器 Origin 的请求不属于该入口，控制协议不能通过 HTTP 访问。

官方 rmcp 负责协议初始化、会话和请求生命周期。私有 Broker 负责业务准入，不能把 bearer、MCP session ID 或 Runtime metadata 当作准入决定。所有51个真实 provider 仍为 `not_qualified`，产品二进制对非空选择返回 `not_qualified`；空选集可以验证控制生命周期，但不暴露任何工具。固定 Codex 出站 client 的窄库边界仍保留，尚未提供任何真实 approved target。

## 租期、幂等与容量

首次准备把 Host 提供且已验证的 Native scope 到期墙钟夹紧到最多300秒，随后仅使用单调时钟判断失效。重复请求不会延长租期。完整 mutation payload（排除传输 requestId）定义进程内幂等意图；同 operationId 改意图冲突。

原 turn operation 的保留键使用稳定 owner/tenant/Native process epoch/session/turn identity，不使用可续期的 authorization revision 或 expiry。到期/撤销后换 mutation ID、正常续期 scope 都不能重建原 turn。租期/调用只保留在该 worker 内存中，不建立安装或审批业务数据库，也不能从历史恢复可执行授权。

每个 generation 最多保留64个租期、1024个 mutation receipt。当前实现将128个 call 上限保守地用于全部 retained call（包括已消费/拒绝的 tombstone），并非只数活动 pending；不驱逐旧记录后重复接纳。不足时明确 `capacity_exceeded`，必须通过正常结束该 generation 开始新的 owner。这是当前候选容量限制，不能宣称无限长会话会自动释放容量。冻结原始参数在消费/拒绝/取消/到期/撤销后立即释放；非秘密 identity/decision receipt 保留以解释重放。

## 首次会话装配

新会话尚无 Runtime Thread 时，prepare 的 `context.nativeThreadId` 必须显式为 null，不能填造 ID，也不需要额外模型预热轮。Host 用已返回的Gateway地址一次性完成真实 `thread/start` 配置，再依据真实 `turn/start` response或通知，通过 `bind_turn` 同时冻结实际 thread/turn ID。它们保存在 Lease 的 `boundNativeThreadId` / `nativeTurnId`，原 CapabilityBinding 和 snapshot 不变。已有会话的非null thread约束必须一致，同一实际thread最多一个live bound lease。

## 调用与批准

只有真实 `tools/call` 才能登记 callRef。`pending_call` 只查询已有记录，`decide_call` 只决定已存在且完全匹配的调用。Gateway 至多等5秒让 owner 完成真实原生 turn 绑定，随后验证固定 Runtime 的 `_meta.threadId` 和 `_meta["x-codex-turn-metadata"].turn_id` 与 owner 绑定一致；metadata 本身从不创建绑定。

参数先按已资格化 schema 检查，再冻结解析值、由 worker 计算 `worker-json-v1` 摘要。Host 使用该摘要做关联而不跨语言重编码。批准元数据从同源 `ElicitationMetadata` 派生，只含 `yijieMarketCallRef` 和固定 kind。Host 必须经独占管道查询实际记录、展示安全 review，再把自身真实批准生命周期的决定传给 worker；单凭 UUID 语法不是产品批准。worker 同时要求 owner `approve_once` 和原生 elicitation `accept`，最多消费一次。已消费表示准入已用，不意味着真实外部写入成功。

当前唯一执行实现来自独立 `market-broker-qualification` 构建：固定安装 ref `11111111-1111-4111-8111-111111111111` / revision1 / generation1，服务 `market-qualification`，只读工具 `lookup`，输入 `{query: string}`（1..256字符），返回进程内公开合成值。它不冒充任意51个产品服务，不读取 Keyring，不使用真实模型、供应商账号、Google 包或外网。

## 构建与正常退出

- `make worker-build`：标准产品二进制，不启用 qualification feature；内容寻址保存且拒绝覆盖不同字节，当前manifest v2为 `qualificationOnly:false`、`externalCallsEnabled:true`、固定 `providerProfile:tushare-oauth-v1`；true仅指受管OAuth/metadata能力，金融工具仍关闭。
- `make worker-broker-qualification`：普通单元验证后，单独 feature / binary / 内容寻址目录 / manifest；`qualificationOnly:true` 并保存源摘要。不能作为 Host 产品 owner 的 artifact。
- `make worker-test` / `make worker-lint` / `make contract-check`：旧状态与新控制边界、源同步和静态检查。

`shutdown` 或 stdin EOF 先停止准入、撤销租期并唤醒等待，再正常关闭官方 Gateway。shutdown 回执仅表示停止准入，不声称外部操作已撤回或完成。Host 关闭 stdin 并等待普通 exit；未确认退出必须保持 `STOP_PENDING`。无 process signal/kill、伪装可执行文件、权限破坏或攻击注入 fixture。

旧 Connectors Go status owner 接受历史无 `qualificationOnly` 的产品 manifest，也接受新明确 false；始终拒绝 true。新 Host Broker owner 要求新 manifest 明确 false，不能把 qualification binary 接入产品。

## 当前未接通

Native 权威 scope → Host 版本化提交、真实安装 generation/readiness、产品 runtime 配置装配、通用批准/结果投影、OAuth/Keyring 与逐项 provider qualification 均仍需接线。控制边界和合成 MCP 成功不等于产品入口已开放或51项服务已接通。固定实际 Runtime 的合成资格与初次装配依赖处理记录在元仓 FEAT-157 evidence；该资格不能替代产品授权链。

## 本轮验证与审计修复

[验证清单与源码摘要](market-broker-verification.json)记录19项Rust测试（16项Broker、3项原基础）、产品/qualification的Clippy与格式检查、实际canonical status worker的Go race/正常EOF回归、来源同步和标准构建。产品制品SHA为`f36db7e719175d4a60b7f0133776704cfe5d30faaf930f8981f75dff4322f415`；独立qualification为`68d4e5c98d3ce46fc9703133f20bb8c2dc3e697ef6792219ec182f2893fb4fea`。

固定实际Runtime的最终普通本地合成资格见元仓 `evidence/broker-qualification/run-06`：新thread首次批准一次、同thread下一轮拒绝不准入、40个控制帧通过同源AJV、Runtime和worker均正常EOF exit0。此前失败run均保留：首次空thread不能cold-resume的问题已经通过required-null契约处理；准确只读annotations只适用于固定合成lookup；官方rmcp在`service.rs:1168`把请求meta移到`RequestContext.meta`，Gateway已按真实库派发方式读取。最终Clippy仅将取消检查的nested-if等价改为let-chain，重建得到与run-06相同的两份二进制字节和SHA，没有用旧制品冒充新源码验证。

独立审查后还补了三项正常取消验证：预取消不登记、等待owner bind期间取消后晚bind不登记、原生accept已就绪但取消也已到达时不消费批准。等待阶段和elicitation阶段均响应`context.ct`，消费前在Broker锁内再检查取消。拒绝/取消/消费/撤销/到期释放冻结参数，终态观察TTL归零。

固定receipt容量达到上限时，`revoke`/`shutdown`不会越过容量再入账。Host必须在串行owner gate内关闭stdin，正常退役整个generation；保留原`capacity_exceeded`结果，并通过`Close`确认exit或保留`STOP_PENDING`。不返回伪成功，不强杀，也不重新启动已关闭owner。
