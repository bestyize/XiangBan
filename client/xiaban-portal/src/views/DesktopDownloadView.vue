<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { BRAND_ICON_URL, assetUrl } from '../data/products'
import { getDesktopProduct, type DesktopProductDownload } from '../lib/desktop'
import { navigate, routeQuery } from '../lib/router'
import { toast } from '../lib/toast'

type DesktopTargetId = 'darwin-aarch64' | 'windows-x86_64' | 'darwin-x86_64'

interface TargetMeta {
  id: DesktopTargetId
  tab: string
  short: string
  ext: string
  req: string
  steps: [string, string, string]
}

const TARGET_METAS: Record<DesktopTargetId, TargetMeta> = {
  'darwin-aarch64': {
    id: 'darwin-aarch64',
    tab: 'macOS',
    short: 'macOS (Apple 芯片)',
    ext: '.dmg',
    req: 'macOS 12+ · M1–M4',
    steps: [
      '双击打开下载完成的 .dmg 镜像',
      '将「小伴桌面」拖入「应用程序」',
      '首次在访达右键图标选「打开」确认一次',
    ],
  },
  'windows-x86_64': {
    id: 'windows-x86_64',
    tab: 'Windows',
    short: 'Windows 64 位',
    ext: '.exe',
    req: 'Windows 10 / 11 · 免管理员',
    steps: [
      '双击运行下载完成的 .exe 安装包',
      '一键安装到当前用户目录，无 UAC 弹窗',
      '打开即用，新版在「设置 → 关于」内直升',
    ],
  },
  'darwin-x86_64': {
    id: 'darwin-x86_64',
    tab: 'macOS Intel',
    short: 'macOS (Intel)',
    ext: '.dmg',
    req: 'macOS 12+ · Intel',
    steps: [
      '双击打开下载完成的 .dmg 镜像',
      '将「小伴桌面」拖入「应用程序」',
      '首次在访达右键图标选「打开」确认一次',
    ],
  },
}

const HERO_SCENES = [
  {
    tab: '控电脑 & 控手机',
    img: assetUrl('desktop-config-ai.webp'),
    caption: '原生看屏操作电脑与连接手机，自动完成多步工作流',
  },
  {
    tab: '外部协作',
    img: assetUrl('desktop-config-agent.webp'),
    caption: '连接 Claude Code / Codex 协同改画布，开放语音 MCP',
  },
  {
    tab: '3D 特效',
    img: assetUrl('desktop-dragon.webp'),
    caption: '14 种顶层透明特效，光照与鱼群实时可调',
  },
  {
    tab: '实时组件',
    img: assetUrl('desktop-widgets.webp'),
    caption: '股票行情、电子木鱼与农历月历，多屏自由摆放',
  },
]

const LIVE_TABS = [
  { tab: '3D 龙鱼特效', img: assetUrl('desktop-dragon.webp'), caption: '14 种顶层透明特效，AI 也能现做专属特效包' },
  { tab: '8 款实时组件', img: assetUrl('desktop-widgets.webp'), caption: '天气、待办、日程、股票、电子木鱼与节假日月历' },
  { tab: '动静壁纸库', img: assetUrl('desktop-wallpapers.webp'), caption: '4K 高清图库与循环视频壁纸，离线缓存即开即用' },
  { tab: '7 只桌宠', img: assetUrl('desktop-companion.webp'), caption: '视线跟随、点击互动与多屏陪伴，不挡工作视线' },
]

const FAQS = [
  {
    q: 'macOS 首次打开提示“无法验证开发者”怎么办？',
    a: '拖入「应用程序」后，在访达中右键点击「小伴桌面」图标选择「打开」确认一次，之后即可正常双击启动。',
  },
  {
    q: 'Windows 安装需要管理员权限吗？',
    a: '不需要。默认安装到当前用户目录，无管理员弹窗、卸载干净无残留（需 WebView2，Win11 自带）。',
  },
  {
    q: '不登录账号也能用吗？后续如何更新？',
    a: '可以。壁纸、特效、组件与桌宠均可直接体验；新版在「设置 → 关于」内直接升级。',
  },
]

function detectDesktopTarget(): DesktopTargetId {
  const override = (routeQuery.value.os || '').toLowerCase()
  if (override === 'win' || override === 'windows') return 'windows-x86_64'
  if (override === 'mac' || override === 'macos') return 'darwin-aarch64'
  if (override === 'intel') return 'darwin-x86_64'
  if (typeof navigator === 'undefined') return 'darwin-aarch64'
  const ua = navigator.userAgent || ''
  if (/Windows/i.test(ua)) return 'windows-x86_64'
  return 'darwin-aarch64'
}

function detectIsMobileDevice(): boolean {
  if (typeof navigator === 'undefined') return false
  return /Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent || '')
}

const detectedId = ref<DesktopTargetId>(detectDesktopTarget())
const selectedId = ref<DesktopTargetId>(detectedId.value)
const isMobileUa = ref(detectIsMobileDevice())
const dark = ref(false)
const loading = ref(true)
const error = ref('')
const downloads = ref<Record<string, DesktopProductDownload>>({})
const heroSceneIdx = ref(0)
const liveTabIdx = ref(0)
const openFaq = ref<number | null>(0)
const sheetMode = ref<'none' | 'download' | 'changelog'>('none')
const pastHero = ref(false)

const visibleTargets = computed<TargetMeta[]>(() => {
  const list: TargetMeta[] = [TARGET_METAS['windows-x86_64'], TARGET_METAS['darwin-aarch64']]
  if (downloads.value['darwin-x86_64']) {
    list.push(TARGET_METAS['darwin-x86_64'])
  }
  return list
})

