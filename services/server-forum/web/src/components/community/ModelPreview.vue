<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { discoverApi } from '@/api/discover'
import { rankModels, vendorColor } from '@/lib/modelRankings'
import { useCommunityResource } from './resource'
import RequestState from './RequestState.vue'
import ModelRankRows from './ModelRankRows.vue'
import UsageRankRows from './UsageRankRows.vue'
import ModelScatter from './ModelScatter.vue'
const capability = useCommunityResource(discoverApi.models)
const usage = useCommunityResource(discoverApi.usage)
onMounted(() => { void capability.load(); void usage.load() })
const ranked = computed(() => rankModels(capability.data.value?.models ?? [],'intelligence').filter(row => row.value !== null))
const speed = computed(() => rankModels(capability.data.value?.models ?? [],'speed').filter(row => row.value !== null).slice(0,5))
const topVendors = computed(() => capability.data.value?.vendors.filter(vendor => ranked.value.slice(0,10).some(row => row.model.vendor_id === vendor.id)) ?? [])
const usageTop = computed(() => [...(usage.data.value?.models ?? [])].sort((a,b)=>a.rank-b.rank).slice(0,5))
function updated(value:string) { const date=new Date(value); return Number.isNaN(date.getTime())?'更新时间未知':date.toLocaleDateString('zh-CN',{timeZone:'Asia/Shanghai'}) }
</script>
<template><div class="boards">
  <section class="board ability"><header><div><h2>AA 智能指数</h2><p v-if="capability.data.value" class="source">指数 v{{ capability.data.value.index_version }} · {{ ranked.length }} 个已测模型<span v-if="capability.data.value.sample"> · 示例数据</span></p></div><RouterLink to="/models">全部排行 <span aria-hidden="true">›</span></RouterLink></header>
    <RequestState :loading="capability.loading.value" :error="capability.error.value" @retry="capability.load"/>
    <template v-if="capability.data.value"><ModelRankRows :rows="ranked.slice(0,10)" :vendors="capability.data.value.vendors"/><RequestState :empty="!ranked.length" message="暂无智能指数测量"/><div class="legend"><span v-for="vendor in topVendors" :key="vendor.id"><i :style="{background:vendorColor(vendor.name)}"/>{{ vendor.name }}</span></div><p class="source foot">{{ updated(capability.data.value.as_of) }} 更新 · 北京时间 · 点模型查看完整名称</p></template>
  </section>
  <ModelScatter v-if="capability.data.value" :board="capability.data.value" class="scatter"/>
  <section v-if="capability.data.value" class="board speed"><header><h2>输出速度 Top 5</h2><span class="source">Token/秒</span></header><ModelRankRows :rows="speed" :vendors="capability.data.value.vendors" metric="speed"/><RequestState :empty="!speed.length" message="暂无速度测量"/></section>
  <section class="board usage"><header><h2>调用量 Top 5</h2><RouterLink to="/models?board=usage">全部排行 <span aria-hidden="true">›</span></RouterLink></header><RequestState :loading="usage.loading.value" :error="usage.error.value" @retry="usage.load"/><template v-if="usage.data.value"><p class="source">OpenRouter · {{ usage.data.value.date }} UTC · 按 Token 用量<span v-if="usage.data.value.sample"> · 示例数据</span></p><UsageRankRows :rows="usageTop" preview/><RequestState :empty="!usageTop.length" message="暂无已发布的调用量排行"/></template></section>
  <p v-if="capability.data.value" class="source attribution">数据来源 <a href="https://artificialanalysis.ai/" target="_blank" rel="noopener noreferrer">{{ capability.data.value.source }}</a> · 智能指数 v{{ capability.data.value.index_version }}<br>暂无数据表示未测量，不计入排名。调用量仅统计 OpenRouter 平台流量。</p>
</div></template>
<style scoped>
.boards{display:grid;grid-template-columns:minmax(0,1fr);gap:10px;padding:4px 0 12px}.board{min-width:0;background:var(--surface);padding:14px;border-radius:16px}header{display:flex;align-items:center;justify-content:space-between;gap:8px;margin-bottom:8px}h2{font-size:16px;font-weight:650;margin:0}header a{font-size:12px;color:var(--ink-3);min-height:44px;display:flex;align-items:center;gap:6px;white-space:nowrap}header a span{font-size:20px}.source{font-size:11px;color:var(--ink-3);line-height:1.6;margin:4px 0 8px}.source.foot{margin:8px 0 0}.legend{display:flex;flex-wrap:wrap;gap:6px 14px;margin-top:10px;font-size:11px;color:var(--ink-3)}.legend span{display:flex;align-items:center;gap:4px}.legend i{width:6px;height:6px;border-radius:50%}.attribution{padding:0 2px}.attribution a{color:var(--brand)}
@media(min-width:900px){.boards{grid-template-columns:minmax(0,1fr) minmax(0,1fr);align-items:start}.ability{grid-column:1;grid-row:1}.scatter{grid-column:2;grid-row:1}.speed{grid-column:1}.usage{grid-column:2}.attribution{grid-column:1/-1}}
</style>
