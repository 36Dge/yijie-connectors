旧 Broker 资格入口修复：2026-11-16 审计发现，2026-11-17 完成验证。

`contract-impact=none`：修复辅助程序的模块装配，并保持它已有的无外部调用约束。
产品模式的协议、批准条件、HTTP 许可和持久化格式不变，不修改 Runtime、契约或依赖版本。

原错误来自 `worker/qualification/broker.rs`：它复用 `gateway.rs`，但没有声明后者新增依赖的
`builtin_web_gateway`、`builtin_web_executor` 和 `builtin_web_generated`。现在通过原有
`#[path]` 方式直接引用三个产品源模块，没有复制实现、伪造执行程序或重建 MCP 客户端。
固定的 `codex-config`、`codex-exec-server`、`codex-rmcp-client` 及官方 rmcp transport
继续由原 `worker/source.lock.json` 和 `Cargo.lock` 约束；固定来源检查通过。

仅补模块会把产品 Web 装配带入资格程序。因此共享 Gateway 仍使用原 `ProviderMode`：
Product 装配原 Web 服务及私有控制；Qualification 不装配 `/builtin-web`，也不处理
Web 安装／清理控制。它继续只接受原 `--broker-control-stdio` 入口，保留进程内合成
Broker 能力，初始化仍返回 `externalCallsEnabled=false`。没有新增授权开关或执行状态机。
资格构建脚本补记三个共享模块及自身构建配方的摘要，未更改 manifest 格式。
独立验证目录使用精确的 `.local/qualification-entry-*/` 忽略规则，避免把编译缓存、
本机产物或保护清单纳入提交；既有本地材料和已跟踪文件的规则保持。

静态检查同时发现共享 Web 模块中原有的两处 Clippy 样式告警；仅将嵌套条件等价改为
let-chain、直接返回原 await 结果。原 epoch 比较、ActiveCall 生命周期及结果保持。
没有压制这些告警或降低断言。

本机验证使用独立 `.local/qualification-entry-fix-20261116-233530/` 目录中的标准 Cargo
产物，没有写入产品或旧资格程序的 `bin/**/current.json`，也没有启动当前易界 App。

| 验证 | 结果 |
| --- | --- |
| 启用 broker-qualification 后同时编译两个测试二进制（`--bins --no-run`） | 通过，原 E0433 消失 |
| 独立资格程序、默认产品程序的标准离线构建 | 均通过 |
| 两个目标的严格 Clippy（`--no-deps -- -D warnings`） | 通过 |
| 回执、单次许可、正常停止及共享 Web 控制／本地适配器用例 | 18 项通过；最后等价整理后复验相关 8 项通过 |
| 实际新二进制的资格／产品模式 × shutdown／EOF | 4 项通过，全部 exit 0，无进程终止信号 |
| 资格边界 | `/builtin-web` 返回 404；有效 Web 清理消息返回原平铺 `invalid_request` |
| 产品边界 | 原 rmcp 路由仍处理请求（无 Accept 的 GET 返回 406）；本地 Web 清理仍返回 applied |
| 固定 Codex 库来源、契约消费及内置 Web 生成物 | 通过 |
| Rust 格式、Shell 语法、Python AST、Git 差异检查 | 通过 |
| 当前应用、Runtime、worker、原数据根及旧资格产物保护 | 108,921 个文件前后内容、权限、属主及链接摘要一致 |

正常入口回归脚本为 `scripts/tests/broker-qualification-entry-test.py`。它只向新构建的本机
进程发送 initialize、只读路由检查、清理和 shutdown；不配置 provider、不传真实凭据、
不发送 tools/call。关闭只使用 stdin EOF 和等待；若超时则报告 STOP_PENDING 并保留
进程所有权，绝不升级为 terminate/kill。共享适配器单元用例仅访问本机合成 HTTP 服务。

可复跑的正常检查如下，先选择一个独立的项目验证目录：

```sh
export CARGO_TARGET_DIR="$PWD/.local/qualification-entry-check/target"
export CARGO_BUILD_BUILD_DIR="$PWD/.local/qualification-entry-check/build-cache"
python3 -B scripts/check-worker-source.py
make contract-check
node ../yijie-contracts/scripts/generate-builtin-web.mjs --consumer=connectors --check
cargo test --offline --locked --manifest-path worker/Cargo.toml --features broker-qualification --bins --no-run
cargo build --offline --locked --manifest-path worker/Cargo.toml --features broker-qualification --bin market-broker-qualification
cargo build --offline --locked --manifest-path worker/Cargo.toml --bin yijie-mcp-worker
cargo clippy --offline --locked --manifest-path worker/Cargo.toml --features broker-qualification --bins --no-deps -- -D warnings
python3 -B scripts/tests/broker-qualification-entry-test.py --target-dir "$CARGO_TARGET_DIR"
```

原始失败和修复后日志均保留在本机验证目录。第一次入口脚本误按 JSON-RPC 包装读取
Broker 错误；已依据既有生成类型修正为精确的平铺 Error 响应，旧失败日志没有改写。
这不是新增或放宽产品错误协议。

最终独立资格二进制 SHA-256 为
`5df9cc58329e2f448966dbcc53702ac74e70d90beccca2cdec857fee40c2e5a2`；
独立默认产品二进制为
`bb86b907d95791279c1d15566a9a8033f634fc2e7d2274275742efda96ae88ca`。
两者只保留在验证目录。测试进程均正常退出后，已删除本次约 5.25 GiB 编译缓存，
并复核保留的两个二进制摘要；未清理其他仓库或原有数据。

当前 App、固定 Runtime、产品 worker、旧资格产物及原 Native/Host/Runtime 数据根
执行前后按文件摘要核对。全量历史测试、真实供应商／付费模型、故障或攻击注入、
可执行文件替换、权限破坏均未执行；本修复不宣称业务全流程或生产发布验收。
