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

const product = ref<DesktopProduct | null>(null)
const loading = ref(true)
const error = ref('')
const modalTargetId = ref<string | null>(null)

function detectPlatformId(): string {
  if (typeof navigator === 'undefined') return 'windows-x86_64'
  const ua = navigator.userAgent || ''
  if (/Android/i.test(ua)) return 'android-arm64'
  if (/Mac OS X|Macintosh/i.test(ua)) return 'darwin-aarch64'
  return 'windows-x86_64'
}

const detectedId = ref(detectPlatformId())

const platformMeta: Record<string, {
  osTag: string
  shortName: string
  format: string
  desc: string
  preview: string
  previewSecond?: string
  highlights: string[]
}> = {
  'windows-x86_64': {
    osTag: 'DESKTOP · WINDOWS',
    shortName: 'Windows 64 位',
    format: '.exe 一键安装包',
    desc: 'NSIS 一键安装包，默认安装到当前用户目录、无需管理员权限；内置全部 AI 工作与桌面装扮能力。',
    preview: '/desktop/desktop-home.webp',
    highlights: [
      '当前用户目录免管理员权限安装，卸载干净无残留',
      '内置应用更新检测，新版发布即可在设置内一键升级',
      '需 Windows 10/11 64 位系统与 WebView2 运行时',
    ],
  },
  'darwin-aarch64': {
    osTag: 'DESKTOP · macOS',
    shortName: 'macOS (Apple 芯片)',
    format: '.dmg 磁盘映像',
    desc: '适配 M1 / M2 / M3 / M4 系列 Mac，提供完整 AI 工作区、动态壁纸、14 种特效与桌面伙伴。',
    preview: '/desktop/desktop-home.webp',
    highlights: [
      '原生独立设置与多模型配置窗口，支持逐模型连通检测',
      '内建更新检测，后续版本在「设置 → 关于」接收新版',
      '包含桌面伙伴、14 种轻量特效与离线壁纸缓存',
    ],
  },
  'darwin-x86_64': {
    osTag: 'DESKTOP · macOS INTEL',
    shortName: 'macOS (Intel)',
    format: '.dmg 磁盘映像',
    desc: '面向 Intel 芯片 Mac 的桌面端安装包，包含 AI 工作助手、壁纸与桌面体验。',
    preview: '/desktop/desktop-home.webp',
    highlights: [
      '原生桌面控制台与 AI 工作区体验',
      '支持壁纸库、桌面特效与桌面伙伴',
      '首次打开如遇安全提示，右键选择「打开」即可',
    ],
  },
  'android-arm64': {
    osTag: 'MOBILE · ANDROID',
    shortName: 'Android 手机版',
    format: '.apk 官方签名包',
    desc: '原生打造的移动端应用：随时与小伴 AI 对话/实时语音通话、浏览社区，并支持手机桌面互联。',
    preview: '/desktop/mobile-ai-home.webp',
    previewSecond: '/desktop/mobile-voice-call.webp',
    highlights: [
      '支持小伴 AI 实时语音通话、多轮任务对话与随身作品试玩',
      '内置超级小伴社区发现、讨论与一键分享卡片',
      '支持与同账号小伴桌面端手机互联，出门也能远程发任务',
    ],
  },
}

const visibleTargets = computed(() =>
  targets.filter(t => t.id !== 'darwin-x86_64' || Boolean(product.value?.downloads[t.id])),
)

const recommendedTarget = computed(() => {
  const list = visibleTargets.value
  const exact = list.find(t => t.id === detectedId.value && product.value?.downloads[t.id])
  if (exact) return exact
  const anyReady = list.find(t => Boolean(product.value?.downloads[t.id]))
  return anyReady ?? list.find(t => t.id === detectedId.value) ?? list[0]
})

const recommendedDownload = computed<DesktopProductDownload | null>(() => {
  const id = recommendedTarget.value?.id
  if (!id || !product.value) return null
  return product.value.downloads[id] ?? null
})

const recommendedMeta = computed(() =>
  platformMeta[recommendedTarget.value?.id || 'windows-x86_64'] ?? platformMeta['windows-x86_64'],
)

