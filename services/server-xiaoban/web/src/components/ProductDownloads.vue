<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getDesktopProduct, targets, type DesktopProduct, type DesktopProductDownload } from '@/desktop'
import { toast } from '@/composables/toast'

const props = withDefaults(defineProps<{
  standalone?: boolean
}>(), {
  standalone: false,
})

const emit = defineEmits<{
  loaded: [info: { version: string; publishedAt: string } | null]
}>()

interface PreviewSlide {
  label: string
  img?: string
  imgs?: [string, string]
  title: string
  sub: string
}

interface PlatformMeta {
  cls: 'os-mac' | 'os-win' | 'os-and'
  modeCls: 'os-mac-mode' | 'os-win-mode' | 'os-and-mode'
  pill: string
  title: string
  short: string
  ext: string
  desc: string
  caps: string[]
  tip: string
  isMobile?: boolean
  previews: PreviewSlide[]
}

const platformMeta: Record<string, PlatformMeta> = {
  'darwin-aarch64': {
    cls: 'os-mac',
    modeCls: 'os-mac-mode',
    pill: 'macOS 12+ · Apple 芯片 (M1–M4)',
    title: '小伴桌面 · macOS 版',
    short: 'macOS',
    ext: '.dmg',
    desc: '为 Apple Silicon 原生打造：全能自主 Agent 支持控电脑与 ADB 控手机，内置无限设计画布直出 .pptx / .fig，更附带超级小伴悬浮截图标注、3D 龙鱼特效与 8 款实时组件。',
    caps: ['控电脑 / ADB 控手机', '无限画布直出 PPTX & Figma', '超级小伴全局截图标注', '3D 龙鱼特效 & 股票组件'],
    tip: '拖入「应用程序」即可安装；首次打开若系统提示未公证，右键点击图标选择「打开」一次即可正常使用。',
    previews: [
      { label: '多模型 & 控电脑', img: '/desktop/desktop-config-ai.webp', title: '小伴桌面 v3.1.8 · 原生独立配置窗', sub: '小伴 AI + 自有模型 · 启用电脑屏幕与应用控制' },
      { label: '3D 龙鱼特效', img: '/desktop/desktop-dragon.webp', title: '14 种透明特效 · 3D 朱红龙鱼巡游', sub: '实时调鱼群数量、光照与游速' },
      { label: '8 款桌面组件', img: '/desktop/desktop-widgets.webp', title: '多屏实时桌面组件', sub: '股票行情 · 电子木鱼 · 农历月历 · 日程天气' },
    ],
  },
  'darwin-x86_64': {
    cls: 'os-mac',
    modeCls: 'os-mac-mode',
    pill: 'macOS 12+ · Intel 芯片',
    title: '小伴桌面 · macOS Intel 版',
    short: 'macOS Intel',
    ext: '.dmg',
    desc: '面向 Intel 芯片 Mac 的原生安装包：完整支持全能自主 Agent、无限设计画布、超级小伴悬浮截图标注、3D 龙鱼特效与实时桌面组件。',
    caps: ['控电脑 / ADB 控手机', '无限画布直出 PPTX & Figma', '超级小伴全局截图标注', '3D 龙鱼特效 & 股票组件'],
    tip: '拖入「应用程序」即可安装；首次打开若系统提示未公证，右键点击图标选择「打开」一次即可。',
    previews: [
      { label: '多模型 & 控电脑', img: '/desktop/desktop-config-ai.webp', title: '小伴桌面 · 原生独立配置窗', sub: '小伴 AI + 自有模型 · 启用电脑屏幕与应用控制' },
      { label: '3D 龙鱼特效', img: '/desktop/desktop-dragon.webp', title: '14 种透明特效 · 3D 朱红龙鱼巡游', sub: '实时调鱼群数量、光照与游速' },
    ],
  },
  'windows-x86_64': {
    cls: 'os-win',
    modeCls: 'os-win-mode',
    pill: 'Windows 10 / 11 · 64 位 NSIS',
    title: '小伴桌面 · Windows 版',
    short: 'Windows',
    ext: '.exe',
    desc: '免管理员权限一键安装至当前用户目录：完整拥有全能 Agent 控电脑/控手机、无限画布导出 PPTX/Figma、腾讯文档/百度网盘直连，以及多屏动态壁纸与 3D 龙鱼特效。',
    caps: ['免管理员权限纯净安装', '原生控屏 + ADB 控手机', '无限画布直出 PPTX & Figma', '多屏 3D 龙鱼 & 实时组件'],
    tip: '全程无管理员 UAC 弹窗、卸载干净无残留；需 WebView2 运行时（Windows 11 默认已自带）。',
    previews: [
      { label: '8 款桌面组件', img: '/desktop/desktop-widgets.webp', title: '多屏实时桌面组件 · Windows 64 位', sub: '股票行情 · 电子木鱼 · 农历休/班日历' },
      { label: '外部 Agent 协作', img: '/desktop/desktop-config-agent.webp', title: '连接 Claude Code / Codex + 本机语音 MCP', sub: '设计画板协同 · Hooks 观察层 · SenseVoice 本地接口' },
      { label: 'AI 专家市场', img: '/desktop/desktop-plugins-v318.webp', title: '小伴市场 & SkillHub 专家技能', sub: '一键安装测试专家、腾讯文档、3D 游戏工坊' },
    ],
  },
  'android-arm64': {
    cls: 'os-and',
    modeCls: 'os-and-mode',
    pill: 'Android 10+ · arm64 官方签名包',
    title: '小伴手机 App · Android',
    short: 'Android 手机',
    ext: '.apk',
    desc: '端侧原生 Compose + Rust Agent 打造：直接在手机上造 3D 游戏、端侧离线实时语音通话、远程查看并审批电脑权限，还内置原生超级小伴社区、万能红外遥控与免登录多模型配置。',
    caps: ['本地 Agent 造 3D 游戏', '端侧离线实时语音通话', '远程控电脑 & 审批权限', '原生社区 & 免登录多模型'],
    tip: '下载官方正式签名 APK 直接安装；后续支持应用内校验签名后覆盖升级，无需卸载旧版。',
    isMobile: true,
    previews: [
      { label: '造作品 & 语音通话', imgs: ['/desktop/mobile-chat-build.webp', '/desktop/mobile-voice-call.webp'], title: '手机本地造作品校验 + 端侧离线实时语音', sub: '实时步骤清单 · Silero+SenseVoice 开口即打断' },
      { label: '遥控电脑 & 权限审批', imgs: ['/desktop/mobile-chat-remote.webp', '/desktop/mobile-chat-approve.webp'], title: '远程遥控电脑 + 手机审批系统权限', sub: '实时文件 Diff 与生图进度 · 一键批准/拒绝写入' },
      { label: '原生社区 & 自由模型', imgs: ['/desktop/mobile-community-new.webp', '/desktop/mobile-model-deepseek.webp'], title: '原生超级小伴社区 + 免登录自定义多模型', sub: '创意作品/排行榜 · 第三方密钥仅保存在本机' },
    ],
  },
}

