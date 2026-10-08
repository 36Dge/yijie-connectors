# Tushare 受管 OAuth 候选

2026-10-08；本地未发布，真实 OAuth/Keyring/metadata 已验证，daily 业务调用尚未验收。目录保留51项，本轮只推进 Tushare。本文接续 Broker 第一阶段，旧 run-06 合成 Runtime 证据和旧制品 SHA 保持历史原义，不能当作本轮真实服务验收。

## 官方事实

官方[接入文档](https://tushare.pro/document/1?doc_id=463)给出 Token 路径接入。但公开无凭据 GET `https://api.tushare.pro/mcp/` 实际返回401及 OAuth protected-resource challenge，其 metadata 指向 `https://tushare.pro`。该 issuer 的公开 authorization-server metadata 声明 `/oauth/authorize`、`/oauth/token`、`/oauth/register`、public client、S256、authorization_code/refresh_token、`mcp:tools`。因此易界使用普通基路径的 OAuth，不把 Token 写入 URL，也不照搬参考应用的 auth 字段。5次公开 GET 原始安全观察和摘要见 [public-metadata-2026-10-07.json](public-metadata-2026-10-07.json)。

首次真实UI授权已尝试：未观察到新Tushare浏览器页，原操作最终为temporarily_unavailable，Host/worker仍正常存活。旧构建未保存阶段计数，实际DCR/token交换/Keyring访问及额外metadata GET次数均未知，不能继续记0。此前5次公开GET仅是历史证据。金融tools/call为0（代码路径封闭）。不使用聊天/文件传递token，不为诊断另读账号凭据。

## 边界与复用

同一个 Host-owned `--broker-control-stdio` worker 同时管理 OAuth 与 Gateway；provider 动作在 Broker initialize 之前可用，不为授权创建 Runtime generation 或模型轮。Provider 5动作使用 Contracts `market-provider/1` 同源 DTO，输出只有状态、opaque credentialRef 和临时浏览器URL；Native/Host 不接 token。

OAuth discovery/DCR/PKCE/code exchange 复用官方 rmcp1.8 `AuthorizationManager` / `AuthorizationSession`。Connectors 只拥有 callback server 的正常退出、cancel epoch 与脱敏。没有使用 Codex 不可取消且可能晚保存的登录 handle；凭据保存、删除、状态与 authenticated MCP metadata client 仍复用固定 Codex 公开 API。Runtime 源码和已有二进制未更改。

所有凭据操作固定 Keyring backend，拒绝 File/Auto fallback。独立 `<HostHome>/market-worker/library-home` 用于 Codex 库私有寻址，由 assembly 设置子进程 `YIJIE_MARKET_WORKER_HOME` 与相同 `CODEX_HOME`，避免 delete API 触及用户原 Codex 凭据文件。alias 哈希含 owner/tenant/installation/generation/opaque cref，排除瞬时 Runtime 与 Native epoch。Keyring状态错误显式阻断；额外 HTTP guard 阻止固定 RmcpClient 在 Keyring错误后匿名 MCP POST 的兼容回退。

HTTP目的地固定 Tushare resource/issuer/OAuth端点，重定向关闭，超时与响应有界。元数据 client 只接受 initialize、notifications/initialized、tools/list、ping、notifications/cancelled。新 daily profile 的独立受管 client 仍默认拒绝 tools/call，仅本次 Broker 双批准后给出的精确参数一次许可可穿过实际 HTTP 边界；401/session recovery 等 SDK 重发不能再次发送业务请求。授权自动连接检查继续 `qualification:not_qualified`、`executionAvailable:false`，不创建 Enable。

Native SQL33 只存非secret payload、脱敏 observation 与一次性 browserOpened；不存 authorizationURL。授权URL只经固定域/路径/PKCE/scope/resource/loopback callback 检查后交给现有系统浏览器 API。管理动作持久幂等，等待后重新检查 Native权限。Disable/Uninstall在 Native admission gate 内先撤销 Host authority（失败则正常关闭管道），再禁用和推进 revision；worker 撤销/取消受管 client，等待正常调用/refresh owner 完成才删除自己 Keyring alias，确认后清 cref 并推进 generation。失败保留清理状态，无假成功、强杀或晚回调重新启用。

## 下一步真实只读资格

OAuth成功后自动在同一provider auth operation内复用metadata probe，仅 initialize/tools/list，再正常shutdown；不创建Enable意图。固定私有目录 `qualification/tushare/<operationId>.json` 留存工具数量、schema/catalog摘要、精确 daily 候选字段结构；不保存上游描述/示例/默认值/自由文本或金融数据。该文件不是资格决定。

候选最小实际读取是[日线 daily](https://tushare.pro/document/2?doc_id=27)：`000001.SZ`、`20260105`，单标的单日1次，最多预期1行。文档列120积分访问门槛；[积分说明](https://tushare.pro/document/1?doc_id=108)描述访问权限，不应虚构逐调用积分扣费。实际账号权益尚未确认；官方 MCP 元数据已读取，daily 完整 inputSchema 摘要为 `ec10409543d1ae690de2b5da5893a0e69387cdd5a398f976fb04d187fc632ae1`。首次样本仍未调用，也没有验证其余50项服务。该股票/日期仅是验收样本，不是产品唯一可查询数据。

## 本地验证

标准 `make worker-test worker-lint worker-build` 通过；当前默认37项测试通过（含授权后自动metadata与daily候选），独立 broker-qualification feature 回归数量见verification.json；Go旧status manifest reader/app通过。普通 OAuth 合成 provider 使用本地127监听与内存 vault，验证 DCR/PKCE/callback、正常取消、owner关闭不保存；不用 Keyring或真实账号。真实执行 guard 和无供应商文本资格投影另有纯逻辑测试。

Native新增3项事务测试验证授权不启用、取消不复活、清理推进generation/旧回调失效。标准产品 manifest升级schemaVersion2，`externalCallsEnabled:true`只表示受管OAuth/metadata网络能力，最初 `providerProfile:tushare-oauth-v1`、`qualificationOnly:false`；不代表金融调用已开放。旧status-only运行模式继续无网络，reader支持旧v1制品。当前产品artifact SHA与源摘要见 [verification.json](verification.json)。

独立审查补强：AuthRun 与 callback server 超时均保留可再次 join 的实际任务；provider actor在正常清理完成前不丢弃回调owner。worker收到EOF后立即撤准入，再持续正常等待实际清理；Host有界等待到期保持STOP_PENDING，未确认退出不假装成功。旧978f内容地址制品保留，当前新制品见verification.json。

真实UI安装后发现授权入口被“工具未验证”状态遮住，因此补了独立同源 `CatalogEntry.authorizationAvailable`（缺省false）。仅Native exact market环境将已实现Tushare OAuth adapter投影true，其他服务显式false；静态51目录availability不变。这只表示可开始授权，不能表示有token、Keyring可用、网络/模型ready或金融工具资格。

2026-10-08补齐授权后的自动连接检查：OAuth提交后保持operation starting、authorization authorized、connection connecting，直到只读metadata完成再给终态。metadata错误、正常取消或原deadline到期保留已完成授权事实，返回安全连接错误；不重新DCR、不延长attempt、不调用金融tools/call。普通状态测试同时覆盖已到期时不创建client（避免timer首次调度先触发Keyring-readiness）。Native和Frontend不需要新action。


首次真实授权失败后的诊断修复（2026-10-08）：已有终态只能确认安全错误temporarily_unavailable，不能反推哪一步失败，也不能把约5秒UI延迟当成管道超时。新worker在固定library-home/diagnostics/provider/<operationId>.jsonl记录白名单阶段、事件、HTTP状态及内部Failure枚举；不记录URL、scope、alias、request/response body、callback code或token。每文件最多128条且每条512字节；创建新文件时目录最多128项/8MiB，满时只停写新诊断，保留历史证据、业务行为不变。采用create_new0600，不覆盖旧文件。stream.recv失败单独记录response_unavailable。正常重试前不扩大timeout、不循环重Auth。

Native精确typed rejection保留安全code。仅本次事务确实新建的操作、首次尚无观察且明确未执行的拒绝可落Failed；新建事实不是由Pending/NULL猜出。已有Pending/NULL（包括旧构建可能发出但未保存响应的操作）及Unknown始终保留原operation/绑定，直到真正ProviderStatus确认终态，不因后续拒绝清掉未知执行。Native安全日志只含固定method和控制错误枚举，不含请求或回执正文。


真实浏览器交接证据（6d4构建）：操作 `7c8e22be-e0b8-48a8-93cf-63d5742b8cd4` 的安全诊断记录3次OAuth discovery公开GET（401/200/200）、1次DCR请求并返回201、0次token_exchange启动；Keyring preflight成功。Chrome中已确认正确的“Yijie Tushare”官方授权页，Native开页链实际成功；此前“操作结果待确认”是pending文案造成的误解。该操作随后因等待超过原Native scope期限而expired，并正常完成callback cleanup。快照见 [原操作诊断](evidence/7c8e22be-e0b8-48a8-93cf-63d5742b8cd4.jsonl)。

随后从Native正常发起新操作 `d50c205a-6351-418d-9d4b-99ce3fd4c44c`，新的官方页已打开并交由用户同意；本文保存的是 `2026-10-07T16:47:48.080112+00:00` 时的安全诊断快照，当时状态为 `expired_and_normal_cleanup_confirmed`。见 [新操作观测快照](evidence/d50c205a-6351-418d-9d4b-99ce3fd4c44c.jsonl)，不能把等待回调当成已经授权。没有由诊断代理重发Auth或读取凭证。

实际本轮HostHome由scheduled assembly覆盖为 `/Users/jack/Library/Application Support/com.yijie.ai/demo-fast-model-candidate-v1/host`，诊断在其 `market-worker/library-home/diagnostics/provider/`。启动脚本 `.local/feat156-candidate/host-home` 只是先前默认值，不能作为本轮实际路径。

上述3次GET是被诊断明确记录的OAuth flow discovery请求，不包含Codex auth-status/Keyring前置检查中可能执行的额外discovery GET。首次无诊断UI尝试的DCR/token/Keyring/额外GET总数仍未知，不能用新快照反填旧计数或宣称全历史总数精确。


真实OAuth/Keyring/metadata现已通过：用户后续操作`62a7c0e8-1e5a-436c-a774-808baa0fe783`具有token响应200、Keyring commit succeeded、initialize/tools_list/artifact succeeded的安全证据。实际工具254个，精确daily唯一，原inputSchema摘要ec10409543d1ae690de2b5da5893a0e69387cdd5a398f976fb04d187fc632ae1，安全schema快照已保留。之前用户自行尝试feb252前缀操作的token transport_unavailable只说明未获得成功响应、未观察到本地Keyring commit；不能断言服务端没有执行。全历史第一次无诊断尝试计数仍未知。

当前通过 fresh Probe 资格的窄能力为正常只读`tushare_daily`→上游daily，两个必填单标的A股代码/单个合法交易日，额外字段关闭、输出最多1行公开行情。000001.SZ/20260105是首个验收样本，不是写死的产品唯一数据。归一化输入由Contracts同源定义，完整上游schema摘要锁定，OAuth成功不自动启用。当前尚未发金融查询或模型请求，业务资格及D4仍未完成。


## daily profile 候选（2026-10-08）

contract-impact=semantic。Contracts market-provider 同源策略定义 `tushare-daily-v1` / `tushare_daily`：两项必填字符串 `ts_code`、`trade_date`；单个6位代码加 SH/SZ/BJ 后缀和真实公历 YYYYMMDD。没有批量、日期范围、fields 或任意额外参数。Gateway 对该固定只读工具声明准确 annotations，仍必须逐次 ask；其余253个上游工具不暴露。

显式启用的 fresh Probe 用同一 worker 的 Keyring-only client 重新 initialize/tools/list；精确唯一 daily 的完整 schema SHA 必须匹配，同时递归检查两个转发属性的所有 anyOf/oneOf 分支、额外 required 和未审核约束。服务器描述/annotations 不参与授权。通过后保留绑定 Host、Native epoch、authorization revision、安装 revision/generation 和 opaque cref 的 client 能力；Native 的显式 desiredEnabled 与当前选集/grant 仍独立必需。已完成 client 能力在同 owner 内保留，每次状态观察与新轮都要求新鲜 Native scope；原 OAuth attempt、turn 和 call 的单调时限绝不续期。正常关闭、撤权、配置变更或忘记立刻撤销能力。

Broker 在真实 tools/call 到达后冻结生成 DTO 的两个参数、argsDigest 和股票/日期安全摘要。Host approve_once 与 Runtime 原生 elicitation Accept 都匹配后，callRef 消费一次；HTTP guard 在真实发送前原子取走许可。相同 SDK 请求的自动重连/重试不会得到第二许可。取消、原 lease 到期或 backend 撤销在发送前再次检查；发送后的不确定结果不假装撤回、不重发。既有 rmcp/Codex 协议客户端和 OAuth refresh/writeback 保持原 owner，forget 等它正常完成再删除凭据。

结果只接受有界结构化 JSON records 或 Tushare fields/items 表：最多一行、股票与日期须一致；仅返回两项 identity 和 open/high/low/close/pre_close/change/pct_chg/vol/amount 的有限数值（close 必需，其余可选），未知字段、供应商文本、描述、链接、metadata 不透传。空行是成功的空观察；未知输出 envelope 安全失败且不重试，尚无真实金融结果可称验收通过。

新的 canonical binary 仅发布 `providerProfile:tushare-daily-v1`。该 manifest 描述代码能力，不是资格或 grant；给旧 OAuth binary 改 profile 不能创造 daily adapter。旧内容地址产物保留。测试使用官方 rmcp in-process 普通 server 和固定 Codex client，没有网络、真实凭据或模型；覆盖一次批准消费、正常取消/EOF、scope 续期不复活旧 turn、完整参数结构、结果投影。真实 fresh Probe 已于本轮正式包通过；首条金融调用与付费模型轮仍未执行，预算待用户确认，并须逐次批准。

rmcp 把 token 请求网络/服务错误归到 typed TokenExchangeFailed；现在映为暂不可用而非用户取消。callback/state/issuer 校验失败仍是授权拒绝；不解析或输出错误字符串，不自动重发 token exchange。首次历史未诊断尝试计数保持 unknown。


## 正式包 fresh Probe 观察（2026-10-08）

显式启用操作 `d2ed9a82-70da-4484-86cb-b5aa7fd9d74a` 在 canonical worker `0aacce914f889a822fcc9e3774af273d77eef03b66f1c782309b2d259c371cfa` 上正常完成：initialize（含 Keyring readiness/重读与 client setup）38.964 秒，tools/list 343 毫秒，artifact 15 毫秒，总计 **39.322 秒**。安全[阶段诊断](</Users/jack/Library/Application Support/com.yijie.ai/demo-fast-model-candidate-v1/host/market-worker/library-home/diagnostics/provider/d2ed9a82-70da-4484-86cb-b5aa7fd9d74a.jsonl>)和[元数据 artifact](</Users/jack/Library/Application Support/com.yijie.ai/demo-fast-model-candidate-v1/host/market-worker/library-home/qualification/tushare/d2ed9a82-70da-4484-86cb-b5aa7fd9d74a.json>)记录 initialize、tools/list、artifact 均 succeeded；254 个工具中唯一 daily 的完整摘要仍为 `ec10409543d1ae690de2b5da5893a0e69387cdd5a398f976fb04d187fc632ae1`，`dailyPolicyReviewPassed=true`。root 另从 UI 观察到 Tushare 已启用并可选择。

artifact 仅是元数据证据，本身仍保留 `qualification:not_qualified`、`executionAvailable:false`，不构成执行授权。当前能力由同 worker 的 Probe 结果及保留的 client 映射确认，仍需 Native 显式启用、当前 scope/选集/grant 和本次双批准。Probe 代码先要求固定 Codex Keyring 的 usable OAuth 状态，再以相同 owned alias 建立 client；成功初始化通过了禁止匿名 POST 的实际 HTTP guard，没有启动 AuthRun/DCR。审计未读取凭据；阶段内部的 Keyring、网络和 refresh 耗时没有细分，不能由39.322秒推测具体次数。

金融 `tools/call=0`、付费模型轮 `paidModelTurns=0`。首条金融样本和完整 D4 仍未执行，预算待用户确认；fresh Probe 通过不能表述为金融数据或付费模型验收通过。历史首次未诊断 OAuth 尝试的总计数仍保持 unknown。
