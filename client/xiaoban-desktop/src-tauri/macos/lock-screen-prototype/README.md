# macOS 系统动态锁屏原型

原型通过 macOS 26+ 的系统壁纸扩展接口输出一段小伴内置视频、雨滴图层和只读天气示例。系统设置会列出「小伴锁屏原型」，由测试者主动选择。天气显示固定示例数据，目前用于检查图层能否进入真实锁屏。

## 打包

用 Apple 芯片 Mac 和 Xcode 构建正常的小伴 `.app`，然后在 `client/xiaoban-desktop` 运行。

`node scripts/build-lock-screen-prototype.mjs /路径/小伴.app /路径/小伴锁屏原型.dmg`

脚本在临时副本内嵌入原型扩展，沿用小伴的签名身份并校验两层签名。打包脚本需单独运行。

## 真机验收

1. 从原型 DMG 打开小伴，将此副本安装到测试位置并运行一次。若系统未列出扩展，可对测试副本执行 `/usr/bin/pluginkit -a "/路径/小伴.app/Contents/PlugIns/XiaobanLockScreenPrototype.appex"`。
2. 在「系统设置 → 墙纸」选择「小伴锁屏原型」，按 Control + Command + Q。确认系统控件可用、视频和雨滴连续播放、天气示例可见。
3. 验证单屏、外接屏、熄屏唤醒、解锁、连续锁屏和切回原壁纸。检查系统控件与天气示例有没有重叠。
4. 使用 `log stream --predicate 'category == "LockScreenPrototype"' --level info` 检查画面未显示的原因。扩展拒绝无法验证身份的连接或无法识别的系统协议。
5. 在系统设置切回原来的墙纸，必要时使用 `/usr/bin/pluginkit -r "/路径/小伴.app/Contents/PlugIns/XiaobanLockScreenPrototype.appex"` 注销该测试副本的扩展。

Apple 接口和小伴自签名安装包的实际兼容性以真机结果为准。通过后再接入壁纸资源、特效参数和组件设置。

协议声明和 Codable 设置结构参考 Phosphene，授权信息见 [LICENSE-Phosphene.txt](LICENSE-Phosphene.txt)。
