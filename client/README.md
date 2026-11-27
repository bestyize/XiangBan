# 客户端

- [xiaoban-desktop](xiaoban-desktop/README.md)：Windows/macOS 桌面应用，包含壁纸、桌宠、组件、游戏与 Agent。
- [xiaoban-multiplatform](xiaoban-multiplatform/README.md)：Android/iOS 使用 KMP + Compose，包含社区、小伴 AI 与手机桌面互联；桌面壳使用独立 Tauri 工程。平台验证范围见项目 README。
- [personality-tests-h5](personality-tests-h5/README.md)：人格贩卖机 VEND·ME 趣味与人格测评移动端 H5（Vue 3 + Vite 纯前端，潮玩盲盒风，含 6 套完整原创题库、专属海报皮肤、Canvas 高清导出、Web Share 系统分享与零门槛双人合盘）。

应用自行维护依赖、内部公共代码和构建发布。允许重复代码，不通过相对路径依赖另一个客户端的源码。需要跨端同步时明确范围并分别验证。
