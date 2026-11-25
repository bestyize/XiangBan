<script setup lang="ts">
import { onActivated, onDeactivated, onMounted, onUnmounted, ref } from 'vue'

type WorkKey = 'home' | 'plan'
type SpaceKey = 'wallpapers' | 'effects' | 'companion'

const activeSection = ref('work')
const workTab = ref<WorkKey>('home')
const spaceTab = ref<SpaceKey>('wallpapers')

const workShots: Record<WorkKey, { src: string; alt: string; title: string; pill: string; cap: string; w: number; h: number }> = {
  home: {
    src: '/desktop/desktop-home.webp',
    alt: '小伴桌面 v3.1.8 新建任务工作台界面',
    title: '✳ 小伴桌面 · 新建任务工作台',
    pill: '最新实机界面',
    cap: '办公 / 学习 / 生活 / 创作快捷入口，随时开启任务',
    w: 1280,
    h: 800,
  },
  plan: {
    src: '/desktop/task-plan.webp',
    alt: '小伴为咖啡店活动页面列出的任务执行步骤',
    title: '✳ 小伴桌面 · 任务拆解与执行过程',
    pill: '步骤 2 / 5',
    cap: '自动拆解任务步骤，并可展开查看实时思考摘要',
    w: 849,
    h: 440,
  },
}

const spaceShots: Record<SpaceKey, { src: string; alt: string; title: string; pill: string }> = {
  wallpapers: {
    src: '/desktop/desktop-wallpapers.webp',
    alt: '小伴桌面 v3.1.8 壁纸库与装扮界面',
    title: '小伴桌面 · 装扮 / 壁纸',
    pill: '精选动静壁纸',
  },
  effects: {
    src: '/desktop/desktop-effects.webp',
    alt: '小伴桌面 v3.1.8 桌面特效与灯火阑珊参数面板',
    title: '小伴桌面 · 装扮 / 桌面特效（灯火阑珊）',
    pill: '14 种轻量特效',
  },
  companion: {
    src: '/desktop/desktop-companion.webp',
    alt: '小伴桌面 v3.1.8 花朵心语与多肉点点桌面伙伴',
    title: '小伴桌面 · 装扮 / 桌面伙伴',
    pill: '花朵心语 & 多肉点点',
  },
}

const sectionIds = ['work', 'create', 'space', 'plugins', 'mobile']

function jumpTo(id: string) {
  const el = document.getElementById(id)
  if (!el) return
  el.scrollIntoView({ behavior: 'smooth', block: 'start' })
  activeSection.value = id === 'play' ? 'plugins' : id
}

function handleScroll() {
  let current = sectionIds[0]
  for (const id of sectionIds) {
    const el = document.getElementById(id)
    if (!el) continue
    if (el.getBoundingClientRect().top <= 220) current = id
  }
  activeSection.value = current
}

onMounted(() => window.addEventListener('scroll', handleScroll, { passive: true }))
onUnmounted(() => window.removeEventListener('scroll', handleScroll))
onActivated(() => window.addEventListener('scroll', handleScroll, { passive: true }))
onDeactivated(() => window.removeEventListener('scroll', handleScroll))
</script>

