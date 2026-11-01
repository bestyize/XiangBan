<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useHead } from '@/lib/head'
import { availableMetrics, METRICS, rankModels, formatTokens, type Board, type Usage } from '@/lib/modelRankings'
import { useCommunityResource } from '@/components/community/resource'
import RequestState from '@/components/community/RequestState.vue'
import ModelRankRows from '@/components/community/ModelRankRows.vue'
import UsageRankRows from '@/components/community/UsageRankRows.vue'
import ModelScatter from '@/components/community/ModelScatter.vue'
import { request } from '@/api/forum'
const route = useRoute(), router = useRouter()
const tab = computed(() => route.query.board === 'usage' ? 'usage' : 'capability')
const date = computed(() => typeof route.query.date === 'string' ? route.query.date : '')
function readMetric() { try { return localStorage.getItem('forum-ranking-metric') || 'intelligence' } catch { return 'intelligence' } }
const metric = ref(readMetric())
const ability = useCommunityResource(() => request<Board>('GET','/discover/models/v2',undefined,false))
const usage = useCommunityResource(() => request<Usage>('GET',`/discover/model-usage${date.value ? `?date=${encodeURIComponent(date.value)}`:''}`,undefined,false))
const history = useCommunityResource(() => request<{items:{date:string}[]}>('GET','/discover/model-usage/dates',undefined,false))
const options = computed(() => ability.data.value ? availableMetrics(ability.data.value) : [METRICS[0]!])
const selectedMetric = computed(() => options.value.find(item=>item.id===metric.value) ?? options.value[0]!)
const rows = computed(() => rankModels(ability.data.value?.models ?? [],selectedMetric.value.id))
const usageRows = computed(() => [...(usage.data.value?.models ?? [])].sort((a,b)=>a.rank-b.rank))
const dates = computed(() => [...new Set(history.data.value?.items.map(item=>item.date) ?? [])].sort().reverse())
function groups<T>(items:T[], size:number):T[][] { return Array.from({length:Math.ceil(items.length/size)},(_,index)=>items.slice(index*size,(index+1)*size)) }
function selectMetric(id:string) { metric.value=id; try {localStorage.setItem('forum-ranking-metric',id)} catch {} }
function setTab(value:string) { void router.push({path:'/models',query:value==='usage'?{board:'usage'}:{}}) }
function setDate(value:string) { void router.replace({path:'/models',query:{board:'usage',...(value?{date:value}:{})}}) }
function reload() { if(tab.value==='usage'){void usage.load();void history.load()}else void ability.load() }
watch([tab,date],reload,{immediate:true})
onMounted(() => { if(tab.value!=='usage') void history.load() })
function updated(value:string) { const date=new Date(value);return Number.isNaN(date.getTime())?'更新时间未知':date.toLocaleString('zh-CN',{timeZone:'Asia/Shanghai'}) }
useHead(() => ({title:tab.value==='usage'?'模型调用量榜':'模型能力榜'}))
</script>
<template><div class="rankings">
  <header class="rankings-header"><div><RouterLink class="back" to="/?tab=rankings">‹ 排行榜概览</RouterLink><h1>{{ tab==='usage'?'模型调用量榜':'模型能力榜' }}</h1></div><button type="button" class="refresh" :disabled="tab==='usage'?usage.loading.value:ability.loading.value" @click="reload">刷新</button></header>
  <nav class="rank-tabs" aria-label="排行榜类型"><button type="button" :aria-pressed="tab==='capability'" :class="{on:tab==='capability'}" @click="setTab('capability')">能力榜</button><button type="button" :aria-pressed="tab==='usage'" :class="{on:tab==='usage'}" @click="setTab('usage')">调用量榜</button></nav>
  <template v-if="tab==='capability'">
    <div class="filters" aria-label="排序指标"><button v-for="option in options" :key="option.id" :aria-pressed="selectedMetric.id===option.id" :class="{on:selectedMetric.id===option.id}" @click="selectMetric(option.id)">{{ option.label }}</button></div>
    <p class="note">{{ selectedMetric.unit }} · {{ selectedMetric.higherIsBetter?'越高越好':'越低越好' }} · 缺失测量不参与排名</p>
    <RequestState :loading="ability.loading.value" :error="ability.error.value" @retry="ability.load"/>
    <template v-if="ability.data.value"><p v-if="ability.data.value.sample" class="note">示例数据</p><RequestState :empty="!rows.length" message="暂无模型数据"/><section v-for="(group,index) in groups(rows,25)" :key="index" class="rank-card" :aria-label="`模型列表第 ${index+1} 组`"><ModelRankRows :rows="group" :vendors="ability.data.value.vendors" :metric="selectedMetric.id"/></section><ModelScatter :board="ability.data.value"/><p class="note source">数据来源 <a href="https://artificialanalysis.ai/" target="_blank" rel="noopener noreferrer">{{ ability.data.value.source }}</a><br>智能指数 v{{ ability.data.value.index_version }} · {{ updated(ability.data.value.as_of) }} 更新（北京时间）。暂无数据表示未测量，不计入排名。</p></template>
  </template>
  <template v-else>
    <div class="filters" aria-label="统计日期"><button :class="{on:!date}" :aria-pressed="!date" @click="setDate('')">最新日榜</button><button v-for="day in dates" :key="day" :class="{on:date===day}" :aria-pressed="date===day" @click="setDate(day)">{{ day.slice(5) }} UTC</button></div>
    <p v-if="history.error.value" class="note" role="alert">日期加载失败 <button class="text-button" @click="history.load">重试</button></p>
    <RequestState :loading="usage.loading.value" :error="usage.error.value" @retry="usage.load"/>
    <button v-if="usage.error.value && date" class="refresh" @click="setDate('')">查看最新日榜</button>
    <template v-if="usage.data.value"><section class="rank-card usage-summary"><header><h2>调用量 Top {{ usageRows.length }}</h2><span>{{ usage.data.value.date }} · UTC</span></header><p class="note">OpenRouter · 输入与输出 Token 总量<span v-if="usage.data.value.sample"> · 示例数据</span></p><p>全天 {{ formatTokens(usage.data.value.total_tokens) }} Token</p></section><RequestState :empty="!usageRows.length" message="暂无模型用量数据"/><section v-for="(group,index) in groups(usageRows,10)" :key="index" class="rank-card"><UsageRankRows :rows="group" :previous-available="usage.data.value.previous_date_available"/></section><p class="note source">数据来源 <a href="https://openrouter.ai/rankings" target="_blank" rel="noopener noreferrer">OpenRouter</a> · {{ updated(usage.data.value.as_of) }} 更新（北京时间）<br>其他模型 {{ formatTokens(usage.data.value.other_tokens) }} Token，计入全天总量。排名变化与前一完整 UTC 日对比，不代表请求次数。</p></template>
  </template>
