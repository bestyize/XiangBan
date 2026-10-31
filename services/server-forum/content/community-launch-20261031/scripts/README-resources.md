# 55 个原创资源包的构建与验证

这些脚本保存本批实际使用的生成、渲染、验证和打包过程。路径改成了相对仓库位置与参数；保存脚本后只运行语法检查，没有再次改写已经上传的 ZIP、PNG 或发布清单。

`pack-release-lock.json` 保留本次 55 个 ZIP 和 55 张封面的 SHA-256、字节数与对象键。已交付上传目录 `/tmp/community-pack-build/data` 和原 `catalog.json` 是这一版本的发布输入，不能用重新渲染的图片覆盖其中字节。

## 依赖与参数

- Python 3.10+；封面阶段需要 Pillow。可用已配置的图像 Python，也可显式指定下面的 `XB_PYTHON_BIN`。
- Node.js；`resource_preview.cjs` 需要 TypeScript，`resource_browser.cjs` 需要 Playwright 和本地 Chromium。
- 字体：`--font` 或 `XB_CJK_FONT` 指向本机支持中文的 TTF/TTC/OTF。未指定时依次找 macOS 的 STHeiti、Linux 的 Noto Sans CJK、Windows 的微软雅黑；不会下载字体或分发系统字体。

Python 脚本支持：

```text
--source-dir PATH   默认本批 packs/，可用 XB_PACK_SOURCE_DIR 设置
--output-dir PATH   默认 /tmp/community-pack-rebuild，可用 XB_PACK_BUILD_DIR 设置
--font PATH         可选，等同 XB_CJK_FONT
```

Node 脚本支持同名 source/output 参数，并支持：

```text
--repo-root PATH          默认由 scripts/ 相对定位仓库；XB_REPO_ROOT
--base-url URL            默认 http://127.0.0.1:55138；XB_PACK_PREVIEW_ORIGIN
--typescript-module PATH  TypeScript 包目录；XB_TYPESCRIPT_MODULE
--playwright-module PATH  Playwright 包目录；XB_PLAYWRIGHT_MODULE
--chromium PATH           Chromium 可执行文件；XB_CHROMIUM_EXECUTABLE
```

模块按显式参数、正常 Node 解析、当前仓库前端依赖、已配置 Codex runtime 寻找。`CODEX_RUNTIME_NODE_MODULES` 可指定其他 runtime 的 node_modules。浏览器优先使用显式路径，然后检查已配置缓存中的 1248 headless shell，最后交给 Playwright 自身解析。没有把某一台机器的仓库绝对路径写死。已有依赖时无需联网或重新安装。

## 在独立目录完整重建

从仓库根执行。下面特意用新的源码与输出目录，避免覆盖 `packs/` 和已上传产物。

```sh
XB_PACK_SCRIPTS="$PWD/services/server-forum/content/community-launch-20261031/scripts"
XB_PACK_REBUILD="/tmp/community-pack-rebuild"
XB_PACK_REBUILD_SOURCE="$XB_PACK_REBUILD/source-packs"
XB_PYTHON_BIN="${XB_PYTHON_BIN:-python3}"

"$XB_PYTHON_BIN" "$XB_PACK_SCRIPTS/resource_build.py" \
  --source-dir "$XB_PACK_REBUILD_SOURCE" --output-dir "$XB_PACK_REBUILD"
"$XB_PYTHON_BIN" "$XB_PACK_SCRIPTS/resource_games.py" \
  --source-dir "$XB_PACK_REBUILD_SOURCE" --output-dir "$XB_PACK_REBUILD"
node "$XB_PACK_SCRIPTS/resource_preview.cjs" \
  --source-dir "$XB_PACK_REBUILD_SOURCE" --output-dir "$XB_PACK_REBUILD"
```

在另一个终端开启本地预览服务器，保持运行：

```sh
python3 -m http.server 55138 --bind 127.0.0.1 \
  --directory /tmp/community-pack-rebuild/preview
```

然后继续：

```sh
node "$XB_PACK_SCRIPTS/resource_browser.cjs" \
  --source-dir "$XB_PACK_REBUILD_SOURCE" --output-dir "$XB_PACK_REBUILD" \
  --base-url http://127.0.0.1:55138
"$XB_PYTHON_BIN" "$XB_PACK_SCRIPTS/resource_finalize.py" \
  --source-dir "$XB_PACK_REBUILD_SOURCE" --output-dir "$XB_PACK_REBUILD"
```

如果本机有多个 Python，可将 `XB_PYTHON_BIN` 设置为已安装 Pillow 的环境。模块不在默认位置时，为对应 Node 命令追加上面的 module/executable 参数。预览端口被占用时，同时修改 http.server 的端口和 `--base-url`。

## 脚本职责与产物

| 脚本 | 内容 |
| --- | --- |
| `resource_build.py` | 35 个具体手动模板、5 特效、5 壁纸、5 组件；生成 draft-catalog.json |
| `resource_games.py` | 5 个不同玩法的真实离线游戏与各自 cover.svg；追加至同一清单 |
| `resource_preview.cjs` | 将仓库真实特效 TS parser/sim/draw 转为临时浏览器模块，创建只读源码链接 |
| `resource_browser.cjs` | 20 个视觉作品渲染；组件 CSP/sandbox 交互；游戏获胜、重开、触控、键盘、暂停；截图与 browser-validation.json |
| `resource_finalize.py` | 补能力包 prompt/canDo/cannotDo/permissions/runLocation；校验并生成 ZIP、PNG、catalog.json、structure-validation.json |
| `pack_build_common.py` / `pack_runtime.cjs` | 参数、路径、运行时与共享创作工具 |
| `pack-release-lock.json` | 本次发布对象的不可变大小与校验值，不含本机绝对路径 |

输出目录下：

```text
catalog.json
browser-validation.json
structure-validation.json
data/
  packs/<kind>/<slug>-1.0.0.zip
  covers/<slug>-1.0.0.png
screenshots/
preview/
```

`data/` 仅包含真实待传对象；不要把预览模块、截图、测试报告一并上传为公开对象。模板封面是标明手动模板的代码原生图形，其他封面基于真实页面或真实粒子引擎截图，不伪造客户端 UI。

## 重建与发布哈希的关系

ZIP 写入固定文件时间与文件顺序，同样源文件和压缩实现可以重复生成。浏览器截图受随机局面、动画帧、浏览器、字体和平台影响，封面 PNG 不承诺跨机器逐字节相同；这也是保留本次 release lock 的原因。脚本中的版本仍是 1.0.0，重建是复核与后续创作的起点，不能把不同字节作为本次同键对象重新发布。若确需变更作品，应先调整版本和对象键，再人工审阅。

本次已验证 55 包结构/大小/JSON/UTF-8/SHA；5 特效用真实 TS 引擎；5 壁纸、5 组件、5 游戏用实际 Chromium。集成任务另用原桌面 Rust validator 检查了 5 effect 和 5 widget；这里没有冒充完整 Tauri ZIP 导入验证。保存并参数化后的脚本仅做 `py_compile` 与 `node --check`，没有重新执行整套构建。
