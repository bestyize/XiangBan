<div align="center">
    <img src="https://raw.githubusercontent.com/luanti-org/luanti/9c424c44ff523ebce076367e52222ac4d71e94c1/textures/base/pack/logo.png" width="32%">
    <h1>Luanti (formerly Minetest)</h1>
    <a href="https://github.com/luanti-org/luanti/actions?query=event%3Apush"><img src="https://github.com/luanti-org/luanti/workflows/linux/badge.svg" alt="Build status"></a>
    <a href="https://hosted.weblate.org/engage/minetest/?utm_source=widget"><img src="https://hosted.weblate.org/widgets/minetest/-/svg-badge.svg" alt="Translation status"></a>
    <a href="https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html"><img src="https://img.shields.io/badge/license-LGPLv2.1%2B-blue.svg" alt="License"></a>
</div>
<br>

> Logo 素材作者为 erle，采用 [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) 许可；详见[固定版本素材许可清单](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/LICENSE.txt)。

Luanti 是一款自由开源的体素游戏引擎，方便制作模组和创建游戏。

版权所有 (C) 2010-2026 Perttu Ahola <celeron55@gmail.com> 及贡献者（参见源文件注释和版本控制日志）

目录
------------------

1. [更多文档](#更多文档)
2. [默认控制](#默认控制)
3. [路径](#路径)
4. [配置文件](#配置文件)
5. [命令行选项](#命令行选项)
6. [编译](#编译)
7. [Docker](#docker)
8. [版本方案](#版本方案)

更多文档
----------------------
- 网站：https://www.luanti.org/
- Luanti 文档：https://docs.luanti.org/
- 论坛：https://forum.luanti.org/
- GitHub：https://github.com/luanti-org/luanti/
- [开发者文档](https://github.com/luanti-org/luanti/tree/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/developing/)
- 源代码发行包的 [doc/](https://github.com/luanti-org/luanti/tree/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/) 目录

默认控制
----------------
所有控制都可以通过设置重新绑定。
部分控制可以在设置选项卡的按键配置对话框中更改。

| 按键 | 操作 |
|---|---|
| 移动鼠标 | 环视 |
| W, A, S, D | 移动 |
| Space | 跳跃/向上移动 |
| Shift | 潜行/向下移动 |
| Q | 丢弃物品堆叠 |
| Shift + Q | 丢弃单个物品 |
| 鼠标左键 | 挖掘/攻击/使用 |
| 鼠标右键 | 放置/使用 |
| Shift + 鼠标右键 | 建造（不使用） |
| I | 物品栏菜单 |
| 鼠标滚轮 | 选择物品 |
| 0-9 | 选择物品 |
| Z | 缩放（需要 zoom 权限） |
| T | 聊天 |
| / | 命令 |
| Esc | 暂停菜单/中止/退出（仅暂停单人游戏） |
| + | 增大视距 |
| - | 减小视距 |
| K | 启用/禁用飞行模式（需要 fly 权限） |
| J | 启用/禁用快速模式（需要 fast 权限） |
| H | 启用/禁用穿墙模式（需要 noclip 权限） |
| E | Aux1（在快速模式下快速移动。游戏可能添加特殊功能） |
| C | 切换摄像机模式 |
| V | 切换小地图模式 |
| Shift + V | 更改小地图方向 |
| F1 | 隐藏/显示 HUD |
| F2 | 隐藏/显示聊天 |
| F3 | 禁用/启用雾 |
| F4 | 禁用/启用摄像机更新（禁用后 Mapblock 不再更新；发布构建中禁用） |
| F5 | 循环切换调试信息屏幕 |
| F6 | 循环切换性能分析信息屏幕 |
| F10 | 显示/隐藏控制台 |
| F12 | 截图 |

路径
-----
位置：

* `bin` - 编译后的二进制文件
* `share` - 分发的只读数据
* `user` - 用户创建的可修改数据

各平台上的位置：

* Windows .zip / RUN_IN_PLACE 源代码：
    * `bin` = `bin`
    * `share` = `.`
    * `user` = `.`
* Windows 安装版：
    * `bin` = `C:\Program Files\Minetest\bin (Depends on the install location)`
    * `share` = `C:\Program Files\Minetest (Depends on the install location)`
    * `user` = `%APPDATA%\Minetest` 或 `%MINETEST_USER_PATH%`
* Linux 安装版：
    * `bin` = `/usr/bin`
    * `share` = `/usr/share/minetest`
    * `user` = `~/.minetest` 或 `$MINETEST_USER_PATH`
* macOS：
    * `bin` = `Contents/MacOS`
    * `share` = `Contents/Resources`
    * `user` = `Contents/User` 或 `~/Library/Application Support/minetest` 或 `$MINETEST_USER_PATH`

世界以独立文件夹形式存放在：`user/worlds/`

配置文件
------------------
- 默认位置：`user/minetest.conf`
- 首次关闭 Luanti 时创建此文件。
- 可通过命令行指定特定文件：`--config <path-to-file>`
- 原地运行构建会在 `location_of_exe/../minetest.conf` 和 `location_of_exe/../../minetest.conf` 中查找配置文件。

命令行选项
--------------------
- 使用 `--help`

编译
---------

- [编译 - 通用信息](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/compiling/README.md)
- [在 GNU/Linux 上编译](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/compiling/linux.md)
- [在 Windows 上编译](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/compiling/windows.md)
- [在 MacOS 上编译](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/compiling/macos.md)

Docker
------

- [使用 Docker 开发 minetestserver](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/developing/docker.md)
- [使用 Docker 运行服务器](https://github.com/luanti-org/luanti/blob/9c424c44ff523ebce076367e52222ac4d71e94c1/doc/docker_server.md)

版本方案
--------------
自 5.0.0-dev 起，我们使用 `major.minor.patch`。此前使用 `0.major.minor`。

- 发行版包含破坏性变更时，主版本号递增，其他数字归零。
- 发行版包含新的非破坏性功能时，次版本号递增，补丁版本号归零。
- 发行版仅包含错误修复和认为必要的非常轻微或琐碎的功能时，补丁版本号递增。

自 5.0.0-dev 和 0.4.17-dev 起，dev 标记表示下一次发布，即 5.0.0-dev 是通向 5.0.0 的开发版本。此前使用 `previous_version-dev`。
