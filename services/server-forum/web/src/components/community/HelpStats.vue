<script setup lang="ts">
import { onMounted } from 'vue'
import { discoverApi } from '@/api/discover'
import { useCommunityResource } from './resource'
const { data, loading, error, load } = useCommunityResource(discoverApi.helpStats)
onMounted(load)
</script>
<template><section class="help-overview" aria-label="求助统计"><div v-if="error" class="message" role="alert">统计加载失败 <button type="button" @click="load">重试</button></div><dl v-else><div><dt>进行中</dt><dd>{{ loading || !data ? '–' : data.open.toLocaleString('zh-CN') }}</dd></div><div><dt>本周已解决</dt><dd>{{ loading || !data ? '–' : data.solved_this_week.toLocaleString('zh-CN') }}</dd></div><div><dt>悬赏积分</dt><dd>{{ loading || !data ? '–' : data.bounty_points.toLocaleString('zh-CN') }}</dd></div></dl><RouterLink class="ask" to="/new?category=resource-help">＋ 发求助</RouterLink></section></template>
<style scoped>
.help-overview{display:flex;align-items:center;gap:8px;padding:14px;background:var(--surface);border-radius:12px;margin:8px 12px}dl{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;margin:0;flex:1;min-width:0}dl>div{display:flex;flex-direction:column-reverse;min-width:0}dt,.message{color:var(--ink-3);font-size:12px}dd{margin:0;font-size:18px;font-weight:700;overflow-wrap:anywhere}.ask{display:grid;place-items:center;min-height:40px;padding:0 12px;border-radius:24px;color:var(--brand);background:color-mix(in srgb,var(--brand) 9%,var(--surface));white-space:nowrap;font-size:14px}.message{flex:1}.message button{border:0;background:none;color:var(--brand);min-height:44px}@media(max-width:340px){.help-overview{flex-wrap:wrap}.ask{margin-left:auto}}
</style>