<template>
  <main class="feat-page">
    <!-- Hero + 6-Capability Bento -->
    <section class="wrap feat-hero">
      <div>
        <span class="kicker"><span class="kicker-dot" /> WHAT YOU CAN DO</span>
        <h1>想做的事，<br><em>从对话到桌面都顺手。</em></h1>
        <p class="hero-desc">
          把 AI 工作助手、创意作品工坊、动态桌面装扮、插件扩展、精选应用与手机端协同收进同一个轻盈空间——用自然对话推进任务，也把每天面对的屏幕打理成喜欢的样子。
        </p>
        <div class="hero-actions">
          <RouterLink class="btn btn-primary" to="/download">
            下载小伴
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 17 17 7" /><path d="M8 7h9v9" /></svg>
          </RouterLink>
          <button type="button" class="btn btn-secondary" @click="jumpTo('work')">
            逐项浏览功能
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M12 5v14" /><path d="m19 12-7 7-7-7" /></svg>
          </button>
        </div>
      </div>

      <div class="cap-overview">
        <div class="cap-overview-head">
          <strong><span>✳</span> 核心能力一览 · 桌面 &amp; 手机</strong>
          <span>点击直达章节</span>
        </div>
        <div class="cap-grid">
          <button type="button" class="cap-tile" @click="jumpTo('work')">
            <span class="cap-ico" aria-hidden="true">
              <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><path d="M9 11l3 3L22 4" /><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" /></svg>
            </span>
            <span class="cap-meta">
              <span class="cap-num">01 / WORK</span>
              <h3>AI 工作助手</h3>
              <p>拆解步骤、调用工具与透明确认</p>
            </span>
          </button>

          <button type="button" class="cap-tile" @click="jumpTo('create')">
            <span class="cap-ico" aria-hidden="true">
              <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><path d="M12 3l1.9 5.8L20 10.5l-5 3.6L16.8 20 12 16.5 7.2 20 9 14.1 4 10.5l6.1-1.7z" /></svg>
            </span>
            <span class="cap-meta">
              <span class="cap-num">02 / CREATE</span>
              <h3>创作与作品</h3>
              <p>设计稿、网页与小游戏实时预览打磨</p>
            </span>
          </button>

          <button type="button" class="cap-tile" @click="jumpTo('space')">
            <span class="cap-ico" aria-hidden="true">
              <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><rect x="3" y="4" width="18" height="13" rx="2" /><path d="M8 21h8" /><path d="M12 17v4" /></svg>
            </span>
            <span class="cap-meta">
              <span class="cap-num">03 / SPACE</span>
              <h3>个性桌面</h3>
              <p>动态壁纸、14 种特效、皮肤与桌宠</p>
            </span>
          </button>

          <button type="button" class="cap-tile" @click="jumpTo('plugins')">
            <span class="cap-ico" aria-hidden="true">
              <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><rect x="3" y="3" width="7" height="7" rx="1.5" /><rect x="14" y="3" width="7" height="7" rx="1.5" /><rect x="14" y="14" width="7" height="7" rx="1.5" /><rect x="3" y="14" width="7" height="7" rx="1.5" /></svg>
            </span>
            <span class="cap-meta">
              <span class="cap-num">04 / EXTEND</span>
              <h3>AI 插件与市场</h3>
              <p>专家角色、技能库与 MCP 连接器</p>
            </span>
          </button>

          <button type="button" class="cap-tile" @click="jumpTo('play')">
            <span class="cap-ico" aria-hidden="true">
              <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><rect x="2" y="6" width="20" height="12" rx="4" /><path d="M7 12h4" /><path d="M9 10v4" /><circle cx="15.5" cy="11" r="1" /><circle cx="17.5" cy="13" r="1" /></svg>
            </span>
            <span class="cap-meta">
              <span class="cap-num">05 / APPS</span>
              <h3>精选应用与游戏</h3>
              <p>内置轻游戏、实用工具与自制作品</p>
            </span>
          </button>

          <button type="button" class="cap-tile" @click="jumpTo('mobile')">
            <span class="cap-ico" aria-hidden="true">
              <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><rect x="6" y="2" width="12" height="20" rx="2.5" /><path d="M11 18h2" /></svg>
            </span>
            <span class="cap-meta">
              <span class="cap-num">06 / MOBILE</span>
              <h3>手机版 &amp; 互联</h3>
              <p>随时语音通话、遥控电脑与社区</p>
            </span>
          </button>
        </div>
      </div>
    </section>

    <!-- Sticky 6-Section Anchor Bar -->
    <div class="subnav-bar">
      <div class="subnav-inner">
        <nav class="subnav-links hide-scrollbar" aria-label="功能导航">
          <button type="button" class="subnav-pill" :class="{ active: activeSection === 'work' }" @click="jumpTo('work')"><em>01</em> AI 工作助手</button>
          <button type="button" class="subnav-pill" :class="{ active: activeSection === 'create' }" @click="jumpTo('create')"><em>02</em> 创作与作品</button>
          <button type="button" class="subnav-pill" :class="{ active: activeSection === 'space' }" @click="jumpTo('space')"><em>03</em> 个性桌面</button>
          <button type="button" class="subnav-pill" :class="{ active: activeSection === 'plugins' }" @click="jumpTo('plugins')"><em>04</em> AI 插件</button>
          <button type="button" class="subnav-pill" :class="{ active: activeSection === 'plugins' }" @click="jumpTo('play')"><em>05</em> 精选应用</button>
          <button type="button" class="subnav-pill" :class="{ active: activeSection === 'mobile' }" @click="jumpTo('mobile')"><em>06</em> 手机版小伴</button>
        </nav>
        <span class="subnav-note">Windows · macOS · Android</span>
      </div>
    </div>

    <div class="wrap">
      <!-- 01 / WORK -->
      <section id="work" class="feat-section">
        <div class="sec-header">
          <span class="sec-num">01 / WORK</span>
          <span class="sec-tag">任务拆解 · 透明执行 · 模型自选</span>
        </div>
        <div class="feat-split">
          <div class="feat-copy">
            <h2>不止是对话，<br>还能一起把事情办成。</h2>
            <p>
              说出目标，小伴会自动拆解计划、检索资料或整理工作区文件。每一步执行过程都能展开查看，涉及敏感操作时按应用设置等你确认。
            </p>
            <div class="feat-points">
              <div class="feat-point">
                <span class="feat-point-ico" aria-hidden="true">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M9 11l3 3L22 4" /></svg>
                </span>
                <div>
                  <strong>任务自动拆解与进度跟踪</strong>
                  <span>复杂需求拆成清晰步骤，实时看到每一步耗时与产出。</span>
                </div>
              </div>
              <div class="feat-point">
                <span class="feat-point-ico" aria-hidden="true">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" /></svg>
                </span>
                <div>
                  <strong>权限清晰，关键动作人工确认</strong>
                  <span>读写文件、执行命令按工作区与安全规则把关，安心放手。</span>
                </div>
              </div>
              <div class="feat-point">
                <span class="feat-point-ico" aria-hidden="true">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4" /></svg>
                </span>
                <div>
                  <strong>小伴 AI 与自有模型自由切换</strong>
                  <span>登录即用官方多档位模型，也可按需配置你自己的模型服务。</span>
                </div>
              </div>
            </div>
            <RouterLink to="/guide" class="text-link">查看玩法指南 <span aria-hidden="true">↗</span></RouterLink>
          </div>

          <div class="visual-stage s1">
            <div class="view-tabs" role="tablist" aria-label="工作助手界面视角">
              <button type="button" role="tab" :aria-selected="workTab === 'home'" :class="{ active: workTab === 'home' }" @click="workTab = 'home'">新建任务工作台</button>
              <button type="button" role="tab" :aria-selected="workTab === 'plan'" :class="{ active: workTab === 'plan' }" @click="workTab = 'plan'">任务拆解执行</button>
            </div>
            <div class="window-frame">
              <div class="window-bar">
                <div class="window-dots" aria-hidden="true"><i /><i /><i /></div>
                <span class="window-title">{{ workShots[workTab].title }}</span>
                <span class="window-pill">{{ workShots[workTab].pill }}</span>
              </div>
              <div class="window-body">
                <img :src="workShots[workTab].src" :alt="workShots[workTab].alt" :width="workShots[workTab].w" :height="workShots[workTab].h">
              </div>
            </div>
            <div class="stage-caption">
              <span>{{ workShots[workTab].cap }}</span>
              <small>点击上方页签切换视角</small>
            </div>
          </div>
        </div>
      </section>

      <!-- 02 / CREATE -->
      <section id="create" class="feat-section">
        <div class="sec-header">
          <span class="sec-num">02 / CREATE</span>
          <span class="sec-tag">对话生成 · 实时预览 · 持续打磨</span>
        </div>
        <div class="feat-split reverse">
          <div class="feat-copy">
            <h2>让灵感变成<br>可以继续打磨的作品。</h2>
            <p>
              从设计稿、网页到游戏或桌面内容，通过对话探索创意、查看成果并继续修改。桌面端提供作品预览、设计画布与完整导出等能力。
            </p>
            <div class="feat-points">
              <div class="feat-point">
                <span class="feat-point-ico" aria-hidden="true">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><rect x="3" y="3" width="18" height="18" rx="2" /><path d="M3 9h18" /></svg>
                </span>
                <div>
                  <strong>边聊边改，成果同屏呈现</strong>
                  <span>不满意的配色、版式和交互，直接用自然语言继续调整。</span>
                </div>
              </div>
              <div class="feat-point">
                <span class="feat-point-ico" aria-hidden="true">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                </span>
                <div>
                  <strong>设计画布与完整作品导出</strong>
                  <span>支持独立网页、资源包与设计稿沉淀，方便直接交付或分享社区。</span>
                </div>
              </div>
            </div>
            <RouterLink to="/guide" class="text-link">寻找创作灵感 <span aria-hidden="true">↗</span></RouterLink>
          </div>

          <div class="visual-stage s2">
            <div class="creative-prompt-bar">
              <span class="chip-mini">示例需求</span>
              <span>“帮春野咖啡做一个奶油色 + 自然绿的春日限定活动单页，适配手机”</span>
            </div>
            <div class="window-frame">
              <div class="window-bar">
                <div class="window-dots" aria-hidden="true"><i /><i /><i /></div>
                <span class="window-title">作品预览 · 春野咖啡春日限定页</span>
                <span class="window-pill">可交互 HTML</span>
              </div>
              <div class="window-body">
                <img src="/desktop/task-result.webp" alt="小伴根据示例需求生成的咖啡店活动页面" width="535" height="387" loading="lazy">
              </div>
            </div>
            <div class="stage-caption">
              <span>图中为真实任务生成的示例活动页面</span>
              <small>支持继续对话改稿与导出</small>
            </div>
          </div>
        </div>
      </section>

      <!-- 03 / YOUR SPACE -->
      <section id="space" class="feat-section">
        <div class="sec-header">
          <span class="sec-num">03 / YOUR SPACE</span>
          <span class="sec-tag">壁纸 · 特效 · 组件 · 桌面伙伴</span>
        </div>
        <div class="pano-head">
          <h2>把每天面对的桌面，<br>装扮成自己喜欢的样子。</h2>
          <p>从精选静态与动态视频壁纸，到十四个轻量桌面特效、实用小组件和会陪伴互动的桌面伙伴，让工作空间拥有自己的节奏。</p>
        </div>

        <div class="pano-stage">
          <div class="pano-bar">
            <div class="view-tabs" style="margin-bottom:0" role="tablist" aria-label="个性桌面界面切换">
              <button type="button" role="tab" :aria-selected="spaceTab === 'wallpapers'" :class="{ active: spaceTab === 'wallpapers' }" @click="spaceTab = 'wallpapers'">动态壁纸</button>
              <button type="button" role="tab" :aria-selected="spaceTab === 'effects'" :class="{ active: spaceTab === 'effects' }" @click="spaceTab = 'effects'">14 种桌面特效</button>
              <button type="button" role="tab" :aria-selected="spaceTab === 'companion'" :class="{ active: spaceTab === 'companion' }" @click="spaceTab = 'companion'">桌面伙伴</button>
            </div>
            <span class="pano-hint">点击页签或下方卡片切换实机界面</span>
          </div>

          <div class="window-frame">
            <div class="window-bar">
              <div class="window-dots" aria-hidden="true"><i /><i /><i /></div>
              <span class="window-title">{{ spaceShots[spaceTab].title }}</span>
              <span class="window-pill">{{ spaceShots[spaceTab].pill }}</span>
            </div>
            <div class="window-body">
              <img :src="spaceShots[spaceTab].src" :alt="spaceShots[spaceTab].alt" width="1280" height="800" loading="lazy">
            </div>
          </div>

          <div class="pano-grid">
            <button type="button" class="pano-card" :class="{ 'is-active': spaceTab === 'wallpapers' }" @click="spaceTab = 'wallpapers'">
              <div class="pano-card-head">
                <span class="pano-card-ico" aria-hidden="true">
                  <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9"><rect x="3" y="3" width="18" height="18" rx="2" /><circle cx="8.5" cy="8.5" r="1.5" /><path d="m21 15-5-5L5 21" /></svg>
                </span>
                <h3>图片与动态视频壁纸</h3>
              </div>
              <p>内置山水客厅、雨林黑水、清绿水草、浮光与一池心灯等动静壁纸，也可直连在线图库或导入自定义资源包。</p>
            </button>

            <button type="button" class="pano-card" :class="{ 'is-active': spaceTab === 'effects' }" @click="spaceTab = 'effects'">
              <div class="pano-card-head">
                <span class="pano-card-ico" aria-hidden="true">
                  <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9"><path d="M12 3v3m0 12v3M3 12h3m12 0h3M5.6 5.6l2.1 2.1m8.6 8.6 2.1 2.1m0-12.8-2.1 2.1m-8.6 8.6-2.1 2.1" /></svg>
                </span>
                <h3>14 种桌面特效与组件</h3>
              </div>
              <p>龙鱼巡游、灯火阑珊、星汉灿烂、庄周梦蝶等轻量特效可叠在任意壁纸之上，支持实时调节数量、速度与大小。</p>
            </button>

            <button type="button" class="pano-card" :class="{ 'is-active': spaceTab === 'companion' }" @click="spaceTab = 'companion'">
              <div class="pano-card-head">
                <span class="pano-card-ico" aria-hidden="true">
                  <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9"><path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1-1.1a5.5 5.5 0 0 0-7.8 7.8l1 1L12 21.2l7.8-7.8 1-1a5.5 5.5 0 0 0 0-7.8z" /></svg>
                </span>
                <h3>主题制桌面伙伴与皮肤</h3>
              </div>
              <p>「花朵心语」「多肉点点」7 只萌宠可同时养在桌面上，搭配春江花月夜、江南烟雨等应用皮肤与关怀提醒。</p>
            </button>
          </div>

          <div class="pano-foot">
            <span>想看动态壁纸实机演示与互动桌宠？</span>
            <RouterLink to="/desktop-experience" class="text-link">探索桌面体验专页 <span aria-hidden="true">↗</span></RouterLink>
          </div>
        </div>
      </section>

      <!-- 04 / EXTEND + 05 / APPS -->
      <section id="plugins" class="feat-section">
        <div class="sec-header">
          <span class="sec-num">04 &amp; 05 / EXTEND &amp; APPS</span>
          <span class="sec-tag">按需扩展能力 · 随时轻松片刻</span>
        </div>

        <div class="twin-grid">
          <article class="twin-card">
            <div class="twin-media s4">
              <div class="window-frame">
                <div class="window-bar">
                  <div class="window-dots" aria-hidden="true"><i /><i /><i /></div>
                  <span class="window-title">AI 插件 · 专家 / 技能 / 连接器</span>
                </div>
                <img src="/desktop/current-plugins.webp" alt="小伴桌面 AI 插件专家列表" width="849" height="670" loading="lazy">
              </div>
            </div>
            <div class="twin-body">
              <span class="sec-num">04 / EXTEND</span>
              <h3>为不同任务，装上合适的能力。</h3>
              <p>
                按场景挑选专业领域的专家角色、SkillHub 技能包与 MCP 连接器，让小伴顺手衔接更多工具；安装和使用前可查看来源与权限说明。
              </p>
              <div class="tag-row">
                <span class="tag-pill">场景化专家</span>
                <span class="tag-pill">SkillHub 技能</span>
                <span class="tag-pill">MCP 连接器</span>
              </div>
              <RouterLink to="/plugins" class="text-link">探索 AI 插件 <span aria-hidden="true">↗</span></RouterLink>
            </div>
          </article>

          <article id="play" class="twin-card">
            <div class="twin-media s1">
              <div class="window-frame">
                <div class="window-bar">
                  <div class="window-dots" aria-hidden="true"><i /><i /><i /></div>
                  <span class="window-title">精选应用 · 游戏 / 工具 / 测评</span>
                </div>
                <img src="/desktop/desktop-apps.webp" alt="小伴桌面精选应用与官方游戏列表" width="1280" height="800" loading="lazy">
              </div>
            </div>
            <div class="twin-body">
              <span class="sec-num">05 / APPS &amp; PLAY</span>
              <h3>工作间隙，也有顺手的轻应用与游戏。</h3>
              <p>
                精选应用覆盖 2048、方块爆破、倒杯快乐、水墨剑客、3D 全阶魔方、MBTI 性格测评与潮汐书房，全部本机存档；也可随时对话定制你的专属作品。
              </p>
              <div class="tag-row">
                <span class="tag-pill">11 款精选作品</span>
                <span class="tag-pill">本机存档</span>
                <span class="tag-pill">一句话共创</span>
              </div>
              <RouterLink to="/desktop-experience" class="text-link">了解更多桌面功能 <span aria-hidden="true">↗</span></RouterLink>
            </div>
          </article>
        </div>
      </section>

      <!-- 06 / MOBILE -->
      <section id="mobile" class="feat-section">
        <div class="sec-header">
          <span class="sec-num">06 / MOBILE</span>
          <span class="sec-tag">小伴手机 App · 随身 AI 与跨端互联</span>
        </div>
        <div class="pano-head">
          <h2>离开电脑，<br>小伴也在手机上陪着你。</h2>
          <p>原生打造的 Android 手机版带来连续对话主线、实时语音通话、随身作品试玩，并支持与同账号小伴桌面互联，出门也能远程发任务。</p>
        </div>

        <div class="mobile-stage">
          <div class="phone-trio">
            <div class="phone-shot-card">
              <div class="phone-bezel">
                <img src="/desktop/mobile-ai-home.webp" alt="小伴手机版 AI 首页连续对话与遥控电脑" width="540" height="1108" loading="lazy">
              </div>
              <div class="phone-shot-meta">
                <strong>连续主线 · 随手交代任务</strong>
                <span>提醒卡、创作入口与遥控电脑一屏直达</span>
              </div>
            </div>

            <div class="phone-shot-card featured">
              <div class="phone-bezel">
                <img src="/desktop/mobile-voice-call.webp" alt="小伴手机版实时语音通话" width="540" height="1108" loading="lazy">
              </div>
              <div class="phone-shot-meta">
                <strong>双星相伴 · 实时语音通话</strong>
                <span>日落珊瑚光球响应，开口即问、随时打断</span>
              </div>
            </div>

            <div class="phone-shot-card">
              <div class="phone-bezel">
                <img src="/desktop/mobile-ai-chat.webp" alt="小伴手机版对话任务摘要与作品交付卡片" width="540" height="1135" loading="lazy">
              </div>
              <div class="phone-shot-meta">
                <strong>任务过程与作品交付卡</strong>
                <span>可展开步骤摘要，生成的工具一键安装打开</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Closing Banner -->
      <section class="closing-banner">
        <div>
          <span class="kicker">READY WHEN YOU ARE</span>
          <h2>下一步，从小伴开始。</h2>
          <p>支持 Windows、macOS 与 Android，下载安装包即可开启体验。</p>
        </div>
        <div class="closing-actions">
          <RouterLink class="btn btn-primary" to="/download">下载小伴 <span aria-hidden="true">↗</span></RouterLink>
          <RouterLink class="btn btn-secondary" to="/guide">玩法指南</RouterLink>
        </div>
      </section>

      <footer class="site-footer">
        <div class="footer-brand">
          <strong>小伴桌面</strong>
          <p>你的 AI 工作与创作助手</p>
        </div>
        <div class="footer-links">
          <RouterLink to="/features">功能介绍</RouterLink>
          <RouterLink to="/download">下载安装</RouterLink>
          <RouterLink to="/guide">玩法指南</RouterLink>
          <RouterLink to="/images">壁纸</RouterLink>
          <a href="https://chat.xiangban.xyz/contact">联系我们</a>
          <a href="https://chat.xiangban.xyz/" target="_blank" rel="noopener noreferrer">社区 ↗</a>
        </div>
      </footer>
    </div>
  </main>
