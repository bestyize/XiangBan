<script setup lang="ts">
import type { TopicItem } from '@/api/types'
import { compactCount } from '@/lib/format'

defineProps<{ topic: TopicItem }>()
</script>

<template>
  <div class="stats" aria-label="帖子统计">
    <span class="stat" :title="`${topic.view_count} 次浏览`">
      <span>浏览</span><span class="value">{{ compactCount(topic.view_count) }}</span>
    </span>
    <RouterLink
      :to="`/t/${topic.id}`"
      class="stat replies"
      :aria-label="`${topic.reply_count} 条回复，查看帖子`"
      :title="`${topic.reply_count} 条回复`"
    >
      <span>回复</span><span class="value">{{ compactCount(topic.reply_count) }}</span>
    </RouterLink>
    <span class="stat" :title="`${topic.like_count} 次点赞`">
      <span>点赞</span><span class="value">{{ compactCount(topic.like_count) }}</span>
    </span>
  </div>
</template>

<style scoped>
.stats {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 14px;
  margin-top: 6px;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
.stat {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}
.value {
  font-family: var(--font-display);
  font-variant-numeric: tabular-nums;
  color: var(--ink-2);
}
.replies:hover {
  color: var(--brand);
}
.replies:hover .value {
  color: inherit;
}
</style>
