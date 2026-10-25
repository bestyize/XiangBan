# 客户端

- [xiaban-web](xiaban-web/README.md)：已实现的网页/PWA与管理后台。
- [xiaoban-desktop-wallpaper](xiaoban-desktop-wallpaper/README.md)：Windows/macOS 桌面应用，包含壁纸、桌宠、组件、游戏与 Agent。
- [xiaoban-mobile-wallpaper](xiaoban-mobile-wallpaper/README.md)：Android 壁纸浏览与下载应用，独立 Compose 工程。
- [xiaoban-pot-garden](xiaoban-pot-garden/README.md)：three.js 表情盆栽原型，尚无养成玩法。
- [xiaoban-multiplatform](xiaoban-multiplatform/README.md)：可运行的网页壳原型；Android/iOS 使用 KMP + Compose，桌面使用独立 Tauri 工程。平台验证范围见项目 README。
- [xiaoban-realbot](xiaoban-realbot/README.md)：未来实体机器人，尚未建立工程。

应用自行维护依赖、内部公共代码和构建发布。允许重复代码，不通过相对路径依赖另一个客户端的源码。需要跨端同步时明确范围并分别验证。
