<script setup lang="ts">
import { computed } from 'vue'
import type { DealInfo } from '@/api/types'
import { DEAL_STATUS_LABEL, timeAgo } from '@/lib/format'

const props = defineProps<{ deal: DealInfo; verbose?: boolean }>()

const text = computed(() => {
  const d = props.deal
  if (d.deal_status === 'expired') return DEAL_STATUS_LABEL.expired
  if (d.deal_status === 'suspect') return `${DEAL_STATUS_LABEL.suspect}，${d.invalid_votes} 票反馈领不到`
  // 没有人投过"有效"时，last_verified_at 只是发帖时间，不能说成"确认可领"。
  if (d.valid_votes === 0) return '新分享，等人验证'
  return `${timeAgo(d.last_verified_at)}确认可领`
})
</script>

<template>
  <span class="state" :class="deal.deal_status === 'valid' && deal.valid_votes === 0 ? 'is-new' : `is-${deal.deal_status}`">
    <span class="dot" aria-hidden="true" />
    <span>{{ text }}</span>
    <span v-if="verbose && deal.deal_status !== 'expired'" class="votes">
      {{ deal.valid_votes }} 有效 / {{ deal.invalid_votes }} 失效
    </span>
  </span>
</template>

<style scoped>
.state {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: var(--fs-xs);
  font-weight: 600;
  white-space: nowrap;
}
.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: currentColor;
  flex: none;
}
.is-valid {
  color: var(--valid);
}
.is-valid .dot {
  box-shadow: 0 0 0 3px var(--valid-soft);
}
.is-new {
  color: var(--ink-2);
}
.is-new .dot {
  background: transparent;
  border: 1.5px solid var(--valid);
}
.is-suspect {
  color: var(--suspect);
}
.is-expired {
  color: var(--expired);
}
.votes {
  color: var(--ink-3);
  font-weight: 500;
  margin-left: 4px;
}
</style>