function detectPlatformId(): string {
  if (typeof navigator === 'undefined') return 'darwin-aarch64'
  const ua = navigator.userAgent || ''
  if (/Android/i.test(ua)) return 'android-arm64'
  if (/Mac OS X|Macintosh/i.test(ua)) return 'darwin-aarch64'
  return 'windows-x86_64'
}

const product = ref<DesktopProduct | null>(null)
const loading = ref(true)
const error = ref('')
const detectedId = ref(detectPlatformId())
const selectedId = ref(detectedId.value)
const previewIdx = ref(0)
const modalTargetId = ref<string | null>(null)

const visibleTargets = computed(() =>
  targets.filter(t => t.id !== 'darwin-x86_64' || Boolean(product.value?.downloads[t.id])),
)

const activeTarget = computed(() => {
  const list = visibleTargets.value
  return list.find(t => t.id === selectedId.value) ?? list[0]
})

const activeMeta = computed(() =>
  platformMeta[activeTarget.value?.id || 'darwin-aarch64'] ?? platformMeta['darwin-aarch64'],
)

const activeDownload = computed<DesktopProductDownload | null>(() => {
  const id = activeTarget.value?.id
  if (!id || !product.value) return null
  return product.value.downloads[id] ?? null
})

const activePreview = computed(() => {
  const list = activeMeta.value.previews
  return list[previewIdx.value] ?? list[0]
})

const modalTarget = computed(() =>
  modalTargetId.value ? targets.find(t => t.id === modalTargetId.value) ?? null : null,
)
const modalMeta = computed(() =>
  modalTargetId.value ? platformMeta[modalTargetId.value] ?? platformMeta['darwin-aarch64'] : null,
)
const modalDownload = computed(() =>
  modalTargetId.value && product.value ? product.value.downloads[modalTargetId.value] ?? null : null,
)

function selectPlatform(id: string) {
  selectedId.value = id
  previewIdx.value = 0
}

