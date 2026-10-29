# 小伴桌面历史功能验证（2026-10-09 至 2026-10-10）

> 本文保留早期测试和待验收事项供追溯，测试数量、临时包路径及“本轮”均指当时基线，不代表当前版本或当前发布状态。运行入口与验证命令见 [项目 README](../README.md)。

本清单对应 PRD 第 1–3 项：游戏空间、关怀提醒和图片特效。Windows 自动化与工程检查已通过；它们不能替代真实桌面图标交互、天气在线查询和视觉/性能验收。2026-10-10 UI 重构结果见下方；2026-10-09 的测试与旧桌面包信息仅作为历史记录。当时尚未发布。

## 2026-10-10 UI 重构

| 检查 | 结果 |
| --- | --- |
| 前端全量测试 | 通过，51 tests / 11 files |
| Rust 测试 | 通过，55 tests |
| 桌面生产构建 | 通过；Windows x64 UI 包为 `artifacts/XiaoBan-UI-Windows-x64.exe`，6,014,464 bytes（5.74 MiB），SHA-256 `7EF780BABBA7F2D545EDBF56D7018D3F80E1E58E65CE0DD0CDE1A388B2420278` |
| Windows 原生 UI + 游戏 + 关怀 smoke | 通过；UI `ok=true`、配置绑定、只读原生控制台标记、5 个页面和 7 个游戏包均匹配；完整报告（本地：`../artifacts/games-smoke-1791564963404.json`） |
| Windows 媒体与特效 smoke | 通过；完整报告（本地：`../artifacts/media-smoke-1791564827748.json`） |
| UI 浏览器检查 | 通过；逐页检查 5 个分区，验证提醒创建、壁纸特效选择、关于页切换，以及 960×680 窗口无水平溢出；截图见桌面宠物（本地：`../artifacts/ui/companion-blue.jpg`）和壁纸库（本地：`../artifacts/ui/wallpapers-blue.jpg`） |

UI smoke 在 `XIAOBAN_UI_SMOKE=1` 时严格核对报告标记与页面、配置和游戏包字段，避免普通游戏 smoke 被误记为 UI 通过。此前 2026-10-09 试用包不代表本轮 UI。

## 2026-10-10 元数据网关错误恢复

| 检查 | 结果 |
| --- | --- |
| Rust 测试 | 通过，60 tests；包含 5 个元数据请求故障 fixture 测试 |
| Windows 生产构建 | 通过；XiaoBan-UI-NetworkFix-Windows-x64.exe（本地：`../artifacts/XiaoBan-UI-NetworkFix-Windows-x64.exe`），6,017,024 bytes（5.74 MiB），SHA-256 `AC52792F39CAF2B239867E0B4AD84EB07C90A632A785F46332F0B0E528E58454` |
| Windows 媒体与特效 smoke | 通过；完整报告（本地：`../artifacts/media-smoke-1791566257695.json`） |
| 前端测试 | 本轮未改前端；51 tests / 11 files 是此前 UI 重构结果，本轮未重复运行 |

修复范围仅限共享 JSON 元数据 HTTP 客户端：每次请求最多等待 8 秒、整体最多 26 秒，最多尝试 3 次，间隔依次为 250 ms 和 750 ms。仅对 502、503、504，以及连接建立阶段的超时进行重试；4xx、无效 UTF-8、超限响应和 redirect 错误不重试。媒体下载仍使用原有 120 秒超时与 SHA-256 校验路径。

此修复改善瞬时网关错误的恢复行为，但没有确认线上偶发错误的真实根因。近期多次在线 probe 返回 200，检查时 Clash/TUN 正在运行；线上日志没有读取，因为 SSH known-host 指纹校验不匹配，未绕过校验继续连接。网站源 fixture 联调仍未验证。

## 可执行检查

在仓库根目录运行：

```powershell
node client/xiaoban-desktop/scripts/check-game-assets.mjs
```

该只读检查核对桌面与网页的七包 catalog、manifest、entry、cover、HTML/CSS/JS 的本地引用、游戏包内源码引用边界。每个桌面游戏包自包含：各自带有同内容的 `desktop-host.js`，浅色主题三款（block-puzzle、merge-2048、water-sort）带有拆分后的 `theme.css`；桌面入口页由网页入口页按固定规则改写（去掉网页分享共享文件与分享按钮、主题与适配层改为本地引用），脚本据此校验，`theme.css` 规则合集须与网页 `fresh-theme.css` 完全对应，其他包资源仍要求与网页一致。**本轮已执行通过**。

根目录集成结果（2026-10-09）：

