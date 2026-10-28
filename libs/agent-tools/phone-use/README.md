# Android 手机操作工具

独立于 Tauri 的 Rust 工具包。设计与首轮边界见 [实现方案](../../../docs/agent/android-phone-use.md)，实机范围见 [小米17联调记录](../../../docs/agent/android-phone-device-validation.md)。

应用包以 `<packagename>/skill.md + basic/ + patch/` 保存。`basic/package.json` 提供应用摘要；`tasks/`、`pages/`、`knowledge/`、`assets/` 按需读取；`routes/core.yaml` 与 `routes/flows/*.yaml` 表达核心骨架和功能路径。

路由支持 core / feature / temporary 角色及独立优先度、稳定性、可信度、边成本。基础包不接受临时路由。补丁使用带版本与证据的 YAML frontmatter，临时路径必须到期。普通文件完整替换，地图按稳定 ID 合并。基础包更新保留补丁，但基础版本变化后旧补丁停止自动应用，等待重新验证。

`Store::install` 接受固定 SHA-256 的 ZIP，仅允许 `skill.md` 与 `basic/`。最多 512 个 ZIP 条目、压缩 12 MiB、解压 32 MiB、单文件 4 MiB；拒绝目录穿越、链接、Windows 保留名、大小写重复和脚本。更新通过暂存与回滚替换。仅保证单进程内读写互斥，宿主应保持单实例；不要由外部进程同时修改包目录。

`Store::read` 的 offset / next_offset 是 UTF-8 字节；单页最多 8192 字节。补丁摘要由 `Store::patches` 动态生成。图片须使用媒体读取工具，不能作为 UTF-8 文本读入。

验证（仓库根）：

```sh
cargo test --manifest-path libs/agent-tools/phone-use/Cargo.toml
```

## 聊天工具

全部工具由宿主按需注册，普通聊天不启动 ADB。`phone_pack_find` 发现摘要；`phone_pack_read` 读取技能、任务、页面和基础图片；`phone_patch_list` 发现学习内容；`phone_routes` 合并路由 / 规划路径。分页读取使用 `next_offset` 直到 `has_more=false`。

## 常驻通道与本地工作流

默认 Adb 在一次任务内按显式设备序列号复用 `adb -s <serial> shell -T`，不再为每个 Shell 命令重启客户端或反复执行 devices -l。设备未授权/离线时，指定设备的通道建立或使用会失败；不会改用另一台手机。命令用随机标记分帧，串行执行、限制输出并响应取消；断连、超时、写入异常后关闭通道，结果未知的命令不自动重放。任务结束清理全部连接。截图、配对与文件传输仍使用独立命令，uiautomator dump 仍逐次启动，尚未提供常驻 UI 服务、画面流或游戏识别器。

操作包可在 tasks/ 放结构化 YAML：schema=1、package_name、name、description、app_versions、parameters、steps。仅支持 launch、wait、tap、fill、verify_fields、yield，最多32步；不支持 Shell、任意工具调用或循环。声明的 APP 版本是执行约束，不是验证证明。工作流必须以可观测后置条件或 yield 结束；序列化审查内容最多12000字节，超出应拆分流程。

1. `phone_workflow_inspect(package,path,app_version)` 原子读取适用基础资料/补丁，返回完整步骤和内容 SHA-256。
2. 核对用户目标和全部动作后，`phone_workflow_run(serial,package,path,sha256,parameters)` 需独立确认（包括完全信任模式），执行前重新读取手机真实版本和固定内容。
3. 已知步骤在本地连续运行，目标控件等待成功后复用刚刚取得的树（最多250ms）定位本次动作；每次输入前仍检查前台包。填写前确认目标 EditText 获焦，填写后回读，最后按字段顺序核对全部参数。输入后不复用旧树。
4. 目标歧义、前台切换、参数/摘要/版本不符或取消时停止；已开始的动作不重放，不自动从头重启。返回 completed / needs_model、checkpoint、逐步耗时、ADB 命令数、Shell 启动/复用数；不回显用户参数正文。执行器在步骤间检查90秒任务预算，单命令另有45秒超时，外层工具最多120秒。

