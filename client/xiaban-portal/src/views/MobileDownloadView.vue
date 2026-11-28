<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { BRAND_ICON_URL, assetUrl } from '../data/products'
import { getDesktopProduct, type DesktopProductDownload } from '../lib/desktop'
import { navigate, routeQuery } from '../lib/router'
import { toast } from '../lib/toast'
import qrcode from '../vendor/qrcode'

type EnvKind = 'android' | 'wechat' | 'ios'

interface ShowcaseScene {
  tab: string
  imgs: [string, string]
  caption: string
}

const SCENES: ShowcaseScene[] = [
  {
    tab: '创造工具/游戏',
    imgs: [assetUrl('mobile-chat-build.webp'), assetUrl('mobile-my-works.webp')],
    caption: '一句话本地创造工具/游戏，当场运行并分享',
  },
  {
    tab: 'AI 对话',
    imgs: [assetUrl('mobile-ai-home.webp'), assetUrl('mobile-ai-chat.webp')],
    caption: '跨话题连续主线对话，生图与并行任务一站搞定',
  },
  {
    tab: '原生社区',
    imgs: [assetUrl('mobile-community-new.webp'), assetUrl('mobile-my-works.webp')],
    caption: '逛原生超级小伴社区，发现创意作品、日报与热榜',
  },
  {
    tab: '我的作品',
    imgs: [assetUrl('mobile-my-works.webp'), assetUrl('mobile-chat-build.webp')],
    caption: '草稿随时续作，一键导出与分享作品海报',
  },
]

const GAMES = [
  { name: '倒杯快乐', icon: assetUrl('plaza/water-sort.webp') },
  { name: '方块爆破', icon: assetUrl('plaza/block-puzzle.webp') },
  { name: '暮色滑翔', icon: assetUrl('plaza/dusk-glider.webp') },
  { name: '水墨剑客', icon: assetUrl('plaza/ink-blade.webp') },
  { name: '魔方大师', icon: assetUrl('plaza/pro-cube.webp') },
  { name: '画框马里奥', icon: assetUrl('plaza/painting-adventure.webp') },
]

const TOOLS = [
  { name: '万能遥控器', icon: assetUrl('plaza/remote-control.webp'), desc: '内置电视空调离线红外码库' },
  { name: '轻启', icon: assetUrl('plaza/quick-launch.webp'), desc: '桌面闪电直达，跳过应用开屏广告' },
  { name: '动态壁纸', icon: assetUrl('plaza/live-wallpapers.webp'), desc: '水下花园与银河，锁屏可分设' },
  { name: 'MBTI 向内看', icon: assetUrl('plaza/mbti.webp'), desc: '48 题六章报告与分享海报' },
]

const FAQS = [
  {
    q: '提示“非应用商店来源”正常吗？',
    a: '正常。官网直装 APK 采用固定证书签名，可核对 SHA-256，后续可在 App 内直接升级。',
  },
  {
    q: '不登录账号也能体验吗？',
    a: '可以。内置小游戏、红外遥控器、动态壁纸、轻启与 MBTI 均可离线直接使用。',
  },
  {
    q: 'iPhone 什么时候支持？',
    a: 'iOS 原生版适配中，目前可先在 Safari 体验游戏厅、人格测评与社区。',
  },
]

const IOS_WEB_SITES = [
  { name: '小伴游戏厅', host: 'game.xiaban.xyz', href: 'https://game.xiaban.xyz', icon: 'game' },
  { name: '人格贩卖机', host: 'people.xiaban.xyz', href: 'https://people.xiaban.xyz', icon: 'spark' },
  { name: '超级小伴社区', host: 'chat.xiangban.xyz', href: 'https://chat.xiangban.xyz', icon: 'chat' },
] as const

function detectEnv(): EnvKind {
  const override = (routeQuery.value.env || '').toLowerCase()
  if (override === 'android' || override === 'wechat' || override === 'ios') return override
  if (typeof navigator === 'undefined') return 'android'
  const ua = navigator.userAgent || ''
  const isWechatLike = /MicroMessenger|QQ\/|Weibo|XiaoHongShu|xhs/i.test(ua)
  const isIos =
    /iPhone|iPad|iPod/i.test(ua) ||
    (navigator.platform === 'MacIntel' && (navigator.maxTouchPoints ?? 0) > 1)
  if (isIos) return 'ios'
  if (isWechatLike) return 'wechat'
  return 'android'
}

const env = ref<EnvKind>(detectEnv())
const dark = ref(false)
const loading = ref(true)
const error = ref('')
const download = ref<DesktopProductDownload | null>(null)
const sceneIdx = ref(0)
const openFaq = ref<number | null>(0)
const pastHero = ref(false)
const sheetMode = ref<'none' | 'download' | 'changelog'>('none')
const showWechatMask = ref(false)
const faqEl = ref<HTMLElement | null>(null)

const activeScene = computed(() => SCENES[sceneIdx.value] ?? SCENES[0])

const shareUrl = computed(() => {
  if (typeof window === 'undefined') return 'https://xiaban.xyz/app'
  const u = new URL(window.location.href)
  u.searchParams.delete('env')
  u.searchParams.delete('theme')
  return u.toString()
})

const qrSvg = computed(() => {
  try {
    const qr = qrcode(0, 'M')
    qr.addData(shareUrl.value)
    qr.make()
    const count = qr.getModuleCount()
    const pad = 2
    const size = count + pad * 2
    const cells: string[] = []
    for (let r = 0; r < count; r++) {
      for (let c = 0; c < count; c++) {
        if (qr.isDark(r, c)) {
          cells.push(`M${c + pad},${r + pad}h1v1h-1z`)
        }
      }
    }
    return { viewBox: `0 0 ${size} ${size}`, path: cells.join('') }
  } catch {
    return null
  }
})

const apkFileName = computed(() => {
  const url = download.value?.url || ''
  const tail = url.split('?')[0].split('/').pop()
  if (tail && /\.apk$/i.test(tail)) return tail
  return download.value?.version ? `xiaoban-v${download.value.version}.apk` : 'xiaoban-android.apk'
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
      '手机本地创造工具/游戏，支持当场运行与分享',
      '跨话题连续主线对话，支持生图与多任务并行',
      '内置万能遥控器、轻启（跳过开屏广告）、动态壁纸与倒杯快乐',
    ]
  }
  return raw
    .split(/\r?\n/)
    .map(line => line.replace(/^[-*•·]\s*/, '').trim())
    .filter(Boolean)
}

function startApkDownload() {
  if (!download.value?.url) {
    toast('当前暂无可用的 Android 安装包')
    return
  }
  const a = document.createElement('a')
  a.href = download.value.url
  a.rel = 'noopener'
  document.body.appendChild(a)
  a.click()
  a.remove()
}

