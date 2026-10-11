<script setup lang="ts">
import { computed } from 'vue'
import type { TopicItem } from '@/api/types'
import UserAvatar from './UserAvatar.vue'
import DealState from './DealState.vue'
import { DEAL_TYPE_LABEL, REGION_LABEL, compactCount, conditionLabels, expiringSoon, expiryText, timeAgo } from '@/lib/format'

const props = defineProps<{ topic: TopicItem }>()
const deal = computed(() => props.topic.deal!)
const value = computed(() => deal.value.value_text.trim() || DEAL_TYPE_LABEL[deal.value.deal_type])
const conditions = computed(() => {
  const list = conditionLabels(deal.value.conditions)
  return list.length ? list.join('、') : '无特殊条件'
})
const expiry = computed(() => expiryText(deal.value.expires_at, deal.value.long_term))
const soon = computed(() => expiringSoon(deal.value.expires_at, deal.value.long_term))
</script>

<template>
  <article class="row" :class="{ expired: deal.deal_status === 'expired' }">
    <RouterLink :to="`/u/${topic.author.uid}`" class="avatar-link" :aria-label="topic.author.nickname">
      <UserAvatar :name="topic.author.nickname" :src="topic.author.avatar_url" :size="36" />
    </RouterLink>
    <div class="body">
      <h3 class="title">
        <span v-if="topic.pinned" class="flag pin">置顶</span>
        <span v-if="topic.featured" class="flag feat">精选</span>
        <RouterLink :to="`/t/${topic.id}`">{{ topic.title }}</RouterLink>
      </h3>
      <!-- 羊毛信息就是普通的一行灰字，和普通帖的"分类 作者 时间"同一个位置 -->
      <div class="meta">
        <RouterLink :to="`/p/${deal.product.slug}`" class="cat">{{ deal.product.name }}</RouterLink>
        <span>{{ value }}</span>
        <span v-if="deal.region !== 'any'">{{ REGION_LABEL[deal.region] }}</span>
        <span>{{ conditions }}</span>
        <span :class="{ soon }">{{ expiry }}</span>
        <DealState :deal="deal" />
        <RouterLink :to="`/u/${topic.author.uid}`" class="author">{{ topic.author.nickname }}</RouterLink>
        <span>{{ topic.reply_count ? `${timeAgo(topic.last_reply_at)}回复` : timeAgo(topic.created_at) }}</span>
      </div>
    </div>
    <RouterLink :to="`/t/${topic.id}`" class="count" :class="{ zero: !topic.reply_count }" :aria-label="`${topic.reply_count} 条回复`">
      {{ compactCount(topic.reply_count) }}
    </RouterLink>
  </article>
</template>

<style scoped>
/* 和 TopicRow 完全同一套骨架，只是 meta 里多了几段文字。 */
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
  align-items: center;
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
.soon {
  color: var(--suspect);
  font-weight: 600;
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
.expired .title a {
  color: var(--ink-3);
}
@media (max-width: 767px) {
  .row {
    padding: 12px 0;
    gap: 10px;
  }
  /* 手机上头像已经能认出是谁，省掉作者名。 */
  .author {
    display: none;
  }
}
</style>
