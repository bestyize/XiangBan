<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { onKeyStroke } from '@vueuse/core'
import { getWallpaper } from '@/api'
import { setAmbient } from '@/composables/ambient'
import { toast } from '@/composables/toast'
import { closeViewer, stepViewer, viewerSource } from '@/composables/viewer'
import type { WallpaperItem } from '@/types'
import {
  cardImage,
  colorOf,
  downloadFile,
  formatDuration,
  orientationLabel,
  providerName,
  viewImage,
} from '@/utils'
import DevicePreview from './DevicePreview.vue'
import Icon from './Icon.vue'

interface Variant {
  label: string
  detail: string
  url: string
}

const route = useRoute()
const router = useRouter()

const currentId = computed(() => (typeof route.query.w === 'string' ? route.query.w : null))
const list = computed<readonly WallpaperItem[]>(() => viewerSource.value?.items.value ?? [])
const index = computed(() => (currentId.value ? list.value.findIndex((i) => i.id === currentId.value) : -1))

// Deep links and page reloads have no feed behind them, so the single item is fetched directly.
const fetched = ref<WallpaperItem | null>(null)
const item = computed<WallpaperItem | null>(() => {
  if (!currentId.value) return null
  if (index.value >= 0) return list.value[index.value]
  return fetched.value?.id === currentId.value ? fetched.value : null
})

const prev = computed(() => (index.value > 0 ? list.value[index.value - 1] : null))
const next = computed(() => (index.value >= 0 && index.value < list.value.length - 1 ? list.value[index.value + 1] : null))

const dir = ref<'next' | 'prev'>('next')
const chrome = ref(true)
const preview = ref(false)
const muted = ref(true)
const downloading = ref(false)
const variantIndex = ref(0)
const fullReady = reactive<Record<string, boolean>>({})
const closeBtn = ref<HTMLButtonElement>()

/** Plays the best clip that is no larger than 1080p; 4K files are for downloading, not for a phone screen. */
function viewVideoUrl(it: WallpaperItem) {
  const clips = [...(it.video?.qualities ?? [])].sort((a, b) => b.width - a.width)
  return (clips.find((q) => Math.max(q.width, q.height) <= 1920) ?? clips[clips.length - 1])?.url
}

const variants = computed<Variant[]>(() => {
  const it = item.value
  if (!it) return []
  if (it.video?.qualities.length) {
    return it.video.qualities.map((q) => ({ label: q.quality, detail: `${q.width}×${q.height}`, url: q.url }))
  }
  const out: Variant[] = []
  const add = (label: string, url: string | undefined, detail = '') => {
    if (url && !out.some((v) => v.url === url)) out.push({ label, detail, url })
  }
  add('原图', it.downloads.original, `${it.resolution.width}×${it.resolution.height}`)
  add('高清', it.downloads.large)
  add('标准', it.downloads.medium)
  return out
})

watch(
  [currentId, index],
  async ([id, idx]) => {
    if (!id || idx >= 0 || fetched.value?.id === id) return
    try {
      const it = await getWallpaper(id)
      if (currentId.value === id) fetched.value = it
    } catch {
      toast('找不到这张壁纸')
      closeViewer(router)
    }
  },
  { immediate: true },
)

watch(
  item,
  (it, old) => {
    if (!it) return
    if (it.id !== old?.id) {
      variantIndex.value = 0
      chrome.value = true
    }
    setAmbient(colorOf(it))
    for (const n of [next.value, prev.value]) {
      if (n && !n.video) new Image().src = viewImage(n)
    }
    if (viewerSource.value?.loadMore && index.value >= list.value.length - 4) viewerSource.value.loadMore()
  },
  { immediate: true },
)

watch(
  () => !!currentId.value,
  async (open) => {
    document.documentElement.style.overflow = open ? 'hidden' : ''
    if (open) {
      await nextTick()
      closeBtn.value?.focus()
    } else {
      setAmbient(null)
      preview.value = false
    }
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  document.documentElement.style.overflow = ''
})

function close() {
  closeViewer(router)
}

function go(delta: 1 | -1) {
  const target = delta > 0 ? next.value : prev.value
  if (!target) return
  dir.value = delta > 0 ? 'next' : 'prev'
  stepViewer(router, target.id)
}

onKeyStroke('ArrowRight', () => currentId.value && go(1))
onKeyStroke('ArrowLeft', () => currentId.value && go(-1))
onKeyStroke('Escape', () => currentId.value && close())

/* ---- touch gestures: swipe sideways to switch, drag down to close, tap to hide the controls ---- */

const drag = reactive({ x: 0, y: 0, active: false })
let startX = 0
let startY = 0
let startAt = 0
let axis: 'x' | 'y' | null = null
let pointerType = 'mouse'

