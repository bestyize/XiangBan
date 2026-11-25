<script setup lang="ts">
import { computed, ref } from 'vue'

type SceneKey = 'agent' | 'collab' | 'plugins' | 'dragon' | 'widgets' | 'mcreate' | 'mvoice' | 'mtools'
type WorkKey = 'dusk' | 'ink' | 'mbti'
type NightKey = 'dragon' | 'widgets' | 'wallpapers' | 'companion'
type MobilePillarKey = 'create' | 'voice' | 'phonelink' | 'tools' | 'community'

interface SceneItem {
  key: SceneKey
  label: string
  plat: '电脑' | '手机'
  title: string
  desc: string
  tag: string
  img?: string
  imgs?: [string, string]
}

const scenes: SceneItem[] = [
  {
    key: 'agent',
    label: '控电脑 & 多模型',
    plat: '电脑',
    img: '/desktop/desktop-config-ai.webp',
    title: '独立多模型窗口 + 原生 Computer Use 控电脑屏幕与 ADB 控手机',
    desc: '官方小伴 AI 之外内置 DeepSeek、智谱、Kimi、MiniMax 与任意 OpenAI 兼容模型；开启电脑屏幕与应用控制后，直接看屏点击、输入并自动化操作。',
    tag: '电脑端最新 · v3.1.8 实机',
  },
  {
    key: 'collab',
    label: '外部 Agent & 语音 MCP',
    plat: '电脑',
    img: '/desktop/desktop-config-agent.webp',
    title: '外部 Agent 协作中枢：连 Claude Code / Codex 改画布，开放本机语音 MCP',
    desc: '一键接入 Codex、ChatGPT 桌面与 Claude Code 协同读写本机设计稿，安装 Hooks 监听终端任务，还能把本地 SenseVoice 识别与 Edge 朗读开放为回环语音 MCP。',
    tag: '电脑端最新 · 外部 Agent',
  },
  {
    key: 'plugins',
    label: '专家市场 & 技能',
    plat: '电脑',
    img: '/desktop/desktop-plugins-v318.webp',
    title: '小伴市场 & SkillHub：54+ 行业专家、技能与 OAuth MCP 连接器',
    desc: '按需安装自动化测试、产品原型设计、腾讯文档/百度网盘、3D 游戏工坊与代码审查专家，来源透明、版本可控。',
    tag: '电脑端最新 · AI 插件',
  },
  {
    key: 'dragon',
    label: '3D 龙鱼 & 14 特效',
    plat: '电脑',
    img: '/desktop/desktop-dragon.webp',
    title: '14 种顶层透明特效：3D 朱红龙鱼 & 锦鲤巡游，光照鱼群实时调',
    desc: '朱红龙鱼、锦鲤、灯火阑珊、星汉灿烂、庄周梦蝶……叠加在任意动静壁纸之上，还能让 AI 现做专属特效包。',
    tag: '电脑端最新 · 3D 特效',
  },
  {
    key: 'widgets',
    label: '8 款实时桌面组件',
    plat: '电脑',
    img: '/desktop/desktop-widgets.webp',
    title: '8 款多屏实时桌面组件：股票行情、电子木鱼、2026 农历休/班月历',
    desc: '桌面时钟、城市天气、待办事项、今日日程、实时股票行情、电子木鱼、法定节假日月历与纪念日，多显示器自由摆放。',
    tag: '电脑端最新 · 实时组件',
  },
  {
    key: 'mcreate',
    label: '手机本地造作品',
    plat: '手机',
    imgs: ['/desktop/mobile-chat-build.webp', '/desktop/mobile-my-works.webp'],
    title: '手机本地原生 Agent 造作品与 3D 游戏：实时校验、草稿续作、一键发布',
    desc: '内置 Rust 原生 Agent 与 Three.js，实时展示步骤清单并自动补齐缺失资源；中断自动保留草稿，支持本机沙箱试玩或导出至电脑桌面。',
    tag: '手机端最新 · 本地创作',
  },
  {
    key: 'mvoice',
    label: '端侧语音 & 控电脑',
    plat: '手机',
    imgs: ['/desktop/mobile-voice-call.webp', '/desktop/mobile-chat-remote.webp'],
    title: '端侧离线实时语音通话 + 远程遥控 MacBook Pro 查进度与改文件',
    desc: '本地 Silero VAD + Smart Turn + SenseVoice 低延迟开口打断；出门随时切换至电脑会话，查看文件修改 Diff、生图进度与方案确认卡。',
    tag: '手机端最新 · 语音 & 遥控',
  },
  {
    key: 'mtools',
    label: '原生社区 & 自由模型',
    plat: '手机',
    imgs: ['/desktop/mobile-community-new.webp', '/desktop/mobile-model-deepseek.webp'],
    title: '全新原生超级小伴社区 + 免登录自由接入 DeepSeek / OpenAI / Anthropic',
    desc: '一站发现《潮汐书房·鲸鱼娘》《江山时序》《龙鱼水族箱》等社区作品与排行榜；第三方 API 密钥仅保存在本机私有存储，同时内置离线红外万能遥控器。',
    tag: '手机端最新 · 社区 & 模型自由',
  },
]

const workTabs: Record<WorkKey, { label: string; src: string; alt: string }> = {
  dusk: { label: '暮色滑翔 3D', src: '/desktop/work-dusk-glider.webp', alt: '暮色滑翔 · 3D 低多边形滑翔互动作品' },
  ink: { label: '水墨剑客', src: '/desktop/work-ink-blade.webp', alt: '水墨剑客 · 国风水墨横版动作作品' },
  mbti: { label: '向内看 MBTI', src: '/desktop/work-mbti.webp', alt: '向内看 · MBTI 原生质感互动测评作品' },
}