const modalTarget = computed(() =>
  modalTargetId.value ? targets.find(t => t.id === modalTargetId.value) ?? null : null,
)

const modalDownload = computed(() =>
  modalTargetId.value && product.value ? product.value.downloads[modalTargetId.value] ?? null : null,
)

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
    const rec = recommendedDownload.value
    emit('loaded', rec ? { version: rec.version, publishedAt: rec.publishedAt } : null)
  } catch {
    error.value = '下载信息暂时无法加载，请稍后重试。'
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
    <!-- Embedded heading (shown on home page, hidden on /download to avoid duplicate hero) -->
    <div v-if="!standalone" class="heading">
      <div>
        <span class="eyebrow">GET XIAOBAN</span>
        <h2 id="downloads-title">开始使用小伴</h2>
      </div>
      <p>选择适合你设备的版本，开始工作与创作。<br>各平台功能有所不同，请以客户端实际功能为准。</p>
    </div>

    <!-- Loading Skeleton -->
    <template v-if="loading">
      <div class="state-banner" role="status">
        <div>
          <strong>正在获取各平台最新版本信息…</strong>
          <p>正在拉取官方安装包清单与更新说明</p>
        </div>
        <span class="eyebrow">LOADING</span>
      </div>
      <div class="skeleton-grid" aria-hidden="true">
        <div class="skeleton-card" />
        <div class="skeleton-card" />
        <div class="skeleton-card" />
      </div>
    </template>

    <!-- Error State -->
    <div v-else-if="error" class="state-banner error" role="alert">
      <div>
        <strong>{{ error }}</strong>
        <p>请检查网络连接后重试，或前往小伴社区了解最新动态。</p>
      </div>
      <button type="button" class="retry-btn" @click="load">重新加载 ↻</button>
    </div>

    <!-- Ready State -->
    <template v-else>
      <!-- Standalone Recommended Spotlight Card -->
      <section v-if="standalone && recommendedTarget" class="dl-spotlight" aria-label="推荐下载">
        <div class="spot-left">
          <span class="spot-pill">
            <span class="spot-dot" aria-hidden="true" />
            已匹配你的设备 · 推荐安装
          </span>
          <div class="spot-title-row">
            <span class="spot-os-ico" aria-hidden="true">
              <svg v-if="recommendedTarget.id.startsWith('windows')" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M3 5.5 10.6 4.4v7.1H3V5.5zm8.7-1.3L21 3v8.5h-9.3V4.2zM3 12.6h7.6v7.1L3 18.6v-6zm8.7 0H21V21l-9.3-1.3v-7.1z" /></svg>
              <svg v-else-if="recommendedTarget.id.startsWith('darwin')" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.7-2.8-.7-1.4 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7 1.3 0 1.7.7 2.8.7 1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.4-.1 0-2.5-.9-2.5-3.5zM14.4 6.3c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 2-.5 2.5-1.2z" /></svg>
              <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M17.5 9.4l1.5-2.6a.45.45 0 0 0-.16-.62.45.45 0 0 0-.62.16l-1.54 2.67A9.4 9.4 0 0 0 12 7.8a9.4 9.4 0 0 0-4.68 1.19L5.78 6.32a.45.45 0 0 0-.62-.16.45.45 0 0 0-.16.62l1.5 2.6C4.1 10.7 2.5 13.1 2.2 16h19.6c-.3-2.9-1.9-5.3-4.3-6.6zM8.5 13.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2zm7 0a1 1 0 1 1 0-2 1 1 0 0 1 0 2z" /></svg>
            </span>
            <h2>{{ recommendedTarget.label }}</h2>
          </div>

          <template v-if="recommendedDownload">
            <div class="spot-meta">
              <span>版本 v{{ recommendedDownload.version }}</span>
              <i aria-hidden="true" />
              <span>{{ formatDate(recommendedDownload.publishedAt) }} 发布</span>
              <template v-if="formatBytes(recommendedDownload.bytes)">
                <i aria-hidden="true" />
                <span>{{ formatBytes(recommendedDownload.bytes) }}</span>
              </template>
              <i aria-hidden="true" />
              <span>{{ recommendedMeta.format }}</span>
            </div>
            <div class="spot-actions">
              <a :href="recommendedDownload.url" class="btn-spot-main">
                <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                <span>立即下载安装包</span>
              </a>
              <button type="button" class="btn-spot-sub" @click="modalTargetId = recommendedTarget.id">
                查看 v{{ recommendedDownload.version }} 更新说明
              </button>
            </div>
            <div v-if="recommendedDownload.sha256" class="spot-sha">
              <span>SHA-256: {{ recommendedDownload.sha256.slice(0, 14) }}…{{ recommendedDownload.sha256.slice(-8) }}</span>
              <button type="button" @click="copySha(recommendedDownload.sha256)">复制校验值</button>
            </div>
          </template>
          <p v-else class="spot-empty">该平台安装包发布后即可下载，你也可以在下方选择其他平台版本。</p>
        </div>

        <div class="spot-right">
          <div v-if="recommendedMeta.previewSecond" class="spot-mobile-pair">
            <div class="spot-mobile-frame"><img :src="recommendedMeta.preview" alt="小伴手机版 AI 首页" width="540" height="1108"></div>
            <div class="spot-mobile-frame"><img :src="recommendedMeta.previewSecond" alt="小伴手机版实时语音通话" width="540" height="1108"></div>
          </div>
          <div v-else class="spot-preview-img">
            <img :src="recommendedMeta.preview" :alt="`${recommendedTarget.label} 界面预览`" width="1280" height="800">
          </div>
          <ul class="spot-checks">
            <li v-for="item in recommendedMeta.highlights" :key="item">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
              <span>{{ item }}</span>
            </li>
          </ul>
        </div>
      </section>

      <!-- All Platforms Grid -->
      <div class="dl-all-wrap">
        <div v-if="standalone" class="dl-sec-head">
          <div>
            <h2 id="downloads-title">全部平台版本</h2>
            <p>需要为另一台电脑或手机安装？直接选择对应平台官方包</p>
          </div>
          <button
            v-if="recommendedTarget && recommendedDownload"
            type="button"
            class="btn-ghost"
            @click="modalTargetId = recommendedTarget.id"
          >
            查看当前版本更新日志
          </button>
        </div>

        <div class="platform-grid">
          <article
            v-for="target in visibleTargets"
            :key="target.id"
            class="plat-card"
            :class="{ 'is-current': target.id === recommendedTarget?.id }"
          >
            <div class="plat-top">
              <span class="plat-icon" aria-hidden="true">
                <svg v-if="target.id.startsWith('windows')" width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M3 5.5 10.6 4.4v7.1H3V5.5zm8.7-1.3L21 3v8.5h-9.3V4.2zM3 12.6h7.6v7.1L3 18.6v-6zm8.7 0H21V21l-9.3-1.3v-7.1z" /></svg>
                <svg v-else-if="target.id.startsWith('darwin')" width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M16.5 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.7-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.7-2.8-.7-1.4 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7 1.3 0 1.7.7 2.8.7 1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.4-.1 0-2.5-.9-2.5-3.5zM14.4 6.3c.6-.7 1-1.7.9-2.7-.9 0-1.9.6-2.5 1.3-.5.6-1 1.6-.9 2.6 1 .1 2-.5 2.5-1.2z" /></svg>
                <svg v-else width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M17.5 9.4l1.5-2.6a.45.45 0 0 0-.16-.62.45.45 0 0 0-.62.16l-1.54 2.67A9.4 9.4 0 0 0 12 7.8a9.4 9.4 0 0 0-4.68 1.19L5.78 6.32a.45.45 0 0 0-.62-.16.45.45 0 0 0-.16.62l1.5 2.6C4.1 10.7 2.5 13.1 2.2 16h19.6c-.3-2.9-1.9-5.3-4.3-6.6zM8.5 13.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2zm7 0a1 1 0 1 1 0-2 1 1 0 0 1 0 2z" /></svg>
              </span>
              <span class="plat-chip">
                {{ target.id === recommendedTarget?.id ? '当前设备推荐' : (target.id.startsWith('android') ? '移动端' : '桌面端') }}
              </span>
            </div>

            <span class="plat-kicker">{{ platformMeta[target.id]?.osTag }}</span>
            <h3>{{ target.label }}</h3>
            <p class="plat-desc">{{ platformMeta[target.id]?.desc }}</p>

            <template v-if="product?.downloads[target.id]">
              <div class="plat-meta-row">
                <span><strong>v{{ product.downloads[target.id].version }}</strong></span>
                <template v-if="formatBytes(product.downloads[target.id].bytes)">
                  <span>·</span>
                  <span>{{ formatBytes(product.downloads[target.id].bytes) }}</span>
                </template>
                <span>·</span>
                <span>{{ platformMeta[target.id]?.format }}</span>
              </div>
              <div class="plat-actions">
                <a
                  :href="product.downloads[target.id].url"
                  class="plat-dl-btn"
                  :class="target.id === recommendedTarget?.id ? 'primary' : 'outline'"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                  <span>下载 {{ platformMeta[target.id]?.shortName }}</span>
                </a>
                <div class="plat-foot-links">
                  <span>{{ formatDate(product.downloads[target.id].publishedAt) }} 发布</span>
                  <button type="button" @click="modalTargetId = target.id">更新说明 ↗</button>
                </div>
              </div>
            </template>
            <div v-else class="plat-pending">安装包发布后即可下载。</div>
          </article>
        </div>
      </div>

      <p v-if="!standalone" class="note">Windows 需要 WebView2；macOS 当前面向 Apple 芯片；Android 需要 Android 10 及以上系统。AI 能力和桌面功能因平台而异。</p>

      <!-- Official Announcements -->
      <div v-if="product?.announcements.length" class="announcements">
        <article v-for="notice in product.announcements" :key="notice.id" class="notice-box">
          <div class="notice-ico" aria-hidden="true">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.73 21a2 2 0 0 1-3.46 0" /></svg>
          </div>
          <div class="notice-body">
            <div class="notice-head">
              <span class="notice-tag">小伴公告</span>
              <h3>{{ notice.title }}</h3>
            </div>
            <p>{{ notice.body }}</p>
          </div>
        </article>
      </div>
    </template>

    <!-- Changelog Modal -->
    <Teleport to="body">
      <div v-if="modalTarget && modalDownload" class="modal-mask" @click.self="modalTargetId = null">
        <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="dl-modal-title">
          <div class="modal-head">
            <h3 id="dl-modal-title">{{ modalTarget.label }} · 更新说明</h3>
            <button type="button" class="modal-close" aria-label="关闭" @click="modalTargetId = null">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18" /><path d="m6 6 12 12" /></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="modal-meta">
              <span>版本 v{{ modalDownload.version }}</span>
              <span>{{ formatDate(modalDownload.publishedAt) }} 发布</span>
              <span v-if="formatBytes(modalDownload.bytes)">{{ formatBytes(modalDownload.bytes) }}</span>
              <span>{{ platformMeta[modalTarget.id]?.format }}</span>
            </div>
            <ul class="changelog-list">
              <li v-for="(line, idx) in noteLines(modalDownload.notes)" :key="idx">{{ line }}</li>
            </ul>
          </div>
          <div class="modal-foot">
            <button v-if="modalDownload.sha256" type="button" class="btn-ghost" @click="copySha(modalDownload.sha256)">复制 SHA-256</button>
            <span v-else class="modal-foot-hint">官方原包分发</span>
            <a :href="modalDownload.url" class="modal-dl-btn">立即下载安装包</a>
          </div>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.downloads {
  --bg: #fafaf9;
  --surface: #ffffff;
  --surface-soft: #f4f3f0;
  --ink: #141416;
  --ink-2: #52525b;
  --ink-3: #71717a;
  --brand: #18181b;
  --brand-hover: #2f2f35;
  --brand-soft: #ededf0;
  --line: rgba(24, 24, 27, 0.08);
  --line-strong: rgba(24, 24, 27, 0.16);
  scroll-margin-top: 90px;
  padding: 88px 0;
  color: var(--ink);
}

.downloads.standalone {
  padding: 0;
}

.heading {
  display: flex;
  justify-content: space-between;
  align-items: end;
  gap: 32px;
  margin-bottom: 32px;
}

.eyebrow,
.plat-kicker {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ink-3);
  text-transform: uppercase;
}

