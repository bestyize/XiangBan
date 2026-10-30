# 社区与小伴AI 手机原型

状态：设计参考。卡片、作者、数字均为示例内容，`N` 表示待填的数；不是 App 运行入口，不参与 Android / iOS 构建。与实现不一致时以代码和[项目说明](../../../README.md)为准。

- 在线画布：<https://claude.ai/artifact/BGtDJ7S7vGenqaY5MvyLLJ>（Claude Design 画布，可评论、可在画布里直接改）
- 内容：小伴手机 App 的社区（首页、周刊、资源、讨论、帖子、搜索、发布）、小伴 AI（首页、对话与创作、模型与额度、资料库）、游戏、我的与设置、分享、我的设备与遥控电脑、聊天里的任务 / 选择 / 权限卡片、实时语音对话和按住说话。
- 画板尺寸：手机竖屏 390 宽；标为“可交互”的画板里，点击卡片和按钮可以在画板之间跳转，输入框内容是写好的示例。
- 聊天状态卡片参考了[小伴桌面 UI 升级设计稿](../../../../xiaoban-desktop/docs/design-specs/ui-upgrade/README.md)。

## 本地预览

```bash
cd client/xiaoban-multiplatform/docs/design-specs/mobile-prototype
python3 -m http.server 8000
```

浏览器打开 `http://localhost:8000/` 查看全部画板（按画布分组，可调缩放），或打开单个 `*.dc.html` 查看全尺寸画板。画板之间的跳转链接需要通过本地服务打开才能生效。

## 目录说明

| 文件 | 说明 |
| --- | --- |
| `index.html` | 本地总览页，读取 `canvas.json` 按分组渲染所有画板（仓库自加，不在画布里） |
| `canvas.json` | 画布索引：每个画板的标题、位置、尺寸，以及画布上的分组标题和便签 |
| `*.dc.html` | 每个画板的完整源码，一个文件一个画板 |
| `support.js` | 画板渲染运行时，从画布导出，勿手改 |

## 画布上的说明便签

**配色**：页面底浅灰，卡片和弹层纯白，不用浅灰描边，层次靠底灰卡白，阴影只给浮层；红色只用在主按钮、选中态和提醒上。

**多设备**：项目 = 设备上的一个文件夹（待确认）。每台设备的项目和对话各自独立，可以同时有对话在执行。我的设备与遥控基于同一账号登录，无需配对。

## 与在线画布同步

本目录是画布 `project/` 下文件的副本，`*.dc.html` 与 `canvas.json` 保持原样未改。

- **画布 → 仓库**：在画布里改完后，让 AI 按上面的链接重新拉取 `project/` 下的文件，覆盖本目录后提交。
- **仓库 → 画布**：改了本目录的 `*.dc.html` 后，让 AI 把改动的文件发布回同一个画布链接。
- 两边都改过时先拉取画布最新版再合并，避免覆盖别人在画布里的手动修改。

## 画板清单

### 社区与小伴AI：手机原型

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 1 社区首页 | [Main.dc.html](Main.dc.html) | 390×1560 · 可交互 |
| 2 社区首页·周刊 | [MainWeekly.dc.html](MainWeekly.dc.html) | 390×1180 · 可交互 |
| 3 装扮详情 | [PackDetail.dc.html](PackDetail.dc.html) | 390×1080 · 可交互 |
| 4 能力详情 | [Ability.dc.html](Ability.dc.html) | 390×1180 · 可交互 |
| 5 小伴AI 首页 | [AIHome.dc.html](AIHome.dc.html) | 390×1040 · 可交互 |
| 6 小伴AI 对话页（含创作） | [Studio.dc.html](Studio.dc.html) | 390×844 · 可交互 |

### 社区子页与发布

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 7 社区·资源 | [Resources.dc.html](Resources.dc.html) | 390×1000 · 可交互 |
| 8 社区·讨论 | [Discuss.dc.html](Discuss.dc.html) | 390×1000 · 可交互 |
| 9 帖子详情（求助） | [Post.dc.html](Post.dc.html) | 390×1120 · 可交互 |
| 10 搜索 | [Search.dc.html](Search.dc.html) | 390×844 · 可交互 |
| 11 桌面方案详情 | [Scheme.dc.html](Scheme.dc.html) | 390×1180 · 可交互 |

