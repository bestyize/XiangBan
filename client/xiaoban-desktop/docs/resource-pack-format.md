# 小伴桌面资源包格式

小伴桌面的自创内容可以打成一个普通的 **zip 文件**分享、备份和导入。这份文档写给想在别的工具（Claude Code、Cursor、脚本、手工）里制作资源包的人：照着做出 zip，在小伴桌面对应页面点“导入资源包”就能用。

小伴桌面里的“导出资源包”和“分享到论坛”生成的也是同样的 zip，反过来也能当作样例解压看看。

## 能做什么

| 类型 | `kind` | 是什么 | 内容 | 导入位置 |
| --- | --- | --- | --- | --- |
| 动态壁纸 | `wallpaper` | 铺满桌面的 HTML 动画或互动页面 | 一个自包含的 `index.html` | 壁纸页，“我的壁纸” |
| 桌面特效 | `effect` | 铺在桌面上的粒子层（雨、雪、花瓣、烟花等） | 一个 JSON 描述，不含代码 | 桌面特效页 |
| 桌面组件 | `widget` | 摆在桌面一角的小卡片（时钟、番茄钟、诗词等） | 一个自包含的 `index.html` | 桌面组件页 |
| 小游戏 | `game` | 离线小游戏（独立窗口，可全屏） | 一个文件夹的文本文件，入口 `index.html` | 游戏空间页 |
| 桌面伙伴 | `pet` | 可自创形象的伙伴主题 | `pet.json`，可带 `assets/` | 桌面伙伴页 |

官方内置组件和关怀提醒不是自创资源包；桌面伙伴主题可导入、导出和自创，详细结构与限制见 [桌面宠物主题](pets.md)。

导入只是把作品放进对应位置，**不会自动应用**：不换壁纸、不开特效、不摆组件、不启动游戏。

### 各类型能带什么

| 类型 | 包里的文件 | 图片、音频、字体 |
| --- | --- | --- |
| 动态壁纸 | `manifest.json`、`index.html` | 不支持放文件，请用 `data:` 地址写进页面 |
| 桌面特效 | `manifest.json`、`effect.json` | 不适用 |
| 桌面组件 | `manifest.json`、`index.html` | 不支持放文件，请用 `data:` 地址写进页面 |
| 小游戏 | `manifest.json` 加任意多个文本文件 | 支持，放在 `assets/` 下 |
| 桌面伙伴 | `manifest.json`、`pet.json` | 支持图片素材，放在 `assets/` 下 |

不支持的类型里出现多余的文件时，导入会直接拒绝并列出是哪些文件，不会悄悄丢掉。壁纸和组件以后会支持 `assets/`，到时候同一套写法可以直接用。

## zip 的通用规则

- 标准 zip，压缩或仅存储都行，文件名以 `.zip` 结尾。
- 最外层必须有 `manifest.json`。把整个文件夹压缩（访达“压缩”、`zip -r 名字.zip 文件夹`）也可以：只要最外层没有 `manifest.json` 且所有文件都在同一个文件夹里，就会自动进入那个文件夹。
- 文件夹条目、`__MACOSX/`、`.DS_Store` 和以 `._` 开头的文件会被忽略。
- 以下会被拒绝：符号链接、加密、绝对路径、含 `..` 的路径、含 `\` 或 `:` 的路径、重名文件（不分大小写）。
- 数量和大小：最多 110 个文件，单个文件不超过 4 MB，zip 本身和解压后的总量都不超过 32 MB。各类型另有更严的限制，见下文。
- 除游戏和伙伴主题的 `assets/` 外，所有文件必须是 UTF-8 文本。

### manifest.json

必填两项，其余字段看类型，不认识的字段会被忽略：

```json
{ "schemaVersion": 1, "kind": "widget" }
```

`kind` 是上表所列值之一。`schemaVersion` 是 `1` 或 `2`：

- 只有文本文件的包用 `1`，旧版本的小伴桌面也能导入。
- 带 `assets/` 资源文件的包必须写 `2`。旧版本遇到 `2` 会提示“请更新小伴桌面”，不会出现看不懂的报错。写 `1` 却放了资源文件会被拒绝。
- 小伴桌面导出时会按内容自动选择版本。

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
assets/hero.png     可选，图片、音频、字体放这里
assets/sfx/jump.ogg
```

