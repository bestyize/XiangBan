<script setup lang="ts">
import type { PackItem } from '@/api/content'
import { coverStyle, kindLabel, trustLabel } from '@/lib/content'

defineProps<{ pack: PackItem; wide?: boolean }>()
</script>

<template>
  <RouterLink :to="`/packs/${pack.id}`" class="card" :class="{ wide }">
    <div class="cover" :style="coverStyle(pack)" role="img" :aria-label="`${pack.title} 封面`">
      <span class="kind">{{ kindLabel(pack.kind) }}</span>
    </div>
    <div class="info">
      <h3>{{ pack.title }}</h3>
      <p v-if="wide && pack.summary" class="summary">{{ pack.summary }}</p>
      <p class="meta">
        <span v-if="pack.trust_level < 2">{{ trustLabel(pack.trust_level) }}</span>
        <span v-if="pack.author">{{ pack.author.nickname }}</span>
        <span v-if="pack.apply_count">{{ pack.apply_count }} 次使用</span>
      </p>
    </div>
  </RouterLink>
</template>

<style scoped>
.card {
  display: block;
  min-width: 0;
}
.cover {
  position: relative;
  aspect-ratio: 4 / 3;
  border-radius: 8px;
  background-size: cover;
  background-position: center;
}
.kind {
  position: absolute;
  left: 8px;
  bottom: 8px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  font-size: var(--fs-xs);
}
.info {
  padding: 8px 2px 0;
}
h3 {
  margin: 0;
  font-size: var(--fs-md);
  font-weight: 500;
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}
.summary {
  margin: 4px 0 0;
  color: var(--ink-2);
  font-size: var(--fs-sm);
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
.meta {
  margin: 4px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 10px;
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
/* 单列：左图右文，卡片更宽才读得完说明。 */
.card.wide {
  display: grid;
  grid-template-columns: 132px minmax(0, 1fr);
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--line);
}
.card.wide .info {
  padding: 0;
}
</style>
