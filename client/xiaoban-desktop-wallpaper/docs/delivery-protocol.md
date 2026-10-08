# 壁纸与更新下发协议 v1

本协议属于 `client/xiaoban-desktop-wallpaper`，供内容提供方实现 HTTP 接口。当前支持 Windows/macOS，使用 UTF-8 JSON；地址在应用的“动态壁纸 → 添加订阅”和“关于小伴 → 更新服务设置”中配置。服务地址尚未确定，不内置假地址，也不把“没有配置服务”当成“已是最新版本”。

## 壁纸目录

`GET <catalogUrl>` 返回 HTTP 200 和 `Content-Type: application/json`。清单不超过 1 MiB，最多 100 项。`schemaVersion` 必须为 `1`，`revision` 是提供方自己的修订标识。

```json
{
  "schemaVersion": 1,
  "revision": "2026-10-08-1",
  "items": [
    {
      "id": "mint-garden",
      "version": "1.0.0",
      "title": "薄荷花园",
      "description": "给桌面留下一点轻盈的绿意。",
      "author": "小伴",
      "kind": "canvas",
      "resource": {
        "url": "https://YOUR-CDN.example/mint-garden.js",
        "sha256": "REPLACE_WITH_64_LOWERCASE_HEX_CHARACTERS",
        "bytes": 12345,
        "mime": "application/javascript"
      },
      "assets": {},
      "controls": [
        { "key": "accent", "label": "光晕颜色", "type": "color", "default": "#78b8a4" },
        { "key": "speed", "label": "流动速度", "type": "range", "default": 0.6, "min": 0.1, "max": 1.5, "step": 0.1 }
      ]
    }
  ]
}
```

上例里的域名、SHA-256 和字节数是说明占位值，**不能直接作为有效清单导入**。实际可运行的内置样例见 [catalog.json](../public/wallpapers/builtin/catalog.json)，它的 `builtin://` 地址仅供应用内部使用，服务清单必须提供 HTTPS 地址。

| 字段 | 约束 |
| --- | --- |
| id | 1–64 个 ASCII 字母、数字、`-`、`_`、`.`；清单内唯一；`companion`、`builtin-` 和 `local-` 前缀保留 |
| version | SemVer，例如 `1.0.0`；同一内容更新时增加版本，改变资源校验值 |
| title / description / author | 纯文本；客户端会转义，不接收 HTML |
| kind | `image`、`video`、`canvas` |
| resource | 图片本身、MP4 本身，或 Canvas 的 ES module 入口 |
| cover | 可选图片资源，最多 2 MiB；缺少或下载失败时显示默认封面 |
| assets | Canvas 附属资源字典，最多 32 项；资源名称不能为 `__proto__`、`constructor`、`prototype` |
| controls | Canvas 参数，最多 16 项；非 Canvas 不接受此字段的非空值 |

每个资源包含 `url`、`sha256`、`bytes`、`mime`。SHA-256 为实际文件完整内容的 64 位小写十六进制；`bytes` 为实际文件字节数，不是字符数。单张壁纸入口及附属资源总量不超过 256 MiB；动画 JS 每个文件不超过 2 MiB。HTTPS 重定向最多 4 次，不接受 `file:`、`javascript:`、URL 内的账户密码或片段。本机接口测试可使用 `http://localhost`、`http://127.0.0.1`、`http://[::1]`。

资源建议返回 `Content-Type`、`Content-Length`，保持 `Accept-Encoding: identity` 的原始文件内容，以免文件大小及 SHA-256 与服务端压缩传输不一致。文件下载到应用自己的缓存目录，校验完成后才发布到缓存；不把下发路径作为本地文件路径。失败时保留之前正在使用的壁纸。资源缓存在本机后，离线仍可应用和播放。

应用的本地文件导入独立于下发协议：从原生文件选择器选择图片或视频，复制到用户目录 `.xiaobanwallpaper/wallpapers/imports`，元数据保存在 `local.json`。内部 `local://<sha256>` 和 `local-` ID 仅由应用生成，服务清单不可使用，也不接受本地文件路径。正式数据目录与迁移规则见 [user-data.md](user-data.md)。

### 图片和视频

- 图片：`kind: "image"`，推荐 PNG、JPEG、WebP。`resource.mime` 使用 `image/png`、`image/jpeg`、`image/webp`。不要下发 SVG 作为外部资源。
- 视频：`kind: "video"`，`mime: "video/mp4"`，推荐 H.264、`yuv420p`、MP4 fast-start。版本 1 固定静音、循环播放；不支持音频控制、播放列表或 HLS。
- 图片/视频的客户端选项为 `fit: "cover" | "contain" | "fill"`、`background: "#RRGGBB"`；视频速度 `playbackRate` 范围为 `0.25–2`。服务提供内容，用户的显示选择独立保存在本地。
- 本机缓存协议实现 HTTP byte ranges，视频可以分段读取和寻址。视频是否可解码仍取决于系统的媒体能力；失败会报告解码错误，不显示成功状态。

## Canvas SDK