创作游戏时应同时提供与玩法和名称匹配的 `cover.svg`，建议 640×360（16:9），不引用外部资源。`game_create` 会自动登记此文件为封面；没有封面的旧游戏会在客户端显示按名称和类型生成的默认插画，自定义封面加载失败时也会回退到默认插画。

manifest 字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `id` | 是 | 2 到 40 个字符，小写字母、数字、`-`，以字母或数字开头。不能与内置游戏重名：`falling-blocks`、`merge-2048`、`block-puzzle`、`water-sort`、`snake-arena`、`ink-blade`、`dusk-glider` |
| `name` | 是 | 游戏名称，最多 24 个字 |
| `description` | 是 | 一句话介绍，最多 80 个字 |
| `save` | 否 | `true` 时启用本机存档，默认 `false` |
| `orientation` | 否 | `landscape` 或 `portrait`：游戏窗口按 16:9 或 9:16 打开。不写时按 9:16 竖版打开 |
| `layer` | 否 | 游戏在哪里运行：`space` 独立游戏窗口（默认，默认不全屏），`desktop` 桌面层，见下文“桌面层游戏” |
| `capabilities` | 否 | 只用于 `desktop`，向宿主申请的能力：`windows`、`pointer`，默认都不给 |

这里的 `manifest.json` 是资源包的清单。包里除它以外的文件原样成为游戏文件夹，游戏自己的清单由小伴桌面生成。

文件规则：

- 只能是 `html`、`js`、`css`、`json`、`svg`、`md`、`txt` 文本文件；最多 40 个，单个不超过 1 MB，合计不超过 4 MB。
- 路径按 `/` 分层，最多 4 层，每段只含字母、数字、`.`、`_`、`-`，不以 `.` 开头。
- 用普通 `<script src="game.js">`，不要 ES modules（`type="module"`、`import`/`export`）；所有资源用相对路径；不联网，不引用外部 CDN、字体或图片。

### 图片、音频和字体（assets/）

需要位图、音效、音乐或字体时，放在包里的 `assets/` 目录下，页面和脚本用相对路径引用，例如 `<img src="assets/hero.png">`、`new Audio('assets/sfx/jump.ogg')`、CSS 里的 `url(assets/fonts/title.woff2)`。包的 `schemaVersion` 要写 `2`。

| 项目 | 规则 |
| --- | --- |
| 类型 | `png`、`jpg`、`jpeg`、`webp`、`gif`、`mp3`、`ogg`、`wav`、`woff2` |
| 位置 | 必须在 `assets/` 下，其他位置的二进制文件会被拒绝；路径规则同上（最多 4 层，只用字母数字和 `. _ -`，不以 `.` 开头） |
| 数量和大小 | 最多 60 个，单个不超过 4 MB，合计不超过 16 MB，不能是空文件 |
| 内容检查 | 会核对文件开头和扩展名是否一致，把别的文件改名冒充会被拒绝 |

`assets/` 里的文本类文件（比如 `assets/data.json`）按普通文本文件算，计入前面 40 个文件和 4 MB 的限制。图片、音频、字体仍然只能被页面读取，没有网络，也不能执行。

> **论坛附件上限。** 带很多图片和音频的包可能超过论坛附件的 5 MB。控制体积，或者把素材压缩（图片用 webp，音频用 ogg 或 mp3）。

运行环境：沙箱 iframe 铺满游戏窗口，没有网络、`localStorage`、cookie、IndexedDB 和任何系统权限。画布按 `100vw × 100vh` 适配（窗口大小可变，默认不全屏），同时支持键盘和鼠标/触摸。Esc 退出由宿主注入的 `desktop-host.js` 处理，游戏不要自己处理 Esc，也不要自己引用这个文件。

