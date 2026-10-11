<script setup lang="ts">
import type { TopicItem } from '@/api/types'
import UserAvatar from './UserAvatar.vue'
import { compactCount, timeAgo } from '@/lib/format'

withDefaults(defineProps<{ topic: TopicItem; showCategory?: boolean }>(), { showCategory: true })
</script>

<template>
  <article class="row">
    <RouterLink :to="`/u/${topic.author.uid}`" class="avatar-link" :aria-label="topic.author.nickname">
      <UserAvatar :name="topic.author.nickname" :src="topic.author.avatar_url" :size="36" />
    </RouterLink>
    <div class="body">
      <h3 class="title">
        <span v-if="topic.pinned" class="flag pin">置顶</span>
        <span v-if="topic.featured" class="flag feat">精选</span>
        <RouterLink :to="`/t/${topic.id}`">{{ topic.title }}</RouterLink>
      </h3>
      <div class="meta">
        <RouterLink v-if="showCategory" :to="`/c/${topic.category.slug}`" class="cat">{{ topic.category.name }}</RouterLink>
        <RouterLink :to="`/u/${topic.author.uid}`" class="author">{{ topic.author.nickname }}</RouterLink>
        <span>{{ topic.reply_count ? `${timeAgo(topic.last_reply_at)}回复` : timeAgo(topic.created_at) }}</span>
        <span v-if="topic.locked">已锁定</span>
      </div>
    </div>
    <RouterLink :to="`/t/${topic.id}`" class="count" :class="{ zero: !topic.reply_count }" :aria-label="`${topic.reply_count} 条回复`">
      {{ compactCount(topic.reply_count) }}
    </RouterLink>
  </article>
</template>

<style scoped>
.row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 12px;
  padding: 12px 4px;
  align-items: start;
}
.body {
  min-width: 0;
}
.title {
  margin: 0 0 2px;
  font-size: var(--fs-md);
  font-weight: 600;
  line-height: 1.5;
}
.title a:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-thickness: 1px;
}
.title a:visited {
  color: var(--ink-2);
}
.flag {
  font-size: var(--fs-xs);
  font-weight: 700;
  margin-right: 6px;
}
.pin {
  color: var(--danger);
}
.feat {
  color: var(--suspect);
}
.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 12px;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
.cat {
  color: var(--ink-2);
  font-weight: 600;
}
.author:hover,
.cat:hover {
  color: var(--ink);
}
.count {
  min-width: 28px;
  text-align: right;
  font-family: var(--font-display);
  font-weight: 600;
  font-size: var(--fs-sm);
  font-variant-numeric: tabular-nums;
  color: var(--ink-2);
  line-height: 1.8;
}
.count.zero {
  color: var(--ink-3);
  font-weight: 400;
}
@media (max-width: 767px) {
  .row {
    padding: 12px 0;
    gap: 10px;
  }
}
</style>
