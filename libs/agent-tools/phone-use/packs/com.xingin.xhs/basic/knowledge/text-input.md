# 中文输入

phone_action(text) 的 Unicode 输入使用已安装且已选中的 ADB Keyboard（com.android.adbkeyboard/.AdbIME）接收 base64 广播。工具不会静默安装 APK 或更改输入法。没有输入法时明确报告依赖，协助用户按其授权配置；不能用乱码或拼音冒充原文。

普通 ASCII 文本可用系统 input text。无论调用返回什么，输入前先聚焦，输入后读取 UI 树或截图核对，注意占位文字不等于已有正文。

小米 17 / Android 16 实测使用上游 `v2.5-dev`（README 标注 A16 fix）的 APK，包名 `com.android.adbkeyboard`。经用户批准安装及临时切换后，定向 ADB_INPUT_B64 广播完成中文标题、中文正文和换行输入，并经页面树与截图回读确认。旧 APK 不因此自动认定兼容 Android 16。

首次配置应记录原输入法，测试结束后恢复并停用调试输入法。本模块不会自行安装或切换；需要配置时由宿主按用户授权处理。来源：https://github.com/senzhk/ADBKeyBoard
