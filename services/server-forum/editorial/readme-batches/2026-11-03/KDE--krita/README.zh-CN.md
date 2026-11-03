![图片](https://krita.org/images/krita-logo-light.svg)

| CI Name     | Master | Stable | Release |
| ------------------- | ---------------- | ------ | ------- |
| Pipeline | [![pipeline status](https://invent.kde.org/graphics/krita/badges/master/pipeline.svg)](https://invent.kde.org/graphics/krita/-/commits/master) | [![pipeline status](https://invent.kde.org/graphics/krita/badges/krita/5.2/pipeline.svg)](https://invent.kde.org/graphics/krita/-/commits/krita/5.2) | [![Latest Release](https://invent.kde.org/graphics/krita/-/badges/release.svg)](https://invent.kde.org/graphics/krita/-/releases) |

注意：此表目前不涵盖每夜构建版。

Krita 是免费开源的数字绘画应用，面向希望从头到尾创作专业作品的艺术家。漫画家、插画师、概念设计师、背景和纹理画师以及数字视觉特效行业都在使用 Krita。

如果你在 GitHub 上阅读此内容，请注意这里仅是镜像。真正的代码仓库由 KDE 提供：https://invent.kde.org/graphics/krita.git

![图片](https://krita.org/images/hero-image-50.webp)

### 仓库状态

分支：`master`

| 冻结类型 | 状态 |
|----------------|----------------------------------------------------------------------|
| 功能冻结 | 允许新增功能 |
| 字符串冻结 | 允许新增字符串 |

### 用户手册
https://docs.krita.org/en/user_manual.html

### 开发说明和构建指南

请遵循[在线文档](https://docs.krita.org/en/untranslatable_pages/building_krita.html)。

其他开发者指南、说明和 Wiki：

https://docs.krita.org/en/untranslatable_pages.html

API 文档：

https://api.kde.org/legacy/krita/html/index.html

### 错误与功能愿望

https://bugs.kde.org/buglist.cgi?bug_status=UNCONFIRMED&bug_status=CONFIRMED&bug_status=ASSIGNED&bug_status=REOPENED&list_id=1315444&product=krita&query_format=advanced

### 讨论论坛

* https://krita-artists.org/

### IRC 频道

大多数开发者都在这里。如果你有兴趣帮助本项目，这是一个很好的起点。

libera.chat，#krita

### 项目网站

https://www.krita.org

### 每夜构建版

#### 不稳定版

* https://cdn.kde.org/ci-builds/graphics/krita/master/

#### 稳定版

* https://cdn.kde.org/ci-builds/graphics/krita/krita-5.2/

#### 开发者构建版

##### 包含 Qt 和 Krita 调试符号的 Linux 构建版

1) 前往 Krita CI 的 Jobs 部分：https://invent.kde.org/graphics/krita/-/jobs
2) 搜索最新的 `linux-debug-weekly` 任务
3) 进入任务并点击 Artifacts->Browse
4) 下载 AppImage

##### Qt 和 Krita 启用 ASAN 的 Linux 构建版

1) 前往 Krita CI 的 Jobs 部分：https://invent.kde.org/graphics/krita/-/jobs
2) 搜索最新的 `linux-asan-weekly` 任务
3) 进入任务并点击 Artifacts->Browse
4) 下载 AppImage
5) 设置 ASAN 环境变量：
    ```bash
        export ASAN_OPTIONS=new_delete_type_mismatch=0:detect_leaks=0
    ```
6) 在修改后的环境中运行 AppImage

##### Qt 和 Krita 启用 ASAN 的 Windows 构建版

1) 前往 Krita CI 的 Jobs 部分：https://invent.kde.org/graphics/krita/-/jobs
2) 搜索最新的 `windows-asan-weekly` 任务
3) 进入任务并点击 Artifacts->Browse
4) 下载 .zip 文件
5) 打开终端
6) 设置 ASAN 环境变量：
    ```
        set ASAN_OPTIONS=new_delete_type_mismatch=0:detect_leaks=0
    ```
7) 将工作目录切换到 `c:\path\where\you\downloaded\krita-5.3.0-prealpha-git12345\bin`。这很重要，否则 ASAN 无法找到 llvm-symbolizer.exe，生成的回溯将不包含正确的符号。
    ```
        cd c:\path\where\you\downloaded\krita-5.3.0-prealpha-git12345\bin
    ```
8) 运行 krita
    ```
        krita.com
    ```

### 许可证

Krita 作为整体采用 GNU 公共许可证第 3 版。单个文件可能采用不同但兼容的许可证。

### 暂停使用 AI

由于 Krita 开发者社区目前无法就使用 AI 工具辅助开发是否可接受达成共识，我们决定暂停使用这些工具，期限至 2026 年 10 月。

将决定推迟至 10 月的原因如下：

1) 如果我们现在允许在开发中使用 AI，用户和支持者的反弹将十分强烈。
2) KDE 整体尚无相关政策；如果我们的政策与 KDE 的政策冲突，可能不得不撤回使用 AI 完成的贡献。
3) 未来一段时间 AI 本身的发展可能变好或变坏。微软已开始淡化其 Copilot 品牌。人们对 AI 的接受程度可能变好或变坏；AI 对环境的影响可能减轻。

在作出决定之前，禁止在 Krita 工作中使用 AI。
