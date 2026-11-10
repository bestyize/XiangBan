# 小伴应用规范（运行时协议 2）

状态：**协议 2 分端实施中**。Android 正式宿主已运行协议 2；桌面已迁移内置作品并实现本地容器；网页已有协议 2 试玩前端，但独立作品域名的文件服务尚未完成；iOS 还没有宿主。第 2 节保留协议 1 的历史说明，具体实施和验证以 [progress.md](progress.md) 为准。本文定下协议 2 的应用包、运行环境、安全模型、生命周期、SDK、权限、多端范围、发布检查和兼容规则。

给 AI 的精简版见 [ai-brief.md](ai-brief.md)，它由本文派生，两者冲突时以本文为准。实施进度见 [progress.md](progress.md)。

2026-11-08 产品负责人定下的取舍：

- 包体积上限 100 MB。
- 作品可以自由联网。
- 作品申请权限后可以使用摄像头、麦克风、通讯录和文件。
- 移动端优先：网页和桌面做不到的、或者会明显增加实现复杂度的能力，先不支持。
- 社区作品和普通帖子一样，谁都能看到、能打开，界面上不显示审核状态。
- 用户只用手机上的小伴 Agent 就能制作并发布作品（P0）。
- 先不做年龄分级，不申请版号，不做机器抽检。
- 作品里的 AI 由作品自己联网调用，小伴不提供按点数计费的 AI 接口。
- 小伴后台可以禁止某个作品联网（第 5.5 节）。
- 原作删除后，改编作品保留；原作者在打开「允许改编」时就会被告知这一点。
- 应用域名用 `app.xiangban.xyz`。
- iOS 宿主暂缓，先做 Android。本文里关于 iOS 的内容保留，作为以后实现时的依据。
- `xb.storage` 只保存在这台设备上，先不做云同步。需要多设备同步的作者用自己的服务器，以 `profile` 的 `openId` 识别用户；官方作品确实需要同步时再单独设计。

## 1. 范围与原则

**作品**指小伴里可以全屏打开的应用：游戏（`kind: "game"`）和工具、测评（`kind: "app"`）。官方作品、社区作品和用户自己做的作品用同一种格式。壁纸、小组件、特效、伙伴不在本文范围内。

原则：

