[![GitHub (master)](https://img.shields.io/github/checks-status/openscad/openscad/master.svg?logo=github&label=build&logoColor=black&colorA=f9d72c&style=plastic)](https://github.com/openscad/openscad/actions)
[![CircleCI (master)](https://img.shields.io/circleci/project/github/openscad/openscad/master.svg?logo=circleci&logoColor=black&colorA=f9d72c&style=plastic)](https://circleci.com/gh/openscad/openscad/tree/master)
[![Coverity Scan](https://img.shields.io/coverity/scan/2510.svg?colorA=f9d72c&logoColor=black&style=plastic)](https://scan.coverity.com/projects/2510)

[![Visit our IRC channel](https://kiwiirc.com/buttons/irc.libera.chat/openscad.png)](https://kiwiirc.com/client/irc.libera.chat/#openscad)

# 什么是 OpenSCAD？
<p><a href="https://opencollective.com/openscad/donate"><img align="right" src="https://opencollective.com/openscad/donate/button@2x.png?color=white" width="200"/></a>

OpenSCAD 是用于创建实体 3D CAD 对象的软件。它是自由软件，适用于 Linux/UNIX、MS Windows 和 macOS。</p>

与多数用于创建 3D 模型的自由软件（例如著名应用 Blender）不同，OpenSCAD 关注的是 3D 建模的 CAD 方面，而非艺术方面。因此，如果你计划制作机械零件的 3D 模型，它可能正是你需要的应用；但它可能不是制作计算机动画电影的工具。

OpenSCAD 不是交互式建模器，而更像一个 3D 编译器：它读取描述对象的脚本文件，并据此渲染 3D 模型（见下方示例）。这使设计者可以完全控制建模过程，轻松修改任何步骤或创建由可配置参数定义的设计。

OpenSCAD 提供两种主要建模技术：构造实体几何（CSG）和二维轮廓拉伸。二维轮廓的数据交换格式使用 AutoCAD DXF 文件。除了用于拉伸的二维路径，还可以从 DXF 文件读取设计参数。除 DXF 外，OpenSCAD 还可以读取和创建 STL 和 OFF 格式的 3D 模型。

# 目录

- [开始使用](#开始使用)
- [文档](#文档)
    - [构建 OpenSCAD](#构建-openscad)
        - [前提条件](#前提条件)
        - [获取源代码](#获取源代码)
        - [为 macOS 构建](#为-macos-构建)
        - [为 Linux/BSD 构建](#为-linuxbsd-构建)
        - [在依赖项较旧或缺失的系统上为 Linux/BSD 构建](#在依赖项较旧或缺失的系统上为-linuxbsd-构建)
        - [为 Windows 构建](#为-windows-构建)
        - [编译](#编译)

# 开始使用

可在 <https://www.openscad.org/downloads.html> 下载 OpenSCAD 最新二进制版本。安装方式与其他软件相同。

打开 OpenSCAD 后，窗口中会看到三个区域。左侧区域用于编写代码以建立 3D 对象，右侧区域显示模型的 3D 渲染结果。

让我们做一棵树！在左侧区域输入以下代码：

```scad
cylinder(h = 30, r = 8);
```

按 F5 渲染 3D 模型。现在可以看到作为树干的圆柱体。接下来添加用球体表示的茂密树冠，为此将圆柱体与球体合并。

```scad
union() {
    cylinder(h = 30, r = 8);
    sphere(20);
}
```

但效果还不对！树冠位于树的底部，需要将球体沿 z 轴上移。

```scad
union() {
    cylinder(h = 30, r = 8);
    translate([0, 0, 40]) sphere(20);
}
```

完成！你创建了第一个 3D 模型。还可以组合其他基本形状、集合运算（并集、交集、差集）和变换（旋转、缩放、平移），制作复杂模型。其他语言功能请参阅 [OpenSCAD 手册](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual)。

# 文档

文档请参阅 OpenSCAD 主页（https://www.openscad.org/documentation.html）。

## 构建 OpenSCAD

从源代码构建 OpenSCAD，请遵循下方适用于你的平台的说明。

### 前提条件

构建 OpenSCAD 需要一些库和工具。括号中的版本号是开发时使用的版本；其他版本可能可用，也可能不可用。

如果使用较新版本的 Ubuntu，可以通过 aptitude 安装这些库。如果使用 Mac 或较旧的 Linux/BSD，有构建脚本可从源代码下载并编译这些库。请遵循下方对应平台的说明。

* 支持 C++17 的 C++ 编译器
* [cmake (3.5 ->)](https://cmake.org/)
* [Qt (5.15 ->)](https://qt.io/)
* [QScintilla2 (2.9 ->)](https://riverbankcomputing.com/software/qscintilla/)
* [CGAL (5.4 ->)](https://www.cgal.org/)
 * [GMP (5.x)](https://gmplib.org/)
 * [MPFR (3.x)](https://www.mpfr.org/)
* [boost (1.70 ->)](https://www.boost.org/)
* [OpenCSG (1.4.2 ->)](http://www.opencsg.org/)
* [GLEW (1.5.4 ->)](http://glew.sourceforge.net/)
* [Eigen (3.x)](https://eigen.tuxfamily.org/)
* [glib2 (2.x)](https://developer.gnome.org/glib/)
* [fontconfig (2.10 -> )](https://fontconfig.org/)
* [freetype2 (2.4 -> )](https://freetype.org/)
* [harfbuzz (0.9.19 -> )](https://www.freedesktop.org/wiki/Software/HarfBuzz/)
* [libzip (0.10.1 -> )](https://libzip.org/)
* [Bison (2.4 -> )](https://www.gnu.org/software/bison/)
* [Flex (2.5.35 -> )](http://flex.sourceforge.net/)
* [pkg-config (0.26 -> )](https://www.freedesktop.org/wiki/Software/pkg-config/)
* [double-conversion (2.0.1 -> )](https://github.com/google/double-conversion/)

测试套件的额外要求：
* Python3 (3.9 -> )
* [Ghostscript (10.x ->)](https://www.ghostscript.com/index.html)
* [Catch2 (3.0 ->)](https://github.com/catchorg/Catch2)

### 获取源代码

在系统上安装 git（https://git-scm.com/），然后克隆：

    git clone https://github.com/openscad/openscad.git

这会将最新源代码下载到名为 `openscad` 的目录。

要拉取各子模块（包括 [MCAD 库](https://github.com/openscad/MCAD)），请执行：

    cd openscad
    git submodule update --init --recursive

### 贡献更改

可以访问 https://github.com/openscad/openscad/issues 创建 issue，以规划和讨论更改。

如果希望处理现有 issue 并计划稍后通过 PR 贡献更改，可以在该 issue 的评论中输入：

`/assign-me`

将其分配给自己。

### 为 macOS 构建

前提条件：

* Xcode
* automake、libtool、cmake、pkg-config、wget、meson、python-packaging（建议使用 Homebrew 安装）

安装依赖：

使用以下任一方式构建依赖后，请遵循*编译*部分的说明。

1. **从源代码**

    运行设置环境变量的脚本：

        source scripts/setenv-macos.sh

    然后运行脚本编译所有依赖：

        ./scripts/macosx-build-dependencies.sh

2. **Homebrew**（假定已安装 [Homebrew](https://brew.sh/)）

        ./scripts/macosx-build-homebrew.sh

### 为 Linux/BSD 构建

首先确保已安装 git（软件包常名为 `git-core` 或 `scmgit`）。克隆 Git 仓库后，使用系统包管理器下载并安装上述依赖包。对于部分系统，可使用提供的便捷脚本：

    sudo ./scripts/uni-get-dependencies.sh qt6

安装后检查依赖版本。可使用以下脚本：

    ./scripts/check-dependencies.sh

注意不要在任何位置保留旧的本地副本（`/usr/local/`）。如果所有依赖都存在且版本足够高，可跳到编译说明。

### 在依赖项较旧或缺失的系统上为 Linux/BSD 构建

如果系统依赖库缺失或较旧，可以按以下步骤将新版下载并构建到 `$HOME/openscad_deps`。首先运行设置环境变量的脚本：

    source ./scripts/setenv-unibuild.sh

然后运行脚本，编译上述所有前提库：

    ./scripts/uni-build-dependencies.sh

请注意，这里不包含 gcc、qt、glib2 等大型依赖，只包含较小的依赖（boost、CGAL、opencsg 等）。构建后再次检查依赖：

    ./scripts/check-dependencies.sh

然后按下方编译说明操作。

### 在 Nix 上构建

仓库包含用于本地增量编译的[开发 Nix shell](https://github.com/openscad/openscad/tree/8e8df3ddae295e8667cbc9f6c4fbb7c88b0ec45b/scripts/nix)。

### 为 Windows 构建

OpenSCAD 的 Windows 版通常从 Linux 交叉编译。如果希望在 Windows 上尝试 MSVC 构建，请参阅：https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Building_on_Windows

OpenSCAD 已增加 MSVC 构建支持，构建说明见[使用 MSVC 构建](https://github.com/openscad/openscad/blob/8e8df3ddae295e8667cbc9f6c4fbb7c88b0ec45b/doc/win-build.md)。

交叉构建前，请确保安装 MXE 项目所需的全部依赖（列于 https://mxe.cc/#requirements）。不要自行安装 MXE；下列脚本会将其安装到 `$HOME/openscad_deps/mxe`。

然后安装开发工具以获得 GCC。克隆 Git 仓库后，启动一个全新的 bash shell，并运行设置环境变量的脚本：

    source ./scripts/setenv-mingw-xbuild.sh 64

接着运行脚本下载并编译上述全部前提库：

    ./scripts/mingw-x-build-dependencies.sh 64

请注意，此过程可能耗时数小时并占用数十 GB 磁盘空间，因为它使用 [https://mxe.cc](https://mxe.cc) 系统交叉构建多个库。完成后构建 OpenSCAD 并打包为安装程序：

    ./scripts/release-common.sh mingw64

对于 32 位 Windows 交叉构建，将上述说明中的 64 替换为 32。

### 为 WebAssembly 构建

我们支持使用 [openscad/openscad-wasm](https://github.com/openscad/openscad-wasm) 中预制的 Docker 镜像，为 WebAssembly 和 Emscripten 构建无界面的 OpenSCAD（该仓库也提供用法示例）。

#### 浏览器

以下命令创建 `build-web/openscad.wasm` 和 `build-web/openscad.js`：

```bash
./scripts/wasm-base-docker-run.sh emcmake cmake -B build-web -DCMAKE_BUILD_TYPE=Debug -DEXPERIMENTAL=1
./scripts/wasm-base-docker-run.sh cmake --build build-web -j2
```

[openscad/openscad-playground](https://github.com/openscad/openscad-playground) 使用此 WASM 构建提供包含 OpenSCAD 部分功能的 [Web 界面](https://ochafik.com/openscad2/)。

> [!NOTE]
> 使用调试构建（`-DCMAKE_BUILD_TYPE=Debug`）时，可以在 Firefox 和 Chrome 中设置 C++ 断点（后者[需要扩展](https://developer.chrome.com/docs/devtools/wasm)）。

#### 独立 node.js 构建

以下命令创建可执行的 `build-node/openscad.js`（需要 `node`）：

```bash
./scripts/wasm-base-docker-run.sh emcmake cmake -B build-node -DWASM_BUILD_TYPE=node -DCMAKE_BUILD_TYPE=Debug -DEXPERIMENTAL=1
./scripts/wasm-base-docker-run.sh cmake --build build-node -j2
build-node/openscad.js --help
```

> [!NOTE]
> 使用调试构建（`-DCMAKE_BUILD_TYPE=Debug`）时，可以在 VSCode + Node 中设置 C++ 断点（[需要扩展](https://code.visualstudio.com/docs/nodejs/nodejs-debugging#_debugging-webassembly)）。

#### node.js 模块构建

以下命令创建作为 node 模块的 `build-node/openscad.js`，可导入其他 node 程序：

```bash
./scripts/wasm-base-docker-run.sh emcmake cmake -B build-node -DWASM_BUILD_TYPE=node-module -DCMAKE_BUILD_TYPE=Debug -DEXPERIMENTAL=1
./scripts/wasm-base-docker-run.sh cmake --build build-node -j2
```

使用示例：

```js
import OpenSCAD from "./build-node/openscad.js";

const instance = await OpenSCAD({
    noInitialRun: true,
    print: (text) => {
        console.debug('stdout: ' + text);
    },
    printErr: (text) => {
        console.debug('stderr: ' + text);
    },
});

const inFile = "model.scad";
const outFile = "model.stl";

instance.callMain([
    inFile,
    "-o", outFile,
    "--backend=manifold",
    "--export-format=binstl",
]);

// Model is now generated
```

### 编译

首先运行 `cmake -B build -DEXPERIMENTAL=1`，在 `build` 文件夹中生成 Makefile。

然后运行 `cmake --build build`。最后，在 Linux 上可以以 root 运行 `cmake --install build`。

如果从源代码编译时遇到问题，请在 [GitHub 页面的问题跟踪器](https://github.com/openscad/openscad/issues)创建新 issue。

这个站点及其子页面也可能有帮助：
https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Building_OpenSCAD_from_Sources

构建完成后，可在 `build` 目录中使用 `ctest` 运行测试。

注意：`cmake --build` 和 `ctest` 均接受 `-j N` 参数，以将负载分配到 `N` 个并行进程。

### 在本地运行 CI 工作流

* 安装 [circleci-cli](https://circleci.com/docs/2.0/local-cli/)（需要 API 密钥）。

    *注意*：我们也使用 GitHub Workflows，但仅用于在 Windows 上运行测试（Windows 版本通过以下基于 Linux 的 CircleCI 工作流交叉构建）。另外，[act](https://github.com/nektos/act) 也不喜欢我们的子模块设置。

* 运行 CI 任务：

```bash
	# When "successful", these will fail to upload at the very end of the workflow.
	circleci local execute --job  openscad-mxe-64bit
	circleci local execute --job  openscad-mxe-32bit
	circleci local execute --job  openscad-appimage-64bit
	```

    *注意*：openscad-macos 无法在本地构建。

* 如果 GCC 随机被终止，请为 Docker 分配更多 RAM（例如每个计划并发运行的镜像 4GB）。

* 若要更交互式地调试任务，可以手动操作（查看 .circleci/config.yml 获取实际需要的 Docker 镜像）：

```bash
	docker run --entrypoint=/bin/bash -it openscad/mxe-x86_64-gui:latest
	```

    获得控制台后：

```bash
	git clone https://github.com/%your username%/openscad.git workspace
	cd workspace
	git checkout %your branch%
	git submodule init
	git submodule update

	# Then execute the commands from .circleci/config.yml:
	#    export NUMCPU=2
	#    ...
	#    ./scripts/release-common.sh -snapshot -mingw64 -v "$OPENSCAD_VERSION"
	```
