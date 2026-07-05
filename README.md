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

当前只初始化 Go 最小 HTTP 服务和目录骨架，不接真实平台 API、不保存真实 token。

## 本地开发

```bash
make dev
curl http://localhost:18081/healthz
```
