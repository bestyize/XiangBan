<script setup lang="ts">
import type { HelpInfo } from '@/api/types'
import { formatDateTime } from '@/lib/format'
defineProps<{ help: HelpInfo; owner: boolean; busy: boolean; expired: boolean }>()
defineEmits<{ close: [] }>()
const labels = { open: '求助中', solved: '已解决', closed: '已关闭', expired: '已到期' }
</script>

<template>
  <section class="panel help-panel" aria-label="资源求助">
    <div class="top"><strong>{{ help.status === 'open' && expired ? '已到期' : labels[help.status] }}</strong><span>{{ help.bounty ? `悬赏 ${help.bounty} 积分` : '免费求助' }}</span></div>
    <p v-if="help.status === 'open' && !expired">截止 {{ formatDateTime(help.expires_at) }}。{{ help.bounty ? '悬赏已暂存，采纳回答后发放给回答者。' : '找到有帮助的回答后，求助者可以标记为已解决。' }}</p>
    <p v-else-if="help.status === 'solved'">求助者已采纳回答。{{ help.bounty ? '悬赏已发放。' : '' }}</p>
    <p v-else-if="help.status === 'open' && expired">求助已到期，悬赏会自动退还。{{ owner ? '也可点击下方立即结算。' : '' }}</p>
    <p v-else>{{ help.bounty ? '未发放的悬赏已退还给求助者。' : '这次求助已经结束。' }}</p>
    <button v-if="owner && help.status === 'open'" type="button" class="btn btn-sm" :disabled="busy" @click="$emit('close')">{{ expired ? '结算并退还悬赏' : '关闭求助' }}</button>
  </section>
</template>

<style scoped>
.help-panel { margin: 12px 0 18px; padding: 14px 16px; background: var(--surface-2); }
.top { display: flex; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
.top span { color: var(--brand); font-weight: 600; font-size: var(--fs-sm); }
p { margin: 8px 0; font-size: var(--fs-sm); color: var(--ink-2); line-height: 1.6; }
</style>
