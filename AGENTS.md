# AGENTS.md

## 仓库职责

`yijie-connectors` 是外部平台 API 与三方应用的工具执行层。

## 禁止事项

- 不承载业务主状态；
- 不写 Skill prompt；
- 不实现 RAG 检索；
- 不绕过审批策略执行高风险写操作；
- 不把真实 token 写入代码、文档、日志或 fixtures。

## 技术栈

Go + MCP 工具服务。当前骨架只提供最小 HTTP 服务。

## 开发命令

```bash
make dev
make test
```