const nightTabs: Record<NightKey, { label: string; src: string; alt: string }> = {
  dragon: { label: '3D 龙鱼巡游', src: '/desktop/desktop-dragon.webp', alt: '小伴桌面 3D 朱红龙鱼巡游特效与参数调节' },
  widgets: { label: '8 款实时组件', src: '/desktop/desktop-widgets.webp', alt: '小伴桌面 8 款实时桌面组件（股票行情、电子木鱼、月历等）' },
  wallpapers: { label: '动静壁纸库', src: '/desktop/desktop-wallpapers.webp', alt: '小伴桌面动静壁纸库' },
  companion: { label: '7 只感知桌宠', src: '/desktop/desktop-companion.webp', alt: '小伴桌面花朵心语与多肉点点 7 只桌宠' },
}

const mobilePillars: {
  key: MobilePillarKey
  num: string
  title: string
  body: string
  imgs: [string, string]
  capTitle: string
  capDesc: string
}[] = [
  {
    key: 'create',
    num: '01',
    title: '手机本地原生 Agent，直接造 3D 游戏 & 小程序',
    body: '内置 Rust 原生 Agent 与 Three.js，可导入相册图片/音频/3D 模型，在本机沙箱试玩、导出电脑兼容包或发布社区；也支持 AI 绘图/GIF 与真实系统闹钟提醒。',
    imgs: ['/desktop/mobile-chat-build.webp', '/desktop/mobile-my-works.webp'],
    capTitle: '端侧原生 Agent 造作品 · 实时步骤校验 & 草稿续作',
    capDesc: '支持本机沙箱试玩、中断自动保存草稿与一键发布，还能同步到电脑桌面使用。',
  },
  {
    key: 'voice',
    num: '02',
    title: '端侧离线级联语音通话，低延迟随时打断',
    body: '本地运行 Silero VAD + Smart Turn + SenseVoice 识别，不依赖云端卡顿；对着日落珊瑚双星光球自然对话，开口即打断追问。',
    imgs: ['/desktop/mobile-voice-call.webp', '/desktop/mobile-ai-home.webp'],
    capTitle: '端侧离线级联语音 · 日落珊瑚双星实时通话',
    capDesc: '本地 Silero VAD + Smart Turn + SenseVoice 管线，反应极快，随时开口打断追问。',
  },
  {
    key: 'phonelink',
    num: '03',
    title: '手机桌面互联：看电脑进度、手机审批高危权限',
    body: '自动发现同账号电脑，出门在外实时查看桌面 Agent 执行流，高危工具或命令卡片直接在手机点批准/拒绝，灵感也能一键推给电脑深度干活。',
    imgs: ['/desktop/mobile-chat-remote.webp', '/desktop/mobile-chat-approve.webp'],
    capTitle: '手机远程控电脑 · 文件 Diff、后台生图与权限审批',
    capDesc: '一键切到电脑遥控会话，查看改动明细与检查点，高危写入操作手机直接批准或拒绝。',
  },
  {
    key: 'tools',
    num: '04',
    title: '离线万能红外遥控器 + 动态壁纸 + 轻启直达',
    body: '内置离线电视与空调红外码库（含美的 SmartIR 精准控温），附带 30fps 系统动态壁纸、104 款应用「轻启」快捷方式与共享游戏广场。',
    imgs: ['/desktop/mobile-ai-home.webp', '/desktop/mobile-chat-approve.webp'],
    capTitle: '真系统闹钟提醒 + AI 生图/GIF + 离线万能红外遥控器',
    capDesc: '一句话写入手机系统日程提醒、生成 4:3 壁纸，并内置电视空调红外码库、动态壁纸与 104 应用「轻启」。',
  },
  {
    key: 'community',
    num: '05',
    title: '原生超级小伴社区 + 免登录自定义多模型',
    body: '一站刷创意作品、AI 日报、开源精选与大模型性价比天梯；官方小伴 AI 跨端同步，也可免登录添加 OpenAI / Anthropic 第三方模型，密钥仅存本机。',
    imgs: ['/desktop/mobile-community-new.webp', '/desktop/mobile-model-deepseek.webp'],
    capTitle: '全新原生超级小伴社区 + 免登录自定义多模型',
    capDesc: '发现 3D 互动与创意作品、AI 资讯与排行榜；第三方 API 密钥仅保存在本机，不上传云端。',
  },
]

const desktopMatrix = [
  { title: '全能自主 Agent', sub: '工作区读写、Shell 与 Git Worktree 并发' },
  { title: '控电脑 & 控手机', sub: '原生 Computer Use + ADB Phone Use' },
  { title: '专业无限设计画布', sub: '批注改稿、直出 .pptx / .fig 与 Design MCP' },
  { title: '超级小伴悬浮胶囊', sub: '全局快捷键跨应用区域截图标注即问' },
  { title: '外部 Agent 与语音 MCP', sub: '连 Claude Code/Codex、Hooks 与本地语音接口' },
  { title: '云文档库 & 图/GIF 生成', sub: '直连腾讯文档/百度网盘，自动归档素材' },
  { title: '3D 龙鱼 & 8 款组件', sub: '14 种特效、实时股票/木鱼/月历/桌宠' },
  { title: '插件市场 & 模型自由', sub: 'SkillHub、OAuth MCP、独立多模型测速窗' },
]

const mobileMatrix = [
  { title: '手机本地造 3D 游戏', sub: '原生 Agent + Three.js，沙箱试玩与导出' },
  { title: '端侧离线实时语音', sub: '本地 Silero + SenseVoice，开口即打断' },
  { title: '手机远程控电脑', sub: '实时看电脑进度，手机审批高危工具权限' },
  { title: '万能红外遥控器', sub: '离线码库直控电视空调，支持美的精准控温' },
  { title: 'AI 绘图/GIF & 真闹钟', sub: '对话文生图与动图，真实创建系统闹钟提醒' },
  { title: '系统动态壁纸 & 轻启', sub: '30fps 桌面/锁屏动态壁纸 + 104 应用快捷入口' },
  { title: '原生超级小伴社区', sub: '创意作品、AI 日报、开源精选与模型天梯' },
  { title: '免登录自定义多模型', sub: 'OpenAI / Anthropic 自由接入，密钥仅存本机' },
]

