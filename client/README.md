# 客户端

- [xiaoban-desktop](xiaoban-desktop/README.md)：Windows/macOS 桌面应用，包含壁纸、桌宠、组件、游戏与 Agent。
- [xiaoban-multiplatform](xiaoban-multiplatform/README.md)：Android/iOS 使用 KMP + Compose，包含社区、小伴 AI 与手机桌面互联；桌面壳使用独立 Tauri 工程。平台验证范围见项目 README。
- [personality-tests-h5](personality-tests-h5/README.md)：人格贩卖机 VEND·ME 趣味与人格测评移动端 H5（Vue 3 + Vite 纯前端，潮玩盲盒风，含 6 套完整原创题库、专属海报皮肤、Canvas 高清导出、Web Share 系统分享与零门槛双人合盘）。
- [xiaoban-games-h5](xiaoban-games-h5/README.md)：小伴游戏厅（`game.xiaban.xyz`）移动端小游戏合集站（Vue 3 + Vite + PWA 纯静态前端，同步内置 8 款小伴官方游戏、双协议运行时宿主、免登录本机存档/导出备份与相伴统一账号登录弹窗）。
- [xiaban-portal](xiaban-portal/README.md)：小伴产品矩阵总站与双端独立下载页（`xiaban.xyz`、`/desktop`、`/app`，Vue 3 + Vite 纯静态前端，图片托管于资源机、反代版本接口）。

- [aireset-web](aireset-web/README.md)：AI订阅测评 QuotaReset（`aireset.xiaban.xyz`）中英双语静态站（零依赖，构建期预渲染 `/`、`/benchmark/`、`/en/`、`/en/benchmark/` 以利 SEO；御三家额度重置雷达与订阅额度实测，重置记录目前为演示数据）。

应用自行维护依赖、内部公共代码和构建发布。允许重复代码，不通过相对路径依赖另一个客户端的源码。需要跨端同步时明确范围并分别验证。
