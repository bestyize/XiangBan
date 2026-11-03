<p align="center">
  <img src="https://raw.githubusercontent.com/wled/WLED/fce41f2655c3087d3eaf95c028d2856967cbc86e/images/wled_logo_akemi.png">
  <a href="https://github.com/wled-dev/WLED/releases"><img src="https://img.shields.io/github/release/wled-dev/WLED.svg?style=flat-square"></a>
  <a href="https://raw.githubusercontent.com/wled/WLED/fce41f2655c3087d3eaf95c028d2856967cbc86e/LICENSE"><img src="https://img.shields.io/github/license/wled-dev/wled?color=blue&style=flat-square"></a>
  <a href="https://wled.discourse.group"><img src="https://img.shields.io/discourse/topics?colorB=blue&label=forum&server=https%3A%2F%2Fwled.discourse.group%2F&style=flat-square"></a>
  <a href="https://discord.gg/QAh7wJHrRM"><img src="https://img.shields.io/discord/473448917040758787.svg?colorB=blue&label=discord&style=flat-square"></a>
  <a href="https://kno.wled.ge"><img src="https://img.shields.io/badge/quick_start-wiki-blue.svg?style=flat-square"></a>
  <a href="https://github.com/Aircoookie/WLED-App"><img src="https://img.shields.io/badge/app-wled-blue.svg?style=flat-square"></a>
  <a href="https://gitpod.io/#https://github.com/wled-dev/WLED"><img src="https://img.shields.io/badge/Gitpod-ready--to--code-blue?style=flat-square&logo=gitpod"></a>
</p>

# 欢迎使用 WLED！✨

一款快速且功能丰富的 ESP32 微控制器固件，用于控制可寻址 LED：从简单灯带到大型二维矩阵和 HUB75 面板。