function onPrimaryAction() {
  if (env.value === 'wechat') {
    showWechatMask.value = true
    return
  }
  if (env.value === 'ios') {
    window.open('https://game.xiaban.xyz', '_blank', 'noopener,noreferrer')
    return
  }
  if (!download.value) {
    if (error.value) {
      load()
    } else {
      toast('Android 安装包即将发布')
    }
    return
  }
  startApkDownload()
  sheetMode.value = 'download'
}

function onRetryDownload() {
  startApkDownload()
  toast('已重新触发 APK 下载')
}

async function copyText(text: string, okMsg: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast(okMsg)
  } catch {
    toast('复制失败，请长按地址栏手动复制')
  }
}

function toggleFaq(idx: number) {
  openFaq.value = openFaq.value === idx ? null : idx
}

function scrollToFaq() {
  faqEl.value?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function toggleTheme() {
  dark.value = !dark.value
}

function onScroll() {
  pastHero.value = window.scrollY > 300
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    sheetMode.value = 'none'
    showWechatMask.value = false
  }
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const product = await getDesktopProduct()
    download.value = product.downloads['android-arm64'] ?? null
  } catch {
    error.value = '版本信息加载失败，请重试'
  } finally {
    loading.value = false
  }
}

let prevTitle = ''

onMounted(() => {
  prevTitle = document.title
  document.title = '小伴手机 App · 官方下载 | xiaban.xyz'
  const themeQ = (routeQuery.value.theme || '').toLowerCase()
  if (themeQ === 'dark') {
    dark.value = true
  } else if (themeQ === 'light') {
    dark.value = false
  } else if (typeof window !== 'undefined' && window.matchMedia) {
    dark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  if (env.value === 'wechat') {
    showWechatMask.value = true
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
  <div class="m-dl-page" :class="{ dark }">
    <!-- 顶部品牌栏 -->
    <header class="lp-topbar">
      <div class="lp-topbar-in">
        <a href="/" class="lp-brand" title="返回小伴产品矩阵" @click.prevent="navigate('/')">
          <span class="lp-brand-icon">
            <img :src="BRAND_ICON_URL" alt="小伴">
          </span>
          <span class="lp-brand-name">小伴 App</span>
          <span v-if="download" class="lp-ver num">v{{ download.version }}</span>
          <span v-else-if="env === 'ios'" class="lp-ver">iOS 适配中</span>
        </a>
        <div class="lp-top-right">
          <button type="button" class="lp-icon-btn" :title="dark ? '切换暖瓷浅色' : '切换曜石深色'" @click="toggleTheme">
            <svg v-if="dark" width="17" height="17" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" /></svg>
            <svg v-else width="17" height="17" viewBox="0 0 24 24"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" /></svg>
          </button>
          <button type="button" class="lp-icon-btn" title="复制页面链接" @click="copyText(shareUrl, '已复制下载页链接')">
            <svg width="17" height="17" viewBox="0 0 24 24"><path d="M4 12v7a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7" /><polyline points="16 6 12 2 8 6" /><line x1="12" y1="2" x2="12" y2="15" /></svg>
          </button>
          <button
            type="button"
            class="lp-mini-dl"
            :class="{ coral: pastHero }"
            @click="onPrimaryAction"
          >
            {{ env === 'ios' ? '网页体验' : '立即下载' }}
          </button>
        </div>
      </div>
    </header>

    <main class="lp-main">
      <!-- ==================== PC 宽屏首屏（≥ 800px 扫码直下） ==================== -->
      <section class="pc-hero-stage" aria-label="电脑访问扫码下载">
        <div class="pc-left">
          <div class="pc-brand-row">
            <div class="lp-brand-icon lg">
              <img :src="BRAND_ICON_URL" alt="小伴">
            </div>
            <div>
              <div class="pc-brand-title">小伴手机 App</div>
              <div class="pc-brand-sub">
                Android 10+ · 官方签名包<template v-if="download"> · v{{ download.version }}</template>
              </div>
            </div>
          </div>

          <h1>把 AI 伙伴与游乐场<br><span class="grad-text">一起装进口袋</span></h1>
          <p class="pc-sub">创造工具/游戏 · 官方游乐场 · 实用百宝箱 · 原生社区</p>

          <div class="qr-panel">
            <div class="qr-box" aria-label="手机扫码下载二维码">
              <svg v-if="qrSvg" :viewBox="qrSvg.viewBox" width="96" height="96" shape-rendering="crispEdges" style="stroke:none">
                <rect width="100%" height="100%" fill="#ffffff" />
                <path :d="qrSvg.path" fill="#19171a" stroke="none" />
              </svg>
            </div>
            <div class="qr-meta">
              <strong>手机扫码直接下载</strong>
              <p class="num">
                <template v-if="download">v{{ download.version }}<template v-if="formatBytes(download.bytes)"> · {{ formatBytes(download.bytes) }}</template> · </template>微信扫码后切浏览器下载
              </p>
              <div class="qr-actions">
                <button
                  type="button"
                  class="qr-btn-sm"
                  :disabled="loading || (!download && !error)"
                  @click="onPrimaryAction"
                >
                  {{ loading ? '加载中…' : error ? '重新加载' : download ? '直接下载 APK' : '即将发布' }}
                </button>
                <button
                  v-if="download"
                  type="button"
                  class="qr-btn-ghost"
                  @click="sheetMode = 'changelog'"
                >
                  更新日志
                </button>
              </div>
            </div>
          </div>

          <a href="/desktop" class="pc-desk-tip" @click.prevent="navigate('/desktop')">
            <span>需要电脑端？</span>
            <b>下载小伴桌面（macOS / Windows）↗</b>
          </a>
        </div>

        <div class="pc-right">
          <div class="shot-frame">
            <img :src="assetUrl('mobile-chat-build.webp')" alt="手机造游戏">
          </div>
          <div class="shot-frame lift">
            <img :src="assetUrl('mobile-community-new.webp')" alt="原生社区">
          </div>
        </div>
      </section>

      <!-- ==================== 移动首屏（< 800px） ==================== -->
      <section class="mobile-hero">
        <div class="hero-icon-wrap">
          <div class="hero-icon-glow" aria-hidden="true" />
          <div class="hero-icon">
            <img :src="BRAND_ICON_URL" alt="小伴双星图标">
          </div>
        </div>

        <h1>把 AI 伙伴与游乐场<br><span class="grad-text">一起装进口袋</span></h1>
        <p class="hero-sub">创造工具/游戏 · 官方游乐场 · 实用百宝箱 · 原生社区</p>

        <!-- iOS 访客承接卡 -->
        <div v-if="env === 'ios'" class="ios-status-card">
          <span class="ios-pill">
            <svg width="13" height="13" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
            <span>iOS 版适配中</span>
          </span>
          <h3>先在 Safari 体验网页生态</h3>
          <p>
            Android 已发布<template v-if="download"> v{{ download.version }}</template>；iPhone 免安装即可畅玩以下站点。
          </p>

          <div class="ios-web-list">
            <a
              v-for="site in IOS_WEB_SITES"
              :key="site.host"
              :href="site.href"
              target="_blank"
              rel="noopener noreferrer"
              class="ios-web-item"
            >
              <div class="ios-web-left">
                <span class="ios-web-ico">
                  <svg v-if="site.icon === 'game'" width="17" height="17" viewBox="0 0 24 24"><rect x="2" y="6" width="20" height="12" rx="4" /><path d="M6 12h4M8 10v4M15 11h.01M18 13h.01" /></svg>
                  <svg v-else-if="site.icon === 'spark'" width="17" height="17" viewBox="0 0 24 24"><path d="M12 3l1.9 5.8L20 11l-6.1 2.2L12 19l-1.9-5.8L4 11l6.1-2.2Z" /></svg>
                  <svg v-else width="17" height="17" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                </span>
                <div>
                  <strong>{{ site.name }}</strong>
                  <span class="mono">{{ site.host }}</span>
                </div>
              </div>
              <span class="ios-go">打开</span>
            </a>
          </div>

          <div class="ios-sub-actions">
            <button type="button" class="btn-secondary sm" @click="copyText(shareUrl, '已复制链接，发给安卓手机即可下载')">发给安卓手机</button>
            <a href="/desktop" class="btn-secondary sm" @click.prevent="navigate('/desktop')">macOS 桌面版</a>
          </div>
        </div>

        <!-- Android / 微信 默认主下载区 -->
        <div v-else class="hero-cta-box">
          <div v-if="error" class="err-box">
            <span>{{ error }}</span>
            <button type="button" class="err-retry" @click="load">重试</button>
          </div>
          <button
            v-else
            type="button"
            class="btn-primary-dl"
            :disabled="loading || !download"
            @click="onPrimaryAction"
          >
            <svg width="18" height="18" viewBox="0 0 24 24"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
            <span>{{ loading ? '正在获取最新版本…' : download ? '立即下载 Android 版' : 'Android 版即将发布' }}</span>
            <span v-if="download && formatBytes(download.bytes)" class="btn-dl-meta num">{{ formatBytes(download.bytes) }}</span>
          </button>

          <div class="trust-row">
            <span class="trust-item"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>官方签名</span>
            <span>·</span>
            <span class="trust-item"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>零广告</span>
            <span>·</span>
            <span class="trust-item"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>Android 10+</span>
          </div>

          <div class="quick-links-row">
            <button v-if="download" type="button" class="text-chip" @click="sheetMode = 'changelog'">更新日志 · SHA-256</button>
            <button type="button" class="text-chip" @click="scrollToFaq">常见疑问</button>
          </div>
        </div>

        <!-- 首屏 4 场景实机切换展台 -->
        <div class="showcase-card">
          <div class="showcase-tabs" role="tablist">
            <button
              v-for="(s, idx) in SCENES"
              :key="s.tab"
              type="button"
              role="tab"
              class="showcase-tab"
              :class="{ active: idx === sceneIdx }"
              :aria-selected="idx === sceneIdx"
              @click="sceneIdx = idx"
            >
              {{ s.tab }}
            </button>
          </div>
          <div class="showcase-stage">
            <div class="phone-pair">
              <div class="shot-frame"><img :src="activeScene.imgs[0]" :alt="activeScene.tab"></div>
              <div class="shot-frame lift"><img :src="activeScene.imgs[1]" :alt="activeScene.tab"></div>
            </div>
            <div class="showcase-caption">
              <strong>{{ activeScene.caption }}</strong>
            </div>
          </div>
        </div>
      </section>

      <!-- ==================== 核心能力长卷（双端自适应） ==================== -->
      <div class="lp-sections">
        <!-- 01 随身 AI 伙伴 -->
        <section class="sec">
          <div class="sec-head">
            <h2>随身 AI 伙伴</h2>
          </div>
          <div class="bento-grid">
            <article class="feature-card">
              <div class="fc-split">
                <div>
                  <span class="fc-ico coral">
                    <svg width="18" height="18" viewBox="0 0 24 24"><path d="M12 3l1.9 5.8L20 11l-6.1 2.2L12 19l-1.9-5.8L4 11l6.1-2.2Z" /></svg>
                  </span>
                  <h3>手机创造工具/游戏</h3>
                  <p>一句话写代码、当场运行，支持导出或发到社区。</p>
                </div>
                <div class="fc-thumb">
                  <img :src="assetUrl('mobile-chat-build.webp')" alt="手机创造工具与游戏" loading="lazy">
                </div>
              </div>
            </article>

            <article class="feature-card">
              <div class="fc-split">
                <div>
                  <span class="fc-ico green">
                    <svg width="18" height="18" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                  </span>
                  <h3>连续对话与 AI 生图</h3>
                  <p>跨话题主线聊天、多任务并行，支持一键生成静态图与 GIF。</p>
                </div>
                <div class="fc-thumb">
                  <img :src="assetUrl('mobile-ai-chat.webp')" alt="连续对话与 AI 生图" loading="lazy">
                </div>
              </div>
            </article>
          </div>
        </section>

        <!-- 02 官方游乐场 -->
        <section class="sec">
          <div class="sec-head">
            <h2>官方游乐场</h2>
          </div>
          <div class="relay-card">
            <div class="relay-glow" aria-hidden="true" />
            <div class="arcade-head">
              <h3>内置精选小游戏，离线断点续玩</h3>
              <span class="arcade-pill">零广告</span>
            </div>
            <div class="arcade-grid">
              <div v-for="g in GAMES" :key="g.name" class="arcade-tile">
                <div class="game-icon"><img :src="g.icon" :alt="g.name" loading="lazy"></div>
                <strong>{{ g.name }}</strong>
              </div>
            </div>
          </div>
        </section>

        <!-- 03 实用百宝箱 -->
        <section class="sec">
          <div class="sec-head">
            <h2>实用百宝箱</h2>
          </div>

          <div class="tool-grid" style="margin-top: 0">
            <article v-for="t in TOOLS" :key="t.name" class="tool-card">
              <div class="tool-head">
                <div class="tool-icon"><img :src="t.icon" :alt="t.name" loading="lazy"></div>
                <strong>{{ t.name }}</strong>
              </div>
              <p>{{ t.desc }}</p>
            </article>
          </div>
        </section>

        <!-- 04 社区与作品 -->
        <section class="sec">
          <div class="sec-head">
            <h2>社区与作品</h2>
          </div>
          <div class="dual-grid">
            <article class="mini-showcase">
              <div class="ms-shot">
                <img :src="assetUrl('mobile-community-new.webp')" alt="超级小伴社区" loading="lazy">
              </div>
              <div class="ms-body">
                <strong>超级小伴社区</strong>
                <p>创意作品即点即玩、前沿日报与热榜</p>
              </div>
            </article>
            <article class="mini-showcase">
              <div class="ms-shot">
                <img :src="assetUrl('mobile-my-works.webp')" alt="我的作品空间" loading="lazy">
              </div>
              <div class="ms-body">
                <strong>我的作品空间</strong>
                <p>草稿随时续作，一键分享海报与作品</p>
              </div>
            </article>
          </div>
        </section>

        <!-- 05 常见疑问与生态 -->
        <section ref="faqEl" class="sec">
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
            <div class="eco-title">
              <span>小伴全端生态</span>
            </div>
            <div class="eco-grid">
              <a class="eco-link" href="/desktop" @click.prevent="navigate('/desktop')">
                <strong>小伴桌面端</strong>
                <span>xiaban.xyz/desktop</span>
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
            <template v-if="download">v{{ download.version }} · </template>arm64 · Android 10+
          </footer>
        </section>
      </div>
    </main>

    <!-- 移动端滚动吸底转化条 -->
    <div class="sticky-dl-bar" :class="{ 'hidden-bar': !pastHero }">
      <div class="sticky-meta">
        <img :src="BRAND_ICON_URL" alt="小伴">
        <div>
          <strong>小伴手机 App</strong>
          <span class="num">
            <template v-if="download">v{{ download.version }}<template v-if="formatBytes(download.bytes)"> · {{ formatBytes(download.bytes) }}</template></template>
            <template v-else>{{ env === 'ios' ? 'iOS 适配中' : 'Android 10+' }}</template>
          </span>
        </div>
      </div>
      <button type="button" class="sticky-btn" @click="onPrimaryAction">
        {{ env === 'ios' ? '网页体验' : '立即下载' }}
      </button>
    </div>

    <!-- 下载指引 / 更新日志 BottomSheet -->
    <Teleport to="body">
      <div
        v-if="sheetMode !== 'none' && download"
        class="m-dl-overlay"
        :class="{ dark }"
        @click.self="sheetMode = 'none'"
      >
        <div class="sheet" role="dialog" aria-modal="true" :aria-label="sheetMode === 'download' ? '已开始下载 APK' : '更新内容'">
          <div class="sheet-handle" />
          <div class="sheet-head">
            <div class="sheet-title-row">
              <span v-if="sheetMode === 'download'" class="fc-ico green sm">
                <svg width="15" height="15" viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12" /></svg>
              </span>
              <h3>{{ sheetMode === 'download' ? '已开始下载 APK' : `v${download.version} 更新内容` }}</h3>
            </div>
            <button type="button" class="sheet-close" aria-label="关闭" @click="sheetMode = 'none'">✕</button>
          </div>

          <div v-if="sheetMode === 'download'" class="steps-list">
            <div class="step-row">
              <span class="step-idx">1</span>
              <strong>下拉通知栏，点击 {{ apkFileName }}</strong>
            </div>
            <div class="step-row">
              <span class="step-idx">2</span>
              <strong>允许浏览器安装并确认</strong>
            </div>
            <div class="step-row">
              <span class="step-idx">3</span>
              <strong>覆盖安装保留原有数据</strong>
            </div>
          </div>
          <div v-else class="sheet-meta num">
            {{ formatDate(download.publishedAt) }}
            <template v-if="formatBytes(download.bytes)"> · {{ formatBytes(download.bytes) }}</template>
            · arm64
          </div>

          <div class="sheet-changelog-head">
            <span>v{{ download.version }} 更新内容</span>
            <span class="num">{{ formatDate(download.publishedAt) }}</span>
          </div>
          <ul class="changelog-ul">
            <li v-for="(line, i) in noteLines(download.notes)" :key="i">{{ line }}</li>
          </ul>

          <div v-if="download.sha256" class="sha-box">
            <div class="sha-code mono">SHA-256: {{ download.sha256.slice(0, 18) }}…{{ download.sha256.slice(-10) }}</div>
            <button type="button" class="sha-copy" @click="copyText(download.sha256, '已复制 SHA-256')">复制</button>
          </div>

          <button
            type="button"
            class="btn-primary-dl compact"
            @click="sheetMode === 'download' ? onRetryDownload() : (startApkDownload(), sheetMode = 'download')"
          >
            {{ sheetMode === 'download' ? '重新下载' : '立即下载 Android 版' }}
            <template v-if="formatBytes(download.bytes)">（{{ formatBytes(download.bytes) }}）</template>
          </button>
        </div>
      </div>

      <!-- 微信 / QQ 内置浏览器拦截引导遮罩 -->
      <div
        v-if="showWechatMask"
        class="wx-guide-mask"
        :class="{ dark }"
      >
        <div class="wx-arrow-hint">
          <svg width="34" height="34" viewBox="0 0 36 36" fill="none" stroke="#fff" stroke-width="2.4">
            <path d="M6 30 C16 26, 23 18, 27 5" />
            <path d="M19 9 L27 5 L30 13" />
          </svg>
          <span>点这里用浏览器打开</span>
        </div>

        <div class="wx-card" role="dialog" aria-modal="true" aria-label="在浏览器中打开下载">
          <h3>微信内无法直接下载 APK</h3>
          <div class="wx-steps">
            <div class="wx-step">
              <span class="wx-step-n">1</span>
              <strong>点击右上角「···」</strong>
            </div>
            <div class="wx-step">
              <span class="wx-step-n">2</span>
              <strong>选择「在浏览器中打开」</strong>
            </div>
          </div>
          <button type="button" class="btn-primary-dl compact" @click="copyText(shareUrl, '已复制链接，去浏览器粘贴即可')">
            <svg width="16" height="16" viewBox="0 0 24 24"><rect x="9" y="9" width="13" height="13" rx="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
            <span>复制链接去浏览器打开</span>
          </button>
          <button type="button" class="btn-secondary sm" style="margin-top: 8px" @click="showWechatMask = false">
            继续浏览页面
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.m-dl-page {
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
  --coral-shadow: 0 12px 28px rgba(242, 92, 114, 0.3);
  --ok: #169c57;
  --ok-soft: #eaf9f0;
  --warn: #d97706;
  --warn-soft: #fff6e8;
  --glass: rgba(252, 250, 247, 0.86);
  --card-shadow: 0 2px 10px rgba(25, 18, 24, 0.04);
  --float-shadow: 0 14px 36px rgba(25, 18, 24, 0.12);
  --ease: cubic-bezier(0.22, 1, 0.36, 1);
  --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;

  min-height: 100vh;
  min-height: 100dvh;
  color: var(--ink);
  color-scheme: light;
  background:
    radial-gradient(80% 340px at 50% -60px, rgba(255, 107, 134, 0.16), transparent 70%),
    radial-gradient(55% 280px at 88% 120px, rgba(249, 144, 86, 0.11), transparent 66%),
    var(--canvas);
}

.m-dl-page.dark {
  --canvas: #111114;
  --canvas-soft: #19191f;
  --surface: #1e1e24;
  --surface-elev: #25252d;
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
  --warn: #fbbf24;
  --warn-soft: rgba(251, 191, 36, 0.14);
  --glass: rgba(17, 17, 20, 0.86);
  --card-shadow: 0 2px 10px rgba(0, 0, 0, 0.22);
  --float-shadow: 0 18px 42px rgba(0, 0, 0, 0.52);

  color-scheme: dark;
  background:
    radial-gradient(80% 340px at 50% -60px, rgba(255, 91, 125, 0.18), transparent 72%),
    radial-gradient(55% 280px at 88% 130px, rgba(249, 116, 75, 0.11), transparent 66%),
    var(--canvas);
}

.m-dl-page svg {
  flex: none;
  stroke-width: 1.85;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.m-dl-page svg:not([fill]) {
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
  max-width: 960px;
  margin: 0 auto;
  height: 52px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.lp-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.lp-brand-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  flex: none;
  overflow: hidden;
  background: var(--coral-grad);
  box-shadow: 0 3px 10px rgba(238, 79, 109, 0.25);
}
.lp-brand-icon.lg {
  width: 42px;
  height: 42px;
  border-radius: 12px;
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
}
.lp-top-right { display: flex; align-items: center; gap: 6px; }
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
  padding: 0 13px;
  border-radius: 999px;
  background: var(--ink);
  color: var(--canvas);
  font-size: 12px;
  font-weight: 650;
  display: inline-flex;
  align-items: center;
  transition: transform 0.14s, background 0.2s;
}
.lp-mini-dl.coral { background: var(--coral-grad); color: #fff; }
.lp-mini-dl:active { transform: scale(0.96); }

/* ==================== 主体布局 ==================== */
.lp-main {
  max-width: 960px;
  margin: 0 auto;
  padding: 20px 16px calc(94px + env(safe-area-inset-bottom, 0px));
}

/* ==================== PC 宽屏首屏 ==================== */
.pc-hero-stage {
  display: none;
}

@media (min-width: 800px) {
  .lp-main { padding: 36px 24px 72px; }
  .pc-hero-stage {
    display: grid;
    grid-template-columns: 1.08fr 0.92fr;
    gap: 32px;
    align-items: center;
    padding: 28px 30px;
    border-radius: 28px;
    background: var(--surface);
    border: 1px solid var(--line);
    box-shadow: var(--card-shadow);
  }
}

.pc-brand-row { display: flex; align-items: center; gap: 11px; }
.pc-brand-title { font-size: 15.5px; font-weight: 750; }
.pc-brand-sub { font-size: 12px; color: var(--ink-3); margin-top: 1px; }
.pc-left h1 {
  font-size: 34px;
  line-height: 1.18;
  letter-spacing: -0.035em;
  font-weight: 780;
  margin-top: 16px;
}
.pc-sub { font-size: 14px; color: var(--ink-2); margin-top: 10px; }

.qr-panel {
  margin-top: 20px;
  padding: 14px;
  border-radius: 20px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  gap: 15px;
}
.qr-box {
  width: 112px;
  height: 112px;
  padding: 8px;
  border-radius: 14px;
  background: #ffffff;
  border: 1px solid rgba(25, 23, 26, 0.08);
  flex: none;
  display: grid;
  place-items: center;
}
.qr-meta strong { display: block; font-size: 14.5px; font-weight: 740; }
.qr-meta p { font-size: 12px; color: var(--ink-2); margin-top: 4px; line-height: 1.5; }
.qr-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 11px; }
.qr-btn-sm {
  height: 34px;
  padding: 0 14px;
  border-radius: 10px;
  font-size: 12.5px;
  font-weight: 660;
  background: var(--coral-grad);
  color: #fff;
  display: inline-flex;
  align-items: center;
}
.qr-btn-sm:disabled { opacity: 0.6; cursor: not-allowed; }
.qr-btn-ghost {
  height: 34px;
  padding: 0 12px;
  border-radius: 10px;
  font-size: 12.5px;
  font-weight: 600;
  background: var(--surface);
  border: 1px solid var(--line-strong);
  color: var(--ink-2);
}
.pc-desk-tip {
  margin-top: 16px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12.5px;
  color: var(--ink-2);
  padding: 8px 13px;
  border-radius: 12px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  transition: border-color 0.15s;
}
.pc-desk-tip:hover { border-color: var(--line-strong); }
.pc-desk-tip b { color: var(--coral-ink); font-weight: 680; }
.pc-right {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 24px 16px;
  border-radius: 22px;
  background:
    radial-gradient(85% 90% at 88% 8%, rgba(255, 107, 134, 0.16), transparent 60%),
    radial-gradient(75% 80% at 10% 95%, rgba(249, 144, 86, 0.14), transparent 60%),
    var(--canvas-soft);
  border: 1px solid var(--line);
}
.pc-right .shot-frame { width: 158px; }

/* ==================== 移动首屏 ==================== */
.mobile-hero {
  max-width: 440px;
  margin: 0 auto;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 6px;
}
.hero-icon-wrap { position: relative; margin-bottom: 14px; }
.hero-icon-glow {
  position: absolute;
  inset: -10px;
  border-radius: 28px;
  background: var(--coral-grad);
  opacity: 0.22;
  filter: blur(16px);
  pointer-events: none;
}
.hero-icon {
  position: relative;
  width: 68px;
  height: 68px;
  border-radius: 19px;
  overflow: hidden;
  background: var(--coral-grad);
  box-shadow: 0 14px 32px rgba(242, 92, 114, 0.32), 0 0 0 1px rgba(255, 255, 255, 0.55) inset;
}
.hero-icon img { width: 100%; height: 100%; object-fit: cover; }
.mobile-hero h1 {
  font-size: 27px;
  line-height: 1.22;
  letter-spacing: -0.035em;
  font-weight: 780;
  color: var(--ink);
}
.hero-sub {
  margin-top: 9px;
  font-size: 13px;
  color: var(--ink-2);
}

.hero-cta-box { width: 100%; margin-top: 18px; }
.btn-primary-dl {
  width: 100%;
  height: 52px;
  padding: 0 18px;
  border-radius: 16px;
  background: var(--coral-grad);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 9px;
  font-size: 15.5px;
  font-weight: 730;
  box-shadow: var(--coral-shadow);
  transition: transform 0.14s var(--ease), filter 0.16s;
}
.btn-primary-dl:hover { filter: brightness(1.04); }
.btn-primary-dl:active { transform: scale(0.982); }
.btn-primary-dl:disabled { opacity: 0.65; cursor: not-allowed; }
.btn-primary-dl.compact { height: 46px; font-size: 14.5px; }
.btn-dl-meta {
  font-size: 11.5px;
  font-weight: 600;
  opacity: 0.9;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.35);
}

.btn-secondary {
  width: 100%;
  height: 44px;
  padding: 0 14px;
  border-radius: 14px;
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line-strong);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 650;
}
.btn-secondary.sm { height: 39px; font-size: 12.5px; }

.err-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 14px;
  background: var(--surface);
  border: 1px solid var(--line-strong);
  font-size: 13px;
  color: var(--ink-2);
}
.err-retry {
  height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  background: var(--coral-grad);
  color: #fff;
  font-size: 12px;
  font-weight: 650;
}

.trust-row {
  margin-top: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 6px 10px;
  font-size: 11.5px;
  color: var(--ink-3);
}
.trust-item { display: inline-flex; align-items: center; gap: 4px; }
.trust-item svg { width: 13px; height: 13px; color: var(--ok); }
.quick-links-row {
  margin-top: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex-wrap: wrap;
}
.text-chip {
  height: 28px;
  padding: 0 11px;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--line);
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ink-2);
}