### 与宿主的握手

宿主会自动回复握手，所以不读存档也不报成绩的游戏可以什么都不写。需要存档或成绩时：

1. 在页面脚本顶层同步监听 `window` 的 `message`。宿主在 iframe 加载完成后发 `{ channel: 'xiaban.game-host', protocol: 1, type: 'host.init', nonce, save, safeArea }`，`save` 是上次的存档对象或 `null`。
2. 只接受 `event.source === window.parent`，且 `channel`、`protocol`、`nonce` 都匹配的消息。`nonce` 与地址栏参数 `hostNonce` 相同：`new URLSearchParams(location.search).get('hostNonce')`。
3. 收到 `host.init` 后回复 `window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type: 'game.ready' }, '*')`。`game.ready` 之前发出的其他消息会被忽略。
4. 成绩（可选）：一局开始发 `type: 'game.start'`，之后可发 `game.progress` 和一次 `game.result`，都带非负整数 `score`，其余结构同上。
5. 存档（仅 `save: true` 时有效）：`game.ready` 之后发 `{ channel: 'xiaban.game-pack', protocol: 1, nonce, type: 'game.save', data: { … } }`。`data` 必须是普通 JSON 对象，序列化后不超过 32 KiB。进度变化时就发，不要等退出时才存。

已有同 `id` 的自创游戏时拒绝导入，最多 40 款自创游戏。游戏存档不会进入资源包。

## 桌面层游戏

`layer` 写 `desktop` 的游戏不进独立游戏窗口，而是直接跑在桌面上：小伴在主显示器上放一个透明窗口，浮在其他应用窗口的上方，游戏页面画在里面。适合会在桌面上走动的角色、跟着鼠标的小玩意、依附在窗口边缘的互动。

包的 `schemaVersion` 必须写 `2`。旧版本的小伴桌面遇到它会提示更新，不会把它当成普通游戏去跑。

### 运行方式

- 透明窗口默认不接收鼠标，点击会穿过去落到下面的应用。游戏要用 `game.regions` 告诉宿主哪些矩形需要鼠标，鼠标移进这些矩形时窗口才接收点击和拖拽，移出就恢复穿透。按住鼠标拖动时，即使移出矩形也会继续接收，直到松开。
- 窗口不会主动获得键盘焦点，游戏不要依赖键盘输入，也没有 Esc 退出。游戏要自己提供退出方式（比如双击，然后发 `game.exit`）。
- 用户随时可以在托盘菜单的“停止桌面游戏”或游戏页的“停止桌面游戏”按钮结束它。同一时间只会运行一个桌面层游戏。打开游戏窗口时它会先被关掉。
- 沙箱、网络、存储、文件规则和普通游戏相同，游戏仍然没有任何系统权限。
- 只支持 Windows 和 macOS。

### 能力

没有申请的能力，宿主一个字也不会发。

| `capabilities` | 宿主会发什么 |
| --- | --- |
| `windows` | 其他应用窗口的位置和大小。不含标题、所属应用和画面。 |
| `pointer` | 鼠标位置和左键是否按下。 |

### 消息

沿用普通游戏窗口的握手（`channel`、`protocol: 1`、`nonce`，见上文），新增三种消息：

宿主发给游戏：

- `host.init` 多带三项：`layer: 'desktop'`、`capabilities`、`screen: { width, height }`。`screen` 是透明窗口的大小，单位是 CSS 像素，也是下面所有坐标的范围。
- `host.frame`，约每秒 30 次，只含游戏申请过的字段：

```json
{
  "channel": "xiaban.game-host", "protocol": 1, "type": "host.frame", "nonce": "…",
  "windows": [{ "x": 120, "y": 80, "w": 900, "h": 640 }],
  "pointer": { "x": 512, "y": 300, "down": false }
}
```

