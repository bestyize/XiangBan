# 小伴桌面资源包格式

小伴桌面的自创内容可以打成一个普通的 **zip 文件**分享、备份和导入。这份文档写给想在别的工具（Claude Code、Cursor、脚本、手工）里制作资源包的人：照着做出 zip，在小伴桌面对应页面点“导入资源包”就能用。

小伴桌面里的“导出资源包”和“分享到论坛”生成的也是同样的 zip，反过来也能当作样例解压看看。

## 能做什么

| 类型 | `kind` | 是什么 | 内容 | 导入位置 |
| --- | --- | --- | --- | --- |
| 动态壁纸 | `wallpaper` | 铺满桌面的 HTML 动画或互动页面 | 一个自包含的 `index.html` | 壁纸页，“我的壁纸” |
| 桌面特效 | `effect` | 铺在桌面上的粒子层（雨、雪、花瓣、烟花等） | 一个 JSON 描述，不含代码 | 桌面特效页 |
| 桌面组件 | `widget` | 摆在桌面一角的小卡片（时钟、番茄钟、诗词等） | 一个自包含的 `index.html` | 桌面组件页 |
| 小游戏 | `game` | 全屏离线小游戏 | 一个文件夹的文本文件，入口 `index.html` | 游戏空间页 |

内置的桌面组件（木鱼求签、小伴日历、天气时钟、便签待办、倒数日）、小伴外壳和关怀提醒不属于资源包，不能自创。

导入只是把作品放进对应页面，**不会自动应用**：不换壁纸、不开特效、不摆组件、不启动游戏。

## zip 的通用规则

