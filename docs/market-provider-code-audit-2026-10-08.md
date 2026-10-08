# 51 项 Provider 代码审计（2026-10-08）

> 本文件保留实施前快照。当前实现与50项范围见 [后续通用实现审计](market-provider-generic-code-audit-2026-10-08.md)。

**验收口径：其他50项不做真实登录、授权、metadata或业务调用；仅代码审计及本地无外部调用的测试。Tushare为唯一真实代表。** 旧诊断不足不阻塞交付。

本表是实现缺口审计，不是51项服务端可用性测试。逐项对照市场目录、原始接入清单与同一真实调用链：Native provider/store/mod → Host provider → worker Providers → Broker → Gateway。Source hashes 见相邻 JSON。

通用安装/默认关闭/非秘密持久与图标覆盖51项；当前 provider 管理显式拒绝非Tushare，Broker只接受单Tushare capability，Gateway只暴露daily。需要补的是真实代码分支，不能通过把调用验证状态改成“不需要”让这些分支变为已实现。

| 服务 | 认证策略 | 执行代码审计 | 真实验收 |
| --- | --- | --- | --- |
| cue (`cue`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 百晓智能 (`baixiao-mcp`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 单小二 (`dxe-mcp-server`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| Hologrow (`Hologrow`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 好多店 (`X-Store`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 银泰商业开放平台 (`yintai-open-platform`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 语忆Neosight (`yuyidata`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 淘宝闪购零售商家版 (`taobao-flash-sale-retail`) | provider_gateway | 易界淘宝开放平台应用/网关适配缺失；不能复用来源应用身份 | Owner不要求 |
| FTShare 金融数据 (`FTShare`) | api_key | 安全凭据输入与固定 FTSHARE_API_KEY header 适配缺失 | Owner不要求 |
| 云听ConsumerLens (`yunting-consumerlens`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| Google 日历 (`google-calendar`) | local_oauth | 固定包、安全凭据/token存储与正常EOF集成缺失 | Owner不要求 |
| Google 地图 (`google-maps`) | stdio_api_key | 固定包、worker内API key注入与正常EOF集成缺失 | Owner不要求 |
| 思必驰AI办公 (`speechclaw`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 草料二维码 (`caoliao`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| WaveNote (`wavenote`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 宜搭 (`yida`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 轻流 (`qingflow`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| UU跑腿 (`uupt`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 分贝通 (`fenbeitong`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 数阔八爪鱼 (`bazhuayu`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 扫描全能王 (`camscanner-mcp`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| Todoist (`todoist`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| Agentkey (`agentkey-qwen`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 惠搭ERP (`huida-erp`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| FastMoss (`FastMoss`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 薪智·市场人才薪酬数据 (`smartsalary`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 启信慧眼 (`qixin_insight`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 执中·金融数据 (`zerone`) | provider_credentials | 安全凭据输入及各平台传递协议适配缺失 | Owner不要求 |
| 智慧芽·专利分析 (`patent-analysis`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| Agentic Engine (`thinkingdata`) | provider_credentials | 安全凭据输入及各平台传递协议适配缺失 | Owner不要求 |
| 晨星 Morningstar (`morningstar`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| CRIC 克而瑞地产数据 (`dichanai-mcp`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 盈米基金 (`Yingmi`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| Wind Alice万得金融数据 (`wind`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 质数幻方·企业数据 (`yidian-company`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 今日投资·金融数据 (`investoday`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 大智慧 (`dzh-mcp`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 八爪鱼Data Hub (`Octoparse-data-hub`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 豆蔻医生 (`doukou-doctor`) | provider_credentials | 安全凭据输入及各平台传递协议适配缺失 | Owner不要求 |
| Tushare·金融数据 (`tushareMcp`) | oauth | 授权、Keyring、metadata、daily批准/拒绝已有实现及真实证据；不代表全部254工具 | Tushare 已通过 |
| 东方财富妙想MCP (`eastmoney`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 恒生聚源金融问数 (`gildata_finance_data_qwen`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| MOSS增长谋士 (`fanruan-growth-advisor`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 微博 (`weibo`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 金数据 (`jinshuju`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 小鹅通 (`xiaoe-mcp`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 飞哨GEO (`feisentry`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 小裂变SCRM (`xiaoliebian`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 智客AI (`ysk_mcp_3f824505d1faf80bc60c052729620929`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 3Chat私域客户运营 (`3chat`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |
| 小裂变GEO (`xiaoliebian-geo`) | oauth | 通用 OAuth/metadata、服务工具适配缺失，当前被 Tushare 专用分支拒绝 | Owner不要求 |

## 下一步实现约束

复用已有私有控制管道、Codex OAuth/Keyring/client、调用绑定和单次批准；将服务定义、元数据及执行策略从Tushare专用判断抽离。输入/输出schema与工具风险仍须有权威，不能仅删掉硬编码后开放未知工具。四类凭据/stdio/平台网关差异在Connectors处理，不把平台token移入Host/Runtime。

FTShare 官方确认使用 `FTSHARE_API_KEY` HTTP header，不是推测的通用 Bearer；[官方文档](https://github.com/FTShare-Lab/FTShare-MCP/blob/main/README.md)。Todoist的OAuth远端有[官方依据](https://developer.todoist.com/api/v1/#tag/Todoist-MCP)。Google日历的文件token路径见[上游说明](https://github.com/nspady/google-calendar-mcp)；未运行这些服务。