/* 首屏 4 场景实机展台 */
.showcase-card {
  margin-top: 18px;
  width: 100%;
  border-radius: 24px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--card-shadow);
  padding: 12px;
}
.showcase-tabs {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
  padding: 3px;
  border-radius: 12px;
  background: var(--canvas-soft);
  margin-bottom: 12px;
}
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
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.07);
}
.showcase-stage {
  border-radius: 18px;
  padding: 14px 12px 12px;
  background:
    radial-gradient(85% 90% at 88% 8%, rgba(255, 107, 134, 0.18), transparent 60%),
    radial-gradient(75% 80% at 10% 95%, rgba(249, 144, 86, 0.16), transparent 60%),
    var(--canvas-soft);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}
.phone-pair {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 10px;
  width: 100%;
  padding-top: 2px;
}
.shot-frame {
  width: 146px;
  max-width: 44%;
  aspect-ratio: 640 / 1330;
  border-radius: 20px;
  overflow: hidden;
  background: #141418;
  border: 3px solid #1b1a1f;
  box-shadow: 0 14px 28px rgba(24, 18, 24, 0.18);
  flex: none;
}
.m-dl-page.dark .shot-frame { border-color: #34333d; box-shadow: 0 16px 32px rgba(0, 0, 0, 0.5); }
.shot-frame.lift { transform: translateY(-8px); }
.shot-frame img { width: 100%; height: 100%; object-fit: cover; object-position: top; }
.showcase-caption {
  width: 100%;
  padding: 9px 12px;
  border-radius: 13px;
  background: rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(25, 23, 26, 0.06);
  text-align: center;
}
.m-dl-page.dark .showcase-caption {
  background: rgba(28, 28, 35, 0.88);
  border-color: rgba(255, 255, 255, 0.08);
}
.showcase-caption strong { display: block; font-size: 12.5px; font-weight: 700; color: var(--ink); }

/* iOS 承接卡 */
.ios-status-card {
  width: 100%;
  margin-top: 16px;
  padding: 16px;
  border-radius: 20px;
  text-align: left;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--card-shadow);
}
.ios-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--warn-soft);
  color: var(--warn);
  font-size: 11.5px;
  font-weight: 700;
}
.ios-status-card h3 { font-size: 16px; font-weight: 740; margin-top: 10px; }
.ios-status-card p { font-size: 12.5px; color: var(--ink-2); line-height: 1.6; margin-top: 5px; }
.ios-web-list { margin-top: 14px; display: flex; flex-direction: column; gap: 8px; }
.ios-web-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 11px 12px;
  border-radius: 14px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
}
.ios-web-left { display: flex; align-items: center; gap: 10px; min-width: 0; }
.ios-web-ico {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  flex: none;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink);
}
.ios-web-item strong { display: block; font-size: 13px; font-weight: 700; }
.ios-web-left > div > span { display: block; font-size: 11px; color: var(--ink-3); margin-top: 1px; }
.ios-go {
  height: 28px;
  padding: 0 11px;
  border-radius: 999px;
  background: var(--ink);
  color: var(--canvas);
  font-size: 11.5px;
  font-weight: 650;
  flex: none;
  display: inline-flex;
  align-items: center;
}
.ios-sub-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 12px;
}