function formatDate(iso?: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

function formatBytes(bytes?: number): string {
  if (!bytes || bytes <= 0) return ''
  const mb = bytes / (1024 * 1024)
  return mb >= 100 ? `${mb.toFixed(0)} MB` : `${mb.toFixed(1)} MB`
}

function noteLines(notes?: string): string[] {
  const raw = (notes || '体验优化与问题修复。').trim()
  return raw
    .split(/\r?\n/)
    .map(line => line.replace(/^[-*•·]\s*/, '').trim())
    .filter(Boolean)
}

async function copySha(sha?: string) {
  if (!sha) return
  try {
    await navigator.clipboard.writeText(sha)
    toast('已复制 SHA-256 校验值')
  } catch {
    toast('复制失败，请手动复制')
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && modalTargetId.value) modalTargetId.value = null
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    product.value = await getDesktopProduct()
    const list = visibleTargets.value
    const exact = list.find(t => t.id === detectedId.value && product.value?.downloads[t.id])
    const fallback = list.find(t => Boolean(product.value?.downloads[t.id])) ?? list[0]
    selectedId.value = (exact ?? fallback)?.id ?? 'darwin-aarch64'
    const rec = product.value.downloads[selectedId.value] ?? null
    emit('loaded', rec ? { version: rec.version, publishedAt: rec.publishedAt } : null)
  } catch {
    error.value = '下载清单暂时无法加载，请稍后重试。'
    emit('loaded', null)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  load()
  window.addEventListener('keydown', onKeydown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <section id="desktop-downloads" class="downloads" :class="{ standalone }" aria-labelledby="downloads-title">
    <!-- Embedded Header (Home Page) -->
    <div v-if="!standalone" class="embed-head">
      <div>
        <span class="eyebrow">GET XIAOBAN</span>
        <h2 id="downloads-title">把小伴装进你的设备</h2>
      </div>
      <p>电脑端负责深度工作、控电脑/控手机与活桌面，Android 手机端内置本地造作品、端侧语音与远程遥控。</p>
    </div>

    <!-- Centered 3-Platform Switcher (Standalone /download) -->
    <div v-if="standalone" class="os-switcher-wrap">
      <div class="os-switcher" role="tablist" aria-label="选择平台">
        <button
          v-for="t in visibleTargets"
          :key="t.id"
          type="button"
          role="tab"
          class="os-tab"
          :class="{ active: activeTarget?.id === t.id }"
          :aria-selected="activeTarget?.id === t.id"
          @click="selectPlatform(t.id)"
        >
          <svg v-if="t.id.startsWith('darwin')" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.7-2.8-.7-1.4 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7 1.3 0 1.7.7 2.8.7 1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.4-.1 0-2.5-.9-2.5-3.5zM14.4 6.3c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 2-.5 2.5-1.2z" /></svg>
          <svg v-else-if="t.id.startsWith('windows')" width="15" height="15" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M3 5.5 10.6 4.4v7.1H3V5.5zm8.7-1.3L21 3v8.5h-9.3V4.2zM3 12.6h7.6v7.1L3 18.6v-6zm8.7 0H21V21l-9.3-1.3v-7.1z" /></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M17.5 9.4l1.5-2.6a.45.45 0 0 0-.16-.62.45.45 0 0 0-.62.16l-1.54 2.67A9.4 9.4 0 0 0 12 7.8a9.4 9.4 0 0 0-4.68 1.19L5.78 6.32a.45.45 0 0 0-.62-.16.45.45 0 0 0-.16.62l1.5 2.6C4.1 10.7 2.5 13.1 2.2 16h19.6c-.3-2.9-1.9-5.3-4.3-6.6zM8.5 13.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2zm7 0a1 1 0 1 1 0-2 1 1 0 0 1 0 2z" /></svg>
          <span>{{ platformMeta[t.id]?.short ?? t.label }}</span>
          <span v-if="t.id === detectedId" class="rec-tag">推荐</span>
        </button>
      </div>
    </div>

    <!-- Loading Skeleton -->
    <template v-if="loading">
      <div class="state-card" role="status">
        <div>
          <strong>正在拉取各平台最新版本…</strong>
          <p>正在请求官方发布清单与校验信息</p>
        </div>
        <span class="state-tag">LOADING</span>
      </div>
      <div class="skel-stage" aria-hidden="true" />
    </template>

    <!-- Error State -->
    <div v-else-if="error" class="state-card err" role="alert">
      <div>
        <strong>{{ error }}</strong>
        <p>请检查网络连接后重试，或前往小伴社区获取最新动态。</p>
      </div>
      <button type="button" class="cta-coral sm" @click="load">重新加载 ↻</button>
    </div>

    <!-- Ready State -->
    <template v-else>
      <!-- Bright Glass Two-Column Main Download Stage -->
      <section v-if="activeTarget" class="dl-stage-card" aria-label="所选平台下载详情">
        <div class="dl-stage-left">
          <span class="dl-os-pill" :class="activeMeta.cls">
            <svg v-if="activeTarget.id.startsWith('darwin')" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.7-2.8-.7-1.4 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7 1.3 0 1.7.7 2.8.7 1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.4-.1 0-2.5-.9-2.5-3.5zM14.4 6.3c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 2-.5 2.5-1.2z" /></svg>
            <svg v-else-if="activeTarget.id.startsWith('windows')" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M3 5.5 10.6 4.4v7.1H3V5.5zm8.7-1.3L21 3v8.5h-9.3V4.2zM3 12.6h7.6v7.1L3 18.6v-6zm8.7 0H21V21l-9.3-1.3v-7.1z" /></svg>
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M17.5 9.4l1.5-2.6a.45.45 0 0 0-.16-.62.45.45 0 0 0-.62.16l-1.54 2.67A9.4 9.4 0 0 0 12 7.8a9.4 9.4 0 0 0-4.68 1.19L5.78 6.32a.45.45 0 0 0-.62-.16.45.45 0 0 0-.16.62l1.5 2.6C4.1 10.7 2.5 13.1 2.2 16h19.6c-.3-2.9-1.9-5.3-4.3-6.6zM8.5 13.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2zm7 0a1 1 0 1 1 0-2 1 1 0 0 1 0 2z" /></svg>
            <span>{{ activeMeta.pill }}</span>
          </span>

          <h2>{{ activeMeta.title }}</h2>
          <p class="dl-stage-desc">{{ activeMeta.desc }}</p>

          <div class="dl-cap-grid" :class="activeMeta.modeCls">
            <div v-for="cap in activeMeta.caps" :key="cap" class="dl-cap-chip">
              <span class="dl-cap-dot" aria-hidden="true" />
              <span>{{ cap }}</span>
            </div>
          </div>

          <template v-if="activeDownload">
            <div class="dl-action-row">
              <a :href="activeDownload.url" class="cta-coral">
                <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                <span>下载 {{ activeMeta.short }} 版</span>
                <span class="dl-btn-meta">
                  <template v-if="formatBytes(activeDownload.bytes)">{{ formatBytes(activeDownload.bytes) }} · </template>{{ activeMeta.ext }}
                </span>
              </a>
              <button type="button" class="cta-soft" @click="modalTargetId = activeTarget.id">更新日志</button>
            </div>

            <div class="dl-trust-strip">
              <span>版本 <strong>v{{ activeDownload.version }}</strong></span>
              <span aria-hidden="true">·</span>
              <span>发布于 <strong>{{ formatDate(activeDownload.publishedAt) }}</strong></span>
              <button
                v-if="activeDownload.sha256"
                type="button"
                class="dl-sha-btn"
                @click="copySha(activeDownload.sha256)"
              >
                SHA-256: {{ activeDownload.sha256.slice(0, 10) }}… 复制
              </button>
            </div>
          </template>
          <p v-else class="dl-empty-tip">该平台安装包发布后即可下载，你可以在下方切换其他已发布的平台版本。</p>

          <div class="dl-install-tip">
            <span class="dl-tip-ico" aria-hidden="true">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4"><circle cx="12" cy="12" r="9" /><path d="M12 8v4" /><path d="M12 16h.01" /></svg>
            </span>
            <span>{{ activeMeta.tip }}</span>
          </div>
        </div>

        <div class="dl-stage-right" :class="activeMeta.cls">
          <div class="dl-preview-tabs" role="tablist">
            <button
              v-for="(pv, idx) in activeMeta.previews"
              :key="pv.label"
              type="button"
              role="tab"
              :class="{ active: idx === previewIdx }"
              :aria-selected="idx === previewIdx"
              @click="previewIdx = idx"
            >
              {{ pv.label }}
            </button>
          </div>

          <div v-if="activeMeta.isMobile && activePreview.imgs" class="stage-phone-pair">
            <div class="mini-phone">
              <img :src="activePreview.imgs[0]" :alt="`${activePreview.title} 1`">
            </div>
            <div class="mini-phone lift">
              <img :src="activePreview.imgs[1]" :alt="`${activePreview.title} 2`">
            </div>
          </div>
          <div v-else class="stage-desk-frame">
            <img :src="activePreview.img" :alt="activePreview.title">
          </div>

          <div class="float-pill-card">
            <div>
              <strong>{{ activePreview.title }}</strong>
              <div class="float-sub">{{ activePreview.sub }}</div>
            </div>
            <span class="float-tag">官方原包</span>
          </div>
        </div>
      </section>

      <!-- Compact All-Platforms Quick Strip -->
      <section class="quick-strip-section" aria-label="全部平台直达">
        <div class="quick-strip-label">
          <strong>全部平台直达</strong>
          <span>点击卡片切换预览，或点击右侧图标直接下载安装包</span>
        </div>
        <div class="quick-strip">
          <div
            v-for="t in visibleTargets"
            :key="t.id"
            class="quick-item"
            :class="{ active: activeTarget?.id === t.id }"
            @click="selectPlatform(t.id)"
          >
            <span class="qi-ico" :class="platformMeta[t.id]?.cls">
              <svg v-if="t.id.startsWith('darwin')" width="19" height="19" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.7-2.8-.7-1.4 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7 1.3 0 1.7.7 2.8.7 1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.4-.1 0-2.5-.9-2.5-3.5zM14.4 6.3c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 2-.5 2.5-1.2z" /></svg>
              <svg v-else-if="t.id.startsWith('windows')" width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M3 5.5 10.6 4.4v7.1H3V5.5zm8.7-1.3L21 3v8.5h-9.3V4.2zM3 12.6h7.6v7.1L3 18.6v-6zm8.7 0H21V21l-9.3-1.3v-7.1z" /></svg>
              <svg v-else width="19" height="19" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M17.5 9.4l1.5-2.6a.45.45 0 0 0-.16-.62.45.45 0 0 0-.62.16l-1.54 2.67A9.4 9.4 0 0 0 12 7.8a9.4 9.4 0 0 0-4.68 1.19L5.78 6.32a.45.45 0 0 0-.62-.16.45.45 0 0 0-.16.62l1.5 2.6C4.1 10.7 2.5 13.1 2.2 16h19.6c-.3-2.9-1.9-5.3-4.3-6.6zM8.5 13.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2zm7 0a1 1 0 1 1 0-2 1 1 0 0 1 0 2z" /></svg>
            </span>
            <div class="qi-meta">
              <strong>
                {{ platformMeta[t.id]?.short ?? t.label }}
                <template v-if="product?.downloads[t.id]"> · v{{ product.downloads[t.id].version }}</template>
              </strong>
              <span>
                <template v-if="product?.downloads[t.id]">
                  <template v-if="formatBytes(product.downloads[t.id].bytes)">{{ formatBytes(product.downloads[t.id].bytes) }} · </template>{{ platformMeta[t.id]?.ext }}
                </template>
                <template v-else>即将发布</template>
              </span>
            </div>
            <a
              v-if="product?.downloads[t.id]"
              :href="product.downloads[t.id].url"
              class="qi-dl"
              :title="`立即下载 ${platformMeta[t.id]?.short ?? t.label}`"
              @click.stop
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
            </a>
          </div>
        </div>
      </section>

      <!-- Official Announcements Ribbon (if published) -->
      <div v-if="product?.announcements?.length" class="notice-list">
        <div v-for="notice in product.announcements" :key="notice.id" class="notice-ribbon">
          <span class="nr-tag">官方公告</span>
          <p><strong>{{ notice.title }}：</strong>{{ notice.body }}</p>
        </div>
      </div>
    </template>

    <!-- Changelog Modal -->
    <Teleport to="body">
      <div v-if="modalTargetId && modalDownload" class="modal-mask" @click.self="modalTargetId = null">
        <div class="modal-card" role="dialog" aria-modal="true" :aria-label="`${modalMeta?.title ?? modalTarget?.label} 更新说明`">
          <div class="modal-head">
            <h3>{{ modalMeta?.title ?? modalTarget?.label }} · 更新说明</h3>
            <button type="button" class="modal-close" aria-label="关闭" @click="modalTargetId = null">✕</button>
          </div>
          <div class="modal-body">
            <div class="modal-meta">
              <span>v{{ modalDownload.version }}</span>
              <span>{{ formatDate(modalDownload.publishedAt) }}</span>
              <span v-if="formatBytes(modalDownload.bytes)">{{ formatBytes(modalDownload.bytes) }}</span>
              <span v-if="modalMeta?.ext">{{ modalMeta.ext }}</span>
            </div>
            <ul class="modal-notes">
              <li v-for="(line, i) in noteLines(modalDownload.notes)" :key="i">{{ line }}</li>
            </ul>
          </div>
          <div class="modal-foot">
            <button
              v-if="modalDownload.sha256"
              type="button"
              class="dl-sha-btn"
              @click="copySha(modalDownload.sha256)"
            >
              复制 SHA-256
            </button>
            <span v-else />
            <a :href="modalDownload.url" class="cta-coral sm" @click="modalTargetId = null">立即下载</a>
          </div>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.downloads {
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
  --win-bg: #eef5ff;
  --win-ink: #0a6ed1;
  --mac-bg: #fff0f2;
  --mac-ink: #e24666;
  --and-bg: #eaf9f0;
  --and-ink: #158f4d;
  --line: rgba(28, 22, 28, 0.075);
  --line-strong: rgba(28, 22, 28, 0.14);
  --shadow-sm: 0 2px 10px rgba(26, 18, 24, 0.04);
  --shadow-md: 0 16px 42px rgba(26, 18, 24, 0.07);
  --ease: cubic-bezier(0.22, 1, 0.36, 1);
  --mono: ui-monospace, 'SFMono-Regular', Menlo, monospace;

  color: var(--ink);
  padding: 20px 0 12px;
}
.downloads.standalone {
  padding-top: 0;
}

.embed-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 24px;
}
.eyebrow {
  font-size: 11px;
  font-weight: 750;
  letter-spacing: 0.14em;
  color: var(--coral);
  text-transform: uppercase;
}
.embed-head h2 {
  margin-top: 6px;
  font-size: clamp(26px, 3vw, 36px);
  letter-spacing: -0.035em;
  font-weight: 750;
}
.embed-head p {
  font-size: 14px;
  color: var(--ink-2);
  max-width: 440px;
  line-height: 1.7;
}

.os-switcher-wrap {
  display: flex;
  justify-content: center;
  margin-bottom: 26px;
}
.os-switcher {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(28, 22, 28, 0.09);
  box-shadow: 0 8px 28px rgba(26, 18, 24, 0.06);
  backdrop-filter: blur(12px);
}
.os-tab {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  height: 44px;
  padding: 0 18px;
  border-radius: 13px;
  font-size: 14px;
  font-weight: 600;
  color: var(--ink-2);
  transition: background 0.2s, color 0.2s, box-shadow 0.2s;
}
.os-tab:hover {
  color: var(--ink);
  background: rgba(28, 22, 28, 0.04);
}
.os-tab.active {
  background: var(--ink);
  color: #fff;
  box-shadow: 0 6px 18px rgba(25, 23, 26, 0.18);
}
.rec-tag {
  font-size: 10.5px;
  font-weight: 700;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--coral-soft);
  color: #d83f5d;
}
.os-tab.active .rec-tag {
  background: rgba(255, 118, 138, 0.28);
  color: #ffd4dc;
}

/* Main Bright Glass Stage */
.dl-stage-card {
  border-radius: 30px;
  background: rgba(255, 255, 255, 0.94);
  border: 1px solid rgba(255, 255, 255, 0.96);
  box-shadow: 0 28px 80px rgba(34, 20, 26, 0.09), 0 0 0 1px rgba(28, 22, 28, 0.06);
  padding: 16px;
  display: grid;
  grid-template-columns: 0.96fr 1.04fr;
  gap: 16px;
  align-items: stretch;
}
.dl-stage-left {
  padding: 28px 26px 26px 30px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.dl-os-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  align-self: flex-start;
  padding: 6px 13px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.dl-os-pill.os-mac { background: var(--mac-bg); color: var(--mac-ink); }
.dl-os-pill.os-win { background: var(--win-bg); color: var(--win-ink); }
.dl-os-pill.os-and { background: var(--and-bg); color: var(--and-ink); }

.dl-stage-left h2 {
  margin-top: 16px;
  font-size: clamp(26px, 2.7vw, 33px);
  line-height: 1.2;
  letter-spacing: -0.03em;
  font-weight: 750;
}
.dl-stage-desc {
  margin-top: 10px;
  font-size: 14.5px;
  color: var(--ink-2);
  line-height: 1.72;
}

.dl-cap-grid {
  margin-top: 18px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.dl-cap-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-radius: 12px;
  background: var(--canvas);
  border: 1px solid var(--line);
  font-size: 12.5px;
  font-weight: 600;
  color: var(--ink);
}
.dl-cap-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--coral-grad);
}
.os-win-mode .dl-cap-dot { background: linear-gradient(135deg, #2563eb, #0a6ed1); }
.os-and-mode .dl-cap-dot { background: linear-gradient(135deg, #16a34a, #0d9488); }

.dl-action-row {
  margin-top: 22px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}
.cta-coral {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 50px;
  padding: 0 26px;
  border-radius: 14px;
  background: var(--coral-grad);
  color: #fff;
  font-size: 15px;
  font-weight: 700;
  box-shadow: var(--coral-shadow);
  transition: transform 0.16s var(--ease), filter 0.18s, box-shadow 0.2s;
  white-space: nowrap;
}
.cta-coral:hover {
  filter: brightness(1.05);
  transform: translateY(-1px);
  box-shadow: 0 18px 40px rgba(247, 102, 94, 0.36);
}
.cta-coral.sm {
  height: 40px;
  padding: 0 20px;
  font-size: 13.5px;
}
.dl-btn-meta {
  font-size: 12.5px;
  font-weight: 600;
  opacity: 0.88;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.34);
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

.dl-trust-strip {
  margin-top: 16px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
  font-size: 12.5px;
  color: var(--ink-3);
}
.dl-trust-strip strong { color: var(--ink-2); font-weight: 650; }
.dl-sha-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-family: var(--mono);
  font-size: 11.5px;
  padding: 3px 9px;
  border-radius: 7px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  color: var(--ink-2);
  transition: 0.16s;
}
.dl-sha-btn:hover {
  border-color: var(--coral-border);
  color: var(--coral);
  background: var(--coral-soft);
}
.dl-empty-tip {
  margin-top: 18px;
  font-size: 13.5px;
  color: var(--ink-3);
}

.dl-install-tip {
  margin-top: 20px;
  padding: 13px 15px;
  border-radius: 14px;
  background: linear-gradient(135deg, #fff6f4 0%, #faf7f3 100%);
  border: 1px solid rgba(242, 96, 114, 0.14);
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-size: 12.8px;
  color: var(--ink-2);
  line-height: 1.65;
}
.dl-tip-ico {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  background: #fff;
  color: var(--coral);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  margin-top: 1px;
  box-shadow: 0 2px 6px rgba(242, 96, 114, 0.14);
}

/* Right Preview Canvas */
.dl-stage-right {
  border-radius: 22px;
  padding: 20px 22px 18px;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 14px;
}
.dl-stage-right.os-mac {
  background:
    radial-gradient(85% 90% at 88% 8%, rgba(255, 112, 136, 0.24), transparent 62%),
    radial-gradient(70% 80% at 10% 95%, rgba(249, 152, 86, 0.20), transparent 60%),
    #f8f1ee;
}
.dl-stage-right.os-win {
  background:
    radial-gradient(85% 90% at 88% 8%, rgba(76, 148, 255, 0.22), transparent 62%),
    radial-gradient(70% 80% at 10% 95%, rgba(255, 128, 146, 0.16), transparent 60%),
    #eef4fb;
}
.dl-stage-right.os-and {
  background:
    radial-gradient(85% 90% at 85% 10%, rgba(255, 108, 134, 0.22), transparent 62%),
    radial-gradient(75% 80% at 12% 90%, rgba(44, 196, 126, 0.20), transparent 60%),
    #f2f5f2;
}

.dl-preview-tabs {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 4px;
  align-self: flex-start;
  padding: 4px;
  border-radius: 11px;
  background: rgba(255, 255, 255, 0.78);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.9);
}
.dl-preview-tabs button {
  height: 28px;
  padding: 0 11px;
  border-radius: 7px;
  font-size: 12px;
  font-weight: 620;
  color: var(--ink-2);
  transition: 0.16s;
}
.dl-preview-tabs button.active {
  background: var(--ink);
  color: #fff;
}

.stage-desk-frame {
  border-radius: 15px;
  overflow: hidden;
  background: #fff;
  border: 1px solid rgba(255, 255, 255, 0.88);
  box-shadow: 0 22px 48px rgba(28, 18, 24, 0.14);
}
.stage-desk-frame img {
  width: 100%;
  height: auto;
  display: block;
}
.stage-phone-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  align-items: center;
  max-width: 420px;
  margin: 2px auto;
}
.mini-phone {
  padding: 5px;
  border-radius: 26px;
  background: #19171a;
  box-shadow: 0 22px 44px rgba(26, 16, 22, 0.22);
}
.mini-phone.lift { transform: translateY(-6px); }
.mini-phone img { width: 100%; border-radius: 21px; display: block; }

.float-pill-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 13px;
  background: rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.94);
  box-shadow: 0 8px 22px rgba(28, 18, 24, 0.06);
  font-size: 12.2px;
  color: var(--ink-2);
}
.float-pill-card strong { color: var(--ink); font-weight: 680; }
.float-sub { font-size: 11.5px; color: var(--ink-3); margin-top: 1px; }
.float-tag { font-size: 11.5px; font-weight: 700; color: var(--coral); white-space: nowrap; }

