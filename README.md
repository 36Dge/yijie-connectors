# yijie-connectors

外部平台 API 和三方应用的工具执行层。

## 仓库职责

- Amazon、Temu、Shopee、TikTok Shop 等平台 API；
- 物流、邮箱、飞书、钉钉、LinkedIn 等三方应用；
- MCP 工具服务；
- OAuth、token vault、token refresh；
- 限流、重试、熔断、幂等；
- 高风险操作 guard；
- 外部 API 审计日志。

## 当前状态

当前对外 Go HTTP 服务保持最小骨架，不接真实平台 API、不保存真实 token。

FEAT-157 本地候选已有51项安全产品目录、固定Codex库依赖的状态查询、Host直接拥有的私有Broker控制模式及官方rmcp loopback Gateway。所有真实服务仍未资格化，产品非空选择执行保持关闭；未暴露公共HTTP管理API。来源、构建和历史基础见[连接器基础说明](docs/market-connectors-foundation.md)，新的瞬时准入边界见[Broker控制说明](docs/market-broker-control.md)。

## 本地开发

```bash
make dev
curl http://localhost:18081/healthz
```

FEAT-157 后续本地候选增加 Host 直接拥有的私有 Broker 模式与官方 rmcp loopback Gateway；真实 provider 全部未资格化，产品非空选集仍拒绝。独立合成资格二进制不能充当产品 worker，详见 [Broker 控制说明](docs/market-broker-control.md)。

Tushare 后续候选已接入同一 worker 的受管 OAuth/Keyring 与只读 metadata probe；真实金融工具仍未资格化，不会因授权成功启用。产品 v2 manifest 的外部网络能力只适用于该固定 profile，详见 [Tushare OAuth 实现及未执行范围](docs/tushare/oauth-implementation.md)。
