# Security Policy

连接器是平台 token 的关键安全边界。真实 token 必须进入 token vault，不得进入 Codex Runtime、Skills、日志或 fixtures。

FEAT-157 当前提供安全51项目录、默认私有 `auth_status` worker，以及独立显式模式的 Host 所有 Broker/Gateway。所有真实服务业务工具未资格化，产品非空选择拒绝；旧默认状态模式不读Keyring、不接收秘密或URL、不打开OAuth。显式受管Tushare profile可在Native授权后进行OAuth/Keyring及metadata网络操作，凭据仍只在worker边界。HTTP/OAuth库边界强制Keyring及scope/connection隔离alias，真实用户凭据、回调、scope和运行能力仍须后续实测资格，不能把编译通过当真实安全存储通过。Google私有凭据文件不能因目录私有就视为已保护。

Native SQLCipher保存非秘密产品意图，Host只做映射/能力，Connectors不建立第二份安装业务数据库。新worker由标准构建产生并按digest保存；owner仅stdin EOF和正常wait，超时保留清理状态，无强杀或替换可执行文件测试。暂无公开管理API或真实工具执行授权。

FEAT-157 后续本地候选增加 Host 直接拥有的私有 Broker 模式与官方 rmcp loopback Gateway；真实 provider 全部未资格化，产品非空选集仍拒绝。独立合成资格二进制不能充当产品 worker，详见 [Broker 控制说明](docs/market-broker-control.md)。

当前 OAuth / metadata 网络与未开放的金融工具边界详见 [Tushare受管候选](docs/tushare/oauth-implementation.md)。
