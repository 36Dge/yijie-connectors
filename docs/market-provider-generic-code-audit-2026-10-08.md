# 49 项通用 Provider 实现审计

2026-10-08。此记录接替同日实施前审计。Owner 已移除淘宝闪购、豆蔻医生；Agentic Engine 已恢复本轮范围。仅 Tushare 真实验收，另外48项只做代码审计及普通本地测试。

注册和代码审计不代表供应商当前可用。实际配置后，仍须授权、完整工具发现、schema 校验、显式启用、本轮有效选择和逐次审批，才能执行。

| 服务 | 路径 | 审计与适用条件 |
| --- | --- | --- |
| cue (`cue`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 百晓智能 (`baixiao-mcp`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 单小二 (`dxe-mcp-server`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Hologrow (`Hologrow`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 好多店 (`X-Store`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 银泰商业开放平台 (`yintai-open-platform`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 语忆Neosight (`yuyidata`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| FTShare 金融数据 (`FTShare`) | fixed_header_credentials | 固定 FTSHARE_API_KEY；worker 自有一次性配置页、Keyring、精确资源 Header 注入，逐次审批。 |
| 云听ConsumerLens (`yunting-consumerlens`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Google 日历 (`google-calendar`) | google_registered_oauth | 官方远程 MCP；需开发者预览资格和自有 Web OAuth 客户端；默认只读，修改权限需配置时显式勾选，删除拒绝。 |
| Google 地图 (`google-maps`) | fixed_header_credentials | 固定 X-Goog-Api-Key；worker 自有一次性配置页、Keyring、精确资源 Header 注入，逐次审批。 |
| 思必驰AI办公 (`speechclaw`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 草料二维码 (`caoliao`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| WaveNote (`wavenote`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 宜搭 (`yida`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 轻流 (`qingflow`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| UU跑腿 (`uupt`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 分贝通 (`fenbeitong`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 数阔八爪鱼 (`bazhuayu`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 扫描全能王 (`camscanner-mcp`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Todoist (`todoist`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Agentkey (`agentkey-qwen`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 惠搭ERP (`huida-erp`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| FastMoss (`FastMoss`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 薪智·市场人才薪酬数据 (`smartsalary`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 启信慧眼 (`qixin_insight`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 执中·金融数据 (`zerone`) | fixed_query_credentials | 官方2026-05手册第13页确认固定api_key URL查询参数；worker发出请求前才注入，Codex client/Host/Runtime持有无密钥地址；不做真实调用。 |
| 智慧芽·专利分析 (`patent-analysis`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Agentic Engine (`thinkingdata`) | explicit_exported_header_credentials | 官方文档说明从 AE 导出远程客户端配置。安全页要求填写该固定地址对应的 Header 名称/完整值，默认原值，不推断 Bearer 或 API_TOKEN 映射。保存后需实际发现资格和逐次审批；官方公开页未独立确认参考端点的 Header 字段。 |
| 晨星 Morningstar (`morningstar`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| CRIC 克而瑞地产数据 (`dichanai-mcp`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 盈米基金 (`Yingmi`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Wind Alice万得金融数据 (`wind`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 质数幻方·企业数据 (`yidian-company`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 今日投资·金融数据 (`investoday`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 大智慧 (`dzh-mcp`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 八爪鱼Data Hub (`Octoparse-data-hub`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| Tushare·金融数据 (`tushareMcp`) | reviewed_tushare_daily | 保留单代码/单日 daily 专用策略及真实批准/拒绝证据；未开放另外253个工具。 |
| 东方财富妙想MCP (`eastmoney`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 恒生聚源金融问数 (`gildata_finance_data_qwen`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| MOSS增长谋士 (`fanruan-growth-advisor`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 微博 (`weibo`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 金数据 (`jinshuju`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 小鹅通 (`xiaoe-mcp`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 飞哨GEO (`feisentry`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 小裂变SCRM (`xiaoliebian`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 智客AI (`ysk_mcp_3f824505d1faf80bc60c052729620929`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 3Chat私域客户运营 (`3chat`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |
| 小裂变GEO (`xiaoliebian-geo`) | standard_oauth_discovery | 固定服务 HTTPS 发现链、DCR/PKCE、Codex Keyring、完整 schema 冻结、单次审批；服务方必须提供兼容的 MCP OAuth 注册/发现能力。 |

## 证据与限制

- Worker56、Native38、前端56、Contracts Broker/Host15项通过；Worker/Native Clippy、前端lint、Native同源检查、目录Go race和Go vet通过。
- 两个普通进程内MCP服务验证同名工具隔离、第二服务精确引用、拒绝零执行、批准一次且不重放；无真实供应商请求。
- Tushare真实批准返回HTTP200/close=11.5，拒绝发送0次；证据对应f8f85fa4构建。随后的执中query认证、目录范围和AE指引修改用本地测试验证，未再消耗模型/查询额度。
- Google采用官方远程MCP，日历需要预览资格和自有Web OAuth应用；默认只读，修改权限显式选择，删除拒绝。
- AE文档主要描述其宿主配置及导出流程，未公开这个参考HTTP端点的确切认证头。配置页按AE导出字段接收原值；产品页的stdio API_TOKEN示例不作为HTTP认证依据，也未安装或运行其npx包。
- Agentic Engine未做真实授权、metadata或查询；若账号导出的是不同端点、多Header或stdio配置，需补充相应官方规格后适配，当前不自动转换。
- 固定Codex Runtime与用户已有工作区改动不修改；没有强杀、二进制冒充、权限破坏、攻击fixture或发布。

机器可读逐项记录和当前源码摘要见同名JSON；真实验收见元仓FEAT-157/17及evidence/generic-final-acceptance-2026-10-08.json。

官方依据：[AE MCP](https://docs.thinkingai.cn/zh/manual/mcp)、[AE认证配置](https://docs.thinkingai.cn/zh/manual/mcp_integration)、[AE对外介绍](https://www.thinkingai.cn/product/mcp-service/)、[Google Maps](https://developers.google.com/maps/ai/grounding-lite)、[Google Calendar](https://developers.google.com/workspace/calendar/api/guides/configure-mcp-server)。执中手册来源见原审计记录和17。