/* ==================== 区块长卷 ==================== */
.lp-sections {
  max-width: 440px;
  margin: 0 auto;
}
@media (min-width: 800px) {
  .lp-sections { max-width: 960px; }
}

.sec { margin-top: 28px; }
.sec-head { margin-bottom: 12px; padding: 0 2px; }
.sec-head h2 {
  font-size: 19px;
  font-weight: 760;
  letter-spacing: -0.025em;
  line-height: 1.28;
  color: var(--ink);
}

.bento-grid { display: grid; grid-template-columns: 1fr; gap: 10px; }
@media (min-width: 800px) {
  .bento-grid { grid-template-columns: 1fr 1fr; gap: 14px; }
}

.feature-card {
  border-radius: 20px;
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 15px;
  box-shadow: var(--card-shadow);
}
.fc-ico {
  width: 36px;
  height: 36px;
  border-radius: 11px;
  display: grid;
  place-items: center;
  flex: none;
}
.fc-ico.sm { width: 28px; height: 28px; border-radius: 9px; }
.fc-ico.coral { background: var(--coral-soft); color: var(--coral-ink); }
.fc-ico.green { background: var(--ok-soft); color: var(--ok); }
.feature-card h3 { font-size: 15.5px; font-weight: 730; margin-top: 10px; letter-spacing: -0.015em; }
.feature-card p { font-size: 12.5px; color: var(--ink-2); line-height: 1.6; margin-top: 4px; }
.fc-split {
  display: grid;
  grid-template-columns: 1fr 102px;
  gap: 12px;
  align-items: center;
}
.fc-thumb {
  width: 102px;
  aspect-ratio: 640 / 1260;
  border-radius: 14px;
  overflow: hidden;
  border: 2px solid var(--line-strong);
  background: #141418;
  box-shadow: 0 8px 18px rgba(20, 16, 20, 0.12);
}
.fc-thumb img { width: 100%; height: 100%; object-fit: cover; object-position: top; }