.heading h2 {
  font-size: clamp(32px, 4vw, 48px);
  letter-spacing: -0.04em;
  line-height: 1.2;
  font-weight: 700;
  margin-top: 10px;
}

.heading p {
  color: var(--ink-2);
  line-height: 1.8;
}

.btn-ghost {
  height: 36px;
  padding: 0 13px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--ink-2);
  border: 1px solid var(--line);
  transition: background 0.18s, color 0.18s, border-color 0.18s;
}
.btn-ghost:hover {
  color: var(--brand);
  border-color: var(--line-strong);
  background: var(--surface);
}

/* Spotlight Recommended Card */
.dl-spotlight {
  border-radius: 24px;
  background: linear-gradient(135deg, #141417 0%, #1e1e23 58%, #27272e 100%);
  color: #f4f4f5;
  padding: 36px 40px;
  display: grid;
  grid-template-columns: 1.15fr 0.85fr;
  gap: 36px;
  align-items: center;
  box-shadow: 0 24px 56px rgba(15, 15, 20, 0.18);
  position: relative;
  overflow: hidden;
}

.dl-spotlight::before {
  content: '';
  position: absolute;
  width: 420px;
  height: 420px;
  right: -120px;
  top: -160px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 255, 255, 0.11), transparent 70%);
  pointer-events: none;
}

