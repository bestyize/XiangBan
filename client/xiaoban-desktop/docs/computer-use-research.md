# ChatGPT Computer Use 与桌面操作产品调研

2026 年 11 月 23 日 · 小伴桌面产品与工程参考

后续实测已将小伴的引擎从调研基线 1.1.2 升至 1.1.5。被遮挡窗口的中文输入、同名窗口的定向操作及新旧版本耗时见[工具包实测记录](../../../libs/agent-tools/computer-use/README.md#115-实机验证)。

用户在编辑器里继续打中文，同时让助手在被遮挡的另一个应用里修改指定文档。这是小伴要追平的一项日常体验。用户能看出助手正在操作哪个窗口，必要时能叫停。助手完成后能指出改了什么，重新打开文档也能看到正确结果。这一任务同时检验窗口选择、输入隔离、状态提示、速度和结果验证。

OpenAI 已公开确认 Mac 后台操作、独立光标和画中画预览。Claude 还明确说明会给正在打字的用户让行，Gemini 则让用户在聊天里观看虚拟屏幕。小伴当前固定的开源引擎已具备后台输入和随窗口移动的软件光标，值得先把现有能力在真实任务里测清楚。本次依据公开产品资料、研究论文和小伴当前源码，速度数字均标明出处。文末提供同机对照方法，尚无本次调研产生的跨产品实测排名。[OpenAI 发布说明](https://openai.com/index/codex-for-almost-everything/) · [Claude 桌面说明](https://support.claude.com/en/articles/14128542-let-claude-use-your-computer-in-cowork) · [Google 发布说明](https://blog.google/innovation-and-ai/products/gemini-app/computer-use-mac/) · [小伴所用引擎版本说明](https://github.com/iFurySt/open-computer-use/releases/tag/v1.1.2)

## 一次 ChatGPT 桌面任务怎样运行

| 用户能感受到什么 | 公开资料能确认到哪一步 |
| --- | --- |
| 指定操作对象 | 在 Work 或 Codex 中可以提及 `@Computer` 或应用名，也可以描述具体窗口和操作流程。Appshot 可以把当前窗口的图像和可提取文字交给模型。实际操作应用还要经过授权。用窗口传达意图和授权应用是两个不同的步骤。[Computer Use 手册](https://learn.chatgpt.com/docs/computer-use) · [Appshots](https://learn.chatgpt.com/docs/appshots) |
| 第一次授权，以后少打断 | macOS 的屏幕录制和辅助功能是系统权限。ChatGPT 另有按应用的本次允许、始终允许、取消和事后撤销。组织管理员可以收紧范围，敏感或干扰性动作仍可能单独请求确认。[权限与管理](https://learn.chatgpt.com/docs/computer-use) |
| 我打我的字，它做它的事 | macOS 版可用自己的光标在后台看、点、输入，用户可继续使用其他应用。官方明确提醒，两个任务同时操作同一应用可能改变彼此的窗口状态。Windows 版在活动桌面前台运行，用户会看到系统指针和键盘被使用。[Mac 后台操作发布说明](https://openai.com/index/codex-for-almost-everything/) · [平台差异和并发限制](https://learn.chatgpt.com/use-cases/use-your-computer-with-codex) |
| 看见它在做什么 | 画中画预览显示正在使用的应用，可移开、打开跟看，也可移到 Pet 旁。窗口左上角电脑图标来自小伴用户的实际观察。另一位用户公开报告过蓝色电脑形象和数字徽标残留。两者均不能确定图标在所有版本中固定于左上角，也不能证明授权锁定到单一窗口。[官方预览说明](https://learn.chatgpt.com/use-cases/use-your-computer-with-codex) · [用户现场记录](https://github.com/openai/codex/issues/32363) · [OpenAI 视频演示](https://www.youtube.com/watch?v=D_FCYsshMI4) |
| 暂停、接管和离开电脑 | 用户可以停止任务或接管电脑。连接桌面主机后，手机可以发起或指导任务。Mac 锁屏后继续执行需要另行启用 Locked use，并有专门的本地输入检测和重新锁定机制。日常“窗口在后台”无需借助锁屏模式。[控制与锁屏机制](https://learn.chatgpt.com/docs/computer-use) · [远程任务示例](https://learn.chatgpt.com/use-cases/use-your-computer-with-codex) |
| 数据可见范围 | 目标应用中的屏幕内容、菜单、打开的文件和剪贴板状态可能成为任务上下文。浏览器扩展还有自己的域名授权规则，读入聊天的页面文字、工具调用或截图可能进入聊天记录。扩展的数据说明只适用于扩展，不能据此推定原生桌面截图的保存期限。[桌面数据范围](https://learn.chatgpt.com/docs/computer-use) · [浏览器扩展数据说明](https://learn.chatgpt.com/docs/chrome-extension) |

**“锁定窗口”至少要分三项核对。** 用户指定了哪扇窗，界面上标出了哪扇窗，这一次输入实际上发给了哪扇窗。用同一个应用打开两份同名文档，操作途中交换前后顺序，是区分三者最直接的办法。OpenAI 公布了指定窗口和按应用授权的行为。对于同名窗口、窗口关闭重建、最小化、切换桌面空间后的身份保持，公开资料不足以给出通用保证。[官方任务范围](https://learn.chatgpt.com/docs/computer-use)

### 同一个聊天入口可以走四条路径

| 路径 | 在哪里执行 | 对体验的影响 |
| --- | --- | --- |
| Computer | 用户授权的本机应用 | 能使用已经打开的原生文档、软件和模拟器。需要验证画布、弹窗、遮挡以及焦点变化。[桌面手册](https://learn.chatgpt.com/docs/computer-use) |
| Chrome 扩展 | 用户已有浏览器配置、标签页和登录态 | 用户可指定当前标签，助手直接使用网页控件。权限可以按网站管理。测试这条路径时应记录是否已有登录态。[扩展说明](https://learn.chatgpt.com/docs/chrome-extension) |
| 内置浏览器或云浏览器 | 内置浏览器有自己的浏览状态，云浏览器运行在独立的云端电脑 | 两者可使用各自的登录会话。云浏览器可让用户接管、登录和交还，即使用户关闭自己的电脑也可继续受支持的任务。云端页面操作无法证明本机原生应用的后台输入能力。[内置浏览器](https://help.openai.com/en/articles/20001277-using-the-built-in-browser-in-the-chatgpt-desktop-app) · [云浏览器](https://help.openai.com/en/articles/20001280-using-cloud-browser-in-chatgpt) |
| 应用插件或连接器 | 通过应用提供的结构化接口读写数据 | 对可重复的数据任务，官方建议优先用专用集成。表格更新可以通过专用接口完成。画图软件里一块画布是否真的被点中，需要另外验收。[工具选择演示](https://developers.openai.com/training/walkthroughs/using-your-computer-and-browser) |

从开发者能复用的部分看，OpenAI 的 Responses Computer 工具支持一次返回多个有序动作，由宿主执行并回传更新后的画面。SDK 另有读取页面和使用浏览器控件的路径。批量点选稳定控件有机会减少模型往返。出现弹窗、切换目标窗口或提交数据时，应以新的界面状态决定下一步。这是小伴可验证的设计选择，公开 API 没有披露 ChatGPT 在某个 Mac 窗口里实际用了哪种系统输入接口。[Computer Use API](https://developers.openai.com/api/docs/guides/tools-computer-use) · [官方集成示例](https://developers.openai.com/api/docs/guides/tools-computer-use-integration)

## 哪些产品值得放在同一台机器上比较

| 产品 | 已公开的执行方式 | 小伴最值得对照的行为 |
| --- | --- | --- |
| **Claude Cowork** | macOS 15 及以上默认在后台窗口操作，通常会等用户打完字。某次会话首次需要接管全屏时会征求同意。每个应用要授权，也有只读、限制输入和禁止访问等差异。Cowork 按连接器、浏览器、屏幕交互选择合适方式。 | 测中文输入法组词中途会不会被打断，以及后台做不了时怎样向用户交代。它是后台输入和接管体验最直接的对手。[Claude 官方说明](https://support.claude.com/en/articles/14128542-let-claude-use-your-computer-in-cowork) |
| **Gemini for Mac** | 2026 年 11 月 19 日公布。把应用操作移到虚拟屏幕，聊天中有实时预览，用户可暂停、控制应用权限和审批关键动作。发布时面向美国英语个人付费账号，要求 Apple Silicon 和 macOS 15 及以上。 | 对比“助手操作现有窗口”和“助手在另一块屏幕操作”两种做法。具体看窗口何时消失、用户能否找到它、结束后回到哪里。[Google 发布说明](https://blog.google/innovation-and-ai/products/gemini-app/computer-use-mac/) · [设置与资格](https://support.google.com/gemini/answer/18956635?hl=en) |
| **Perplexity Personal Computer** | Mac 桌面应用连接本机文件和原生应用，已登录网站交给 Comet，可在手机上批准敏感步骤。可让一台 Mac mini 持续工作。 | 参照长任务何时需要通知，以及用户如何从手机接回任务。持续在另一台 Mac 工作，和同一台 Mac 上共用键鼠，要分别验证。[安装与能力](https://www.perplexity.ai/personal-computer-setup) · [通知行为](https://www.perplexity.ai/en-GB/changelog/personal-computer-for-all-users-on-mac) |
| **Microsoft Copilot Actions / Agent Workspace** | 微软在 Windows Insider 的公开方案中，为代理提供本机独立账号和受控工作区，用户继续用自己的桌面。共享文件和连接器需要配置。 | 适合作为 Windows 后台方案的参照。验收要问清独立工作区能否取得用户当前打开的文档和既有登录态。[微软支持文档](https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features) |
| **Power Automate for desktop** | 已定义好的桌面流程可在画中画模式运行。子会话有独立进程和应用状态。产品还提供虚拟桌面模式，两种模式各有应用兼容限制。 | 对固定的重复任务，比较一次授权后连续执行、可观察性和失败定位。它运行预定义流程，成绩应与首次收到自然语言任务的产品分开记。[微软画中画模式](https://learn.microsoft.com/en-us/power-automate/desktop-flows/run-desktop-flows-pip) |
| **Manus** | Browser Operator 可在用户本机浏览器的新标签页工作，使用已有登录态。桌面 My Computer 的官方说明则以终端命令操作本机文件、工具和应用。 | 在已有登录态的网站跨页填表，既能验证标签页定位，也便于看清究竟是哪条路径完成任务。[My Browser](https://manus.im/my-browser) · [My Computer](https://www.manus.im/blog/manus-my-computer-desktop) |
| **字节跳动 UI-TARS Desktop** | 开源桌面 GUI Agent，提供截图识别、操作状态反馈及本地和远程执行选项，可配置视觉模型。 | 作为视觉操作参照，用同一张没有标准控件的画布测试定位、重试和结果核验。使用远程环境取得的成绩单独记录。[项目仓库](https://github.com/bytedance/UI-TARS-desktop) · [官方快速开始](https://github.com/bytedance/UI-TARS-desktop/blob/main/docs/quick-start.md) |
| **百度 DuMate** | 中文桌面办公助手。其 Chrome 扩展通过 Debugger API 操作标签页，文档列举填表、查询、下载和页面截图。 | 作为中文网页办公参照，测试登录后的表格、分页和批量下载。原生 Mac 窗口后台点击仍需另行验证。[百度浏览器接入说明](https://cloud.baidu.com/discover/dumate-browser-guide-entry.html) · [产品概览](https://cloud.baidu.com/discover/dumate-product-access-capability.html) |

第一轮同机测评优先选 **ChatGPT、Claude、Gemini 和小伴**，理由是它们都直接涉及“我用这台 Mac 时，助手还能不能操作另一个原生应用”。网页测试另选 ChatGPT 扩展、Manus 或 DuMate。Windows 同时保留前台操作和独立会话两组，二者对现有窗口、登录态的继承能力不同。

## “执行很快”有哪些证据

OpenAI 公布过两组有明确口径的数字。GPT-6 Astra 与当时的 GPT-5.6 Sol 相比，在 **OSWorld 2.0 离线任务的延迟模拟**中，每项约 40 分钟和 75 分钟，部分得分为 72.6% 和 65.7%。在 **Mind2Web 网页基准**上，新模型和更新后的 Codex 运行框架合计带来 **1.9 倍任务完成速度**。这两组数字都没有测量用户 Mac 上从按下发送到第一次点击的时间。更新的 GPT-6.2 发布材料展示了精细 CAD 操作，也没有给出这项本机计时。[OpenAI 性能报告](https://openai.com/index/gpt-6-astra/) · [GPT-6.2 产品示例](https://openai.com/index/introducing-gpt-6-2-astra-sol/)

OSWorld-Human 的作者把时间拆成规划、界面定位、点击、截图和结果判断。在他们测的 Agent S2、GTA1 上，模型规划占比很高，重复定位也会浪费步骤。沿用这种方法，小伴可以先数完整任务的模型往返和失败重试，再测本机权限检查、控件树读取、截图与动画各花多久。该研究没有测小伴或生产版 ChatGPT。[OSWorld-Human](https://arxiv.org/abs/2506.16042)

| 从什么时候计时 | 到什么时候停止 | 能回答的问题 |
| --- | --- | --- |
| 用户发送指令 | 目标窗口第一次出现可核验的有效变化 | 用户等了多久才看到它开始做事。另记首次显示“正在操作”的时间，辨别空转 |
| 前一步生效 | 下一步生效 | 连续输入是否流畅。区分模型等待、系统动作和页面加载 |
| 用户发送指令 | 通过独立检查确认任务目标达成 | 按时完成了多少，失败花了多久，是否需要用户介入 |
| 用户点击停止或接管 | 最后一个实际动作结束 | 停止是否可靠，是否有排队动作在停止后继续写入 |

## 小伴当前的起点

| 当前证据 | 对体验的影响与可验证问题 |
| --- | --- |
| 小伴固定 Open Computer Use **1.1.2**。这版上游已改进软件光标跟随移动窗口。小伴技能记载了对被覆盖窗口使用 `sky_click`、`sky_key`，也支持临时移动到代理显示器再恢复。 | 已有后台输入和窗口跟随基础。首先要验证常规聊天会不会选到这些动作，以及移动窗口、出现弹窗、停止任务时表现是否一致。[版本锁](../../../libs/agent-tools/computer-use/runtime-lock.json) · [小伴操作规则](../../../libs/agent-tools/computer-use/skills/open-computer-use/SKILL.md) · [上游 1.1.2](https://github.com/iFurySt/open-computer-use/releases/tag/v1.1.2) |
| 上游 **1.1.5** 已发布。维护者列出光标动画改为非阻塞、加快 Chrome 无障碍树读取和应用发现、改进窗口恢复与过期目标检查。1.1.3 也增加了按需获取控件树或截图的能力。 | 这是成本较低且可独立验证的变量。可让同一模型、同一主机和同一任务分别使用两个引擎版本，观察哪些阶段受益。升级效果目前没有小伴端的实测数据。[上游 1.1.5](https://github.com/iFurySt/open-computer-use/releases/tag/v1.1.5) · [上游 1.1.3](https://github.com/iFurySt/open-computer-use/releases/tag/v1.1.3) |
| 聊天宿主复用已连接的 MCP 服务，电脑操作工具按需发现。提示要求先读技能、必要时查工具，再找应用和窗口。macOS 的常用桌面动作执行前会启动引擎的 `doctor --json` 检查当前授权。 | 冷启动、模型准备和每次动作的本地开销是三个变量。应分别计时，保留授权撤销立即生效的语义。[聊天宿主](../src-tauri/src/chat/mod.rs) · [任务提示](../src-tauri/src/chat/prompt.rs) · [动作包装](../../../libs/agent-tools/computer-use/src/lib.rs) · [权限检查](../../../libs/agent-tools/computer-use/src/permissions.rs) |
| 小伴的 AI 配置暴露电脑控制总开关。当前动作包装检查引擎的系统授权，聊天层另有工具审批。这些入口中没有看到与 ChatGPT 设置对等的“允许哪些目标应用”列表。 | 对用户而言，“允许操作这次的备忘录”和“今后都能操作所有已打开的应用”差别很大。实机对照可让任务从已授权应用跳到另一个应用，观察目标切换时的提示及拒绝后的行为。[小伴配置](../src-tauri/src/ai_config.rs) · [小伴宿主权限策略](../src-tauri/src/chat/permission_policy.rs) · [OpenAI 应用授权](https://learn.chatgpt.com/docs/computer-use) |
| 电脑工具声明了同名的运行期独占资源。1.1.2 上游还说明，用持久 MCP 客户端时，需要向引擎告知轮次结束才能按期隐藏软件光标。小伴包装器会转发结束回调，但当前通用 MCP 工具实现没有发出对应的 `notifications/turn-ended` 通知。 | 已有避免不同任务抢用电脑的设计。应优先验证正常完成、取消、出错三种结束路径是否清除光标和预览，再谈多窗口并行。源码差异是排查线索，还不能断言已发生界面残留。[资源与结束回调](../../../libs/agent-tools/computer-use/src/lib.rs) · [MCP 实现](../../../libs/agent-kernel/src/mcp.rs) · [默认回调](../../../libs/agent-kernel/src/tool.rs) · [1.1.2 上游说明](https://github.com/iFurySt/open-computer-use/blob/v1.1.2/skills/open-computer-use/references/installation.md) |
| 工具链可接收 MCP 图片和结构化输出，技能也要求动作后回读。针对浏览器，控件树缺失时当前指引会尝试地址栏和 Tab 键。现有工具包文档把真实点击和输入回读列为实机验收项。 | 页面操作、画布点击和原生文本框要分别测。给模型返回“事件已发出”只能证明尝试过，验收应检查目标文件或目标控件的最终状态。[MCP 结果处理](../../../libs/agent-kernel/src/mcp.rs) · [操作与回读规则](../../../libs/agent-tools/computer-use/skills/open-computer-use/SKILL.md) · [现有验证范围](../../../libs/agent-tools/computer-use/README.md) |

## 如何判定追平和超越

下列是小伴的候选验收任务，可全程使用测试文档、测试网页和虚构数据。先按各家默认设置比较完整产品，再给小伴固定模型、引擎版本和执行路径做归因。原生 GUI 组明确要求操作指定应用。同一任务另开“允许任何工具”组，以反映真实用户只关心交付结果的情况。

| 测试情境 | 独立验收 |
| --- | --- |
| 被遮挡的目标窗口，用户正在另一应用用中文输入法打字 | 确认目标文件新增了指定内容，同时确认前台应用、用户选区和未提交的拼音保持正常，剪贴板标记未被意外替换 |
| 同一应用、两个同名文档，途中移动窗口或拉到副屏 | 目标以测试文档标记区分。只有指定文档改变，视觉标记跟随正确窗口。窗口被关闭时应停在可解释状态 |
| 原生菜单弹出对话框，操作一次因焦点变化失败 | 能重新观察、定位正确对话框并完成，记录误点、重复写入和人工接管次数 |
| 登入测试网站，跨页找一条记录，填写草稿 | 回读指定记录和所有字段，确认草稿状态正确。记录使用本地标签页、内置浏览器、云浏览器或桌面点击中的哪条路径 |
| 画布没有标准控件，在不同缩放比例的屏幕点击、拖动 | 检查导出图像的落点或形状位置，同时记录系统鼠标是否跳转。跨屏和高 DPI 各测一次 |
| 连续录入十项，一次在中途停止，另一次在运行时撤销权限 | 停止后统计新增的动作与写入。授权撤销后目标保持不变，能说明停在哪一步，光标和预览可收起 |
| 两个不同应用同时工作，再让两个任务写同一窗口 | 分别记录完成、排队、拒绝或要求接管。同一窗口不能静默交叉写入，一项结束不应清除另一项的运行标记 |

同一轮记录产品版本、模型档位、系统版本、硬件、显示器布局与缩放、网络、登录状态和授权状态。首次安装授权、冷启动、已授权热启动分别记录。起步可每个场景配对跑 5 次，轮换产品顺序，报告原始次数、中位数和范围。样本增加后再看 P95。将超时、主动接管、假完成都纳入分母，产品不公开的成本或模型往返数记为未知。

若小伴在这组任务上的通过率、首个动作耗时和全程耗时都接近 ChatGPT，且测试中未发生抢焦点、串窗或停止后继续写入，才有依据说它在这些场景中**追平**。如果在中文输入法和双屏并用的任务里，小伴还能保持正确率、减少输入打断并缩短接管时间，就能具体说明它在哪些体验上实现了**超越**。可用 MacArena 补充任务覆盖度。它 有 421 个经人工核验的任务、覆盖 50 个应用，真机干扰则可借鉴记录前台应用和系统指针的方法。任何“比 ChatGPT 快”的结论都应附上任务、日期、版本和样本量。[MacArena](https://arxiv.org/abs/2606.06560) · [OSWorld 执行结果评测](https://arxiv.org/abs/2404.07972) · [真机干扰测量示例](https://github.com/PrithviSeran/mac-agent-bench)

小伴首轮实测最值得保留的是一段“后台写字、前台打字”的全屏录像，一次从发送指令到逐字回读的分段计时，一次中途停止后的窗口和光标状态。如果后台输入已经可靠，下一步可利用桌宠在不打扰用户的位置显示“正在操作哪个文档”，点开即可看预览或停止。如果观察到串窗或抢焦点，先修正输入目标。两种情况需要的开发工作完全不同。