/* 手机遥控电脑 */
.relay-card {
  border-radius: 22px;
  padding: 16px;
  position: relative;
  overflow: hidden;
  background: linear-gradient(155deg, #1d1b22 0%, #141318 100%);
  color: #f5f4f8;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 16px 36px rgba(20, 16, 24, 0.18);
}
.m-dl-page.dark .relay-card {
  background: linear-gradient(155deg, #22212b 0%, #18181f 100%);
  border-color: rgba(255, 255, 255, 0.11);
}
.relay-glow {
  position: absolute;
  top: -50px;
  right: -40px;
  width: 180px;
  height: 180px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 91, 125, 0.3), transparent 68%);
  pointer-events: none;
}
.relay-diagram {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 11px 12px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.09);
  margin-bottom: 13px;
  position: relative;
  z-index: 1;
}
.relay-node { display: flex; align-items: center; gap: 7px; font-size: 12px; font-weight: 650; }
.relay-node-ico {
  width: 30px;
  height: 30px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}
.relay-line {
  flex: 1;
  height: 2px;
  position: relative;
  background: linear-gradient(90deg, rgba(255, 91, 125, 0.25), #ff6b86, rgba(249, 116, 75, 0.3));
  border-radius: 999px;
}
.relay-line span {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
  background: #282530;
  color: #ffb4c2;
  border: 1px solid rgba(255, 107, 134, 0.35);
  white-space: nowrap;
}
.relay-card h3 { font-size: 16px; font-weight: 740; position: relative; z-index: 1; }
.arcade-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  position: relative;
  z-index: 1;
}
.arcade-pill {
  font-size: 11px;
  font-weight: 650;
  padding: 3px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(245, 244, 248, 0.85);
  flex: none;
}
.arcade-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px 10px;
  position: relative;
  z-index: 1;
  margin-top: 16px;
}
@media (min-width: 800px) {
  .arcade-grid { grid-template-columns: repeat(6, 1fr); gap: 14px; }
}
.arcade-tile { text-align: center; }
.arcade-tile .game-icon {
  width: 62px;
  height: 62px;
  margin: 0 auto 7px;
  border-radius: 16px;
  border-color: rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.06);
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.32);
}
.arcade-tile strong {
  display: block;
  font-size: 12px;
  font-weight: 650;
  color: #f5f4f8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.relay-shots {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-top: 13px;
  position: relative;
  z-index: 1;
}
.relay-shot {
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: #0f0f13;
}
.relay-shot img { width: 100%; aspect-ratio: 640 / 760; object-fit: cover; object-position: top; }
@media (min-width: 800px) {
  .relay-shot img { aspect-ratio: 16 / 11; }
}
.relay-shot figcaption {
  padding: 7px 10px;
  font-size: 11.5px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.84);
  background: rgba(255, 255, 255, 0.05);
}