const layerStyle = computed(() => ({
  transform: `translate3d(${drag.x}px, ${drag.y}px, 0) scale(${1 - Math.min(drag.y / 1400, 0.1)})`,
  transition: drag.active ? 'none' : 'transform 0.35s var(--ease)',
}))
const backdropStyle = computed(() => ({ opacity: String(1 - Math.min(drag.y / 360, 0.7)) }))

function onDown(e: PointerEvent) {
  pointerType = e.pointerType
  if (e.pointerType === 'mouse') return
  startX = e.clientX
  startY = e.clientY
  startAt = performance.now()
  axis = null
  drag.active = true
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
}

function onMove(e: PointerEvent) {
  if (!drag.active) return
  const dx = e.clientX - startX
  const dy = e.clientY - startY
  if (!axis && Math.hypot(dx, dy) > 8) axis = Math.abs(dx) > Math.abs(dy) ? 'x' : 'y'
  if (axis === 'x') {
    const blocked = (dx < 0 && !next.value) || (dx > 0 && !prev.value)
    drag.x = blocked ? dx * 0.25 : dx
  } else if (axis === 'y') {
    drag.y = Math.max(0, dy)
  }
}

function onUp(e: PointerEvent) {
  if (!drag.active) return
  drag.active = false
  const dx = e.clientX - startX
  const dy = e.clientY - startY
  const elapsed = Math.max(performance.now() - startAt, 1)
  drag.x = 0
  drag.y = 0

  if (axis === null) {
    if (elapsed < 300) onTap()
    return
  }
  if (axis === 'x' && (Math.abs(dx) > 70 || Math.abs(dx) / elapsed > 0.5)) go(dx < 0 ? 1 : -1)
  else if (axis === 'y' && (dy > 110 || dy / elapsed > 0.6)) close()
}

function onCancel() {
  drag.active = false
  drag.x = 0
  drag.y = 0
}

function onTap() {
  chrome.value = !chrome.value
}

function onStageClick(e: MouseEvent) {
  if (pointerType !== 'mouse') return
  const el = e.target as HTMLElement
  if (el.classList.contains('stage') || el.classList.contains('slide') || el.classList.contains('layer')) close()
}

/* ---- actions ---- */

async function download() {
  const it = item.value
  const variant = variants.value[variantIndex.value]
  if (!it || !variant) return
  downloading.value = true
  const outcome = await downloadFile(variant.url, it)
  downloading.value = false
  toast(
    {
      saved: '已开始下载',
      opened: '没能直接下载，已在新标签页打开，长按或右键即可保存',
      blocked: '没能直接下载，新标签页也被浏览器拦截了。请允许本站弹出窗口后重试',
    }[outcome],
    outcome === 'saved' ? 2200 : 5000,
  )
}

async function share() {
  const it = item.value
  if (!it) return
  if (navigator.share) {
    try {
      await navigator.share({ title: it.title, url: it.source_page_url })
    } catch {
      // the share sheet was dismissed
    }
    return
  }
  try {
    await navigator.clipboard.writeText(it.source_page_url)
    toast('链接已复制')
  } catch {
    toast('复制失败，请手动复制')
  }
}
</script>

