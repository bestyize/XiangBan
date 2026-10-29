# 小伴壁纸用户数据

正式应用的数据根目录固定为当前用户的 `~/.xiaobanwallpaper`：Windows 为 `C:\Users\<用户名>\.xiaobanwallpaper`，macOS 为 `/Users/<用户名>/.xiaobanwallpaper`。使用系统解析的用户主目录，不依赖程序安装目录、工作目录或某个固定用户名。“关于小伴”展示实际路径并提供打开目录按钮。

```text
.xiaobanwallpaper/
├── config/
│   ├── preferences.json       角色、布局、自启动、壁纸、桌面特效选择与服务地址设置
│   ├── pets.json               多盆植物实例：植物引用、位置、大小、层级、心情与开关
│   └── care-reminders.json    关怀规则、天气来源与结果、待展示提醒及去重状态
├── wallpapers/
│   ├── catalog.json           订阅或导入 JSON 清单的元数据
│   ├── local.json             用户导入图片、视频的元数据
│   ├── active.json            当前壁纸定义快照，用于离线与重启恢复
│   ├── imports/<sha256>       导入媒体的持久副本，属于用户内容
│   └── cache/<sha256>         订阅下载缓存，可重新从资源地址获取；网站来源链接下载的文件同样按本地计算的 SHA-256 保存
├── effects/                   用户自己的特效包（*.json），可选；首次从控制台打开该文件夹时创建并放入示例
├── pets/
│   └── themes/<id>.json        自定义植物主题，属于用户作品
├── chat/
│   ├── sessions/              完整聊天历史（包含工具结果与结构化摘要）
│   ├── index.json             对话名称、分组等索引
│   ├── runs/<session>/        各次运行的执行事件日志 run-<run-id>.jsonl
│   ├── spill/                较长工具输出的独立文件
│   ├── memory/               持久记忆笔记
│   └── skills/               宿主管理的技能
├── worktrees/                 对话的“独立副本”（git worktree），每个对话一个文件夹；合并或丢弃后自动删除
├── cache/
│   └── webview/               Windows WebView2 配置与运行缓存
├── games/
│   └── saves/
│       └── ink-blade.json     水墨剑客的本地进度（可选，最多 32 KiB）
├── backups/
│   └── legacy-v1/             首次迁移的旧设置、目录元数据与迁移来源记录
└── development/              开发预览的独立数据根，内部结构相同
```

`backups/legacy-v1` 只在存在旧版本数据、执行迁移时创建。内置壁纸仍打包在程序中，不重复复制到用户目录。Windows 的设置、桌宠、壁纸和预览窗口共用上述 WebView2 目录；macOS 使用非持久 WKWebView 数据存储，应用设置与壁纸资源仍由上述目录持久化，WebKit 临时文件由系统管理。系统自启动项依然位于 Windows 注册表或 macOS `Library/LaunchAgents`。

游戏包没有文件系统或浏览器存储权限。桌面宿主仅允许水墨剑客使用 `games/saves/ink-blade.json`，以及 manifest 声明 `save: true` 的自创游戏使用 `games/saves/user-<id>.json`；自创游戏本身放在 `games/packs/<id>/`（由 `game-pack:` 只读协议提供，路径限制在该目录内）；规则配置和提醒状态由原生宿主写入 `config/care-reminders.json`。天气城市由用户手动选择，不读取系统定位；查询天气需用户启用来源并联网。天气缓存和提醒去重账本属于关怀配置。备份时复制 `config`、`wallpapers`、`pets` 与需要保留的 `games`；游戏存档损坏或读写失败时应用不会用空进度覆盖它。

`worktrees/` 里是还没合并的改动，属于用户内容：保留副本、合并失败（冲突）或应用中途退出时，副本和它的分支会留在原处，不要手动清理。删除一个仍在副本里工作的对话会被拒绝，需要先在工作区菜单里选择合并、保留或丢弃。

聊天历史与执行日志可能包含用户输入、模型输出、工具参数及结果，不是可随意删除的缓存。需要保留对话时一并备份 `chat`。执行日志不自动上传；删除聊天会先停止该会话的受管进程，再删除对应日志与聊天历史。当前没有日志自动过期或磁盘总量回收策略；日志不可写时会停止本轮运行，避免继续执行而丢失记录。运行日志不用于自动重放命令，也不表示应用重启后原进程仍存活。

## 本地图片、视频

在“动态壁纸”点击“导入图片 / 视频”，从原生系统选择器选择一个文件。导入后自动选中“我的导入”中的该项，可单独预览、应用到桌面，配置铺满 / 完整显示 / 拉伸，视频可调整速度并静音循环。不会自动改变当前背景，应用和解码失败时保留之前的背景。

- 图片：PNG、JPEG、WebP、GIF、BMP，单张不超过 64 MiB。
- 视频：MP4、M4V、WebM，单个不超过 1 GiB。优先使用 H.264 MP4；WebM 与其他 MP4 编码是否可播放取决于系统 WebView。没有内置 FFmpeg、视频转码或音轨播放。
- 文件先检查真实格式，再流式复制、计算 SHA-256、发布元数据。同内容重复导入只保留一份；取消选择器不改变壁纸库。非法或不支持的文件不进入库。
- `imports` 保存程序管理的原始字节副本。源文件被移动或删除后仍可播放。设置和元数据不保存原始绝对路径，渲染端只获得按哈希访问的只读资源地址。
- 大视频的原生资源响应按最多 8 MiB 的块提供 byte ranges，避免为一条请求把整个视频读进内存。
- `local.json`、`imports` 是用户内容；不能当作可随意清理的缓存。需要备份时复制 `config`、`wallpapers`，以及需要保留的 `games`，无需备份 `cache/webview`。不要在应用运行时手动修改这些文件。

## 旧版本迁移

首次使用新目录、且 `config/preferences.json` 不存在时，从 Tauri 旧应用配置目录复制旧 `preferences.json`、`wallpapers/catalog.json`、`wallpapers/active.json` 与有效命名的资源缓存。正式和开发环境分别读取对应的旧配置目录；测试完全不读取正式配置。

原目录保留，旧设置和目录元数据另存 `backups/legacy-v1`。每个文件通过临时文件提交，设置文件最后复制，避免迁移中断后指向尚未复制的缓存。已存在的新数据不覆盖；中断后可补齐剩余文件。旧设置损坏或迁移失败会报错，不静默开启自启动或清空用户数据。

## AI 开发边界

所有应用设置、媒体和缓存路径从 `src-tauri/src/storage.rs` 获取，禁止另行使用当前工作目录或程序旁的 `data` 目录。`local_media.rs` 负责选择器、格式检查、复制和本地目录元数据；订阅 JSON 协议继续由 `library.rs` 维护，不接受 `local://` 或任意本地路径。新增存储版本使用迁移，保持旧数据可恢复。

开发根为 `.xiaobanwallpaper/development`，不登记正式自启动；smoke test 使用唯一临时目录或 `XIAOBAN_TEST_CONFIG_DIR`，不修改用户的 `.xiaobanwallpaper/config`。测试媒体在临时目录生成并清理，不读取或删除用户相册。

## 桌面植物

`config/pets.json` 保存多盆实例；`pets/themes/<id>.json` 保存自创植物。两者都属于用户内容，需一起备份。第一次升级且实例文件不存在时，从旧小伴偏好迁移一盆；已有实例文件不会被偏好重置，坏文件会保留并在宠物页提示。官方主题由程序提供，不复制到用户目录。植物包不能替换原花盆和表情，格式见 [pets.md](pets.md)。