/* 游戏与小工具 */
.plaza-strip-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.plaza-strip-head strong { font-size: 13.5px; font-weight: 720; }
.plaza-strip-head span { font-size: 11px; color: var(--ink-3); }
.game-rail {
  display: flex;
  gap: 12px;
  overflow-x: auto;
  padding-bottom: 2px;
  scrollbar-width: none;
}
.game-rail::-webkit-scrollbar { display: none; }
.game-tile { width: 74px; flex: none; text-align: center; }
.game-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 6px;
  border-radius: 16px;
  overflow: hidden;
  border: 1px solid var(--line);
  background: var(--canvas-soft);
  box-shadow: 0 4px 12px rgba(20, 16, 20, 0.07);
}
.game-icon img { width: 100%; height: 100%; object-fit: cover; }
.game-tile strong {
  display: block;
  font-size: 11.5px;
  font-weight: 680;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tool-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-top: 12px;
}
@media (min-width: 800px) {
  .tool-grid { grid-template-columns: repeat(4, 1fr); gap: 12px; }
}
.tool-card {
  padding: 13px;
  border-radius: 16px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--card-shadow);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.tool-head { display: flex; align-items: center; gap: 9px; }
.tool-icon {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  overflow: hidden;
  flex: none;
  border: 1px solid var(--line);
  background: var(--canvas-soft);
}
.tool-icon img { width: 100%; height: 100%; object-fit: cover; }
.tool-head strong { font-size: 13px; font-weight: 720; line-height: 1.25; }
.tool-card p { font-size: 11.5px; color: var(--ink-2); line-height: 1.5; }