| 命令 | 结果 |
| --- | --- |
| `npm test`（桌面工程） | 通过，34 tests / 5 files |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked`（桌面工程） | 通过，55 tests |
| `npm run desktop:build`（桌面工程） | 通过；输出便携 Windows x64 EXE 5,814,272 bytes（约 5.54 MiB，SHA-256 `71AC7CFDB01BC65A2949400D047503F3B86D2F58739D542ABD1A452B3E8D04D0`），见 试用包（本地：`../artifacts/XiaoBan-P0-Windows-x64.exe`） |
| `scripts/verify-games.mjs`（`XIAOBAN_CARE_SMOKE=1`，Windows 原生 smoke） | 通过：七包 ready、存档往返、Esc、IPC 隔离、桌面状态恢复及 care 12 项字段；报告（本地：`../artifacts/games-smoke-1791560600186.json`） |
| `scripts/verify-media.mjs`（`XIAOBAN_EFFECTS_SMOKE=1`，Windows 原生 smoke） | 通过：六类媒体、两个本地导入用例、三种特效、三个图片来源、两个非图片模式、预览和持久化；报告（本地：`../artifacts/media-smoke-1791560320106.json`） |
| `scripts/check-game-assets.mjs` | 通过：桌面/网页资源目录和登记的桌面适配差异检查 |

桌面工程内的定向命令：

```powershell
cd client/xiaoban-desktop
npm test -- src/effects
npm exec vitest run src/care-model.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --locked games::tests
cargo test --manifest-path src-tauri/Cargo.toml --locked reminders::tests
cargo test --manifest-path src-tauri/Cargo.toml --locked care_weather::tests
```

下面列出的定向测试是可复跑命令；本轮完整 Rust 与前端测试结果见上表。其他原型回归命令按桌面 README 执行。

特效原生 smoke 是可选项；在 Windows PowerShell 中设置开关后运行媒体 smoke。它使用脚本启动的隔离本地 fixture，不发送天气请求：

```powershell
$env:XIAOBAN_EFFECTS_SMOKE = '1'
node scripts/verify-media.mjs
Remove-Item Env:XIAOBAN_EFFECTS_SMOKE
```

启用后，成功报告必须包含三个特效用例、三个图片来源、两个非图片模式、逐项持久化、预览不改设置且可关闭、磁盘配置匹配和离线缓存。本轮 smoke 已通过；后续改动特效代码后需重跑。

`scripts/verify-website.mjs` 未验证：本轮没有提供隔离 localhost 网站服务及 `XIAOBAN_WEBSITE_TEST_URL`，因此未启动该检查。天气 live 查询也未验证；care smoke 特意不发天气请求。

## 待人工验收

- Windows 10/11：仍需真实 Explorer 桌面检查双击入口：确认空白处双击进入、图标双击不进入、普通应用窗口双击不进入，并确认鼠标输入由 Explorer 处理。游戏烟测没有验证真实图标交互。该功能默认关闭，失败时保留显式入口。
- Apple 芯片 macOS：实机检查从设置、Dock/菜单栏进入游戏空间，以及窗口覆盖、焦点、Esc/可见返回、切换应用、关闭与恢复。macOS 当前没有可靠桌面空白双击入口。
- Windows 自动化已逐款确认七包 ready、隔离消息、Esc、存档往返及桌面状态恢复；两平台实际图像、操作和断网体验仍需人工验收。重启后确认水墨剑客存档可续玩；其余六款不应显示本地存档标签。
- 水墨剑客：验证超过 32 KiB、损坏或不可读的存档不会被空内容覆盖；模拟保存失败后检查提示和重试路径。
- 关怀提醒：关闭设置面板后验证一次、每天/指定星期、下班、休息与自定义规则；验证稍后、免打扰、游戏期间暂缓、关怀气泡关闭及应用退出边界。系统通知目前应显示不可用。
- 天气 live 查询：当前未测试；手动选择城市后检查来源、更新时间、过期和错误处理。确认来源失败时不显示伪实时天气；本地定时提醒仍能离线工作。
- 图片特效：Windows smoke 已检查三种特效、内置/下载/本地图片、视频与 Canvas 不渲染、预览取消、参数保存/恢复、缓存离线。真实画面质量、强度速度的可见差异、桌面图标操作和性能（设备、系统、分辨率、缩放、CPU/GPU/内存及帧耗时）仍待人工验收；macOS 也待实机验收。

人工结果需记录系统/设备、步骤、实际观察与通过/失败。Windows 双击的真实图标交互、macOS 实机表现、天气 live 查询、网站源联调和特效视觉/性能尚未验证；不代表小伴已正式发布。PRD 第 4–8 项仍属后续规划。

## 桌面特效改版后的验证状态（2026-10-10）

特效已从“每张壁纸一份”改为桌面上独立的一层覆盖窗口（见 README“桌面特效”）。上文中关于“三种特效”“内置/下载/本地图片来源”“视频与 Canvas 不渲染”“预览取消”的 Windows smoke 结果，是改版前的记录，对现在的行为不再成立。

- 已通过：前端测试（特效包检查、模拟、绘制、画布生命周期、七个内置包的合法性与大小密度检查、特效页、覆盖窗口页面、控制台状态）、Rust 测试（旧配置迁移、选择的数据模型、特效包文件夹的加载与出错报告）、类型检查；macOS 开发版上确认覆盖窗口出现在桌面图标之下、动态壁纸窗口之上，铺满主屏，旧版保存的特效已迁移成全局选择，七个特效在 1920×1080 下逐个看过。
- 已改写、尚未运行：`XIAOBAN_EFFECTS_SMOKE=1` 的原生 smoke 现在验证十二个内置特效各自到达页面和磁盘、换四种壁纸（含系统桌面）后覆盖窗口仍在、选“无特效”后窗口关闭；`scripts/verify-media.mjs` 的断言已同步为十三个特效。
- 待验证：Windows 上覆盖窗口是否确实位于所有应用窗口之下并允许鼠标穿过；macOS 多显示器、全屏应用和 Mission Control 下的表现；长时间运行的 CPU/GPU 占用。