const activeSceneKey = ref<SceneKey>('agent')
const activeWorkKey = ref<WorkKey>('dusk')
const activeNightKey = ref<NightKey>('dragon')
const activeMobileKey = ref<MobilePillarKey>('create')

const currentScene = computed(() => scenes.find(s => s.key === activeSceneKey.value) ?? scenes[0])
const currentWork = computed(() => workTabs[activeWorkKey.value])
const currentNight = computed(() => nightTabs[activeNightKey.value])
const currentMobile = computed(() => mobilePillars.find(m => m.key === activeMobileKey.value) ?? mobilePillars[0])

function scrollToId(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}
</script>

<template>
  <main class="feat-page">
    <div class="wrap">
      <!-- Centered Commanding Hero -->
      <section class="feat-hero">
        <div class="hero-badge">
          <img src="/desktop/app-icon-new.webp" alt="" width="22" height="22">
          <span>小伴 3.1 · 电脑端 &amp; Android 手机端</span>
        </div>
        <h1>不止聊天与装扮，<br><span class="grad-text">真正会动手的全能 AI 伙伴。</span></h1>
        <p>
          在电脑上替你写代码、直接控屏幕与 Android 手机、在无限画布直出可编辑 PPTX 与 Figma、养一屏 3D 龙鱼与实时股票组件；出门切手机，端侧离线实时语音通话、本地造 3D 游戏、远程审批电脑权限，还内置万能红外遥控器。
        </p>
        <div class="hero-cta-row">
          <RouterLink to="/download" class="cta-coral">
            <span>免费下载小伴（电脑 / 手机）</span>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" aria-hidden="true"><path d="M7 17 17 7" /><path d="M8 7h9v9" /></svg>
          </RouterLink>
          <button type="button" class="cta-soft" @click="scrollToId('mobileFlagship')">
            直达手机版硬核能力 ↓
          </button>
        </div>
      </section>

      <!-- Interactive First-Screen 8-Scene Flagship Showcase -->
      <section class="live-stage-wrap" aria-label="双端核心能力实机体验">
        <div class="scene-dock" role="tablist">
          <button
            v-for="s in scenes"
            :key="s.key"
            type="button"
            role="tab"
            class="scene-chip"
            :class="{ active: activeSceneKey === s.key }"
            :aria-selected="activeSceneKey === s.key"
            @click="activeSceneKey = s.key"
          >
            <span>✦ {{ s.label }}</span>
            <span class="chip-plat">{{ s.plat }}</span>
          </button>
        </div>

        <div class="showcase-frame">
          <div class="showcase-inner">
            <div class="showcase-media" :class="{ 'phone-mode': Boolean(currentScene.imgs) }">
              <template v-if="currentScene.imgs">
                <div v-for="(src, idx) in currentScene.imgs" :key="src" class="phone-demo">
                  <img :src="src" :alt="`${currentScene.title} 预览 ${idx + 1}`" loading="lazy">
                </div>
              </template>
              <img v-else :src="currentScene.img" :alt="currentScene.title" class="desk-img">
            </div>
          </div>
          <div class="showcase-caption">
            <div>
              <h3>{{ currentScene.title }}</h3>
              <p>{{ currentScene.desc }}</p>
            </div>
            <span class="showcase-pill">{{ currentScene.tag }}</span>
          </div>
        </div>
      </section>

      <!-- SECTION 1: DESKTOP FLAGSHIP CAPABILITIES -->
      <section id="work" class="bento-sec">
        <div class="sec-title-center">
          <span>DESKTOP FLAGSHIP · WINDOWS &amp; MACOS</span>
          <h2>电脑端：你的全能 AI 工作台与活桌面</h2>
          <p>从底层自主操控系统与手机、专业多画板设计，到 3D 龙鱼巡游与实时股票桌面组件——把深度生产力与桌面松弛感合二为一。</p>
        </div>

        <div class="bento-grid">
          <!-- Card 1 (8 cols): Autonomous Agent + Computer Use + Phone Use -->
          <article class="bento-card span-8">
            <span class="bento-kicker">AUTONOMOUS AGENT · 全能自主执行</span>
            <h3>不止给建议：替你操作工作区、控电脑、控手机</h3>
            <p>复杂任务自动拆解并透明展示每一步工具调用与思考摘要；执行中可随时补充指令排队，更支持设定验收命令的「长期目标」，重启断点也能自动续跑。</p>
            <div class="agent-pillars">
              <div class="ap-box">
                <strong>原生 Computer Use 控电脑</strong>
                <span>直接看懂屏幕、点击与输入，免额外环境替你操作桌面应用</span>
              </div>
              <div class="ap-box">
                <strong>ADB Phone Use 控手机</strong>
                <span>USB / 无线连接 Android 手机，在电脑对话里自动完成手机流程</span>
              </div>
              <div class="ap-box">
                <strong>Git Worktree 并发隔离</strong>
                <span>多会话各自在独立副本改代码、跑测试，互不干扰也支持一键回溯</span>
              </div>
            </div>
            <div class="bento-media">
              <img src="/desktop/desktop-config-ai.webp" alt="小伴桌面 v3.1.8 电脑屏幕控制、多模型独立设置窗" loading="lazy">
            </div>
          </article>

          <!-- Card 2 (4 cols Warm Peach): Design Canvas PPTX/Figma + 3D Works -->
          <article id="create" class="bento-card span-4 warm-peach">
            <span class="bento-kicker">DESIGN CANVAS &amp; STUDIO · 画布与创作</span>
            <h3>无限画布直出 PPTX / Figma，也能造 3D 作品</h3>
            <p>内置专业多画板设计画布，支持点击画板直接批注改稿、本机 Design MCP 与 Claude Code / Codex 协同，并一键导出可编辑 .pptx 与原生 Figma .fig；同时支持边聊边打磨 3D 与互动作品。</p>
            <div class="bento-tags">
              <span class="hi">导出可编辑 PPTX</span>
              <span class="hi">导出原生 .fig</span>
              <span>本机 Design MCP</span>
            </div>
            <div class="warm-tabs" role="tablist">
              <button
                v-for="(item, key) in workTabs"
                :key="key"
                type="button"
                role="tab"
                :class="{ active: activeWorkKey === key }"
                :aria-selected="activeWorkKey === key"
                @click="activeWorkKey = key as WorkKey"
              >
                {{ item.label }}
              </button>
            </div>
            <div class="bento-media" style="margin-top:14px">
              <img :src="currentWork.src" :alt="currentWork.alt" loading="lazy">
            </div>
          </article>

          <!-- Card 3 (5 cols Sky Mist): External Agent Collaboration + Local Voice MCP -->
          <article class="bento-card span-5 sky-mist">
            <span class="bento-kicker">EXTERNAL AGENT &amp; VOICE MCP · 跨工具协作中枢</span>
            <h3>连 Claude Code / Codex 协同改画布，开放本机语音 MCP</h3>
            <p>一键把本机设计画板接入 Codex、ChatGPT 桌面与 Claude Code，安装 Hooks 观察终端任务完成提醒；还能将本地 SenseVoice 识别与 Edge 朗读开放为回环 OpenAI / MCP 语音服务。</p>
            <div class="bento-tags">
              <span class="hi">Codex / Claude Code 协作</span>
              <span class="hi">本机语音 MCP (127.0.0.1)</span>
              <span>超级小伴截图标注</span>
            </div>
            <div class="bento-media" style="margin-top:auto">
              <img src="/desktop/desktop-config-agent.webp" alt="小伴桌面 v3.1.8 外部 Agent 协作、Hooks 观察层与本机语音 MCP" loading="lazy">
            </div>
          </article>

          <!-- Card 4 (7 cols Night Peak): 3D Dragon Fish + 8 Live Widgets + Wallpapers + 7 Pets -->
          <article class="bento-card span-7 night-peak">
            <span class="bento-kicker">LIVING DESKTOP · 有生命力的活桌面</span>
            <h3>3D 龙鱼巡游、8 款实时组件与感知作息的桌宠</h3>
            <p>14 种顶层透明特效含 3D 朱红龙鱼/锦鲤巡游，实时调光照与鱼群；8 款多屏组件涵盖股票行情、电子木鱼与 2026 休/班农历月历；搭配动态壁纸、国风皮肤与 7 只迎你回家的萌宠。</p>
            <div class="night-tabs" role="tablist">
              <button
                v-for="(item, key) in nightTabs"
                :key="key"
                type="button"
                role="tab"
                :class="{ active: activeNightKey === key }"
                :aria-selected="activeNightKey === key"
                @click="activeNightKey = key as NightKey"
              >
                {{ item.label }}
              </button>
            </div>
            <div class="bento-media">
              <img :src="currentNight.src" :alt="currentNight.alt" loading="lazy">
            </div>
          </article>

          <!-- Card 5 (5 cols): AI Plugins, MCP & Cloud Docs -->
          <article class="bento-card span-5">
            <span class="bento-kicker">PLUGINS, MCP &amp; CLOUD DOCS · 专家市场与云资料</span>
            <h3>小伴市场、SkillHub 与腾讯文档 / 百度网盘直连</h3>
            <p>在最新版 AI 插件页按需安装行业专家、技能包与 OAuth MCP 连接器；本地资料库更直连腾讯文档与百度网盘，对话还能文生图与生成 GIF 动图自动归档。</p>
            <div class="bento-tags">
              <span>小伴市场 &amp; SkillHub</span>
              <span>腾讯文档 / 百度网盘</span>
              <span>OAuth MCP 连接器</span>
            </div>
            <div class="bento-media" style="margin-top:auto">
              <img src="/desktop/desktop-plugins-v318.webp" alt="小伴桌面 v3.1.8 AI 插件专家与技能目录" loading="lazy">
            </div>
          </article>

          <!-- Card 6 (7 cols): Shared Apps & Games Plaza -->
          <article class="bento-card span-7 warm-ivory">
            <span class="bento-kicker">APPS &amp; GAMES · 精选应用广场</span>
            <h3>内置 11 款开箱即玩作品，双端互通本地存档</h3>
            <p>工作间隙直接开一局《水墨剑客》《暮色滑翔》《3D 全阶魔方》或打开《水族箱》《潮汐书房》《向内看 MBTI》；同一套协议 2 作品在电脑和 Android 手机上完全互通，有灵感还能让小伴现做一款。</p>
            <div class="bento-tags">
              <span>水墨剑客</span>
              <span>暮色滑翔 3D</span>
              <span>3D 全阶魔方</span>
              <span>水族箱</span>
              <span>潮汐书房 · 鲸鱼娘</span>
            </div>
            <div class="bento-media" style="margin-top:auto">
              <img src="/desktop/desktop-apps.webp" alt="小伴桌面精选应用与小游戏广场" loading="lazy">
            </div>
          </article>
        </div>
      </section>

      <!-- SECTION 2: MOBILE FLAGSHIP CAPABILITIES -->
      <section id="mobileFlagship" class="mobile-flagship-sec">
        <div class="sec-title-center">
          <span>MOBILE FLAGSHIP · ANDROID 原生打造</span>
          <h2>手机版：不是桌面配角，装进兜里的端侧 AI 超级应用</h2>
          <p>内置原生端侧 Agent 与离线语音管线，既能随时在手机上造 3D 游戏、离线遥控家里电视空调，也能无缝接管与审批电脑端的 AI 任务。</p>
        </div>

        <div class="mobile-hero-box">
          <div>
            <span class="bento-kicker and-kicker">5 BIG MOBILE POWERS · 点击切换最新实机 UI</span>
            <h3 class="mobile-box-title">随身创作、端侧语音、<br>遥控电脑与离线工具箱</h3>

            <div class="mobile-pillars-list" role="tablist">
              <button
                v-for="m in mobilePillars"
                :key="m.key"
                type="button"
                role="tab"
                class="mp-card"
                :class="{ active: activeMobileKey === m.key }"
                :aria-selected="activeMobileKey === m.key"
                @click="activeMobileKey = m.key"
              >
                <span class="mp-num">{{ m.num }}</span>
                <div>
                  <strong>{{ m.title }}</strong>
                  <span>{{ m.body }}</span>
                </div>
              </button>
            </div>
          </div>

          <div class="mobile-stage-right">
            <div class="mobile-dual-stage">
              <div class="m-device focus">
                <img :src="currentMobile.imgs[0]" :alt="`${currentMobile.capTitle} 预览 1`" loading="lazy">
              </div>
              <div class="m-device">
                <img :src="currentMobile.imgs[1]" :alt="`${currentMobile.capTitle} 预览 2`" loading="lazy">
              </div>
            </div>
            <div class="mobile-stage-caption">
              <div>
                <strong>{{ currentMobile.capTitle }}</strong>
                <span>{{ currentMobile.capDesc }}</span>
              </div>
              <span class="showcase-pill and-pill">最新 Android UI</span>
            </div>
          </div>
        </div>
      </section>

      <!-- SECTION 3: COMPLETE 8x2 CAPABILITY MATRIX -->
      <section class="matrix-sec">
        <div class="sec-title-center">
          <span>COMPLETE CAPABILITY MATRIX</span>
          <h2>桌面端与手机端，完整核心能力一览</h2>
        </div>

        <div class="matrix-grid">
          <div class="matrix-card">
            <div class="matrix-head">
              <h3>Windows &amp; macOS 桌面端</h3>
              <span class="matrix-badge desk">深度生产力 &amp; 活桌面</span>
            </div>
            <ul class="matrix-list">
              <li v-for="item in desktopMatrix" :key="item.title">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
                <div>{{ item.title }}<small>{{ item.sub }}</small></div>
              </li>
            </ul>
          </div>

          <div class="matrix-card and-card">
            <div class="matrix-head">
              <h3>Android 手机版</h3>
              <span class="matrix-badge and">端侧智能 &amp; 跨端接力</span>
            </div>
            <ul class="matrix-list">
              <li v-for="item in mobileMatrix" :key="item.title">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
                <div>{{ item.title }}<small>{{ item.sub }}</small></div>
              </li>
            </ul>
          </div>
        </div>
      </section>

      <!-- Final CTA -->
      <section class="final-cta">
        <h2>电脑与手机装齐，体验完整小伴</h2>
        <p>下载对应设备的官方原包，同一账号自动同步额度、对话、作品与跨端遥控。</p>
        <div class="hero-cta-row">
          <RouterLink to="/download" class="cta-coral">
            <span>前往下载小伴</span>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" aria-hidden="true"><path d="M7 17 17 7" /><path d="M8 7h9v9" /></svg>
          </RouterLink>
          <RouterLink to="/guide" class="cta-soft">看看玩法指南</RouterLink>
        </div>
      </section>

      <footer class="site-footer">
        <div class="footer-brand">
          <strong>小伴桌面</strong>
          <p>端云一体、双端接力的个人 AI 伙伴 · wallpaper.xiangban.xyz</p>
        </div>
        <div class="footer-links">
          <RouterLink to="/features">功能介绍</RouterLink>
          <RouterLink to="/download">下载安装</RouterLink>
          <RouterLink to="/guide">玩法指南</RouterLink>
          <RouterLink to="/images">壁纸库</RouterLink>
          <a href="https://chat.xiangban.xyz/contact" target="_blank" rel="noopener noreferrer">联系我们</a>
          <a href="https://chat.xiangban.xyz/" target="_blank" rel="noopener noreferrer">超级小伴社区 ↗</a>
        </div>
      </footer>
    </div>
  </main>