</template>

<style scoped>
.feat-page {
  --bg: #fafaf9;
  --surface: #ffffff;
  --surface-soft: #f4f3f0;
  --s1: #f2f1ed;
  --s2: #f4f2ee;
  --s3: #efeeeb;
  --s4: #f1f2f4;
  --ink: #141416;
  --ink-2: #52525b;
  --ink-3: #71717a;
  --brand: #18181b;
  --brand-hover: #2f2f35;
  --brand-soft: #ededf0;
  --line: rgba(24, 24, 27, 0.08);
  --line-strong: rgba(24, 24, 27, 0.16);
  background: var(--bg);
  color: var(--ink);
  min-height: 100vh;
}

.wrap {
  width: min(1220px, calc(100% - 64px));
  margin: 0 auto;
}

.kicker {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  letter-spacing: 0.14em;
  font-weight: 700;
  color: var(--ink-2);
  text-transform: uppercase;
}

.kicker-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--brand);
  box-shadow: 0 0 0 4px rgba(24, 24, 27, 0.12);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 46px;
  padding: 0 22px;
  border-radius: 10px;
  font-size: 14.5px;
  font-weight: 650;
  transition: transform 0.15s var(--ease), background 0.2s, border-color 0.2s, box-shadow 0.2s;
  white-space: nowrap;
}

