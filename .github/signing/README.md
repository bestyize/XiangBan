# 小伴 macOS 固定自签名证书

此目录用于在未购买 Apple Developer Program 期间，让 macOS 版本继续使用同一张自签名代码签名证书。它不是 Apple Developer ID，**不会获得 Gatekeeper 信任或 Apple 公证**。

## 文件和有效期

- `macos-xiaoban.p12.base64`：包含私钥和证书的 PKCS#12 归档，以 base64 文本存放。
- `macos-xiaoban.p12.password`：PKCS#12 解密密码。
- 证书主题：`CN=XiangBan Self-Signed Code Signing 2026, O=XiangBan, OU=Desktop Apps`。
- 证书到期时间：2036-10-17 UTC。
- 证书 SHA-256 指纹：`04:D1:75:03:D9:3E:E0:D6:8F:1F:40:E0:74:33:1B:62:31:FA:6C:6D:0E:04:C4:1C:F6:DA:1F:1E:FB:C9:57:05`。
- PKCS#12 文件 SHA-256：`d6be6ee1d4e50bfe3d4d6a861b347120486fd97aae4d53f137aa0e4b4fdc77a5`。

## 严重安全提示

按仓库所有者的明确要求，**私钥归档和密码都已提交到同一个私有仓库**。这意味着密码对拥有仓库读取权限的人**没有保密作用**；获得这两份文件的任何人都能够使用该证书签名伪装成小伴的程序。私有仓库不等于不可泄露，Git 历史也会长期保留文件。

务必限制仓库读权限。不要把此目录复制到公开仓库，不要将此自签名证书安装为终端用户机器上的受信任系统根证书。后续面向公众正式发布时，建议换成 Apple Developer ID + 公证，并把新私钥和密码分别放入 Actions Secrets。若证书已泄露，需要更换证书，可能导致一次新的 TCC 权限重新授权。

## 构建行为

`.github/workflows/build-xiaoban-wallpaper.yml` 的 macOS job 从仓库读取 PKCS#12 和密码，在 GitHub 托管的临时 macOS runner 上导入钥匙串、临时信任自签名证书、使用 `APPLE_SIGNING_IDENTITY` 指定固定证书，并在上传 Release 前检查签名及 designated requirement。如果导入或校验失败，工作流应停止，不回退到 ad-hoc 发布。

本地 `tauri.macos.conf.json` 仍默认使用 ad-hoc；正式发布包由工作流提供的环境变量覆盖。这份证书不应在每次构建时重新生成，否则会失去跨版本保持签名身份的意义。

## 权限和兼容性验收

首次从旧 ad-hoc 安装包过渡时，用户可能需要在「隐私与安全性 → 屏幕与系统音频录制」中删除旧记录、完全退出应用、重新授权。随后在同一台 Mac 上顺序安装两个不同的 **固定自签名** Release 版本：

1. 第一个版本授权后验证截图成功。
2. 安装第二个版本，不清除授权，确认截图仍然成功。
3. 使用 `codesign -dv --verbose=4 /Applications/小伴.app` 核对证书签名；使用 `codesign -dr - /Applications/小伴.app` 检查 designated requirement 在版本间没有绑定 `cdhash`。

跨版本保留 TCC 授权是本方案的验证目标，不代表已经在目标 macOS 系统上实机验证通过；不同 macOS 版本的权限行为仍需测试。没有 Apple Developer ID 时，Gatekeeper 仍会提示应用来自未识别开发者。
