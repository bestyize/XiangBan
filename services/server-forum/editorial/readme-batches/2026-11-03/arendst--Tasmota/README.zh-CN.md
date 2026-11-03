> 根据 [Tasmota 固定版本 README](https://github.com/arendst/Tasmota/blob/568220d493f13cadec7c82c66500f71190790aae/README.md) 翻译；固件刷写和市电设备存在风险，请完整阅读下文警告和最新官方文档。项目采用 GPL-3.0-only。

![Tasmota logo](https://raw.githubusercontent.com/arendst/Tasmota/568220d493f13cadec7c82c66500f71190790aae/tools/logo/TASMOTA_FullLogo_Vector.svg#gh-light-mode-only)![Tasmota logo](https://raw.githubusercontent.com/arendst/Tasmota/568220d493f13cadec7c82c66500f71190790aae/tools/logo/TASMOTA_FullLogo_Vector_White.svg#gh-dark-mode-only)

适用于基于 [ESP8266](https://en.wikipedia.org/wiki/ESP8266) 和 [ESP32](https://en.wikipedia.org/wiki/ESP32) 的设备的替代固件，支持**通过 Web 界面轻松配置、OTA 更新、使用定时器或规则实现自动化、扩展，以及通过 MQTT、HTTP、串口或 KNX 完全本地控制**。
_为 PlatformIO 编写。_

[![GitHub version](https://img.shields.io/github/release/arendst/Tasmota.svg)](http://ota.tasmota.com/tasmota/release)
[![GitHub download](https://img.shields.io/github/downloads/arendst/Tasmota/total.svg)](https://github.com/arendst/Tasmota/releases/latest)
[![License](https://img.shields.io/github/license/arendst/Tasmota.svg)](https://github.com/arendst/Tasmota/blob/568220d493f13cadec7c82c66500f71190790aae/LICENSE.txt)
[![Discord](https://img.shields.io/discord/479389167382691863.svg?logo=discord&logoColor=white&color=5865F2&label=Discord)](https://discord.gg/Ks2Kzd4)

<hr></hr>

**鉴于当前事件，我们希望支持 _PlatformIO 项目_背后的人们，特别是 Ivan Kravets，并祝愿他们有力量帮助制止战争。你可以做些什么，请参阅 [PlatformIO 是乌克兰项目，请帮助我们制止战争](https://community.platformio.org/t/platformio-is-ukrainian-project-please-help-us-stop-the-war/26330)。**

<hr></hr>

## 为什么选择 Tasmota？

Tasmota 是为 **ESP8266**/**ESP8285** 而生的开源固件，现在覆盖整个 ESP32 家族：**ESP32**、**ESP32-S2**、**ESP32-S3**、**ESP32-C2**、**ESP32-C3**、**ESP32-C5**、**ESP32-C6**、**ESP32-C61**、**ESP32-P4**，并在设备支持时提供 PSRAM 和 USB/CDC 支持。

它可以通过 TLS 使用 MQTT（v3.1.1 或 v5）连接自己的代理服务器或云端代理（[AWS IoT Core](https://tasmota.github.io/docs/AWS-IoT/) 和 [Azure IoT Hub](https://tasmota.github.io/docs/Azure-IoT-Hub/)），也可以完全独立运行，无需任何服务器——由你选择。

以下概览它能做什么：

| 领域 | 功能 |
|---|---|
| **智能家居集成** | 通过[本地或云端 MQTT](https://tasmota.github.io/docs/MQTT/)（[AWS IoT Core](https://tasmota.github.io/docs/AWS-IoT/) 和 [Azure IoT Hub](https://tasmota.github.io/docs/Azure-IoT-Hub/)）连接 [Home Assistant](https://tasmota.github.io/docs/Home-Assistant/)、[Domoticz](https://tasmota.github.io/docs/Domoticz/)、[openHAB](https://tasmota.github.io/docs/openHAB/)、[Node-RED](https://tasmota.github.io/docs/NodeRed/)、[KNX](https://tasmota.github.io/docs/KNX/) 等。参见[智能家居集成](https://tasmota.github.io/docs/Integrations/)。 |
| **Matter 协议** | 在 ESP32 上原生运行，无需 MQTT、集线器或配套应用。可直接与 Apple Home、Google Home、Amazon Alexa 和 Home Assistant 配对。ESP32 还可以将现有 ESP8266 设备桥接到 Matter。参见 [Matter](https://tasmota.github.io/docs/Matter/)。 |
| **传感器和外设** | 支持 150 多种通过 I²C、SPI、1-Wire 和模拟接口连接的传感器和外设，大多数接线后自动检测：温湿度（[DHT11](https://tasmota.github.io/docs/DHT11/)/DHT22、[DS18x20](https://tasmota.github.io/docs/DS18x20/)、[BME280](https://tasmota.github.io/docs/BME280/)、[SHT30](https://tasmota.github.io/docs/SHT30/)）、CO2（[MH-Z19](https://tasmota.github.io/docs/MH-Z19B/)、[SCD4x](https://tasmota.github.io/docs/SCD4x/)）、空气质量/粉尘（PMS5003、SPS30）、能源监测（INA219、Eastron SDM）、运动（PIR、毫米波雷达）、光照（[BH1750](https://tasmota.github.io/docs/BH1750/)）等。参见[外设](https://tasmota.github.io/docs/Peripherals/)和[设备数据库](https://templates.blakadder.com/)。 |
| **家用设备** | 驱动[卷帘和百叶窗](https://tasmota.github.io/docs/Blinds-and-Shutters/)（脉冲、步进或伺服电机）、用于供暖/制冷的[恒温器](https://tasmota.github.io/docs/Thermostat/)控制，并读取[智能仪表接口](https://tasmota.github.io/docs/Smart-Meter-Interface/)（SML、OBIS、Modbus 仪表）的能源和公用事业数据。 |
| **灯光** | 对[灯泡和灯带](https://tasmota.github.io/docs/Lights/)进行调光、色温调节或全彩控制：单通道调光器、CT、RGB、RGBW、RGBCW 以及可寻址 LED 灯带（WS2812、SK6812 等）。 |
| **协议桥接** | 支持[红外](https://tasmota.github.io/docs/Tasmota-IR/)（发送和接收）、[RF](https://tasmota.github.io/docs/RF-Protocol/)、[DALI](https://tasmota.github.io/docs/DALI/)、[Modbus](https://tasmota.github.io/docs/Modbus-Bridge/)、RS-485、[OpenTherm](https://tasmota.github.io/docs/OpenTherm/)、[TWAI](https://tasmota.github.io/docs/TWAI/)/CAN、[LoRa/LoRaWan](https://tasmota.github.io/docs/LoRa-and-LoRaWan-Bridge/)、[HDMI-CEC](https://tasmota.github.io/docs/HDMI_CEC/)、[Telegram](https://tasmota.github.io/docs/Telegram/) 和 SMTP 邮件，以及通用 Arduino MCU 桥接。 |
| **Zigbee 和蓝牙** | 将 ESP32 变为 [Zigbee](https://tasmota.github.io/docs/Zigbee/) 协调器或 [BLE 网关](https://tasmota.github.io/docs/Bluetooth/)（包括 BTHome），无需单独集线器即可桥接设备。 |
| **显示和触摸界面** | 驱动[字符、图形和电子纸显示器](https://tasmota.github.io/docs/Displays/)，并在 ESP32 上使用 [LVGL](https://tasmota.github.io/docs/LVGL/) 渲染丰富的动画触摸界面，支持电阻式或电容式触摸及 SPI 面板。[HASPmota](https://tasmota.github.io/docs/HASPmota/) 使用简单 JSON 模板构建完整界面，无需 LVGL 或 Berry 代码。 |
| **自动化与脚本** | [规则](https://tasmota.github.io/docs/Rules/)和[定时器](https://tasmota.github.io/docs/Timers/)处理设备端自动化；[Berry](https://tasmota.github.io/docs/Berry/) 脚本语言提供完整的程序化控制，同时驱动 LVGL、HASPmota 和动画引擎。 |
| **网络** | 支持有线以太网、[IPv6](https://tasmota.github.io/docs/IPv6/)、[Wi-Fi 范围扩展](https://tasmota.github.io/docs/Range-Extender/)模式、用于安全远程访问的内置 [WireGuard VPN](https://tasmota.github.io/docs/Wireguard/) 客户端，以及为电池供电节点提供加密 ESP-NOW 网状连接的 [TasMesh](https://github.com/arendst/Tasmota/blob/development/info/xdrv_57_tasmesh.md)。 |
| **LED 动画** | 专用的 DSL 引擎可为可寻址 LED 灯带（WS2812、SK6812 等）制作脉冲、呼吸、彗星和闪烁等效果。刷写之前可在[在线浏览器模拟器](https://tasmota.github.io/docs/Tasmota-Berry-emulator/index.html)中设计和预览动画。 |
| **音频** | [I2S 音频](https://tasmota.github.io/docs/I2S-Audio_ESP32/)支持增加麦克风输入和扬声器/DAC 输出，包括 MP3/AAC/OPUS 流媒体。 |
| **更多功能** | 为涂鸦 Wi-Fi/MCU 设备提供 [TuyaMCU](https://tasmota.github.io/docs/TuyaMCU/) 支持、[Shelly Gen2 模拟](https://tasmota.github.io/docs/Shelly-Emulation/)、电池设备的[深度](https://tasmota.github.io/docs/DeepSleep/)/[动态](https://tasmota.github.io/docs/Dynamic-Sleep/)休眠、用于同步设备的[设备组](https://tasmota.github.io/docs/Device-Groups/)、[ArtNet DMX](https://tasmota.github.io/docs/ArtNet/)、ESP32 上的 [TensorFlow Lite](https://tasmota.github.io/docs/TFL/) 和 [ULP 协处理器](https://tasmota.github.io/docs/ULP/)脚本，以及支持可选 SD 卡存储的[文件系统](https://tasmota.github.io/docs/UFS/)。 |

由于闪存大小限制，预编译发行二进制文件无法容纳所有功能——有些功能需要[自行编译](https://tasmota.github.io/docs/Compile-your-build/)。详情参见完整[文档](https://tasmota.github.io/docs)。

<hr></hr>

## 简单安装

可以使用 [Tasmota WebInstaller](https://tasmota.github.io/install/) 轻松完成 Tasmota 初始安装。

如果喜欢 **Tasmota**，请给它加星，或者 fork 后参与贡献！

[![GitHub stars](https://img.shields.io/github/stars/arendst/Tasmota.svg?style=social&label=Star)](https://github.com/arendst/Tasmota/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/arendst/Tasmota.svg?style=social&label=Fork)](https://github.com/arendst/Tasmota/network)
[![donate](https://img.shields.io/badge/donate-PayPal-blue.svg)](https://paypal.me/tasmota)

发行信息参见 [RELEASENOTES.md](https://github.com/arendst/Tasmota/blob/master/RELEASENOTES.md)。

固件二进制文件可从 http://ota.tasmota.com/tasmota/release/ 下载；ESP32 二进制文件可从 http://ota.tasmota.com/tasmota32/release/ 下载。

## 开发

[![Dev Version](https://img.shields.io/badge/development%20version-v16.0.x.x-blue.svg)](https://github.com/arendst/Tasmota)
[![Download Dev](https://img.shields.io/badge/download-development-yellow.svg)](http://ota.tasmota.com/tasmota/)
[![Tasmota CI](https://github.com/arendst/Tasmota/actions/workflows/build_all_the_things.yml/badge.svg)](https://github.com/arendst/Tasmota/actions/workflows/build_all_the_things.yml)
[![Build_development](https://github.com/arendst/Tasmota/actions/workflows/Tasmota_build_devel.yml/badge.svg)](https://github.com/arendst/Tasmota/actions/workflows/Tasmota_build_devel.yml)

详细变更信息请参阅 [CHANGELOG.md](https://github.com/arendst/Tasmota/blob/568220d493f13cadec7c82c66500f71190790aae/CHANGELOG.md)。

除非运行 Tasmota 的设备出现问题或缺少你需要的功能，否则请保持设备原样——既然能工作，就不要做不必要的更改！如果发行版（即 master 分支）在你的设备和配置上出现意外行为，应升级到最新开发版，看看问题是否解决，因为旧发行版或开发构建中的某些错误可能已经修复。

开发分支每个成功编译的提交都会在 http://ota.tasmota.com/tasmota/ 发布新的二进制文件（此网址也可用于 OTA 更新）。重要的是，这些二进制文件基于当前开发代码。我们会尽可能测试这些提交，它们通常相当稳定，但无法对数百种不同设备及所有允许的配置选项进行测试。

请注意，像任何升级一样，设备可能无法按预期工作。必须始终考虑 OTA 升级失败时可能需要通过串口编程接口重新刷写设备。即使是 master 发行版，在升级生产环境中或难以接触的设备之前，也应先尝试测试该设备或类似原型。和往常一样，在开始任何固件更新前备份设备配置。

## 免责声明

:warning: **触电危险** :warning:

如果设备连接市电（交流电源），安装不当会有触电危险。如果不知道如何安装，请联系电工（***注意：***某些国家禁止在没有持证电工在场的情况下安装）。请记住：_**安全第一**_。如果不完全了解自己的操作，不值得让自己、家人和家庭冒险。设备连接市电（交流电源）时，绝不要摆弄设备或尝试通过串口编程接口刷写。

对于使用本软件、安装本软件，或本网站及相关网站成员提供的任何提示、建议、视频等，我们不承担任何责任。

## 注意

除非新设备需要新增功能代码，否则请不要要求添加设备。如果设备未列为模块，请先尝试使用[模板](https://tasmota.github.io/docs/Templates)。如果它未列入 [Tasmota 设备模板仓库](http://templates.blakadder.com)，请自行创建[模板](https://tasmota.github.io/docs/Templates#creating-your-template)。

## 快速安装
从 http://ota.tasmota.com/tasmota/release/ 或 http://ota.tasmota.com/tasmota32/release/ 下载一个发行二进制文件，并[按照安装指南](https://tasmota.github.io/docs/Getting-Started)刷写到硬件。

## 重要的用户编译信息
如果想自行编译 Tasmota，请记住：

- 基于 ESP8285 的设备仅支持 Flash Mode **DOUT**。不要使用 DIO / QIO / QOUT 闪存模式，否则设备可能看似变砖。
- 对于基于 ESP8285 的设备，Tasmota 使用不含 spiffs 的 1M 链接脚本 **1M (no SPIFFS)**，以获得最佳代码空间。
- 要在编译时修改 Tasmota，请使用 `user_config_override.h` 文件。这可确保下载和编译新版本时保留自定义设置。请复制提供的 `user_config_override_sample.h` 文件并添加设置覆盖。

## 配置信息

请参阅[文档](https://tasmota.github.io/docs)中的安装和配置文章。

## 迁移信息

有关迁移到主要版本的说明，请参阅[迁移路径](https://tasmota.github.io/docs/Upgrading#migration-path)。

**不要从 minimal 版本升级到 minimal 版本。这样很可能在某个时候失败，并需要通过串口刷写。** 如果必须使用 minimal 版本，请始终先通过 OTA 更新到同一发行版的完整版本，再应用下一个 minimal 版本。

请注意以下由于动态设置更新而产生的版本断点：

1. 迁移到 **Sonoff-Tasmota 3.9.x**
2. 迁移到 **Sonoff-Tasmota 4.x**
3. 迁移到 **Sonoff-Tasmota 5.14**
4. 迁移到 **Sonoff-Tasmota 6.7.1**（http://ota.tasmota.com/tasmota/release_6.7.1/sonoff.bin）——注意使用下划线，因为旧版本不支持连字符
5. 迁移到 **Tasmota 7.2.0**（http://ota.tasmota.com/tasmota/release-7.2.0/tasmota.bin）

--- 参数存储布局发生重大变化 ---

6. 迁移到 **Tasmota 8.5.1**（http://ota.tasmota.com/tasmota/release-8.5.1/tasmota.bin）

--- 内部 GPIO 功能表示发生重大变化 ---

7. 迁移到 **Tasmota 9.1**（http://ota.tasmota.com/tasmota/release-9.1.0/tasmota.bin.gz）
8. 升级到**最新发行版**（http://ota.tasmota.com/tasmota/release/tasmota.bin.gz）

尽管回退或降级很常见，但由于较新版本会添加或更改设置，该操作从未获得支持。从 **v9.1.0 Imogen** 开始，内部 GPIO 功能表示发生变化，只能回退到安装 **v9.1.0** 之前的最新 GPIO 配置。

## 支持信息

<img src="https://user-images.githubusercontent.com/5904370/68332933-e6e5a600-00d7-11ea-885d-50395f7239a1.png" width=150 align="right" />

支持设备的数据库参见 [Tasmota 设备模板仓库](https://templates.blakadder.com)。

如果需要 **Tasmota** 支持，有以下选择：

### 文档

* [文档网站](https://tasmota.github.io/docs)：了解如何刷写、配置、使用和扩展 Tasmota。
* [常见问题和故障排查](https://tasmota.github.io/docs/FAQ/)：了解常见问题与解决方案。
* [命令信息](https://tasmota.github.io/docs/Commands)：了解 Tasmota 支持的所有命令。

### 社区支持

* [Tasmota 讨论](https://github.com/arendst/Tasmota/discussions)：用于 Tasmota 使用问题、功能请求和项目。
* [Tasmota 用户聊天](https://discord.gg/Ks2Kzd4)：用于支持、故障排查和一般问题；你更有机会从 Tasmota 社区成员那里快速得到答案。
* [搜索 Issues](https://github.com/arendst/Tasmota/issues)：搜索当前或已关闭的 issue，可能找到问题答案。
* [软件问题报告](https://github.com/arendst/Tasmota/issues/new?template=Bug_report.md)：用于报告 Tasmota 软件问题。

### 非官方社区资源
* [Tasmota-DE](https://t.me/TasmotaDE)：与 Tasmota 相关的德语 Telegram 群组。

## 参与贡献

你可以通过以下方式为 Tasmota 做贡献：
- 提交 Pull Request（功能、概念验证、语言文件或修复）
- 测试新发布的功能并报告问题
- 捐赠以购买硬件用于测试和实现，或表达感谢
- 为功能和设备补充缺失的[文档](https://tasmota.github.io/docs)

[![donate](https://img.shields.io/badge/donate-PayPal-blue.svg)](https://paypal.me/tasmota)

## 致谢

帮助项目持续运行的人们：
- Sfromis：提供广泛的用户支持
- Barbudor：提供用户支持以及代码修复和新增功能
- David Lang：初步问题处理和代码优化
- Heiko Krupp：IRSend、HTU21、SI70xx 和 Wemo/Hue 模拟驱动
- Wiktor Schmidt：实现 Travis CI
- Thom Dietrich：PlatformIO 优化
- Marinus van den Broek：EspEasy 基础工作
- Pete Ba：更方便用户使用的能源监测校准
- Lobradov：编译优化技巧
- Flexiti：最初的定时器实现
- reloxx13：[TasmoAdmin](https://github.com/reloxx13/TasmoAdmin) 管理工具
- Joachim Banzhaf：TSL2561 库和驱动
- Andre Thomas：提供许多驱动
- Gijs Noorlander：MHZ19、SenseAir 和更新后的 PubSubClient 驱动
- Erik Montnemery：HomeAssistant Discovery 概念和大量代码调优建议
- Federico Leoni：持续支持 HomeAssistant Discovery
- Aidan Mountford：HSB 支持
- Daniel Ztolnai：实现串口桥接
- Gerhard Mutz：多个传感器与显示器驱动、日出/日落和脚本
- Nuno Ferreira：HC-SR04 驱动
- Adrian Scillato：（安全）修复以及 KNX 的实现和维护
- Gennaro Tortone：实现并维护 Eastron 驱动
- Raymond Mouthaan：管理 Wemos Wiki 信息
- Norbert Richter：[decode-config.py](https://github.com/tasmota/decode-config) 工具
- Joel Stein、digiblur 和 Shantur Rathore：Tuya 研究和驱动
- Frogmore42：回答许多问题
- Jason2866：PlatformIO 支持和大量问题解答
- Blakadder：管理文档网站并提供模板管理
- Stephan Hadinger：重构灯光驱动，增强 HueEmulation、LVGL、Zigbee 和 Berry 支持
- tmo：设计官方 Tasmota 标志
- Stefan Bode：Shutter 和深度休眠驱动
- Jacek Ziółkowski：[TDM](https://github.com/jziolkowski/tdm) 管理工具和 [Tasmotizer](https://github.com/tasmota/tasmotizer) 刷写工具
- Christian Staars：NRF24L01 和 HM-10 蓝牙传感器支持
- Paul Diem：UDP 组通信支持
- Jörg Schüler-Maroldt：最初的 ESP32 移植
- Javier Arigita：恒温器驱动
- Simon Hailes：ESP32 蓝牙扩展
- 以及提供技巧、进行中工作、概念验证、PR 和捐赠的许多人

## 许可证

本程序采用 GPL-3.0-only 许可。