</template>

<style scoped>
.feat-page {
  --canvas: #fcfaf7;
  --canvas-soft: #f6f2ec;
  --surface: #ffffff;
  --ink: #19171a;
  --ink-2: #524d54;
  --ink-3: #837d86;
  --coral: #ee4f6d;
  --coral-grad: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  --coral-soft: #fff0f2;
  --coral-border: rgba(242, 92, 114, 0.24);
  --coral-shadow: 0 14px 34px rgba(247, 102, 94, 0.28);
  --and-ink: #158f4d;
  --line: rgba(28, 22, 28, 0.075);
  --line-strong: rgba(28, 22, 28, 0.14);
  --shadow-sm: 0 2px 10px rgba(26, 18, 24, 0.04);
  --shadow-md: 0 16px 42px rgba(26, 18, 24, 0.07);
  --shadow-lg: 0 28px 72px rgba(26, 18, 24, 0.11);
  --ease: cubic-bezier(0.22, 1, 0.36, 1);

  min-height: 100vh;
  color: var(--ink);
  background:
    radial-gradient(66% 520px at 50% -100px, rgba(255, 110, 134, 0.14), transparent 68%),
    radial-gradient(45% 400px at 12% 160px, rgba(249, 152, 92, 0.10), transparent 65%),
    radial-gradient(42% 380px at 90% 180px, rgba(130, 178, 255, 0.08), transparent 65%),
    var(--canvas);
}

