> 根据 [Sunshine 固定版本 README](https://github.com/LizardByte/Sunshine/blob/c7e585c652dad552b96662f0c315550a62c2b4a6/README.md) 翻译。功能兼容性和硬件要求为上游正在完善的资料，上游明确提醒不要据此购买硬件。

<div align="center">
  <img
    src="https://raw.githubusercontent.com/LizardByte/Sunshine/c7e585c652dad552b96662f0c315550a62c2b4a6/sunshine.svg"
    alt="Sunshine icon"
    width="256"
/>
  <h1 align="center">Sunshine</h1>
  <h4 align="center">为 Moonlight 自托管的游戏串流主机。</h4>
</div>

<div align="center">
  <!-- Keep whitespace outside the badge links to avoid an underlined gap. -->
  <!-- Use raw HTML aliases to keep Doxygen's emitted comment delimiters balanced. -->
  <!--! @htmlonly_start -->
  <a href="https://www.star-history.com/lizardbyte/sunshine"><picture><source
    media="(prefers-color-scheme: dark)"
    srcset="https://api.star-history.com/badge?repo=LizardByte/Sunshine&type=rank&theme=dark"
  /><source
    media="(prefers-color-scheme: light)"
    srcset="https://api.star-history.com/badge?repo=LizardByte/Sunshine&type=rank"
  /><img
    alt="Star History Rank"
    src="https://api.star-history.com/badge?repo=LizardByte/Sunshine&type=rank"
  /></picture></a>
  <a href="https://www.star-history.com/lizardbyte/sunshine"><picture><source
    media="(prefers-color-scheme: dark)"
    srcset="https://api.star-history.com/badge?repo=LizardByte/Sunshine&type=trending&theme=dark"
  /><source
    media="(prefers-color-scheme: light)"
    srcset="https://api.star-history.com/badge?repo=LizardByte/Sunshine&type=trending"
  /><img
    alt="GitHub Trending Repository of the Day"
    src="https://api.star-history.com/badge?repo=LizardByte/Sunshine&type=trending"
  /></picture></a><br><br>
  <!--! @htmlonly_end -->
  <a href="https://github.com/LizardByte/Sunshine"><img src="https://img.shields.io/github/stars/lizardbyte/sunshine.svg?logo=github&style=for-the-badge" alt="GitHub stars"></a>
  <a href="https://github.com/LizardByte/Sunshine/releases/latest"><img src="https://img.shields.io/github/downloads/lizardbyte/sunshine/total.svg?style=for-the-badge&logo=github" alt="GitHub Releases"></a>
  <a href="https://hub.docker.com/r/lizardbyte/sunshine"><img src="https://img.shields.io/docker/pulls/lizardbyte/sunshine.svg?style=for-the-badge&logo=docker" alt="Docker"></a>
  <a href="https://github.com/LizardByte/Sunshine/pkgs/container/sunshine"><img src="https://img.shields.io/badge/dynamic/json.svg?url=https%3A%2F%2Fipitio.github.io%2Fbackage%2FLizardByte%2FSunshine%2Fsunshine.json&query=%24.downloads&label=ghcr%20pulls&style=for-the-badge&logo=github" alt="GHCR"></a>
  <a href="https://flathub.org/apps/dev.lizardbyte.app.Sunshine"><img src="https://img.shields.io/flathub/downloads/dev.lizardbyte.app.Sunshine.svg?style=for-the-badge&logo=flathub" alt="Flathub installs"></a>
  <a href="https://flathub.org/apps/dev.lizardbyte.app.Sunshine"><img src="https://img.shields.io/flathub/v/dev.lizardbyte.app.Sunshine.svg?style=for-the-badge&logo=flathub" alt="Flathub Version"></a>
  <a href="https://github.com/microsoft/winget-pkgs/tree/master/manifests/l/LizardByte/Sunshine"><img src="https://img.shields.io/winget/v/LizardByte.Sunshine.svg?style=for-the-badge&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAHuSURBVFhH7ZfNTtRQGIYZiMDwN/IrCAqIhMSNKxcmymVwG+5dcDVsWHgDrtxwCYQVl+BChzDEwSnPY+eQ0sxoOz1mQuBNnpyvTdvz9jun5/SrjfxnJUkyQbMEz2ELduF1l0YUA3QyTrMAa2AnPtyOXsELeAYNyKtV2EC3k3lYgTOwg09ghy/BTp7CKBRV844BOpmmMV2+ySb4BmInG7AKY7AHH+EYqqhZo9PPBG/BVDlOizAD/XQFmnoPXzxRQX8M/CCYS48L6RIc4ygGHK9WGg9HZSZMUNRPVwNJGg5Hg2Qgqh4N3FsDsb6EmgYm07iwwvUxstdxJTwgmILf4CfZ6bb5OHANX8GN5x20IVxnG8ge94pt2xpwU3GnCwayF4Q2G2vgFLzHndFzQdk4q77nNfCdwL28qNyMtmEf3A1/QV5FjDiPWo5jrwf8TWZChTlgJvL4F9QL50/A43qVidTvLcuoM2wDQ1+IkgefgUpLcYwMVBqCKNJA2b0gKNocOIITOIef8C/F/CdMbh/GklynsSawKLHS8d9/B1x2LUqsfFyy3TMsWj5A1cLkotDbYO4JjWWZlZEGv8EbOIR1CAVN2eG8W5oNKgxaeC6DmTJjZs7ixUxpznLPLT+v4sXpoMLcLI3mzFSonDXIEI/M3QCIO4YuimBJ/gAAAABJRU5ErkJggg==" alt="Winget Version"></a>
  <a href="https://github.com/LizardByte/Sunshine/actions/workflows/ci.yml?query=branch%3Amaster"><img src="https://img.shields.io/github/actions/workflow/status/lizardbyte/sunshine/ci.yml.svg?branch=master&label=CI%20build&logo=github&style=for-the-badge" alt="GitHub Workflow Status (CI)"></a>
  <a href="https://github.com/LizardByte/Sunshine/actions/workflows/localize.yml?query=branch%3Amaster"><img src="https://img.shields.io/github/actions/workflow/status/lizardbyte/sunshine/localize.yml.svg?branch=master&label=localize%20build&logo=github&style=for-the-badge" alt="GitHub Workflow Status (localize)"></a>
  <a href="https://codecov.io/gh/LizardByte/Sunshine"><img src="https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fapp.lizardbyte.dev%2Fdashboard%2Fshields%2Fcodecov%2FSunshine.json&style=for-the-badge&logo=codecov" alt="Codecov"></a>
  <a href="https://sonarcloud.io/project/overview?id=LizardByte_Sunshine"><img src="https://img.shields.io/sonar/quality_gate/LizardByte_Sunshine.svg?server=https%3A%2F%2Fsonarcloud.io&style=for-the-badge&logo=sonarqubecloud&label=sonarcloud" alt="SonarCloud"></a>
</div>

## ℹ️ 关于

Sunshine 是为 Moonlight 提供的自托管游戏串流主机，提供低延迟云游戏服务器能力，支持使用 AMD、Intel 和 Nvidia GPU 进行硬件编码，也支持软件编码。你可以从各种设备上的任何 Moonlight 客户端连接 Sunshine。项目提供 Web 界面，可在常用浏览器中配置及配对客户端；可以从本地服务器或任何移动设备进行配对。

LizardByte 在 [Read the Docs](https://docs.lizardbyte.dev/projects/sunshine) 托管完整文档。

* [稳定版文档](https://docs.lizardbyte.dev/projects/sunshine/latest/)
* [Beta 版文档](https://docs.lizardbyte.dev/projects/sunshine/master/)

## 🎮 功能兼容性

<table>
    <caption id="gamepad_emulation">游戏手柄模拟</caption>
    <tr>
        <th>功能</th>
        <th>FreeBSD</th>
        <th>Linux</th>
        <th>macOS</th>
        <th>Windows</th>
    </tr>
    <tr>
        <td colspan="5" align="center">
        主机可以模拟哪些类型的游戏手柄。<br>
        客户端可能支持其他手柄。
        </td>
    </tr>
    <tr>
        <td>通用</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
    <tr>
        <td>DualShock / DS4 (PlayStation 4)</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
    <tr>
        <td>DualSense / DS5 (PlayStation 5)</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
    <tr>
        <td>Nintendo Switch Pro</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
    <tr>
        <td>Xbox 360</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
    <tr>
        <td>Xbox One</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
    <tr>
        <td>Xbox Series</td>
        <td>🟡<sup>1</sup></td>
        <td>✅</td>
        <td>✅<sup>2</sup></td>
        <td>✅<sup>3</sup></td>
    </tr>
</table>

> [!NOTE]
> <sup>1</sup> 缺少动作感应、触摸板输入、电池状态、RGB LED、自适应扳机和原始 HID 输出报告。
> <sup>2</sup> 需要单独安装并获得许可的 Virtual HID Broker。
> <sup>3</sup> 所有配置均可通过单独安装并获得许可的 Virtual HID Broker 使用。Xbox 360 和 DualShock 4 也可使用 ViGEmBus。

<table>
    <caption id="encoding_api">编码 API</caption>
    <tr>
        <th>编码 API</th>
        <th>GPU 厂商</th>
        <th>FreeBSD</th>
        <th>Linux</th>
        <th>macOS</th>
        <th>Windows</th>
    </tr>
    <tr>
        <td>AMF</td>
        <td>AMD</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>Media Foundation</td>
        <td>Qualcomm</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>NVENC</td>
        <td>NVIDIA</td>
        <td>➖</td>
        <td>✅</td>
        <td>➖</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>QuickSync</td>
        <td>Intel</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
    </tr>
    <tr>
        <td rowspan="3">VAAPI</td>
        <td>AMD</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>Intel</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>NVIDIA</td>
        <td>➖</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td rowspan="2">Video Toolbox</td>
        <td>Apple</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>Intel</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
        <td>➖</td>
    </tr>
    <tr>
        <td rowspan="3">Vulkan Video</td>
        <td>AMD</td>
        <td>🟡</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>Intel</td>
        <td>🟡</td>
        <td>🟡</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>NVIDIA</td>
        <td>➖</td>
        <td>🟡</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>软件</td>
        <td>任意</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
    </tr>
</table>

<table>
    <caption id="screen_capture">屏幕捕获</caption>
    <tr>
        <th>捕获方式</th>
        <th>FreeBSD</th>
        <th>Linux</th>
        <th>macOS</th>
        <th>Windows</th>
    </tr>
    <tr>
        <td>DXGI Desktop Duplication</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>KMS/DRM</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>NvFBC（仅 X11）</td>
        <td>➖</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>ScreenCaptureKit</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>Wayland (wlroots)</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>Windows.Graphics.Capture</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>🟡</td>
    </tr>
    <tr>
        <td>&nbsp;&nbsp;↳ 便携版</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>&nbsp;&nbsp;↳ 服务</td>
        <td>➖</td>
        <td>➖</td>
        <td>➖</td>
        <td>❌</td>
    </tr>
    <tr>
        <td>X11</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>XDG Desktop Portal</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
    <tr>
        <td>KWin Screencast</td>
        <td>✅</td>
        <td>✅</td>
        <td>➖</td>
        <td>➖</td>
    </tr>
</table>

<table>
    <caption id="capture_encoding_compat">捕获 → 编码兼容性（Linux/FreeBSD）</caption>
    <tr>
        <th>捕获方式</th>
        <th>VAAPI</th>
        <th>Vulkan Video</th>
        <th>NVENC (CUDA)</th>
        <th>软件</th>
    </tr>
    <tr>
        <td>KMS/DRM</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>NvFBC</td>
        <td>❌</td>
        <td>❌</td>
        <td>✅</td>
        <td>❌</td>
    </tr>
    <tr>
        <td>Wayland (wlroots)</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>X11</td>
        <td>✅</td>
        <td>❌</td>
        <td>✅</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>XDG Desktop Portal</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>KWin Screencast</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
        <td>✅</td>
    </tr>
</table>

**图例：**✅ 支持 | 🟡 部分支持 | ❌ 尚不支持 | ➖ 不适用

## 🖥️ 系统要求

> [!WARNING]
> 这些表格尚在完善中。不要依据此信息购买硬件。

<table>
    <caption id="minimum_requirements">最低要求</caption>
    <tr>
        <th>组件</th>
        <th>要求</th>
    </tr>
    <tr>
        <td rowspan="3">GPU</td>
        <td>AMD：VCE 1.0 或更高版本，参见： <a href="https://github.com/obsproject/obs-amd-encoder/wiki/Hardware-Support">obs-amd 硬件支持</a></td>
    </tr>
    <tr>
        <td>
            Intel:<br>
            &nbsp;&nbsp;FreeBSD/Linux：兼容 VAAPI，参见： <a href="https://www.intel.com/content/www/us/en/developer/articles/technical/linuxmedia-vaapi.html">VAAPI 硬件支持</a><br>
            &nbsp;&nbsp;Windows：Skylake 或更新处理器，支持 QuickSync 编码
        </td>
    </tr>
    <tr>
        <td>Nvidia：支持 NVENC 的显卡，参见： <a href="https://developer.nvidia.com/video-encode-and-decode-gpu-support-matrix-new">NVENC 支持矩阵</a></td>
    </tr>
    <tr>
        <td rowspan="2">CPU</td>
        <td>AMD：Ryzen 3 或更高</td>
    </tr>
    <tr>
        <td>Intel：Core i3 或更高</td>
    </tr>
    <tr>
        <td>RAM</td>
        <td>4GB 或更多</td>
    </tr>
    <tr>
        <td rowspan="6">OS</td>
        <td>FreeBSD: 15.1+</td>
    </tr>
    <tr>
        <td>Linux/Debian: 13+ (trixie)</td>
    </tr>
    <tr>
        <td>Linux/Fedora: 43+</td>
    </tr>
    <tr>
        <td>Linux/Ubuntu: 22.04+ (jammy)</td>
    </tr>
    <tr>
        <td>macOS: 15.0+</td>
    </tr>
    <tr>
        <td>Windows: 11+</td>
    </tr>
    <tr>
        <td rowspan="2">网络</td>
        <td>主机：5GHz，802.11ac</td>
    </tr>
    <tr>
        <td>客户端：5GHz，802.11ac</td>
    </tr>
</table>

<table>
    <caption id="4k_suggestions">4K 建议配置</caption>
    <tr>
        <th>组件</th>
        <th>要求</th>
    </tr>
    <tr>
        <td rowspan="3">GPU</td>
        <td>AMD：Video Coding Engine 3.1 或更高</td>
    </tr>
    <tr>
        <td>
            Intel:<br>
            &nbsp;&nbsp;FreeBSD/Linux：HD Graphics 510 或更高<br>
            &nbsp;&nbsp;Windows：Skylake 或更新处理器，支持 QuickSync 编码
        </td>
    </tr>
    <tr>
        <td>
            Nvidia:<br>
            &nbsp;&nbsp;FreeBSD/Linux：GeForce RTX 2000 系列或更高<br>
            &nbsp;&nbsp;Windows：Geforce GTX 1080 或更高
        </td>
    </tr>
    <tr>
        <td rowspan="2">CPU</td>
        <td>AMD：Ryzen 5 或更高</td>
    </tr>
    <tr>
        <td>Intel：Core i5 或更高</td>
    </tr>
    <tr>
        <td rowspan="2">网络</td>
        <td>主机：CAT5e 以太网或更好</td>
    </tr>
    <tr>
        <td>客户端：CAT5e 以太网或更好</td>
    </tr>
</table>

<table>
    <caption id="hdr_suggestions">HDR 建议配置</caption>
    <tr>
        <th>组件</th>
        <th>要求</th>
    </tr>
    <tr>
        <td rowspan="3">GPU</td>
        <td>AMD：Video Coding Engine 3.4 或更高</td>
    </tr>
    <tr>
        <td>Intel：HD Graphics 730 或更高</td>
    </tr>
    <tr>
        <td>Nvidia：基于 Pascal 的 GPU（GTX 10 系列）或更高</td>
    </tr>
    <tr>
        <td rowspan="2">CPU</td>
        <td>AMD：Ryzen 5 或更高</td>
    </tr>
    <tr>
        <td>Intel：Core i5 或更高</td>
    </tr>
    <tr>
        <td rowspan="2">网络</td>
        <td>主机：CAT5e 以太网或更好</td>
    </tr>
    <tr>
        <td>客户端：CAT5e 以太网或更好</td>
    </tr>
</table>

## ❓ 支持

支持方式列于 [LizardByte 文档](https://docs.lizardbyte.dev/latest/about/support.html)。

## 💲 赞助者和支持者

<p align="center">
  <img src='https://cdn.jsdelivr.net/gh/LizardByte/contributors@dist/sponsors.svg' alt="Sponsors"/>
</p>

## 👥 贡献者

感谢所有帮助 Sunshine 变得更好的贡献者！

### GitHub

<p align="center">
  <img src='https://cdn.jsdelivr.net/gh/LizardByte/contributors@dist/github.Sunshine.svg' alt="GitHub contributors"/>
</p>

### CrowdIn

<p align="center">
  <img src='https://cdn.jsdelivr.net/gh/LizardByte/contributors@dist/crowdin.606145.svg' alt="CrowdIn contributors"/>
</p>