<template>
  <Transition name="viewer">
    <div
      v-if="currentId"
      class="viewer"
      :class="{ 'chrome-off': !chrome }"
      role="dialog"
      aria-modal="true"
      :aria-label="item?.title ?? '壁纸预览'"
      :style="{ '--tint': item ? colorOf(item) : '#2b2b33' }"
    >
      <div class="backdrop" :style="backdropStyle" />

      <div
        class="stage"
        @pointerdown="onDown"
        @pointermove="onMove"
        @pointerup="onUp"
        @pointercancel="onCancel"
        @click="onStageClick"
      >
        <Transition :name="`slide-${dir}`">
          <div v-if="item" :key="item.id" class="slide">
            <div class="layer" :style="layerStyle">
              <DevicePreview
                v-if="preview"
                :item="item"
                :src="viewImage(item)"
                :video-src="item.video ? viewVideoUrl(item) : undefined"
              />
              <div v-else class="media-container">
                <video
                  v-if="item.video"
                  class="media"
                  :src="viewVideoUrl(item)"
                  :poster="item.preview_url || undefined"
                  autoplay
                  loop
                  playsinline
                  :muted="muted"
                />
                <template v-else>
                  <img class="media low" :src="cardImage(item)" alt="" aria-hidden="true" draggable="false" />
                  <img
                    class="media full"
                    :class="{ ready: fullReady[item.id] }"
                    :src="viewImage(item)"
                    :alt="item.title"
                    draggable="false"
                    @load="fullReady[item.id] = true"
                  />
                </template>
              </div>
            </div>
          </div>
        </Transition>

        <button v-if="prev" class="nav left" type="button" aria-label="上一张" @click.stop="go(-1)">
          <Icon name="chevron-left" :size="26" />
        </button>
        <button v-if="next" class="nav right" type="button" aria-label="下一张" @click.stop="go(1)">
          <Icon name="chevron-right" :size="26" />
        </button>
      </div>

      <header class="bar" :class="{ off: !chrome }">
        <button ref="closeBtn" class="round" type="button" aria-label="关闭" @click="close">
          <Icon name="close" :size="22" />
        </button>
        <span v-if="index >= 0" class="counter">{{ index + 1 }} / {{ list.length }}</span>
        <button
          v-if="item?.video && !preview"
          class="round"
          type="button"
          :aria-label="muted ? '开启声音' : '静音'"
          @click="muted = !muted"
        >
          <Icon :name="muted ? 'volume-off' : 'volume'" :size="20" />
        </button>
        <span v-else class="round ghost" />
      </header>

      <aside v-if="item" class="info" :class="{ off: !chrome }">
        <h2 class="title display">{{ item.title }}</h2>
        <p class="byline">
          <a v-if="item.author.profile_url" :href="item.author.profile_url" target="_blank" rel="noopener noreferrer">
            {{ item.author.name }}
          </a>
          <span v-else>{{ item.author.name }}</span>
          <span class="from">{{ providerName(item.provider) }}</span>
        </p>

        <ul class="specs">
          <li>{{ item.resolution.width }} × {{ item.resolution.height }}</li>
          <li>{{ item.resolution.aspect_ratio }}</li>
          <li v-if="orientationLabel(item)">{{ orientationLabel(item) }}</li>
          <li v-if="item.video">{{ formatDuration(item.video.duration_secs) }}</li>
        </ul>

        <div v-if="variants.length > 1" class="variants" role="group" aria-label="下载规格">
          <button
            v-for="(v, i) in variants"
            :key="v.url"
            class="chip"
            type="button"
            :aria-pressed="variantIndex === i"
            @click="variantIndex = i"
          >
            {{ v.label }}
            <small v-if="v.detail">{{ v.detail }}</small>
          </button>
        </div>

        <div class="actions">
          <button class="btn btn-primary grow" type="button" :disabled="downloading" @click="download">
            <Icon name="download" :size="20" />
            {{ downloading ? '下载中…' : '下载' }}
          </button>
          <button
            class="btn icon"
            type="button"
            :aria-pressed="preview"
            aria-label="在设备上预览"
            title="在设备上预览"
            @click="preview = !preview"
          >
            <Icon name="device" :size="22" />
          </button>
          <button class="btn icon" type="button" aria-label="分享" @click="share">
            <Icon name="share" :size="22" />
          </button>
        </div>

        <p class="fine">
          {{ item.license }}
          <a :href="item.source_page_url" target="_blank" rel="noopener noreferrer">
            在 {{ providerName(item.provider) }} 查看
            <Icon name="external" :size="14" />
          </a>
        </p>
      </aside>
    </div>
  </Transition>
</template>

<style scoped>
.viewer {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  background: #08080a;
  color: var(--text);
  overflow: hidden;
}

.backdrop {
  position: absolute;
  inset: 0;
  background: radial-gradient(85% 65% at 50% 28%, color-mix(in srgb, var(--tint) 50%, #08080a), #08080a 78%);
  transition: background 0.8s;
}

.stage {
  position: relative;
  grid-area: 1 / 1;
  min-height: 0;
  overflow: hidden;
  touch-action: none;
}

.slide {
  position: absolute;
  inset: 0;
}

.layer {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  will-change: transform;
  padding: calc(var(--safe-top) + 60px) 16px calc(var(--safe-bottom) + 260px);
  box-sizing: border-box;
  transition: padding 0.35s var(--ease);
}

.viewer.chrome-off .layer {
  padding: calc(var(--safe-top) + 24px) 16px calc(var(--safe-bottom) + 24px);
}

.media-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  min-height: 0;
  min-width: 0;
}

.media {
  max-width: 100%;
  max-height: 100%;
  width: auto;
  height: auto;
  object-fit: contain;
  display: block;
  user-select: none;
  -webkit-user-drag: none;
  border-radius: 8px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.45);
}

.media.low {
  position: relative;
  z-index: 1;
}

.media.full {
  position: absolute;
  inset: 0;
  margin: auto;
  z-index: 2;
  opacity: 0;
  transition: opacity 0.35s ease;
}

.media.full.ready {
  opacity: 1;
}

