<script setup lang="ts">
import type { TopicItem } from '@/api/types'
import TopicRow from './TopicRow.vue'

withDefaults(defineProps<{ items: TopicItem[]; loading?: boolean; showCategory?: boolean; empty?: string }>(), {
  showCategory: true,
})
</script>

<template>
  <div class="list" :aria-busy="loading">
    <template v-if="items.length">
      <TopicRow v-for="t in items" :key="t.id" :topic="t" :show-category="showCategory" />
    </template>
    <div v-else-if="loading" class="skeleton" aria-label="加载中">
      <div v-for="i in 5" :key="i" class="bar" />
    </div>
    <div v-else class="empty">
      <slot name="empty">{{ empty ?? '这里还没有帖子。' }}</slot>
    </div>
  </div>
</template>

<style scoped>
/* 平铺列表：没有外框和圆角，只有行与行之间的细线。 */
.list {
  border-top: 1px solid var(--line);
}
.list > :deep(*) {
  border-bottom: 1px solid var(--line);
}
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