/* Quick Strip */
.quick-strip-section { margin-top: 24px; }
.quick-strip-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 11px;
  padding: 0 4px;
  font-size: 13px;
  color: var(--ink-3);
}
.quick-strip-label strong { color: var(--ink); font-weight: 680; }

.quick-strip {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}
.quick-item {
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 15px 16px;
  border-radius: 18px;
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  transition: border-color 0.2s, transform 0.2s var(--ease), box-shadow 0.2s;
}
.quick-item:hover {
  border-color: var(--coral-border);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}
.quick-item.active {
  border-color: var(--coral);
  background: linear-gradient(180deg, #fff9f8 0%, #fff 100%);
}
.qi-ico {
  width: 40px;
  height: 40px;
  border-radius: 11px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}
.qi-ico.os-win { background: var(--win-bg); color: var(--win-ink); }
.qi-ico.os-mac { background: var(--mac-bg); color: var(--mac-ink); }
.qi-ico.os-and { background: var(--and-bg); color: var(--and-ink); }
.qi-meta { min-width: 0; flex: 1; }
.qi-meta strong { display: block; font-size: 14px; font-weight: 700; color: var(--ink); }
.qi-meta span { display: block; font-size: 12px; color: var(--ink-3); margin-top: 2px; }
.qi-dl {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--canvas-soft);
  color: var(--ink);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  transition: 0.18s;
}
.quick-item:hover .qi-dl,
.quick-item.active .qi-dl {
  background: var(--coral-grad);
  color: #fff;
}