.wrap {
  width: min(1180px, calc(100% - 64px));
  margin: 0 auto;
}

.grad-text {
  background: var(--coral-grad);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.cta-coral {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 50px;
  padding: 0 28px;
  border-radius: 14px;
  background: var(--coral-grad);
  color: #fff;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
  box-shadow: var(--coral-shadow);
  transition: transform 0.16s var(--ease), filter 0.18s, box-shadow 0.2s;
  white-space: nowrap;
}
.cta-coral:hover {
  filter: brightness(1.05);
  transform: translateY(-1px);
  box-shadow: 0 18px 40px rgba(247, 102, 94, 0.36);
}

.cta-soft {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 50px;
  padding: 0 22px;
  border-radius: 14px;
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line-strong);
  font-size: 14.5px;
  font-weight: 650;
  transition: background 0.18s, border-color 0.18s;
  white-space: nowrap;
}
.cta-soft:hover {
  background: var(--canvas-soft);
  border-color: rgba(28, 22, 28, 0.24);
}

/* Hero */
.feat-hero {
  padding: 60px 0 26px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px 5px 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid var(--coral-border);
  box-shadow: 0 4px 16px rgba(242, 92, 114, 0.09);
  font-size: 12.5px;
  font-weight: 650;
  color: #d6405e;
}
.hero-badge img {
  width: 22px;
  height: 22px;
  border-radius: 6px;
}
.feat-hero h1 {
  margin-top: 20px;
  font-size: clamp(34px, 4.9vw, 60px);
  line-height: 1.13;
  letter-spacing: -0.045em;
  font-weight: 760;
  max-width: 880px;
}
.feat-hero p {
  margin-top: 16px;
  font-size: 16px;
  color: var(--ink-2);
  max-width: 680px;
  line-height: 1.78;
}
.hero-cta-row {
  margin-top: 26px;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 12px;
}

