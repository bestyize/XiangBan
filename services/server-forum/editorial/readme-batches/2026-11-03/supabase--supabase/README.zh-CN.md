<p align="center">
<img src="https://user-images.githubusercontent.com/8291514/213727234-cda046d6-28c6-491a-b284-b86c5cede25d.png#gh-light-mode-only">
<img src="https://user-images.githubusercontent.com/8291514/213727225-56186826-bee8-43b5-9b15-86e839d89393.png#gh-dark-mode-only">
</p>

# Supabase

[Supabase](https://supabase.com) 是一个 Postgres 开发平台。我们使用企业级开源工具构建 Firebase 的功能。

- [x] 托管 Postgres 数据库。[文档](https://supabase.com/docs/guides/database)
- [x] 身份验证与授权。[文档](https://supabase.com/docs/guides/auth)
- [x] 自动生成的 API。
  - [x] REST。[文档](https://supabase.com/docs/guides/api)
  - [x] GraphQL。[文档](https://supabase.com/docs/guides/graphql)
  - [x] 实时订阅。[文档](https://supabase.com/docs/guides/realtime)
- [x] 函数。
  - [x] 数据库函数。[文档](https://supabase.com/docs/guides/database/functions)
  - [x] 边缘函数。[文档](https://supabase.com/docs/guides/functions)
- [x] 文件存储。[文档](https://supabase.com/docs/guides/storage)
- [x] AI 与向量／嵌入工具集。[文档](https://supabase.com/docs/guides/ai)
- [x] 控制面板。

![Supabase Dashboard](https://raw.githubusercontent.com/supabase/supabase/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/apps/www/public/images/github/supabase-dashboard.png)

关注此仓库的“releases”以接收重大更新通知。

<kbd><img src="https://raw.githubusercontent.com/supabase/supabase/d5f7f413ab356dc1a92075cb3cee4e40a957d5b1/web/static/watch-repo.gif" alt="Watch this repo"/></kbd>

## 文档

完整文档请访问 [supabase.com/docs](https://supabase.com/docs)。

有关如何贡献，请参阅[入门指南](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/DEVELOPERS.md)。

## 社区与支持

- [社区论坛](https://github.com/supabase/supabase/discussions)：适合寻求开发帮助及讨论数据库最佳实践。
- [GitHub Issues](https://github.com/supabase/supabase/issues)：适合报告使用 Supabase 遇到的缺陷和错误。
- [邮件支持](https://supabase.com/docs/support#business-support)：适合解决数据库或基础设施问题。
- [Discord](https://discord.supabase.com)：适合分享应用并与社区交流。

## 工作方式

Supabase 是一组开源工具的组合。我们使用企业级开源产品构建 Firebase 的功能。如果已有工具及社区，并采用 MIT、Apache 2 或同等开放许可，我们就会使用并支持它；如果工具不存在，我们就自行开发并开源。Supabase 并非与 Firebase 一一对应，我们的目标是使用开源工具为开发者提供类似 Firebase 的开发体验。

**架构**

Supabase 是一个[托管平台](https://supabase.com/dashboard)。你可以注册并开始使用，无需安装任何软件。你也可以[自托管](https://supabase.com/docs/guides/hosting/overview)并[在本地开发](https://supabase.com/docs/guides/local-development)。

![Architecture](https://raw.githubusercontent.com/supabase/supabase/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/apps/docs/public/img/supabase-architecture.svg)

- [Postgres](https://www.postgresql.org/) 是对象关系型数据库系统，历经 30 多年的持续开发，以可靠性、健壮的功能和性能享有良好声誉。
- [Realtime](https://github.com/supabase/realtime) 是一个 Elixir 服务器，允许你通过 WebSocket 监听 PostgreSQL 的插入、更新和删除。Realtime 轮询 Postgres 内置复制功能获取数据库变更，将变更转换为 JSON，再通过 WebSocket 广播给已授权的客户端。
- [PostgREST](http://postgrest.org/) 是一个 Web 服务器，可将 PostgreSQL 数据库直接转换为 RESTful API。
- [GoTrue](https://github.com/supabase/gotrue) 是基于 JWT 的身份验证 API，可简化应用的用户注册、登录和会话管理。
- [Storage](https://github.com/supabase/storage-api) 是用于管理 S3 文件的 RESTful API，权限由 Postgres 处理。
- [pg_graphql](http://github.com/supabase/pg_graphql/) 是提供 GraphQL API 的 PostgreSQL 扩展。
- [postgres-meta](https://github.com/supabase/postgres-meta) 是管理 Postgres 的 RESTful API，可获取表、添加角色和运行查询等。
- [Envoy](https://github.com/envoyproxy/envoy) 是云原生高性能边缘和服务代理。

#### 客户端库

我们的客户端库采用模块化设计。每个子库都是针对单个外部系统的独立实现，这是我们支持现有工具的方式之一。

<table style="table-layout:fixed; white-space: nowrap;">
  <tr>
    <th>语言</th>
    <th>客户端</th>
    <th colspan="5">功能客户端（随 Supabase 客户端提供）</th>
  </tr>
  <!-- notranslate -->
  <tr>
    <th></th>
    <th>Supabase</th>
    <th><a href="https://github.com/postgrest/postgrest" target="_blank" rel="noopener noreferrer">PostgREST</a></th>
    <th><a href="https://github.com/supabase/gotrue" target="_blank" rel="noopener noreferrer">GoTrue</a></th>
    <th><a href="https://github.com/supabase/realtime" target="_blank" rel="noopener noreferrer">Realtime</a></th>
    <th><a href="https://github.com/supabase/storage-api" target="_blank" rel="noopener noreferrer">Storage</a></th>
    <th>Functions</th>
  </tr>
  <!-- TEMPLATE FOR NEW ROW -->
  <!-- START ROW
  <tr>
    <td>lang</td>
    <td><a href="https://github.com/supabase-community/supabase-lang" target="_blank" rel="noopener noreferrer">supabase-lang</a></td>
    <td><a href="https://github.com/supabase-community/postgrest-lang" target="_blank" rel="noopener noreferrer">postgrest-lang</a></td>
    <td><a href="https://github.com/supabase-community/gotrue-lang" target="_blank" rel="noopener noreferrer">gotrue-lang</a></td>
    <td><a href="https://github.com/supabase-community/realtime-lang" target="_blank" rel="noopener noreferrer">realtime-lang</a></td>
    <td><a href="https://github.com/supabase-community/storage-lang" target="_blank" rel="noopener noreferrer">storage-lang</a></td>
  </tr>
  END ROW -->
  <!-- /notranslate -->
  <th colspan="7">⚡️ 官方 ⚡️</th>
  <!-- notranslate -->
  <tr>
    <td>JavaScript (TypeScript)</td>
    <td><a href="https://github.com/supabase/supabase-js" target="_blank" rel="noopener noreferrer">supabase-js</a></td>
    <td><a href="https://github.com/supabase/supabase-js/tree/master/packages/core/postgrest-js" target="_blank" rel="noopener noreferrer">postgrest-js</a></td>
    <td><a href="https://github.com/supabase/supabase-js/tree/master/packages/core/auth-js" target="_blank" rel="noopener noreferrer">auth-js</a></td>
    <td><a href="https://github.com/supabase/supabase-js/tree/master/packages/core/realtime-js" target="_blank" rel="noopener noreferrer">realtime-js</a></td>
    <td><a href="https://github.com/supabase/supabase-js/tree/master/packages/core/storage-js" target="_blank" rel="noopener noreferrer">storage-js</a></td>
    <td><a href="https://github.com/supabase/supabase-js/tree/master/packages/core/functions-js" target="_blank" rel="noopener noreferrer">functions-js</a></td>
  </tr>
    <tr>
    <td>Flutter</td>
    <td><a href="https://github.com/supabase/supabase-flutter" target="_blank" rel="noopener noreferrer">supabase-flutter</a></td>
    <td><a href="https://github.com/supabase/supabase-flutter/tree/main/packages/postgrest" target="_blank" rel="noopener noreferrer">postgrest</a></td>
    <td><a href="https://github.com/supabase/supabase-flutter/tree/main/packages/supabase_auth" target="_blank" rel="noopener noreferrer">supabase_auth</a></td>
    <td><a href="https://github.com/supabase/supabase-flutter/tree/main/packages/supabase_realtime" target="_blank" rel="noopener noreferrer">supabase_realtime</a></td>
    <td><a href="https://github.com/supabase/supabase-flutter/tree/main/packages/supabase_storage" target="_blank" rel="noopener noreferrer">supabase_storage</a></td>
    <td><a href="https://github.com/supabase/supabase-flutter/tree/main/packages/supabase_functions" target="_blank" rel="noopener noreferrer">supabase_functions</a></td>
  </tr>
  <tr>
    <td>Swift</td>
    <td><a href="https://github.com/supabase/supabase-swift" target="_blank" rel="noopener noreferrer">supabase-swift</a></td>
    <td><a href="https://github.com/supabase/supabase-swift/tree/main/Sources/PostgREST" target="_blank" rel="noopener noreferrer">postgrest-swift</a></td>
    <td><a href="https://github.com/supabase/supabase-swift/tree/main/Sources/Auth" target="_blank" rel="noopener noreferrer">auth-swift</a></td>
    <td><a href="https://github.com/supabase/supabase-swift/tree/main/Sources/Realtime" target="_blank" rel="noopener noreferrer">realtime-swift</a></td>
    <td><a href="https://github.com/supabase/supabase-swift/tree/main/Sources/Storage" target="_blank" rel="noopener noreferrer">storage-swift</a></td>
    <td><a href="https://github.com/supabase/supabase-swift/tree/main/Sources/Functions" target="_blank" rel="noopener noreferrer">functions-swift</a></td>
  </tr>
  <tr>
    <td>Python</td>
    <td><a href="https://github.com/supabase/supabase-py" target="_blank" rel="noopener noreferrer">supabase-py</a></td>
    <td><a href="https://github.com/supabase/postgrest-py" target="_blank" rel="noopener noreferrer">postgrest-py</a></td>
    <td><a href="https://github.com/supabase/gotrue-py" target="_blank" rel="noopener noreferrer">gotrue-py</a></td>
    <td><a href="https://github.com/supabase/realtime-py" target="_blank" rel="noopener noreferrer">realtime-py</a></td>
    <td><a href="https://github.com/supabase/storage-py" target="_blank" rel="noopener noreferrer">storage-py</a></td>
    <td><a href="https://github.com/supabase/functions-py" target="_blank" rel="noopener noreferrer">functions-py</a></td>
  </tr>
  <!-- /notranslate -->
  <th colspan="7">💚 社区 💚</th>
  <!-- notranslate -->
  <tr>
    <td>C#</td>
    <td><a href="https://github.com/supabase-community/supabase-csharp" target="_blank" rel="noopener noreferrer">supabase-csharp</a></td>
    <td><a href="https://github.com/supabase-community/postgrest-csharp" target="_blank" rel="noopener noreferrer">postgrest-csharp</a></td>
    <td><a href="https://github.com/supabase-community/gotrue-csharp" target="_blank" rel="noopener noreferrer">gotrue-csharp</a></td>
    <td><a href="https://github.com/supabase-community/realtime-csharp" target="_blank" rel="noopener noreferrer">realtime-csharp</a></td>
    <td><a href="https://github.com/supabase-community/storage-csharp" target="_blank" rel="noopener noreferrer">storage-csharp</a></td>
    <td><a href="https://github.com/supabase-community/functions-csharp" target="_blank" rel="noopener noreferrer">functions-csharp</a></td>
  </tr>
  <tr>
    <td>Go</td>
    <td>-</td>
    <td><a href="https://github.com/supabase-community/postgrest-go" target="_blank" rel="noopener noreferrer">postgrest-go</a></td>
    <td><a href="https://github.com/supabase-community/gotrue-go" target="_blank" rel="noopener noreferrer">gotrue-go</a></td>
    <td>-</td>
    <td><a href="https://github.com/supabase-community/storage-go" target="_blank" rel="noopener noreferrer">storage-go</a></td>
    <td><a href="https://github.com/supabase-community/functions-go" target="_blank" rel="noopener noreferrer">functions-go</a></td>
  </tr>
  <tr>
    <td>Java</td>
    <td>-</td>
    <td>-</td>
    <td><a href="https://github.com/supabase-community/gotrue-java" target="_blank" rel="noopener noreferrer">gotrue-java</a></td>
    <td>-</td>
    <td><a href="https://github.com/supabase-community/storage-java" target="_blank" rel="noopener noreferrer">storage-java</a></td>
    <td>-</td>
  </tr>
  <tr>
    <td>Kotlin</td>
    <td><a href="https://github.com/supabase-community/supabase-kt" target="_blank" rel="noopener noreferrer">supabase-kt</a></td>
    <td><a href="https://github.com/supabase-community/supabase-kt/tree/master/Postgrest" target="_blank" rel="noopener noreferrer">postgrest-kt</a></td>
    <td><a href="https://github.com/supabase-community/supabase-kt/tree/master/Auth" target="_blank" rel="noopener noreferrer">auth-kt</a></td>
    <td><a href="https://github.com/supabase-community/supabase-kt/tree/master/Realtime" target="_blank" rel="noopener noreferrer">realtime-kt</a></td>
    <td><a href="https://github.com/supabase-community/supabase-kt/tree/master/Storage" target="_blank" rel="noopener noreferrer">storage-kt</a></td>
    <td><a href="https://github.com/supabase-community/supabase-kt/tree/master/Functions" target="_blank" rel="noopener noreferrer">functions-kt</a></td>
  </tr>
  <tr>
    <td>Ruby</td>
    <td><a href="https://github.com/supabase-community/supabase-rb" target="_blank" rel="noopener noreferrer">supabase-rb</a></td>
    <td><a href="https://github.com/supabase-community/postgrest-rb" target="_blank" rel="noopener noreferrer">postgrest-rb</a></td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
  </tr>
  <tr>
    <td>Rust</td>
    <td>-</td>
    <td><a href="https://github.com/supabase-community/postgrest-rs" target="_blank" rel="noopener noreferrer">postgrest-rs</a></td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
  </tr>
  <tr>
    <td>Godot Engine (GDScript)</td>
    <td><a href="https://github.com/supabase-community/godot-engine.supabase" target="_blank" rel="noopener noreferrer">supabase-gdscript</a></td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
    <td>-</td>
  </tr>
  <!-- /notranslate -->
</table>

<!--- Remove this list if you're translating to another language, it's hard to keep updated across multiple files-->
<!--- Keep only the link to the list of translation files-->

## 徽章

![Made with Supabase](https://raw.githubusercontent.com/supabase/supabase/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/apps/www/public/badge-made-with-supabase.svg)

```md
[![Made with Supabase](https://supabase.com/badge-made-with-supabase.svg)](https://supabase.com)
```

```html
<a href="https://supabase.com">
  <img
    width="168"
    height="30"
    src="https://supabase.com/badge-made-with-supabase.svg"
    alt="Made with Supabase"
  />
</a>
```

![Made with Supabase (dark)](https://raw.githubusercontent.com/supabase/supabase/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/apps/www/public/badge-made-with-supabase-dark.svg)

```md
[![Made with Supabase](https://supabase.com/badge-made-with-supabase-dark.svg)](https://supabase.com)
```

```html
<a href="https://supabase.com">
  <img
    width="168"
    height="30"
    src="https://supabase.com/badge-made-with-supabase-dark.svg"
    alt="Made with Supabase"
  />
</a>
```

## 翻译

- [Arabic | العربية](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ar.md)
- [Albanian / Shqip](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.sq.md)
- [Bangla / বাংলা](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.bn.md)
- [Bulgarian / Български](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.bg.md)
- [Catalan / Català](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ca.md)
- [Croatian / Hrvatski](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.hr.md)
- [Czech / čeština](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.cs.md)
- [Danish / Dansk](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.da.md)
- [Dutch / Nederlands](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.nl.md)
- [English](https://github.com/supabase/supabase)
- [Estonian / eesti keel](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.et.md)
- [Finnish / Suomalainen](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.fi.md)
- [French / Français](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.fr.md)
- [German / Deutsch](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.de.md)
- [Greek / Ελληνικά](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.el.md)
- [Gujarati / ગુજરાતી](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.gu.md)
- [Hebrew / עברית](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.he.md)
- [Hindi / हिंदी](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.hi.md)
- [Hungarian / Magyar](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.hu.md)
- [Nepali / नेपाली](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ne.md)
- [Indonesian / Bahasa Indonesia](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.id.md)
- [Italiano / Italian](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.it.md)
- [Japanese / 日本語](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.jp.md)
- [Korean / 한국어](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ko.md)
- [Lithuanian / lietuvių](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.lt.md)
- [Latvian / latviski](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.lv.md)
- [Malay / Bahasa Malaysia](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ms.md)
- [Norwegian (Bokmål) / Norsk (Bokmål)](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.nb.md)
- [Persian / فارسی](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.fa.md)
- [Polish / Polski](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.pl.md)
- [Portuguese / Português](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.pt.md)
- [Portuguese (Brazilian) / Português Brasileiro](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.pt-br.md)
- [Romanian / Română](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ro.md)
- [Russian / Pусский](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.ru.md)
- [Serbian / Srpski](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.sr.md)
- [Sinhala / සිංහල](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.si.md)
- [Slovak / slovenský](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.sk.md)
- [Slovenian / Slovenščina](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.sl.md)
- [Spanish / Español](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.es.md)
- [Simplified Chinese / 简体中文](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.zh-cn.md)
- [Swedish / Svenska](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.sv.md)
- [Thai / ไทย](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.th.md)
- [Traditional Chinese / 繁體中文](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.zh-tw.md)
- [Turkish / Türkçe](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.tr.md)
- [Ukrainian / Українська](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.uk.md)
- [Vietnamese / Tiếng Việt](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/README.vi-vn.md)
- [翻译列表](https://github.com/supabase/supabase/blob/5ce8532dfffcfc8f2acc11918966a9858bbb8cbe/i18n/languages.md) <!--- Keep only this -->
