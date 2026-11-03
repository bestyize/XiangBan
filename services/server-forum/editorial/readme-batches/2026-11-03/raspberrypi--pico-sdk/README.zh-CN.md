# Raspberry Pi Pico SDK

Raspberry Pi Pico SDK（以下简称 SDK）提供头文件、库和构建系统，使你可以使用 C、C++ 或汇编语言为基于 RP 系列微控制器的设备（如 Raspberry Pi Pico 或 Raspberry Pi Pico 2）编写程序。

SDK 旨在提供嵌入式和非嵌入式 C 开发者都熟悉的 API 和编程环境。设备每次运行一个程序，并从常规的 `main()` 方法开始。它支持标准 C/C++ 库，也提供 C 级别的库和 API，以访问 RP 系列微控制器的全部硬件，包括 PIO（可编程输入输出）。

此外，SDK 还提供用于处理定时器、同步、Wi-Fi 和蓝牙网络、USB 以及多核编程的高级库。这些库应足够全面，使应用代码很少或完全不必直接访问硬件寄存器。不过，如果确实需要或更愿意直接访问原始硬件寄存器，SDK 也提供完整且注释充分的寄存器定义头文件，无需查阅数据手册中的地址。

SDK 可用于构建各种软件，从简单应用、MicroPython 这样的完整运行环境，到 RP 系列微控制器片上 bootrom 本身这样的底层软件。

整个 SDK 的设计目标是简单而强大。

