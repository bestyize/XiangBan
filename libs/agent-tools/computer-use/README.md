# agent-computer-use-tools

可复用的 Open Computer Use 原生引擎接入包，依赖 agent-kernel 的 MCP 客户端，不依赖 Tauri。宿主负责分发原生资源、启用开关、权限提示和工具注册。

## 内置运行环境

固定使用上游 `open-computer-use` **1.1.2**。`runtime-lock.json` 固定下载地址和 SHA-512；只在构建机下载并校验 npm 分发包，不执行 npm 安装脚本。用户电脑直接运行原生程序，无需 Node.js、npm、npx、Python 或首次联网下载。保留上游 MIT 许可和第三方声明。

```sh
# 从仓库根准备某个平台的资源。省略 --target 时优先使用 TAURI_ENV_TARGET_TRIPLE，然后才是构建机平台。
node libs/agent-tools/computer-use/scripts/prepare-runtime.mjs --target x86_64-pc-windows-msvc --output client/xiaoban-desktop-wallpaper/src-tauri/resources/computer-use
```

| 目标 | 只打包的原生资源 |
| --- | --- |
| Windows x64 / arm64 | 对应架构的 `open-computer-use.exe` |
| macOS arm64 / x64 | `Open Computer Use.app`，主程序及所有辅助程序均裁剪成目标架构后重新签名 |
| Linux x64 / arm64（工具包支持，桌面客户端未发布） | 对应架构的 ELF 程序 |
| Android / 其他目标 | 不支持；Android 独立构建流程不调用本工具、不包含桌面引擎 |

准备资源时只提取所选平台目录及许可文件，并整体替换输出目录。构建前检查 runtime.json、PE/ELF/Mach-O 文件格式和实际 CPU 架构；残留其他平台、缺少资源、错误架构均中止构建。macOS 必须在 Mac 构建，使用 `lipo` 裁剪通用二进制，并使用 `APPLE_SIGNING_IDENTITY` 或小伴的固定自签名证书从内到外签名；共享构建机设置见 [.github/signing](../../../.github/signing/README.md)。禁止 ad-hoc 回退，检查 designated requirement 不包含版本专属 `cdhash`。安装包内的辅助 App 保留独立 Bundle ID；macOS 权限条目以系统实际显示为准。

小伴的 Tauri `beforeDevCommand` / `beforeBuildCommand` 自动准备资源；release 的 build.rs 再校验资源与 Cargo TARGET 一致。通过 `bundle.resources` 映射到 `$RESOURCE/agent-tools/computer-use`。资源及下载缓存均不提交 git。固定引擎升级需要一起修改 lock、Rust `RUNTIME_VERSION` 并重新验证。

## 运行接口

```rust,ignore
let root = installed_resource_dir.join("agent-tools/computer-use");
let client = agent_computer_use_tools::spawn(&root)?;
for tool in agent_computer_use_tools::checked_tools(&client, &root) {
    builder = builder.tool_arc(tool);
}
```

- `executable(root)` / `config(root)`：只解析宿主传入的内置资源目录，检查版本、平台、程序存在及 Unix 执行权限；不会查找 PATH 或回退 npx。
- `spawn(root)`：启动原生 MCP 服务并发现工具；空工具列表视为失败，风险为 Exec、调用超时 60 秒。
- `checked_tools(client, root)`：宿主使用这些包装后的工具注册到 Agent；macOS 每次桌面操作前运行内置引擎的 `doctor --json` 检查真实授权，缺失时明确报错并指向诊断/引导，不执行原始动作、不自动改走脚本。保留 MCP 图片、结构化输出和清理接口。
- `permissions::diagnose(root)`：无弹窗检查真实权限、签名，以及只针对当前引擎的近期 TCC 签名冲突日志；以 `accessibilityTrusted` 和 `screenCaptureGranted` 判定权限，不能用退出码或 MCP 连接代替。`authorizationMismatch.services` 有项才表示找到对应旧签名冲突证据；日志不可用或空结果不证明授权有效。
- `permissions::request_permissions(root)`：运行原生 `doctor` 引导并在 Finder 选中当前引擎，不自动批准或重置权限。`permissions::repair_permissions(root, permission)` 由宿主**逐次确认后**调用：仅执行固定 `com.ifuryst.opencomputeruse` 的 `tccutil reset Accessibility` 或 `ScreenCapture`，已生效的所选权限不重置；报告重置结果并打开重新授权入口，最终系统授权仍需用户点击。同标识独立安装版的所选权限也受影响。宿主即使完全信任也必须拦截确认，不能只依赖 `always_ask`。不提供全局重置或写权限数据库接口。诊断、引导和修复由宿主单独注册，MCP 启动失败时仍可排障。
- 参数校验失败表示没执行工具，应修正并重试原调用；`list_apps` 最简参数是 `{}`，不能带 schema 未声明的 `reason` 或 `command`。不能因此重置权限或改用 Python/AppKit/AppleScript 绕过。
- `SKILL` / `SKILL_NAME` / `SKILL_ORIGIN_MARKER`：随包操作规则与已有安装副本的识别标记。
- macOS 引擎要求 14+ 及辅助功能、屏幕录制授权；不降低宿主自身对旧系统的兼容范围。Windows/Linux 需要已登录图形桌面。

## 验证

```sh
cargo test --manifest-path libs/agent-tools/computer-use/Cargo.toml --locked
node --test libs/agent-tools/computer-use/scripts/prepare-runtime.test.mjs
node libs/agent-tools/computer-use/scripts/prepare-runtime.mjs --check --target x86_64-pc-windows-msvc --output <已准备目录>
XIAOBAN_COMPUTER_USE_RUNTIME=<已准备目录> cargo test --manifest-path libs/agent-tools/computer-use/Cargo.toml --locked real_bundled_engine_discovers_tools -- --ignored --nocapture
XIAOBAN_COMPUTER_USE_RUNTIME=<已准备目录> cargo test --manifest-path libs/agent-tools/computer-use/Cargo.toml --locked real_bundled_engine_permission_diagnosis -- --ignored --nocapture
```

普通测试不启动控制服务。最后一项使用真实原生程序完成 MCP 握手和工具发现，不执行桌面操作。macOS/Windows 安装、权限、点击及输入回读仍需对应平台实机验收。CI 仅手动触发；macOS 打包后还检查实际 App 内的资源、辅助程序架构及签名。