### 游戏、我的与设置

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 12 游戏 | [Games.dc.html](Games.dc.html) | 390×1180 · 可交互 |
| 13 游戏试玩 | [GamePlay.dc.html](GamePlay.dc.html) | 390×844 · 可交互 |
| 14 我的 | [Me.dc.html](Me.dc.html) | 390×1000 · 可交互 |
| 15 我的作品 | [MyWorks.dc.html](MyWorks.dc.html) | 390×900 · 可交互 |
| 16 设置 | [Settings.dc.html](Settings.dc.html) | 390×960 · 可交互 |
| 17 发布作品 | [PublishPack.dc.html](PublishPack.dc.html) | 390×1040 · 可交互 |
| 18 发帖 | [NewPost.dc.html](NewPost.dc.html) | 390×960 · 可交互 |
| 66 设置·主题（跟随系统 / 浅色 / 深色） | [SettingsTheme.dc.html](SettingsTheme.dc.html) | 390×480 · 可交互 |
| 67 设置·主题色（六套） | [SettingsAccent.dc.html](SettingsAccent.dc.html) | 390×628 · 可交互 |
| 68 设置·深色模式示例 | [SettingsDark.dc.html](SettingsDark.dc.html) | 390×960 |
| 69 设置·声音（识别、朗读、通话） | [SettingsVoice.dc.html](SettingsVoice.dc.html) | 390×760 · 可交互 |
| 70 设置·声音·各状态 | [SettingsVoiceStates.dc.html](SettingsVoiceStates.dc.html) | 390×1620 |

### 模型与额度

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 19 模型与额度 | [Models.dc.html](Models.dc.html) | 390×860 · 可交互 |
| 20 小伴AI 配置弹窗 | [ModelsXb.dc.html](ModelsXb.dc.html) | 390×860 · 可交互 |
| 21 供应商配置弹窗 | [ModelsProvider.dc.html](ModelsProvider.dc.html) | 390×1000 · 可交互 |
| 22 添加自定义供应商弹窗 | [ModelsCustom.dc.html](ModelsCustom.dc.html) | 390×1000 · 可交互 |
| 23 对话页·选择模型 | [StudioModel.dc.html](StudioModel.dc.html) | 390×844 · 可交互 |
| 24 对话页·思考强度弹窗 | [StudioLevel.dc.html](StudioLevel.dc.html) | 390×844 · 可交互 |

### 资料库

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 25 资料库·我的文档 | [Library.dc.html](Library.dc.html) | 390×1000 · 可交互 |
| 26 资料库·收藏 | [LibrarySaved.dc.html](LibrarySaved.dc.html) | 390×1000 · 可交互 |
| 27 资料库·添加资料 | [LibraryAdd.dc.html](LibraryAdd.dc.html) | 390×1000 · 可交互 |
| 28 对话页·从资料库选择 | [StudioLib.dc.html](StudioLib.dc.html) | 390×844 · 可交互 |

### 分享

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 29 帖子详情·分享 | [PostShare.dc.html](PostShare.dc.html) | 390×1120 · 可交互 |
| 30 装扮详情·分享 | [PackShare.dc.html](PackShare.dc.html) | 390×1080 · 可交互 |
| 31 分享海报 | [Poster.dc.html](Poster.dc.html) | 390×844 · 可交互 |
| 32 分享落地页（微信里打开） | [ShareLanding.dc.html](ShareLanding.dc.html) | 390×844 · 可交互 |

### 我的设备与遥控：同一账号登录，无需配对

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 33 我的设备 | [Devices.dc.html](Devices.dc.html) | 390×844 · 可交互 |
| 34 我的设备·还没有设备 | [DevicesEmpty.dc.html](DevicesEmpty.dc.html) | 390×844 · 可交互 |
| 35 设备·项目与对话侧栏 | [RemoteList.dc.html](RemoteList.dc.html) | 390×844 · 可交互 |
| 36 设备·聊天页（侧栏收起） | [RemoteChat.dc.html](RemoteChat.dc.html) | 390×844 · 可交互 |
| 37 遥控·审批卡片 | [RemoteApprove.dc.html](RemoteApprove.dc.html) | 390×844 · 可交互 |