/* Interactive 8-Scene Stage */
.live-stage-wrap {
  padding: 12px 0 70px;
}
.scene-dock {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 20px;
}
.scene-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.86);
  border: 1px solid var(--line);
  color: var(--ink-2);
  font-size: 13px;
  font-weight: 600;
  transition: all 0.2s var(--ease);
}
.scene-chip:hover {
  border-color: var(--coral-border);
  color: var(--ink);
  transform: translateY(-1px);
}
.scene-chip.active {
  background: var(--ink);
  color: #fff;
  border-color: var(--ink);
  box-shadow: 0 8px 22px rgba(25, 23, 26, 0.16);
}
.chip-plat {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(28, 22, 28, 0.06);
  color: var(--ink-3);
}
.scene-chip.active .chip-plat {
  background: rgba(255, 118, 138, 0.28);
  color: #ffd4dc;
}

.showcase-frame {
  border-radius: 30px;
  padding: 22px 22px 18px;
  background:
    radial-gradient(60% 70% at 88% 12%, rgba(255, 114, 134, 0.16), transparent 62%),
    radial-gradient(50% 60% at 12% 92%, rgba(249, 152, 92, 0.14), transparent 60%),
    #f4efe9;
  border: 1px solid rgba(255, 255, 255, 0.85);
  box-shadow: var(--shadow-lg);
}
.showcase-inner {
  border-radius: 20px;
  overflow: hidden;
  background: #fff;
  border: 1px solid rgba(28, 22, 28, 0.08);
  box-shadow: 0 18px 46px rgba(26, 18, 24, 0.10);
}
.showcase-media {
  background: #fafafa;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 440px;
  max-height: 620px;
  overflow: hidden;
}
.showcase-media .desk-img {
  width: 100%;
  height: auto;
  display: block;
}
.showcase-media.phone-mode {
  background:
    radial-gradient(55% 80% at 50% 50%, rgba(255, 101, 132, 0.22), transparent 70%),
    linear-gradient(135deg, #19161b 0%, #241b22 100%);
  padding: 28px 24px;
  gap: 28px;
}
.phone-demo {
  width: 244px;
  padding: 6px;
  border-radius: 30px;
  background: #0f0e11;
  box-shadow: 0 24px 54px rgba(0, 0, 0, 0.45), 0 0 0 1px rgba(255, 255, 255, 0.12);
}
.phone-demo img {
  width: 100%;
  border-radius: 24px;
  display: block;
}

.showcase-caption {
  margin-top: 15px;
  padding: 0 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.showcase-caption h3 {
  font-size: 16px;
  font-weight: 720;
  color: var(--ink);
}
.showcase-caption p {
  font-size: 13.5px;
  color: var(--ink-2);
  margin-top: 2px;
}
.showcase-pill {
  font-size: 12px;
  font-weight: 650;
  padding: 5px 12px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.88);
  color: #d6405e;
  border: 1px solid var(--coral-border);
  white-space: nowrap;
}
.showcase-pill.and-pill {
  color: var(--and-ink);
  border-color: rgba(21, 143, 77, 0.25);
  background: #f3fbf6;
}

/* Section Headers */
.sec-title-center {
  text-align: center;
  margin-bottom: 32px;
}
.sec-title-center span {
  font-size: 11.5px;
  font-weight: 750;
  letter-spacing: 0.14em;
  color: var(--coral);
  text-transform: uppercase;
}
.sec-title-center h2 {
  margin-top: 8px;
  font-size: clamp(24px, 3vw, 34px);
  letter-spacing: -0.03em;
  font-weight: 750;
}
.sec-title-center p {
  margin: 10px auto 0;
  max-width: 620px;
  font-size: 14.5px;
  color: var(--ink-2);
}

/* Bento Grid */
.bento-sec {
  padding: 20px 0 76px;
  scroll-margin-top: 90px;
}
.bento-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 18px;
}
.bento-card {
  border-radius: 26px;
  padding: 28px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--shadow-sm);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: transform 0.24s var(--ease), box-shadow 0.24s var(--ease);
}
.bento-card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-md);
}
.span-8 { grid-column: span 8; }
.span-7 { grid-column: span 7; }
.span-5 { grid-column: span 5; }
.span-4 { grid-column: span 4; }

