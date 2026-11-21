<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppIcon from '@/components/AppIcon.vue'

const props = defineProps<{ images: string[]; title?: string; badge?: string; fallbackCover?: string; mobile: boolean }>()
const slide = ref(0)
const failed = ref<string>()
const index = computed(() => Math.min(slide.value, Math.max(0, props.images.length - 1)))
const image = computed(() => {
  const source = props.images[index.value]
  return index.value === 0 && failed.value === source && props.fallbackCover ? props.fallbackCover : source
})
watch(() => props.images[0], () => { slide.value = 0; failed.value = undefined })

let start: Touch | undefined
let swiped = false
function move(step: number) {
  const length = props.images.length
  if (length > 1) slide.value = (index.value + step + length) % length
}
function beginSwipe(event: TouchEvent) {
  swiped = false
  start = event.touches.length === 1 ? event.touches[0] : undefined
}
function endSwipe(event: TouchEvent) {
  const end = event.changedTouches[0]
  if (start && end && !event.touches.length) {
    const dx = end.clientX - start.clientX, dy = end.clientY - start.clientY
    if (Math.abs(dx) > 36 && Math.abs(dx) > Math.abs(dy)) {
      swiped = true
      move(dx < 0 ? 1 : -1)
    }
  }
  start = undefined
}
function advance(event: MouseEvent) {
  if (!swiped || event.detail === 0) move(1)
  swiped = false
}
function useFallback() {
  if (index.value === 0) failed.value = props.images[0]
}
</script>

<template>
  <div v-if="image" :class="mobile ? 'xhs-h5-media' : 'xhs-pc-media'">
    <button v-if="images.length > 1" class="detail-media-button" type="button" aria-label="查看下一张图片"
      @click="advance" @keydown.left.prevent="move(-1)" @keydown.right.prevent="move(1)"
      @touchstart.passive="beginSwipe" @touchend.passive="endSwipe" @touchcancel.passive="start = undefined">
      <img :key="image" :src="image" :alt="title || '详情图片'" @error="useFallback" />
    </button>
    <img v-else :src="image" :alt="title || '详情图片'" @error="useFallback" />
    <span v-if="!mobile && badge" class="xhs-media-pill">{{ badge }}</span>
    <template v-if="!mobile && images.length > 1">
      <span class="xhs-media-count">{{ index + 1 }}/{{ images.length }}</span>
      <button class="xhs-pc-nav is-prev" type="button" aria-label="上一张" @click="move(-1)"><AppIcon name="back" :size="16" /></button>
      <button class="xhs-pc-nav is-next" type="button" aria-label="下一张" @click="move(1)"><AppIcon name="arrow" :size="16" /></button>
    </template>
  </div>
  <div v-if="images.length > 1" class="xhs-media-dots" role="group" aria-label="切换图片">
    <button v-for="(source, i) in images" :key="source" type="button" class="xhs-media-dot" :class="{ 'is-active': i === index }"
      :aria-label="'第 ' + (i + 1) + ' 张'" :aria-current="i === index ? 'true' : undefined" @click="slide = i" />
  </div>
</template>

<style scoped>
.detail-media-button { display: block; width: 100%; padding: 0; border: 0; background: transparent; touch-action: pan-y pinch-zoom; }
.detail-media-button:focus-visible { outline-offset: -2px; }
.xhs-media-pill, .xhs-media-count { pointer-events: none; }
</style>
