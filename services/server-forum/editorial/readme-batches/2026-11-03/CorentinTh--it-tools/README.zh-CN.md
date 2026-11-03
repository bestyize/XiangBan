> 译注：原文 TypeScript Vue Plugin 扩展页面本次检查返回 HTTP 404；可先查看原文同时提供的 [Volar 扩展页面](https://marketplace.visualstudio.com/items?itemName=Vue.volar)。原链接保留，替代效果需自行确认。

<picture>
    <source srcset="https://raw.githubusercontent.com/CorentinTh/it-tools/d505845f918e946ec300af7b36efc107e2f66e9e/.github/logo-dark.png" media="(prefers-color-scheme: light)">
    <source srcset="https://raw.githubusercontent.com/CorentinTh/it-tools/d505845f918e946ec300af7b36efc107e2f66e9e/.github/logo-white.png" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/CorentinTh/it-tools/d505845f918e946ec300af7b36efc107e2f66e9e/.github/logo-dark.png" alt="logo">
</picture>

<p align="center">
为开发者和 IT 从业者提供实用工具。 <a href="https://it-tools.tech">试用！</a>
</p>

## 功能和路线图

请查看 [issues](https://github.com/CorentinTh/it-tools/issues)，了解是否已有待实现的功能。

有工具创意？请提交[功能请求](https://github.com/CorentinTh/it-tools/issues/new/choose)!

## 自托管

适合家庭实验环境的自托管方案

**从 Docker Hub 获取：**

```sh
docker run -d --name it-tools --restart unless-stopped -p 8080:80 corentinth/it-tools:latest
```

**从 GitHub Packages 获取：**

```sh
docker run -d --name it-tools --restart unless-stopped -p 8080:80 ghcr.io/corentinth/it-tools:latest
```

**其他方案：**

- [Cloudron](https://www.cloudron.io/store/tech.ittools.cloudron.html)
- [Tipi](https://www.runtipi.io/docs/apps-available)
- [Unraid](https://unraid.net/community/apps?q=it-tools)

## 贡献

### 推荐的 IDE 配置

[VSCode](https://code.visualstudio.com/) 及以下扩展：

- [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) （并禁用 Vetur）
- [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin).
- [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint)
- [i18n Ally](https://marketplace.visualstudio.com/items?itemName=lokalise.i18n-ally)

使用以下设置：

```json
{
  "editor.formatOnSave": false,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "i18n-ally.localesPaths": ["locales", "src/tools/*/locales"],
  "i18n-ally.keystyle": "nested"
}
```

### TS 中导入 `.vue` 的类型支持

TypeScript 默认无法处理导入 `.vue` 文件的类型信息，因此我们使用 `vue-tsc` 替代 `tsc` CLI 进行类型检查。在编辑器中，我们需要 [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin)使 TypeScript 语言服务识别 `.vue` 类型。

如果你觉得独立 TypeScript 插件不够快，Volar 还实现了[接管模式](https://github.com/johnsoncodehk/volar/discussions/471#discussioncomment-1361669)，性能更好。可以通过以下步骤启用：

1. 禁用内置 TypeScript 扩展
   1. 在 VSCode 命令面板中运行 `Extensions: Show Built-in Extensions`
   2. 找到 `TypeScript and JavaScript Language Features`，右键选择 `Disable (Workspace)`
2. 从命令面板运行 `Developer: Reload Window`，重新加载 VSCode 窗口。

### 项目配置

```sh
pnpm install
```

### 开发环境编译和热重载

```sh
pnpm dev
```

### 生产环境类型检查、编译和压缩

```sh
pnpm build
```

### 使用 [Vitest](https://vitest.dev/) 运行单元测试

```sh
pnpm test
```

### 使用 [ESLint](https://eslint.org/) 运行 lint

```sh
pnpm lint
```

### 创建新工具

项目提供生成新工具样板代码的脚本，只需运行：

```sh
pnpm run script:create:tool my-tool-name
```

它会在 `src/tools` 中创建包含相应文件的目录，并在 `src/tools/index.ts` 中添加导入。你只需要将导入的工具添加到正确的类别并开发该工具。

## 贡献者

非常感谢所有已经参与贡献的人！

[![contributors](https://contrib.rocks/image?repo=corentinth/it-tools&refresh=1)](https://github.com/corentinth/it-tools/graphs/contributors)

## 致谢

由 [Corentin Thomasset](https://corentin.tech?utm_source=it-tools&utm_medium=readme) 用 ❤️ 编写.

本项目使用 [vercel.com](https://vercel.com) 持续部署.

贡献者图由 [contrib.rocks](https://contrib.rocks/preview?repo=corentinth/it-tools) 生成.

<a href="https://www.producthunt.com/posts/it-tools?utm_source=badge-featured&utm_medium=badge&utm_souce=badge-it&#0045;tools" target="_blank"><img src="https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=345793&theme=light" alt="IT&#0032;Tools - Collection&#0032;of&#0032;handy&#0032;online&#0032;tools&#0032;for&#0032;devs&#0044;&#0032;with&#0032;great&#0032;UX | Product Hunt" style="width: 250px; height: 54px;" width="250" height="54" /></a>
<a href="https://www.producthunt.com/posts/it-tools?utm_source=badge-top-post-badge&utm_medium=badge&utm_souce=badge-it&#0045;tools" target="_blank"><img src="https://api.producthunt.com/widgets/embed-image/v1/top-post-badge.svg?post_id=345793&theme=light&period=daily" alt="IT&#0032;Tools - Collection&#0032;of&#0032;handy&#0032;online&#0032;tools&#0032;for&#0032;devs&#0044;&#0032;with&#0032;great&#0032;UX | Product Hunt" style="width: 250px; height: 54px;" width="250" height="54" /></a>

## 许可证

本项目采用 [GNU GPLv3](https://github.com/CorentinTh/it-tools/blob/d505845f918e946ec300af7b36efc107e2f66e9e/LICENSE) 许可证.
