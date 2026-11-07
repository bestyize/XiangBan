# Android 运行时协议 2 技术小样

本小样只进入 Android Debug 包，不接入现有游戏运行页。安装 Debug APK 后运行：

```sh
adb shell am start -n xyz.xiangban.xiaoban/.RuntimeV2SpikeActivity
```

测试地址是 `https://spike1.app.xiangban.xyz/v1/index.html`。需要 Android 7.0（API 24）及以上，且设备的 WebView 实现必须同时支持 `DOCUMENT_START_SCRIPT` 和 `WEB_MESSAGE_LISTENER`；这两个特性由实际 WebView 实现决定，不能仅凭 Android 系统版本判断。缺少任意一项时，Activity 显示缺失的特性名称以及更新 Android System WebView 或 Chrome 的提示，不加载测试页。依赖使用 [AndroidX WebKit 1.18.0](https://developer.android.com/jetpack/androidx/releases/webkit)。

## 实现与验证方式

1. 通过 WebView 请求拦截把作品 HTTPS 源映射到 Debug assets；页面显示安全上下文及来源。返回 JS、MJS、Wasm 等 MIME 类型、CORS 和 CSP，其他 HTTPS 地址正常联网。路径拒绝目录穿越和编码分隔符。
2. localStorage 提供写入、重启 Activity 后读取按钮。需人工确认重启后仍存在。
3. IndexedDB 建库后写入并读取。
4. OPFS 创建文件、写入并读回。
5. 模块测试分别调用包含静态 import 的模块及动态 import。
6. 分别启动普通 Worker 和 module Worker。
7. 加载资产里的加法 Wasm，校验 `add(2, 3)`。
8. 公网使用 `no-cors` fetch，记录响应类型和状态；内网 HTTP 请求应被拒绝。CSP 限制网络为 HTTPS/WSS，原生请求拦截拒绝列出的本机和局域网字面地址。拒绝可能由浏览器混合内容限制、CSP 或宿主拦截产生，单靠页面结果不能区分。
9. 分别点击摄像头、麦克风按钮；宿主先询问，再请求 Android 系统权限，通过后验证媒体流并停止轨道。
10. document-start 脚本注入 `xb`、安全区信息和 CSS 变量；安全区由 WindowInsets 转成 CSS 像素，变化时更新。设备真实安全区全为零时非零检查会失败，不代表值是伪造的。
11. 按钮经 WebMessage 双向传递 3 MiB 字符串，对比写入私有原子文件并读回的内容。消息按协议 2 的 channel、protocol、kind、id 封装，接收端检查主框架和完整来源。小样支持 `ready`、`storage.get/set` 和事件，不是完整 SDK。
12. 点击后创建两条历史，手动连续按系统返回，页面检查是否依次回退；无历史时两秒内再按返回退出。
13. 切后台再回前台，页面检查 `hide`、`show` 顺序。

纯 Kotlin 的请求路径、MIME 和指定内网地址范围另有 JVM 单元测试。小样 CSP 及 `shouldInterceptRequest` 并不是完整网络防火墙：WebSocket 不经过该回调，DNS 重绑定、重定向等情况仍需专项评估；本小样未实现域名封禁、网络记录或完整权限策略，不能直接作为生产安全边界。

## 本机结果与待实机确认

- Debug APK：`./gradlew :androidApp:assembleDebug` 构建成功。本次同时运行单元测试，因沙箱限制使用临时 Gradle 和 Android 缓存及签名目录。
- JVM 单元测试 `:androidApp:testDebugUnitTest --tests '*RuntimeV2RequestsTest*'`：3 项通过，0 失败。
- 本机 JavaScript 语法检查通过；Node.js 可实例化 Wasm 并得到 `add(2, 3) = 5`。这些检查不能替代 WebView 测试。
- 尚未在真机安装和运行该小样。连接的设备已有同包名应用，本次未覆盖安装。上述 1–13 项的 WebView 能力、权限弹窗、存储持久性、消息大小、系统返回与生命周期均需在手机上确认；localStorage、摄像头、麦克风、大消息、历史和生命周期项目需要人工操作。
