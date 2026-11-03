<!--
 - SPDX-FileCopyrightText: 2016-2024 Nextcloud GmbH and Nextcloud contributors
 - SPDX-FileCopyrightText: 2013-2016 ownCloud, Inc.
 - SPDX-License-Identifier: AGPL-3.0-or-later
-->
# Nextcloud Server ☁
[![REUSE status](https://api.reuse.software/badge/github.com/nextcloud/server)](https://api.reuse.software/info/github.com/nextcloud/server)
[![codecov](https://codecov.io/gh/nextcloud/server/branch/master/graph/badge.svg)](https://codecov.io/gh/nextcloud/server)
[![CII Best Practices](https://bestpractices.coreinfrastructure.org/projects/209/badge)](https://bestpractices.coreinfrastructure.org/projects/209)
[设计状态](https://contribute.design/nextcloud/server)

> 译注：上游 Design 动态徽章暂不可用，此处保留其原始目标链接。

**为你的所有数据提供安全的家。**

![](https://raw.githubusercontent.com/nextcloud/screenshots/master/nextcloud-hub-25-files.png)

## 为什么它如此出色？🤩

* 📁 **访问数据** 可以将文件、联系人、日历等存储在自己选择的服务器上。
* 🔄 **同步数据** 在设备之间同步文件、联系人、日历等内容。
* 🙌 **分享数据** 让其他人访问你希望他们查看或共同协作的内容。
* 🚀 **可通过数百个应用扩展**，例如 [Calendar](https://github.com/nextcloud/calendar), [Contacts](https://github.com/nextcloud/contacts), [Mail](https://github.com/nextcloud/mail), [Video Chat](https://github.com/nextcloud/spreed)以及可在 [App Store](https://apps.nextcloud.com) 中发现的其他应用
* 🔒 **安全性**：通过加密机制、 [HackerOne 漏洞赏金计划](https://hackerone.com/nextcloud)和双因素认证提供保护。

想了解如何在家中和组织内使用 Nextcloud 访问、分享和保护文件、日历、联系人、通信等内容吗？[**了解全部功能**](https://nextcloud.com/athome/)。

## 获取 Nextcloud 🚚

- ☑️ [**直接注册**](https://nextcloud.com/signup/)：通过我们的网站或直接通过应用向服务提供商注册。
- 🖥 [自行**安装**服务器](https://nextcloud.com/install/#instructions-server)，使用自己的硬件或现成的**设备**
- 📦 购买预装 Nextcloud 的[**设备**](https://nextcloud.com/devices/)
- 🏢 寻找[服务**提供商**](https://nextcloud.com/providers/)，为你或公司托管 Nextcloud

企业、公共部门或教育用户？可以了解由 Nextcloud GmbH 提供的 [**Nextcloud Enterprise**](https://nextcloud.com/enterprise/)。

## 联系我们 💬

* [📋 论坛](https://help.nextcloud.com)
* [🦋 Bluesky](https://bsky.app/profile/nextcloud.bsky.social)
* [👥 Facebook](https://www.facebook.com/nextclouders)
* [🐘 Mastodon](https://mastodon.xyz/@nextcloud)

也可以[获取 Nextcloud 支持](https://nextcloud.com/support)!


## 加入团队 👪

贡献有很多方式，开发只是其中之一！了解[如何参与](https://nextcloud.com/contribute/)，例如担任翻译、设计师、测试者、帮助他人等！😍


### 开发环境配置 👩‍💻

1. 🚀 [配置本地开发环境](https://docs.nextcloud.com/server/latest/developer_manual/getting_started/devenv.html)
2. 🐛 [选择适合新手的 issue](https://github.com/nextcloud/server/labels/good%20first%20issue)
3. 👩‍🔧 创建分支并进行修改。记得使用 `git commit -sm "Your commit message"` 为提交签署 sign-off
4. ⬆ 创建 [pull request](https://opensource.guide/how-to-contribute/#opening-a-pull-request)并 `@mention` issue 中的相关人员进行审查
5. 👍 修复审查中发现的问题
6. 🎉 等待合并！

第三方组件作为 Git 子模块处理，需要先初始化。因此，除了常规 Git checkout，还需要运行 `git submodule update --init` 或类似命令；详情请参阅 Git 文档。

正式发行版中默认包含的一些应用，例如 [First run wizard](https://github.com/nextcloud/firstrunwizard)或 [Activity](https://github.com/nextcloud/activity)在 `master` 中不存在，需要手动克隆到 `apps` 子文件夹进行安装。

除此之外，使用 `stable*` 分支时，Git checkout 可以像发行归档一样处理。请注意，不应将它们用于生产系统。

#### 测试代码

我们针对代码的不同部分使用多种测试框架：
- PHPUnit 用于 PHP 单元测试
- Behat 用于 PHP 集成测试
- Vitest 用于 JavaScript / TypeScript 单元测试
- Playwright 用于端到端测试

有关使用 Playwright 的端到端测试，请参阅[文档](https://github.com/nextcloud/server/blob/5cc43956c923fabb689fbe1acc3e9a1ad0d8e088/tests/playwright/README.md)，了解如何调试错误及贡献新的测试用例。

### 使用的工具 🛠

- [👀 BrowserStack](https://browserstack.com)用于跨浏览器测试
- [🌊 WAVE](https://wave.webaim.org/extension/)用于无障碍测试
- [🚨 Lighthouse](https://developers.google.com/web/tools/lighthouse/)用于测试性能、无障碍等

#### GitHub 上的实用机器人 :robot:

- 在 pull request 中评论 `/update-3rdparty` 可更新第三方子模块。它会更新至与 PR 目标同名的第三方分支的最新提交。

#### 在 git blame 中忽略代码风格更新

`git config blame.ignoreRevsFile .git-blame-ignore-revs`

## 贡献指南 📜

本仓库自 2016 年 6 月 16 日及之后的所有贡献均视为根据 AGPLv3 或任何后续版本授权。

Nextcloud 不要求签署 CLA（贡献者许可协议）。版权属于各个贡献者。因此，我们建议对代码做出重大更改的贡献者在 [AUTHORS](https://github.com/nextcloud/server/blob/5cc43956c923fabb689fbe1acc3e9a1ad0d8e088/AUTHORS)文件中加入以下一行：

```
- <your name> <your email address>
```

请阅读[行为准则](https://nextcloud.com/community/code-of-conduct/)。该文档提供指导，帮助 Nextcloud 参与者在积极、鼓舞人心的氛围中有效合作，并说明如何共同增强和支持彼此。

请阅读本仓库的[贡献指南](https://github.com/nextcloud/server/blob/5cc43956c923fabb689fbe1acc3e9a1ad0d8e088/.github/CONTRIBUTING.md)。

有关如何贡献的更多信息： [https://nextcloud.com/contribute/](https://nextcloud.com/contribute/)
