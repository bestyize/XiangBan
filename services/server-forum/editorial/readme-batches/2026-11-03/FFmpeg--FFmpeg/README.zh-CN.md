# FFmpeg 自述文件

FFmpeg 是一组用于处理音频、视频、字幕及相关元数据等多媒体内容的库和工具。

## 库

* `libavcodec` 提供多种编解码器的实现。
* `libavformat` 实现流媒体协议、容器格式和基本 I/O 访问。
* `libavutil` 包含哈希工具、解压工具及其他实用函数。
* `libavfilter` 通过互相连接的滤镜有向图来处理解码后的音频和视频。
* `libavdevice` 提供访问采集和播放设备的抽象层。
* `libswresample` 实现音频混合与重采样功能。
* `libswscale` 实现颜色转换和缩放功能。

## 工具

* [ffmpeg](https://ffmpeg.org/ffmpeg.html)是一个用于处理、转换和流式传输多媒体内容的命令行工具箱。
* [ffplay](https://ffmpeg.org/ffplay.html)是一个简约的多媒体播放器。
* [ffprobe](https://ffmpeg.org/ffprobe.html)是一个用于检查多媒体内容的简单分析工具。
* 其他小工具，例如 `aviocat`、`ismindex` 和 `qt-faststart`。

## 文档

离线文档位于 **doc/** 目录。

在线文档可在主[网站](https://ffmpeg.org)及 [wiki](https://trac.ffmpeg.org) 查阅。

### 示例

代码示例位于 **doc/examples** 目录。

## 许可证

FFmpeg 代码库主要依据 LGPL 许可，部分可选组件依据 GPL 许可。详细信息请参阅 LICENSE 文件。

## 参与贡献

补丁应使用 `git format-patch` 或 `git send-email` 提交到 [Forgejo](https://code.ffmpeg.org/FFmpeg/FFmpeg/pulls) 或 [ffmpeg-devel 邮件列表](https://lists.ffmpeg.org/mailman3/lists/ffmpeg-devel.ffmpeg.org/)。应避免提交 GitHub Pull Request，因为它们不属于我们的审查流程，**将被忽略**。

更多信息请参阅 [https://ffmpeg.org/developer.html](https://ffmpeg.org/developer.html)。
