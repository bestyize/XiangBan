# Windows 原生锁屏组件原型

这是独立于小伴 NSIS 安装版的 Windows App SDK 组件包。它注册一个小尺寸的「小伴天气（测试）」组件，用固定温度和回调时间验证系统锁屏的添加、显示与刷新。

## 打包

在 Windows 11 x64 的开发者 PowerShell 使用 .NET 8 SDK、Windows SDK 的 MakeAppx 和 SignTool，以及当前用户证书库中的受信任代码签名证书。

`./Build-Prototype.ps1 -Output ./XiaobanLockScreenWidget.msix -CertificateThumbprint <证书指纹>`

## 验收

1. 安装签名的 MSIX，并在「设置 → 个性化 → 锁屏界面 → 小组件」选择「添加小组件」。
2. 添加「小伴天气（测试）」，按 Win + L，检查温度示例和时间。重新锁屏或重新添加时记录系统是否触发更新。
3. 验证与小伴 NSIS 版同时安装、MSIX 覆盖更新、移除组件，以及卸载原型包。

系统能发现并展示该组件后，再接小伴的天气数据和组件菜单。微软公开的锁屏小组件设置及包身份规则适用。[锁屏设置](https://support.microsoft.com/en-us/windows/experience/personalization/customize-the-lock-screen-in-windows) · [Win32 组件注册](https://learn.microsoft.com/en-us/windows/apps/develop/widgets/widget-provider-manifest)
