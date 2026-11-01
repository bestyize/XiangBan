<script setup lang="ts">
import { computed } from 'vue'
import RankingVendor from './RankingVendor.vue'
import { formatTokens, formatShare, movement, vendorColor, type UsageRow } from '@/lib/modelRankings'
const props = withDefaults(defineProps<{rows:UsageRow[];preview?:boolean;previousAvailable?:boolean}>(),{preview:false,previousAvailable:false})
const maxShare = computed(() => Math.max(1,...props.rows.map(row => Number.isFinite(row.share_pct) ? row.share_pct : 0)))
function width(row:UsageRow) { return `${Math.max(0,Math.min(100,(row.share_pct || 0)/(props.preview ? maxShare.value : 100)*100))}%` }
</script>
<template><div class="usage-ranks"><details v-for="row in rows" :key="row.id" :style="{'--vendor':vendorColor(row.id.split('/')[0] || row.id)}">
  <summary :aria-label="`${row.name}，${formatShare(row.share_pct,2)}，查看详情`"><span class="rank">{{ row.rank }}</span><RankingVendor :name="row.id.split('/')[0] || row.id"/><span class="model-name" :class="{preview}"><i v-if="preview" :style="{width:width(row)}"/><span>{{ row.name.includes('/') ? row.name.slice(row.name.indexOf('/')+1) : row.name }}</span></span><strong>{{ formatShare(row.share_pct,2) }}</strong><span v-if="!preview" class="usage-track"><i :style="{width:width(row)}"/></span><span v-if="!preview" class="movement">{{ movement(row,previousAvailable) }}</span></summary>
  <p class="detail">{{ row.name }}<br>{{ formatTokens(row.tokens) }} Token · {{ formatShare(row.share_pct) }}</p>
</details></div></template>
<style scoped>
summary{display:grid;grid-template-columns:18px 18px minmax(0,1fr) max-content;align-items:center;gap:7px;list-style:none;min-height:42px;padding:5px 0;cursor:pointer}summary::-webkit-details-marker{display:none}.rank,.movement{color:var(--ink-3);font-size:11px;font-variant-numeric:tabular-nums}.model-name{min-width:0;font-size:12px;position:relative;display:flex;align-items:center;overflow:hidden;height:28px;border-radius:5px}.model-name>span{white-space:nowrap;text-overflow:ellipsis;overflow:hidden;position:relative}.model-name.preview{background:var(--surface-2);padding:0 7px}.model-name>i{position:absolute;inset:0 auto 0 0;background:color-mix(in srgb,var(--vendor) 22%,transparent)}strong{min-width:46px;font-size:12px;font-weight:500;text-align:right;white-space:nowrap;font-variant-numeric:tabular-nums}.usage-track{grid-column:2/4;height:5px;border-radius:4px;background:var(--surface-2);overflow:hidden}.usage-track i{display:block;height:100%;background:var(--vendor);border-radius:4px}.movement{text-align:right}.detail{margin:2px 0 10px 25px;font-size:12px;line-height:1.7;color:var(--ink-3);overflow-wrap:anywhere}
</style>
