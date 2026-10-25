# FEAT-164 固定库与 Runtime wrapper 的兼容

contract-impact=semantic（私有构建来源）；公共协议、Provider、凭据、工具和审批未变化。

Worker继续固定 `7fd463bcef07f37b0211acd9f62b9f93ea0a4b12:codex-rs`。FEAT-164 Runtime wrapper `074417e5e2f5f55704ea5e1777e7a22d514076b4`只增加构建补丁与元数据，两者库tree均为`755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41`。

source.lock仅登记这一个精确完整wrapper commit。构建仍要求HEAD属于明确集合、每个兼容commit的库tree与原pin完全一致、实际工作树相对pin零diff，以及原Rust/依赖/schema锁检查全部通过。不是接受任意新HEAD，也没有升级库或复制/修改Runtime程序。Worker manifest的codexSource仍是原库commit；由标准构建产生独立content-addressed产物，不覆盖既有二进制。未调用真实供应商。
