# AGENTS.md

## 适用范围

本文件适用于 `yijie-connectors` 整个仓库。平台或第三方应用目录中若出现更具体的 `AGENTS.md`，修改对应目录时以更具体的规则为准。

## 仓库职责与当前状态

`yijie-connectors` 是外部平台 API、第三方应用和 MCP 工具的执行边界，负责平台客户端、OAuth/token vault 适配、权限 scope、限流、重试、熔断、幂等、高风险操作 guard 和外部调用审计。

当前仓库只有最小 HTTP 骨架：

- 只有 `cmd/connector-gateway` 和 `internal/app` 有实际实现；
- `/healthz`、`/readyz` 和 `/v1/status` 返回静态状态；
- status 中列出的 Amazon、Temu、Shopee 和 TikTok Shop 只是规划平台，不代表已经接通；
- MCP server、平台 client、OAuth、token vault、webhook、worker、限流、幂等和审计均未实现；
- `make generate` 当前是占位命令。

当前健康检查成功只说明进程可响应，不能被描述为平台连接、凭据或写操作已经验证。

## 仓库边界

- 不承载租户、任务、审批等业务主状态；
- 不编写 Skill prompt，不实现 RAG 或维护 Codex Runtime；
- 不替 `yijie-api` 决定身份、RBAC、审批主状态或业务规则；
- 不把原始平台 token 暴露给 Desktop、Admin Web、Runtime、Skills、日志或响应；
- 不绕过审批策略执行高风险写操作；
- 不在没有官方 API 依据、授权 scope 和沙箱验证时猜测平台行为。

连接器可以保存必要的凭据引用和可靠性运行状态，但 token vault、幂等、限流、webhook 去重等持久化方案必须先确认，不能顺手建立第二套业务主数据库。

## 代码组织

- `cmd/connector-gateway/`：统一入口和健康检查，保持轻量；
- `cmd/mcp-*/`：按平台或应用隔离的 MCP 服务入口，目前均为占位；
- `cmd/sync-worker/`、`cmd/webhook-server/`：异步同步与 webhook 占位；
- `internal/platforms/`：Amazon、Temu、Shopee、TikTok Shop 等官方平台适配；
- `internal/third_party/`：邮箱、物流、飞书、钉钉等第三方应用适配；
- `internal/auth/`：OAuth 和 token vault 端口及实现；
- `internal/mcp/`：MCP tool 暴露与 schema 适配；
- `internal/reliability/`：限流、重试、熔断和幂等机制；
- `internal/safety/`：风险分级、审批上下文校验和参数约束；
- `internal/audit/`：脱敏外部调用审计；
- `api/`：局部实现材料，稳定跨仓库契约以 `yijie-contracts` 为源。

平台模块不得互相导入私有实现。共享抽象只承载确实一致的 transport、错误分类或可靠性行为，不强行抹平平台差异。

## Contract First 与平台接入

- 每个任务先标记 `contract-impact = none | additive | semantic | breaking`；分类覆盖跨进程、跨仓、跨版本及持久化/重放边界，tool scope、风险、审批、错误、幂等和重试变化即使形状不变也属于契约影响，`none` 必须说明理由；
- 按 `breaking > semantic > additive > none` 的最高风险唯一选择；任一受支持交互可能失效即 breaking，不确定时不能假定 additive/none；
- 第三方平台 API/webhook 以当前官方规范为外部权威；任何易界归一化 MCP tool、OpenAPI 和公共消息 schema 仍先在 `yijie-contracts` 定义、评审、执行兼容检查并形成不可变引用；
- 每个工具必须声明名称、输入输出 schema、平台、权限 scope、风险等级、幂等语义、审计字段和错误模型；
- 不手写与生成契约重复的 DTO，不直接编辑生成 client 或 schema；
- 本仓固定精确 contract version、完整 commit 和可用时的 digest/generator 版本后，实现才可合并或启用；`make generate` 仍为占位时不能宣称已完成契约消费门禁；
- 接入平台前必须依据当前官方文档确认 API 版本、环境、认证流程、scope、配额、错误码和 webhook 规则；
- 官方 SDK 与自建 client 的选择、SDK 版本和许可证必须明确确认；
- sandbox、mock 和 production 配置严格隔离，默认本地开发不得访问生产环境。