.btn:active { transform: scale(0.98); }

.btn-primary {
  background: var(--brand);
  color: #fff;
  box-shadow: 0 6px 18px rgba(15, 15, 20, 0.14);
}
.btn-primary:hover {
  background: var(--brand-hover);
  box-shadow: 0 10px 24px rgba(15, 15, 20, 0.18);
}

.btn-secondary {
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line-strong);
}
.btn-secondary:hover {
  border-color: var(--ink-3);
  background: var(--surface-soft);
}

.text-link {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 650;
  color: var(--brand);
  transition: gap 0.2s var(--ease);
}
.text-link:hover {
  gap: 12px;
  text-decoration: underline;
  text-underline-offset: 4px;
}

/* Hero */
.feat-hero {
  padding: 64px 0 56px;
  display: grid;
  grid-template-columns: 1.02fr 0.98fr;
  gap: 48px;
  align-items: center;
}

.feat-hero h1 {
  font-size: clamp(40px, 4.6vw, 64px);
  line-height: 1.15;
  letter-spacing: -0.045em;
  font-weight: 700;
  margin: 22px 0 20px;
}

.feat-hero h1 em {
  font-style: normal;
  color: #52525b;
}

.hero-desc {
  font-size: 16px;
  line-height: 1.85;
  color: var(--ink-2);
  max-width: 470px;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 30px;
}