`windows` 从前到后排列，第一个在最上面；已经裁剪到屏幕内，最多 40 个，小于 80×40 的窗口、菜单栏、程序坞、本应用自己的窗口都不会出现。坐标以屏幕左上角为原点。窗口被别的窗口挡住的部分不会被扣掉，游戏想站在窗口上沿，需要自己用前面的窗口去遮挡（参考示例 `docs/examples/desk-pal/game.js` 里的 `floors()`）。`pointer` 同一原点，鼠标在别的显示器上时坐标可能在 `screen` 范围之外。

游戏发给宿主：

- `game.regions`：`{ type: 'game.regions', rects: [{ x, y, w, h }] }`，需要接收鼠标的矩形，最多 32 个，会被裁剪到屏幕内，格式不对的项直接丢弃。每次整体替换上一次的结果；发空数组表示完全穿透。位置变了就重新发，但不必每帧都发。
- `game.exit`：结束游戏。

### 示例

`docs/examples/desk-pal/` 是一个完整的桌面层游戏：一个站在窗口上沿走来走去的小家伙，会跟着窗口移动，窗口被挡住或移走就掉下去，眼睛看向鼠标，可以拖拽，双击退出。三个文件就能做成包：

```bash
cd docs/examples/desk-pal && zip -r ../desk-pal.zip .
```

## 导入时的检查

壁纸、特效、组件、游戏各页都有“导入资源包”，在哪一页选都行：应用按 `manifest.json` 里的 `kind` 把作品放到它该去的地方，并提示去哪里查看。应用按下面的顺序检查，任何一步失败都不会写入任何东西，并说明原因：

1. 是不是有效的 zip，数量、大小、路径是否合规。
2. 最外层有没有 `manifest.json`，`schemaVersion` 是不是 `1` 或 `2`。
3. 包里的文件是否都在该类型允许的范围内（壁纸、组件、特效不允许多余文件）。
4. 该类型自己的规则：上文各节的字段、长度、大小、`id` 格式、重名检查，以及游戏资源文件的类型和内容检查。

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
- [ ] 除游戏 `assets/` 下的图片、音频、字体外，全是 UTF-8 文本；带 `assets/` 的包 `schemaVersion` 是 `2`。
- [ ] 壁纸和组件里的图片已经用 `data:` 地址写进页面，没有额外的图片文件。
- [ ] `id` 符合该类型的格式，且不与内置项重名。
- [ ] 名称、简介没超过字数，页面和文件没超过大小。
- [ ] 没有外部链接、CDN 和联网请求；没有 `localStorage`、cookie；资源引用都是相对路径。
- [ ] 组件和壁纸在 `100vw × 100vh` 或指定的宽高里自适应；游戏用经典脚本，不用 ES modules。
- [ ] 组件的交互不依赖内联脚本，或已在应用里确认能运行。
- [ ] 桌面层游戏：`schemaVersion` 是 `2`，`layer` 是 `desktop`，只申请真正用到的 `capabilities`；用 `game.regions` 声明要鼠标的区域；自己提供退出方式，不依赖键盘。

## 实现位置

| 内容 | 位置 |
| --- | --- |
| zip 的读写、检查和导入 | `src-tauri/src/resource_packs.rs` |
| 组件的检查和保存 | `src-tauri/src/custom_widgets.rs` |
| 特效的检查和保存 | `src-tauri/src/effect_packs.rs` |
| 游戏的检查和保存 | `src-tauri/src/game_packs.rs` |
| 壁纸的保存 | `src-tauri/src/local_media.rs`（`save_local_html`） |
| 导入导出按钮 | `src/ui/components/ResourceShareActions.vue` |



## 桌宠资源包

桌宠沿用 ZIP 导入导出入口，最外层 `manifest.json` 为 `{"schemaVersion":2,"kind":"pet"}`，作品写在 `pet.json`，图片放在 `assets/`。主题与图片格式、限制及完整示例见 [桌宠格式](pets.md)。ZIP 只含创作素材，导入后在桌宠页自定义主题中选择并添加，不包含桌面实例与位置，也不会自动运行。