.spot-left {
  position: relative;
  z-index: 2;
}

.spot-pill {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 12px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
  border: 1px solid rgba(255, 255, 255, 0.16);
  font-size: 12px;
  font-weight: 600;
  color: #f4f4f5;
}

.spot-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #fff;
}

.spot-title-row {
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 18px 0 10px;
}

.spot-os-ico {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.13);
  border: 1px solid rgba(255, 255, 255, 0.18);
  display: grid;
  place-items: center;
  color: #fff;
  flex-shrink: 0;
}

.spot-title-row h2 {
  font-size: clamp(24px, 2.5vw, 32px);
  font-weight: 700;
  letter-spacing: -0.02em;
  color: #fff;
  line-height: 1.2;
}

.spot-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  font-size: 13.5px;
  color: rgba(244, 244, 245, 0.78);
  margin-bottom: 24px;
}

.spot-meta i {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.4);
}

.spot-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.btn-spot-main {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 48px;
  padding: 0 24px;
  border-radius: 11px;
  background: #ffffff;
  color: #141416;
  font-size: 15px;
  font-weight: 700;
  box-shadow: 0 10px 26px rgba(0, 0, 0, 0.22);
  transition: transform 0.15s var(--ease), background 0.18s;
}
.btn-spot-main:hover {
  background: #f4f4f5;
  transform: translateY(-1px);
}