const activeMeta = computed(() => TARGET_METAS[selectedId.value] ?? TARGET_METAS['darwin-aarch64'])
const activeDownload = computed<DesktopProductDownload | null>(() => downloads.value[selectedId.value] ?? null)
const alternateMeta = computed(() =>
  selectedId.value === 'windows-x86_64' ? TARGET_METAS['darwin-aarch64'] : TARGET_METAS['windows-x86_64'],
)
const activeHeroScene = computed(() => HERO_SCENES[heroSceneIdx.value] ?? HERO_SCENES[0])
const activeLiveTab = computed(() => LIVE_TABS[liveTabIdx.value] ?? LIVE_TABS[0])

const shareUrl = computed(() => {
  if (typeof window === 'undefined') return 'https://xiaban.xyz/desktop'
  const u = new URL(window.location.href)
  u.searchParams.delete('os')
  u.searchParams.delete('theme')
  return u.toString()
})

function formatBytes(bytes?: number): string {
  if (!bytes || bytes <= 0) return ''
  const mb = bytes / (1024 * 1024)
  return mb >= 100 ? `${mb.toFixed(0)} MB` : `${mb.toFixed(1)} MB`
}

function formatDate(iso?: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

function noteLines(notes?: string): string[] {
  const raw = (notes || '').trim()
  if (!raw) {
    return [
      '支持原生控电脑屏幕与 ADB 控手机，独立多模型配置窗',
      '外部 Agent 协作接入 Claude Code / Codex，开放本机语音 MCP',
      '14 种 3D 透明特效、8 款多屏实时组件与 7 只感知桌宠',
    ]
  }
  return raw
    .split(/\r?\n/)
    .map(line => line.replace(/^[-*•·]\s*/, '').trim())
    .filter(Boolean)
}

function triggerFileDownload(dl: DesktopProductDownload) {
  const a = document.createElement('a')
  a.href = dl.url
  a.rel = 'noopener'
  document.body.appendChild(a)
  a.click()
  a.remove()
}

function onPrimaryDownload() {
  if (error.value) {
    load()
    return
  }
  if (!activeDownload.value) {
    toast('该平台安装包即将发布')
    return
  }
  triggerFileDownload(activeDownload.value)
  sheetMode.value = 'download'
}

function onRetryDownload() {
  if (!activeDownload.value) return
  triggerFileDownload(activeDownload.value)
  toast(`已重新触发 ${activeMeta.value.tab} 安装包下载`)
}

async function copyText(text: string, okMsg: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast(okMsg)
  } catch {
    toast('复制失败，请手动复制')
  }
}

function toggleFaq(idx: number) {
  openFaq.value = openFaq.value === idx ? null : idx
}

function toggleTheme() {
  dark.value = !dark.value
}

function onScroll() {
  pastHero.value = window.scrollY > 360
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') sheetMode.value = 'none'
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const product = await getDesktopProduct()
    downloads.value = product.downloads || {}
    if (!downloads.value[selectedId.value]) {
      const fallbackTargets: DesktopTargetId[] = ['darwin-aarch64', 'windows-x86_64', 'darwin-x86_64']
      const fallbackId = fallbackTargets.find(id => downloads.value[id])
      if (fallbackId) selectedId.value = fallbackId
    }
  } catch {
    error.value = '版本信息加载失败，请重试'
  } finally {
    loading.value = false
  }
}

let prevTitle = ''