.cap-overview {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 22px;
  padding: 18px;
  box-shadow: 0 14px 36px rgba(15, 15, 20, 0.06);
}

.cap-overview-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 6px 14px;
  border-bottom: 1px solid var(--line);
  margin-bottom: 14px;
  font-size: 12px;
  color: var(--ink-3);
}

.cap-overview-head strong {
  color: var(--ink);
  font-weight: 650;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.cap-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.cap-tile {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  border-radius: 13px;
  background: var(--bg);
  border: 1px solid transparent;
  text-align: left;
  transition: background 0.2s, border-color 0.2s, transform 0.2s var(--ease);
}

.cap-tile:hover {
  background: var(--surface-soft);
  border-color: var(--line-strong);
  transform: translateY(-2px);
}

.cap-ico {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--brand);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.cap-meta {
  min-width: 0;
  flex: 1;
}

.cap-num {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--ink-3);
  display: block;
  margin-bottom: 2px;
}

.cap-tile h3 {
  font-size: 14.5px;
  font-weight: 650;
  color: var(--ink);
  margin-bottom: 3px;
}

.cap-tile p {
  font-size: 12.5px;
  color: var(--ink-3);
  line-height: 1.5;
}

/* Sticky Sub-nav */
.subnav-bar {
  position: sticky;
  top: var(--header-h);
  z-index: 30;
  background: rgba(250, 250, 249, 0.9);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  border-top: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
}