1. **只用 Web 标准。** 作品就是一个普通网站，用 HTML、CSS、JavaScript、ES modules、Web Worker、WebAssembly、Canvas、WebGL、WebAudio 都可以，不另造标记语言。前端框架、游戏引擎的 Web 导出、小伴 AI 写的代码都能直接运行。
2. **小伴特有的部分做薄。** 只有标准 Web 做不到、或者各平台做法不一致的事情才放进 SDK（全局对象 `xb`），包括生命周期、安全区、持久存储、震动、分享、通知、通讯录等，二十个左右接口。
3. **每个作品一个独立的源（origin）。** 作品之间、作品和小伴之间互相读不到存储和登录状态。这一条是联网之后安全模型的基础，见第 5 节。
4. **移动端优先，并且能在手机上做出来。** 用户只用手机上的小伴 Agent 就能制作和发布作品，见第 11 节。 Android 和 iOS 必须完整实现本文，并且体验一致。网页和桌面是第二档，按第 9 节的范围支持。作品用 `xb.can()` 判断能力是否可用，不按平台名写分支。
5. **命名参考 W3C MiniApp 草案。** 清单字段和生命周期的命名参考 [MiniApp Manifest](https://www.w3.org/TR/2021/WD-miniapp-manifest-20210826/) 和 [MiniApp Lifecycle](https://www.w3.org/TR/miniapp-lifecycle/)。

术语：

- **宿主**：运行作品的客户端，即桌面、Android、iOS 或网页。
- **应用包**：一个作品某一个版本的全部文件。
- **应用域名**：专门用来运行作品、和小伴主站分开的域名，见第 5.1 节。

## 2. 现状：运行时协议 1

协议 1 的规则写在三个地方，内容大体一致：

- 桌面：`client/xiaoban-desktop/src-tauri/src/chat/desktop_tools.rs` 里的 `GAME_FORMAT`
- 手机：`client/xiaoban-multiplatform/common/community-core/.../MobileGames.kt` 里的 `MobileGameFormat.instructions`
- 网页：只有部署说明 `services/server-forum/web/game-sandbox/README.md`，没有写给作者的规则

消息校验分别在桌面 `client/xiaoban-desktop/src/games/protocol.ts`、Android `MobileGameActivity.kt` 和网页 `services/server-forum/web/src/lib/builtinCreation.ts`。

**三个宿主共同的部分**

- 作品跑在 `sandbox="allow-scripts"` 的 iframe 里，源是不透明的（opaque origin），所以 localStorage 这类存储都用不了。
- 用普通 `<script>`，不能联网。
- 握手：宿主发 `host.init`（带 `nonce`、`save`、`safeArea`），作品回 `game.ready`。
- 作品可以发 `game.start`、`game.progress`、`game.result` 报成绩，发 `game.exit` 请求退出。
- 清单里 `save: true` 时，作品可以用 `game.save` 存最多 32 KiB 的 JSON。

**各宿主多出来的消息**

- 桌面还接受 `game.regions`、`game.share`、`game.wallet`、`game.security`，并且在作品没有握手时自动替它回 `game.ready`（`game_packs.rs` 里的 `AUTO_READY`）。
- Android 试玩时接受 `game.document.ready`，用于没有握手的作品；作品出错时宿主会上报 `runtime.error`。

**三个宿主不一致的地方**

| 项目 | 桌面（Tauri） | Android | 网页 | iOS |
| --- | --- | --- | --- | --- |
| 二进制素材 | 只能经 ZIP 导入放进 `assets/`；给 AI 的规则仍要求用 SVG 或 canvas 自己画 | 不支持 | 不支持 | 无宿主 |
| 存档 | 支持 | 支持 | 不支持 | 无宿主 |
| 方向 | 不适用 | 按 `orientation` 锁定 | 官方游戏播放器会尝试全屏后锁定方向；社区试玩不锁定 | 无宿主 |
| `safeArea` | 全 0 | 全 0；竖屏和试玩时用视图内边距避开挖孔，横屏不避开 | 官方播放器传全 0 | 无宿主 |
| 退出 | Esc（宿主注入的 `desktop-host.js`） | 系统返回直接退出；试玩时顶部有一条 44dp 的工具栏 | 页面里的退出按钮 | 无宿主 |
| Worker 和 WebAssembly | 允许 `blob:` 的 Worker；不允许 WebAssembly | 都不允许 | 都不允许 | 无宿主 |
| 社区作品试玩的包体积 | 不适用 | 5 MB 以内（服务端允许 8 MB） | 5 MB 以内 | 无宿主 |
| 社区作品试玩的条件 | 不适用 | 不按审核等级限制，但页面上显示审核等级 | 只运行审核等级为 0 或 1 的作品，报错文字写着「未审核」 | 无宿主 |
| 网页内置游戏从哪里加载 | 不适用 | 不适用 | 论坛自己的域名 | 不适用 |

**协议 1 用到的三种清单写法：**

- 桌面游戏文件夹：`runtimeProtocol: 1`、`entry`、`cover`。
- 手机导出的 ZIP：`schemaVersion: 1`、`kind`、`layer`，没有 `runtimeProtocol`。
- 桌面导出的 ZIP：带素材或者是桌面层游戏时写 `schemaVersion: 2`，见 `resource_packs.rs`。

## 3. 应用包

### 3.1 结构

```
manifest.json      清单，必须放在最外层
index.html         入口（可在清单里改名）
icon.png / .svg    1:1 图标，至少 192×192
cover.png / .svg   16:9 封面，至少 640×360
其余文件           任意目录结构，用相对路径引用
```

作品可以有多个 HTML 页面，见第 6.4 节。

### 3.2 体积、文件类型与路径

| 项目 | 上限 |
| --- | --- |
| 解压后合计 | 100 MB（ZIP 本身也不会超过 100 MB） |
| 文件数 | 3000 个 |
| 单个文件 | 50 MB |
| `manifest.json` | 64 KB |
| 路径 | 最多 8 层；每段只用字母、数字和 `. _ - @`；不以 `.` 开头；不能只靠大小写区分；同一个名字不能既是文件又是目录 |

允许的文件类型如下。不在列表里的扩展名会被拒绝，需要的话可以改名后再打包。

- 文本：html、htm、js、mjs、css、json、svg、md、txt、xml、csv、map、atlas、glsl
- 图片：png、jpg、jpeg、webp、avif、gif、ico
- 音视频：mp3、ogg、wav、m4a、aac、mp4、webm
- 字体：woff、woff2、ttf、otf
- 其他：wasm、glb、gltf、bin、data、ktx2、basis

宿主和应用域名都按扩展名设置 `Content-Type`，例如 `.wasm` 用 `application/wasm`，`.mjs` 用 `text/javascript`。

### 3.3 清单字段

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `schemaVersion` | 整数 | 是 | 写 `3`。`1` 和 `2` 已被协议 1 的包占用（第 2 节） |
| `runtimeProtocol` | 整数 | 是 | 写 `2`。宿主只看这个字段判断协议版本 |
| `kind` | 文本 | 是 | `game` 或 `app` |
| `id` | 文本 | 是 | 2～64 位，只用小写字母、数字、`-`、`_`；和服务端现有的内容包标识规则一致，在所有类型里唯一 |
| `name` | 文本 | 是 | 最多 24 字 |
| `description` | 文本 | 是 | 最多 80 字 |
| `version` | 文本 | 是 | 最多 32 个字符，只含字母、数字、点和连字符，例如 `1.2.0` |
| `category` | 文本 | 是 | 游戏取 `action`、`puzzle`、`board`、`casual`、`story`；应用取 `tool`、`quiz` |
| `entry` | 文本 | 否 | 入口页面，默认 `index.html` |
| `icon` | 文本 | 是 | 1:1 图标的路径 |
| `cover` | 文本 | 是 | 16:9 封面的路径 |
| `orientation` | 文本 | 否 | `portrait`（默认）、`landscape`，或 `any`（跟随手机方向，但遵守系统的方向锁定开关） |
| `input` | 文本列表 | 是 | 支持的操作方式，取 `touch`、`mouse`、`keyboard` |
| `capabilities` | 文本列表 | 否 | 需要的能力，见第 7.2 节 |
| `intents` | 文本列表 | 否 | 最多 8 个、每个最多 12 字，供小伴在对话里推荐作品，例如 `["性格测试", "MBTI"]` |
| `swipeBack` | 布尔 | 否 | 是否允许在 iOS 上从屏幕左边缘右滑返回上一页，默认 `false`；游戏一般不要打开 |
| `layer` | 文本 | 否 | 只有桌面专用的作品写 `desktop`，其余省略 |

示例：

```json
{
  "schemaVersion": 3,
  "runtimeProtocol": 2,
  "kind": "app",
  "id": "mbti-lite",
  "name": "MBTI 性格测评",
  "description": "48 道情境题，约 10 分钟。",
  "version": "1.0.0",
  "category": "quiz",
  "icon": "icon.png",
  "cover": "cover.png",
  "orientation": "portrait",
  "input": ["touch", "mouse", "keyboard"],
  "capabilities": ["haptics", "share"],
  "intents": ["性格测试", "MBTI"]
}
```

协议 1 的 `save` 字段在协议 2 里不用了，持久数据统一用 `xb.storage`（第 7.1 节）。

下面这些信息不写在清单里，由服务端在发布时记录，避免作者填错或冒充：作者、能在哪些设备上运行、是否允许改编、改编自哪个作品、每个文件的哈希。

## 4. 运行环境

作品和普通网站一样运行，有一个真实的 HTTPS 源，下面的东西都可以用。

- **联网**：`fetch`、`XMLHttpRequest`、`WebSocket`，以及外部的脚本、样式、图片和字体都可以用。只允许 HTTPS 和 WSS，不允许访问本机和局域网地址（第 5.3 节）。
- **存储**：`localStorage`、`IndexedDB`、Cache API、OPFS（`navigator.storage.getDirectory()`）都可以用，按作品隔离。这些标准存储可能被系统回收，重要数据请用 `xb.storage`。
- **代码**：ES modules（相对路径、同源的绝对路径、import maps）、Web Worker、WebAssembly 都可以用。前端框架构建出来的默认产物可以直接运行。
- **权限能力**：摄像头和麦克风用标准的 `getUserMedia`，选文件用标准的 `<input type="file">`；这两样都要先在清单里声明，并经过用户同意（第 8 节）。
- **画面**：页面占满宿主给的区域。手机上就是整个屏幕。Android 协议 2 的所有小程序和小游戏统一由宿主在右上角显示竖三点，触控区为 48 × 48 dp。内置、社区和本地创作作品共用这个入口。样式和工具小程序子页导航见[手机端设计规范](../../product/手机端设计规范.md#43-小程序运行页)。

**宿主不提供、或者会拦截的**：

- `alert`、`confirm`、`prompt`。
- `window.open`、离开作品的跳转（第 5.4 节）。
- 定位、剪贴板读取、蓝牙、USB、NFC 这类本文没有列出的系统能力。
- 全屏 API 和屏幕方向 API。方向由清单里的 `orientation` 决定。

**安全区和胶囊**：宿主在作品的每个页面上，赶在页面脚本运行之前，给 `<html>` 元素设好以下 CSS 变量，单位是 CSS 像素：

- `--xb-safe-top`、`--xb-safe-right`、`--xb-safe-bottom`、`--xb-safe-left`：状态栏、挖孔、手势条占用的边距。
- `--xb-capsule-top`、`--xb-capsule-right`、`--xb-capsule-width`、`--xb-capsule-height`：胶囊的位置和大小。

按钮和其他可交互区域必须按胶囊的实际位置和大小避让。选关、对局、暂停、结算和滚动后的控件也要检查。可将滚动容器限制在胶囊下方，或把控件布置在胶囊左侧。同样的数值也放在 `xb.info.safeArea` 和 `xb.info.capsule` 里，数值变化时触发 `safearea` 事件。没有对应元素的宿主（例如桌面没有挖孔）把相应的值设为 0。

**声音**：浏览器要求用户先有一次操作才能播放声音。作品在第一次 `pointerup` 或 `touchend` 时调用 `AudioContext.resume()`，收到 `show` 事件后再调用一次。iOS 宿主的声音遵守手机的静音开关。

## 5. 安全模型

作品可以联网，交给作品的任何数据都可能被发到外部。所以隔离的重点放在作品**拿不到什么**上，同时让平台有办法事后处理问题作品。

### 5.1 每个作品一个源

- 每个作品的地址是 `https://<作品编号>.<应用域名>/v<版本号>/<文件路径>`。同一个作品的不同版本共用一个源，所以升级后数据还在。
- **作品编号**取服务端的内容包编号。本机做的、还没发布的作品用 `local-<本机编号>`。
- **应用域名是 `app.xiangban.xyz`**，作品地址形如 `https://55.app.xiangban.xyz/v3/index.html`。这台服务器只提供作品文件，不提供任何需要登录的接口，也不设置 cookie。证书用 `*.app.xiangban.xyz` 的通配证书，要用 DNS 验证签发。
- **`app.xiangban.xyz` 要申请加入公共后缀列表（Public Suffix List）的私有域名部分。** 加入后，浏览器把每个作品的子域名当成不同的网站：作品之间不能互相写 cookie，作品和 `xiangban.xyz` 下的其他服务也不再是「同一个网站」。申请通过之前，这两条都不成立。
- **`xiangban.xyz` 下的其他服务要遵守两条规则**，否则作品可能借用户的登录状态发请求：
  - 不设置 `Domain=xiangban.xyz` 这种对所有子域名都生效的 cookie。现在都满足：账号中心只用 Authorization 头传凭证，论坛把登录信息存在 `chat.xiangban.xyz` 自己的 localStorage 里。
  - 会修改数据的接口，不能只靠 cookie 的 SameSite 属性防跨站请求，还要检查请求的 Origin 头。
- 存储按「作品编号 + 来源」区分。来源是官方、社区或本机，和清单里的 `id`、本地缓存都无关。删掉缓存的应用包不会删掉数据；用户在设置里选择「清除这个作品的数据」才会删除。

各宿主怎么提供这个源：

- **网页**：应用域名的服务器按上面的地址提供文件；宿主页面用 iframe 嵌入作品。
- **Android**：WebView 打开同样的地址，宿主在 `shouldInterceptRequest` 里拦截这个域名的请求，从本机缓存里返回文件，其他地址照常联网。这样作品在 Android 上的源和网页上一致，也是安全上下文，标准存储和 `getUserMedia` 都能用。
- **iOS**：`WKURLSchemeHandler` 不能接管 `https`，所以先用自定义 scheme（`xbapp://<作品编号>/v<版本号>/`）从本机缓存提供文件。还需要验证这种 scheme 下的页面算不算安全上下文、标准存储能不能持久保存、`getUserMedia`、ES modules、Worker 和 WebAssembly 能不能用（第 15 节）。如果不行，改为直接从应用域名在线加载，靠 HTTP 缓存支持离线。
- **桌面**：用 Tauri 的自定义协议，每个作品一个主机名。

### 5.2 作品拿不到的东西

- 小伴的账号、登录状态、cookie 和接口，以及其他作品的数据。
- 原生接口。宿主只按第 7 节转接列出来的能力，不把原生对象直接暴露给作品。
- 没有经过用户同意的个人数据。凡是要用户同意的能力，同意框里都要写明「这个作品可以把这些内容发送到网络」。

### 5.3 网络限制

- 宿主拦截发往本机和局域网的请求，包括 `localhost`、`*.local`、`127.0.0.0/8`、`10.0.0.0/8`、`172.16.0.0/12`、`192.168.0.0/16`、`169.254.0.0/16`、`::1`、`fc00::/7`，防止作品探测用户家里的设备。
  - Android 在 `shouldInterceptRequest` 里拦截，但这里拦不到 WebSocket。
  - iOS 用 `WKContentRuleList` 拦截。
  - 通过 DNS 把域名解析到内网地址的情况（DNS 重绑定）拦不住，属于剩余风险。
- **按域名封禁。** 宿主从服务端拉一份封禁域名列表，作品访问列表里的域名一律拦截。
- **网络记录。** Android 宿主在每次运行时记录作品访问过的域名和次数（不记完整地址），汇总后上报，用于处理举报。iOS 和网页拿不到逐条请求，只能靠封禁列表和发布时的试运行记录（第 10 节）。
- 作品详情页显示「这个作品会联网」，依据是发布时的试运行记录和 Android 的网络记录。

### 5.4 页面跳转

- 在同一个作品的源里跳转，例如打开包里的另一个 HTML 页面，宿主照常允许。
- 跳到其他网站的顶层页面、`window.open`、`target="_blank"` 的链接，宿主都不在作品里打开，而是弹出「打开外部链接？」，用户同意后交给系统浏览器。
- 作品可以在页面里用 iframe 嵌入其他网站。

### 5.5 下架与封禁

- 服务端可以停用某个作品或者某个版本。宿主每次打开作品时检查停用状态，结果最多缓存 24 小时，作品已停用就显示「这个作品已下架」。
- **禁止联网。** 小伴后台可以对某个作品打开「禁止联网」。宿主打开这个作品时，给它的每个页面加上只允许同源的 CSP（`default-src 'self' data: blob:`），作品只能访问自己包里的文件，fetch、WebSocket、外部图片和脚本都会失败。Android 和 iOS 还会在原生层拦截作品源以外的所有请求。作品不会收到专门的通知，看到的就是普通的断网。WebRTC 不受 CSP 控制，宿主额外去掉 `RTCPeerConnection`，但不能保证完全拦住。这个状态和停用状态一起检查、一起缓存。
- 举报、拉黑和下架沿用社区帖子的流程。
- 联网以后，作品运行时加载的代码可以随时变化，发布时检查过的不等于用户实际运行的。所以平台主要靠举报、下架和域名封禁这些事后手段管理。

## 6. 生命周期

### 6.1 打开

1. 用户点「打开」。本机没有缓存而包超过 20 MB，并且用的是移动网络时，先提示包的大小，用户确认后再下载。
2. 下载支持断点续传，下载完校验 SHA-256，再解压到这个版本自己的目录。
3. 宿主显示启动卡（图标和名称；横屏作品加「横过手机」提示），并按 `orientation` 设好方向。
4. 宿主加载入口页面，在每个页面的脚本运行之前注入 `xb` 和 CSS 变量（第 7.3 节）。
5. 作品画出第一屏后调用 `xb.ready()`，宿主收起启动卡。10 秒内没有调用，宿主显示加载失败页。

### 6.2 运行和退出

- 作品被遮挡或切到后台时，宿主触发 `hide`；回到前台时触发 `show`。作品在 `hide` 里暂停游戏、停掉声音、保存进度。
- **存储的保证**：宿主在卸载页面之前，保证已收到的最后一次 `xb.storage` 写入已经落盘。Android、桌面和网页会在 `hide` 之后再让作品运行大约 500 毫秒，但这只是尽力而为。iOS 可能在切到后台时立刻挂起页面，所以作品要随时保存，不要等到 `hide` 才保存。
- 用户在宿主菜单里选「退出」，或在作品没有可回退的历史时使用 Android 系统返回（第 6.3 节），宿主就退出作品。作品没法阻止退出。
- Android 协议 2 游戏运行页不另设左上角返回按钮。退出由宿主处理，选关等游戏内层级接入浏览器历史。

### 6.3 返回

返回用标准的浏览器历史实现，作品不用调用 SDK。

- **Android 的系统返回**：宿主菜单打开时先关闭菜单。作品有可以回退的历史记录时，宿主执行 `history.back()`，作品会收到 `popstate`，前端框架的路由会自动处理。没有可回退的记录时直接退出作品。
- 为防止作品不停地压入历史、让用户退不出去：
  - 宿主跳过没有用户操作就压入的历史记录（Chromium 本身就这样处理）。
  - 宿主菜单的「退出」总是直接退出。
- **iOS 没有系统返回键。** 默认不加边缘右滑手势，以免和游戏操作冲突；清单写 `swipeBack: true` 的作品，宿主才打开右滑返回上一页。
- **网页**：浏览器的返回按钮本来就会先退 iframe 里的历史，效果和 Android 一样。
- 作品想在「回退一步」时做些额外处理（例如先关掉弹窗），就自己压入一条历史记录，再在 `popstate` 里处理。

### 6.4 多页面

推荐把作品做成单页应用（SPA），用前端框架的路由或者自己切换界面。也可以有多个 HTML 页面：同一个作品的源里的普通跳转都允许，宿主给每个页面都注入 `xb`。`xb.ready()` 只需要在第一个页面里调用。

小程序（例如微信）的做法不一样：在清单里登记所有页面，由宿主维护一个最多 10 层的页面栈，用 `navigateTo`、`navigateBack` 这类接口跳转，每个页面由宿主单独渲染。本规范不采用这种做法，因为单页应用加标准历史记录就是 Web 开发者的习惯做法，宿主也简单得多。

### 6.5 出错

- 宿主自动收集 `window.onerror` 和未处理的 Promise 拒绝。每条只记错误类型、最多 300 字的错误信息、文件名、行号和出现次数，最多每 10 秒上报一次，作者可以在作品后台看到。
- 两种情况下宿主显示失败页，提供「重新打开」「在电脑上打开」「反馈给作者」：一是 `xb.ready()` 之前就出错，二是连续出错导致画面停止更新。

## 7. SDK

宿主在作品的每个页面上注入全局对象 `xb`。作品不要自己打包 SDK 的副本。开发时用的 TypeScript 类型和本地模拟器见第 16 节。

所有异步接口都返回 Promise。失败时的 `error.code` 只有以下几种：

- `unsupported`：这个宿主不支持这项能力
- `not-declared`：清单里没有声明这项能力
- `denied`：用户拒绝了
- `gesture`：这个接口必须由用户点击触发
- `invalid`：参数不对
- `quota`：超出次数或容量限制
- `busy`：同一类请求还没结束
- `login`：需要登录，宿主已经提示用户

### 7.1 基础接口（所有宿主都必须实现）

| 接口 | 说明 |
| --- | --- |
| `xb.info` | 同步对象：`platform`（`android`、`ios`、`desktop`、`web`，只用于统计和排查）、`orientation`、`safeArea`、`capsule`、`theme`（`light`、`dark`）、`locale`、`query`、`legacySave`、`sdkVersion` |
| `xb.info.query` | 从分享链接或通知带进来的参数，是一个扁平的「文本 → 文本」对象。键和值只能用字母、数字、`-`、`_`，合计不超过 1 KB。作品不要把它当 HTML 插进页面 |
| `xb.info.legacySave` | 同一个作品以前的协议 1 版本留下的存档，没有就是 `null`。读一次，迁移到 `xb.storage` |
| `xb.ready()` | 第一屏画好后调用一次 |
| `xb.on(name, fn)`、`xb.off(name, fn)` | 事件：`show`、`hide`、`safearea`、`orientation`、`theme` |
| `xb.can(name)` | 同步返回这个宿主当前有没有某项能力（第 7.2 节的名称）；清单里没声明的能力一律返回 `false` |
| `xb.exit()` | 请求退出，效果等同于点 ✕ |
| `xb.storage.get(key)`、`set(key, value)`、`remove(key)`、`keys()` | 宿主保管的持久存储，系统不会清掉。每个作品合计最多 10 MB，键最多 128 个字符，值是 JSON。只存在这台设备上 |
| `xb.game.start()`、`progress({score})`、`result({score})` | 报成绩，`score` 是非负整数，规则和协议 1 相同 |

### 7.2 能力接口（要在清单的 `capabilities` 里声明）

| 能力 | 用法 | 用户同意 | 说明 |
| --- | --- | --- | --- |
| `haptics` | `xb.haptics.impact("light" \| "medium" \| "heavy")`、`xb.haptics.notify("success" \| "warning" \| "error")` | 不需要 | 遵守系统和小伴设置里的震动开关 |
| `share` | `xb.share({title, text, image, query})` | 用户点击触发 | 分享卡片由宿主生成：作品封面、标题、最多 60 字的文字，加一张可选图片（PNG 或 JPEG，最大 2 MB）。文字和图片先过内容审核再发出。链接带上 `query`，接收方打开后作品从 `xb.info.query` 读到 |
| `clipboard` | `xb.clipboard.writeText(text)` | 用户点击触发 | 只能写不能读，最多 10000 字 |
| `screen` | `xb.screen.keepOn(true \| false)` | 不需要 | 保持屏幕常亮，只在作品显示在前台时有效 |
| `notify` | `xb.notify.schedule({id, at, title, body, repeat})`、`cancel(id)`、`list()` | 第一次使用时 | 本地定时提醒，作品关掉后也会到点通知。每个作品最多 10 条待发提醒；标题最多 20 字，正文最多 60 字；通知上会带作品名称；`repeat` 取 `daily` 或省略；到点时间可能有几分钟误差。点通知打开作品，`query` 里带上提醒的 `id` |
| `profile` | `xb.user.profile()` | 每个作品单独同意 | 返回昵称、头像和 `openId`。`openId` 由服务端用 HMAC(账号编号, 作品编号) 算出来，同一个人在不同作品里拿到的值不一样，作品可以拿它在自己的服务器上做排行榜 |
| `camera`、`microphone` | 标准的 `navigator.mediaDevices.getUserMedia()` | 每个作品单独同意，再走系统权限 | 使用期间胶囊上显示一个提示点 |
| `contacts` | `xb.contacts.pick({multiple, fields})` | 每次都由用户在系统选择器里挑 | `fields` 取 `name`、`phone`、`email`。只返回用户选中的联系人，不能读取整个通讯录 |
| `files` | 读文件用标准的 `<input type="file">`；保存用 `xb.files.save({name, mime, data})` | 每次都由用户在系统面板里操作 | 读：用户挑选文件。写：系统的保存面板，单个最大 100 MB。作品的私有文件用标准存储（OPFS 或 IndexedDB） |
| `windows`、`pointer` | 协议 1 的桌面层能力 | 不需要 | 只能用在 `layer: "desktop"` 的作品上，只有桌面宿主支持 |

规则：

- 清单没声明的能力，宿主直接拒绝。对摄像头和麦克风，宿主不响应 WebView 的权限请求；对 SDK 接口，返回 `not-declared`。
- 用户的同意按作品记录，可以在「小伴设置 → 应用权限」里撤销。
- **所有限制都在宿主那一侧执行**，包括能力声明、次数、大小，以及「必须由用户点击触发」。作品可以绕过 SDK 直接给宿主发消息，所以不能指望 SDK 里的检查。判断是不是用户点击：原生宿主看最近一次触摸是不是在 1 秒以内；网页宿主看 `navigator.userActivation`。

#### Android 轻启专用接口（开发中）

Android 宿主为「轻启」提供 `xb.quickLaunch.available(name)`、`xb.quickLaunch.open(name)` 和 `xb.quickLaunch.pin(name)`。入口脚本必须与宿主内置的轻启入口逐字节一致；非该作品、未知目标、非前台或缺少近期用户手势时拒绝调用。宿主根据固定白名单选择目标包名与 URI，不接收作品提供的任意 Intent。创建桌面图标交给系统桌面确认，`pin` 成功返回表示宿主已确认该快捷方式处于固定状态。当前白名单包含 21 个常用应用的候选入口，实际首页和广告效果因应用版本而异；不属于通用协议 2 能力。

### 7.3 宿主怎么注入 SDK

- **Android**：用 `WebViewCompat.addDocumentStartJavaScript` 注入 `xb` 和 CSS 变量，只对作品的源生效；用 `WebViewCompat.addWebMessageListener` 接收消息，只接受作品的源发来的消息。单条消息最少要能传 4 MB，二进制数据用 ArrayBuffer 传。现有实现会丢掉超过 40 KiB 的消息，要改。
- **iOS**：用 `WKUserScript`（`atDocumentStart`，只注入主页面）注入，用 `WKScriptMessageHandler` 接收消息。
- **网页**：宿主不能往别的源的 iframe 里注入脚本，所以由应用域名的服务器在每个 HTML 文件的 `<head>` 最前面插入 `<script src="/__xb/sdk.js"></script>`。SDK 用 `postMessage` 和宿主页面通信，两边都检查 `event.origin`，对方的源不对就忽略消息。
- 消息格式：
  - 作品发给宿主：`{channel: "xiaban.app", protocol: 2, kind: "call", id, method, params}`
  - 宿主回复：`{channel: "xiaban.host", protocol: 2, kind: "result", id, ok, value | error}`
  - 宿主主动通知：`{channel: "xiaban.host", protocol: 2, kind: "event", name, data}`
- 有了真实的源以后，协议 2 不再需要协议 1 的 `nonce`。

### 7.4 实现状态

| 部分 | 状态 |
| --- | --- |
| 协议 1 的握手、成绩、存档 | 历史兼容路径仍见桌面和网页；Android 正式宿主已删除协议 1 |
| Android 作品源、联网、标准存储、ES modules、Worker、WebAssembly | 正式宿主已实现，摄像头、麦克风与网络过滤已真机验证 |
| 桌面作品隔离与联网 | 本地协议 2 容器已实现；Tauri 自定义协议的主机名固定，桌面以每作品独立 WebView 数据存储、按窗口校验文件请求和限制导航实现隔离；Windows 与 macOS 14+ 的 HTTPS/WSS 经本机过滤代理。桌面暂不支持外部 iframe 和 WebRTC；旧版 macOS 离线运行且标准 Web 存储为临时存储。仍需安装包实测 |
| 网页作品源与联网 | 试玩前端和存储桥已实现；独立作品域名文件服务、证书和线上联动待完成 |
| `xb` 基础接口、CSS 变量、`show`/`hide`、返回 | Android 正式宿主已实现基础部分；桌面和网页仅实现当前桥接代码中的子集，按第 9 节验收仍待完成 |
| `haptics`、`share`、`clipboard`、`screen`、`camera`、`microphone`、`files` | Android 摄像头和麦克风已实现；其他能力按宿主逐项待实现 |
| `notify`、`profile`、`contacts` | 待实现 |
| iOS 宿主 | 未实现 |

实现顺序和未完成项见 [progress.md](progress.md)。iOS 宿主暂缓（2026-11-08 决定）。

## 8. 权限

需要用户同意的能力，第一次使用时宿主弹出同意框。

- **同意框的写法**：「《作品名》想使用摄像头。作品可以把拍到的内容发送到网络。」下面两个按钮：「允许」「不允许」。其他能力照这个格式写，把「摄像头」和「拍到的内容」换成对应的说法。
- **系统权限**：小伴自己还没有系统权限时，用户在小伴的同意框里点了允许，宿主再申请系统权限。用户拒绝系统权限，作品收到 `denied`。
- **记录和撤销**：同意按作品记录，在「小伴设置 → 应用权限」里可以逐个撤销。撤销后，作品下次调用时重新询问。
- **iOS**：摄像头和麦克风用 `WKUIDelegate` 的 `requestMediaCapturePermissionFor` 处理，避免系统弹窗和网页弹窗各弹一次。

## 9. 多端范围

**Android 和 iOS 之间必须一致。** 两者允许的差别只有两种：一是某项能力在某个平台上确实做不到，这时 `xb.can()` 返回 `false`；二是渲染引擎不同（Android 是 Chromium，iOS 是 WebKit），这一点和普通网页开发一样，发布时在两种引擎上各试运行一遍。

**网页和桌面是第二档。** 包格式、运行环境、生命周期和基础接口与手机一致，同一个作品不用改就能打开。下表列出它们和手机的差别。标「不支持」的能力，要么平台做不到，要么做起来明显更复杂，按移动端优先的原则先不做。

| 项目 | 网页 | 桌面 |
| --- | --- | --- |
| 方向 | 先尝试进入全屏并锁定方向（现在官方游戏播放器就是这样做的）；锁不了就在方向不对时盖一层「横过手机」提示 | 不适用，横屏作品用横向窗口 |
| 尺寸 | 按可见区域（`visualViewport`）定尺寸，浏览器地址栏伸缩时更新 `safearea` | 窗口大小 |
| 返回 | 浏览器的返回按钮（第 6.3 节） | Esc 先打开菜单，再按一次退出 |
| `xb.storage` | 存在宿主自己的源里，只在这个浏览器里有效 | 支持 |
| `haptics`、`notify`、`contacts` | 不支持 | 不支持 |
| `share` | 用浏览器的分享面板，没有就复制链接 | 复制链接 |
| `clipboard`、`screen` | 支持 | 支持 |
| `camera`、`microphone` | 支持（iframe 上加 `allow="camera; microphone"`） | 支持 |
| `files` | 读文件支持；保存改为浏览器下载 | 支持 |
| `profile` | 支持，需要登录 | 支持 |
| 桌面层（`windows`、`pointer`） | 不支持 | 支持 |
| 在哪里展示作品 | 清单 `input` 含 `touch` 或 `mouse` | 清单 `input` 含 `mouse` 或 `keyboard` |

**宿主一致性测试。** 用一组专门的测试作品覆盖：握手和生命周期、存储往返、每项能力、权限同意框、网络限制、安全区数值、退出前写盘、返回和历史记录。每个宿主发布前都要跑一遍。Android 和 iOS 要求全部通过；网页和桌面只要求上表标为支持的部分通过，不支持的能力要正确返回 `xb.can() === false`。

现有的这几处测试要合并进这一套：

- `client/xiaoban-multiplatform/androidApp/scripts/test-mobile-game-host.cjs`
- `client/xiaoban-desktop/src/games/protocol.test.ts`
- `services/server-forum/web/tests/` 里的试玩测试

## 10. 发布与检查

作品发布到社区时，服务端检查应用包，并把结果记在这个版本上。客户端按检查结果决定在哪些设备上展示作品，不采信作者自己的声明。

**静态检查**（不通过就不能发布）：

- 清单的字段和取值符合第 3.3 节。
- 文件类型、数量、体积和路径符合第 3.2 节。
- 图标、封面和入口页面都存在。

**静态提示**（不阻止发布，只提醒作者）：

- 用了 `alert`、`confirm` 或 `prompt`。
- 引用了 `http://` 地址，或者局域网、本机地址。
- 用到了某项能力却没在清单里声明，例如调用了 `getUserMedia` 但没写 `camera`。这类检查很容易绕过，只作提示；真正的限制在运行时由宿主执行。

**试运行**：

- 在隔离的容器里用无头 Chromium 和无头 WebKit 各跑一遍。容器可以访问外网，但不能访问服务器所在的内网。Linux 上的 WebKit 和 iOS 的 WKWebView 不完全一样，所以 WebKit 的结果只作参考。
- 按清单的方向选画面尺寸：竖屏 390×844，横屏 844×390。
- 检查内容：
  - 10 秒内有没有调用 `xb.ready()`。
  - 前 15 秒自动点几次，期间有没有出错。
  - 点击后画面有没有变化，用来判断是否支持触控。
- 截三张图作为默认截图，并记录作品访问了哪些外部域名。

**记录在版本上的结果**：

- 能在哪些平台运行（`android`、`ios`、`desktop`、`web`）
- 是否支持触控
- 从打开到调用 `xb.ready()` 用了多长时间
- 协议版本
- 包大小
- 访问了哪些外部域名
- 每个文件的哈希，为以后只下载有变化的文件做准备

**服务端在发布时记录、不写进清单的信息**：

- 作者。
- 是否允许改编，默认允许。
- 改编自哪个作品的哪个版本。改编作品发布时自动带上「改编自 @原作者」。

**审核状态不影响展示和运行。** 社区作品和普通帖子一样，任何人都能看到、能打开，界面上也不显示审核状态。现有代码有两处要改，见第 16 节：网页社区试玩只运行审核等级为 0 或 1 的作品，Android 在页面上显示审核等级。

## 11. 在手机上制作与发布

这是 P0 要求：用户只用手机上的小伴 Agent，就能从一个想法做出作品、试用、改好，再发布到社区。

### 11.1 两种开发方式

两种方式做出来的应用包完全一样，宿主分不出来。

- **免构建**：源码就是实际运行的代码，用原生 JavaScript、ES modules 和 import maps，不需要 npm 和打包工具。手机上没有 Node.js，所以手机 Agent 做的作品都用这种方式；AI 生成和新手入门也推荐用它。免构建作品的源码可以直接读、直接改，「做一个我的版本」的效果最好。
- **需要构建**：开发者在电脑上用 Vite、TypeScript、Vue 单文件组件、React JSX，或者用 Cocos、Unity 导出，上传的是构建后的文件。这类作品被改编时，小伴只能拿到构建产物，改起来比较吃力。

### 11.2 官方库

免构建也要能用上界面框架和游戏引擎。小伴内置一批固定版本的库，Agent 用 `lib_add` 把需要的库复制进作品的 `lib/` 目录。这样作品不依赖外部 CDN：没网也能用，不受国外 CDN 在国内是否稳定的影响，库的版本也不会被人悄悄改掉。

Android 已提供 Three.js r128 的经典脚本版本。`lib_add(library="three", source_dir="drafts/my-game")` 会把它复制为作品内的 `lib/three.min.js`，由普通 `<script src>` 加载全局 `THREE`。以下是库规划。

| 用途 | 库 | 说明 |
| --- | --- | --- |
| 界面 | Preact + htm | 用模板字符串写组件，不需要 JSX 编译 |
| 界面 | petite-vue | 在 HTML 上直接写 Vue 风格的绑定 |
| 2D 游戏 | PixiJS、Phaser | |
| 3D | three.js | 暮色滑翔用的就是它 |

计划中的库提供可以直接 `import` 或用 `<script src>` 加载的文件。每个库固定版本并保留来源和许可证。

### 11.3 手机 Agent 的工具

现有的工具（`client/xiaoban-multiplatform` 的手机 Agent）：

- 文件：`read_file`、`write_file`、`copy_file`、`edit_file`、`delete_file`、`list_dir`、`grep`
- 作品：`app_read_format`、`app_list`、`app_validate`、`app_create`、`app_open`、`app_runtime_read`、`app_export`
- 联网：`web_search`、`web_fetch`

下表涵盖协议 2 的现有接口和后续规划。

| 工具 | 作用 |
| --- | --- |
| `app_read_format` | 返回当前平台支持的创作规范 |
| `app_create`、`app_validate` | 按协议 2 安装和校验游戏或小程序 |
| `app_open`、`app_close`、`app_runtime_read` | 在手机上打开作品试用；读运行错误、`xb.ready()` 用时和访问过的域名 |
| `app_snapshot` | 给正在运行的作品截一张图交给模型，让 Agent 自己检查画面 |
| `lib_add` | 把第 11.2 节的官方库复制进作品 |
| `asset_image` | 用小伴 AI 生成一张图片存进作品，图标、封面、角色、背景都可以用。点数照常扣，生成前告诉用户大约花多少点 |
| `asset_speech` | 用手机上的朗读引擎把一段文字生成音频文件，给测评和故事类作品配旁白 |
| `asset_import` | 请用户从相册或文件里挑一个素材放进作品，由用户自己操作 |
| `app_publish` | 打包并上传，填好发布页：标题、简介、标签、封面、是否允许改编。**停在确认页，由用户点「发布」。** 发布是对外公开的动作，Agent 不能替用户点 |
| `app_check_read` | 读服务端对这个版本的检查结果（第 10 节），没通过就改好后重新发布 |

Android 已接入 `lib_add` 和 `asset_import`。`app_create` 从草稿目录安装源码和素材，`icon` 与 `cover` 可指定包内图片。Android 本地创作的上限为单个源码 1 MB、单个素材 10 MB、作品文件合计 24 MB、工作区合计 64 MB。社区作品包使用第 3.2 节的上限。

### 11.4 在手机上的流程

1. 用户说出想法。Agent 先给一个简短方案，写明是游戏还是工具、竖屏还是横屏、要用哪些能力，用户确认后再动手。
2. Agent 用文件工具写代码，按需要用 `lib_add` 加库、用 `asset_image` 生成图片，然后用 `app_validate` 检查。
3. Agent 用 `app_open` 打开作品，用 `app_runtime_read` 和 `app_snapshot` 自己检查一遍；用户也亲手试玩，再说哪里要改。
4. 用户满意后，Agent 用 `app_publish` 打开填好的发布页，用户检查一遍后点「发布」。
5. 服务端检查完，作品出现在社区作品里。Agent 用 `app_check_read` 读检查结果，有问题就告诉用户并修改。

iOS 宿主做好之前，iOS 上的小伴 Agent 不能制作作品。

### 11.5 手机上做不了的

- 需要构建步骤的项目（第 11.1 节），由开发者在电脑上做。
- 3D 模型、成套音乐这类 AI 生成不了的素材。用户手里已经有的，可以用 `asset_import` 导入。

## 12. 内容与合规

- **不卖东西。** 作品里不能售卖数字商品，也不能引导用户去外部付款（App Store 审核指南 3.1）。游戏里靠玩赚到的虚拟币，比如小伴贪吃蛇的糖果币，可以保留。
- **广告**不做专门限制。作品有问题时，后台可以禁止它联网或者把它下架（第 5.5 节）。
- **AI。** 小伴不给作品提供 AI 接口。作品要用 AI，就自己联网调用自己的服务，费用由作者承担。
- **不收集身份信息。** 不能让用户填手机号、身份证号这类身份信息。作品要用 `profile`、通讯录、摄像头和麦克风，都必须经过第 8 节的同意流程。
- **年龄分级先不做。** 风险：App Store 审核指南 4.7.5 要求 App 能让用户分辨超出 App 年龄分级的作品，并按用户的年龄限制未成年人使用。iOS 上架前要再评估一次。
- **不申请版号。** 风险：国内应用商店对游戏有版号要求，社区游戏多了以后，安卓应用商店的审核可能会问到。
- **举报和下架。** 沿用社区帖子的举报、拉黑和下架流程，并加上第 5.5 节的停用和域名封禁（App Store 审核指南 4.7.1）。
- **作品索引。** 应用广场就是全部作品的索引，每个作品都有可以直接打开的通用链接（App Store 审核指南 4.7.4）。
- **转接原生能力的风险。** App Store 审核指南 4.7.2 要求，没有苹果的事先许可，不能把原生平台的接口开放给 App 里运行的作品。震动、通知、通讯录、保存文件这些都是宿主转接的能力，是否需要苹果许可还没有确认。iOS 第一个版本可以先只上标准 Web 能力和风险最小的几项能力，并在提交审核时主动说明。
- **改编授权。** 作者发布时打开「允许改编」，就是同意其他用户在小伴里基于这个作品的源码做衍生作品；衍生作品会自动署名原作。原作删除或下架后，已经发布的改编作品保留，署名也保留。这一点写在「允许改编」开关下面，作者打开开关时就能看到。

## 13. 版本与兼容

- **判断协议版本只看 `runtimeProtocol`。** 没有这个字段的包按协议 1 处理，协议 2 的包必须写 `runtimeProtocol: 2`。`schemaVersion` 只表示清单格式：`1` 和 `2` 是协议 1 的几种写法（第 2 节），`3` 是协议 2。
- **协议 1 的作品在新宿主上照常运行。** 宿主按协议 1 加载它：用不透明源的 iframe，不联网，不注入 `xb`，第 2 节列出的消息全部照旧支持，桌面继续注入 `desktop-host.js`。
- **旧版小伴看不到协议 2 的作品。** 客户端请求时带上请求头 `X-Xiaoban-Runtime: 1,2`，服务端只把客户端支持的协议版本的作品返回给它。旧版客户端没法显示「更新后可用」，因为它根本不知道有这种作品。
- **协议 1 的作品放进应用广场时，缺的字段按下面补：**
  - 图标：从封面中间裁一块。
  - 分类：`casual`。
  - 操作方式：`touch`、`mouse`、`keyboard` 都算支持，因为协议 1 的规则要求同时支持触控和键盘。
  - 版本：用服务端的版本号。现在手机导出的 ZIP 没有写 `version`，而服务端要求必须有，所以导出要补上。
- **协议 2 之内只增加、不删除接口。** 作品用 `xb.can()` 判断，用 `xb.info.sdkVersion` 排查问题。需要破坏兼容的改动升到协议 3。

## 14. 待定问题

目前没有。2026-11-08 已定下的取舍写在文首。

## 15. 先要验证的技术问题

这些问题决定实现方式，应该在动手实现之前先做小样验证：

1. **iOS 自定义 scheme（iOS 暂缓，做 iOS 前再验证）。** 用 `xbapp://` 加载的页面，是不是安全上下文；`localStorage`、IndexedDB、OPFS 能不能持久保存，会不会被 WebKit 的防跟踪机制定期清掉；`getUserMedia`、ES modules、Worker、WebAssembly 能不能用；`WKContentRuleList` 能不能拦住发往局域网的 WebSocket。
2. **Android WebView 版本。** `addDocumentStartJavaScript` 和 `addWebMessageListener` 需要比较新的 WebView。要统计用户设备上的 WebView 版本，并决定太旧时怎么办。
3. **网页返回。** 在 iOS Safari 和 Android Chrome 上，确认浏览器的返回按钮会先回退跨源 iframe 里的历史记录。
4. **100 MB 包的体验。** 在参考设备上测下载、校验、解压的时间和占用的空间，并定下本机缓存的总上限（建议 1 GB，最久没用的先删）。

## 16. 维护与配套

**唯一来源。** 本文是规范的唯一来源。第 3.2 节的上限、第 3.3 节的字段和第 7 节的接口有改动时，要同步修改下面这些地方：

- 桌面：`game_packs.rs`、`resource_packs.rs`、`desktop_tools.rs`
- 手机：`MobileGames.kt`、`MobileGameActivity.kt`
- 服务端：`services/server-forum/src/content.rs`
- 网页：`game-sandbox/`
- 给 AI 的规则：[ai-brief.md](ai-brief.md)

计划把上限和字段抽成一份 `limits.json` 和清单的 JSON Schema，各端都从这里读，避免手抄出错。

**实现协议 2 时要改的现有代码**：

- 服务端：
  - 新增 `app` 内容类型，`game` 和 `app` 的大小上限提高到 100 MB，支持断点续传。
  - 按 `X-Xiaoban-Runtime` 请求头过滤作品。
  - 记录每个文件的哈希。
  - 新增停用作品、禁止作品联网和封禁域名的接口，以及后台对应的开关。
  - 新建应用域名的文件服务，负责注入 SDK。
  - 名称上限（现在 60 字）按作品类型收紧到 24 字。
- 网页：社区试玩不再按审核等级拦截，去掉「未审核」的报错文字；包大小上限跟着服务端提高。
- Android：页面上不再显示审核等级，发布页「新版本会重新进入审核」的说明也要改；手机 Agent 按第 11.3 节新增工具，工作区支持二进制文件并把合计上限提高到 100 MB；手机导出的 ZIP 补上 `version`；消息大小限制从 40 KiB 改为至少 4 MB；本机缓存从「保留 8 个 ZIP」改为按总容量清理。
- 桌面：带素材的 ZIP 导出继续写 `schemaVersion: 2`，不受影响。

**给开发者的配套工具**（都还没做）：

- **起步模板**：`npm create xiaoban-app`，可选 Vue 3、React、Preact、原生 JS 和 Phaser，都用 Vite 构建。工具和测评推荐 Vue 3 或 Preact；2D 游戏推荐 Phaser 或 PixiJS；3D 游戏推荐 three.js。
- **`@xiaoban/sdk`**：`xb` 的 TypeScript 类型定义；在没有宿主的普通浏览器里打开时，提供一套模拟实现。
- **本地宿主模拟器**：在浏览器里模拟手机外框、胶囊、安全区、横竖屏、`show`/`hide`、返回键、权限同意框和存储。
- **命令行工具**：一条命令打包 ZIP，并运行第 10 节的静态检查。
