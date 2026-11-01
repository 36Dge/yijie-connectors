# FEAT-164 固定库与 Runtime wrapper 的兼容

contract-impact=semantic（私有构建来源）；公共协议、Provider、凭据、工具和审批未变化。

Worker继续固定 `7fd463bcef07f37b0211acd9f62b9f93ea0a4b12:codex-rs`。FEAT-164 Runtime wrapper `074417e5e2f5f55704ea5e1777e7a22d514076b4`只增加构建补丁与元数据，两者库tree均为`755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41`。

source.lock仅登记这一个精确完整wrapper commit。构建仍要求HEAD属于明确集合、每个兼容commit的库tree与原pin完全一致、实际工作树相对pin零diff，以及原Rust/依赖/schema锁检查全部通过。不是接受任意新HEAD，也没有升级库或复制/修改Runtime程序。Worker manifest的codexSource仍是原库commit；由标准构建产生独立content-addressed产物，不覆盖既有二进制。未调用真实供应商。

## 2026-11-01 执行前约束 Runtime

本次增量 `contract-impact = none`：连接器协议、工具语义、依赖、实际库代码和权限均未改变，仅登记已审查的构建 wrapper 来源。完整任务在 Runtime/Host 中按 semantic 管理。

新增精确 wrapper `9ebafd71de2dea4c2f016bb38307b0a19a722384`。它新增可重放执行前拦截补丁与 canonical 构建脚本；实际补丁应用在专属构建工作区，未改动 worker 依赖的 `codex-rs`。该提交与原库 pin 的 tree 同为 `755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41`，实际工作树零 diff、无额外未跟踪源码。复用现有 `compatibleRuntimeWrapperCommits` 机制，保留所有来源/依赖校验，未修改校验器或放宽任意 HEAD。日常入口继续使用原 Cargo 缓存和内容寻址产物。