- 标准 zip，压缩或仅存储都行，文件名以 `.zip` 结尾。
- 最外层必须有 `manifest.json`。把整个文件夹压缩（访达“压缩”、`zip -r 名字.zip 文件夹`）也可以：只要最外层没有 `manifest.json` 且所有文件都在同一个文件夹里，就会自动进入那个文件夹。
- 文件夹条目、`__MACOSX/`、`.DS_Store` 和以 `._` 开头的文件会被忽略。
- 以下会被拒绝：符号链接、加密、绝对路径、含 `..` 的路径、含 `\` 或 `:` 的路径、重名文件（不分大小写）。
- 数量和大小：最多 100 个文件，单个文件不超过 4 MB，zip 本身和解压后的总量都不超过 32 MB。各类型另有更严的限制，见下文。
- 所有文件必须是 UTF-8 文本。图片、字体等二进制不能放进包里，用内联 SVG、CSS 或 canvas 绘制。

### manifest.json

必填两项，其余字段看类型，不认识的字段会被忽略：

```json
{ "schemaVersion": 1, "kind": "widget" }
```

`schemaVersion` 现在只能是 `1`，`kind` 是上表四个值之一。

## 动态壁纸 `wallpaper`

文件：

```
manifest.json
index.html
```

manifest 字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `title` | 是 | 壁纸名称，最多 48 个字 |
| `description` | 否 | 简短介绍，不超过 2 KB |

`index.html` 不超过 4 MB，写成单个自包含页面：样式和脚本都内联，用 CSS 动画、canvas 或 WebGL 画，铺满 `100vw × 100vh` 并随窗口自适应。不要引用外部 CDN、字体或图片，不要依赖联网。

壁纸运行在 `sandbox="allow-scripts"` 的 iframe 里：脚本能跑，但没有同源权限，所以不能用 `localStorage`、cookie、IndexedDB，不能弹窗、提交表单或跳转外层页面。

内容完全相同的壁纸导入时只保留一份。

## 桌面特效 `effect`

文件：

```
manifest.json
effect.json
```

manifest 只需要 `schemaVersion` 和 `kind`。`effect.json` 就是特效包本身，它的 `id`、`name` 写在里面。特效包是纯数据（几层粒子、每层多少个、多大、怎么运动、什么形状），不含代码。

字段、形状、爆发、带状分布的完整规则见 [effects.md](effects.md)，可直接运行的样例见 [effect-pack-example.json](effect-pack-example.json)。要点：

- `effect.json` 不超过 64 KB，`layers` 为 1 到 6 层，各层 `count` 合计大于 0 且不超过 3000。
- `id` 只用小写字母、数字、`-`、`_`，不超过 48 个字符，不能以 `-` 或 `_` 开头，不能与内置特效重名；已有同 `id` 的自创特效时拒绝导入。
- 最多加载 40 个自创特效。

## 桌面组件 `widget`

文件：

```
manifest.json
index.html
```

manifest 字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `id` | 是 | 1 到 48 个字符，小写字母或数字开头，其余用小写字母、数字、`-`、`_`。不能与内置组件重名：`fortune`、`calendar`、`weather`、`notes`、`countdown` |
| `name` | 是 | 显示名，最多 24 个字 |
| `description` | 否 | 一句话介绍，最多 80 个字 |
| `width` | 否 | 默认宽度，100 到 800 像素，默认 280 |
| `height` | 否 | 默认高度，80 到 800 像素，默认 240 |
| `theme` | 否 | 底板风格：`transparent` 透明无底（默认）、`glass` 毛玻璃、`paper` 纸张 |

`index.html` 不超过 128 KB，单个自包含页面，要求：

- 在 `width × height` 里自适应，不出现滚动条。应用会给页面加上 `html, body { margin: 0; overflow: hidden }`。
- 默认透明无底：`html`、`body` 不要画铺满的背景，只在用户需要时才加底板。
- 不引用外部资源，不依赖联网。

组件运行在 `sandbox="allow-scripts"` 的 iframe 里，没有同源权限，所以和壁纸一样不能用 `localStorage`、cookie、IndexedDB。

> **脚本的已知限制。** 组件通过 `srcdoc` 嵌入应用页面，会继承应用的内容安全策略（`script-src 'self'`）。在浏览器里用同样的策略实测，这种子框架里的内联 `<script>` 不会执行，去掉策略则可以执行。这一点还没有在打包后的应用里实测过。需要交互时，先用纯 HTML 和 CSS 实现（`:hover`、`:checked`、`<details>`、CSS 动画），要用脚本的组件请在应用里放出来确认能跑再分享。

已有同 `id` 的自创组件时拒绝导入，最多加载 40 个自创组件。

## 小游戏 `game`

文件：

```
manifest.json
index.html          必须，入口页
game.js             其余文件按需
style.css
cover.svg           可选，作封面
```

manifest 字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `id` | 是 | 2 到 40 个字符，小写字母、数字、`-`，以字母或数字开头。不能与内置游戏重名：`falling-blocks`、`merge-2048`、`block-puzzle`、`water-sort`、`snake-arena`、`ink-blade`、`dusk-glider` |
| `name` | 是 | 游戏名称，最多 24 个字 |
| `description` | 是 | 一句话介绍，最多 80 个字 |
| `save` | 否 | `true` 时启用本机存档，默认 `false` |

这里的 `manifest.json` 是资源包的清单。包里除它以外的文件原样成为游戏文件夹，游戏自己的清单由小伴桌面生成。

文件规则：

- 只能是 `html`、`js`、`css`、`json`、`svg`、`md`、`txt` 文本文件；最多 40 个，单个不超过 1 MB，合计不超过 4 MB。
- 路径按 `/` 分层，最多 4 层，每段只含字母、数字、`.`、`_`、`-`，不以 `.` 开头。
- 用普通 `<script src="game.js">`，不要 ES modules（`type="module"`、`import`/`export`）；所有资源用相对路径；不联网，不引用外部 CDN、字体或图片。

运行环境：全屏沙箱 iframe，没有网络、`localStorage`、cookie、IndexedDB 和任何系统权限。画布按 `100vw × 100vh` 适配，同时支持键盘和鼠标/触摸。Esc 退出由宿主注入的 `desktop-host.js` 处理，游戏不要自己处理 Esc，也不要自己引用这个文件。

### 与宿主的握手

宿主会自动回复握手，所以不读存档也不报成绩的游戏可以什么都不写。需要存档或成绩时：

1. 在页面脚本顶层同步监听 `window` 的 `message`。宿主在 iframe 加载完成后发 `{ channel: 'xiaban.game-host', protocol: 1, type: 'host.init', nonce, save, safeArea }`，`save` 是上次的存档对象或 `null`。
2. 只接受 `event.source === window.parent`，且 `channel`、`protocol`、`nonce` 都匹配的消息。`nonce` 与地址栏参数 `hostNonce` 相同：`new URLSearchParams(location.search).get('hostNonce')`。
3. 收到 `host.init` 后回复 `window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type: 'game.ready' }, '*')`。`game.ready` 之前发出的其他消息会被忽略。
4. 成绩（可选）：一局开始发 `type: 'game.start'`，之后可发 `game.progress` 和一次 `game.result`，都带非负整数 `score`，其余结构同上。
5. 存档（仅 `save: true` 时有效）：`game.ready` 之后发 `{ channel: 'xiaban.game-pack', protocol: 1, nonce, type: 'game.save', data: { … } }`。`data` 必须是普通 JSON 对象，序列化后不超过 32 KiB。进度变化时就发，不要等退出时才存。

已有同 `id` 的自创游戏时拒绝导入，最多 40 款自创游戏。游戏存档不会进入资源包。

## 导入时的检查

在对应页面点“导入资源包”，选 zip。应用按下面的顺序检查，任何一步失败都不会写入任何东西，并说明原因：

1. 是不是有效的 zip，数量、大小、路径是否合规。
2. 最外层有没有 `manifest.json`，`schemaVersion` 是不是 `1`。
3. `kind` 是否与当前页面一致（在壁纸页导入组件包会提示类型不匹配）。
4. 该类型自己的规则：上文各节的字段、长度、大小、`id` 格式和重名检查。

## 最小示例

组件包，文件夹里两个文件：

`manifest.json`

```json
{ "schemaVersion": 1, "kind": "widget", "id": "hello-card", "name": "问候卡片", "width": 240, "height": 120, "theme": "glass" }
```

`index.html`

```html
<!doctype html>
<meta charset="utf-8">
<style>
  html, body { margin: 0; height: 100%; background: transparent; }
  body { display: grid; place-items: center; font: 600 22px "PingFang SC", "Microsoft YaHei", sans-serif; color: #fff; }
  span { animation: breathe 3s ease-in-out infinite; }
  @keyframes breathe { 50% { transform: scale(1.08); opacity: 0.8; } }
</style>
<span>今天也辛苦啦</span>
```

打包时在文件夹里面压缩，让 `manifest.json` 在最外层：

```bash
cd hello-card && zip -r ../hello-card.zip .
```

壁纸包同理，`manifest.json` 写 `{ "schemaVersion": 1, "kind": "wallpaper", "title": "水墨涟漪" }`，再放 `index.html`。特效包放 `manifest.json` 和 `effect.json`。游戏包放 `manifest.json`、`index.html` 和其他游戏文件。

## 给 AI 工具的检查清单

做完一个包，逐项确认：

- [ ] zip 最外层（或唯一的外层文件夹下）有 `manifest.json`，`schemaVersion` 是 `1`，`kind` 正确。
- [ ] 所需文件齐全：壁纸和组件有 `index.html`，特效有 `effect.json`，游戏有 `index.html`。
- [ ] 全是 UTF-8 文本，没有图片、字体等二进制。
- [ ] `id` 符合该类型的格式，且不与内置项重名。
- [ ] 名称、简介没超过字数，页面和文件没超过大小。
- [ ] 没有外部链接、CDN 和联网请求；没有 `localStorage`、cookie。
- [ ] 组件和壁纸在 `100vw × 100vh` 或指定的宽高里自适应；游戏用经典脚本，不用 ES modules。
- [ ] 组件的交互不依赖内联脚本，或已在应用里确认能运行。

## 实现位置

| 内容 | 位置 |
| --- | --- |
| zip 的读写、检查和导入 | `src-tauri/src/resource_packs.rs` |
| 组件的检查和保存 | `src-tauri/src/custom_widgets.rs` |
| 特效的检查和保存 | `src-tauri/src/effect_packs.rs` |
| 游戏的检查和保存 | `src-tauri/src/game_packs.rs` |
| 壁纸的保存 | `src-tauri/src/local_media.rs`（`save_local_html`） |
| 导入导出按钮 | `src/ui/components/ResourceShareActions.vue` |
