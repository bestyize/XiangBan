<script setup lang="ts">
import { onActivated, ref, watch } from 'vue'
import { setAmbient } from '@/composables/ambient'
import { useCatalog } from '@/stores/catalog'
import type { Category } from '@/types'
import { cardImage, colorOf } from '@/utils'

const catalog = useCatalog()

// Which kind of page a tile opens. The covers are photos either way.
const kind = ref<'image' | 'video'>('image')

watch(
  () => catalog.categories.length,
  () => catalog.loadCovers(),
  { immediate: true },
)

onActivated(() => setAmbient(null))

function tint(c: Category) {
  const cover = catalog.covers[c.id]
  if (cover) return colorOf(cover)
  let hash = 0
  for (const ch of c.id) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0
  return `hsl(${hash % 360} 26% 28%)`
}
</script>

<template>
  <main class="page">
    <header class="heading">
      <h1 class="display">分类</h1>
      <div class="kinds" role="group" aria-label="内容类型">
        <button class="chip" type="button" :aria-pressed="kind === 'image'" @click="kind = 'image'">图片</button>
        <button class="chip" type="button" :aria-pressed="kind === 'video'" @click="kind = 'video'">视频</button>
      </div>
    </header>

    <div class="tiles">
      <RouterLink
        v-for="c in catalog.categories"
        :key="c.id"
        :to="kind === 'video' ? `/videos/c/${c.id}` : `/c/${c.id}`"
        class="tile"
        :style="{ background: tint(c) }"
        @pointerenter="setAmbient(tint(c), 120)"
      >
        <img
          v-if="catalog.covers[c.id]"
          :src="cardImage(catalog.covers[c.id])"
          alt=""
          loading="lazy"
          decoding="async"
          draggable="false"
        />
        <span class="name display">{{ c.name }}</span>
      </RouterLink>
    </div>
  </main>
</template>

<style scoped>
.heading {
  padding: 28px 0 16px;
}

.kinds {
  display: flex;
  gap: 8px;
  margin-top: 14px;
}

.heading h1 {
  font-size: clamp(44px, 7vw, 76px);
}

.tiles {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

@media (min-width: 640px) {
  .tiles {
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
  }
}

@media (min-width: 1100px) {
  .tiles {
    grid-template-columns: repeat(4, 1fr);
  }
}

.tile {
  position: relative;
  display: flex;
  align-items: flex-end;
  aspect-ratio: 4 / 5;
  overflow: hidden;
  border-radius: var(--r-tile);
  isolation: isolate;
}

.tile img {
  position: absolute;
  inset: 0;
  z-index: -2;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.7s var(--ease);
}

.tile::after {
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  background: linear-gradient(to top, rgba(8, 8, 10, 0.72), transparent 62%);
}

.name {
  padding: 16px 16px 14px;
  font-size: clamp(30px, 3.4vw, 42px);
  line-height: 1;
}

@media (hover: hover) and (pointer: fine) {
  .tile:hover img {
    transform: scale(1.05);
  }
}
</style>
