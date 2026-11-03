![Logo](https://raw.githubusercontent.com/Anuken/Mindustry/b873c01ed559a8803c2dad5c5c79d7ad750d5b95/core/assets-raw/sprites/ui/logo.png)

[![Build Status](https://github.com/Anuken/Mindustry/workflows/Tests/badge.svg?event=push)](https://github.com/Anuken/Mindustry/actions)
[![Discord](https://img.shields.io/discord/391020510269669376.svg?logo=discord&logoColor=white&logoWidth=20&labelColor=7289DA&label=Discord&color=17cf48)](https://discord.gg/mindustry)  

一款用 Java 编写的自动化塔防即时战略游戏。

_[Trello 看板](https://trello.com/b/aE2tcUwF/mindustry-40-plans)_  
_[Wiki](https://mindustrygame.github.io/wiki)_  
_[Javadoc](https://mindustrygame.github.io/docs/)_ 

## 贡献

请参阅 [CONTRIBUTING](https://github.com/Anuken/Mindustry/blob/b873c01ed559a8803c2dad5c5c79d7ad750d5b95/CONTRIBUTING.md)了解一般代码风格和 PR 指南。

如果你是首次贡献者，正在寻找可实现的功能或可修复的问题，请查看 [Mindustry-Suggestions 仓库](https://github.com/Anuken/Mindustry-Suggestions/issues?q=is%3Aissue%20state%3Aopen%20label%3Acandidate)中标记为 candidate 的问题。

## 构建

每次提交都会自动生成最新开发构建，可在[这里](https://github.com/Anuken/MindustryBuilds/releases)查看.

如果想自行编译，请按以下说明操作。
首先确保已安装 [JDK 17](https://adoptium.net/temurin/releases/?os=any&arch=any&version=17)。**其他 JDK 版本无法使用。** 在 Mindustry 目录中打开终端并运行以下命令：

### Windows

_运行：_ `gradlew desktop:run`  
_构建：_ `gradlew desktop:dist`  
_精灵图打包：_ `gradlew tools:pack`

### Linux/Mac OS

_运行：_ `./gradlew desktop:run`  
_构建：_ `./gradlew desktop:dist`  
_精灵图打包：_ `./gradlew tools:pack`

### 服务器

每个发布构建（Releases）均包含服务器构建。如果想自行编译，请将 'desktop' 替换为 'server'，例如 `gradlew server:dist`。

### Android

1. 在[这里安装 Android SDK。](https://developer.android.com/studio#command-tools) 请确保下载“仅命令行工具”，不需要 Android Studio。
2. 在解压后的 Android SDK 文件夹中找到 cmdline-tools 目录。在该目录内创建名为 `latest` 的文件夹，并将目录中的全部内容放入新文件夹。
3. 在同一目录运行 `sdkmanager --licenses`（Linux/macOS 使用 `./sdkmanager --licenses`）。
4. 将 `ANDROID_HOME` 环境变量设置为解压后的 Android SDK 目录。
5. 在设备或模拟器上启用开发者模式。如果使用手机测试，请参阅[这些说明](https://developer.android.com/studio/command-line/adb#Enabling),；如果使用模拟器，请搜索如何为该模拟器启用开发者模式。
6. 运行 `gradlew android:assembleDebug`（Linux/macOS 使用 `./gradlew`）。这会在 `android/build/outputs/apk` 中创建未签名的 APK。

要在已连接的设备或模拟器上调试应用，请运行 `gradlew android:installDebug android:run`。

### 故障排除

#### 权限被拒绝

如果 Mac/Linux 终端返回 `Permission denied` 或 `Command not found`，请在运行 `./gradlew` 前运行 `chmod +x ./gradlew`。*此操作只需进行一次。*

#### `mindustry.gen` 包在哪里？

顾名思义，`mindustry.gen` 是根据其他代码在*构建时*生成的。仓库中没有这个包的源代码，也不应手动编辑。

以下是 `mindustry.gen` 中生成代码“来源”的部分列表：

- `Call`、`*Packet` 类：由标记为 `@Remote` 的方法生成。
- 所有实体类（`Unit`、`EffectState`、`Posc` 等）：由 `mindustry.entities.comp` 包中的组件类生成，并使用 `mindustry.content.UnitTypes` 中的定义组合。
- `Sounds`、`Musics`、`Tex`、`Icon` 等：根据相应资源文件夹中的文件生成。

---

Gradle 下载文件可能需要几分钟，请耐心等待。 <br>
构建完成后，桌面构建的 .JAR 输出文件应位于 `/desktop/build/libs/Mindustry.jar`，服务器构建则位于 `/server/build/libs/server-release.jar`。

## 功能请求

请在[这里](https://github.com/Anuken/Mindustry-Suggestions/issues/new/choose)提交功能请求和反馈.

## 下载

| [![](https://static.itch.io/images/badge.svg)](https://anuke.itch.io/mindustry)    |    [![](https://play.google.com/intl/en_us/badges/images/generic/en-play-badge.png)](https://play.google.com/store/apps/details?id=io.anuke.mindustry)   |    [![](https://fdroid.gitlab.io/artwork/badge/get-it-on.png)](https://f-droid.org/packages/io.anuke.mindustry)	| [![](https://flathub.org/assets/badges/flathub-badge-en.svg)](https://flathub.org/apps/details/com.github.Anuken.Mindustry)  
|---	|---	|---	|---	|
