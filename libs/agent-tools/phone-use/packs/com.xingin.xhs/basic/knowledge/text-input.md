# 中文输入

phone_action(text) 的 Unicode 输入使用已安装且已选中的 ADB Keyboard（com.android.adbkeyboard/.AdbIME）接收 base64 广播。工具不会静默安装 APK 或更改输入法。没有输入法时明确报告依赖，协助用户按其授权配置；不能用乱码或拼音冒充原文。

普通 ASCII 文本可用系统 input text。无论调用返回什么，输入前先聚焦，输入后读取 UI 树或截图核对，注意占位文字不等于已有正文。
