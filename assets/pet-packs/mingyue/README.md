# 潮汐书房 · 溟月

在小伴桌面进入「装扮 → 伙伴 → 全部」，在「潮汐书房」下点击溟月的添加按钮，选择「贴在桌面」或「始终置顶」。伙伴包也可用于手动导入和分享。已经摆好的溟月打开新版客户端即可继续使用。[伙伴 ZIP](../mingyue.zip)

新版小伴桌面内置《潮汐书房》的女仆版 GLB，在透明窗口里实时播放模型自带的 16 段动作。点击她会打招呼，按住拖动时会迈步。默认会跟随心情，在「思考」时捧书，闲置后坐下，再侧卧入睡。右键溟月，在「常规 → 原作动作」中可以选定写字、坐着读书、玩游戏等动作循环播放。全局暂停会定格当前动作。

伙伴 ZIP 提供八张透明姿态图，用于缩略图以及 WebGL 2 不可用时的显示。睁眼和闭眼使用同一姿势。桌面版沿用 RealWorldBuilder 绘制的眼睑和保护刘海的遮罩，按原动作的时机眨眼，睡眠时持续闭眼。桌面客户端首次展示 3D 形象时从资源机下载模型，校验缓存后可离线播放。下载期间或失败时显示平面姿态。许可文件随客户端提供。[书房眼睑源码](https://github.com/bestyize/RealWorldBuilder/blob/037ef10fbe6c368eb931c0c958b6283d25ef0fa2/games/whale-girl/eyelids.js) · [模型与许可](../../../client/xiaoban-desktop/public/pets/mingyue/)

![溟月站立、抱鲸与捧书的平面姿态](preview.webp)

## 素材与许可

角色原型「溟月」由上善无形创作，DeepSeek 女仆造型由 ZipZipPipe 设计，Q 版三维造型、服装和动作由 PekolandBBQ 制作。动作和部分造型参考了 merzlin 的 shenshen 桌宠和 PC2005-cloud 的 dsh-pet。围裙上的 DeepSeek 标志归相应权利人所有。[原作者与第三方署名](https://github.com/PekolandBBQ/Deepseek-Dafeiyu-Chibi-3D-Model/blob/v1.0.2/THIRD_PARTY_NOTICES.md)

八张姿态图取自 RealWorldBuilder《潮汐书房》使用的 v1.0.2 女仆模型。桌面版重新取景、布光，在拥抱姿态中加入了新绘制的小鲸鱼布偶，并渲染为透明图像。画面、表情与姿态改编沿用 CC BY-NC-SA 4.0，仅限非商业使用。每张 WebP 的 XMP 元数据保留了署名、来源和许可链接。[模型发布页](https://github.com/PekolandBBQ/Deepseek-Dafeiyu-Chibi-3D-Model/releases/tag/v1.0.2) · [书房源文件](https://github.com/bestyize/RealWorldBuilder/tree/037ef10fbe6c368eb931c0c958b6283d25ef0fa2/games/whale-girl) · [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/)

在本目录执行以下命令即可打包。

```sh
zip -X -q -FS ../mingyue.zip manifest.json pet.json assets/*.webp
```
