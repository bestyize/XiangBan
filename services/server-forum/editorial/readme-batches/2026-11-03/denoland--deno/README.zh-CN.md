# Deno

[![](https://img.shields.io/crates/v/deno.svg)](https://crates.io/crates/deno)
[![Twitter badge][]][Twitter link] [![Bluesky badge][]][Bluesky link]
[![Discord badge][]][Discord link] [![YouTube badge][]][YouTube link]

<img align="right" src="https://deno.land/logo.svg" height="150px" alt="the deno mascot dinosaur standing in the rain">

[Deno](https://deno.com)（[/ˈdiːnoʊ/](https://ipa-reader.com/?text=%CB%88di%CB%90no%CA%8A)，发音为 `dee-no`）是一个 JavaScript、TypeScript 和 WebAssembly 运行时，默认配置注重安全，并提供良好的开发体验。它基于 [V8](https://v8.dev/)、[Rust](https://www.rust-lang.org/) 和 [Tokio](https://tokio.rs/) 构建。

在[文档](https://docs.deno.com/runtime/manual)中了解更多 Deno 运行时信息。

## 安装

使用下列命令之一在系统上安装 Deno 运行时。Deno 有多种安装方式；完整的安装选项列表见[此处](https://docs.deno.com/runtime/manual/getting_started/installation)。

Shell (Mac, Linux):

```sh
curl -fsSL https://deno.land/install.sh | sh
```

PowerShell (Windows):

```powershell
irm https://deno.land/install.ps1 | iex
```

[Homebrew](https://formulae.brew.sh/formula/deno) (Mac):

```sh
brew install deno
```

[Chocolatey](https://community.chocolatey.org/packages/deno) (Windows):

```powershell
choco install deno
```

[WinGet](https://winstall.app/apps/DenoLand.Deno) (Windows):

```powershell
winget install --id=DenoLand.Deno
```

[Scoop](https://scoop.sh/#/apps?q=deno&id=678d8fb557b611df996989c675b1099630a5bbee)
(Windows):

```powershell
scoop install main/deno
```

### 从源代码构建并安装

从源代码构建 Deno 的完整说明见[此处](https://github.com/denoland/deno/blob/main/.github/CONTRIBUTING.md#building-from-source)。

## 你的第一个 Deno 程序

Deno 可用于许多不同的应用，最常见的用途是构建 Web 服务器。创建一个名为 `server.ts` 的文件，并写入以下 TypeScript 代码：

```ts
Deno.serve((_req: Request) => {
  return new Response("Hello, world!");
});
```

使用以下命令运行服务器：

```sh
deno run --allow-net server.ts
```

这会在 [http://localhost:8000](http://localhost:8000) 启动一个本地 Web 服务器。

在[文档](https://docs.deno.com/runtime/manual)中进一步了解如何编写和运行 Deno 程序。

## 更多资源

- **[Deno Docs](https://docs.deno.com)**: Deno 运行时、[Deno Deploy](https://deno.com/deploy) 等的官方指南与参考文档。
- **[Deno Standard Library](https://jsr.io/@std)**: 官方支持的 Deno 程序常用工具库。
- **[JSR](https://jsr.io/)**: 面向现代 JavaScript 和 TypeScript 的开源包注册中心。
- **[Developer Blog](https://deno.com/blog)**: Deno 团队发布的产品更新、教程等内容。

## 参与贡献

感谢你的帮助！参与贡献前，请阅读我们的[贡献说明](https://github.com/denoland/deno/blob/e7b39ebd55c85aaf6a29290c1f5a99e2e21e09dd/.github/CONTRIBUTING.md).

[Build status - Cirrus]: https://github.com/denoland/deno/workflows/ci/badge.svg?branch=main&event=push
[Build status]: https://github.com/denoland/deno/actions
[Twitter badge]: https://img.shields.io/twitter/follow/deno_land.svg?style=social&label=Follow
[Twitter link]: https://twitter.com/intent/follow?screen_name=deno_land
[Bluesky badge]: https://img.shields.io/badge/Follow-whitesmoke?logo=bluesky
[Bluesky link]: https://bsky.app/profile/deno.land
[YouTube badge]: https://img.shields.io/youtube/channel/subscribers/UCqC2G2M-rg4fzg1esKFLFIw?style=social
[YouTube link]: https://www.youtube.com/@deno_land
[Discord badge]: https://img.shields.io/discord/684898665143206084?logo=discord&style=social
[Discord link]: https://discord.gg/deno