.subnav-inner {
  width: min(1220px, calc(100% - 64px));
  margin: 0 auto;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.subnav-links {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow-x: auto;
}

.subnav-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 13px;
  border-radius: 999px;
  font-size: 13px;
  color: var(--ink-2);
  white-space: nowrap;
  transition: background 0.18s, color 0.18s;
}

.subnav-pill em {
  font-style: normal;
  font-size: 11px;
  font-weight: 700;
  opacity: 0.6;
}

.subnav-pill:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.subnav-pill.active {
  background: var(--brand);
  color: #fff;
  font-weight: 600;
}

.subnav-pill.active em { opacity: 0.8; }

.subnav-note {
  font-size: 12px;
  color: var(--ink-3);
  white-space: nowrap;
}

/* Feature Sections */
.feat-section {
  padding: 88px 0;
  scroll-margin-top: 124px;
  border-bottom: 1px solid var(--line);
}

.feat-section:last-of-type {
  border-bottom: 0;
}

.sec-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 36px;
}

.sec-num {
  font-size: 11.5px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ink-2);
}

.sec-tag {
  font-size: 12.5px;
  color: var(--ink-3);
}

.feat-split {
  display: grid;
  grid-template-columns: 0.88fr 1.12fr;
  gap: 56px;
  align-items: center;
}

.feat-split.reverse {
  grid-template-columns: 1.12fr 0.88fr;
}

.feat-split.reverse .feat-copy {
  order: 2;
}

.feat-copy h2 {
  font-size: clamp(28px, 3.2vw, 42px);
  line-height: 1.24;
  letter-spacing: -0.035em;
  font-weight: 700;
  margin-bottom: 18px;
}

