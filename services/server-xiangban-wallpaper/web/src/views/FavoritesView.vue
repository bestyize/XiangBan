<script setup lang="ts">
import { onActivated, toRef } from 'vue'
import { useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import StateBlock from '@/components/StateBlock.vue'
import WallpaperGrid from '@/components/WallpaperGrid.vue'
import { setAmbient } from '@/composables/ambient'
import { usePullToRefresh } from '@/composables/pullToRefresh'
import { openViewer } from '@/composables/viewer'
import { useFavorites } from '@/stores/favorites'
import type { WallpaperItem } from '@/types'

const router = useRouter()
const favorites = useFavorites()
const items = toRef(favorites, 'items')

const { pull, refreshing, ready } = usePullToRefresh(() => favorites.load())

onActivated(() => setAmbient(null))

function open(item: WallpaperItem) {
  openViewer(router, { items }, item)
}
</script>

<template>
  <main class="page">
    <div class="ptr" :style="{ height: `${pull}px` }" aria-hidden="true">
      <Icon name="refresh" :size="22" :class="{ spin: refreshing, ready }" />
    </div>

    <header class="heading">
      <h1 class="display">收藏</h1>
      <p v-if="items.length">{{ items.length }} 张壁纸</p>
    </header>

    <WallpaperGrid v-if="items.length" :items="items" @open="open" />
    <StateBlock
      v-else-if="favorites.loaded"
      kind="empty"
      title="还没有收藏"
      hint="点壁纸上的心形图标，喜欢的会出现在这里。"
      action="去逛逛"
      @action="router.push('/')"
    />
    <WallpaperGrid v-else skeleton />
  </main>
</template>

<style scoped>
.ptr {
  display: grid;
  place-items: center;
  overflow: hidden;
  color: var(--text-3);
  transition: height 0.25s var(--ease);
}

.ptr .ready {
  color: var(--text);
}

.ptr .spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.heading {
  padding: 28px 0 16px;
}

.heading h1 {
  font-size: clamp(44px, 7vw, 76px);
}

.heading p {
  margin-top: 4px;
  color: var(--text-2);
}
</style>
