# mitmproxy

[![Continuous Integration Status](https://github.com/mitmproxy/mitmproxy/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/mitmproxy/mitmproxy/actions?query=branch%3Amain)
[![autofix.ci: enabled](https://shields.mitmproxy.org/badge/autofix.ci-yes-success?logo=data:image/svg+xml;base64,PHN2ZyBmaWxsPSIjZmZmIiB2aWV3Qm94PSIwIDAgMTI4IDEyOCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCB0cmFuc2Zvcm09InNjYWxlKDAuMDYxLC0wLjA2MSkgdHJhbnNsYXRlKC0yNTAsLTE3NTApIiBkPSJNMTMyNSAtMzQwcS0xMTUgMCAtMTY0LjUgMzIuNXQtNDkuNSAxMTQuNXEwIDMyIDUgNzAuNXQxMC41IDcyLjV0NS41IDU0djIyMHEtMzQgLTkgLTY5LjUgLTE0dC03MS41IC01cS0xMzYgMCAtMjUxLjUgNjJ0LTE5MSAxNjl0LTkyLjUgMjQxcS05MCAxMjAgLTkwIDI2NnEwIDEwOCA0OC41IDIwMC41dDEzMiAxNTUuNXQxODguNSA4MXExNSA5OSAxMDAuNSAxODAuNXQyMTcgMTMwLjV0MjgyLjUgNDlxMTM2IDAgMjU2LjUgLTQ2IHQyMDkgLTEyNy41dDEyOC41IC0xODkuNXExNDkgLTgyIDIyNyAtMjEzLjV0NzggLTI5OS41cTAgLTEzNiAtNTggLTI0NnQtMTY1LjUgLTE4NC41dC0yNTYuNSAtMTAzLjVsLTI0MyAtMzAwdi01MnEwIC0yNyAzLjUgLTU2LjV0Ni41IC01Ny41dDMgLTUycTAgLTg1IC00MS41IC0xMTguNXQtMTU3LjUgLTMzLjV6TTEzMjUgLTI2MHE3NyAwIDk4IDE0LjV0MjEgNTcuNXEwIDI5IC0zIDY4dC02LjUgNzN0LTMuNSA0OHY2NGwyMDcgMjQ5IHEtMzEgMCAtNjAgNS41dC01NCAxMi41bC0xMDQgLTEyM3EtMSAzNCAtMiA2My41dC0xIDU0LjVxMCA2OSA5IDEyM2wzMSAyMDBsLTExNSAtMjhsLTQ2IC0yNzFsLTIwNSAyMjZxLTE5IC0xNSAtNDMgLTI4LjV0LTU1IC0yNi41bDIxOSAtMjQydi0yNzZxMCAtMjAgLTUuNSAtNjB0LTEwLjUgLTc5dC01IC01OHEwIC00MCAzMCAtNTMuNXQxMDQgLTEzLjV6TTEyNjIgNjE2cS0xMTkgMCAtMjI5LjUgMzQuNXQtMTkzLjUgOTYuNWw0OCA2NCBxNzMgLTU1IDE3MC41IC04NXQyMDQuNSAtMzBxMTM3IDAgMjQ5IDQ1LjV0MTc5IDEyMXQ2NyAxNjUuNWg4MHEwIC0xMTQgLTc3LjUgLTIwNy41dC0yMDggLTE0OXQtMjg5LjUgLTU1LjV6TTgwMyA1OTVxODAgMCAxNDkgMjkuNXQxMDggNzIuNWwyMjEgLTY3bDMwOSA4NnE0NyAtMzIgMTA0LjUgLTUwdDExNy41IC0xOHE5MSAwIDE2NSAzOHQxMTguNSAxMDMuNXQ0NC41IDE0Ni41cTAgNzYgLTM0LjUgMTQ5dC05NS41IDEzNHQtMTQzIDk5IHEtMzcgMTA3IC0xMTUuNSAxODMuNXQtMTg2IDExNy41dC0yMzAuNSA0MXEtMTAzIDAgLTE5Ny41IC0yNnQtMTY5IC03Mi41dC0xMTcuNSAtMTA4dC00MyAtMTMxLjVxMCAtMzQgMTQuNSAtNjIuNXQ0MC41IC01MC41bC01NSAtNTlxLTM0IDI5IC01NCA2NS41dC0yNSA4MS41cS04MSAtMTggLTE0NSAtNzB0LTEwMSAtMTI1LjV0LTM3IC0xNTguNXEwIC0xMDIgNDguNSAtMTgwLjV0MTI5LjUgLTEyM3QxNzkgLTQ0LjV6Ii8+PC9zdmc+)](https://autofix.ci)
[![Coverage Status](https://shields.mitmproxy.org/codecov/c/github/mitmproxy/mitmproxy/main.svg?label=codecov)](https://codecov.io/gh/mitmproxy/mitmproxy)
[![Latest Version](https://shields.mitmproxy.org/pypi/v/mitmproxy.svg)](https://pypi.python.org/pypi/mitmproxy)
[![Supported Python versions](https://shields.mitmproxy.org/pypi/pyversions/mitmproxy.svg)](https://pypi.python.org/pypi/mitmproxy)

``mitmproxy`` 是一个交互式拦截代理，支持 SSL/TLS，提供用于 HTTP/1、HTTP/2 和 WebSockets 的控制台界面。

``mitmdump`` 是 mitmproxy 的命令行版本，可以把它看作面向 HTTP 的 tcpdump。

``mitmweb`` 是 mitmproxy 的网页界面。

## 安装

安装说明请见[这里](https://docs.mitmproxy.org/stable/overview-installation).
如果想从源代码安装，请参阅 [CONTRIBUTING.md](https://github.com/mitmproxy/mitmproxy/blob/b08148b35d7679bc9be8d4a3d85b32bcea5ce250/CONTRIBUTING.md).

## 文档与帮助

mitmproxy 网站提供一般信息、教程和预编译二进制文件。

[![mitmproxy.org](https://shields.mitmproxy.org/badge/https%3A%2F%2F-mitmproxy.org-blue.svg)](https://mitmproxy.org/)

mitmproxy 的文档可在我们的网站上找到：

[![mitmproxy documentation stable](https://shields.mitmproxy.org/badge/docs-stable-brightgreen.svg)](https://docs.mitmproxy.org/stable/)
[![mitmproxy documentation dev](https://shields.mitmproxy.org/badge/docs-dev-brightgreen.svg)](https://docs.mitmproxy.org/dev/)

如果对 mitmproxy 的使用有疑问，请使用 GitHub Discussions！

[![mitmproxy discussions](https://shields.mitmproxy.org/badge/help-github%20discussions-orange.svg)](https://github.com/mitmproxy/mitmproxy/discussions)

## 贡献

作为开源项目，mitmproxy 欢迎各种形式的贡献。

[![Dev Guide](https://shields.mitmproxy.org/badge/dev_docs-CONTRIBUTING.md-blue)](https://github.com/mitmproxy/mitmproxy/blob/b08148b35d7679bc9be8d4a3d85b32bcea5ce250/CONTRIBUTING.md)
