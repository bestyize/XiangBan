# 服务端项目

- [xiaban-api](xiaban-api/README.md)：已实现的主 API，包含 backend 和 api-contracts。
- [server-asr](server-asr/README.md)：规划的语音识别服务。
- [server-edgetts](server-edgetts/README.md)：规划的 Edge TTS 服务。
- [server-websearch](server-websearch/README.md)：规划的搜索服务。

按运行和部署职责组织，不因大仓重组自动拆微服务。当前聊天、语音、打卡与后台任务仍在同一个 Ktor 服务内。
