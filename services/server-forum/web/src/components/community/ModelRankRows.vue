<script setup lang="ts">
import RankingVendor from './RankingVendor.vue'
import { formatValue, vendorColor, type ModelRank } from '@/lib/modelRankings'
const props = withDefaults(defineProps<{ rows:ModelRank[]; vendors:{id:string;name:string}[]; metric?:string }>(),{metric:'intelligence'})
const vendor = (id:string) => props.vendors.find(item => item.id === id)?.name || '未知厂商'
</script>
<template><div class="model-ranks"><details v-for="row in rows" :key="row.model.id">
  <summary :aria-label="`${row.model.name}，${formatValue(row.value,metric)}，查看详情`"><span class="rank">{{ row.rank ?? '—' }}</span><RankingVendor :name="vendor(row.model.vendor_id)" /><span class="bar" :style="{'--vendor':vendorColor(vendor(row.model.vendor_id))}"><i :style="{width:`${(row.fraction ?? 0)*100}%`}"/><span>{{ row.model.name }}</span></span><strong>{{ formatValue(row.value,metric) }}</strong></summary>
  <div class="model-detail"><b>{{ row.model.name }}</b><span>{{ vendor(row.model.vendor_id) }}</span><span>智能指数 {{ formatValue(row.model.metrics.intelligence ?? null) }} · 输出速度 {{ formatValue(row.model.performance.output_tokens_per_second,'speed') }} Token/秒</span><span>输入 {{ formatValue(row.model.pricing.input,'input') }} / 输出 {{ formatValue(row.model.pricing.output,'output') }} · 美元/百万 Token</span></div>
</details></div></template>
<style scoped>
.model-ranks{min-width:0}details{min-width:0}summary{list-style:none;display:grid;grid-template-columns:18px 18px minmax(0,1fr) max-content;gap:7px;align-items:center;min-height:40px;cursor:pointer}summary::-webkit-details-marker{display:none}.rank{font-size:11px;color:var(--ink-3);font-variant-numeric:tabular-nums}.bar{position:relative;min-width:0;height:28px;display:flex;align-items:center;border-radius:5px;background:var(--surface-2);overflow:hidden}.bar i{position:absolute;inset:0 auto 0 0;background:color-mix(in srgb,var(--vendor) 20%,transparent);border-radius:5px}.bar>span{position:relative;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;padding:0 7px;font-size:12px}strong{font-size:12px;min-width:40px;max-width:110px;overflow-wrap:anywhere;font-variant-numeric:tabular-nums;text-align:right;font-weight:600}.model-detail{display:grid;gap:3px;padding:6px 4px 12px 25px;color:var(--ink-3);font-size:12px;line-height:1.7;overflow-wrap:anywhere}.model-detail b{color:var(--ink);font-weight:550}:root[data-theme=dark] .bar i{background:color-mix(in srgb,var(--vendor) 32%,transparent)}
</style>
