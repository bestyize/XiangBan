<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { discoverApi, licenseLabel, type ToolSummary, type DocItem, type Group } from '@/api/discover'
import type { Page, TopicItem } from '@/api/types'
import CommunityTabs from '@/components/community/CommunityTabs.vue'
import HelpStats from '@/components/community/HelpStats.vue'
import RankingVendor from '@/components/community/RankingVendor.vue'
import ResourceToolIcon from '@/components/community/ResourceToolIcon.vue'
import vendorManifest from '../../public/vendor-logos/manifest.json'
import legacyVendors from '../../public/vendor-logos/legacy.json'
import RequestState from '@/components/community/RequestState.vue'
import ResourceTopicRow from '@/components/community/ResourceTopicRow.vue'
import EditorialArticles from '@/components/community/EditorialArticles.vue'
import PagerNav from '@/components/PagerNav.vue'
import { useCommunityResource } from '@/components/community/resource'
import { webLink } from '@/components/community/contract'
import { useHead } from '@/lib/head'
import { RESOURCE_TABS, resourceTopicCategory } from '@/lib/communityStructure'
import '@/components/community/community.css'
const route = useRoute(), router = useRouter()
const brokenIcons = ref<string[]>([])
const normalizeVendor = (value:string) => value.toLowerCase().split('/')[0]!.replace(/[^\p{L}\p{N}]/gu, '')
function knownBrand(value:string) {
  const n = normalizeVendor(value), lower = value.toLowerCase()
  return vendorManifest.vendors.some(v => v.aliases.some(a => normalizeVendor(a) === n)) || legacyVendors.some(v => v.aliases.some(a => lower.includes(a)))
}
function toolBrand(tool:ToolSummary) { return knownBrand(tool.name) ? tool.name : tool.developer && knownBrand(tool.developer) ? tool.developer : tool.name }

const tabs = RESOURCE_TABS
const tab = computed(() => tabs.some(t => t.key === route.query.tab) ? String(route.query.tab) : 'tools')
const sectionTitle = computed(() => tabs.find(item => item.key === tab.value)?.label || '资源')
const sectionIntro = computed(() => ({ tools:'发现适合自己的 AI 工具', docs:'查阅官方文档和学习资料', media:'分享片单、影评和乐评', help:'提出问题，也帮助他人', open:'发现开源软件与免费素材' }[tab.value] || ''))
const page = computed(() => Math.max(1, Math.floor(Number(route.query.page) || 1)))
const filter = computed(() => String(route.query.filter || ''))
type ResourceData = { kind:'tools'; items:ToolSummary[] } | { kind:'docs'; groups:Group<DocItem>[] } | { kind:'topics'; result:Page<TopicItem> }
const { data, loading, error, load } = useCommunityResource<ResourceData>(async () => {
  if (tab.value === 'tools') return { kind:'tools', ...(await discoverApi.tools()) }
  if (tab.value === 'docs') return { kind:'docs', ...(await discoverApi.docs()) }
  const help_status = tab.value === 'help' && ['open','solved','bounty'].includes(filter.value) ? filter.value as 'open'|'solved'|'bounty' : undefined
  const category = resourceTopicCategory(tab.value, filter.value)
  return { kind:'topics', result: await api.topics({ category, help_status, page:page.value, sort:'new' }) }
})
const filters = computed(() => tab.value === 'help' ? [{key:'',label:'全部'},{key:'open',label:'进行中'},{key:'solved',label:'已采纳'},{key:'bounty',label:'有悬赏'}] : [{key:'',label:'全部'},{key:'movies',label:'影视'},{key:'music',label:'音乐'}])
watch([tab,page,filter], load, { immediate:true })
function setTab(tab:string) { void router.replace({ query: tab === 'tools' ? {} : { tab } }) }
function setFilter(filter:string) { void router.replace({ query:{ tab:tab.value, ...(filter ? {filter} : {}) } }) }
function go(page:number) { void router.replace({ query:{ ...route.query, page } }) }
useHead(() => ({ title:'资源' }))
</script>
<template>
  <main class="community-page resource-page">
    <header class="desktop-heading"><div><p>社区 / 资源</p><h1>资源</h1><span>{{ sectionIntro }}</span></div></header><h1 class="sr-only mobile-heading">社区资源</h1>
    <CommunityTabs :items="tabs" :active="tab" label="资源栏目" @change="setTab" />
    <CommunityTabs v-if="tab === 'media' || tab === 'help'" :items="filters" :active="filter" label="资源筛选" @change="setFilter" />
    <HelpStats v-if="tab === 'help'" />
    <EditorialArticles v-if="tab === 'tools' || tab === 'docs'" :category="resourceTopicCategory(tab,filter)" :title="tab === 'tools' ? '工具导读' : '资料导读'" />
    <p v-if="tab === 'docs'" class="resource-note">官方文档的索引和导读，原文一律跳转官网。</p>
    <p v-else-if="tab === 'media'" class="resource-note">只收片单、影评、乐评和公版或授权内容。</p>
    <p v-else-if="tab === 'open'" class="resource-note">只收开源、免费软件和可免费使用的设计素材。</p>
    <RequestState :loading="loading" :error="error" @retry="load" />
    <template v-if="data?.kind === 'tools'"><div class="desktop-feed-heading"><h2>AI 工具</h2><span>{{ data.items.length }} 款</span></div><div class="resource-feed tool-grid"><RouterLink v-for="tool in data.items" :key="tool.key" :to="`/tools/${encodeURIComponent(tool.key)}`" class="tool-card"><span class="tool-icon"><img v-if="webLink(tool.icon_url) && !brokenIcons.includes(tool.key)" :src="webLink(tool.icon_url)" alt="" loading="lazy" @error="brokenIcons.push(tool.key)"><ResourceToolIcon v-else :name="tool.name" :fallback="toolBrand(tool)" /></span><span class="tool-body"><strong>{{ tool.name }} <span v-if="tool.sample" class="sample">示例</span></strong><span class="tool-meta"><span class="license">{{ licenseLabel(tool.license) }}</span><span v-if="tool.latest_version">{{ tool.latest_version }}</span><span v-if="tool.latest_date">{{ tool.latest_date }} 更新</span><span v-if="tool.type_label">{{ tool.type_label }}</span></span></span><span class="arrow" aria-hidden="true">›</span></RouterLink></div><RequestState :empty="!data.items.length" /></template>
    <template v-else-if="data?.kind === 'docs'"><div class="desktop-feed-heading"><h2>资料索引</h2><span>{{ data.groups.length }} 个分组</span></div><div class="resource-feed doc-grid"><section v-for="group in data.groups" :key="group.name" class="doc-group"><header><span class="doc-icon"><RankingVendor :name="group.name" /></span><h2>{{ group.name || '资料' }}</h2><span>{{ group.items.length }} 篇</span></header><template v-for="doc in group.items" :key="doc.key"><a v-if="webLink(doc.url)" :href="webLink(doc.url)" class="doc-row" target="_blank" rel="noopener noreferrer"><span>{{ doc.title }} <span v-if="doc.sample" class="sample">示例</span></span><span aria-hidden="true">↗</span></a></template></section></div><RequestState :empty="!data.groups.length" /></template>
    <template v-else-if="data?.kind === 'topics'"><div class="desktop-feed-heading"><h2>{{ sectionTitle }}</h2><span>共 {{ data.result.total.toLocaleString('zh-CN') }} 条</span></div><div class="resource-feed topic-grid" :class="`feed-${tab}`"><ResourceTopicRow v-for="topic in data.result.items" :key="topic.id" :topic="topic" :mode="tab === 'media' ? 'media' : tab === 'help' ? 'help' : 'open'" /></div><RequestState :empty="!data.result.items.length" /><PagerNav :page="data.result.page" :page-size="data.result.page_size" :total="data.result.total" @go="go" /></template>
  </main>
