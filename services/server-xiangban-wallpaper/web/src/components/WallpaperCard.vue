<script setup lang="ts">
import { computed, ref } from 'vue'
import { useIntersectionObserver } from '@vueuse/core'
import Icon from './Icon.vue'
import { setAmbient } from '@/composables/ambient'
import { pauseVideo, playExclusive } from '@/composables/autoplay'
import type { WallpaperItem } from '@/types'
import { badgeOf, cardImage, colorOf, providerName, tileRatio } from '@/utils'

const props = withDefaults(
  defineProps<{
    item: WallpaperItem
    collectorMode?: boolean
    isCollected?: boolean
  }>(),
  {
    collectorMode: false,
    isCollected: false,
  },
)
const emit = defineEmits<{
  open: [item: WallpaperItem]
  collect: [item: WallpaperItem]
}>()

const root = ref<HTMLElement>()
const video = ref<HTMLVideoElement>()
const loaded = ref(false)
const failed = ref(false)
const playing = ref(false)

const hoverCapable = window.matchMedia('(hover: hover) and (pointer: fine)').matches

const isVideo = computed(() => props.item.media_type === 'video' && !!props.item.video)
const color = computed(() => colorOf(props.item))
const badge = computed(() => badgeOf(props.item))

async function startPreview() {
  if (!isVideo.value || !video.value) return
  playExclusive(video.value)
}

function stopPreview() {
  if (video.value) {
    pauseVideo(video.value)
  }
  playing.value = false
}

function onEnter() {
  if (!hoverCapable) return
  setAmbient(color.value, 140)
  startPreview()
}

function onLeave() {
  if (!hoverCapable) return
  stopPreview()
}

// On touch screens there is no hover, so a clip plays while it sits mostly in view.
useIntersectionObserver(
  root,
  ([entry]) => {
    if (hoverCapable || !isVideo.value) return
    if (entry.isIntersecting && entry.intersectionRatio >= 0.65) startPreview()
    else stopPreview()
  },
  { threshold: [0, 0.65] },
)
</script>

<template>
  <article
    ref="root"
    class="card"
    :style="{ aspectRatio: `1 / ${tileRatio(item)}`, background: color }"
    @pointerenter="onEnter"
    @pointerleave="onLeave"
  >
    <video
      v-if="isVideo && item.video"
      ref="video"
      :src="item.video.preview_url + '#t=0.001'"
      :poster="item.preview_url || undefined"
      muted
      loop
      playsinline
      preload="metadata"
      @playing="playing = true"
    />
    <img
      v-else-if="!failed"
      :src="cardImage(item)"
      :alt="item.title"
      loading="lazy"
      decoding="async"
      draggable="false"
      :class="{ loaded }"
      @load="loaded = true"
      @error="failed = true"
    />

    <button class="hit" type="button" :aria-label="`查看：${item.title}`" @click="emit('open', item)" />

    <span v-if="badge" class="badge">
      <Icon v-if="isVideo" name="play" :size="13" />
      {{ badge }}
    </span>

    <div class="caption">
      <span class="title">{{ item.title }}</span>
      <span class="source">{{ providerName(item.provider) }}</span>
    </div>

    <template v-if="collectorMode">
      <span v-if="isCollected" class="collect-badge on">
        <Icon name="check" :size="13" />
        已入库
      </span>
      <button
        v-else
        class="collect-btn"
        type="button"
        title="一键采集入库"
        @click.stop="emit('collect', item)"
      >
        <Icon name="download" :size="14" />
        <span>采集</span>
      </button>
    </template>
  </article>
</template>

<style scoped>
.card {
  position: relative;
  width: 100%;
  overflow: hidden;
  border-radius: var(--r-tile);
  isolation: isolate;
  contain: layout paint;
}

img,
video {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

img {
  opacity: 0;
  transition: opacity 0.5s ease;
}

img.loaded {
  opacity: 1;
}

video {
  opacity: 1;
}

.hit {
  position: absolute;
  inset: 0;
  z-index: 1;
  border-radius: inherit;
}

.badge {
  position: absolute;
  left: 8px;
  top: 8px;
  z-index: 2;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 22px;
  padding: 0 8px;
  border-radius: 999px;
  background: rgba(10, 10, 12, 0.55);
  -webkit-backdrop-filter: blur(10px);
  backdrop-filter: blur(10px);
  font-size: 12px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
}

.caption {
  position: absolute;
  inset: auto 0 0;
  z-index: 2;
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 36px 12px 10px;
  background: linear-gradient(to top, rgba(8, 8, 10, 0.78), transparent);
  opacity: 0;
  transform: translateY(6px);
  transition:
    opacity 0.25s,
    transform 0.35s var(--ease);
  pointer-events: none;
}

.title {
  font-size: 13px;
  font-weight: 500;
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.source {
  font-size: 12px;
  color: rgba(241, 239, 234, 0.7);
}

.collect-badge {
  position: absolute;
  right: 8px;
  top: 8px;
  z-index: 3;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 24px;
  padding: 0 9px;
  border-radius: 999px;
  background: rgba(16, 185, 129, 0.85);
  -webkit-backdrop-filter: blur(10px);
  backdrop-filter: blur(10px);
  font-size: 11px;
  font-weight: 600;
  color: #fff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.collect-btn {
  position: absolute;
  right: 8px;
  top: 8px;
  z-index: 3;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 26px;
  padding: 0 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.9);
  color: #111;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.collect-btn:hover {
  background: #fff;
  transform: scale(1.05);
}

@media (hover: hover) and (pointer: fine) {
  .card:hover .caption,
  .card:focus-within .caption {
    opacity: 1;
    transform: none;
  }
}
</style>
