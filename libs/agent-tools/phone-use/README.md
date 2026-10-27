# Android 手机操作工具

独立于 Tauri 的 Rust 工具包。设计与首轮边界见 [实现方案](../../../docs/agent/android-phone-use.md)。

应用包以 `<packagename>/skill.md + basic/ + patch/` 保存。`basic/package.json` 提供应用摘要；`tasks/`、`pages/`、`knowledge/`、`assets/` 按需读取；`routes/core.yaml` 与 `routes/flows/*.yaml` 表达核心骨架和功能路径。

路由支持 core / feature / temporary 角色及独立优先度、稳定性、可信度、边成本。基础包不接受临时路由。补丁使用带版本与证据的 YAML frontmatter，临时路径必须到期。普通文件完整替换，地图按稳定 ID 合并。基础包更新保留补丁，但基础版本变化后旧补丁停止自动应用，等待重新验证。

`Store::install` 接受固定 SHA-256 的 ZIP，仅允许 `skill.md` 与 `basic/`。最多 512 个 ZIP 条目、压缩 12 MiB、解压 32 MiB、单文件 4 MiB；拒绝目录穿越、链接、Windows 保留名、大小写重复和脚本。更新通过暂存与回滚替换。仅保证单进程内读写互斥，宿主应保持单实例；不要由外部进程同时修改包目录。

`Store::read` 的 offset / next_offset 是 UTF-8 字节；单页最多 8192 字节。补丁摘要由 `Store::patches` 动态生成。图片须使用媒体读取工具，不能作为 UTF-8 文本读入。

验证（仓库根）：

```sh
cargo test --manifest-path libs/agent-tools/phone-use/Cargo.toml
```

首轮种子知识不代表当前 APP 真机适配，`verified_app_versions` 为空。任务执行必须读取实际页面、核验用户目标，不能把基础包描述或学习证据当成已完成结果。
