> 根据 [Bun 固定版本 README](https://github.com/oven-sh/bun/blob/6cdb3835d90f8e05e4ebff86c7135078c9eb7e88/README.md) 翻译；性能与兼容性描述为上游表述，未由本次独立测量。Bun 自身采用 MIT，发行物含不同许可的第三方组件，详见[上游许可说明](https://github.com/oven-sh/bun/blob/6cdb3835d90f8e05e4ebff86c7135078c9eb7e88/LICENSE.md)。

<p align="center">
  <a href="https://bun.com"><img src="https://github.com/user-attachments/assets/50282090-adfd-4ddb-9e27-c30753c6b161" alt="Logo" height=170></a>
</p>
<h1 align="center">Bun</h1>

<p align="center">
<a href="https://bun.com/discord" target="_blank"><img height=20 src="https://img.shields.io/discord/876711213126520882" /></a>
<img src="https://img.shields.io/github/stars/oven-sh/bun" alt="stars">
<a href="https://twitter.com/jarredsumner/status/1542824445810642946"><img src="https://img.shields.io/static/v1?label=speed&message=fast&color=success" alt="Bun speed" /></a>
</p>

<div align="center">
  <a href="https://bun.com/docs">文档</a>
  <span>&nbsp;&nbsp;•&nbsp;&nbsp;</span>
  <a href="https://bun.com/discord">Discord</a>
  <span>&nbsp;&nbsp;•&nbsp;&nbsp;</span>
  <a href="https://github.com/oven-sh/bun/issues/new">Issues</a>
  <span>&nbsp;&nbsp;•&nbsp;&nbsp;</span>
  <a href="https://github.com/oven-sh/bun/issues/159">路线图</a>
  <br />
</div>

### [阅读文档 →](https://bun.com/docs)

## Bun 是什么？

Bun 是面向 JavaScript 和 TypeScript 应用的一体化工具包，以名为 `bun` 的单个可执行文件提供。

其核心是 _Bun 运行时_：一个快速的 JavaScript 运行时，旨在**直接替代 Node.js**。它用 Rust 编写，底层由 JavaScriptCore 驱动，可大幅降低启动时间和内存用量。

```bash
bun run index.tsx             # TS and JSX supported out-of-the-box
```

注：示例说明 Bun 开箱即用支持 TS 和 JSX。

`bun` 命令行工具还实现了测试运行器、脚本运行器和兼容 Node.js 的包管理器。开发时无需 1,000 个 node_modules，只需 `bun`。Bun 内置工具比现有选项快得多，在现有 Node.js 项目中通常只需很少修改甚至无需修改即可使用。

```bash
bun test                      # run tests
bun run start                 # run the `start` script in `package.json`
bun install <pkg>             # install a package
bunx cowsay 'Hello, world!'   # execute a package
```

上述命令依次运行测试、运行 `package.json` 中的 `start` 脚本、安装软件包，以及执行软件包。

## 安装

Bun 支持 Linux（x64 和 arm64）、macOS（x64 和 Apple Silicon）以及 Windows（x64 和 arm64）。

> **Linux 用户**——强烈推荐内核 5.6 或更高版本，最低为 5.1。

> **x64 用户**——如果遇到“illegal instruction”或类似错误，请查看 [CPU 要求](https://bun.com/docs/installation#cpu-requirements)。

```sh
# with install script (recommended)
curl -fsSL https://bun.com/install | bash

# on windows
powershell -c "irm bun.sh/install.ps1 | iex"

# with npm
npm install -g bun

# with Homebrew
brew tap oven-sh/bun
brew install bun

# with Docker
docker pull oven/bun
docker run --rm --init --ulimit memlock=-1:-1 oven/bun
```

这些示例依次使用推荐的安装脚本、Windows 安装脚本、npm、Homebrew 和 Docker。

### 升级

要升级到最新版本的 Bun，请运行：

```sh
bun upgrade
```

每次向 `main` 提交时，Bun 都会自动发布 canary 构建。要升级到最新 canary 构建，请运行：

```sh
bun upgrade --canary
```

[查看 canary 构建](https://github.com/oven-sh/bun/releases/tag/canary)

## 快速链接

- 简介
  - [Bun 是什么？](https://bun.com/docs/index)
  - [安装](https://bun.com/docs/installation)
  - [快速入门](https://bun.com/docs/quickstart)
  - [TypeScript](https://bun.com/docs/typescript)
  - [TypeScript 6](https://bun.com/docs/typescript-6)

- 模板
  - [`bun init`](https://bun.com/docs/runtime/templating/init)
  - [`bun create`](https://bun.com/docs/runtime/templating/create)

- 运行时
  - [`bun run`](https://bun.com/docs/runtime/index)
  - [文件类型（加载器）](https://bun.com/docs/runtime/file-types)
  - [JSX](https://bun.com/docs/runtime/jsx)
  - [环境变量](https://bun.com/docs/runtime/environment-variables)
  - [Bun API](https://bun.com/docs/runtime/bun-apis)
  - [Web API](https://bun.com/docs/runtime/web-apis)
  - [Node.js 兼容性](https://bun.com/docs/runtime/nodejs-compat)
  - [插件](https://bun.com/docs/runtime/plugins)
  - [监听模式 / 热重载](https://bun.com/docs/runtime/watch-mode)
  - [模块解析](https://bun.com/docs/runtime/module-resolution)
  - [自动安装](https://bun.com/docs/runtime/auto-install)
  - [bunfig.toml](https://bun.com/docs/runtime/bunfig)
  - [调试器](https://bun.com/docs/runtime/debugger)
  - [REPL](https://bun.com/docs/runtime/repl)
  - [$ Shell](https://bun.com/docs/runtime/shell)

- 包管理器
  - [`bun install`](https://bun.com/docs/pm/cli/install)
  - [`bun add`](https://bun.com/docs/pm/cli/add)
  - [`bun remove`](https://bun.com/docs/pm/cli/remove)
  - [`bun update`](https://bun.com/docs/pm/cli/update)
  - [`bun link`](https://bun.com/docs/pm/cli/link)
  - [`bun pm`](https://bun.com/docs/pm/cli/pm)
  - [`bun outdated`](https://bun.com/docs/pm/cli/outdated)
  - [`bun publish`](https://bun.com/docs/pm/cli/publish)
  - [`bun patch`](https://bun.com/docs/pm/cli/patch)
  - [`bun why`](https://bun.com/docs/pm/cli/why)
  - [`bun audit`](https://bun.com/docs/pm/cli/audit)
  - [`bun info`](https://bun.com/docs/pm/cli/info)
  - [全局缓存](https://bun.com/docs/pm/global-cache)
  - [全局存储](https://bun.com/docs/pm/global-store)
  - [隔离安装](https://bun.com/docs/pm/isolated-installs)
  - [工作区](https://bun.com/docs/pm/workspaces)
  - [目录](https://bun.com/docs/pm/catalogs)
  - [生命周期脚本](https://bun.com/docs/pm/lifecycle)
  - [过滤](https://bun.com/docs/pm/filter)
  - [锁文件](https://bun.com/docs/pm/lockfile)
  - [作用域与注册表](https://bun.com/docs/pm/scopes-registries)
  - [覆盖与解析](https://bun.com/docs/pm/overrides)
  - [安全扫描器 API](https://bun.com/docs/pm/security-scanner-api)
  - [`.npmrc`](https://bun.com/docs/pm/npmrc)

- 打包器
  - [`Bun.build`](https://bun.com/docs/bundler/index)
  - [加载器](https://bun.com/docs/bundler/loaders)
  - [插件](https://bun.com/docs/bundler/plugins)
  - [宏](https://bun.com/docs/bundler/macros)
  - [与 esbuild 对比](https://bun.com/docs/bundler/esbuild)
  - [CSS](https://bun.com/docs/bundler/css)
  - [HTML 与静态网站](https://bun.com/docs/bundler/html-static)
  - [热模块替换（HMR）](https://bun.com/docs/bundler/hot-reloading)
  - [使用 HTML 导入构建全栈应用](https://bun.com/docs/bundler/fullstack)
  - [独立 HTML](https://bun.com/docs/bundler/standalone-html)
  - [代码压缩器](https://bun.com/docs/bundler/minifier)

- 编译器
  - [单文件可执行程序](https://bun.com/docs/compiler/executables)
  - [字节码缓存](https://bun.com/docs/compiler/bytecode)

- 测试运行器
  - [`bun test`](https://bun.com/docs/test/index)
  - [编写测试](https://bun.com/docs/test/writing-tests)
  - [生命周期钩子](https://bun.com/docs/test/lifecycle)
  - [模拟](https://bun.com/docs/test/mocks)
  - [快照](https://bun.com/docs/test/snapshots)
  - [日期与时间](https://bun.com/docs/test/dates-times)
  - [DOM 测试](https://bun.com/docs/test/dom)
  - [代码覆盖率](https://bun.com/docs/test/code-coverage)
  - [配置](https://bun.com/docs/test/configuration)
  - [测试发现](https://bun.com/docs/test/discovery)
  - [报告器](https://bun.com/docs/test/reporters)
  - [运行时行为](https://bun.com/docs/test/runtime-behavior)

- 包运行器
  - [`bunx`](https://bun.com/docs/pm/bunx)

- API
  - [HTTP 服务器（`Bun.serve`）](https://bun.com/docs/runtime/http/server)
  - [HTTP 路由](https://bun.com/docs/runtime/http/routing)
  - [HTTP 错误处理](https://bun.com/docs/runtime/http/error-handling)
  - [HTTP 指标](https://bun.com/docs/runtime/http/metrics)
  - [WebSockets](https://bun.com/docs/runtime/http/websockets)
  - [Workers](https://bun.com/docs/runtime/workers)
  - [二进制数据](https://bun.com/docs/runtime/binary-data)
  - [流](https://bun.com/docs/runtime/streams)
  - [文件 I/O（`Bun.file`）](https://bun.com/docs/runtime/file-io)
  - [归档（tar）](https://bun.com/docs/runtime/archive)
  - [SQLite (`bun:sqlite`)](https://bun.com/docs/runtime/sqlite)
  - [PostgreSQL (`Bun.sql`)](https://bun.com/docs/runtime/sql)
  - [Redis (`Bun.redis`)](https://bun.com/docs/runtime/redis)
  - [S3 客户端（`Bun.s3`）](https://bun.com/docs/runtime/s3)
  - [FileSystemRouter](https://bun.com/docs/runtime/file-system-router)
  - [TCP 套接字](https://bun.com/docs/runtime/networking/tcp)
  - [UDP 套接字](https://bun.com/docs/runtime/networking/udp)
  - [全局对象](https://bun.com/docs/runtime/globals)
  - [子进程（spawn）](https://bun.com/docs/runtime/child-process)
  - [Cron (`Bun.cron`)](https://bun.com/docs/runtime/cron)
  - [WebView](https://bun.com/docs/runtime/webview)
  - [转译器（`Bun.Transpiler`）](https://bun.com/docs/runtime/transpiler)
  - [哈希](https://bun.com/docs/runtime/hashing)
  - [颜色（`Bun.color`）](https://bun.com/docs/runtime/color)
  - [Console](https://bun.com/docs/runtime/console)
  - [FFI (`bun:ffi`)](https://bun.com/docs/runtime/ffi)
  - [C 编译器（`bun:ffi` cc）](https://bun.com/docs/runtime/c-compiler)
  - [HTMLRewriter](https://bun.com/docs/runtime/html-rewriter)
  - [Cookie（`Bun.Cookie`）](https://bun.com/docs/runtime/cookies)
  - [CSRF (`Bun.CSRF`)](https://bun.com/docs/runtime/csrf)
  - [密钥（`Bun.secrets`）](https://bun.com/docs/runtime/secrets)
  - [YAML (`Bun.YAML`)](https://bun.com/docs/runtime/yaml)
  - [TOML (`Bun.TOML`)](https://bun.com/docs/runtime/toml)
  - [JSON5](https://bun.com/docs/runtime/json5)
  - [JSONL](https://bun.com/docs/runtime/jsonl)
  - [Markdown](https://bun.com/docs/runtime/markdown)
  - [图像处理](https://bun.com/docs/runtime/image)
  - [实用工具](https://bun.com/docs/runtime/utils)
  - [Node-API](https://bun.com/docs/runtime/node-api)
  - [Glob (`Bun.Glob`)](https://bun.com/docs/runtime/glob)
  - [Semver (`Bun.semver`)](https://bun.com/docs/runtime/semver)
  - [DNS](https://bun.com/docs/runtime/networking/dns)
  - [fetch API 扩展](https://bun.com/docs/runtime/networking/fetch)

## 指南

- 部署
  - [部署到 Vercel](https://bun.com/guides/deployment/vercel)
  - [部署到 Railway](https://bun.com/guides/deployment/railway)
  - [部署到 Render](https://bun.com/guides/deployment/render)
  - [部署到 AWS Lambda](https://bun.com/guides/deployment/aws-lambda)
  - [部署到 DigitalOcean](https://bun.com/guides/deployment/digital-ocean)
  - [部署到 Google Cloud Run](https://bun.com/guides/deployment/google-cloud-run)

- 二进制
  - [将 Blob 转换为字符串](https://bun.com/guides/binary/blob-to-string)
  - [将 Buffer 转换为 Blob](https://bun.com/guides/binary/buffer-to-blob)
  - [将 Blob 转换为 DataView](https://bun.com/guides/binary/blob-to-dataview)
  - [将 Buffer 转换为字符串](https://bun.com/guides/binary/buffer-to-string)
  - [将 Blob 转换为 ReadableStream](https://bun.com/guides/binary/blob-to-stream)
  - [将 Blob 转换为 Uint8Array](https://bun.com/guides/binary/blob-to-typedarray)
  - [将 DataView 转换为字符串](https://bun.com/guides/binary/dataview-to-string)
  - [将 Uint8Array 转换为 Blob](https://bun.com/guides/binary/typedarray-to-blob)
  - [将 Blob 转换为 ArrayBuffer](https://bun.com/guides/binary/blob-to-arraybuffer)
  - [将 ArrayBuffer 转换为 Blob](https://bun.com/guides/binary/arraybuffer-to-blob)
  - [将 Buffer 转换为 Uint8Array](https://bun.com/guides/binary/buffer-to-typedarray)
  - [将 Uint8Array 转换为 Buffer](https://bun.com/guides/binary/typedarray-to-buffer)
  - [将 Uint8Array 转换为字符串](https://bun.com/guides/binary/typedarray-to-string)
  - [将 Buffer 转换为 ArrayBuffer](https://bun.com/guides/binary/buffer-to-arraybuffer)
  - [将 ArrayBuffer 转换为 Buffer](https://bun.com/guides/binary/arraybuffer-to-buffer)
  - [将 ArrayBuffer 转换为字符串](https://bun.com/guides/binary/arraybuffer-to-string)
  - [将 Uint8Array 转换为 DataView](https://bun.com/guides/binary/typedarray-to-dataview)
  - [将 Buffer 转换为 ReadableStream](https://bun.com/guides/binary/buffer-to-readablestream)
  - [将 Uint8Array 转换为 ArrayBuffer](https://bun.com/guides/binary/typedarray-to-arraybuffer)
  - [将 ArrayBuffer 转换为 Uint8Array](https://bun.com/guides/binary/arraybuffer-to-typedarray)
  - [将 ArrayBuffer 转换为数字数组](https://bun.com/guides/binary/arraybuffer-to-array)
  - [将 Uint8Array 转换为 ReadableStream](https://bun.com/guides/binary/typedarray-to-readablestream)

- 生态系统
  - [使用 React 和 JSX](https://bun.com/guides/ecosystem/react)
  - [在 Bun 中使用 Gel](https://bun.com/guides/ecosystem/gel)
  - [在 Bun 中使用 Prisma](https://bun.com/guides/ecosystem/prisma)
  - [在 Bun 中使用 Prisma Postgres](https://bun.com/guides/ecosystem/prisma-postgres)
  - [为 Bun 应用添加 Sentry](https://bun.com/guides/ecosystem/sentry)
  - [创建 Discord 机器人](https://bun.com/guides/ecosystem/discordjs)
  - [使用 PM2 将 Bun 作为守护进程运行](https://bun.com/guides/ecosystem/pm2)
  - [在 Bun 中使用 Drizzle ORM](https://bun.com/guides/ecosystem/drizzle)
  - [在 Bun 中使用 Upstash Redis](https://bun.com/guides/ecosystem/upstash)
  - [使用 Nuxt 和 Bun 构建应用](https://bun.com/guides/ecosystem/nuxt)
  - [使用 Qwik 和 Bun 构建应用](https://bun.com/guides/ecosystem/qwik)
  - [使用 Astro 和 Bun 构建应用](https://bun.com/guides/ecosystem/astro)
  - [使用 Remix 和 Bun 构建应用](https://bun.com/guides/ecosystem/remix)
  - [使用 Vite 和 Bun 构建前端](https://bun.com/guides/ecosystem/vite)
  - [使用 Next.js 和 Bun 构建应用](https://bun.com/guides/ecosystem/nextjs)
  - [使用 systemd 将 Bun 作为守护进程运行](https://bun.com/guides/ecosystem/systemd)
  - [使用 Hono 和 Bun 构建 HTTP 服务器](https://bun.com/guides/ecosystem/hono)
  - [使用 SvelteKit 和 Bun 构建应用](https://bun.com/guides/ecosystem/sveltekit)
  - [使用 SolidStart 和 Bun 构建应用](https://bun.com/guides/ecosystem/solidstart)
  - [使用 TanStack Start 和 Bun 构建应用](https://bun.com/guides/ecosystem/tanstack-start)
  - [使用 Elysia 和 Bun 构建 HTTP 服务器](https://bun.com/guides/ecosystem/elysia)
  - [使用 StricJS 和 Bun 构建 HTTP 服务器](https://bun.com/guides/ecosystem/stric)
  - [使用 Docker 将 Bun 应用容器化](https://bun.com/guides/ecosystem/docker)
  - [使用 Express 和 Bun 构建 HTTP 服务器](https://bun.com/guides/ecosystem/express)
  - [通过 Drizzle ORM 使用 Neon Postgres](https://bun.com/guides/ecosystem/neon-drizzle)
  - [在服务端渲染（SSR）React 组件](https://bun.com/guides/ecosystem/ssr-react)
  - [使用 Mongoose 和 Bun 读写 MongoDB 数据](https://bun.com/guides/ecosystem/mongoose)
  - [在 Bun 中使用 Neon 的 Serverless Postgres](https://bun.com/guides/ecosystem/neon-serverless-postgres)

- HTMLRewriter
  - [使用 HTMLRewriter 从网页提取链接](https://bun.com/guides/html-rewriter/extract-links)
  - [提取社交分享图片和 Open Graph 标签](https://bun.com/guides/html-rewriter/extract-social-meta)

- HTTP
  - [热重载 HTTP 服务器](https://bun.com/guides/http/hot)
  - [常见 HTTP 服务器用法](https://bun.com/guides/http/server)
  - [编写简单的 HTTP 服务器](https://bun.com/guides/http/simple)
  - [为 HTTP 服务器配置 TLS](https://bun.com/guides/http/tls)
  - [使用 fetch 发送 HTTP 请求](https://bun.com/guides/http/fetch)
  - [使用 fetch() 代理 HTTP 请求](https://bun.com/guides/http/proxy)
  - [启动 HTTP 服务器集群](https://bun.com/guides/http/cluster)
  - [以 HTTP Response 流式传输文件](https://bun.com/guides/http/stream-file)
  - [在 Bun 中通过 Unix 域套接字使用 fetch](https://bun.com/guides/http/fetch-unix)
  - [使用 FormData 通过 HTTP 上传文件](https://bun.com/guides/http/file-uploads)
  - [使用异步迭代器的流式 HTTP 服务器](https://bun.com/guides/http/stream-iterator)
  - [使用 Node.js 流的流式 HTTP 服务器](https://bun.com/guides/http/stream-node-streams-in-bun)
  - [在 Bun 中使用服务器发送事件（SSE）](https://bun.com/guides/http/sse)

- 安装
  - [添加依赖](https://bun.com/guides/install/add)
  - [添加 Git 依赖](https://bun.com/guides/install/add-git)
  - [添加对等依赖](https://bun.com/guides/install/add-peer)
  - [添加可信依赖](https://bun.com/guides/install/trusted)
  - [添加开发依赖](https://bun.com/guides/install/add-dev)
  - [添加 tarball 依赖](https://bun.com/guides/install/add-tarball)
  - [添加可选依赖](https://bun.com/guides/install/add-optional)
  - [生成兼容 yarn 的锁文件](https://bun.com/guides/install/yarnlock)
  - [使用工作区配置单一仓库](https://bun.com/guides/install/workspaces)
  - [以不同名称安装软件包](https://bun.com/guides/install/npm-alias)
  - [在 GitHub Actions 中使用 Bun 安装依赖](https://bun.com/guides/install/cicd)
  - [将 bun install 与 Artifactory 配合使用](https://bun.com/guides/install/jfrog-artifactory)
  - [配置 git 比较 Bun 的 lockb 锁文件](https://bun.com/guides/install/git-diff-bun-lockfile)
  - [覆盖 bun install 的默认 npm 注册表](https://bun.com/guides/install/custom-registry)
  - [将 bun install 与 Azure Artifacts npm 注册表配合使用](https://bun.com/guides/install/azure-artifacts)
  - [从 npm install 迁移到 bun install](https://bun.com/guides/install/from-npm-install-to-bun-install)
  - [用 bun install 为组织作用域配置私有注册表](https://bun.com/guides/install/registry-scope)

- 进程
  - [从标准输入读取](https://bun.com/guides/process/stdin)
  - [监听 CTRL+C](https://bun.com/guides/process/ctrl-c)
  - [启动子进程](https://bun.com/guides/process/spawn)
  - [监听操作系统信号](https://bun.com/guides/process/os-signals)
  - [解析命令行参数](https://bun.com/guides/process/argv)
  - [读取子进程的 stderr](https://bun.com/guides/process/spawn-stderr)
  - [读取子进程的 stdout](https://bun.com/guides/process/spawn-stdout)
  - [以纳秒获取进程运行时间](https://bun.com/guides/process/nanoseconds)
  - [启动子进程并通过 IPC 通信](https://bun.com/guides/process/ipc)

- 读取文件
  - [读取 JSON 文件](https://bun.com/guides/read-file/json)
  - [检查文件是否存在](https://bun.com/guides/read-file/exists)
  - [将文件读取为字符串](https://bun.com/guides/read-file/string)
  - [将文件读取到 Buffer](https://bun.com/guides/read-file/buffer)
  - [获取文件的 MIME 类型](https://bun.com/guides/read-file/mime)
  - [监听目录变化](https://bun.com/guides/read-file/watch)
  - [将文件读取为 ReadableStream](https://bun.com/guides/read-file/stream)
  - [将文件读取到 Uint8Array](https://bun.com/guides/read-file/uint8array)
  - [将文件读取到 ArrayBuffer](https://bun.com/guides/read-file/arraybuffer)

- 运行时
  - [删除文件](https://bun.com/guides/runtime/delete-file)
  - [运行 Shell 命令](https://bun.com/guides/runtime/shell)
  - [导入 JSON 文件](https://bun.com/guides/runtime/import-json)
  - [导入 TOML 文件](https://bun.com/guides/runtime/import-toml)
  - [导入 YAML 文件](https://bun.com/guides/runtime/import-yaml)
  - [导入 JSON5 文件](https://bun.com/guides/runtime/import-json5)
  - [在 Bun 中设置时区](https://bun.com/guides/runtime/timezone)
  - [设置环境变量](https://bun.com/guides/runtime/set-env)
  - [重新映射导入路径](https://bun.com/guides/runtime/tsconfig-paths)
  - [删除目录](https://bun.com/guides/runtime/delete-directory)
  - [读取环境变量](https://bun.com/guides/runtime/read-env)
  - [将 HTML 文件作为文本导入](https://bun.com/guides/runtime/import-html)
  - [在 GitHub Actions 中安装并运行 Bun](https://bun.com/guides/runtime/cicd)
  - [使用网页调试器调试 Bun](https://bun.com/guides/runtime/web-debugger)
  - [为 Bun 安装 TypeScript 声明](https://bun.com/guides/runtime/typescript)
  - [使用 VS Code 扩展调试 Bun](https://bun.com/guides/runtime/vscode-debugger)
  - [使用 V8 堆快照检查内存用量](https://bun.com/guides/runtime/heap-snapshot)
  - [定义和替换静态全局变量及常量](https://bun.com/guides/runtime/define-constant)
  - [使用 --define 设置构建时常量](https://bun.com/guides/runtime/build-time-constants)
  - [在 macOS 上对单文件 JavaScript 可执行程序进行代码签名](https://bun.com/guides/runtime/codesign-macos-executable)

- 流
  - [将 ReadableStream 转换为 JSON](https://bun.com/guides/streams/to-json)
  - [将 ReadableStream 转换为 Blob](https://bun.com/guides/streams/to-blob)
  - [将 ReadableStream 转换为 Buffer](https://bun.com/guides/streams/to-buffer)
  - [将 ReadableStream 转换为字符串](https://bun.com/guides/streams/to-string)
  - [将 ReadableStream 转换为 Uint8Array](https://bun.com/guides/streams/to-typedarray)
  - [将 ReadableStream 转换为数据块数组](https://bun.com/guides/streams/to-array)
  - [将 Node.js Readable 转换为 JSON](https://bun.com/guides/streams/node-readable-to-json)
  - [将 ReadableStream 转换为 ArrayBuffer](https://bun.com/guides/streams/to-arraybuffer)
  - [将 Node.js Readable 转换为 Blob](https://bun.com/guides/streams/node-readable-to-blob)
  - [将 Node.js Readable 转换为字符串](https://bun.com/guides/streams/node-readable-to-string)
  - [将 Node.js Readable 转换为 Uint8Array](https://bun.com/guides/streams/node-readable-to-uint8array)
  - [将 Node.js Readable 转换为 ArrayBuffer](https://bun.com/guides/streams/node-readable-to-arraybuffer)

- 测试
  - [在 `bun test` 中监视方法](https://bun.com/guides/test/spy-on)
  - [使用 Bun 测试运行器提前终止](https://bun.com/guides/test/bail)
  - [在 `bun test` 中模拟函数](https://bun.com/guides/test/mock-functions)
  - [使用 Bun 在监听模式下运行测试](https://bun.com/guides/test/watch-mode)
  - [在 `bun test` 中使用快照测试](https://bun.com/guides/test/snapshot)
  - [使用 Bun 测试运行器跳过测试](https://bun.com/guides/test/skip-tests)
  - [在 Bun 中使用 Testing Library](https://bun.com/guides/test/testing-library)
  - [在 `bun test` 中更新快照](https://bun.com/guides/test/update-snapshots)
  - [使用 Bun 测试运行器运行测试](https://bun.com/guides/test/run-tests)
  - [在 Bun 测试运行器中设置系统时间](https://bun.com/guides/test/mock-clock)
  - [使用 Bun 测试运行器为每个测试设置超时](https://bun.com/guides/test/timeout)
  - [从 Jest 迁移到 Bun 测试运行器](https://bun.com/guides/test/migrate-from-jest)
  - [使用 Bun 和 happy-dom 编写浏览器 DOM 测试](https://bun.com/guides/test/happy-dom)
  - [使用 Bun 测试运行器将测试标记为“待办”](https://bun.com/guides/test/todo-tests)
  - [使用 Bun 测试运行器多次重新运行测试](https://bun.com/guides/test/rerun-each)
  - [使用 Bun 测试运行器生成代码覆盖率报告](https://bun.com/guides/test/coverage)
  - [使用 bun test 导入、require 并测试 Svelte 组件](https://bun.com/guides/test/svelte-test)
  - [使用 Bun 测试运行器设置代码覆盖率阈值](https://bun.com/guides/test/coverage-threshold)
  - [使用 glob 模式选择性地并发运行测试](https://bun.com/guides/test/concurrent-test-glob)

- 实用工具
  - [生成 UUID](https://bun.com/guides/util/javascript-uuid)
  - [对密码计算哈希](https://bun.com/guides/util/hash-a-password)
  - [转义 HTML 字符串](https://bun.com/guides/util/escape-html)
  - [获取当前 Bun 版本](https://bun.com/guides/util/version)
  - [将 Bun 升级到最新版本](https://bun.com/guides/util/upgrade)
  - [编码和解码 base64 字符串](https://bun.com/guides/util/base64)
  - [使用 gzip 压缩和解压数据](https://bun.com/guides/util/gzip)
  - [休眠固定的毫秒数](https://bun.com/guides/util/sleep)
  - [检测代码何时通过 Bun 执行](https://bun.com/guides/util/detect-bun)
  - [检查两个对象是否深度相等](https://bun.com/guides/util/deep-equals)
  - [使用 DEFLATE 压缩和解压数据](https://bun.com/guides/util/deflate)
  - [获取当前入口的绝对路径](https://bun.com/guides/util/main)
  - [获取当前文件所在目录](https://bun.com/guides/util/import-meta-dir)
  - [检查当前文件是否为入口](https://bun.com/guides/util/entrypoint)
  - [获取当前文件名](https://bun.com/guides/util/import-meta-file)
  - [将文件 URL 转换为绝对路径](https://bun.com/guides/util/file-url-to-path)
  - [将绝对路径转换为文件 URL](https://bun.com/guides/util/path-to-file-url)
  - [获取当前文件的绝对路径](https://bun.com/guides/util/import-meta-path)
  - [获取可执行二进制文件的路径](https://bun.com/guides/util/which-path-to-executable-bin)

- WebSocket
  - [构建发布/订阅 WebSocket 服务器](https://bun.com/guides/websocket/pubsub)
  - [构建简单的 WebSocket 服务器](https://bun.com/guides/websocket/simple)
  - [为 WebSocket 消息启用压缩](https://bun.com/guides/websocket/compression)
  - [为 WebSocket 设置每个套接字的上下文数据](https://bun.com/guides/websocket/context)

- 写入文件
  - [删除文件](https://bun.com/guides/write-file/unlink)
  - [写入标准输出](https://bun.com/guides/write-file/stdout)
  - [将文件写入标准输出](https://bun.com/guides/write-file/cat)
  - [将 Blob 写入文件](https://bun.com/guides/write-file/blob)
  - [将字符串写入文件](https://bun.com/guides/write-file/basic)
  - [向文件追加内容](https://bun.com/guides/write-file/append)
  - [增量写入文件](https://bun.com/guides/write-file/filesink)
  - [将 Response 写入文件](https://bun.com/guides/write-file/response)
  - [将文件复制到另一位置](https://bun.com/guides/write-file/file-cp)
  - [将 ReadableStream 写入文件](https://bun.com/guides/write-file/stream)

## 参与贡献

开始为 Bun 做贡献，请参阅[项目 > 贡献](https://bun.com/docs/project/contributing)指南。

## 许可证

关于 Bun 许可的信息，请参阅[项目 > 许可证](https://bun.com/docs/project/license)页面。
