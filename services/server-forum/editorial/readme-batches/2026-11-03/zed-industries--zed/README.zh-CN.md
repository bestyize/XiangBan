# Zed

[![Zed](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/zed-industries/zed/main/assets/badge/v0.json)](https://zed.dev)
[![CI](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml/badge.svg)](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml)

欢迎使用 Zed，一款由 [Atom](https://github.com/atom/atom) 和 [Tree-sitter](https://github.com/tree-sitter/tree-sitter) 的创建者开发的高性能多人协作代码编辑器。

---

### 安装

在 macOS、Linux 和 Windows 上，你可以[直接下载 Zed](https://zed.dev/download)，或通过本地包管理器安装（[macOS](https://zed.dev/docs/installation#macos)／[Linux](https://zed.dev/docs/linux#installing-via-a-package-manager)／[Windows](https://zed.dev/docs/windows#package-managers)）。

其他平台目前尚不可用：

- Web（[跟踪讨论](https://github.com/zed-industries/zed/discussions/26195)）

### 开发 Zed

- [为 macOS 构建 Zed](https://github.com/zed-industries/zed/blob/504a7f2cf25b817cda3998cab93473adef15f93c/docs/src/development/macos.md)
- [为 Linux 构建 Zed](https://github.com/zed-industries/zed/blob/504a7f2cf25b817cda3998cab93473adef15f93c/docs/src/development/linux.md)
- [为 Windows 构建 Zed](https://github.com/zed-industries/zed/blob/504a7f2cf25b817cda3998cab93473adef15f93c/docs/src/development/windows.md)

### 参与贡献

参阅 [CONTRIBUTING.md](https://github.com/zed-industries/zed/blob/504a7f2cf25b817cda3998cab93473adef15f93c/CONTRIBUTING.md)，了解可以如何为 Zed 贡献。

另外……我们正在招聘！请查看[招聘页面](https://zed.dev/jobs)上的开放职位。

### 许可证

Zed 源代码主要采用 GPL-3.0 或更高版本许可；标明的组件采用 Apache-2.0 许可。

必须正确提供第三方依赖的许可信息，CI 才能通过。

我们使用 [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) 自动遵守开源许可证。如果 CI 失败，请检查：

- 如果你创建的 crate 显示 `no license specified` 错误，请在其 Cargo.toml 的 `[package]` 下添加 `publish = false`。
- 如果某个依赖报告 `failed to satisfy license requirements`，请先确认该项目使用什么许可，以及当前系统是否足以满足其要求。如果不确定，请咨询律师。确认此系统可接受后，将许可证的 SPDX 标识符加入 `script/licenses/zed-licenses.toml` 中的 `accepted` 数组。
- 如果 `cargo-about` 无法找到某个依赖的许可证，请按照 [cargo-about 手册](https://embarkstudios.github.io/cargo-about/cli/generate/config.html#crate-configuration)的说明，在 `script/licenses/zed-licenses.toml` 末尾添加澄清字段。

## 赞助

Zed 由营利性公司 **Zed Industries, Inc.** 开发。

如果你希望为项目提供资金支持，可以通过 GitHub Sponsors 赞助。赞助款直接支付给 Zed Industries，作为公司的普通收入使用。赞助不附带任何特权或权益。
