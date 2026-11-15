<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import { galleryRunLocation } from '@/lib/ideaPlayers'
import { galleryAssets, galleryExhibits, galleryTab } from '@/lib/microGallery'

const route = useRoute()
const router = useRouter()
const exhibit = computed(() => galleryExhibits.find(item => item.id === route.query.exhibit))
const player = useTemplateRef<HTMLIFrameElement>('player')
const photoMode = ref(false)
const previewFailed = ref(false)

function coverUrl(cover: string) {
  return `${galleryAssets}assets/gallery/${cover}.webp`
}

function togglePhoto() {
  photoMode.value = !photoMode.value
  previewFailed.value = false
}

function exhibitLocation(id?: string) {
  return galleryRunLocation(id) ?? { name: 'ideas', query: { ...route.query, tab: galleryTab.key, exhibit: id } }
}

watch(exhibit, async (current, previous) => {
  photoMode.value = false
  previewFailed.value = false
  await nextTick()
  if (current) {
    document.getElementById('exhibit-title')?.focus({ preventScroll: true })
  } else if (previous) {
    document.getElementById(`exhibit-${previous.id}`)?.focus({ preventScroll: true })
  }
})

function returnToGallery() {
  if (window.history.state?.back === router.resolve(exhibitLocation()).fullPath) {
    router.back()
  } else {
    void router.replace(exhibitLocation())
  }
}

function handleBackClick(event: MouseEvent) {
  if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return
  event.preventDefault()
  returnToGallery()
}

function onExhibitMessage(event: MessageEvent) {
  const frame = player.value
  if (!frame || event.source !== frame.contentWindow || event.origin !== 'null') return
  if (event.data === 'gallery:back') {
    returnToGallery()
  } else if (event.data === 'gallery:aquarium') {
    void router.replace(exhibitLocation('aquarium'))
  } else if (event.data === 'gallery:unavailable') {
    previewFailed.value = true
    photoMode.value = true
  }
}

watch(exhibit, (current, _, onCleanup) => {
  if (!current) return
  window.addEventListener('message', onExhibitMessage)
  onCleanup(() => window.removeEventListener('message', onExhibitMessage))
}, { immediate: true })
</script>

<template>
  <section class="gallery" aria-label="微观展厅">
    <template v-if="exhibit">
      <div class="gallery-toolbar">
        <a :href="router.resolve(exhibitLocation()).href" class="gallery-back" @click="handleBackClick">
          <AppIcon name="back" :size="16" /> 返回展厅
        </a>
        <h2 id="exhibit-title" tabindex="-1">{{ exhibit.name }}</h2>
        <button type="button" class="gallery-photo-toggle" @click="togglePhoto">{{ photoMode ? '试用 3D' : '看图片' }}</button>
      </div>
      <figure v-if="photoMode" class="gallery-photo">
        <img :src="coverUrl(exhibit.cover)" :alt="exhibit.name">
        <figcaption v-if="previewFailed" role="status">当前无法显示 3D 预览，已切换为展品图片。</figcaption>
      </figure>
      <iframe
        v-else
        ref="player"
        :key="exhibit.id"
        :src="galleryAssets + exhibit.source + exhibit.query"
        :title="exhibit.name"
        sandbox="allow-scripts"
        referrerpolicy="no-referrer"
        allowfullscreen
      ></iframe>
    </template>
    <ul v-else class="gallery-grid">
      <li v-for="(item, index) in galleryExhibits" :key="item.id" :class="{ 'gallery-feature': index === 0 }">
        <RouterLink :id="`exhibit-${item.id}`" :to="exhibitLocation(item.id)" class="gallery-card">
          <img :src="coverUrl(item.cover)" alt="" :loading="index === 0 ? 'eager' : 'lazy'" decoding="async">
          <div class="gallery-copy">
            <h2>{{ item.name }}</h2>
            <p>{{ item.description }}</p>
          </div>
          <AppIcon name="arrow" :size="16" class="gallery-arrow" />
        </RouterLink>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.gallery {
  margin: 8px 12px 0;
}

.gallery-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.gallery-feature {
  grid-column: 1 / -1;
}

.gallery-card {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  border: 1px solid var(--line);
  border-radius: 12px;
  background: var(--surface);
  overflow: hidden;
}

.gallery-card:hover {
  border-color: var(--line-strong);
}

.gallery-card img {
  display: block;
  width: 100%;
  aspect-ratio: 16 / 9;
  object-fit: cover;
  background: var(--surface-2);
}

.gallery-feature img {
  max-height: 370px;
}

.gallery-copy {
  padding: 12px 38px 14px 14px;
}

.gallery-copy h2,
.gallery-toolbar h2 {
  margin: 0;
  font-size: 14px;
  font-weight: 550;
}

.gallery-feature h2 {
  font-size: 17px;
}

.gallery-copy p {
  margin: 4px 0 0;
  color: var(--ink-2);
  font-size: 13px;
}

.gallery-arrow {
  position: absolute;
  right: 14px;
  bottom: 14px;
  color: var(--ink-3);
}

.gallery-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
}

.gallery-back {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-height: 44px;
  color: var(--brand);
  font-size: 13px;
}

.gallery-photo-toggle {
  margin-left: auto;
  min-height: 44px;
  padding: 0 8px;
  border: 0;
  background: transparent;
  color: var(--brand);
  font-size: 13px;
  cursor: pointer;
}

.gallery-photo {
  margin: 0;
  padding: 16px;
  border: 1px solid var(--line);
  border-radius: 12px;
  background: var(--surface);
  text-align: center;
}

.gallery-photo img {
  display: block;
  width: 100%;
  max-height: min(70dvh, 720px);
  object-fit: contain;
}

.gallery-photo figcaption {
  margin-top: 12px;
  color: var(--ink-2);
  font-size: 13px;
}

.gallery iframe {
  display: block;
  width: 100%;
  height: clamp(360px, 100dvh - 180px, 960px);
  border: 1px solid var(--line);
  border-radius: 12px;
  background: var(--surface-2);
}

@container page (min-width: 840px) {
  .gallery-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); }
}

@media (max-width: 767px) {
  .gallery { margin-inline: 4px; }
  .gallery-copy { padding: 10px 28px 10px 10px; }
  .gallery-arrow { right: 8px; bottom: 10px; }
}

@media (min-width: 1100px) {
  .gallery { margin-inline: 0; }
}
</style>