dirty/floating sibling 只能用于本地候选验证，不能作为发布来源。兄弟元仓存在时同时
遵循 `../yijie/docs/dev/contract-first.md`。

## Token 与租户安全

- 真实 token 只能通过批准的 token vault 获取，仓库当前尚未配置 vault；
- 代码、`.env.example`、fixtures、测试、日志和错误不得包含真实 token、client secret、cookie、授权码或签名密钥；
- 服务间只传 credential reference 和最小租户/店铺上下文，不传原始 token；
- refresh 必须处理并发、轮换、吊销、过期和失败回滚，不能让旧 token 覆盖新 token；
- 所有缓存、幂等键、限流键和审计字段显式包含租户及店铺边界；
- URL、header、body 和平台错误在记录前脱敏，禁止记录完整订单、买家消息或 PII。

## 写操作、审批与审计

- 工具按只读、低风险写、高风险写和不可逆操作分级，未知工具默认高风险并拒绝；
- 高风险执行必须校验来自权威服务的批准上下文，绑定 tenant、task、tool、参数摘要、影响对象和有效期；
- 审批凭证格式、签名和校验方式尚未确定时停止实现，不自行发明；
- 写操作必须支持幂等键，重复请求返回一致结果，不重复发布、改价、调预算或发送消息；
- 审计同时记录请求意图、批准依据、外部 request ID、脱敏结果、状态和错误分类；
- 平台调用失败、超时或结果不确定时不得伪造成功，应返回可恢复状态并避免盲目重试。

## 限流、重试与 Webhook

- 限流至少考虑平台、租户、店铺、应用、工具和 API 配额，尊重平台返回的限流信息；
- 仅对明确可重试且幂等的请求使用有上限的指数退避和 jitter；
- 认证、权限、校验和业务拒绝默认不重试，写请求没有幂等保障时不得自动重试；
- 设置连接、请求和空闲超时，错误需区分限流、暂时故障、认证失败、业务拒绝和未知结果；
- webhook 必须验证签名、时间窗口和来源，并进行重放防护、事件去重和乱序处理；
- webhook 或异步任务所需的 durable state、队列和数据库必须在技术方案确认后再引入。

## 必须先确认的决策

- 目标平台、站点、官方 API 版本、sandbox/production endpoint 和使用条款；
- OAuth 应用、授权 scope、token vault、密钥管理和租户/店铺绑定模型；
- 工具 schema、风险等级、审批凭证和审计保留策略；
- 幂等、限流、重试、熔断、队列、webhook 去重和持久化方案；
- 官方 SDK、新 Go 依赖、MCP transport 和跨仓库发布顺序；
- 任何真实账户、付费 API、生产凭据或可能产生平台写入的测试。

## 开发与验证

```bash
make lint     # gofmt 检查和 go vet
make test     # race 单元测试和覆盖率
make generate # 当前为占位，不能视为契约生成完成
make dev      # 启动 connector-gateway 骨架
```

- client 测试使用官方 sandbox、录制后彻底脱敏的 fixtures 或可控 fake，不访问生产账户；
- 写工具测试覆盖幂等重复、审批缺失/过期/参数不匹配、跨租户和未知结果；
- 可靠性测试覆盖 429、`Retry-After`、超时、连接中断、部分成功和重试上限；
- webhook 测试覆盖签名失败、过期、重复、乱序和恶意 payload；
- 真实平台集成测试必须由用户明确批准，并标识成本和副作用。

## 完成标准

- 工具契约、scope、风险、幂等、限流、错误和审计语义完整；
- 当 `contract-impact != none` 时按权威源路由：第三方原始协议固定官方 API/SDK/schema 版本并验证 adapter/webhook；易界归一化公共表面固定 contracts 引用、consumer pin 和 conformance；私有 token/state 变化走所属存储 migration/恢复验证；不适用的 contracts 字段写 `N/A + 理由`；所有路径记录部署/回滚顺序；`none` 只需分类理由；
- token 始终留在连接器安全边界，日志和测试数据完成脱敏；
- 高风险写操作缺少有效审批时默认拒绝，重复请求不会重复执行；
- 平台差异有隔离实现，失败和未知结果没有被包装成成功；
- `make lint`、`make test` 及相关 sandbox/集成测试通过；
- 尚未接通的 vault、MCP、平台、webhook 或生产验证被明确说明。
