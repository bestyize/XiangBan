# 小伴 macOS 固定自签名证书

此目录用于在未购买 Apple Developer Program 期间，让 macOS 版本继续使用同一张自签名代码签名证书。它不是 Apple Developer ID，**不会获得 Gatekeeper 信任或 Apple 公证**。

## 文件和有效期

- `macos-xiaoban.p12.base64`：包含私钥和证书的 PKCS#12 归档，以 base64 文本存放。
- `macos-xiaoban.p12.password`：PKCS#12 解密密码。
- 证书主题：`CN=XiangBan Self-Signed Code Signing 2026, O=XiangBan, OU=Desktop Apps`。
- 代码签名身份指纹（SHA-1）：`E8B36162292F910859DFDFD66D81D8D265764779`；本地配置按指纹选择证书，同名但不同证书不能代替。
- 证书到期时间：2036-10-17 UTC。
- 证书 SHA-256 指纹：`04:D1:75:03:D9:3E:E0:D6:8F:1F:40:E0:74:33:1B:62:31:FA:6C:6D:0E:04:C4:1C:F6:DA:1F:1E:FB:C9:57:05`。
- PKCS#12 文件 SHA-256：`d6be6ee1d4e50bfe3d4d6a861b347120486fd97aae4d53f137aa0e4b4fdc77a5`。

## 严重安全提示

按仓库所有者的明确要求，**私钥归档和密码都已提交到同一个私有仓库**。这意味着密码对拥有仓库读取权限的人**没有保密作用**；获得这两份文件的任何人都能够使用该证书签名伪装成小伴的程序。私有仓库不等于不可泄露，Git 历史也会长期保留文件。

务必限制仓库读权限。不要把此目录复制到公开仓库，不要将此自签名证书安装为终端用户机器上的受信任系统根证书。后续面向公众正式发布时，建议换成 Apple Developer ID + 公证，并把新私钥和密码分别放入 Actions Secrets。若证书已泄露，需要更换证书，可能导致一次新的 TCC 权限重新授权。

## 构建行为

`.github/workflows/build-xiaoban-wallpaper.yml` 的 macOS job 从仓库读取 PKCS#12 和密码，在 GitHub 托管的临时 macOS runner 上导入钥匙串、临时信任自签名证书、使用 `APPLE_SIGNING_IDENTITY` 指定固定证书，并在上传 Release 前检查签名及 designated requirement。如果导入或校验失败，工作流应停止，不回退到 ad-hoc 发布。

本地与正式发布均默认使用同一张固定证书，不再回退到 ad-hoc。`identity.mjs` 在构建机准备签名身份：优先使用已有有效身份，缺失时从本目录固定归档导入专用钥匙串；本地文件位于 `~/Library/Application Support/XiaoBanBuildSigning`，目录权限 0700、密码文件 0600。只在当前用户域为代码签名用途信任证书，不添加系统根或 TLS 信任。专用钥匙串解锁后优先搜索，保留原有搜索列表条目，避免同一证书的旧私钥副本造成签名弹窗或卡住。脚本不随安装包分发，最终用户无需导入证书。

`npm run desktop:macos:build` 为主应用、内置引擎及辅助程序使用稳定证书；`npm run desktop:build` 在 macOS 也会为未打包二进制签名。`npm run desktop` 通过 Cargo target runner 在每次重编译后生成并签名 `<二进制目录>/xiaoban-dev/小伴.app` 再启动，保持原有开发数据目录和热更新。直接运行 `cargo run` / `npx tauri dev` 不经过这个开发启动器，不能代替上述命令验证权限身份。证书缺失、钥匙串失败或签名仍绑定 `cdhash` 时中止构建。`APPLE_SIGNING_IDENTITY=-` 被明确拒绝；CI 可以继续提供固定证书指纹。这份证书不应在每次构建时重新生成。

针对性验证：`XIAOBAN_TEST_SIGNING=1 npm --prefix client/xiaoban-desktop-wallpaper run test:computer-use`，其中真实签名测试会替换测试 App 的原生二进制，核对前后 designated requirement 一致；不读取用户屏幕、不更改 TCC。默认不带此变量时跳过真实签名测试，纯平台打包测试仍可在其他系统运行。

## 权限和兼容性验收

首次从旧 ad-hoc 安装包过渡时，用户可能需要在「隐私与安全性 → 屏幕与系统音频录制」中删除旧记录、完全退出应用、重新授权。随后在同一台 Mac 上顺序安装两个不同的 **固定自签名** Release 版本：

1. 第一个版本授权后验证截图成功。
2. 安装第二个版本，不清除授权，确认截图仍然成功。
3. 使用 `codesign -dv --verbose=4 /Applications/小伴.app` 核对证书签名；使用 `codesign -dr - /Applications/小伴.app` 检查 designated requirement 在版本间没有绑定 `cdhash`。

跨版本保留 TCC 授权是本方案的验证目标，不代表已经在目标 macOS 系统上实机验证通过；不同 macOS 版本的权限行为仍需测试。没有 Apple Developer ID 时，Gatekeeper 仍会提示应用来自未识别开发者。
