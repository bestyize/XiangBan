# 服务端项目

- [xiaban-api](xiaban-api/README.md)：已实现的主 API，包含 backend 和 api-contracts。
- [server-pan](server-pan/README.md)：已实现的自托管网盘（PanDown），Rust 后端 + React 前端，自带账号与 SQLite 存储。
- [server-xiangban-wallpaper](server-xiangban-wallpaper/README.md)：正式壁纸网站 wallpaper.xiangban.xyz，图片/视频采集与上传、小伴桌面下载、版本与公告；桌面客户端接入此服务。
- [server-account](server-account/README.md)：统一账号中心，验证码验证身份、用户名与密码登录、JWT 签发及管理后台。
- [server-xiaoban-appstore](server-xiaoban-appstore/README.md)：应用商店与上下架管理后台。
- [server-xiaoban-market](server-xiaoban-market/README.md)：专家、技能、连接器目录与采集工作台。
- [server-forum](server-forum/README.md)：超级小伴论坛 chat.xiangban.xyz，技术交流、资源分享、福利羊毛与闲聊的社区，一期网页版已实现，已部署在香港机。
- [server-resource](server-resource/README.md)：资源机，社区内容包与封面图的直传校验和签名下载，部署在独立的大带宽机器上。
- [server-link](server-link/README.md)：手机与桌面互联的中转服务：同一账号登录的手机和电脑自动互相发现，转发文本消息与在线状态；不存消息历史。
- [server-ai](server-ai/README.md)：AI 网关与额度系统，一期已实现、未部署；账号中心验签，经 New API 调用上游模型，额度按账号与产品隔离。
- [server-asr](server-asr/README.md)：规划的语音识别服务。
- [server-edgetts](server-edgetts/README.md)：规划的 Edge TTS 服务。
- [server-websearch](server-websearch/README.md)：规划的搜索服务。

按运行和部署职责组织，不因大仓重组自动拆微服务。当前聊天、语音、打卡与后台任务仍在同一个 Ktor 服务内。