/* 社区与自由模型 */
.dual-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
@media (min-width: 800px) {
  .dual-grid { gap: 14px; }
}
.mini-showcase {
  border-radius: 18px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--card-shadow);
  overflow: hidden;
}
.ms-shot { height: 148px; overflow: hidden; background: var(--canvas-soft); border-bottom: 1px solid var(--line); }
@media (min-width: 800px) {
  .ms-shot { height: 220px; }
}
.ms-shot img { width: 100%; height: 100%; object-fit: cover; object-position: top; }
.ms-body { padding: 12px 13px 14px; }
.ms-body strong { display: block; font-size: 13.5px; font-weight: 720; }
.ms-body p { font-size: 12px; color: var(--ink-2); line-height: 1.5; margin-top: 4px; }

/* FAQ & 生态 */
.faq-box {
  border-radius: 18px;
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
  box-shadow: var(--card-shadow);
}
.faq-item { border-bottom: 1px solid var(--line); }
.faq-item:last-child { border-bottom: 0; }
.faq-q {
  width: 100%;
  padding: 14px 15px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  font-size: 13.5px;
  font-weight: 660;
  text-align: left;
  color: var(--ink);
}
.faq-q svg { width: 16px; height: 16px; color: var(--ink-3); transition: transform 0.2s var(--ease); }
.faq-item.open .faq-q svg { transform: rotate(180deg); }
.faq-a { padding: 0 15px 14px; font-size: 12.5px; color: var(--ink-2); line-height: 1.65; }

.eco-card {
  margin-top: 16px;
  border-radius: 18px;
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 14px;
  box-shadow: var(--card-shadow);
}
.eco-title { font-size: 12px; font-weight: 700; color: var(--ink-2); margin-bottom: 9px; }
.eco-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
@media (min-width: 800px) {
  .eco-grid { grid-template-columns: repeat(4, 1fr); }
}
.eco-link {
  padding: 10px 11px;
  border-radius: 12px;
  background: var(--canvas-soft);
  border: 1px solid transparent;
  transition: border-color 0.15s, background 0.15s;
}
.eco-link:hover { border-color: var(--line-strong); background: var(--fill); }
.eco-link strong { display: block; font-size: 12.5px; font-weight: 700; }
.eco-link span { display: block; font-size: 11px; color: var(--ink-3); margin-top: 2px; }

.lp-foot {
  margin-top: 18px;
  text-align: center;
  font-size: 11.5px;
  color: var(--ink-3);
}

