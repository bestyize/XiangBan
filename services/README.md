# 服务端项目

- [xiaban-api](xiaban-api/README.md)：已实现的主 API，包含 backend 和 api-contracts。
- [server-pan](server-pan/README.md)：已实现的自托管网盘（PanDown），Rust 后端 + React 前端，自带账号与 SQLite 存储。
- [server-xiangban-wallpaper](server-xiangban-wallpaper/README.md)：正式壁纸网站 wallpaper.xiangban.xyz，图片/视频采集与上传、小伴桌面下载、版本与公告；桌面客户端接入此服务。
- [server-forum](server-forum/README.md)：超级小伴论坛 chat.xiangban.xyz，技术交流、资源分享、福利羊毛与闲聊的社区，一期网页版已实现，已部署在香港机。
- [server-asr](server-asr/README.md)：规划的语音识别服务。
- [server-edgetts](server-edgetts/README.md)：规划的 Edge TTS 服务。
- [server-websearch](server-websearch/README.md)：规划的搜索服务。

按运行和部署职责组织，不因大仓重组自动拆微服务。当前聊天、语音、打卡与后台任务仍在同一个 Ktor 服务内。