.feat-copy > p {
  font-size: 15.5px;
  line-height: 1.85;
  color: var(--ink-2);
  max-width: 450px;
}

.feat-points {
  margin: 26px 0 30px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.feat-point {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  background: var(--surface);
  border: 1px solid var(--line);
}

.feat-point-ico {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  background: var(--brand-soft);
  color: var(--brand);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  margin-top: 1px;
}

.feat-point strong {
  display: block;
  font-size: 14px;
  font-weight: 650;
  color: var(--ink);
  margin-bottom: 2px;
}

.feat-point span {
  display: block;
  font-size: 13px;
  color: var(--ink-3);
  line-height: 1.5;
}

/* Visual Stage */
.visual-stage {
  border-radius: 22px;
  padding: 28px 28px 20px;
  position: relative;
  overflow: hidden;
}

.s1 { background: var(--s1); }
.s2 { background: var(--s2); }
.s4 { background: var(--s4); }

.view-tabs {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px;
  border-radius: 11px;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid var(--line);
  margin-bottom: 14px;
}

.view-tabs button {
  height: 30px;
  padding: 0 12px;
  border-radius: 7px;
  font-size: 12.5px;
  font-weight: 550;
  color: var(--ink-2);
  transition: background 0.18s, color 0.18s;
}

.view-tabs button:hover { color: var(--ink); }

.view-tabs button.active {
  background: var(--brand);
  color: #fff;
  font-weight: 650;
  box-shadow: 0 2px 8px rgba(15, 15, 20, 0.12);
}

.window-frame {
  background: #fff;
  border-radius: 14px;
  border: 1px solid var(--line);
  box-shadow: 0 20px 44px rgba(15, 15, 20, 0.09);
  overflow: hidden;
}

.window-bar {
  height: 34px;
  padding: 0 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #fbfbfb;
  border-bottom: 1px solid var(--line);
  font-size: 11.5px;
  color: var(--ink-3);
}

.window-dots {
  display: inline-flex;
  gap: 5px;
}

.window-dots i {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #d4d4d8;
}

.window-title {
  font-weight: 600;
  color: var(--ink-2);
}

.window-pill {
  font-size: 10.5px;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--brand-soft);
  color: var(--brand);
  font-weight: 600;
}

.window-body {
  background: #fafafa;
}

.window-body img {
  width: 100%;
  height: auto;
}

.stage-caption {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 15px;
  font-size: 12px;
  color: var(--ink-2);
  padding: 0 4px;
}

.stage-caption small {
  font-size: 11.5px;
  color: var(--ink-3);
}

.creative-prompt-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  margin-bottom: 14px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.86);
  border: 1px solid var(--line);
  font-size: 12.5px;
  color: var(--ink-2);
}

.creative-prompt-bar .chip-mini {
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--brand);
  color: #fff;
  font-size: 11px;
  font-weight: 650;
  flex-shrink: 0;
}

/* Panoramic Bento (03) */
.pano-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 32px;
  margin-bottom: 32px;
}

.pano-head h2 {
  font-size: clamp(28px, 3.2vw, 42px);
  line-height: 1.24;
  letter-spacing: -0.035em;
  font-weight: 700;
  max-width: 520px;
}

.pano-head p {
  font-size: 15px;
  color: var(--ink-2);
  max-width: 420px;
  line-height: 1.8;
}

.pano-stage {
  padding: 32px 32px 22px;
  border-radius: 22px;
  background: var(--s3);
}

.pano-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 14px;
}

.pano-hint {
  font-size: 12px;
  color: var(--ink-3);
}

.pano-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
  margin-top: 18px;
}

.pano-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 15px;
  padding: 20px 22px;
  text-align: left;
  transition: border-color 0.2s, transform 0.2s var(--ease), box-shadow 0.2s;
}

.pano-card:hover {
  border-color: var(--line-strong);
  transform: translateY(-2px);
}

.pano-card.is-active {
  border-color: var(--brand);
  box-shadow: 0 14px 36px rgba(15, 15, 20, 0.06);
}

.pano-card-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.pano-card-ico {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  background: var(--surface-soft);
  color: var(--brand);
  display: grid;
  place-items: center;
}

.pano-card.is-active .pano-card-ico {
  background: var(--brand);
  color: #fff;
}

.pano-card h3 {
  font-size: 15.5px;
  font-weight: 650;
}

.pano-card p {
  font-size: 13.5px;
  color: var(--ink-2);
  line-height: 1.7;
}

.pano-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 22px;
  padding-top: 18px;
  border-top: 1px solid var(--line);
  font-size: 13px;
  color: var(--ink-2);
}

/* Twin Cards (04 & 05) */
.twin-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
}

.twin-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 22px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 2px 10px rgba(15, 15, 20, 0.04);
  transition: box-shadow 0.25s var(--ease), transform 0.25s var(--ease);
  scroll-margin-top: 124px;
}

.twin-card:hover {
  box-shadow: 0 14px 36px rgba(15, 15, 20, 0.06);
  transform: translateY(-3px);
}

