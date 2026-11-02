<script setup lang="ts">
import type { TopicItem } from '@/api/types'
import ResourceTopicRow from './community/ResourceTopicRow.vue'
import type { ArticleLayout } from '@/lib/cardPresentation'
import MasonryGrid from './community/MasonryGrid.vue'

withDefaults(defineProps<{ items: TopicItem[]; loading?: boolean; showCategory?: boolean; empty?: string; presentation?: ArticleLayout }>(), {
  showCategory: true,
  presentation:'reading',
})
</script>

<template>
  <div class="topic-list-region"><div class="list" :class="[`feed-${presentation}`,{'adaptive-card-grid':presentation==='visual'}]" :aria-busy="loading">
    <MasonryGrid v-if="items.length && presentation==='masonry'" :columns="2"><ResourceTopicRow v-for="t in items" :key="t.id" :topic="t" :show-category="showCategory" layout="masonry" /></MasonryGrid>
    <template v-else-if="items.length">
      <ResourceTopicRow v-for="t in items" :key="t.id" :topic="t" :show-category="showCategory" :layout="presentation" />
    </template>
    <div v-else-if="loading" class="skeleton" aria-label="加载中">
      <div v-for="i in 5" :key="i" class="bar" />
    </div>
    <div v-else class="empty">
      <slot name="empty">{{ empty ?? '这里还没有帖子。' }}</slot>
    </div>
  </div></div>
</template>

<style scoped>
/* 平铺列表：没有外框和圆角，只有行与行之间的细线。 */
.list { display:grid; gap:12px; }
.feed-visual { grid-template-columns:repeat(var(--card-columns,2),minmax(0,1fr)); }
.empty,.skeleton { grid-column:1/-1; }
.list > :deep(.empty),
.list > :deep(.skeleton) {
  border-bottom: 0;
}
.empty {
  padding: 48px 20px;
  text-align: center;
  color: var(--ink-3);
  font-size: var(--fs-sm);
}
.skeleton {
  padding: 8px 0;
}
.bar {
  height: 54px;
  margin: 10px 0;
  border-radius: var(--r-sm);
  background: linear-gradient(90deg, var(--surface-2), var(--line), var(--surface-2));
  background-size: 200% 100%;
  animation: shimmer 1.4s linear infinite;
}
@keyframes shimmer {
  to {
    background-position: -200% 0;
  }
}
</style>