onMounted(() => {
  prevTitle = document.title
  document.title = '小伴桌面 · 官方下载 | xiaban.xyz'
  const themeQ = (routeQuery.value.theme || '').toLowerCase()
  if (themeQ === 'dark') {
    dark.value = true
  } else if (themeQ === 'light') {
    dark.value = false
  } else if (typeof window !== 'undefined' && window.matchMedia) {
    dark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  load()
  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  if (prevTitle) document.title = prevTitle
  window.removeEventListener('scroll', onScroll)
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div class="d-dl-page" :class="{ dark }">
    <!-- 顶部极简独立栏 -->
    <header class="lp-topbar">
      <div class="lp-topbar-in">
        <a href="/" class="lp-brand" title="返回小伴产品矩阵" @click.prevent="navigate('/')">
          <span class="lp-brand-icon">
            <img :src="BRAND_ICON_URL" alt="小伴桌面">
          </span>
          <span class="lp-brand-name">小伴桌面</span>
          <span v-if="activeDownload" class="lp-ver num">v{{ activeDownload.version }}</span>
        </a>
        <div class="lp-top-right">
          <a href="/app" class="lp-link-chip" @click.prevent="navigate('/app')">手机版</a>
          <button type="button" class="lp-icon-btn" :title="dark ? '切换暖瓷浅色' : '切换曜石深色'" @click="toggleTheme">
            <svg v-if="dark" width="17" height="17" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" /></svg>
            <svg v-else width="17" height="17" viewBox="0 0 24 24"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" /></svg>
          </button>
          <button type="button" class="lp-icon-btn" title="复制页面链接" @click="copyText(shareUrl, '已复制下载页链接')">
            <svg width="17" height="17" viewBox="0 0 24 24"><path d="M4 12v7a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7" /><polyline points="16 6 12 2 8 6" /><line x1="12" y1="2" x2="12" y2="15" /></svg>
          </button>
          <button type="button" class="lp-mini-dl" :class="{ coral: pastHero }" @click="onPrimaryDownload">
            立即下载
          </button>
        </div>
      </div>
    </header>

    <main class="lp-main">
      <!-- 手机访问提示条 -->
      <div v-if="isMobileUa" class="mobile-notice">
        <span>正在手机浏览，可复制链接到电脑下载</span>
        <div class="mobile-notice-acts">
          <button type="button" class="mn-btn" @click="copyText(shareUrl, '已复制电脑版链接')">复制链接</button>
          <a href="/app" class="mn-btn primary" @click.prevent="navigate('/app')">下手机版</a>
        </div>
      </div>

      <!-- ==================== 首屏双栏下载展台 ==================== -->
      <section class="hero-stage" aria-label="小伴桌面下载">
        <div class="hero-left">
          <div class="hero-brand-row">
            <div class="hero-icon">
              <img :src="BRAND_ICON_URL" alt="小伴双星图标">
            </div>
            <div>
              <div class="hero-brand-title">小伴桌面</div>
              <div class="hero-brand-sub">{{ activeMeta.req }}</div>
            </div>
          </div>

          <h1>你的 AI 工作台<br>与<span class="grad-text">活桌面</span></h1>
          <p class="hero-sub">控电脑与手机 · 无限画布 · 3D 特效与组件 · 桌宠壁纸</p>

          <!-- OS 切换器 -->
          <div class="os-seg" role="tablist" aria-label="选择电脑系统">
            <button
              v-for="t in visibleTargets"
              :key="t.id"
              type="button"
              role="tab"
              class="os-tab"
              :class="{ active: selectedId === t.id }"
              :aria-selected="selectedId === t.id"
              @click="selectedId = t.id"
            >
              <svg v-if="t.id.startsWith('darwin')" width="15" height="15" viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true"><path d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.7-2.8-.7-1.4 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7 1.3 0 1.7.7 2.8.7 1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.4-.1 0-2.5-.9-2.5-3.5zM14.4 6.3c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 2-.5 2.5-1.2z" /></svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true"><path d="M3 5.5 10.6 4.4v7.1H3V5.5zm8.7-1.3L21 3v8.5h-9.3V4.2zM3 12.6h7.6v7.1L3 18.6v-6zm8.7 0H21V21l-9.3-1.3v-7.1z" /></svg>
              <span>{{ t.tab }}</span>
              <span v-if="t.id === detectedId" class="rec-dot" title="本机推荐" />
            </button>
          </div>

          <!-- 主下载区 -->
          <div v-if="error" class="err-box">
            <span>{{ error }}</span>
            <button type="button" class="err-retry" @click="load">重试</button>
          </div>
          <div v-else class="cta-row">
            <button
              type="button"
              class="btn-primary-dl"
              :disabled="loading || !activeDownload"
              @click="onPrimaryDownload"
            >
              <svg width="18" height="18" viewBox="0 0 24 24"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
              <span>{{ loading ? '正在获取版本…' : activeDownload ? `下载 ${activeMeta.tab} 版` : `${activeMeta.tab} 版即将发布` }}</span>
              <span v-if="activeDownload" class="btn-dl-meta num">
                <template v-if="formatBytes(activeDownload.bytes)">{{ formatBytes(activeDownload.bytes) }} · </template>{{ activeMeta.ext }}
              </span>
            </button>
            <button
              v-if="activeDownload"
              type="button"
              class="btn-soft"
              @click="sheetMode = 'changelog'"
            >
              更新日志
            </button>
          </div>

          <!-- 信任条 + 另一平台快切 -->
          <div class="trust-row">
            <span class="trust-item"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>官方签名</span>
            <span>·</span>
            <span class="trust-item"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>零广告</span>
            <button
              v-if="activeDownload?.sha256"
              type="button"
              class="sha-inline mono"
              @click="copyText(activeDownload.sha256, '已复制 SHA-256')"
            >
              SHA-256 复制
            </button>
          </div>

          <div class="alt-row">
            <button type="button" class="alt-link" @click="selectedId = alternateMeta.id">
              切换 {{ alternateMeta.short }}
            </button>
            <span aria-hidden="true">·</span>
            <a href="/app" class="alt-link" @click.prevent="navigate('/app')">需要手机版？前往下载 ↗</a>
          </div>
        </div>

        <!-- 右侧实机 4 场景切换展台 -->
        <div class="hero-right">
          <div class="showcase-tabs" role="tablist">
            <button
              v-for="(s, idx) in HERO_SCENES"
              :key="s.tab"
              type="button"
              role="tab"
              class="showcase-tab"
              :class="{ active: idx === heroSceneIdx }"
              :aria-selected="idx === heroSceneIdx"
              @click="heroSceneIdx = idx"
            >
              {{ s.tab }}
            </button>
          </div>
          <div class="desk-frame">
            <img :src="activeHeroScene.img" :alt="activeHeroScene.tab">
          </div>
          <div class="showcase-caption">
            <strong>{{ activeHeroScene.caption }}</strong>
          </div>
        </div>
      </section>

      <!-- ==================== 01 AI 自主工作台 ==================== -->
      <section class="sec">
        <div class="sec-head">
          <h2>AI 自主工作台</h2>
        </div>
        <div class="bento-2">
          <article class="feature-card">
            <div class="fc-shot">
              <img :src="assetUrl('desktop-config-ai.webp')" alt="控电脑与多模型" loading="lazy">
            </div>
            <div class="fc-body">
              <div class="fc-title-row">
                <span class="fc-ico coral">
                  <svg width="16" height="16" viewBox="0 0 24 24"><rect x="2" y="3" width="20" height="14" rx="2" /><line x1="8" y1="21" x2="16" y2="21" /><line x1="12" y1="17" x2="12" y2="21" /></svg>
                </span>
                <h3>控电脑与 ADB 控手机</h3>
              </div>
              <p>原生看屏点击、输入与多步自动化操作，复杂重复任务交给 AI 自主推进。</p>
            </div>
          </article>

          <article class="feature-card">
            <div class="fc-shot">
              <img :src="assetUrl('desktop-plugins-v318.webp')" alt="专家市场与外部协作" loading="lazy">
            </div>
            <div class="fc-body">
              <div class="fc-title-row">
                <span class="fc-ico green">
                  <svg width="16" height="16" viewBox="0 0 24 24"><path d="M12 3l1.9 5.8L20 11l-6.1 2.2L12 19l-1.9-5.8L4 11l6.1-2.2Z" /></svg>
                </span>
                <h3>专家市场与外部 Agent 协作</h3>
              </div>
              <p>一键安装 54+ 专家、技能与连接器，协同 Claude Code / Codex 改画布并直出 PPTX / Figma。</p>
            </div>
          </article>
        </div>
      </section>

      <!-- ==================== 02 活桌面深色展台 ==================== -->
      <section class="sec">
        <div class="sec-head">
          <h2>让桌面活起来</h2>
        </div>
        <div class="live-stage-card">
          <div class="live-glow" aria-hidden="true" />
          <div class="live-top">
            <h3>3D 透明特效、实时组件、动态壁纸与 7 只桌宠</h3>
            <div class="live-tabs" role="tablist">
              <button
                v-for="(t, idx) in LIVE_TABS"
                :key="t.tab"
                type="button"
                role="tab"
                class="live-tab"
                :class="{ active: idx === liveTabIdx }"
                :aria-selected="idx === liveTabIdx"
                @click="liveTabIdx = idx"
              >
                {{ t.tab }}
              </button>
            </div>
          </div>
          <div class="live-frame">
            <img :src="activeLiveTab.img" :alt="activeLiveTab.tab" loading="lazy">
          </div>
          <p class="live-caption">{{ activeLiveTab.caption }}</p>
          <a
            class="desktop-case"
            href="https://chat.xiangban.xyz/?tab=gallery&amp;exhibit=arowana"
            target="_top"
            rel="noreferrer"
            aria-labelledby="desktop-case-title desktop-case-action"
            aria-describedby="desktop-case-description"
          >
            <div>
              <h4 id="desktop-case-title">龙鱼巡游</h4>
              <p id="desktop-case-description">让 3D 朱红龙鱼游过桌面，也可以和孔雀鱼同游。</p>
            </div>
            <span id="desktop-case-action" class="desktop-case-action">查看社区展品 <span aria-hidden="true">↗</span></span>
          </a>
        </div>
      </section>

      <!-- ==================== 03 创作空间与超级小伴 ==================== -->
      <section class="sec">
        <div class="sec-head">
          <h2>创作空间与超级小伴</h2>
        </div>
        <div class="bento-2">
          <article class="feature-card">
            <div class="fc-shot">
              <img :src="assetUrl('desktop-apps.webp')" alt="内置游戏与作品空间" loading="lazy">
            </div>
            <div class="fc-body">
              <h3>内置游戏与作品空间</h3>
              <p>零广告离线畅玩官方小游戏，或让 AI 现做壁纸、特效、组件与工具/游戏。</p>
            </div>
          </article>

          <article class="feature-card">
            <div class="fc-shot">
              <img :src="assetUrl('assistant.webp')" alt="超级小伴悬浮助手" loading="lazy">
            </div>
            <div class="fc-body">
              <h3>超级小伴悬浮助手</h3>
              <p>全局快捷键随时呼出小球与胶囊，支持区域截图标注、语音快问与多轮对话。</p>
            </div>
          </article>
        </div>
      </section>

      <!-- ==================== 04 常见疑问与全端生态 ==================== -->
      <section class="sec">
        <div class="sec-head">
          <h2>常见疑问</h2>
        </div>

        <div class="faq-box">
          <div
            v-for="(item, idx) in FAQS"
            :key="item.q"
            class="faq-item"
            :class="{ open: openFaq === idx }"
          >
            <button type="button" class="faq-q" :aria-expanded="openFaq === idx" @click="toggleFaq(idx)">
              <span>{{ item.q }}</span>
              <svg viewBox="0 0 24 24"><path d="m6 9 6 6 6-6" /></svg>
            </button>
            <div v-show="openFaq === idx" class="faq-a">{{ item.a }}</div>
          </div>
        </div>

        <div class="eco-card">
          <div class="eco-title">小伴全端生态</div>
          <div class="eco-grid">
            <a class="eco-link" href="/app" @click.prevent="navigate('/app')">
              <strong>小伴手机版</strong>
              <span>xiaban.xyz/app</span>
            </a>
            <a class="eco-link" href="https://wallpaper.xiangban.xyz" target="_blank" rel="noopener noreferrer">
              <strong>小伴壁纸站</strong>
              <span>wallpaper.xiangban.xyz</span>
            </a>
            <a class="eco-link" href="https://chat.xiangban.xyz" target="_blank" rel="noopener noreferrer">
              <strong>超级小伴社区</strong>
              <span>chat.xiangban.xyz</span>
            </a>
            <a class="eco-link" href="https://game.xiaban.xyz" target="_blank" rel="noopener noreferrer">
              <strong>小伴游戏厅</strong>
              <span>game.xiaban.xyz</span>
            </a>
          </div>
        </div>

        <footer class="lp-foot num">
          <template v-if="activeDownload">v{{ activeDownload.version }} · </template>macOS / Windows
        </footer>
      </section>
    </main>

    <!-- 下载指引 / 更新日志弹窗 -->
    <Teleport to="body">
      <div
        v-if="sheetMode !== 'none' && activeDownload"
        class="d-dl-overlay"
        :class="{ dark }"
        @click.self="sheetMode = 'none'"
      >
        <div class="sheet" role="dialog" aria-modal="true" :aria-label="sheetMode === 'download' ? '已开始下载' : '更新内容'">
          <div class="sheet-handle" />
          <div class="sheet-head">
            <div class="sheet-title-row">
              <span v-if="sheetMode === 'download'" class="fc-ico green sm">
                <svg width="15" height="15" viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>
              </span>
              <h3>{{ sheetMode === 'download' ? `已开始下载 ${activeMeta.tab} 版` : `v${activeDownload.version} 更新内容` }}</h3>
            </div>
            <button type="button" class="sheet-close" aria-label="关闭" @click="sheetMode = 'none'">✕</button>
          </div>

          <div v-if="sheetMode === 'download'" class="steps-list">
            <div v-for="(step, idx) in activeMeta.steps" :key="step" class="step-row">
              <span class="step-idx">{{ idx + 1 }}</span>
              <strong>{{ step }}</strong>
            </div>
          </div>
          <div v-else class="sheet-meta num">
            {{ activeMeta.short }} · {{ formatDate(activeDownload.publishedAt) }}
            <template v-if="formatBytes(activeDownload.bytes)"> · {{ formatBytes(activeDownload.bytes) }}</template>
          </div>

          <div class="sheet-changelog-head">
            <span>v{{ activeDownload.version }} 更新内容</span>
            <span class="num">{{ formatDate(activeDownload.publishedAt) }}</span>
          </div>
          <ul class="changelog-ul">
            <li v-for="(line, i) in noteLines(activeDownload.notes)" :key="i">{{ line }}</li>
          </ul>

          <div v-if="activeDownload.sha256" class="sha-box">
            <div class="sha-code mono">SHA-256: {{ activeDownload.sha256.slice(0, 18) }}…{{ activeDownload.sha256.slice(-10) }}</div>
            <button type="button" class="sha-copy" @click="copyText(activeDownload.sha256, '已复制 SHA-256')">复制</button>
          </div>

          <button
            type="button"
            class="btn-primary-dl compact"
            @click="sheetMode === 'download' ? onRetryDownload() : (triggerFileDownload(activeDownload), sheetMode = 'download')"
          >
            {{ sheetMode === 'download' ? '重新下载' : `立即下载 ${activeMeta.tab} 版` }}
            <template v-if="formatBytes(activeDownload.bytes)">（{{ formatBytes(activeDownload.bytes) }}）</template>
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.d-dl-page {
  --canvas: #fcfaf7;
  --canvas-soft: #f5f1eb;
  --surface: #ffffff;
  --surface-elev: #ffffff;
  --ink: #19171a;
  --ink-2: #524d54;
  --ink-3: #837d86;
  --line: rgba(28, 22, 28, 0.075);
  --line-strong: rgba(28, 22, 28, 0.14);
  --fill: #f2eee8;
  --coral: #ee4f6d;
  --coral-grad: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  --coral-soft: #fff0f3;
  --coral-ink: #d83b5a;
  --coral-shadow: 0 12px 28px rgba(242, 92, 114, 0.28);
  --ok: #169c57;
  --ok-soft: #eaf9f0;
  --glass: rgba(252, 250, 247, 0.86);
  --card-shadow: 0 2px 10px rgba(25, 18, 24, 0.04);
  --ease: cubic-bezier(0.22, 1, 0.36, 1);
  --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;

  min-height: 100vh;
  min-height: 100dvh;
  color: var(--ink);
  color-scheme: light;
  background:
    radial-gradient(75% 380px at 50% -70px, rgba(255, 107, 134, 0.16), transparent 70%),
    radial-gradient(50% 300px at 88% 120px, rgba(249, 144, 86, 0.12), transparent 66%),
    var(--canvas);
}

.d-dl-page.dark {
  --canvas: #111114;
  --canvas-soft: #19191f;
  --surface: #1e1e24;
  --surface-elev: #26262f;
  --ink: #f4f3f6;
  --ink-2: #b4b0ba;
  --ink-3: #817d8a;
  --line: rgba(255, 255, 255, 0.085);
  --line-strong: rgba(255, 255, 255, 0.15);
  --fill: #282730;
  --coral-soft: rgba(255, 91, 125, 0.16);
  --coral-ink: #ff839c;
  --coral-shadow: 0 12px 30px rgba(242, 92, 114, 0.34);
  --ok: #34d399;
  --ok-soft: rgba(52, 211, 153, 0.14);
  --glass: rgba(17, 17, 20, 0.86);
  --card-shadow: 0 2px 10px rgba(0, 0, 0, 0.22);

  color-scheme: dark;
  background:
    radial-gradient(75% 380px at 50% -70px, rgba(255, 91, 125, 0.18), transparent 72%),
    radial-gradient(50% 300px at 88% 130px, rgba(249, 116, 75, 0.11), transparent 66%),
    var(--canvas);
}

.d-dl-page svg {
  flex: none;
  stroke-width: 1.85;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.d-dl-page svg:not([fill]) {
  fill: none;
  stroke: currentColor;
}
.num { font-variant-numeric: tabular-nums; }
.mono { font-family: var(--mono); }
.grad-text {
  background: var(--coral-grad);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

/* ==================== 顶栏 ==================== */
.lp-topbar {
  position: sticky;
  top: 0;
  z-index: 30;
  background: var(--glass);
  backdrop-filter: blur(18px) saturate(1.4);
  -webkit-backdrop-filter: blur(18px) saturate(1.4);
  border-bottom: 1px solid var(--line);
  padding-top: env(safe-area-inset-top, 0px);
}
.lp-topbar-in {
  max-width: 1080px;
  margin: 0 auto;
  height: 54px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.lp-brand { display: flex; align-items: center; gap: 9px; min-width: 0; }
.lp-brand-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  flex: none;
  overflow: hidden;
  background: var(--coral-grad);
  box-shadow: 0 3px 10px rgba(238, 79, 109, 0.25);
}
.lp-brand-icon img { width: 100%; height: 100%; object-fit: cover; }
.lp-brand-name { font-size: 15px; font-weight: 740; letter-spacing: -0.02em; white-space: nowrap; }
.lp-ver {
  font-size: 10.5px;
  font-weight: 650;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--fill);
  color: var(--ink-2);
  white-space: nowrap;
  flex: none;
}
.lp-top-right { display: flex; align-items: center; gap: 6px; flex: none; }
.lp-link-chip {
  height: 30px;
  padding: 0 11px;
  border-radius: 999px;
  background: var(--fill);
  color: var(--ink-2);
  font-size: 12px;
  font-weight: 620;
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
  flex: none;
  transition: color 0.15s;
}
.lp-link-chip:hover { color: var(--ink); }
.lp-icon-btn {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  color: var(--ink-2);
  transition: background 0.15s, color 0.15s;
}
.lp-icon-btn:hover { background: var(--fill); color: var(--ink); }
.lp-mini-dl {
  height: 32px;
  padding: 0 14px;
  border-radius: 999px;
  background: var(--ink);
  color: var(--canvas);
  font-size: 12px;
  font-weight: 660;
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
  flex: none;
  transition: transform 0.14s, background 0.2s;
}
.lp-mini-dl.coral { background: var(--coral-grad); color: #fff; }
.lp-mini-dl:active { transform: scale(0.96); }

/* ==================== 主体 ==================== */
.lp-main {
  max-width: 1080px;
  margin: 0 auto;
  padding: 28px 20px 72px;
}

.mobile-notice {
  margin-bottom: 16px;
  padding: 11px 14px;
  border-radius: 14px;
  background: var(--surface);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
  font-size: 12.5px;
  color: var(--ink-2);
}
.mobile-notice-acts { display: flex; gap: 6px; }
.mn-btn {
  height: 28px;
  padding: 0 10px;
  border-radius: 999px;
  background: var(--fill);
  color: var(--ink);
  font-size: 11.5px;
  font-weight: 650;
  display: inline-flex;
  align-items: center;
}
.mn-btn.primary { background: var(--coral-grad); color: #fff; }

/* ==================== 首屏双栏展台 ==================== */
.hero-stage {
  border-radius: 28px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--card-shadow);
  padding: 24px;
  display: grid;
  grid-template-columns: 0.94fr 1.06fr;
  gap: 24px;
  align-items: center;
}
.hero-left { padding: 8px 6px 8px 10px; }
.hero-brand-row { display: flex; align-items: center; gap: 12px; }
.hero-icon {
  width: 50px;
  height: 50px;
  border-radius: 14px;
  overflow: hidden;
  background: var(--coral-grad);
  box-shadow: 0 10px 24px rgba(242, 92, 114, 0.28);
  flex: none;
}
.hero-icon img { width: 100%; height: 100%; object-fit: cover; }
.hero-brand-title { font-size: 16px; font-weight: 760; }
.hero-brand-sub { font-size: 12px; color: var(--ink-3); margin-top: 2px; }

.hero-left h1 {
  margin-top: 18px;
  font-size: clamp(32px, 3.8vw, 44px);
  line-height: 1.14;
  letter-spacing: -0.04em;
  font-weight: 780;
  color: var(--ink);
}
.hero-sub {
  margin-top: 11px;
  font-size: 14.5px;
  color: var(--ink-2);
}

.os-seg {
  margin-top: 20px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px;
  border-radius: 13px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
}
.os-tab {
  height: 36px;
  padding: 0 14px;
  border-radius: 9px;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 13px;
  font-weight: 620;
  color: var(--ink-2);
  transition: background 0.16s, color 0.16s, box-shadow 0.16s;
}
.os-tab:hover { color: var(--ink); }
.os-tab.active {
  background: var(--surface-elev);
  color: var(--ink);
  font-weight: 700;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
.rec-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--coral-grad);
}

.cta-row {
  margin-top: 18px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}
.btn-primary-dl {
  height: 50px;
  padding: 0 24px;
  border-radius: 15px;
  background: var(--coral-grad);
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 9px;
  font-size: 15px;
  font-weight: 730;
  box-shadow: var(--coral-shadow);
  transition: transform 0.14s var(--ease), filter 0.16s;
  white-space: nowrap;
}
.btn-primary-dl:hover { filter: brightness(1.04); transform: translateY(-1px); }
.btn-primary-dl:active { transform: scale(0.985); }
.btn-primary-dl:disabled { opacity: 0.62; cursor: not-allowed; transform: none; }
.btn-dl-meta {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.9;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.35);
}
.btn-soft {
  height: 50px;
  padding: 0 18px;
  border-radius: 15px;
  background: var(--canvas-soft);
  border: 1px solid var(--line-strong);
  color: var(--ink);
  font-size: 14px;
  font-weight: 650;
  transition: background 0.15s;
}
.btn-soft:hover { background: var(--fill); }

.err-box {
  margin-top: 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 14px;
  background: var(--canvas-soft);
  border: 1px solid var(--line-strong);
  font-size: 13px;
  color: var(--ink-2);
}
.err-retry {
  height: 30px;
  padding: 0 13px;
  border-radius: 999px;
  background: var(--coral-grad);
  color: #fff;
  font-size: 12px;
  font-weight: 660;
}

.trust-row {
  margin-top: 14px;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 10px;
  font-size: 12px;
  color: var(--ink-3);
}
.trust-item { display: inline-flex; align-items: center; gap: 4px; }
.trust-item svg { width: 13px; height: 13px; color: var(--ok); }
.sha-inline {
  padding: 2px 8px;
  border-radius: 7px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  font-size: 11px;
  color: var(--ink-2);
}
.sha-inline:hover { border-color: var(--line-strong); color: var(--ink); }

.alt-row {
  margin-top: 14px;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12.5px;
  color: var(--ink-3);
}
.alt-link { color: var(--ink-2); font-weight: 600; transition: color 0.15s; }
.alt-link:hover { color: var(--coral-ink); }

/* 右侧桌面展台 */
.hero-right {
  border-radius: 22px;
  padding: 14px;
  background:
    radial-gradient(85% 90% at 88% 8%, rgba(255, 107, 134, 0.18), transparent 60%),
    radial-gradient(75% 80% at 10% 95%, rgba(249, 144, 86, 0.15), transparent 60%),
    var(--canvas-soft);
  border: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.showcase-tabs {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
  padding: 3px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.65);
}
.d-dl-page.dark .showcase-tabs { background: rgba(17, 17, 20, 0.55); }
.showcase-tab {
  height: 30px;
  border-radius: 9px;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ink-2);
  white-space: nowrap;
  transition: background 0.16s, color 0.16s, box-shadow 0.16s;
}
.showcase-tab.active {
  background: var(--surface-elev);
  color: var(--ink);
  font-weight: 700;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
}
.desk-frame {
  border-radius: 14px;
  overflow: hidden;
  aspect-ratio: 16 / 10;
  background: #141418;
  border: 1px solid rgba(25, 23, 26, 0.1);
  box-shadow: 0 14px 30px rgba(24, 18, 24, 0.14);
}
.d-dl-page.dark .desk-frame { border-color: rgba(255, 255, 255, 0.1); box-shadow: 0 18px 36px rgba(0, 0, 0, 0.48); }
.desk-frame img { width: 100%; height: 100%; object-fit: cover; object-position: top; }
.showcase-caption {
  padding: 9px 12px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.86);
  border: 1px solid rgba(25, 23, 26, 0.06);
  text-align: center;
}
.d-dl-page.dark .showcase-caption {
  background: rgba(28, 28, 35, 0.86);
  border-color: rgba(255, 255, 255, 0.08);
}
.showcase-caption strong { font-size: 12.5px; font-weight: 680; color: var(--ink); }

/* ==================== 区块 ==================== */
.sec { margin-top: 34px; }
.sec-head { margin-bottom: 14px; padding: 0 2px; }
.sec-head h2 {
  font-size: 21px;
  font-weight: 760;
  letter-spacing: -0.025em;
  color: var(--ink);
}

.bento-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.feature-card {
  border-radius: 22px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--card-shadow);
  overflow: hidden;
}
.fc-shot {
  aspect-ratio: 16 / 9.8;
  overflow: hidden;
  background: var(--canvas-soft);
  border-bottom: 1px solid var(--line);
}
.fc-shot img { width: 100%; height: 100%; object-fit: cover; object-position: top; }
.fc-body { padding: 16px 18px 18px; }
.fc-title-row { display: flex; align-items: center; gap: 9px; }
.fc-ico {
  width: 30px;
  height: 30px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  flex: none;
}
.fc-ico.coral { background: var(--coral-soft); color: var(--coral-ink); }
.fc-ico.green { background: var(--ok-soft); color: var(--ok); }
.fc-body h3 { font-size: 16px; font-weight: 740; letter-spacing: -0.015em; }
.fc-body p { margin-top: 6px; font-size: 13px; color: var(--ink-2); line-height: 1.62; }