.nav {
  position: absolute;
  top: 50%;
  z-index: 3;
  display: none;
  place-items: center;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(20, 20, 24, 0.55);
  -webkit-backdrop-filter: blur(12px);
  backdrop-filter: blur(12px);
  transform: translateY(-50%);
  transition: background 0.2s;
}

.nav:hover {
  background: rgba(40, 40, 46, 0.8);
}

.nav.left {
  left: 20px;
}

.nav.right {
  right: 20px;
}

@media (hover: hover) and (pointer: fine) {
  .nav {
    display: grid;
  }
}

.bar {
  position: absolute;
  inset: 0 0 auto;
  z-index: 5;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: calc(var(--safe-top) + 12px) 14px 12px;
  pointer-events: none;
  transition:
    opacity 0.25s,
    transform 0.35s var(--ease);
}

.bar > * {
  pointer-events: auto;
}

.bar.off {
  opacity: 0;
  transform: translateY(-12px);
  pointer-events: none;
}

.bar.off > * {
  pointer-events: none;
}

.round {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: rgba(20, 20, 24, 0.55);
  -webkit-backdrop-filter: blur(12px);
  backdrop-filter: blur(12px);
}

.round.ghost {
  background: none;
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
  pointer-events: none;
}

.counter {
  font-size: 13px;
  color: var(--text-2);
  font-variant-numeric: tabular-nums;
}

.info {
  position: absolute;
  inset: auto 0 0;
  z-index: 4;
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 62%;
  overflow-y: auto;
  padding: 56px 18px calc(var(--safe-bottom) + 18px);
  background: linear-gradient(to top, rgba(8, 8, 10, 0.94) 55%, transparent);
  transition:
    opacity 0.25s,
    transform 0.35s var(--ease);
}

.info.off {
  opacity: 0;
  transform: translateY(16px);
  pointer-events: none;
}

.title {
  font-size: 30px;
  line-height: 1.12;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.byline {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
  color: var(--text-2);
  font-size: 14px;
}

.byline a:hover {
  color: var(--text);
  text-decoration: underline;
  text-underline-offset: 3px;
}

.from {
  color: var(--text-3);
}

.specs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  list-style: none;
}

.specs li {
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--line);
  font-size: 12px;
  color: var(--text-2);
  font-variant-numeric: tabular-nums;
}

.variants {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.variants .chip {
  height: 32px;
  gap: 6px;
  font-size: 13px;
}

.variants small {
  font-size: 11px;
  opacity: 0.7;
  font-variant-numeric: tabular-nums;
}

.actions {
  display: flex;
  gap: 8px;
}

.grow {
  flex: 1;
}

.btn.icon {
  width: 44px;
  padding: 0;
  background: rgba(241, 239, 234, 0.06);
}

.btn.icon[aria-pressed='true'] {
  background: rgba(241, 239, 234, 0.16);
}

.fine {
  font-size: 12px;
  color: var(--text-3);
}

.fine a {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: 10px;
  color: var(--text-2);
}

.fine a:hover {
  color: var(--text);
}

/* Desktop: the artwork gets the stage, details live in a side panel. */
@media (min-width: 960px) {
  .viewer {
    grid-template-columns: minmax(0, 1fr) 400px;
  }

  .stage {
    grid-column: 1;
  }

  .layer {
    padding: 68px 72px 48px !important;
  }

  .info {
    position: relative;
    grid-area: 1 / 2;
    inset: auto;
    z-index: 4;
    max-height: none;
    justify-content: flex-end;
    padding: 96px 32px 36px;
    background: rgba(18, 18, 21, 0.78);
    border-left: 1px solid var(--line);
    -webkit-backdrop-filter: blur(24px);
    backdrop-filter: blur(24px);
  }

  .info.off {
    opacity: 1;
    transform: none;
    pointer-events: auto;
  }

  .bar {
    right: 400px;
  }

  .bar.off {
    opacity: 1;
    transform: none;
    pointer-events: none;
  }

  .bar.off > * {
    pointer-events: auto;
  }

  .title {
    font-size: 36px;
  }
}

/* Open / close */
.viewer-enter-active,
.viewer-leave-active {
  transition:
    opacity 0.25s,
    transform 0.4s var(--ease);
}

.viewer-enter-from,
.viewer-leave-to {
  opacity: 0;
  transform: translateY(24px) scale(0.985);
}

/* Switching between items */
.slide-next-enter-active,
.slide-next-leave-active,
.slide-prev-enter-active,
.slide-prev-leave-active {
  transition:
    transform 0.4s var(--ease),
    opacity 0.3s;
}

.slide-next-enter-from,
.slide-prev-leave-to {
  opacity: 0;
  transform: translateX(14%);
}

.slide-next-leave-to,
.slide-prev-enter-from {
  opacity: 0;
  transform: translateX(-14%);
}
</style>
