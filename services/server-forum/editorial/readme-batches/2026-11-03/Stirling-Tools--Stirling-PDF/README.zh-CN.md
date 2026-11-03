<p align="center">
  <img src="https://raw.githubusercontent.com/Stirling-Tools/Stirling-PDF/c687f2f5eb915b72900ae6c5890d352751ec844f/docs/stirling.png" width="80" alt="Stirling PDF logo">
</p>

<h1 align="center">Stirling PDF - 开源 PDF 平台</h1>

Stirling PDF 是一个开源 PDF 编辑平台。你可以将它作为个人桌面应用、在浏览器中运行，或部署到自己的服务器并使用私有 API。无需将文档发送给外部服务，即可编辑、签署、遮盖、转换 PDF 并自动化处理。

<p align="center">
  <a href="https://hub.docker.com/r/stirlingtools/stirling-pdf">
    <img src="https://img.shields.io/docker/pulls/frooodle/s-pdf" alt="Docker Pulls">
  </a>
  <a href="https://discord.gg/HYmhKj45pU">
    <img src="https://img.shields.io/discord/1068636748814483718?label=Discord" alt="Discord">
  </a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/Stirling-Tools/Stirling-PDF">
    <img src="https://api.scorecard.dev/projects/github.com/Stirling-Tools/Stirling-PDF/badge" alt="OpenSSF Scorecard">
  </a>
  <a href="https://github.com/Stirling-Tools/stirling-pdf">
    <img src="https://img.shields.io/github/stars/stirling-tools/stirling-pdf?style=social" alt="GitHub Repo stars">
  </a>
</p>

![Stirling PDF - Dashboard](https://raw.githubusercontent.com/Stirling-Tools/Stirling-PDF/c687f2f5eb915b72900ae6c5890d352751ec844f/docs/images/home-light.png)

## 主要功能

- **适用多种工作场景** - 桌面客户端、浏览器界面，以及带私有 API 的自托管服务器。
- **50 多种 PDF 工具** - 编辑、合并、拆分、签署、遮盖、转换、OCR、压缩等。
- **自动化与工作流** - 在界面中直接构建无代码流程，并通过 API 处理数百万份 PDF。
- **面向企业** - 单点登录、审计与灵活的本地部署。
- **开发者平台** - 几乎所有工具都提供 REST API，可集成到现有系统中。
- **多语言界面** - 界面提供 40 多种语言。

完整功能列表请参阅文档： **https://docs.stirlingpdf.com**

## 快速开始

```bash
docker run -p 8080:8080 docker.stirlingpdf.com/stirlingtools/stirling-pdf
```

然后打开： http://localhost:8080

完整安装选项（包括桌面版和 Kubernetes）请参阅[文档指南](https://docs.stirlingpdf.com/#documentation-guide).

## 资源

- [**文档**](https://docs.stirlingpdf.com)
- [**官网**](https://stirling.com)
- [**API 文档**](https://registry.scalar.com/@stirlingpdf/apis/stirling-pdf-processing-api/)
- [**服务器方案与企业版**](https://docs.stirlingpdf.com/Paid-Offerings)

## 支持

- **社区**： [Discord](https://discord.gg/HYmhKj45pU)
- **错误报告**： [GitHub Issues](https://github.com/Stirling-Tools/Stirling-PDF/issues)

## 参与贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](https://github.com/Stirling-Tools/Stirling-PDF/blob/c687f2f5eb915b72900ae6c5890d352751ec844f/.github/CONTRIBUTING.md)了解相关指南。

本项目使用 [Task](https://taskfile.dev/) 作为构建、开发和测试命令的统一运行工具。运行 `task dev` 启动编辑器，运行 `task` 查看最常用命令，或参阅[开发者指南](https://github.com/Stirling-Tools/Stirling-PDF/blob/c687f2f5eb915b72900ae6c5890d352751ec844f/docs/developer/DeveloperGuide.md)了解完整详情。

添加翻译请参阅[翻译指南](https://github.com/Stirling-Tools/Stirling-PDF/blob/c687f2f5eb915b72900ae6c5890d352751ec844f/docs/developer/HowToAddNewLanguage.md).

## 许可证

Stirling PDF 采用开放核心模式。详情请参阅 [LICENSE](https://github.com/Stirling-Tools/Stirling-PDF/blob/c687f2f5eb915b72900ae6c5890d352751ec844f/LICENSE)。