/* 活桌面深色展台 */
.live-stage-card {
  border-radius: 24px;
  padding: 20px;
  position: relative;
  overflow: hidden;
  background: linear-gradient(155deg, #1d1b22 0%, #141318 100%);
  color: #f5f4f8;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 18px 42px rgba(20, 16, 24, 0.18);
}
.d-dl-page.dark .live-stage-card {
  background: linear-gradient(155deg, #22212b 0%, #18181f 100%);
  border-color: rgba(255, 255, 255, 0.11);
}
.live-glow {
  position: absolute;
  top: -70px;
  right: -50px;
  width: 240px;
  height: 240px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 91, 125, 0.28), transparent 68%);
  pointer-events: none;
}
.live-top {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}
.live-top h3 { font-size: 17px; font-weight: 740; letter-spacing: -0.015em; }
.live-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 3px;
  border-radius: 11px;
  background: rgba(255, 255, 255, 0.07);
}
.live-tab {
  height: 30px;
  padding: 0 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  color: rgba(245, 244, 248, 0.72);
  transition: background 0.15s, color 0.15s;
}
.live-tab.active {
  background: #ffffff;
  color: #19171a;
  font-weight: 700;
}
.live-frame {
  position: relative;
  z-index: 1;
  border-radius: 16px;
  overflow: hidden;
  aspect-ratio: 16 / 9.2;
  background: #0d0d11;
  border: 1px solid rgba(255, 255, 255, 0.1);
}
.live-frame img { width: 100%; height: 100%; object-fit: cover; }
.live-caption {
  position: relative;
  z-index: 1;
  margin-top: 12px;
  font-size: 13px;
  color: rgba(245, 244, 248, 0.78);
  text-align: center;
}
.desktop-case {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px 24px;
  margin-top: 16px;
  padding: 14px 2px 0;
  border-top: 1px solid rgba(245, 244, 248, 0.2);
}
.desktop-case h4 { font-size: 15px; }
.desktop-case p {
  margin-top: 3px;
  font-size: 13px;
  color: rgba(245, 244, 248, 0.8);
}
.desktop-case-action {
  color: #ffd0dc;
  font-size: 13px;
  font-weight: 650;
  white-space: nowrap;
}
.desktop-case:hover .desktop-case-action { text-decoration: underline; text-underline-offset: 3px; }
.desktop-case:focus-visible { outline: 2px solid #ffd0dc; outline-offset: 4px; }

/* FAQ & 生态 */
.faq-box {
  border-radius: 20px;
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
  box-shadow: var(--card-shadow);
}
.faq-item { border-bottom: 1px solid var(--line); }
.faq-item:last-child { border-bottom: 0; }
.faq-q {
  width: 100%;
  padding: 15px 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 14px;
  font-weight: 660;
  text-align: left;
  color: var(--ink);
}
.faq-q svg { width: 16px; height: 16px; color: var(--ink-3); transition: transform 0.2s var(--ease); }
.faq-item.open .faq-q svg { transform: rotate(180deg); }
.faq-a { padding: 0 18px 15px; font-size: 13px; color: var(--ink-2); line-height: 1.65; }

.eco-card {
  margin-top: 16px;
  border-radius: 20px;
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 16px;
  box-shadow: var(--card-shadow);
}
.eco-title { font-size: 12.5px; font-weight: 700; color: var(--ink-2); margin-bottom: 10px; }
.eco-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
.eco-link {
  padding: 11px 12px;
  border-radius: 13px;
  background: var(--canvas-soft);
  border: 1px solid transparent;
  transition: border-color 0.15s, background 0.15s;
}
.eco-link:hover { border-color: var(--line-strong); background: var(--fill); }
.eco-link strong { display: block; font-size: 13px; font-weight: 700; }
.eco-link span { display: block; font-size: 11.5px; color: var(--ink-3); margin-top: 2px; }

.lp-foot {
  margin-top: 20px;
  text-align: center;
  font-size: 12px;
  color: var(--ink-3);
}

@media (max-width: 860px) {
  .hero-stage { grid-template-columns: 1fr; padding: 18px; gap: 20px; }
  .hero-left { padding: 4px 2px 0; }
  .bento-2 { grid-template-columns: 1fr; }
  .eco-grid { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 540px) {
  .lp-main { padding: 16px 14px 56px; }
  .lp-topbar-in { padding: 0 14px; }
  .lp-link-chip { display: none; }
  .cta-row { flex-direction: column; align-items: stretch; }
  .btn-primary-dl, .btn-soft { width: 100%; }
  .live-tabs { width: 100%; display: grid; grid-template-columns: repeat(2, 1fr); }
}
</style>

<style>
.d-dl-overlay {
  --canvas-soft: #f5f1eb;
  --surface: #ffffff;
  --ink: #19171a;
  --ink-2: #524d54;
  --ink-3: #837d86;
  --line: rgba(28, 22, 28, 0.08);
  --line-strong: rgba(28, 22, 28, 0.15);
  --fill: #f2eee8;
  --coral: #ee4f6d;
  --coral-soft: #fff0f3;
  --coral-ink: #d83b5a;
  --ok: #169c57;
  --ok-soft: #eaf9f0;

  position: fixed;
  inset: 0;
  z-index: 200;
  background: rgba(15, 13, 18, 0.58);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}
.d-dl-overlay.dark {
  --canvas-soft: #19191f;
  --surface: #1e1e24;
  --ink: #f4f3f6;
  --ink-2: #b4b0ba;
  --ink-3: #817d8a;
  --line: rgba(255, 255, 255, 0.09);
  --line-strong: rgba(255, 255, 255, 0.16);
  --fill: #282730;
  --coral-soft: rgba(255, 91, 125, 0.16);
  --coral-ink: #ff839c;
  --ok: #34d399;
  --ok-soft: rgba(52, 211, 153, 0.14);
}
@media (min-width: 640px) {
  .d-dl-overlay { align-items: center; padding: 20px; }
}
.d-dl-overlay .sheet {
  width: 100%;
  max-width: 460px;
  max-height: 86dvh;
  overflow-y: auto;
  background: var(--surface);
  color: var(--ink);
  border-radius: 20px 20px 0 0;
  padding: 10px 18px calc(22px + env(safe-area-inset-bottom, 0px));
  box-shadow: 0 -14px 40px rgba(0, 0, 0, 0.22);
}
@media (min-width: 640px) {
  .d-dl-overlay .sheet { border-radius: 24px; padding: 20px 22px 22px; }
  .d-dl-overlay .sheet-handle { display: none; }
}
.d-dl-overlay .sheet-handle {
  width: 36px;
  height: 4px;
  border-radius: 999px;
  background: var(--line-strong);
  margin: 0 auto 14px;
}
.d-dl-overlay .sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}
.d-dl-overlay .sheet-title-row { display: flex; align-items: center; gap: 8px; }
.d-dl-overlay .sheet-head h3 { font-size: 16px; font-weight: 740; }
.d-dl-overlay .sheet-close {
  width: 30px;
  height: 30px;
  border-radius: 999px;
  background: var(--fill);
  color: var(--ink-2);
  display: grid;
  place-items: center;
  font-size: 14px;
}
.d-dl-overlay .fc-ico.green.sm {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  background: var(--ok-soft);
  color: var(--ok);
}
.d-dl-overlay .fc-ico svg { fill: none; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; stroke-linejoin: round; }
.d-dl-overlay .steps-list {
  border-radius: 16px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  padding: 13px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.d-dl-overlay .step-row { display: flex; align-items: center; gap: 10px; }
.d-dl-overlay .step-idx {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  flex: none;
  display: grid;
  place-items: center;
  font-size: 11.5px;
  font-weight: 780;
  background: var(--coral-soft);
  color: var(--coral-ink);
}
.d-dl-overlay .step-row strong { font-size: 13px; font-weight: 700; }
.d-dl-overlay .sheet-meta { font-size: 12px; color: var(--ink-3); }
.d-dl-overlay .sheet-changelog-head {
  margin-top: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 700;
}
.d-dl-overlay .sheet-changelog-head span:last-child { font-weight: 500; color: var(--ink-3); }
.d-dl-overlay .changelog-ul {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 9px 0 14px;
}
.d-dl-overlay .changelog-ul li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 12.5px;
  color: var(--ink-2);
  line-height: 1.55;
}
.d-dl-overlay .changelog-ul li::before {
  content: "";
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--coral);
  margin-top: 8px;
  flex: none;
}
.d-dl-overlay .sha-box {
  padding: 10px 12px;
  border-radius: 12px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.d-dl-overlay .sha-code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  color: var(--ink-2);
  word-break: break-all;
}
.d-dl-overlay .sha-copy {
  height: 28px;
  padding: 0 10px;
  border-radius: 8px;
  flex: none;
  background: var(--surface);
  border: 1px solid var(--line-strong);
  font-size: 11.5px;
  font-weight: 650;
  color: var(--ink);
}
.d-dl-overlay .btn-primary-dl.compact {
  width: 100%;
  height: 46px;
  border-radius: 14px;
  background: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  color: #fff;
  font-size: 14.5px;
  font-weight: 720;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 10px 24px rgba(242, 92, 114, 0.3);
}
</style>