入口是自包含的 JavaScript ES module，默认导出 `create(context)`，允许返回 Promise。Canvas 可使用 2D 或 WebGL。入口预先打包，不从网络加载 npm/CDN 依赖；图片、JSON、二进制或附属 JS 通过 `assets` 下发。

```js
export default function create({ canvas, width, height, dpr, settings, assets }) {
  const ctx = canvas.getContext('2d')
  let options = settings
  return {
    render({ time, dt }) {
      ctx.fillStyle = options.accent || '#78b8a4'
      ctx.fillRect(0, 0, canvas.width, canvas.height)
    },
    update(nextSettings) { options = nextSettings },
    resize({ width, height, dpr }) {},
    destroy() {}
  }
}
```

- `canvas.width/height` 是像素尺寸；`width/height` 是 CSS 尺寸；`dpr` 上限 2。
- `render` 必须提供，`time`、`dt` 单位均为秒。不要自建永久计时器；宿主负责帧率、暂停和释放。
- `settings` 是控制项的当前参数。可选 `update`、`resize`、`destroy` 对应参数、窗口变化和退出。
- `assets[name]` 为 `{ url, mime, bytes }`，`url` 是沙箱自己生成的 Blob URL，`bytes` 为 ArrayBuffer。可通过 `Image` 加载图片，或读取 Blob/ArrayBuffer 解析 JSON、纹理、模型。附属 JS 可用 `import(assets.module.url)`。
- 脚本运行在独立的低权限壁纸 WebView 内的 opaque-origin iframe。沙箱不允许外部网络、弹窗、顶层导航、原生系统命令或访问设置页。壁纸窗口的应用 IPC 仅允许报告场景就绪，不能设置自启动、退出应用、导入文件或打开下载网址。
- 模块不是独立网页，不能通过 `window.parent` 读取宿主，也不能依赖 Tauri API。提供方应控制脚本的 CPU/GPU 使用；权限隔离不等于对任意计算量的资源配额。

### 可调参数

`controls` 的 `key` 唯一，并与 `settings[key]` 对应；`label` 为显示名称，`default` 必须匹配类型。支持：

| type | default | 额外字段 |
| --- | --- | --- |
| range | 数字，位于范围内 | 有限数值 min/max/step，min < max，step > 0 |
| color | `#RRGGBB` | 无 |
| boolean | true/false | 无 |
| select | 某个选项字符串 | options：1–16 个非空字符串 |

## 应用更新检查

`GET <updateUrl>?currentVersion=0.1.0&target=windows-x86_64`。已有查询参数会保留。目标值当前为 `windows-x86_64`、`darwin-aarch64`；协议也预留 `darwin-x86_64`，但本项目尚未提供 Intel 包。

HTTP 204 表示没有更新；HTTP 200 返回如下 JSON，版本按 SemVer 比较，不按字符串排序：

```json
{
  "schemaVersion": 1,
  "version": "0.2.0",
  "notes": "新版设置界面与动态壁纸体验优化。",
  "publishedAt": "2026-10-08T08:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://YOUR-CDN.example/XiaoBan-Windows-x64.zip",
      "sha256": "REPLACE_WITH_64_LOWERCASE_HEX_CHARACTERS",
      "bytes": 1234567,
      "mime": "application/zip"
    },
    "darwin-aarch64": {
      "url": "https://YOUR-CDN.example/XiaoBan-macOS-arm64.zip",
      "sha256": "REPLACE_WITH_64_LOWERCASE_HEX_CHARACTERS",
      "bytes": 1234567,
      "mime": "application/zip"
    }
  }
}
```

字段约束同资源对象；更新包最多 60 MiB。比当前版本更新时，必须提供当前平台的包，否则提示未提供安装包。`notes` 为纯文本，最多 16000 字节。

版本 1 实现“检查更新 → 展示版本/说明 → 在浏览器打开下载地址”，**不会静默执行安装包或覆盖程序**。如需自动升级，另行接入带公钥、签名校验和正式签名安装包的更新机制；仅有 SHA-256 不等于发布者签名。当前 Windows 便携 EXE 与 macOS ad-hoc `.app` 的打包状态不改变。

## 接口与内容方自检

JSON Schema：[壁纸目录](wallpapers-v1.schema.json)、[更新清单](updates-v1.schema.json)。JSON Schema 检查基础结构；ID 唯一性、SemVer、参数默认值/范围关系、UTF-8 字节长度和资源总量等跨字段约束由客户端进一步验证。PNG/JPEG/WebP 是跨端推荐格式；AVIF 能否解码取决于系统 WebKit/WebView 版本。

1. 计算原始资源的实际大小和 SHA-256，再填写目录。
2. JS 以 ES module 导出 SDK 方法，避免网络依赖及原生接口。
3. 在应用中导入清单或添加 HTTPS 订阅，确认预览、参数和应用效果。
4. 断开网络后检查已下载内容；替换资源但不改校验值应被拒绝。
5. 更新接口分别检查 Windows 与 Apple silicon 目标，未提供的目标不要伪造下载地址。

协议解析、资源约束与版本比较的实现见 `src-tauri/src/library.rs`；交付窗口与权限边界见 `src-tauri/src/content.rs`；可运行本地测试服务见 `scripts/wallpaper-fixture.mjs`。
