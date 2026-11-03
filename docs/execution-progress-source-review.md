# Execution-progress Runtime wrapper 来源核对

2026-11-03，本次执行链路修复的正常启动遇到既有 Codex library 来源门禁。复用 `compatibleRuntimeWrapperCommits` 的精确审核入口，未修改或绕过 `check-worker-source.py`。

新增允许的 wrapper commit：`55b4e51049d6f4fc0af4b35f12832357f5e4ea06`。它与固定库源 `7fd463bcef07f37b0211acd9f62b9f93ea0a4b12` 的 `codex-rs` Git tree 都是 `755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41`。工作树库源码也由原检查器再次比较。原 `sourceCommit`、依赖锁、Rust 版本、MCP/权限/凭据/工具执行逻辑均保持。

`contract-impact = additive` 仅指部署来源准入新增这一已验证的 wrapper，旧来源仍有效；公共 wire 和库 ABI 无变化。整个执行链路修复仍按其 semantic 范围验证。标准 worker 构建与内容寻址发布机制保持，旧二进制不覆盖；没有进行真实平台业务或模型调用。
