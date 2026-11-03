# ![MuseScore Studio](https://raw.githubusercontent.com/musescore/MuseScore/f4194eb421cbdc51f6758ba91a96d3495e9c61aa/share/icons/musescore_logo_full.png)

乐谱记谱与作曲软件

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![Coverage](https://s3.us-east-1.amazonaws.com/extensions.musescore.org/test/code_coverage/coverage_badge.svg?)](https://github.com/musescore/MuseScore/actions/workflows/check_unit_tests.yml)

MuseScore Studio 是免费开源的乐谱记谱软件。有关支持、贡献和错误报告，请访问 MuseScore.org。欢迎 fork 并提交 pull request！

## 功能

- 所见即所得设计，在“虚拟乐谱纸”上输入音符
- 使用 TrueType 字体打印和显示，可高质量缩放至各种尺寸
- 简单快速地输入音符
- 多种编辑功能
- MusicXML 导入和导出
- MIDI（SMF）导入和导出
- MEI 导入和导出
- MuseData 导入
- 使用 MIDI 输入音符
- 集成音序器和软件合成器以播放乐谱
- 打印或创建 PDF 文件

## 更多信息

- [MuseScore Studio 主页](https://musescore.org)
- [MuseScore Studio Git 工作流说明](https://musescore.org/en/developers-handbook/git-workflow)
- [如何编译 MuseScore Studio？](https://github.com/musescore/MuseScore/wiki/Set-up-developer-environment)

## 许可证

MuseScore Studio 采用 GPL 3.0 许可。请参阅同一目录中的[许可证文件](https://github.com/musescore/MuseScore/blob/f4194eb421cbdc51f6758ba91a96d3495e9c61aa/LICENSE.txt)。

## 软件包

参阅 [Wiki 上的代码结构](https://github.com/musescore/MuseScore/wiki/CodeStructure)。

## 构建

**有关完整构建步骤和依赖列表，请阅读 [MuseScore Wiki](https://github.com/musescore/MuseScore/wiki) 的[编译章节](https://github.com/musescore/MuseScore/wiki/Set-up-developer-environment)。**

### 获取源代码

如果使用 Git 下载包含完整代码历史的仓库，请输入：

    git clone  --recurse-submodules https://github.com/musescore/MuseScore.git
    cd MuseScore

（`--recurse-submodules` 确保将 [`muse_framework`](https://github.com/musescore/muse_framework) Git 子模块检出到 `muse/` 子目录。）

否则，你也可以从[发布页面](https://github.com/musescore/MuseScore/releases)下载最新的源代码发行 tarball，然后在下载目录中输入：

    tar xzf MuseScore-x.x.x.tar.gz
    cd MuseScore-x.x.x

### Release 构建

要编译 MuseScore Studio 的发布版本，请输入：

    cmake -P build.cmake -DCMAKE_BUILD_TYPE=Release

在 MacOS 上，追加 `-G Ninja` 以使用 `ninja` 构建工具（可能需要安装），因为编译 Swift 组件需要它。

如果出现问题，在上述命令后追加单词“clean”以删除构建子目录：

    cmake -P build.cmake -DCMAKE_BUILD_TYPE=Release clean

然后重新运行第一条命令。

### 运行

要启动 MuseScore Studio，请输入：

    cmake -P build.cmake -DCMAKE_BUILD_TYPE=Release run

或者直接运行编译后的可执行文件。

### Debug 构建

将上述命令中的 `-DCMAKE_BUILD_TYPE=Release` 替换为 `-DCMAKE_BUILD_TYPE=Debug`，即可构建和运行调试版本。

如果完全省略 `-DCMAKE_BUILD_TYPE` 选项，默认使用 `RelWithDebInfo`，以便在 Release 和 Debug 之间取得实用的折中。

### 测试

有关如何运行测试套件的说明，请参阅 [MuseScore Studio Wiki](https://github.com/musescore/MuseScore/wiki) 的[单元测试章节](https://github.com/musescore/MuseScore/wiki/Unit-tests)。

### 代码格式

运行 `./hooks/install.sh` 安装 pre-commit 钩子，以格式化暂存文件。需要先安装 `uncrustify`。

如果遇到问题，请报告。卸载时运行 `./hooks/uninstall.sh`。