.bento-kicker {
  font-size: 11px;
  font-weight: 750;
  letter-spacing: 0.12em;
  color: var(--coral);
  text-transform: uppercase;
}
.bento-kicker.and-kicker {
  color: var(--and-ink);
}
.bento-card h3 {
  margin-top: 8px;
  font-size: 23px;
  line-height: 1.26;
  letter-spacing: -0.025em;
  font-weight: 740;
}
.bento-card > p {
  margin-top: 10px;
  font-size: 13.8px;
  color: var(--ink-2);
  line-height: 1.72;
}
.bento-media {
  margin-top: 20px;
  border-radius: 16px;
  overflow: hidden;
  border: 1px solid var(--line);
  background: #fafafa;
  box-shadow: 0 12px 28px rgba(26, 18, 24, 0.07);
}
.bento-media img {
  width: 100%;
  height: auto;
  display: block;
}

.bento-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
  margin-top: 15px;
}
.bento-tags span {
  font-size: 12px;
  font-weight: 600;
  padding: 4px 11px;
  border-radius: 999px;
  background: var(--canvas-soft);
  color: var(--ink-2);
}
.bento-tags span.hi {
  background: var(--coral-soft);
  color: #d63d5b;
}

.agent-pillars {
  margin-top: 16px;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}
.ap-box {
  padding: 12px 13px;
  border-radius: 14px;
  background: var(--canvas);
  border: 1px solid var(--line);
}
.ap-box strong {
  display: block;
  font-size: 13px;
  font-weight: 700;
  color: var(--ink);
}
.ap-box span {
  display: block;
  font-size: 12px;
  color: var(--ink-3);
  margin-top: 3px;
  line-height: 1.5;
}

.warm-tabs {
  margin-top: 14px;
  display: inline-flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 4px;
  border-radius: 11px;
  background: rgba(255, 255, 255, 0.88);
  border: 1px solid rgba(242, 96, 114, 0.16);
  align-self: flex-start;
}
.warm-tabs button {
  height: 27px;
  padding: 0 10px;
  border-radius: 7px;
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-2);
  transition: 0.16s;
}
.warm-tabs button.active {
  background: var(--coral-grad);
  color: #fff;
}

