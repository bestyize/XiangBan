<script setup lang="ts">
import { watch } from 'vue'
import { useRoute,useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import RequestState from './RequestState.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import DiscoverRepoRow from './DiscoverRepoRow.vue'
import { useCommunityResource } from './resource'
import { repoMetric,repoGrowth } from './contract'
const route=useRoute(),router=useRouter()
const issueKey=()=>typeof route.query.issue==='string'?route.query.issue:''
const {data,loading,error,load}=useCommunityResource(async()=>{const [issue,refs]=await Promise.all([discoverApi.repos(issueKey()),discoverApi.issues()]);return {issue,refs:refs.items}})
watch(issueKey,load,{immediate:true})
function observationTime(raw:string){const date=new Date(raw);return Number.isNaN(date.getTime())?raw:date.toISOString().slice(0,19).replace('T',' ')+' UTC'}
function select(value:string){void router.replace({query:{...route.query,issue:value||undefined}})}
</script>
<template><div class="repo-board"><RequestState :loading="loading" :error="error" @retry="load"/><template v-if="data"><section class="period-card"><header><div class="period-title"><span class="section-icon"><DiscoverGlyph name="github" :size="20"/></span><div><h2>{{ data.issue.metric==='star total baseline'?'开源项目快照':'开源观测榜' }}</h2><p class="period-date">{{ data.issue.week_start }}<span v-if="data.issue.week_end&&data.issue.week_end!==data.issue.week_start"> – {{ data.issue.week_end }}</span></p></div></div><label class="issue-picker"><DiscoverGlyph name="clock" :size="16"/><span>往期</span><select :value="data.issue.key" aria-label="选择开源榜单期数" @change="select(($event.target as HTMLSelectElement).value)"><option v-for="ref in data.refs" :key="ref.key" :value="ref.key">第 {{ ref.issue_no }} 期 · {{ ref.week_end }}</option></select></label></header><p class="period-summary"><strong>{{ data.issue.ranks.length }}</strong> 个项目<span v-if="data.issue.metric==='star total baseline'"> · 按总 Star 排序 · 增量待积累</span><span v-else> · {{ data.issue.observed_days!=null?`${data.issue.observed_days} 日观测`:'本期观测' }}<span v-if="data.issue.total_gain!=null"> · 净增 {{ repoGrowth(data.issue.total_gain) }}</span></span><span v-if="data.issue.sample" class="sample"> · 示例数据</span></p><details class="method"><summary><span>{{ data.issue.source||'数据来源' }} · 口径与更新时间</span><DiscoverGlyph name="down" :size="14"/></summary><div><p>{{ repoMetric(data.issue) }}。{{ data.issue.metric==='star total baseline'?'当前为初始总量快照，至少两次观测后才计算增量。':data.issue.complete?'已覆盖完整观察周期。':'观测不足完整周期或存在缺采，未补造缺失数据。' }}</p><p v-if="data.issue.as_of">整理于 {{ observationTime(data.issue.as_of) }}</p><p v-if="data.issue.wow_pct!=null">周对比 {{ data.issue.wow_pct }}%</p></div></details></section><div class="repo-list"><DiscoverRepoRow v-for="repo in data.issue.ranks" :key="`${data.issue.key}:${repo.full_name}`" :repo="repo" :baseline="data.issue.metric==='star total baseline'"/></div><RequestState :empty="!data.issue.ranks.length" message="这一期还没有榜单。"/></template></div></template>
<style scoped>
.repo-board{display:grid;gap:8px;padding-top:8px;min-width:0}.period-card{padding:16px;background:var(--surface);border-radius:16px}header{display:flex;align-items:center;justify-content:space-between;gap:8px}.period-title{display:flex;align-items:center;gap:10px;min-width:0}.section-icon{display:grid;place-items:center;flex:none;width:36px;height:36px;background:var(--surface-2);border-radius:10px;color:var(--ink-2)}h2{font-size:16px;margin:0;line-height:1.5}.period-date{font-size:12px;margin:4px 0 0;color:var(--ink-3);line-height:1.5}.issue-picker{display:flex;align-items:center;gap:4px;flex:none;position:relative;font-size:12px;color:var(--ink-3);min-height:44px;cursor:pointer}.issue-picker select{position:absolute;inset:0;opacity:0;width:100%;cursor:pointer}.issue-picker:focus-within{outline:2px solid var(--brand);outline-offset:3px;border-radius:6px}.period-summary{margin:14px 0 4px;font-size:12px;line-height:1.7;color:var(--ink-2)}.period-summary strong{font-size:16px;font-weight:600;color:var(--ink)}.sample{color:var(--brand)}.method{font-size:12px;color:var(--ink-3)}summary{display:flex;align-items:center;justify-content:space-between;gap:8px;min-height:44px;list-style:none;cursor:pointer}summary::-webkit-details-marker{display:none}details[open] summary svg{transform:rotate(180deg)}.method p{margin:4px 0 8px;line-height:1.7}.repo-list{display:grid;gap:8px;min-width:0}.bookmark-note{display:flex;align-items:center;gap:6px;padding:4px;margin:0;font-size:12px;color:var(--ink-3)}@media(min-width:768px){.repo-board{gap:12px}.period-card{padding:20px}.repo-list{grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}}

@media(min-width:1024px){
@container page (min-width:960px){
  .repo-board{grid-template-columns:280px minmax(0,1fr);gap:20px 24px;padding-top:24px;align-items:start}
  .repo-board>:deep(.request-state){grid-column:1/-1}
  .period-card{grid-column:1;grid-row:1;position:sticky;top:calc(var(--header-h,56px) + 24px);padding:24px;align-self:start}
  .period-card header{flex-wrap:wrap;gap:16px}
  .period-title{align-items:flex-start;gap:12px}
  .section-icon{width:40px;height:40px}
  .period-title h2{font-size:20px;line-height:1.4}
  .period-date{margin-top:8px;line-height:1.7}
  .issue-picker{width:100%;justify-content:space-between;padding:0 12px;background:var(--surface-2);border-radius:8px;font-size:14px}
  .issue-picker span{flex:1}
  .period-summary{font-size:14px;line-height:1.9;margin:20px 0 8px}
  .period-summary strong{font-size:24px}
  .method summary{font-size:12px;gap:4px}
  .method p{line-height:1.9}
  .repo-list{grid-column:2;grid-row:1/span 2;grid-template-columns:minmax(0,1fr);gap:10px}
  .repo-list :deep(.repo-card){display:grid;grid-template-columns:minmax(0,1fr) auto;gap:4px 20px;align-items:center;padding:18px 20px;min-height:84px}
  .repo-list :deep(.repo-main){gap:10px}
  .repo-list :deep(.repo-link h3){font-size:16px}
  .repo-list :deep(.repo-link .external){display:none}
  .repo-list :deep(.repo-meta){padding:0;min-width:170px;justify-content:flex-end;gap:8px 12px;max-width:260px}
  .repo-list :deep(.repo-meta .growth){margin-left:0}
  .repo-list :deep(.note){grid-column:1/-1;padding-left:32px;margin-top:0}
  .bookmark-note{grid-column:1;grid-row:2;line-height:1.7;align-self:start;padding:0 8px}
}
}
</style>
