> 中文译文由 Terra 翻译并审校，基于上游固定版本的 [README 原文](https://github.com/gchq/CyberChef/blob/b75398f1134fc59fd74d288f620aa1eecba77dc9/README.md)；原文许可与版权声明见 [上游许可证](https://github.com/gchq/CyberChef/blob/b75398f1134fc59fd74d288f620aa1eecba77dc9/LICENSE)。

# CyberChef

[![](https://github.com/gchq/CyberChef/workflows/Master%20Build,%20Test%20&%20Deploy/badge.svg)](https://github.com/gchq/CyberChef/actions?query=workflow%3A%22Master+Build%2C+Test+%26+Deploy%22)
[![npm](https://img.shields.io/npm/v/cyberchef.svg)](https://www.npmjs.com/package/cyberchef)
[![](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/gchq/CyberChef/blob/master/LICENSE)
[![Gitter](https://badges.gitter.im/gchq/CyberChef.svg)](https://gitter.im/gchq/CyberChef?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)


#### *网络瑞士军刀*

CyberChef 是一个简单直观的网页应用，可在浏览器中执行各类“网络”操作。这些操作包括 XOR 和 Base64 等简单编码、AES、DES 和 Blowfish 等较复杂加密、生成二进制和十六进制转储、数据压缩和解压、计算哈希与校验和、解析 IPv6 和 X.509、更改字符编码等等。

该工具旨在让技术和非技术分析人员都能以复杂方式处理数据，而无需面对复杂工具或算法。它由一名分析人员利用 10% 的创新时间，历经数年构思、设计、构建并逐步改进。

## 目录

- [官方网站](#官方网站)
- [本地运行](#本地运行)
  - [使用 Docker](#使用-docker)
  - [从源码构建](#从源码构建)
- [工作原理](#工作原理)
- [功能](#功能)
- [深度链接](#深度链接)
- [浏览器支持](#浏览器支持)
- [Node.js 支持](#nodejs-支持)
- [安全](#安全)
- [贡献](#贡献)
- [许可](#许可)

## 官方网站

[CyberChef 的官方网站在此][1]，祝使用愉快！

## 本地运行

可以使用 Docker 运行 CyberChef（无需工具链），或使用 Node.js 从源码构建。

### 使用 Docker

**前提条件**

- [Docker](https://www.docker.com/products/docker-desktop/)
  - 必须在本机打开并运行 Docker Desktop。

#### 选项 1：自行构建 Docker 镜像

1. 构建 Docker 镜像
```bash
docker build --tag cyberchef --ulimit nofile=10000 .
```
2. 运行 Docker 容器
```bash
docker run -it -p 8080:8080 cyberchef
```
3. 在浏览器中访问 `http://localhost:8080`

#### 选项 2：使用预构建 Docker 镜像

如果想跳过构建，可以使用预构建镜像：

```bash
docker run -it -p 8080:8080 ghcr.io/gchq/cyberchef:latest
```

同样，在浏览器中访问 `http://localhost:8080`。

此镜像通过 [GitHub Workflows](https://github.com/gchq/CyberChef/blob/b75398f1134fc59fd74d288f620aa1eecba77dc9/.github/workflows/releases.yml) 构建并发布。

### 从源码构建

如果要开发 CyberChef 或不使用 Docker 运行，可以直接使用 Node.js 构建。

**前提条件**

- [Node.js](https://nodejs.org/) `v24`（参阅 [Node.js 支持](#nodejs-支持)）

> [!NOTE]
>
> 可以使用 [nvm](https://github.com/nvm-sh/nvm) 管理 Node.js 版本，并使用[本项目](https://github.com/gchq/CyberChef/blob/b75398f1134fc59fd74d288f620aa1eecba77dc9/.nvmrc)当前开发版本，避免与计算机上的其他项目冲突。

**设置**

```bash
git clone https://github.com/gchq/CyberChef.git
cd CyberChef
npm install
```

**常用任务**

| 命令 | 说明 |
| --- | --- |
| `npm start` | 在 `http://localhost:8080` 运行支持实时重载的开发服务器。 |
| `npm run build` | 在 `build/prod` 目录生成生产构建。 |
| `npm test` | 运行 Node.js 和操作测试套件。 |
| `npm run testui` | 运行浏览器（UI）测试。 |
| `npm run lint` | 按 lint 规则检查代码。 |
| `npm run newop` | 通过交互式快速入门脚本创建新操作的框架。 |

如果构建大型配方时遇到内存不足错误，可使用 `npm run setheapsize` 增加 Node 的堆大小。

## 工作原理

CyberChef 有四个主要区域：

1. 右上角的**输入**框，可粘贴、输入或拖入要处理的文本或文件。
2. 右下角的**输出**框，显示处理结果。
3. 最左侧的**操作**列表，可在分类列表中查找或搜索 CyberChef 支持的全部操作。
4. 中间的**配方**区域，可将要使用的操作拖入并指定参数和选项。

你可以按简单或复杂的方式使用任意数量的操作。例如：

- [解码 Base64 编码的字符串][2]
- [将日期和时间转换到其他时区][3]
- [解析 Teredo IPv6 地址][4]
- [转换十六进制转储数据并解压][5]
- [解密并反汇编 shellcode][6]
- [将多个时间戳显示为完整日期][7]
- [对不同类型的数据执行不同操作][8]
- [将部分输入作为操作参数][9]
- [执行 AES 解密，从密文流开头提取 IV][10]
- [以更简单的方式执行相同的 AES 解密][13]
- [自动检测多层嵌套编码][12]

## 功能

- 拖放
  - 可将操作拖入或拖出配方列表，也可重新排序。
  - 可将最大 2GB 的文件拖到输入框，直接载入浏览器。
- 自动烘焙
  - 每当修改输入或配方，CyberChef 会自动“烘焙”并立即生成输出。
  - 如果影响性能（例如输入非常大时），可关闭并手动操作。
- 自动编码检测
  - CyberChef 使用[多种技术](https://github.com/gchq/CyberChef/wiki/Automatic-detection-of-encoded-data-using-CyberChef-Magic)尝试自动检测数据的编码。如果找到能解释数据的合适操作，输出框会显示“魔法”图标，点击即可解码。
- 断点
  - 可在配方的任意操作上设置断点，在运行该操作前暂停。
  - 也可逐个操作单步执行配方，查看每个阶段的数据。
- 保存和加载配方
  - 如创建了日后还想使用的配方，只需点击“Save recipe”将其加入本地存储，下次访问 CyberChef 时仍可使用。
  - 也可以复制包含配方和输入的 URL，轻松分享给他人。
- 搜索
  - 如果知道所需操作的名称或相关词语，在搜索框中输入即可立即显示匹配操作。
- 高亮
  - 选中输入或输出中的文本时，会显示偏移量和长度，并在可能时高亮输出或输入中的对应数据（例如：[高亮输入中的 'question'，查看它在输出中的位置][11]）。
- 保存到文件和从文件加载
  - 随时可以将输出保存到文件，或将文件拖到输入框加载。支持约 2GB 的文件（取决于浏览器），但某些操作处理这么多数据可能耗时很长。
- CyberChef 完全在客户端运行
  - 配方配置或输入（文本或文件）均不会发送到 CyberChef 网页服务器；所有处理都在你自己计算机上的浏览器中进行。
  - 因此可以下载 CyberChef 并在本地运行。可使用应用左上角链接下载完整副本，将其放入虚拟机、分享给他人或托管在封闭网络中。

## 深度链接

通过修改 CyberChef 的 URL hash，可以更改页面打开时的初始设置。格式为 `https://gchq.github.io/CyberChef/#recipe=Operation()&input=...`

支持的参数为 `recipe`、`input`（Base64 编码）和 `theme`。

## 浏览器支持

CyberChef 设计支持：

- Google Chrome 50+
- Mozilla Firefox 38+

## Node.js 支持

CyberChef 设计为完全支持 Node.js `v24`，并提前针对即将成为活跃 LTS 版本的 Node.js `v26` 进行测试。详情请参阅 [“Node API” wiki 页面](https://github.com/gchq/CyberChef/wiki/Node-API)。

## 安全

请参阅 [CyberChef 安全政策](https://github.com/gchq/CyberChef/blob/b75398f1134fc59fd74d288f620aa1eecba77dc9/SECURITY.md)。

## 贡献

为 CyberChef 贡献新操作很容易！快速入门脚本会引导你完成整个过程。只要会编写基础 JavaScript，就能编写 CyberChef 操作。

[“Contributing” wiki 页面](https://github.com/gchq/CyberChef/wiki/Contributing)提供安装教程、添加操作和主题的指南，以及仓库结构、可用数据类型和编码约定的说明。

- 将更改推送到自己的 fork。
- 提交 pull request。首次提交时，系统会通过该 pull request 上的 CLA 助手提示你签署 [GCHQ 贡献者许可协议](https://cla-assistant.io/gchq/CyberChef)。其中还会询问你是否愿意让 GCHQ 联系你，向你赠送感谢礼物或介绍 GCHQ 的就业机会。

## 许可

CyberChef 基于 [Apache 2.0 许可证](https://www.apache.org/licenses/LICENSE-2.0)发布，并受[英国王室版权](https://www.nationalarchives.gov.uk/information-management/re-using-public-sector-information/uk-government-licensing-framework/crown-copyright/)保护。

  [1]: https://gchq.github.io/CyberChef
  [2]: https://gchq.github.io/CyberChef/#recipe=From_Base64('A-Za-z0-9%2B/%3D',true)&input=VTI4Z2JHOXVaeUJoYm1RZ2RHaGhibXR6SUdadmNpQmhiR3dnZEdobElHWnBjMmd1
  [3]: https://gchq.github.io/CyberChef/#recipe=Translate_DateTime_Format('Standard%20date%20and%20time','DD/MM/YYYY%20HH:mm:ss','UTC','dddd%20Do%20MMMM%20YYYY%20HH:mm:ss%20Z%20z','Australia/Queensland')&input=MTUvMDYvMjAxNSAyMDo0NTowMA
  [4]: https://gchq.github.io/CyberChef/#recipe=Parse_IPv6_address()&input=MjAwMTowMDAwOjQxMzY6ZTM3ODo4MDAwOjYzYmY6M2ZmZjpmZGQy
  [5]: https://gchq.github.io/CyberChef/#recipe=From_Hexdump()Gunzip()&input=MDAwMDAwMDAgIDFmIDhiIDA4IDAwIDEyIGJjIGYzIDU3IDAwIGZmIDBkIGM3IGMxIDA5IDAwIDIwICB8Li4uLi6881cu/y7HwS4uIHwKMDAwMDAwMTAgIDA4IDA1IGQwIDU1IGZlIDA0IDJkIGQzIDA0IDFmIGNhIDhjIDQ0IDIxIDViIGZmICB8Li7QVf4uLdMuLsouRCFb/3wKMDAwMDAwMjAgIDYwIGM3IGQ3IDAzIDE2IGJlIDQwIDFmIDc4IDRhIDNmIDA5IDg5IDBiIDlhIDdkICB8YMfXLi6%2BQC54Sj8uLi4ufXwKMDAwMDAwMzAgIDRlIGM4IDRlIDZkIDA1IDFlIDAxIDhiIDRjIDI0IDAwIDAwIDAwICAgICAgICAgICB8TshObS4uLi5MJC4uLnw
  [6]: https://gchq.github.io/CyberChef/#recipe=RC4(%7B'option':'UTF8','string':'secret'%7D,'Hex','Hex')Disassemble_x86('64','Full%20x86%20architecture',16,0,true,true)&input=MjFkZGQyNTQwMTYwZWU2NWZlMDc3NzEwM2YyYTM5ZmJlNWJjYjZhYTBhYWJkNDE0ZjkwYzZjYWY1MzEyNzU0YWY3NzRiNzZiM2JiY2QxOTNjYjNkZGZkYmM1YTI2NTMzYTY4NmI1OWI4ZmVkNGQzODBkNDc0NDIwMWFlYzIwNDA1MDcxMzhlMmZlMmIzOTUwNDQ2ZGIzMWQyYmM2MjliZTRkM2YyZWIwMDQzYzI5M2Q3YTVkMjk2MmMwMGZlNmRhMzAwNzJkOGM1YTZiNGZlN2Q4NTlhMDQwZWVhZjI5OTczMzYzMDJmNWEwZWMxOQ
  [7]: https://gchq.github.io/CyberChef/#recipe=Fork('%5C%5Cn','%5C%5Cn',false)From_UNIX_Timestamp('Seconds%20(s)')&input=OTc4MzQ2ODAwCjEwMTI2NTEyMDAKMTA0NjY5NjQwMAoxMDgxMDg3MjAwCjExMTUzMDUyMDAKMTE0OTYwOTYwMA
  [8]: https://gchq.github.io/CyberChef/#recipe=Fork('%5C%5Cn','%5C%5Cn',false)Conditional_Jump('1',false,'base64',10)To_Hex('Space')Return()Label('base64')To_Base64('A-Za-z0-9%2B/%3D')&input=U29tZSBkYXRhIHdpdGggYSAxIGluIGl0ClNvbWUgZGF0YSB3aXRoIGEgMiBpbiBpdA
  [9]: https://gchq.github.io/CyberChef/#recipe=Register('key%3D(%5B%5C%5Cda-f%5D*)',true,false)Find_/_Replace(%7B'option':'Regex','string':'.*data%3D(.*)'%7D,'$1',true,false,true)RC4(%7B'option':'Hex','string':'$R0'%7D,'Hex','Latin1')&input=aHR0cDovL21hbHdhcmV6LmJpei9iZWFjb24ucGhwP2tleT0wZTkzMmE1YyZkYXRhPThkYjdkNWViZTM4NjYzYTU0ZWNiYjMzNGUzZGIxMQ
  [10]: https://gchq.github.io/CyberChef/#recipe=Register('(.%7B32%7D)',true,false,false)Drop_bytes(0,32,false)AES_Decrypt(%7B'option':'Hex','string':'1748e7179bd56570d51fa4ba287cc3e5'%7D,%7B'option':'Hex','string':'$R0'%7D,16,'CTR','Hex','Raw',%7B'option':'Hex','string':''%7D,%7B'option':'Hex','string':''%7D,'Off')&input=NTFlMjAxZDQ2MzY5OGVmNWY3MTdmNzFmNWI0NzEyYWYyMGJlNjc0YjNiZmY1M2QzODU0NjM5NmVlNjFkYWFjNDkwOGUzMTljYTNmY2Y3MDg5YmZiNmIzOGVhOTllNzgxZDI2ZTU3N2JhOWRkNmYzMTFhMzk0MjBiODk3OGU5MzAxNGIwNDJkNDQ3MjZjYWVkZjU0MzZlYWY2NTI0MjljMGRmOTRiNTIxNjc2YzdjMmNlODEyMDk3YzI3NzI3M2M3YzcyY2Q4OWFlYzhkOWZiNGEyNzU4NmNjZjZhYTBhZWUyMjRjMzRiYTNiZmRmN2FlYjFkZGQ0Nzc2MjJiOTFlNzJjOWU3MDlhYjYwZjhkYWY3MzFlYzBjYzg1Y2UwZjc0NmZmMTU1NGE1YTNlYzI5MWNhNDBmOWU2MjlhODcyNTkyZDk4OGZkZDgzNDUzNGFiYTc5YzFhZDE2NzY3NjlhN2MwMTBiZjA0NzM5ZWNkYjY1ZDk1MzAyMzcxZDYyOWQ5ZTM3ZTdiNGEzNjFkYTQ2OGYxZWQ1MzU4OTIyZDJlYTc1MmRkMTFjMzY2ZjMwMTdiMTRhYTAxMWQyYWYwM2M0NGY5NTU3OTA5OGExNWUzY2Y5YjQ0ODZmOGZmZTljMjM5ZjM0ZGU3MTUxZjZjYTY1MDBmZTRiODUwYzNmMWMwMmU4MDFjYWYzYTI0NDY0NjE0ZTQyODAxNjE1YjhmZmFhMDdhYzgyNTE0OTNmZmRhN2RlNWRkZjMzNjg4ODBjMmI5NWIwMzBmNDFmOGYxNTA2NmFkZDA3MWE2NmNmNjBlNWY0NmYzYTIzMGQzOTdiNjUyOTYzYTIxYTUzZg
  [11]: https://gchq.github.io/CyberChef/#recipe=XOR(%7B'option':'Hex','string':'3a'%7D,'Standard',false)To_Hexdump(16,false,false)&input=VGhlIGFuc3dlciB0byB0aGUgdWx0aW1hdGUgcXVlc3Rpb24gb2YgbGlmZSwgdGhlIFVuaXZlcnNlLCBhbmQgZXZlcnl0aGluZyBpcyA0Mi4
  [12]: https://gchq.github.io/CyberChef/#recipe=Magic(3,false,false)&input=V1VhZ3dzaWFlNm1QOGdOdENDTFVGcENwQ0IyNlJtQkRvREQ4UGFjZEFtekF6QlZqa0syUXN0RlhhS2hwQzZpVVM3UkhxWHJKdEZpc29SU2dvSjR3aGptMWFybTg2NHFhTnE0UmNmVW1MSHJjc0FhWmM1VFhDWWlmTmRnUzgzZ0RlZWpHWDQ2Z2FpTXl1QlY2RXNrSHQxc2NnSjg4eDJ0TlNvdFFEd2JHWTFtbUNvYjJBUkdGdkNLWU5xaU45aXBNcTFaVTFtZ2tkYk51R2NiNzZhUnRZV2hDR1VjOGc5M1VKdWRoYjhodHNoZVpud1RwZ3FoeDgzU1ZKU1pYTVhVakpUMnptcEM3dVhXdHVtcW9rYmRTaTg4WXRrV0RBYzFUb291aDJvSDRENGRkbU5LSldVRHBNd21uZ1VtSzE0eHdtb21jY1BRRTloTTE3MkFQblNxd3hkS1ExNzJSa2NBc3lzbm1qNWdHdFJtVk5OaDJzMzU5d3I2bVMyUVJQ
  [13]: https://gchq.github.io/CyberChef/#recipe=AES_Decrypt(%7B'option':'Hex','string':'1748e7179bd56570d51fa4ba287cc3e5'%7D,%7B'option':'Hex','string':'$R0'%7D,16,'CTR','Hex','Raw',%7B'option':'Hex','string':''%7D,%7B'option':'Hex','string':''%7D,'From%20start')&input=NTFlMjAxZDQ2MzY5OGVmNWY3MTdmNzFmNWI0NzEyYWYyMGJlNjc0YjNiZmY1M2QzODU0NjM5NmVlNjFkYWFjNDkwOGUzMTljYTNmY2Y3MDg5YmZiNmIzOGVhOTllNzgxZDI2ZTU3N2JhOWRkNmYzMTFhMzk0MjBiODk3OGU5MzAxNGIwNDJkNDQ3MjZjYWVkZjU0MzZlYWY2NTI0MjljMGRmOTRiNTIxNjc2YzdjMmNlODEyMDk3YzI3NzI3M2M3YzcyY2Q4OWFlYzhkOWZiNGEyNzU4NmNjZjZhYTBhZWUyMjRjMzRiYTNiZmRmN2FlYjFkZGQ0Nzc2MjJiOTFlNzJjOWU3MDlhYjYwZjhkYWY3MzFlYzBjYzg1Y2UwZjc0NmZmMTU1NGE1YTNlYzI5MWNhNDBmOWU2MjlhODcyNTkyZDk4OGZkZDgzNDUzNGFiYTc5YzFhZDE2NzY3NjlhN2MwMTBiZjA0NzM5ZWNkYjY1ZDk1MzAyMzcxZDYyOWQ5ZTM3ZTdiNGEzNjFkYTQ2OGYxZWQ1MzU4OTIyZDJlYTc1MmRkMTFjMzY2ZjMwMTdiMTRhYTAxMWQyYWYwM2M0NGY5NTU3OTA5OGExNWUzY2Y5YjQ0ODZmOGZmZTljMjM5ZjM0ZGU3MTUxZjZjYTY1MDBmZTRiODUwYzNmMWMwMmU4MDFjYWYzYTI0NDY0NjE0ZTQyODAxNjE1YjhmZmFhMDdhYzgyNTE0OTNmZmRhN2RlNWRkZjMzNjg4ODBjMmI5NWIwMzBmNDFmOGYxNTA2NmFkZDA3MWE2NmNmNjBlNWY0NmYzYTIzMGQzOTdiNjUyOTYzYTIxYTUzZg
