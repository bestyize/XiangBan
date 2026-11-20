<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps<{ images: string[]; title: string; fallbackCover?: string }>()
const track = ref<HTMLDivElement>()
const active = ref(0)
const failedCover = ref<string>()
const cover = computed(() => failedCover.value === props.images[0] && props.fallbackCover ? props.fallbackCover : props.images[0])

function syncPage() {
  const rail = track.value
  if (!rail?.clientWidth) return
  active.value = Math.max(0, Math.min(props.images.length - 1, Math.round(rail.scrollLeft / rail.clientWidth)))
}

function goTo(index: number) {
  const rail = track.value
  if (!rail || index < 0 || index >= props.images.length) return
  rail.scrollTo({ left: index * rail.clientWidth })
}
</script>

<template>
  <div class="work-gallery">
    <img v-if="images.length === 1" :src="cover" :alt="title + '封面'" class="cover" @error="failedCover = images[0]">
    <template v-else>
      <div
        ref="track"
        class="rail"
        role="region"
        aria-label="作品图集"
        aria-roledescription="轮播"
        tabindex="0"
        @scroll.passive="syncPage"
        @keydown.left.prevent="goTo(active - 1)"
        @keydown.right.prevent="goTo(active + 1)"
      >
        <div v-for="(image, index) in images" :key="image" class="slide" :aria-hidden="index !== active">
          <img
            :src="index === 0 ? cover : image"
            :alt="title + '，第 ' + (index + 1) + ' 张'"
            :loading="index === 0 ? 'eager' : 'lazy'"
            @error="index === 0 && (failedCover = image)"
          >
        </div>
      </div>
      <div class="dots" role="group" aria-label="切换作品图片">
        <button
          v-for="(image, index) in images"
          :key="image"
          type="button"
          :aria-label="'第 ' + (index + 1) + ' 张，共 ' + images.length + ' 张'"
          :aria-current="index === active ? 'true' : undefined"
          @click="goTo(index)"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.work-gallery { position: relative; min-width: 0; }
.cover, .slide:first-child img { display: block; width: 100%; height: auto; background: var(--surface-2); }
.slide:first-child img { aspect-ratio: auto 1; }
.rail { display: flex; overflow-x: auto; overflow-y: hidden; scroll-snap-type: x mandatory; overscroll-behavior-inline: contain; scroll-behavior: smooth; scrollbar-width: none; background: var(--surface-2); }
.rail::-webkit-scrollbar { display: none; }
.slide { flex: 0 0 100%; min-width: 0; position: relative; scroll-snap-align: start; }
.slide:not(:first-child) img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; }
.dots { position: absolute; right: 8px; bottom: 8px; display: flex; flex-wrap: wrap; justify-content: center; padding: 0 4px; }
.dots button { display: grid; place-items: center; width: 12px; height: 20px; padding: 0; border: 0; background: none; cursor: pointer; }
.dots button::after { content: ''; width: 5px; height: 5px; border-radius: 50%; background: rgb(255 255 255 / 48%); }
.dots [aria-current]::after { background: #fff; }
</style>