.btn-spot-sub {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 48px;
  padding: 0 18px;
  border-radius: 11px;
  background: rgba(255, 255, 255, 0.11);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #f4f4f5;
  font-size: 13.5px;
  font-weight: 600;
  transition: background 0.18s;
}
.btn-spot-sub:hover {
  background: rgba(255, 255, 255, 0.18);
}

.spot-sha {
  margin-top: 16px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: rgba(244, 244, 245, 0.68);
  font-family: ui-monospace, 'SFMono-Regular', Menlo, monospace;
}

.spot-sha button {
  font-family: var(--font-ui);
  font-size: 11.5px;
  padding: 2px 8px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.12);
  color: #f4f4f5;
}
.spot-sha button:hover {
  background: rgba(255, 255, 255, 0.22);
}

.spot-empty {
  margin-top: 12px;
  color: rgba(244, 244, 245, 0.78);
  font-size: 14px;
}

.spot-right {
  background: rgba(255, 255, 255, 0.07);
  border: 1px solid rgba(255, 255, 255, 0.13);
  border-radius: 16px;
  padding: 16px;
  position: relative;
  z-index: 2;
}

.spot-preview-img {
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.14);
  margin-bottom: 14px;
  background: #fafafa;
}

.spot-preview-img img {
  width: 100%;
  aspect-ratio: 16 / 10;
  object-fit: cover;
  object-position: top;
}

