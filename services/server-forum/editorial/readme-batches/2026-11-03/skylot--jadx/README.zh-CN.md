> 根据 [jadx 固定版本 README](https://github.com/skylot/jadx/blob/c5a451e4ecaea6913bdacb36d272899e0476b5ed/README.md) 翻译。原始命令帮助输出保留，后附逐项中文说明；安全开关可能关闭 XML 或压缩包保护，请仔细阅读。

<img src="https://raw.githubusercontent.com/skylot/jadx/c5a451e4ecaea6913bdacb36d272899e0476b5ed/jadx-gui/src/main/resources/logos/jadx-logo.png" width="64" align="left" alt="jadx logo" />

## JADX

![Build status](https://img.shields.io/github/actions/workflow/status/skylot/jadx/build-artifacts.yml)
![GitHub contributors](https://img.shields.io/github/contributors/skylot/jadx)
![GitHub all releases](https://img.shields.io/github/downloads/skylot/jadx/total)
![GitHub downloads (latest release)](https://img.shields.io/github/downloads/skylot/jadx/latest/total)
![Latest release](https://img.shields.io/github/release/skylot/jadx.svg)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.skylot/jadx-core)](https://central.sonatype.com/namespace/io.github.skylot)
![Java 11+](https://img.shields.io/badge/Java-11%2B-blue)
[![License](https://img.shields.io/github/license/skylot/jadx)](https://www.apache.org/licenses/LICENSE-2.0.html)

**jadx**——Dex 到 Java 的反编译器

从 Android Dex 和 APK 文件生成 Java 源代码的命令行和图形界面工具。

> [!WARNING]
> 请注意，大多数情况下 **jadx** 无法反编译 100% 的代码，因此会出现错误。<br />
> 解决方法请参阅[故障排查指南](https://github.com/skylot/jadx/wiki/Troubleshooting-Q&A#decompilation-issues)。

**主要功能：**
- 将 APK、dex、aab、apks、apkm、xapk 和 zip 文件中的 Dalvik 字节码反编译为 Java 代码
- 也可反编译 Java 字节码（jar、class、aar）和 smali 文件
- 从 `resources.arsc` 解码 `AndroidManifest.xml` 和其他资源
- 包含反混淆工具

**jadx-gui 功能：**
- 查看带语法高亮的反编译代码
- 跳转到声明
- 查找使用位置
- 全文搜索
- smali 调试器；设置和用法参见 [Wiki 页面](https://github.com/skylot/jadx/wiki/Smali-debugger)

jadx-gui 快捷键参见[这里](https://github.com/skylot/jadx/wiki/JADX-GUI-Key-bindings)。

查看这些功能的实际演示：[jadx-gui 功能概览](https://github.com/skylot/jadx/wiki/jadx-gui-features-overview)。

<img src="https://user-images.githubusercontent.com/118523/142730720-839f017e-38db-423e-b53f-39f5f0a0316f.png" width="700" alt="jadx-gui screenshot"/>

### 下载
- 从 [GitHub: ![最新发行版](https://img.shields.io/github/release/skylot/jadx.svg)](https://github.com/skylot/jadx/releases/latest) 下载发行版
- 最新[不稳定构建 ![自标记版本以来的 GitHub 提交](https://img.shields.io/github/commits-since/skylot/jadx/latest/master)](https://nightly.link/skylot/jadx/workflows/build-artifacts/master)

对于 `jadx-<version>.zip`，解压后进入 `bin` 目录并运行：
- `jadx`——命令行版本
- `jadx-gui`——图形界面版本

在 Windows 上，双击运行 `.bat` 文件。对于 Windows 捆绑包，解压并运行 `jadx-gui-<version>.exe`。

**注意：**请确保已安装 64 位 Java 11 或更高版本（`with-jre` 捆绑包不需要）。Windows 用户可从 [oracle.com](https://www.oracle.com/java/technologies/downloads/#jdk25-windows) 下载（选择 x64 安装程序）。

### 安装
- Arch Linux
  [![Arch Linux package](https://img.shields.io/archlinux/v/extra/any/jadx)](https://archlinux.org/packages/extra/any/jadx/)
  [![AUR Version](https://img.shields.io/aur/version/jadx-git)](https://aur.archlinux.org/packages/jadx-git)
  ```bash
  sudo pacman -S jadx
  ```
- macOS
  [![Homebrew Version](https://img.shields.io/homebrew/v/jadx)](https://formulae.brew.sh/formula/jadx)
  ```bash
  brew install jadx
  ```
- Flathub
  [![Flathub Version](https://img.shields.io/flathub/v/com.github.skylot.jadx)](https://flathub.org/apps/com.github.skylot.jadx)
  ```bash
  flatpak install flathub com.github.skylot.jadx
  ```

### 用法
```
jadx[-gui] [command] [options] <input files> (.apk, .dex, .jar, .class, .smali, .zip, .aar, .arsc, .aab, .xapk, .apkm, .apks, .jadx.kts)
commands (use '<command> --help' for command options):
  plugins	  - manage jadx plugins

options:
  -d, --output-dir                              - output directory
  -ds, --output-dir-src                         - output directory for sources
  -dr, --output-dir-res                         - output directory for resources
  -r, --no-res                                  - do not decode resources
  -s, --no-src                                  - do not decompile source code
  -j, --threads-count                           - processing threads count, default: half of CPU threads
  --single-class                                - decompile a single class, full name, raw or alias
  --single-class-output                         - file or dir for write if decompile a single class
  --output-format                               - can be 'java' or 'json', default: java
  -e, --export-gradle                           - save as gradle project (set '--export-gradle-type' to 'auto')
  --export-gradle-type                          - Gradle project template for export:
                                                   'auto' - detect automatically
                                                   'android-app' - Android Application (apk)
                                                   'android-library' - Android Library (aar)
                                                   'simple-java' - simple Java
  -m, --decompilation-mode                      - code output mode:
                                                   'auto' - trying best options (default)
                                                   'restructure' - restore code structure (normal java code)
                                                   'simple' - simplified instructions (linear, with goto's)
                                                   'fallback' - raw instructions without modifications
  --show-bad-code                               - show inconsistent code (incorrectly decompiled)
  --no-xml-pretty-print                         - do not prettify XML
  --no-imports                                  - disable use of imports, always write entire package name
  --no-debug-info                               - disable debug info parsing and processing
  --add-debug-lines                             - add comments with debug line numbers if available
  --no-inline-anonymous                         - disable anonymous classes inline
  --no-inline-methods                           - disable methods inline
  --no-move-inner-classes                       - disable move inner classes into parent
  --no-inline-kotlin-lambda                     - disable inline for Kotlin lambdas
  --no-finally                                  - don't extract finally block
  --no-restore-switch-over-string               - don't restore switch over string
  --no-replace-consts                           - don't replace constant value with matching constant field
  --escape-unicode                              - escape non latin characters in strings (with \u)
  --respect-bytecode-access-modifiers           - don't change original access modifiers
  --mappings-path                               - deobfuscation mappings file or directory. Allowed formats: Tiny and Tiny v2 (both '.tiny'), Enigma (.mapping) or Enigma directory
  --mappings-mode                               - set mode for handling the deobfuscation mapping file:
                                                   'read' - just read, user can always save manually (default)
                                                   'read-and-autosave-every-change' - read and autosave after every change
                                                   'read-and-autosave-before-closing' - read and autosave before exiting the app or closing the project
                                                   'ignore' - don't read or save (can be used to skip loading mapping files referenced in the project file)
  --deobf                                       - activate deobfuscation
  --deobf-min                                   - min length of name, renamed if shorter, default: 3
  --deobf-max                                   - max length of name, renamed if longer, default: 64
  --deobf-whitelist                             - space separated list of classes (full name) and packages (ends with '.*') to exclude from deobfuscation, default: android.support.v4.* android.support.v7.* android.support.v4.os.* android.support.annotation.Px androidx.core.os.* androidx.annotation.Px
  --deobf-cfg-file                              - deobfuscation mappings file used for JADX auto-generated names (in the JOBF file format), default: same dir and name as input file with '.jobf' extension
  --deobf-cfg-file-mode                         - set mode for handling the JADX auto-generated names' deobfuscation map file:
                                                   'read' - read if found, don't save (default)
                                                   'read-or-save' - read if found, save otherwise (don't overwrite)
                                                   'overwrite' - don't read, always save
                                                   'ignore' - don't read and don't save
  --deobf-res-name-source                       - better name source for resources:
                                                   'auto' - automatically select best name (default)
                                                   'resources' - use resources names
                                                   'code' - use R class fields names
  --use-source-name-as-class-name-alias         - use source name as class name alias:
                                                   'always' - always use source name if it's available
                                                   'if-better' - use source name if it seems better than the current one
                                                   'never' - never use source name, even if it's available
  --source-name-repeat-limit                    - allow using source name if it appears less than a limit number, default: 10
  --use-kotlin-methods-for-var-names            - use kotlin intrinsic methods to rename variables, values: disable, apply, apply-and-hide, default: apply
  --use-headers-for-detect-resource-extensions  - Use headers for detect resource extensions if resource obfuscated
  --rename-flags                                - fix options (comma-separated list of):
                                                   'case' - fix case sensitivity issues (according to --fs-case-sensitive option),
                                                   'valid' - rename java identifiers to make them valid,
                                                   'printable' - remove non-printable chars from identifiers,
                                                  or single 'none' - to disable all renames
                                                  or single 'all' - to enable all (default)
  --integer-format                              - how integers are displayed:
                                                   'auto' - automatically select (default)
                                                   'decimal' - use decimal
                                                   'hexadecimal' - use hexadecimal
  --type-update-limit                           - type update limit count (per one instruction), default: 10
  --fs-case-sensitive                           - treat filesystem as case sensitive, false by default
  --cfg                                         - save methods control flow graph to dot file
  --raw-cfg                                     - save methods control flow graph (use raw instructions)
  --call-graph                                  - save app call graph in format: 'dot' or 'json', default: none
  -f, --fallback                                - set '--decompilation-mode' to 'fallback' (deprecated)
  --use-dx                                      - use dx/d8 to convert java bytecode
  --comments-level                              - set code comments level, values: error, warn, info, debug, user-only, none, default: info
  --log-level                                   - set log level, values: quiet, progress, error, warn, info, debug, default: progress
  -v, --verbose                                 - verbose output (set --log-level to DEBUG)
  -q, --quiet                                   - turn off output (set --log-level to QUIET)
  --disable-plugins                             - comma separated list of plugin ids to disable
  --config <config-ref>                         - load configuration from file, <config-ref> can be:
                                                   path to '.json' file
                                                   short name - uses file with this name from config directory
                                                   'none' - to disable config loading
  --save-config <config-ref>                    - save current options into configuration file and exit, <config-ref> can be:
                                                   empty - for default config
                                                   path to '.json' file
                                                   short name - file will be saved in config directory
  --print-files                                 - print files and directories used by jadx (config, cache, temp)
  --version                                     - print jadx version
  -h, --help                                    - print this help

Plugin options (-P<name>=<value>):
  dex-input: Load .dex and .apk files
    - dex-input.verify-checksum                 - verify dex file checksum before load, values: [yes, no], default: yes
  java-convert: Convert .class, .jar and .aar files to dex
    - java-convert.mode                         - convert mode, values: [dx, d8, both], default: both
    - java-convert.d8-desugar                   - use desugar in d8, values: [yes, no], default: no
  kotlin-metadata: Use kotlin.Metadata annotation for code generation
    - kotlin-metadata.class-alias               - rename class alias, values: [yes, no], default: yes
    - kotlin-metadata.method-args               - rename function arguments, values: [yes, no], default: yes
    - kotlin-metadata.fields                    - rename fields, values: [yes, no], default: yes
    - kotlin-metadata.companion                 - rename companion object, values: [yes, no], default: yes
    - kotlin-metadata.data-class                - add data class modifier, values: [yes, no], default: yes
    - kotlin-metadata.to-string                 - rename fields using toString, values: [yes, no], default: yes
    - kotlin-metadata.getters                   - rename simple getters to field names, values: [yes, no], default: yes
  kotlin-smap: Use kotlin.SourceDebugExtension annotation for rename class alias
    - kotlin-smap.class-alias-source-dbg        - rename class alias from SourceDebugExtension, values: [yes, no], default: no
  rename-mappings: various mappings support
    - rename-mappings.format                    - mapping format, values: [AUTO, TINY_FILE, TINY_2_FILE, ENIGMA_FILE, ENIGMA_DIR, PROGUARD_FILE, SRG_FILE, XSRG_FILE, JAM_FILE, CSRG_FILE, TSRG_FILE, TSRG_2_FILE, INTELLIJ_MIGRATION_MAP_FILE, RECAF_SIMPLE_FILE, JOBF_FILE], default: AUTO
    - rename-mappings.invert                    - invert mapping on load, values: [yes, no], default: no
  smali-input: Load .smali files
    - smali-input.api-level                     - Android API level, default: 27

Environment variables:
  JADX_DISABLE_XML_SECURITY - set to 'true' to disable all security checks for XML files
  JADX_DISABLE_ZIP_SECURITY - set to 'true' to disable all security checks for zip files
  JADX_DISABLE_ALL_SECURITY_FLAGS - set to 'true' to disable all security flags (xml, string, app package)
  JADX_ZIP_MAX_ENTRIES_COUNT - maximum allowed number of entries in zip files (default: 100 000)
  JADX_ZIP_BOMB_MIN_UNCOMPRESSED_SIZE - min size of zip entry to check compression ratio, in bytes (default: 25 MB)
  JADX_ZIP_BOMB_DETECTION_FACTOR - maximum allowed compression ratio of zip entry (default: 100)
  JADX_CONFIG_DIR - custom config directory, using system by default
  JADX_CACHE_DIR - custom cache directory, using system by default
  JADX_TMP_DIR - custom temp directory, using system by default

Examples:
  jadx -d out classes.dex
  jadx --rename-flags "none" classes.dex
  jadx --rename-flags "valid, printable" classes.dex
  jadx --log-level ERROR app.apk
  jadx -Pdex-input.verify-checksum=no app.apk
```
上面的命令帮助说明如下（实际参数和示例以原文为准）：

- 用法：`jadx[-gui] [command] [options] <input files>`；支持的输入扩展名为 `.apk`、`.dex`、`.jar`、`.class`、`.smali`、`.zip`、`.aar`、`.arsc`、`.aab`、`.xapk`、`.apkm`、`.apks` 和 `.jadx.kts`。命令选项可用 `<command> --help` 查看；`plugins` 管理 jadx 插件。
- `-d, --output-dir`：输出目录；`-ds, --output-dir-src`：源代码输出目录；`-dr, --output-dir-res`：资源输出目录。
- `-r, --no-res`：不解码资源；`-s, --no-src`：不反编译源代码；`-j, --threads-count`：处理线程数，默认 CPU 线程数的一半。
- `--single-class`：反编译单个类，可使用完整名称、原始名称或别名；`--single-class-output`：反编译单个类时写入的文件或目录。
- `--output-format`：`java` 或 `json`，默认为 `java`；`-e, --export-gradle`：保存为 Gradle 项目（将 `--export-gradle-type` 设为 `auto`）。
- `--export-gradle-type`：导出 Gradle 项目模板，`auto` 自动检测，`android-app` 为 Android 应用（apk），`android-library` 为 Android 库（aar），`simple-java` 为简单 Java 项目。
- `-m, --decompilation-mode`：代码输出模式；`auto` 尝试最佳选项（默认），`restructure` 恢复代码结构（普通 Java 代码），`simple` 使用简化指令（线性，含 goto），`fallback` 使用未经修改的原始指令。
- `--show-bad-code`：显示不一致的代码（反编译错误）；`--no-xml-pretty-print`：不美化 XML；`--no-imports`：禁用 imports，始终写完整包名。
- `--no-debug-info`：禁用调试信息解析和处理；`--add-debug-lines`：如有调试行号则作为注释添加。
- `--no-inline-anonymous`：禁用匿名类内联；`--no-inline-methods`：禁用方法内联；`--no-move-inner-classes`：不将内部类移入父类；`--no-inline-kotlin-lambda`：不内联 Kotlin lambda。
- `--no-finally`：不提取 finally 块；`--no-restore-switch-over-string`：不恢复字符串 switch；`--no-replace-consts`：不将常量值替换为匹配的常量字段。
- `--escape-unicode`：使用 `\u` 转义字符串中的非拉丁字符；`--respect-bytecode-access-modifiers`：不改变原始访问修饰符。
- `--mappings-path`：反混淆映射文件或目录，允许 Tiny 和 Tiny v2（均为 `.tiny`）、Enigma（`.mapping`）或 Enigma 目录。
- `--mappings-mode`：映射文件处理模式：`read` 仅读取、用户可随时手动保存（默认）；`read-and-autosave-every-change` 每次更改后自动保存；`read-and-autosave-before-closing` 在退出应用或关闭项目之前自动保存；`ignore` 不读取也不保存，可跳过项目文件引用的映射文件。
- `--deobf`：启用反混淆；`--deobf-min`：名称最小长度，短于此值则重命名，默认 3；`--deobf-max`：名称最大长度，长于此值则重命名，默认 64。
- `--deobf-whitelist`：以空格分隔要排除反混淆的类完整名称和包（以 `.*` 结尾）。默认值为 `android.support.v4.* android.support.v7.* android.support.v4.os.* android.support.annotation.Px androidx.core.os.* androidx.annotation.Px`。
- `--deobf-cfg-file`：用于 JADX 自动生成名称的反混淆映射文件（JOBF 格式）；默认与输入文件位于同一目录、同名，扩展名为 `.jobf`。
- `--deobf-cfg-file-mode`：自动生成名称映射的处理模式：`read` 找到则读取、不保存（默认）；`read-or-save` 找到则读取，否则保存（不覆盖）；`overwrite` 不读取、始终保存；`ignore` 不读取也不保存。
- `--deobf-res-name-source`：资源的较佳名称来源：`auto` 自动选择最佳名称（默认）；`resources` 使用资源名；`code` 使用 R 类字段名。
- `--use-source-name-as-class-name-alias`：将源名称用作类别名：`always` 可用时始终使用，`if-better` 看起来优于当前名称时使用，`never` 即使可用也不使用。
- `--source-name-repeat-limit`：源名称出现次数低于限制时允许使用，默认 10；`--use-kotlin-methods-for-var-names`：使用 Kotlin 内建方法重命名变量，可选 `disable`、`apply`、`apply-and-hide`，默认 `apply`。
- `--use-headers-for-detect-resource-extensions`：资源被混淆时通过文件头检测资源扩展名。
- `--rename-flags`：逗号分隔的修复选项；`case` 根据 `--fs-case-sensitive` 修复大小写问题，`valid` 将 Java 标识符重命名为有效名称，`printable` 移除不可打印字符；也可单独用 `none` 禁用所有重命名，或用 `all` 启用所有重命名（默认）。
- `--integer-format`：整数显示方式；`auto` 自动选择（默认），`decimal` 十进制，`hexadecimal` 十六进制。
- `--type-update-limit`：每条指令的类型更新次数限制，默认 10；`--fs-case-sensitive`：将文件系统视为区分大小写，默认 false。
- `--cfg`：将方法控制流图保存为 dot 文件；`--raw-cfg`：使用原始指令保存方法控制流图；`--call-graph`：保存应用调用图，格式为 `dot` 或 `json`，默认 none。
- `-f, --fallback`：将 `--decompilation-mode` 设为 `fallback`（已弃用）；`--use-dx`：使用 dx/d8 转换 Java 字节码。
- `--comments-level`：代码注释级别，可选 `error`、`warn`、`info`、`debug`、`user-only`、`none`，默认 `info`；`--log-level`：日志级别，可选 `quiet`、`progress`、`error`、`warn`、`info`、`debug`，默认 `progress`。
- `-v, --verbose`：详细输出（将 `--log-level` 设为 DEBUG）；`-q, --quiet`：关闭输出（将 `--log-level` 设为 QUIET）；`--disable-plugins`：逗号分隔的禁用插件 ID 列表。
- `--config <config-ref>`：从文件加载配置。`<config-ref>` 可以是 `.json` 文件路径、配置目录内的短名称，或 `none`（禁用配置加载）。
- `--save-config <config-ref>`：将当前选项保存到配置文件并退出；可以为空（默认配置）、`.json` 文件路径或短名称（保存到配置目录）。
- `--print-files`：输出 jadx 使用的文件和目录（配置、缓存、临时文件）；`--version`：输出版本；`-h, --help`：输出帮助。

插件选项使用 `-P<name>=<value>`：

- `dex-input` 加载 `.dex` 和 `.apk`；`dex-input.verify-checksum` 控制加载前验证 dex 校验和，可选 `[yes, no]`，默认 yes。
- `java-convert` 将 `.class`、`.jar`、`.aar` 转为 dex；`java-convert.mode` 可选 `[dx, d8, both]`，默认 both；`java-convert.d8-desugar` 控制 d8 中的 desugar，可选 `[yes, no]`，默认 no。
- `kotlin-metadata` 使用 kotlin.Metadata 注解生成代码。`kotlin-metadata.class-alias` 重命名类别名；`kotlin-metadata.method-args` 重命名函数参数；`kotlin-metadata.fields` 重命名字段；`kotlin-metadata.companion` 重命名伴生对象；`kotlin-metadata.data-class` 添加 data class 修饰符；`kotlin-metadata.to-string` 利用 toString 重命名字段；`kotlin-metadata.getters` 将简单 getter 重命名为字段名。各项均为 `[yes, no]`，默认 yes。
- `kotlin-smap` 使用 kotlin.SourceDebugExtension 注解重命名类别名；`kotlin-smap.class-alias-source-dbg` 从 SourceDebugExtension 重命名类别名，取值 `[yes, no]`，默认 no。
- `rename-mappings` 提供多种映射支持。`rename-mappings.format` 的取值为 `[AUTO, TINY_FILE, TINY_2_FILE, ENIGMA_FILE, ENIGMA_DIR, PROGUARD_FILE, SRG_FILE, XSRG_FILE, JAM_FILE, CSRG_FILE, TSRG_FILE, TSRG_2_FILE, INTELLIJ_MIGRATION_MAP_FILE, RECAF_SIMPLE_FILE, JOBF_FILE]`，默认 AUTO；`rename-mappings.invert` 控制加载时反转映射，取值 `[yes, no]`，默认 no。
- `smali-input` 加载 `.smali` 文件；`smali-input.api-level` 为 Android API 级别，默认 27。

环境变量：

- `JADX_DISABLE_XML_SECURITY`：设为 `true` 时禁用所有 XML 文件安全检查。
- `JADX_DISABLE_ZIP_SECURITY`：设为 `true` 时禁用所有 zip 文件安全检查。
- `JADX_DISABLE_ALL_SECURITY_FLAGS`：设为 `true` 时禁用所有安全标志（xml、string、app package）。
- `JADX_ZIP_MAX_ENTRIES_COUNT`：zip 文件允许的最大条目数，默认 100 000。
- `JADX_ZIP_BOMB_MIN_UNCOMPRESSED_SIZE`：检查压缩率的 zip 条目最小大小，单位字节，默认 25 MB。
- `JADX_ZIP_BOMB_DETECTION_FACTOR`：zip 条目允许的最大压缩率，默认 100。
- `JADX_CONFIG_DIR`、`JADX_CACHE_DIR`、`JADX_TMP_DIR`：分别指定自定义配置、缓存和临时目录，默认使用系统目录。

示例命令保留在上面的原始帮助输出中。

从命令行运行 jadx-gui 时，这些选项同样有效，并会覆盖偏好设置对话框中的选项。

`plugins` 命令用法
```
usage: plugins [options]
options:
  -i, --install <locationId>      - install plugin with locationId
  -j, --install-jar <path-to.jar> - install plugin from jar file
  -l, --list                      - list installed plugins
  -a, --available                 - list available plugins from jadx-plugins-list (aka marketplace)
  -u, --update                    - update installed plugins
  --uninstall <pluginId>          - uninstall plugin with pluginId
  --disable <pluginId>            - disable plugin with pluginId
  --enable <pluginId>             - enable plugin with pluginId
  --list-all                      - list all plugins including bundled and dropins
  --list-versions <locationId>    - fetch latest versions of plugin from locationId (will download all artefacts, limited to 10)
  -h, --help                      - print this help
```

`plugins` 命令帮助：

- `-i, --install <locationId>`：使用 locationId 安装插件；`-j, --install-jar <path-to.jar>`：从 jar 文件安装插件。
- `-l, --list`：列出已安装插件；`-a, --available`：列出 jadx-plugins-list（即市场）中的可用插件；`-u, --update`：更新已安装插件。
- `--uninstall <pluginId>`：卸载指定插件；`--disable <pluginId>`：禁用指定插件；`--enable <pluginId>`：启用指定插件。
- `--list-all`：列出所有插件，包括内置插件和 dropins；`--list-versions <locationId>`：获取指定插件的最新版本（将下载所有构件，最多 10 个）；`-h, --help`：输出帮助。

### 将 jadx 作为库使用
可以在 Java 项目中使用 jadx，详情参见 [Wiki 页面](https://github.com/skylot/jadx/wiki/Use-jadx-as-a-library)。

### 从源代码构建
必须安装 JDK 17 或更高版本：
```
git clone https://github.com/skylot/jadx.git
cd jadx
./gradlew dist
```

（Windows 上使用 `gradlew.bat` 代替 `./gradlew`）

运行 jadx 的脚本将放在 `build/jadx/bin`，同时打包到 `build/jadx-<version>.zip`。

### 故障排查
请查看 Wiki 页面[故障排查问答](https://github.com/skylot/jadx/wiki/Troubleshooting-Q&A)。

### 参与贡献
要支持本项目，你可以：
- 分享对你重要的新功能或优化的想法
- 提交反编译问题（请先阅读[如何提交 issue](https://github.com/skylot/jadx/blob/c5a451e4ecaea6913bdacb36d272899e0476b5ed/CONTRIBUTING.md#Open-Issue)）
- 提交 pull request（请遵循 [pull request 规则](https://github.com/skylot/jadx/blob/c5a451e4ecaea6913bdacb36d272899e0476b5ed/CONTRIBUTING.md#Pull-Request-Process)）

---------------------------------------
*采用 Apache 2.0 许可证*