尚未准备纳入 SDK 的其他库和 API 可在 [pico-extras](https://github.com/raspberrypi/pico-extras) 中找到。

# 文档

有关如何设置硬件、IDE 或开发环境，以及如何为 Raspberry Pi Pico 和其他基于 RP 系列微控制器的设备构建和调试软件，请参阅 [Raspberry Pi Pico 系列入门指南](https://rptl.io/pico-get-started)。

有关编写连接互联网的 Raspberry Pi Pico W 应用，请参阅 [将 Raspberry Pi Pico W 连接到互联网](https://rptl.io/picow-connect)。

有关使用 SDK 编程、更高级功能以及完整的 PDF 格式 API 文档，请参阅 [Raspberry Pi Pico 系列 C/C++ SDK](https://rptl.io/pico-c-sdk)。

HTML 格式 API 文档请参阅[在线 Raspberry Pi Pico SDK API 文档](https://rptl.io/pico-doxygen)。

# 示例代码

可构建的示例代码请参阅 [pico-examples](https://github.com/raspberrypi/pico-examples)。

# 获取最新 SDK 代码

GitHub 上 `pico-sdk` 的 [master](https://github.com/raspberrypi/pico-sdk/tree/master/) 分支包含 SDK 的*最新稳定版本*。如果需要或想测试即将推出的功能，可以尝试 [develop](https://github.com/raspberrypi/pico-sdk/tree/develop/) 分支。

# 快速开始自己的项目

## 使用 Visual Studio Code

你可以在 VS Code 中安装 [Raspberry Pi Pico Visual Studio Code 扩展](https://marketplace.visualstudio.com/items?itemName=raspberry-pi.raspberry-pi-pico)。

## Unix 命令行

这些说明非常简略，且仅适用于 Linux。有关详细步骤、其他平台的说明以及一般信息，我们建议参阅 [Raspberry Pi Pico 系列 C/C++ SDK](https://rptl.io/pico-c-sdk)。

1. 安装 CMake（至少 3.13 版）、python 3、本机编译器和 GCC 交叉编译器。
   ```
   sudo apt install cmake python3 build-essential gcc-arm-none-eabi libnewlib-arm-none-eabi libstdc++-arm-none-eabi-newlib
   ```
1. 设置项目，使其使用 Raspberry Pi Pico SDK。

   * 可以在本地克隆 SDK（最常见）：
      1. 使用 `git clone` 克隆此 Raspberry Pi Pico SDK 仓库。
      1. 将 SDK 中的 [pico_sdk_import.cmake](https://github.com/raspberrypi/pico-sdk/blob/079c6f39023649b154152db30f1d781e884879bc/external/pico_sdk_import.cmake)复制到项目目录。
      2. 在环境中将 `PICO_SDK_PATH` 设置为 SDK 的位置，或稍后将其（`-DPICO_SDK_PATH=`）传给 cmake。
      3. 按如下方式设置 `CMakeLists.txt`：

          ```cmake
          cmake_minimum_required(VERSION 3.13...3.27)

          # initialize the SDK based on PICO_SDK_PATH
          # note: this must happen before project()
          include(pico_sdk_import.cmake)

          project(my_project)

          # initialize the Raspberry Pi Pico SDK
          pico_sdk_init()

          # rest of your project

          ```
   * 或将 Raspberry Pi Pico SDK 用作子模块：
      1. 将 SDK 克隆为名为 `pico-sdk` 的子模块。
      1. 按如下方式设置 `CMakeLists.txt`：

          ```cmake
          cmake_minimum_required(VERSION 3.13...3.27)

          # initialize pico-sdk from submodule
          # note: this must happen before project()
          include(pico-sdk/pico_sdk_init.cmake)

          project(my_project)

          # initialize the Raspberry Pi Pico SDK
          pico_sdk_init()

          # rest of your project

          ```
   * 或从 GitHub 自动下载：
      1. 将 SDK 中的 [pico_sdk_import.cmake](https://github.com/raspberrypi/pico-sdk/blob/079c6f39023649b154152db30f1d781e884879bc/external/pico_sdk_import.cmake)复制到项目目录。
      1. 按如下方式设置 `CMakeLists.txt`：

          ```cmake
          cmake_minimum_required(VERSION 3.13)

          # initialize pico-sdk from GIT
          # (note this can come from environment, CMake cache etc)
          set(PICO_SDK_FETCH_FROM_GIT on)

          # pico_sdk_import.cmake is a single file copied from this SDK
          # note: this must happen before project()
          include(pico_sdk_import.cmake)

          project(my_project)

          # initialize the Raspberry Pi Pico SDK
          pico_sdk_init()

          # rest of your project

          ```
   * 或在本地克隆 SDK，但不复制 `pico_sdk_import.cmake`：
       1. 使用 `git clone` 克隆此 Raspberry Pi Pico SDK 仓库。
       2. 按如下方式设置 `CMakeLists.txt`：

           ```cmake
           cmake_minimum_required(VERSION 3.13)
 
           # initialize the SDK directly
           include(/path/to/pico-sdk/pico_sdk_init.cmake)
 
           project(my_project)
 
           # initialize the Raspberry Pi Pico SDK
           pico_sdk_init()
 
           # rest of your project
 
           ```
1. 编写代码（更多信息请参阅 [pico-examples](https://github.com/raspberrypi/pico-examples) 或 [Raspberry Pi Pico 系列 C/C++ SDK](https://rptl.io/pico-c-sdk) 文档）。

   最简单的例子可能只有一个源文件（例如 hello_world.c）：

   ```c
   #include <stdio.h>
   #include "pico/stdlib.h"

   int main() {
       stdio_init_all();
       printf("Hello, world!\n");
       return 0;
   }
   ```
   然后在 `CMakeLists.txt` 中添加以下内容：

   ```cmake
   add_executable(hello_world
       hello_world.c
   )

   # Add pico_stdlib library which aggregates commonly used features
   target_link_libraries(hello_world pico_stdlib)

   # create map/bin/hex/uf2 file in addition to ELF.
   pico_add_extra_outputs(hello_world)
   ```
   请注意，此示例使用默认 UART 作为 _stdout_；如果想使用默认 USB，请参阅 [hello-usb](https://github.com/raspberrypi/pico-examples/tree/master/hello_world/usb) 示例。

1. 设置 CMake 构建目录。例如，如果不使用 IDE：
      ```
      $ cmake -S . -B build
      ```
      > cmake 的 -S 标志指定源目录，-B 标志告诉 cmake 要创建的输出目录名称。它不必命名为“build”，可以使用任意名称。

   为 Raspberry Pi Pico 以外的开发板构建时，应在上述 `cmake` 命令中传入 `-DPICO_BOARD=board_name`，例如 `cmake -S . -B build -DPICO_BOARD=pico2` 或 `cmake -S . -B build -DPICO_BOARD=pico_w`，以便针对该开发板相应配置 SDK 和构建选项。

   指定 `PICO_BOARD=<boardname>` 会设置各种编译器定义（例如 UART 和其他硬件的默认引脚编号），在某些情况下还会启用其他库（例如为 `PICO_BOARD=pico_w` 构建时的无线支持）；这些库只有在开发板提供所需硬件功能时才能构建。

   SDK 自带的开发板列表见[此目录](https://github.com/raspberrypi/pico-sdk/tree/079c6f39023649b154152db30f1d781e884879bc/src/boards/include/boards)，其中每个指定开发板都有一个头文件。

1. 在创建的构建目录中构建目标。
      ```sh
      $ cmake --build build --target hello_world
      ```
      > 提供给 `--build` 标志的目录名必须与之前 cmake 命令中传给 `-B` 标志的目录名一致。

1. 现在你有了可通过调试器加载的 `hello_world.elf`，或可通过拖放安装并在 Raspberry Pi Pico 系列设备上运行的 `hello_world.uf2`。

# RP2350 上的 RISC-V 支持

有关为 RP2350 上的 RISC-V 设置构建环境，请参阅 [Raspberry Pi Pico 系列 C/C++ SDK](https://rptl.io/pico-c-sdk)。

## RISC-V 快速入门

[pico-sdk-tools](https://github.com/raspberrypi/pico-sdk-tools/releases) 仓库包含一些预构建的 RISC-V 编译器版本。

例如，你可以使用它们在 Raspberry Pi OS 上获得可用的 RISC-V 编译器。

```
wget https://github.com/raspberrypi/pico-sdk-tools/releases/download/v2.0.0-5/riscv-toolchain-14-aarch64-lin.tar.gz
sudo mkdir -p /opt/riscv/riscv-toolchain-14
sudo chown $USER /opt/riscv/riscv-toolchain-14
tar xvf riscv-toolchain-14-aarch64-lin.tar.gz -C /opt/riscv/riscv-toolchain-14
```
要使用 RISC-V 编译器构建代码，需要设置几个环境变量并从头运行 cmake。

```
export PICO_TOOLCHAIN_PATH=/opt/riscv/riscv-toolchain-14/
export PICO_PLATFORM=rp2350-riscv
```