.spot-mobile-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 14px;
}

.spot-mobile-frame {
  padding: 5px;
  border-radius: 18px;
  background: rgba(10, 10, 14, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.16);
  box-shadow: 0 12px 26px rgba(0, 0, 0, 0.28);
}

.spot-mobile-frame img {
  width: 100%;
  aspect-ratio: 9 / 17.5;
  object-fit: cover;
  object-position: top;
  border-radius: 14px;
}

.spot-checks {
  display: grid;
  gap: 7px;
  font-size: 12.5px;
  color: rgba(244, 244, 245, 0.88);
  list-style: none;
}

.spot-checks li {
  display: flex;
  align-items: center;
  gap: 8px;
}

.spot-checks svg {
  color: #e4e4e7;
  flex-shrink: 0;
}

/* All Platforms Grid */
.standalone .dl-all-wrap {
  padding-top: 52px;
}

.dl-sec-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 22px;
}

.dl-sec-head h2 {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.dl-sec-head p {
  font-size: 13.5px;
  color: var(--ink-3);
  margin-top: 4px;
}

.platform-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 16px;
}

.plat-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 18px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s var(--ease);
}

.plat-card:hover {
  border-color: var(--line-strong);
  box-shadow: 0 14px 36px rgba(15, 15, 20, 0.06);
  transform: translateY(-2px);
}

.plat-card.is-current {
  border-color: var(--brand);
  background: linear-gradient(180deg, #fafafa 0%, #ffffff 42%);
  box-shadow: 0 14px 36px rgba(15, 15, 20, 0.06);
}

.plat-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 18px;
}

.plat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--brand);
  display: grid;
  place-items: center;
}

.plat-card.is-current .plat-icon {
  background: var(--brand);
  color: #fff;
  border-color: var(--brand);
}

.plat-chip {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 9px;
  border-radius: 999px;
  background: var(--surface-soft);
  color: var(--ink-2);
  letter-spacing: 0.04em;
}

.plat-card.is-current .plat-chip {
  background: var(--brand-soft);
  color: var(--brand);
}

.plat-card h3 {
  font-size: 19px;
  font-weight: 700;
  margin: 6px 0 6px;
  color: var(--ink);
}

.plat-desc {
  font-size: 13px;
  color: var(--ink-3);
  line-height: 1.65;
  margin-bottom: 16px;
}

.plat-meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 20px;
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--bg);
  font-size: 12.5px;
  color: var(--ink-2);
}

.plat-actions {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.plat-dl-btn {
  width: 100%;
  height: 44px;
  border-radius: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 650;
  transition: background 0.18s, border-color 0.18s, color 0.18s;
}

.plat-dl-btn.primary {
  background: var(--brand);
  color: #fff;
}
.plat-dl-btn.primary:hover {
  background: var(--brand-hover);
}

.plat-dl-btn.outline {
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line-strong);
}
.plat-dl-btn.outline:hover {
  background: var(--brand-soft);
  color: var(--brand);
  border-color: var(--brand);
}

.plat-foot-links {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12.5px;
  color: var(--ink-3);
  padding-top: 2px;
}

.plat-foot-links button {
  font-size: 12.5px;
  color: var(--ink-2);
  font-weight: 550;
}
.plat-foot-links button:hover {
  color: var(--brand);
  text-decoration: underline;
}

.plat-pending {
  margin-top: auto;
  padding: 12px;
  border-radius: 10px;
  background: var(--surface-soft);
  font-size: 13px;
  color: var(--ink-3);
  text-align: center;
}

.note {
  font-size: 13px;
  line-height: 1.8;
  color: var(--ink-3);
  margin-top: 20px;
}

/* Loading & Error */
.state-banner {
  border-radius: 18px;
  padding: 24px 28px;
  background: var(--surface);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 18px;
}

.state-banner strong {
  display: block;
  font-size: 15.5px;
}