</template>
<style scoped>
.desktop-heading,.desktop-feed-heading{display:none}.resource-feed{display:grid;gap:8px;padding:8px 12px}.resource-note{margin:8px 12px 0;font-size:12px;color:var(--ink-3);line-height:1.6}.tool-card{display:flex;align-items:center;gap:12px;min-width:0;background:var(--surface);border-radius:12px;padding:12px 14px}.tool-icon{width:48px;height:48px;flex:none;display:grid;place-items:center;border-radius:12px;background:var(--surface-2)}.tool-icon img{width:100%;height:100%;object-fit:contain;border-radius:12px}.tool-icon :deep(.ranking-vendor){transform:scale(1.5)}.tool-body{display:grid;gap:4px;min-width:0;flex:1}.tool-body strong{font-size:15px;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.tool-meta{display:flex;gap:6px;align-items:center;flex-wrap:wrap;color:var(--ink-3);font-size:12px}.license{padding:1px 6px;border-radius:5px;background:var(--surface-2)}.sample{font-size:12px;color:var(--brand);font-weight:400}.arrow{font-size:20px;color:var(--ink-3)}.doc-group{background:var(--surface);border-radius:12px;padding:14px}.doc-group header{display:flex;align-items:center;gap:8px;margin-bottom:4px}.doc-group header h2{font-size:16px;margin:0;flex:1}.doc-group header>span:last-child{font-size:12px;color:var(--ink-3)}.doc-icon{width:28px;height:28px;display:grid;place-items:center;border-radius:8px;background:var(--surface-2)}.doc-row{display:flex;justify-content:space-between;align-items:center;gap:10px;min-height:52px;padding:12px 0;font-size:14px;border-top:.5px solid var(--line)}.doc-row>span:first-child{min-width:0;overflow-wrap:anywhere}.doc-row>span:last-child{color:var(--ink-3)}@media(min-width:768px){.resource-feed{gap:10px}.tool-card{padding:14px 16px}}
@media(min-width:1100px){
 .mobile-heading{display:none}
 .resource-page{max-width:1120px}.desktop-heading{display:flex;align-items:end;justify-content:space-between;gap:24px;padding:26px 12px 22px}.desktop-heading p{margin:0 0 6px;color:var(--ink-3);font-size:12px}.desktop-heading h1{font-size:28px;line-height:1.2;margin:0 0 8px;letter-spacing:-.02em}.desktop-heading div>span,.heading-section{color:var(--ink-3);font-size:14px}.heading-section{font-size:13px}
 .desktop-feed-heading{display:flex;align-items:baseline;justify-content:space-between;padding:20px 12px 2px}.desktop-feed-heading h2{font-size:17px;margin:0}.desktop-feed-heading span{font-size:12px;color:var(--ink-3)}
 .resource-feed{gap:12px}.tool-grid,.doc-grid,.topic-grid{grid-template-columns:repeat(2,minmax(0,1fr));align-items:start}.tool-card{min-height:96px;padding:16px 18px}.doc-group{padding:18px}.doc-group header{margin-bottom:8px}.topic-grid :deep(.topic-card){height:100%;padding:18px}.resource-note{margin-top:14px}
}
</style>