.bento-card.warm-peach {
  background: linear-gradient(160deg, #fff5f1 0%, #fffaf7 55%, #ffffff 100%);
  border-color: rgba(242, 108, 92, 0.16);
}
.bento-card.warm-ivory {
  background: linear-gradient(155deg, #ffffff 0%, #fbf7f2 100%);
}
.bento-card.sky-mist {
  background: linear-gradient(160deg, #f1f6ff 0%, #f9fbff 60%, #ffffff 100%);
  border-color: rgba(59, 130, 246, 0.14);
}
.bento-card.sky-mist .bento-kicker { color: #2563eb; }
.bento-card.sky-mist .bento-tags span.hi { background: #e8f1ff; color: #1d4ed8; }

.bento-card.night-peak {
  background:
    radial-gradient(65% 85% at 88% 8%, rgba(255, 114, 134, 0.28), transparent 62%),
    radial-gradient(60% 70% at 10% 95%, rgba(249, 152, 75, 0.24), transparent 60%),
    linear-gradient(145deg, #171419 0%, #231a21 60%, #1c1822 100%);
  border-color: rgba(255, 188, 196, 0.16);
  color: #fff8f7;
}
.bento-card.night-peak .bento-kicker { color: #ff9aab; }
.bento-card.night-peak > p { color: rgba(255, 244, 244, 0.8); }
.bento-card.night-peak .bento-media {
  border-color: rgba(255, 255, 255, 0.14);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.36);
}

.night-tabs {
  margin-top: 15px;
  display: inline-flex;
  flex-wrap: wrap;
  gap: 5px;
  padding: 4px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  align-self: flex-start;
}
.night-tabs button {
  height: 30px;
  padding: 0 12px;
  border-radius: 8px;
  font-size: 12.2px;
  font-weight: 600;
  color: rgba(255, 244, 244, 0.78);
  transition: 0.18s;
}
.night-tabs button.active {
  background: var(--coral-grad);
  color: #fff;
}

/* Mobile Flagship Showcase */
.mobile-flagship-sec {
  padding: 8px 0 80px;
  scroll-margin-top: 90px;
}
.mobile-hero-box {
  border-radius: 32px;
  padding: 44px 40px;
  background:
    radial-gradient(58% 82% at 88% 12%, rgba(255, 108, 134, 0.18), transparent 64%),
    radial-gradient(50% 72% at 12% 92%, rgba(44, 196, 126, 0.14), transparent 62%),
    #f6f2ed;
  border: 1px solid rgba(28, 22, 28, 0.08);
  box-shadow: var(--shadow-md);
  display: grid;
  grid-template-columns: 0.98fr 1.02fr;
  gap: 36px;
  align-items: center;
}
.mobile-box-title {
  margin-top: 8px;
  font-size: clamp(24px, 2.8vw, 32px);
  line-height: 1.22;
  letter-spacing: -0.03em;
  font-weight: 750;
}
.mobile-pillars-list {
  margin-top: 22px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.mp-card {
  text-align: left;
  width: 100%;
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.74);
  border: 1px solid rgba(28, 22, 28, 0.07);
  transition: all 0.2s var(--ease);
  display: flex;
  align-items: flex-start;
  gap: 13px;
}
.mp-card:hover {
  background: rgba(255, 255, 255, 0.95);
  border-color: var(--coral-border);
}
.mp-card.active {
  background: #fff;
  border-color: var(--coral);
  box-shadow: 0 12px 30px rgba(242, 92, 114, 0.12);
  transform: translateX(4px);
}
.mp-num {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  background: var(--canvas-soft);
  color: var(--ink-2);
  font-size: 12px;
  font-weight: 800;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  margin-top: 1px;
  transition: 0.18s;
}
.mp-card.active .mp-num {
  background: var(--coral-grad);
  color: #fff;
}
.mp-card strong {
  display: block;
  font-size: 14.5px;
  font-weight: 720;
  color: var(--ink);
}
.mp-card span {
  display: block;
  font-size: 12.8px;
  color: var(--ink-2);
  margin-top: 3px;
  line-height: 1.58;
}

.mobile-stage-right {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}
.mobile-dual-stage {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  align-items: center;
  width: 100%;
  max-width: 490px;
}
.m-device {
  padding: 7px;
  border-radius: 30px;
  background: #18161a;
  box-shadow: 0 26px 56px rgba(26, 16, 22, 0.24);
  transition: transform 0.28s var(--ease);
}
.m-device.focus {
  transform: translateY(-10px) scale(1.03);
  box-shadow: 0 32px 68px rgba(26, 16, 22, 0.30);
}
.m-device img {
  width: 100%;
  border-radius: 23px;
  display: block;
}
.mobile-stage-caption {
  width: 100%;
  max-width: 490px;
  padding: 13px 16px;
  border-radius: 15px;
  background: rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.95);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 12.8px;
  color: var(--ink-2);
}
.mobile-stage-caption strong {
  color: var(--ink);
  font-weight: 700;
  display: block;
  font-size: 13.5px;
}

/* Matrix */
.matrix-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
  margin-bottom: 52px;
}
.matrix-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 26px;
  padding: 28px;
  box-shadow: var(--shadow-sm);
}
.matrix-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 18px;
}
.matrix-head h3 {
  font-size: 20px;
  font-weight: 740;
}
.matrix-badge {
  font-size: 12px;
  font-weight: 700;
  padding: 4px 11px;
  border-radius: 999px;
}
.matrix-badge.desk { color: #d83f5d; background: #fff0f2; }
.matrix-badge.and { color: var(--and-ink); background: #eaf9f0; }

.matrix-list {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  list-style: none;
}
.matrix-list li {
  padding: 11px 13px;
  border-radius: 13px;
  background: var(--canvas);
  border: 1px solid rgba(28, 22, 28, 0.045);
  font-size: 13px;
  color: var(--ink);
  font-weight: 600;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  line-height: 1.45;
}
.matrix-list li small {
  display: block;
  font-size: 11.5px;
  font-weight: 450;
  color: var(--ink-3);
  margin-top: 2px;
}
.matrix-list svg {
  color: var(--coral);
  flex-shrink: 0;
  margin-top: 2px;
}
.matrix-card.and-card .matrix-list svg {
  color: var(--and-ink);
}

/* Final CTA & Footer */
.final-cta {
  text-align: center;
  padding: 62px 28px;
  border-radius: 30px;
  background:
    radial-gradient(60% 90% at 50% 0%, rgba(255, 114, 134, 0.20), transparent 70%),
    linear-gradient(180deg, #fff7f5 0%, #fcfaf7 100%);
  border: 1px solid rgba(242, 96, 114, 0.16);
  margin-bottom: 56px;
}
.final-cta h2 {
  font-size: clamp(26px, 3.6vw, 42px);
  letter-spacing: -0.035em;
  font-weight: 750;
}
.final-cta p {
  margin: 12px auto 26px;
  font-size: 15.5px;
  color: var(--ink-2);
  max-width: 500px;
}

.site-footer {
  border-top: 1px solid var(--line);
  padding: 34px 0 calc(var(--tabbar-h, 58px) + var(--safe-bottom, 0px) + 38px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}
@media (min-width: 769px) {
  .site-footer { padding-bottom: 60px; }
}
.footer-brand strong { font-size: 17px; font-weight: 720; display: block; }
.footer-brand p { font-size: 12.5px; color: var(--ink-3); margin-top: 3px; }
.footer-links { display: flex; flex-wrap: wrap; gap: 22px; font-size: 13.5px; color: var(--ink-2); }
.footer-links a:hover { color: var(--coral); }

@media (max-width: 960px) {
  .bento-grid, .mobile-hero-box, .matrix-grid, .agent-pillars { grid-template-columns: 1fr; }
  .span-8, .span-7, .span-5, .span-4 { grid-column: auto; }
}

@media (max-width: 768px) {
  .wrap { width: calc(100% - 32px); }
  .feat-hero { padding: 32px 0 20px; }
  .feat-hero p { font-size: 14px; }
  .showcase-frame { padding: 12px; border-radius: 20px; }
  .showcase-media { min-height: 210px; }
  .showcase-media.phone-mode { flex-direction: column; padding: 20px; }
  .phone-demo { width: 216px; }
  .showcase-caption { flex-direction: column; align-items: flex-start; gap: 8px; }
  .bento-card { padding: 20px; border-radius: 20px; }
  .mobile-hero-box { padding: 24px 18px; border-radius: 22px; gap: 24px; }
  .m-device.focus { transform: none; }
  .matrix-list { grid-template-columns: 1fr; }
  .site-footer { flex-direction: column; align-items: flex-start; }
}
</style>