.state-banner p {
  font-size: 13px;
  color: var(--ink-3);
  margin-top: 4px;
}

.state-banner.error {
  background: #fdf7f6;
  border-color: rgba(186, 68, 56, 0.22);
}

.state-banner.error strong { color: #9b3528; }
.state-banner.error p { color: #7a4b44; }

.retry-btn {
  height: 40px;
  padding: 0 18px;
  border-radius: 10px;
  background: var(--brand);
  color: #fff;
  font-size: 13.5px;
  font-weight: 650;
  flex-shrink: 0;
}

.skeleton-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.skeleton-card {
  height: 290px;
  border-radius: 18px;
  background: linear-gradient(90deg, #f0f0ee 25%, #f8f8f6 50%, #f0f0ee 75%);
  background-size: 200% 100%;
  animation: shimmer 1.4s infinite;
  border: 1px solid var(--line);
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* Announcements */
.announcements {
  margin-top: 32px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.notice-box {
  border-radius: 18px;
  background: linear-gradient(135deg, #f5f4f1 0%, #faf9f7 100%);
  border: 1px solid var(--line);
  padding: 22px 26px;
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.notice-ico {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--surface);
  color: var(--brand);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  border: 1px solid var(--line);
}

.notice-body { flex: 1; }

.notice-head {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 6px;
}

.notice-tag {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--brand);
  color: #fff;
}

.notice-body h3 {
  font-size: 15.5px;
  font-weight: 700;
}

.notice-body p {
  font-size: 13.5px;
  color: var(--ink-2);
  line-height: 1.7;
  white-space: pre-wrap;
}

/* Changelog Modal */
.modal-mask {
  position: fixed;
  inset: 0;
  z-index: 300;
  background: rgba(15, 15, 20, 0.45);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-card {
  width: min(520px, 100%);
  background: #ffffff;
  color: #141416;
  border-radius: 20px;
  border: 1px solid rgba(24, 24, 27, 0.08);
  box-shadow: 0 24px 60px rgba(15, 15, 20, 0.16);
  overflow: hidden;
}

.modal-head {
  padding: 20px 24px;
  border-bottom: 1px solid rgba(24, 24, 27, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.modal-head h3 {
  font-size: 17px;
  font-weight: 700;
}

.modal-close {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: #71717a;
}
.modal-close:hover {
  background: #f4f3f0;
  color: #141416;
}

.modal-body {
  padding: 20px 24px 24px;
  max-height: 60vh;
  overflow-y: auto;
}

.modal-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.modal-meta span {
  font-size: 12px;
  padding: 3px 9px;
  border-radius: 999px;
  background: #f4f3f0;
  color: #52525b;
}

.changelog-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 14px;
  color: #52525b;
  line-height: 1.7;
  list-style: none;
}

.changelog-list li {
  display: flex;
  align-items: flex-start;
  gap: 9px;
}

.changelog-list li::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #18181b;
  margin-top: 9px;
  flex-shrink: 0;
}

.modal-foot {
  padding: 16px 24px;
  background: #fafaf9;
  border-top: 1px solid rgba(24, 24, 27, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.modal-foot-hint {
  font-size: 12.5px;
  color: #71717a;
}

.modal-dl-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 38px;
  padding: 0 18px;
  border-radius: 9px;
  background: #18181b;
  color: #fff;
  font-size: 13.5px;
  font-weight: 650;
}
.modal-dl-btn:hover { background: #2f2f35; }

@media (max-width: 960px) {
  .dl-spotlight { grid-template-columns: 1fr; padding: 28px 24px; gap: 24px; }
  .skeleton-grid { grid-template-columns: 1fr; }
}

@media (max-width: 700px) {
  .downloads { padding: 56px 0; }
  .downloads.standalone { padding: 0; }
  .heading,
  .dl-sec-head,
  .state-banner { flex-direction: column; align-items: flex-start; gap: 12px; }
  .spot-actions { flex-direction: column; align-items: stretch; }
  .btn-spot-main,
  .btn-spot-sub { width: 100%; justify-content: center; }
  .notice-box { padding: 18px; flex-direction: column; gap: 12px; }
}
</style>