</div></template>
<style scoped>
.rankings{max-width:960px;margin:auto;min-width:0}.rankings-header{display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:14px}h1{font-size:21px;margin:6px 0 0}h2{font-size:16px;margin:0}.back{font-size:12px;color:var(--ink-3);display:inline-flex;min-height:32px;align-items:center}.refresh{border:1px solid var(--line);border-radius:10px;padding:8px 16px;min-height:44px;background:var(--surface)}button:disabled{opacity:.5}.rank-tabs{display:flex;gap:20px;border-bottom:1px solid var(--line);margin-bottom:8px}.rank-tabs button{background:none;border:0;padding:10px 2px;min-height:44px;color:var(--ink-3);font-size:15px}.rank-tabs .on{color:var(--ink);border-bottom:3px solid var(--brand);font-weight:650}.filters{display:flex;gap:7px;overflow-x:auto;padding:6px 1px;scrollbar-width:thin}.filters button{white-space:nowrap;flex:none;border:0;border-radius:9px;background:var(--surface);color:var(--ink-2);padding:8px 12px;min-height:44px;font-size:12px}.filters .on{background:var(--ink);color:var(--paper)}.note{font-size:11px;line-height:1.7;color:var(--ink-3);margin:8px 2px 12px}.rank-card{background:var(--surface);border-radius:14px;padding:12px 14px;margin:0 0 10px}.usage-summary header{display:flex;justify-content:space-between;gap:12px;align-items:center}.usage-summary header>span{font-size:11px;color:var(--ink-3)}.usage-summary p:last-child{margin:6px 0 0;font-size:13px;overflow-wrap:anywhere}.source a,.text-button{color:var(--brand)}.text-button{background:none;border:0;min-height:44px;padding:0 10px}.source{padding:8px 2px}@media(min-width:900px){.rankings{max-width:900px}.rank-card{padding:16px 22px}}
</style>
