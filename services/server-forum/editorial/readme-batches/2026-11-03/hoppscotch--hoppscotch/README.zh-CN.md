<div align="center">
  <a href="https://hoppscotch.io">
    <img
      src="https://raw.githubusercontent.com/hoppscotch/hoppscotch/39d25650bdb4e86464b2cd69174c567b86f57892/packages/hoppscotch-common/public/logo.svg"
      alt="Hoppscotch"
      height="64"
    />
  </a>
  <h3>
    <b>
      Hoppscotch
    </b>
  </h3>
  <b>
    开源 API 开发生态系统
  </b>
  <p>

[![contributions welcome](https://img.shields.io/badge/contributions-welcome-brightgreen?logo=github)](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/CODE_OF_CONDUCT.md) [![Website](https://img.shields.io/website?url=https%3A%2F%2Fhoppscotch.io&logo=hoppscotch)](https://hoppscotch.io) [![Tests](https://github.com/hoppscotch/hoppscotch/actions/workflows/tests.yml/badge.svg)](https://github.com/hoppscotch/hoppscotch/actions) [![Tweet](https://img.shields.io/twitter/url?url=https%3A%2F%2Fhoppscotch.io%2F)](https://x.com/share?text=%F0%9F%91%BD%20Hoppscotch%20%E2%80%A2%20Open%20source%20API%20development%20ecosystem%20-%20Helps%20you%20create%20requests%20faster,%20saving%20precious%20time%20on%20development.&url=https://hoppscotch.io&hashtags=hoppscotch&via=hoppscotch_io)

  </p>
  <p>
    <sub>
      由以下人员用 ❤︎ 构建：
      <a href="https://github.com/hoppscotch/hoppscotch/graphs/contributors">
        贡献者
      </a>
    </sub>
  </p>
  <br />
  <p>
    <a href="https://hoppscotch.io">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/hoppscotch/hoppscotch/39d25650bdb4e86464b2cd69174c567b86f57892/packages/hoppscotch-common/public/images/banner-dark.png">
        <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/hoppscotch/hoppscotch/39d25650bdb4e86464b2cd69174c567b86f57892/packages/hoppscotch-common/public/images/banner-light.png">
        <img alt="Hoppscotch" src="https://raw.githubusercontent.com/hoppscotch/hoppscotch/39d25650bdb4e86464b2cd69174c567b86f57892/packages/hoppscotch-common/public/images/banner-dark.png">
      </picture>
    </a>
  </p>
</div>

_强烈建议查看 [**Hoppscotch 文档**](https://docs.hoppscotch.io)，了解更多应用信息。_

#### **支持**

[![Chat on Discord](https://img.shields.io/badge/chat-Discord-7289DA?logo=discord)](https://hoppscotch.io/discord) [![Chat on Telegram](https://img.shields.io/badge/chat-Telegram-2CA5E0?logo=telegram)](https://hoppscotch.io/telegram) [![Discuss on GitHub](https://img.shields.io/badge/discussions-GitHub-333333?logo=github)](https://github.com/hoppscotch/hoppscotch/discussions)

### **功能**

❤️ **轻量：** 采用简约的界面设计。

⚡️ **快速：** 实时发送请求并获取响应。

🗄️ **HTTP 方法：** 请求方法定义你要求执行的操作类型。

- `GET` - 请求获取资源信息
- `POST` - 服务器在数据库中创建新条目
- `PUT` - 更新现有资源
- `PATCH` - 与 `PUT` 很相似，但对资源进行部分更新
- `DELETE` - 删除资源或相关组件
- `HEAD` - 获取与 GET 请求相同的响应头，但不包含响应体。
- `CONNECT` - 建立通向目标资源所标识服务器的隧道
- `OPTIONS` - 描述目标资源的通信选项
- `TRACE` - 沿通向目标资源的路径执行消息回环测试
- `<custom>` - 某些 API 使用自定义请求方法，例如 `LIST`。请输入你的自定义方法。

🌈 **主题：** 可自定义背景、前景和强调色组合——[立即自定义](https://hoppscotch.io/settings)。

- 选择主题：跟随系统、浅色、深色和黑色
- 选择强调色：绿色、蓝绿色、蓝色、靛蓝色、紫色、黄色、橙色、红色和粉色
- 无干扰禅模式

_自定义主题会与云端／本地会话同步。_

🔥 **PWA：** 在设备上作为[渐进式 Web 应用](https://web.dev/progressive-web-apps)安装。

- 使用 Service Worker 即时加载
- 支持离线
- 内存和 CPU 占用低
- 添加到主屏幕
- 桌面 PWA

🚀 **请求：** 即时获取端点响应。

1. 选择 `method`
2. 输入 `URL`
3. 发送

- 复制／分享公开的“Share URL”
- 为 10 多种语言和框架生成／复制请求代码片段
- 导入 `cURL`
- 为请求添加标签

🔌 **WebSocket：** 通过单个 TCP 连接建立全双工通信通道。

📡 **Server-Sent Events：** 通过 HTTP 连接接收服务器更新流，无需轮询。

🌩 **Socket.IO：** 与 SocketIO 服务器收发数据。

🦟 **MQTT：** 订阅 MQTT Broker 的主题并向其发布消息。

🔮 **GraphQL：** GraphQL 是一种 API 查询语言，也是使用现有数据执行这些查询的运行时。

- 设置端点并获取模式
- 多列文档
- 设置自定义请求头
- 查询模式
- 获取查询响应

🔐 **授权：** 用于识别最终用户。

- 无
- Basic
- Bearer Token
- OAuth 2.0
- OIDC Access Token/PKCE

📢 **请求头：** 描述请求体发送时使用的格式。

📫 **参数：** 使用请求参数设置模拟请求中可变的部分。

📃 **请求体：** 用于通过 REST API 发送和接收数据。

- 设置 `Content Type`
- FormData、JSON 等
- 在键值和 RAW 输入参数列表之间切换

📮 **响应：** 包含状态行、响应头和消息／响应体。

- 将响应复制到剪贴板
- 将响应下载为文件
- 查看响应头
- 查看原始响应并预览 HTML、图片、JSON 和 XML 响应

⏰ **历史记录：** 请求条目与云端／本地会话存储同步。

📁 **集合：** 使用集合和文件夹组织 API 请求，一键复用。

- 不限数量的集合、文件夹和请求
- 嵌套文件夹
- 通过文件或 GitHub gist 导出和导入

_集合会与云端／本地会话存储同步。_

📜 **请求前脚本：** 与请求关联、在发送请求之前执行的代码片段。

- 设置环境变量
- 在请求头中包含时间戳
- 在 URL 参数中发送随机字母数字字符串
- 任意 JavaScript 函数

👨‍👩‍👧‍👦 **团队：** 帮助团队协作，更快地设计、开发和测试 API。

- 创建不限数量的团队
- 创建不限数量的共享集合
- 创建不限数量的团队成员
- 基于角色的访问控制
- 云端同步
- 多设备

👥 **工作空间：** 将个人及团队的集合和环境组织到工作空间中，轻松切换以管理多个项目。

- 创建不限数量的工作空间
- 在个人和团队工作空间之间切换

⌨️ **键盘快捷键：** 为提高效率而优化。

> **[阅读键盘快捷键文档](https://docs.hoppscotch.io/documentation/features/shortcuts)**

🌐 **代理：** 在设置中启用代理模式以访问被阻止的 API。

- 隐藏你的 IP 地址
- 解决 [`CORS`](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)（跨源资源共享）问题
- 访问通过非 HTTPS（`http://`）端点提供的 API
- 使用你的代理 URL

_官方代理服务器由 Hoppscotch 托管 - **[GitHub](https://github.com/hoppscotch/proxyscotch)** - **[隐私政策](https://docs.hoppscotch.io/support/privacy)**。_

🌎 **i18n：** 使用你的语言体验应用。

帮助我们翻译 Hoppscotch。请阅读 [`TRANSLATIONS`](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/TRANSLATIONS.md)，了解我们的 [`CODE OF CONDUCT`](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/CODE_OF_CONDUCT.md)以及提交 Pull Request 的流程。

☁️ **认证与同步：** 登录并在所有设备间实时同步数据。

**登录方式：**

- GitHub
- Google
- Microsoft
- 电子邮件
- SSO（单点登录）[^EE]

**🔄 同步数据：** 通过接续功能在其他设备上继续任务。

- 工作空间
- 历史记录
- 集合
- 环境
- 设置

✅ **请求后测试：** 编写与请求关联、在收到响应后执行的测试。

- 将状态码作为整数检查
- 过滤响应头
- 解析响应数据
- 设置环境变量
- 编写 JavaScript 代码

🌱 **环境：** 环境变量允许你在请求和脚本中存储并复用值。

- 不限数量的环境与变量
- 通过请求前脚本初始化
- 导出到／从 GitHub gist 导入

<details>
  <summary><i>使用场景</i></summary>

---

- 将值存入变量后，可以在请求的各部分引用它
- 如果需要更新该值，只需在一个位置修改
- 使用变量有助于提高工作效率并减少出错可能

---

</details>

🚚 **批量编辑：** 批量编辑键值对。

- 条目以换行分隔
- 键和值以 `:` 分隔
- 对希望添加但保持禁用的行，在前面加上 `#`

🎛️ **管理仪表盘：** 管理团队并邀请成员。

- 洞察
- 管理用户
- 管理团队

📦 **附加组件：** Hoppscotch 官方附加组件。

- **[Hoppscotch CLI](https://github.com/hoppscotch/hoppscotch/tree/main/packages/hoppscotch-cli)** - Hoppscotch 的命令行界面。
- **[Proxy](https://github.com/hoppscotch/proxyscotch)** - 为 Hoppscotch 创建的简单代理服务器。
- **[Browser Extensions](https://github.com/hoppscotch/hoppscotch-extension)** - 改善 Hoppscotch 体验的浏览器扩展。

  [![Firefox](https://raw.github.com/alrra/browser-logos/master/src/firefox/firefox_16x16.png) **Firefox**](https://addons.mozilla.org/en-US/firefox/addon/hoppscotch) &nbsp;|&nbsp; [![Chrome](https://raw.github.com/alrra/browser-logos/master/src/chrome/chrome_16x16.png) **Chrome**](https://chrome.google.com/webstore/detail/hoppscotch-extension-for-c/amknoiejhlmhancpahfcfcfhllgkpbld)

  > **扩展可解决 `CORS` 问题。**

_附加组件由 **[Hoppscotch Organization](https://github.com/hoppscotch)** 开发和维护。_

**完整功能列表请阅读[文档](https://docs.hoppscotch.io)。**

## **演示**

- Web： [hoppscotch.io](https://hoppscotch.io)
- Windows/Linux/macOS : [桌面应用](https://docs.hoppscotch.io/documentation/clients/desktop#download-hoppscotch-desktop-app)

## 使用

1. 在 URL 字段输入 API 端点
2. 点击“Send”模拟请求
3. 查看响应

## 开发

按照[自托管文档](https://docs.hoppscotch.io/documentation/self-host/getting-started)开始使用开发环境。

## 参与贡献

请使用 [GitHub Flow](https://guides.github.com/introduction/flow) 参与贡献：创建分支、添加提交并[发起 Pull Request](https://github.com/hoppscotch/hoppscotch/compare)。

请阅读 [`CONTRIBUTING`](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/CONTRIBUTING.md)，了解我们的 [`CODE OF CONDUCT`](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/CODE_OF_CONDUCT.md)，以及提交 Pull Request 的流程。

## 持续集成

我们使用 [GitHub Actions](https://github.com/features/actions) 进行持续集成。请查看[构建工作流](https://github.com/hoppscotch/hoppscotch/actions)。

## 更新日志

详情请参阅 [`CHANGELOG`](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/CHANGELOG.md) 文件。

## 作者

本项目得以存在，离不开所有贡献者的共同努力——[立即贡献](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/CONTRIBUTING.md).

<div align="center">
  <a href="https://github.com/hoppscotch/hoppscotch/graphs/contributors">
    <img src="https://opencollective.com/hoppscotch/contributors.svg?width=840&button=false"
      alt="Contributors"
      width="100%" />
  </a>
</div>

## 许可证

本项目采用 [MIT 许可证](https://opensource.org/licenses/MIT)——详情请参阅 [`LICENSE`](https://github.com/hoppscotch/hoppscotch/blob/39d25650bdb4e86464b2cd69174c567b86f57892/LICENSE) 文件。

[^EE]: 企业版功能。[了解更多](https://docs.hoppscotch.io/documentation/self-host/getting-started).