内置小红书两段工作流：`tasks/open-image-picker.yaml` 到相册后 yield，让模型根据当前画面选指定素材；`tasks/prepare-image-note.yaml` 从图片编辑页进入正文编辑，填写并核对 title / body，停在预览，不发布、不存草稿。初始字段必须为空；已有文字会停止而非覆盖。对新执行器的验证仅限本地模拟，原先的真机逐步操作记录不能算其性能验收。

本地协议模拟器只运行 Python 标准库脚本、不调用 ADB；应用运行时不需要 Python。测试覆盖4条 Shell 命令复用1个本地进程、设备隔离、断连不重放、取消清理、工作流完成/交回模型、焦点/长正文回读及摘要更新拦截。尚无真实手机 P50/P95 或端到端提速结论。

`phone_pack_install` 接受 HTTPS URL 或当前工作区 ZIP 及固定 SHA-256；宿主对第三方安装逐次确认。`phone_patch_save` 在包内记录版本化经验，属于本机应用数据写入；它不会把模型自述的 evidence 转换为独立验证证明。补丁图片采集和编辑尚未实现。

`phone_devices`、`phone_connect`、`phone_observe`、`phone_action`、`phone_push_image` 提供设备连接、状态 / UI / 图片、单次动作、图片传输；必须显式选择已授权设备。输入前检查预期前台包名，但系统页面可能在检查后变化，仍须每次行动后回读。ADB 客户端有输出预算、45 秒内部超时、停止与清理；不启动后台自动操作循环。手机端已经开始的动作不能保证撤回，停止后先观察再继续。

本机需安装 Android Platform Tools，默认使用 PATH 中的 adb；小伴宿主允许 `XIAOBAN_ADB` 指定程序。Unicode 输入要求手机已安装且选中 [ADB Keyboard](https://github.com/senzhk/ADBKeyBoard) 的 `com.android.adbkeyboard/.AdbIME`；工具不会安装 APK 或自动更改输入法。

优先使用 `phone_action(action=tap_element, selector={description: "发布"})` 等精确语义选择器，支持 text / description / resource_id / class 组合。每次重新读取 UI，只有唯一、启用且可点击（或有可点击祖先）的目标才执行；混淆后的重复资源 ID 必须补充条件。坐标动作可加 `guard_selector`，例如选图前要求当前页唯一出现“草稿箱”，页面变化时拒绝输入。

`phone_observe(kind=ui)` 使用设备私有临时文件读取并清理 XML，兼容部分 Android 16 系统 `/dev/tty` 只有成功提示却不返回 XML 的行为。分页续读必须传首屏返回的 `snapshot_id` 和 `next_offset`，固定同一份树，120 秒内有效；输入前会使缓存失效。

`phone_match3` 是普通棋盘的本地候选交换评分器，不包含图像识别、重力 / 连锁模拟或完整关卡策略。两个种子包首次按需使用时安装到用户数据目录，已有包不被内置资料覆盖。

生成可分发 ZIP（输出目录建议放在工作区外，仓库不提交 ZIP）：

```sh
cargo run --manifest-path libs/agent-tools/phone-use/Cargo.toml --example export-packs -- <输出目录>
```

每个 ZIP 附带 `.sha256` 文件；可放入用户工作区导入，或由后续操作包托管服务提供 HTTPS 下载。首轮不包含在线目录、托管部署、十款应用完整适配或手机管理 UI。

`verified_app_versions` 仍为空：已在小米17 Android16上验证小红书9.51.0的图文准备（未发布）和消消乐1.162第4181关的两次交换，未对整条发布或通关任务验收。任务执行必须读取实际页面、核验用户目标，不能把基础包描述或学习证据当成已完成结果。

开发者真机调试入口（直接调用真实工具，不经过宿主模型与授权中间件，只用于已获用户授权的联调）：

```sh
cargo run --manifest-path libs/agent-tools/phone-use/Cargo.toml --example device-debug -- <独立包目录> <工作区> <请求JSON路径> <输出目录>
```

请求格式为 `{"tool":"phone_observe","arguments":{"serial":"显式设备序列号","kind":"ui"}}`。截图保存为 PNG，UI 自动续读同一快照并保存完整 `ui.xml`；不会把图片 Base64 打印到终端。调试截图、XML、设备序列号和账号信息不提交仓库。使用独立包目录隔离生产资料。