/* Announcements */
.notice-list {
  margin-top: 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.notice-ribbon {
  padding: 15px 22px;
  border-radius: 18px;
  background: linear-gradient(135deg, #fff3ef 0%, #fffaf7 100%);
  border: 1px solid rgba(242, 96, 114, 0.18);
  display: flex;
  align-items: center;
  gap: 14px;
}
.nr-tag {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 10px;
  border-radius: 999px;
  background: var(--coral-grad);
  color: #fff;
  flex-shrink: 0;
}
.notice-ribbon p { font-size: 13.5px; color: var(--ink-2); flex: 1; }
.notice-ribbon strong { color: var(--ink); }

/* States */
.state-card {
  padding: 32px;
  border-radius: 24px;
  background: var(--surface);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  box-shadow: var(--shadow-sm);
}
.state-card p { font-size: 13.5px; color: var(--ink-3); margin-top: 4px; }
.state-card.err { background: #fff5f5; border-color: rgba(220, 68, 68, 0.24); }
.state-card.err strong { color: #b83232; }
.state-tag { font-size: 12px; font-weight: 700; color: var(--coral); letter-spacing: 0.1em; }
.skel-stage {
  margin-top: 16px;
  height: 340px;
  border-radius: 26px;
  background: linear-gradient(90deg, #f3efea 25%, #faf8f5 50%, #f3efea 75%);
  background-size: 200% 100%;
  animation: shimmer 1.4s infinite;
}
@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

@media (max-width: 960px) {
  .dl-stage-card, .quick-strip { grid-template-columns: 1fr; }
  .embed-head { flex-direction: column; align-items: flex-start; gap: 10px; }
}
@media (max-width: 768px) {
  .os-switcher { width: 100%; display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px; padding: 5px; }
  .os-tab { justify-content: center; padding: 0 6px; font-size: 12.5px; height: 40px; }
  .rec-tag { display: none; }
  .dl-stage-card { padding: 12px; border-radius: 22px; }
  .dl-stage-left { padding: 14px 10px 8px; }
  .dl-cap-grid { grid-template-columns: 1fr; }
  .dl-action-row { flex-direction: column; align-items: stretch; }
  .cta-coral, .cta-soft { width: 100%; height: 46px; }
  .dl-stage-right { padding: 14px; border-radius: 16px; }
  .quick-strip-label { flex-direction: column; align-items: flex-start; gap: 4px; }
  .notice-ribbon { flex-direction: column; align-items: flex-start; gap: 8px; }
}
</style>

<style>
.modal-mask {
  position: fixed;
  inset: 0;
  z-index: 300;
  background: rgba(20, 16, 22, 0.45);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.modal-card {
  width: min(500px, 100%);
  background: #fff;
  color: #19171a;
  border-radius: 24px;
  border: 1px solid rgba(28, 22, 28, 0.08);
  box-shadow: 0 28px 72px rgba(26, 18, 24, 0.22);
  overflow: hidden;
}
.modal-head {
  padding: 20px 24px;
  border-bottom: 1px solid rgba(28, 22, 28, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.modal-head h3 { font-size: 17px; font-weight: 720; }
.modal-close {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: #837d86;
}
.modal-close:hover { background: #f6f2ec; color: #19171a; }
.modal-body { padding: 20px 24px 24px; max-height: 58vh; overflow-y: auto; }
.modal-meta { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 16px; }
.modal-meta span {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 999px;
  background: #f6f2ec;
  color: #524d54;
  font-weight: 550;
}
.modal-notes {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 14px;
  color: #524d54;
  line-height: 1.7;
}
.modal-notes li { display: flex; gap: 9px; }
.modal-notes li::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #ee4f6d;
  margin-top: 9px;
  flex-shrink: 0;
}
.modal-foot {
  padding: 16px 24px;
  background: #fcfaf7;
  border-top: 1px solid rgba(28, 22, 28, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
</style>
