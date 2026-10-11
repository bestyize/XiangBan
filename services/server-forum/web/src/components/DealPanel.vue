<script setup lang="ts">
import { computed } from 'vue'
import type { TopicDetail } from '@/api/types'
import { conditionLabels, expiryText, formatDateTime } from '@/lib/format'

const props = defineProps<{ topic: TopicDetail }>()

const deal = computed(() => props.topic.deal!)
const value = computed(() => deal.value.value_text.trim())
const conditions = computed(() => conditionLabels(deal.value.conditions).join('、'))
const hasExpiry = computed(() => deal.value.long_term || !!deal.value.expires_at)
/** 什么都没填时整块不显示。 */
const empty = computed(
  () => !value.value && !conditions.value && !hasExpiry.value && !deal.value.link_url,
)
</script>

<template>
  <section v-if="!empty" class="deal" aria-label="羊毛信息">
    <dl class="facts">
      <div v-if="value" class="gain">
        <dt>能领到</dt>
        <dd>{{ value }}</dd>
      </div>
      <div v-if="conditions">
        <dt>领取条件</dt>
        <dd>{{ conditions }}</dd>
      </div>
      <div v-if="hasExpiry">
        <dt>有效期</dt>
        <dd>
          {{ expiryText(deal.expires_at, deal.long_term) }}
          <span v-if="deal.expires_at && !deal.long_term" class="muted">（{{ formatDateTime(deal.expires_at) }}）</span>
        </dd>
      </div>
    </dl>
    <a v-if="deal.link_url" :href="deal.link_url" target="_blank" rel="nofollow noopener noreferrer" class="go">
      打开活动页
    </a>
  </section>
</template>

<style scoped>
/* 帖子顶部的几行信息，就是正文前面的普通文字，不画框。 */
.deal {
  margin-bottom: 16px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--line);
}
.facts {
  margin: 0;
  display: grid;
  gap: 4px;
  font-size: var(--fs-md);
}
.facts > div {
  display: grid;
  grid-template-columns: 5.5em minmax(0, 1fr);
  gap: 12px;
  align-items: baseline;
}
dt {
  color: var(--ink-3);
  font-size: var(--fs-sm);
}
dd {
  margin: 0;
  overflow-wrap: anywhere;
}
.gain dd {
  font-weight: 700;
}
.go {
  display: inline-block;
  margin-top: 10px;
  font-size: var(--fs-sm);
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 3px;
}
@media (max-width: 767px) {
  .facts > div {
    grid-template-columns: 5em minmax(0, 1fr);
    gap: 8px;
  }
}
</style>