/* 吸底浮动下载条 */
.sticky-dl-bar {
  position: fixed;
  left: 12px;
  right: 12px;
  bottom: calc(12px + env(safe-area-inset-bottom, 0px));
  max-width: 440px;
  margin: 0 auto;
  z-index: 32;
  padding: 9px 10px 9px 12px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(18px) saturate(1.5);
  -webkit-backdrop-filter: blur(18px) saturate(1.5);
  border: 1px solid var(--line-strong);
  box-shadow: var(--float-shadow);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  transition: transform 0.24s var(--ease), opacity 0.24s ease;
}
.m-dl-page.dark .sticky-dl-bar {
  background: rgba(30, 30, 37, 0.92);
  border-color: rgba(255, 255, 255, 0.12);
}
.sticky-dl-bar.hidden-bar {
  transform: translateY(130%);
  opacity: 0;
  pointer-events: none;
}
.sticky-meta { display: flex; align-items: center; gap: 9px; min-width: 0; }
.sticky-meta img { width: 34px; height: 34px; border-radius: 10px; flex: none; }
.sticky-meta strong { display: block; font-size: 13px; font-weight: 730; line-height: 1.2; }
.sticky-meta span { display: block; font-size: 11px; color: var(--ink-3); margin-top: 1px; }
.sticky-btn {
  height: 36px;
  padding: 0 15px;
  border-radius: 12px;
  flex: none;
  background: var(--coral-grad);
  color: #fff;
  font-size: 12.5px;
  font-weight: 700;
  box-shadow: 0 6px 16px rgba(242, 92, 114, 0.28);
}

@media (min-width: 800px) {
  .mobile-hero,
  .sticky-dl-bar {
    display: none;
  }
}
</style>

<style>
/* Teleported BottomSheet & WeChat Mask (inherits exact palette) */
.m-dl-overlay {
  --canvas-soft: #f5f1eb;
  --surface: #ffffff;
  --ink: #19171a;
  --ink-2: #524d54;
  --ink-3: #837d86;
  --line: rgba(28, 22, 28, 0.08);
  --line-strong: rgba(28, 22, 28, 0.15);
  --fill: #f2eee8;
  --coral: #ee4f6d;
  --coral-grad: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
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
.m-dl-overlay.dark,
.wx-guide-mask.dark {
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
  .m-dl-overlay { align-items: center; padding: 20px; }
}
.m-dl-overlay .sheet {
  width: 100%;
  max-width: 440px;
  max-height: 86dvh;
  overflow-y: auto;
  background: var(--surface);
  color: var(--ink);
  border-radius: 20px 20px 0 0;
  padding: 10px 18px calc(22px + env(safe-area-inset-bottom, 0px));
  box-shadow: 0 -14px 40px rgba(0, 0, 0, 0.22);
}
@media (min-width: 640px) {
  .m-dl-overlay .sheet { border-radius: 24px; padding: 20px 22px 22px; }
  .m-dl-overlay .sheet-handle { display: none; }
}
.m-dl-overlay .sheet-handle {
  width: 36px;
  height: 4px;
  border-radius: 999px;
  background: var(--line-strong);
  margin: 0 auto 14px;
}
.m-dl-overlay .sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}
.m-dl-overlay .sheet-title-row { display: flex; align-items: center; gap: 8px; }
.m-dl-overlay .sheet-head h3 { font-size: 16px; font-weight: 740; }
.m-dl-overlay .sheet-close {
  width: 30px;
  height: 30px;
  border-radius: 999px;
  background: var(--fill);
  color: var(--ink-2);
  display: grid;
  place-items: center;
  font-size: 14px;
}
.m-dl-overlay .fc-ico.green.sm {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  background: var(--ok-soft);
  color: var(--ok);
}
.m-dl-overlay .fc-ico svg { fill: none; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; stroke-linejoin: round; }
.m-dl-overlay .steps-list {
  border-radius: 16px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  padding: 13px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.m-dl-overlay .step-row { display: flex; align-items: center; gap: 10px; }
.m-dl-overlay .step-idx {
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
.m-dl-overlay .step-row strong { font-size: 13px; font-weight: 700; word-break: break-all; }
.m-dl-overlay .sheet-meta { font-size: 12px; color: var(--ink-3); }
.m-dl-overlay .sheet-changelog-head {
  margin-top: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 700;
}
.m-dl-overlay .sheet-changelog-head span:last-child { font-weight: 500; color: var(--ink-3); }
.m-dl-overlay .changelog-ul {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 9px 0 14px;
}
.m-dl-overlay .changelog-ul li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 12.5px;
  color: var(--ink-2);
  line-height: 1.55;
}
.m-dl-overlay .changelog-ul li::before {
  content: "";
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--coral);
  margin-top: 8px;
  flex: none;
}
.m-dl-overlay .sha-box {
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
.m-dl-overlay .sha-code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  color: var(--ink-2);
  word-break: break-all;
}
.m-dl-overlay .sha-copy {
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
.m-dl-overlay .btn-primary-dl.compact,
.wx-guide-mask .btn-primary-dl.compact {
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
  gap: 8px;
  box-shadow: 0 10px 24px rgba(242, 92, 114, 0.3);
}
.wx-guide-mask .btn-primary-dl.compact svg { fill: none; stroke: currentColor; stroke-width: 1.9; stroke-linecap: round; stroke-linejoin: round; }

/* 微信遮罩引导 */
.wx-guide-mask {
  --canvas-soft: #f5f1eb;
  --surface: #ffffff;
  --ink: #19171a;
  --line: rgba(28, 22, 28, 0.08);
  --line-strong: rgba(28, 22, 28, 0.15);

  position: fixed;
  inset: 0;
  z-index: 210;
  background: rgba(16, 14, 20, 0.78);
  backdrop-filter: blur(5px);
  -webkit-backdrop-filter: blur(5px);
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
.wx-arrow-hint {
  position: fixed;
  top: calc(14px + env(safe-area-inset-top, 0px));
  right: 18px;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 5px;
  color: #fff;
}
.wx-arrow-hint span {
  font-size: 12.5px;
  font-weight: 700;
  padding: 5px 12px;
  border-radius: 999px;
  background: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  box-shadow: 0 8px 20px rgba(242, 92, 114, 0.42);
}
.wx-card {
  width: 100%;
  max-width: 360px;
  border-radius: 24px;
  background: var(--surface);
  color: var(--ink);
  padding: 22px 18px 18px;
  box-shadow: 0 24px 56px rgba(0, 0, 0, 0.38);
}
.wx-card h3 { font-size: 17px; font-weight: 760; letter-spacing: -0.02em; }
.wx-steps { margin: 14px 0 16px; display: flex; flex-direction: column; gap: 10px; }
.wx-step {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 12px;
  border-radius: 14px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
}
.wx-step-n {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  background: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  color: #fff;
  display: grid;
  place-items: center;
  font-size: 12px;
  font-weight: 750;
  flex: none;
}
.wx-step strong { font-size: 13.5px; font-weight: 700; }
.wx-guide-mask .btn-secondary.sm {
  width: 100%;
  height: 40px;
  border-radius: 14px;
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line-strong);
  font-size: 12.5px;
  font-weight: 650;
}
</style>