### 遥控的审批与异常，以及登录

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 38 遥控·高危审批（只确认一次） | [RemoteApproveHigh.dc.html](RemoteApproveHigh.dc.html) | 390×844 · 可交互 |
| 39 遥控·电脑离线 | [RemoteOffline.dc.html](RemoteOffline.dc.html) | 390×844 · 可交互 |
| 40 遥控·异常状态 | [RemoteStates.dc.html](RemoteStates.dc.html) | 390×844 · 可交互 |
| 41 登录（弹窗） | [Login.dc.html](Login.dc.html) | 390×844 · 可交互 |
| 42 完成注册（弹窗） | [Register.dc.html](Register.dc.html) | 390×844 · 可交互 |

### 多设备：每台设备有自己的项目和对话

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 43 设备·切换设备 | [RemoteSwitch.dc.html](RemoteSwitch.dc.html) | 390×844 · 可交互 |
| 44 设备·新项目 | [RemoteProject.dc.html](RemoteProject.dc.html) | 390×844 · 可交互 |

### 聊天里的各种状态卡片（参考桌面版升级设计稿）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 45 聊天·任务执行卡（全部状态） | [ChatTask.dc.html](ChatTask.dc.html) | 390×1844 |
| 46 聊天·调整计划与迷你进度条 | [ChatAdjust.dc.html](ChatAdjust.dc.html) | 390×852 |
| 47 聊天·选择卡（含各状态） | [ChatAsk.dc.html](ChatAsk.dc.html) | 390×2098 |
| 48 聊天·权限申请卡与权限切换 | [ChatApproval.dc.html](ChatApproval.dc.html) | 390×1610 |
| 49 聊天·思考与工具调用 | [ChatThink.dc.html](ChatThink.dc.html) | 390×1065 |
| 50 聊天·回复完成态（文件、操作栏、提示条） | [ChatDone.dc.html](ChatDone.dc.html) | 390×1323 |
| 51 聊天·完整会话示例 | [ChatFull.dc.html](ChatFull.dc.html) | 390×844 |

### 实时对话（语音通话）设计稿

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 52 实时对话·入口与首次下载 | [VoiceEntry.dc.html](VoiceEntry.dc.html) | 390×1327 |
| 53 实时对话·聆听 | [VoiceListen.dc.html](VoiceListen.dc.html) | 390×844 |
| 54 实时对话·听到你说话 | [VoiceHear.dc.html](VoiceHear.dc.html) | 390×844 |
| 55 实时对话·思考 | [VoiceThink.dc.html](VoiceThink.dc.html) | 390×844 |
| 56 实时对话·小伴在说 | [VoiceSpeak.dc.html](VoiceSpeak.dc.html) | 390×844 |
| 57 实时对话·提示与异常 | [VoiceNotes.dc.html](VoiceNotes.dc.html) | 390×1288 |
| 58 实时对话·收起成胶囊 | [VoiceMini.dc.html](VoiceMini.dc.html) | 390×411 |

### 语音输入：按住说话，松手发送（类似微信）

| 画板 | 文件 | 尺寸 |
| --- | --- | --- |
| 59 语音输入·入口 | [PttEntry.dc.html](PttEntry.dc.html) | 390×872 |
| 60 语音输入·按住录音 | [PttRecord.dc.html](PttRecord.dc.html) | 390×844 |
| 61 语音输入·上滑取消 | [PttCancel.dc.html](PttCancel.dc.html) | 390×844 |
| 62 语音输入·转文字 | [PttText.dc.html](PttText.dc.html) | 390×844 |
| 63 语音输入·松手之后 | [PttResult.dc.html](PttResult.dc.html) | 390×560 |
| 64 语音输入·提示与异常 | [PttNotes.dc.html](PttNotes.dc.html) | 390×792 |
| 65 输入框工具栏·文字按钮改图标 | [PttToolbar.dc.html](PttToolbar.dc.html) | 390×734 |