.twin-media {
  padding: 22px 22px 0;
}

.twin-media .window-frame {
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
  border-bottom: 0;
  box-shadow: 0 12px 30px rgba(15, 15, 20, 0.08);
}

.twin-media img {
  aspect-ratio: 16 / 10.6;
  width: 100%;
  object-fit: cover;
  object-position: top;
}

.twin-body {
  padding: 28px 28px 30px;
  display: flex;
  flex-direction: column;
  flex: 1;
}

.twin-body h3 {
  font-size: 24px;
  line-height: 1.3;
  letter-spacing: -0.025em;
  font-weight: 700;
  margin: 10px 0 12px;
}

.twin-body > p {
  font-size: 14.5px;
  color: var(--ink-2);
  line-height: 1.78;
}

.tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 20px 0 24px;
}

.tag-pill {
  display: inline-flex;
  align-items: center;
  padding: 5px 11px;
  border-radius: 999px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  font-size: 12.5px;
  color: var(--ink-2);
  font-weight: 500;
}

.twin-body .text-link {
  margin-top: auto;
}

/* Mobile Showcase (06) */
.mobile-stage {
  border-radius: 24px;
  background: var(--s1);
  border: 1px solid var(--line);
  padding: 40px 36px 32px;
  overflow: hidden;
}

.phone-trio {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
  align-items: end;
  max-width: 960px;
  margin: 0 auto;
}

.phone-shot-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
}

.phone-bezel {
  width: 100%;
  max-width: 268px;
  padding: 7px;
  border-radius: 30px;
  background: #18181b;
  box-shadow: 0 22px 48px rgba(15, 15, 20, 0.18);
  transition: transform 0.25s var(--ease);
}

.phone-shot-card.featured .phone-bezel {
  max-width: 290px;
  transform: translateY(-10px);
  box-shadow: 0 28px 64px rgba(15, 15, 20, 0.26);
}

.phone-bezel:hover { transform: translateY(-6px); }
.phone-shot-card.featured .phone-bezel:hover { transform: translateY(-14px); }

.phone-bezel img {
  width: 100%;
  height: auto;
  border-radius: 24px;
}

.phone-shot-meta {
  text-align: center;
}

.phone-shot-meta strong {
  display: block;
  font-size: 14.5px;
  font-weight: 650;
  color: var(--ink);
}

.phone-shot-meta span {
  display: block;
  font-size: 12.5px;
  color: var(--ink-3);
  margin-top: 2px;
}

/* Closing & Footer */
.closing-banner {
  margin: 28px 0 64px;
  padding: 64px 56px;
  border-radius: 24px;
  background: linear-gradient(135deg, #f3f2ef 0%, #f7f6f4 60%, #efeee9 100%);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 32px;
}

.closing-banner h2 {
  font-size: clamp(28px, 3vw, 40px);
  line-height: 1.25;
  letter-spacing: -0.035em;
  font-weight: 700;
  margin: 12px 0 10px;
}

.closing-banner p {
  font-size: 15px;
  color: var(--ink-2);
}

.closing-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.site-footer {
  border-top: 1px solid var(--line);
  padding: 36px 0 calc(var(--tabbar-h) + var(--safe-bottom) + 40px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}

@media (min-width: 769px) {
  .site-footer {
    padding-bottom: 64px;
  }
}

.footer-brand strong {
  font-size: 18px;
  font-weight: 700;
  display: block;
}

.footer-brand p {
  font-size: 12.5px;
  color: var(--ink-3);
  margin-top: 4px;
}

.footer-links {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  font-size: 13.5px;
  color: var(--ink-2);
}

.footer-links a:hover {
  color: var(--brand);
  text-decoration: underline;
}

@media (max-width: 960px) {
  .feat-hero,
  .feat-split,
  .feat-split.reverse,
  .pano-grid,
  .twin-grid,
  .phone-trio {
    grid-template-columns: 1fr;
  }
  .feat-split.reverse .feat-copy { order: 0; }
  .pano-head { flex-direction: column; align-items: flex-start; gap: 14px; }
  .closing-banner { flex-direction: column; align-items: flex-start; padding: 36px 28px; }
  .phone-shot-card.featured .phone-bezel { transform: none; max-width: 268px; }
}

@media (max-width: 768px) {
  .wrap,
  .subnav-inner {
    width: calc(100% - 32px);
  }
  .subnav-bar { top: 0; }
  .subnav-note { display: none; }
  .feat-hero { padding: 32px 0 36px; gap: 28px; }
  .cap-grid { grid-template-columns: 1fr; }
  .feat-section { padding: 52px 0; scroll-margin-top: 64px; }
  .visual-stage,
  .pano-stage { padding: 16px 16px 14px; border-radius: 16px; }
  .mobile-stage { padding: 24px 16px; }
  .window-pill { display: none; }
  .stage-caption,
  .pano-foot,
  .site-footer { flex-direction: column; align-items: flex-start; }
  .closing-actions { width: 100%; }
  .closing-actions .btn { flex: 1; }
}
</style>