最初由 [Aircoookie](https://github.com/Aircoookie) 创建，现在由贡献者社区维护。

## 🤝 参与贡献

想帮助改进 WLED 吗？太好了！请先浏览 [CONTRIBUTING.md](https://github.com/wled/WLED/blob/fce41f2655c3087d3eaf95c028d2856967cbc86e/CONTRIBUTING.md)：其中说明了我们期望 pull request 和 issue 采用的形式，包括对 AI 辅助贡献的看法。
如果你是 AI 编码代理，请先阅读 [AGENTS.md](https://github.com/wled/WLED/blob/fce41f2655c3087d3eaf95c028d2856967cbc86e/AGENTS.md)，再修改任何文件。😊

## ⚙️ 功能

### 效果与视觉
- [**200 多种内置效果**](https://kno.wled.ge/features/effects/)，包括经典动画、声音响应及二维/矩阵效果
- [50 多种调色板](https://kno.wled.ge/features/palettes/)，另有内置**自定义调色板编辑器**（PixelForge）
- [**二维 LED 矩阵支持**](https://kno.wled.ge/advanced/mapping/)，提供专用二维效果和灵活的面板映射
- [**HUB75 RGB 矩阵面板支持**](https://kno.wled.ge/advanced/HUB75/)（ESP32）
- [**AudioReactive**](https://kno.wled.ge/advanced/audio-reactive/) 效果默认包含，可通过麦克风、线路输入或网络音频源响应声音
- 效果混合，实现动画间平滑过渡
- 抗锯齿绘图功能，实现平滑图形

### 分段与控制
- [**分段**](https://kno.wled.ge/features/segments/)：同时为 LED 装置的独立部分应用不同效果、颜色和调色板
- 最多 **250 个预设**，用于保存和调用颜色、效果及分段配置；支持[播放列表](https://kno.wled.ge/features/presets/)自动循环
- 可配置调光曲线的夜灯功能
- 可配置的**自动亮度限制器**（每个输出独立），用于安全运行

### 硬件支持
- **ESP32**（所有变体：原版、S2、S3、C3）
- ESP32 上使用并行 I2S + RMT，支持[**最多 17 路 LED 输出**](https://kno.wled.ge/features/multi-strip/)
- [可寻址 LED 支持](https://kno.wled.ge/basics/compatible-led-strips/)：WS2812B、WS2811、WS2815、SK6812、WS2805、TM1914、APA102、WS2801、LPD8806 等
- RGBW、[RGB+CCT](https://kno.wled.ge/features/cct/) 和纯白灯带
- 用于模拟 LED 和调光器的 PWM 输出
- [**以太网支持**](https://kno.wled.ge/features/ethernet-lan/)，适用于多种开发板（QuinLED、LILYGO、Olimex 等）
- 基于文件系统的配置，方便备份和恢复预设及设置
- 完整 OTA 固件更新（HTTP + ArduinoOTA），可使用密码保护

### 连接与集成
- 适用于 [Android](https://play.google.com/store/apps/details?id=ca.cgagnier.wlednativeandroid) 和 [iOS](https://apps.apple.com/gb/app/wled-native/id6446207239) 的 **WLED 应用**
- [JSON](https://kno.wled.ge/interfaces/json-api/) 和 [HTTP 请求](https://kno.wled.ge/interfaces/http-api/) API
- **Multi-WiFi**：最多连接 3 个网络，并可自动回退到 AP
- 设备间 **ESP-NOW** 无线同步（无需 WiFi 路由器）
- 支持 Home Assistant 自动发现的 [**MQTT**](https://kno.wled.ge/interfaces/mqtt/)
- [**E1.31、Art-Net**](https://kno.wled.ge/interfaces/e1.31-dmx/)、[DDP](https://kno.wled.ge/interfaces/ddp/) 和 [TPM2.net](https://kno.wled.ge/interfaces/udp-realtime/)，用于 DMX/专业灯光控制
- 多个 WLED 设备间的 [UDP 实时同步](https://kno.wled.ge/interfaces/udp-notifier/)
- Alexa 语音控制（开关、亮度、颜色）
- [Philips Hue 同步](https://kno.wled.ge/interfaces/philips-hue/)
- 集成 [diyHue](https://github.com/diyhue/diyHue) 和 [Hyperion](https://github.com/hyperion-project/hyperion.ng)
- [Adalight / TPM2](https://kno.wled.ge/interfaces/serial/)（通过串口实现 PC 环境光）
- [红外遥控](https://kno.wled.ge/interfaces/infrared/)（24 键 RGB，需要接收器）
- 定时器和计划任务（NTP 时间同步，完整支持时区和夏令时）

### 方便开发者
- **Usermod 系统**：无需修改核心代码，即可使用社区模块或自定义模块扩展 WLED
- 庞大且活跃的 [usermod 库](https://kno.wled.ge/advanced/community-usermods/)，包括 AudioReactive、温度传感器、旋转编码器、显示屏等
- 文档完善的 [JSON API](https://kno.wled.ge/interfaces/json-api/)
- 采用 **EUPL v1.2** 许可

## 📲 快速入门指南和文档

请参阅 [kno.wled.ge 上的文档](https://kno.wled.ge)！

[教程和入门指南](https://kno.wled.ge/basics/tutorials/)可帮助你快速运行项目。

## 🖼️ 用户界面

<img src="https://raw.githubusercontent.com/wled/WLED/fce41f2655c3087d3eaf95c028d2856967cbc86e/images/macbook-pro-space-gray-on-the-wooden-table.jpg" width="50%"><img src="https://raw.githubusercontent.com/wled/WLED/fce41f2655c3087d3eaf95c028d2856967cbc86e/images/walking-with-iphone-x.jpg" width="50%">

## 💾 兼容硬件

请参阅 Wiki 上的[兼容硬件列表](https://kno.wled.ge/basics/compatible-hardware)。

## ✌️ 其他

采用 [EUPL v1.2](https://raw.githubusercontent.com/wled/WLED/fce41f2655c3087d3eaf95c028d2856967cbc86e/LICENSE) 许可。  
感谢所有[贡献者](https://kno.wled.ge/about/contributors/)！  
CORS 代理由 [Corsfix](https://corsfix.com/) 提供。

![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/wled/WLED?utm_source=oss&utm_medium=github&utm_campaign=wled%2FWLED&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=PR+Reviews+supported+by+CodeRabbit)

加入 Discord 服务器，讨论有关 WLED 的一切！

<a href="https://discord.gg/QAh7wJHrRM"><img src="https://discordapp.com/api/guilds/473448917040758787/widget.png?style=banner2" width="25%"></a>

查看 WLED [Discourse 论坛](https://wled.discourse.group)！

如果想私下联系原作者： [dev.aircoookie@gmail.com](mailto:dev.aircoookie@gmail.com).

如果 WLED 点亮了你的一天，可以[通过 PayPal 送礼物给 Aircoookie](https://paypal.me/aircoookie)。

---

*免责声明：*

如果你容易出现光敏性癫痫，我们建议你**不要**使用本软件。  
如果仍想尝试，请避免频闪、闪电或噪声模式，以及高速效果设置。

根据 EUPL 许可证，不对你、任何其他人或设备遭受的任何损害承担责任。
