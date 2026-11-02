<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { contentApi, type PackItem } from '@/api/content'
import type { Page, TopicItem } from '@/api/types'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { useHead } from '@/lib/head'
import { PACK_KINDS } from '@/lib/content'
import { DISCUSSION_TABS, LEARN_FILTERS, LEARN_CATEGORIES } from '@/lib/communityStructure'
import CommunityTabs from '@/components/community/CommunityTabs.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import ResourceTopicRow from '@/components/community/ResourceTopicRow.vue'
import RequestState from '@/components/community/RequestState.vue'
import type { CommunityScheme } from '@/api/schemes'
import UserAvatar from '@/components/UserAvatar.vue'
import PagerNav from '@/components/PagerNav.vue'
import { useCommunityResource } from '@/components/community/resource'
import '@/components/community/community.css'
const route = useRoute(), router = useRouter(), session = useSession(), meta = useMeta()
const tabs = DISCUSSION_TABS
const sorts = [{key:'latest',label:'最新回复'},{key:'new',label:'新帖'},{key:'hot',label:'热门'},{key:'featured',label:'精选'},{key:'following',label:'关注'}]
const tab = computed(() => {
  const raw = String(route.query.tab || 'create')
  return tabs.some(t => t.key === raw) ? raw : 'all'
})
const sort = computed(() => {
  const raw = String(route.query.sort || route.query.tab || 'latest')
  return sorts.some(t => t.key === raw) ? raw as 'latest'|'new'|'hot'|'featured'|'following' : 'latest'
})
const sectionTitle = computed(() => tab.value === 'all' ? '全部讨论' : tabs.find(item => item.key === tab.value)?.label || '讨论')
const sectionIntro = computed(() => ({ create:'发现社区分享的作品和桌面方案', learn:'交流教程、实践和经验', casual:'聊聊生活里的见闻', all:'浏览社区的最新讨论' }[tab.value] || ''))
const page = computed(() => Math.max(1,Math.floor(Number(route.query.page) || 1)))
const filter = computed(() => typeof route.query.filter === 'string' ? route.query.filter : '')
const learnTags = LEARN_FILTERS
const filters = computed(() => tab.value === 'create' ? [{key:'',label:'全部'},...PACK_KINDS,{key:'schemes',label:'方案'}] : tab.value === 'learn' ? learnTags : [{key:'',label:'全部'},...meta.categories.filter(c => c.slug.startsWith('casual-')).map(c => ({key:c.slug,label:c.name}))])
type Feed = { kind:'packs'; result:Page<PackItem> } | { kind:'schemes'; result:Page<CommunityScheme> } | { kind:'topics'; result:Page<TopicItem> }
const { data, loading, error, load } = useCommunityResource<Feed>(async () => {
  if (tab.value === 'create' && filter.value === 'schemes') return { kind:'schemes',result:await contentApi.schemes(page.value) }
  if (tab.value === 'create') return { kind:'packs',result:await contentApi.packs({ kind:filter.value || undefined, sort:'new',page:page.value }) }
  if (sort.value === 'following' && tab.value === 'all') return {kind:'topics',result:session.loggedIn ? await api.followingTopics(page.value) : {items:[],page:1,page_size:30,total:0}}
  return {kind:'topics',result:await api.topics({category:tab.value === 'learn' ? LEARN_CATEGORIES : tab.value === 'casual' ? filter.value || 'casual' : undefined, tag:tab.value === 'learn' ? filter.value || undefined : undefined, home:tab.value === 'all', sort:sort.value === 'featured' || sort.value === 'following' ? 'new' : sort.value, featured:sort.value === 'featured',page:page.value})}
})
watch([tab,sort,page,filter,() => session.user?.uid,() => session.loggedIn],load,{immediate:true})
// The shell loads category metadata; an empty category list still permits the broad casual feed.
const categoryError = ref('')
async function loadCategories() { categoryError.value = ''; try { meta.categories = await api.categories() } catch { categoryError.value = '分类加载失败' } }
watch(tab,value => { if (value === 'casual' && !meta.categories.length) void loadCategories() },{immediate:true})
function setTab(tab:string) { void router.replace({query:{tab}}) }
function setFilter(filter:string) { void router.replace({query:{tab:tab.value,...(filter ? {filter} : {})}}) }
function setSort(value:string) { if (value === 'following' && !session.requireLogin()) return; void router.replace({query:{...route.query,tab:'all',sort:value,page:undefined}}) }
function go(page:number) { void router.replace({query:{...route.query,page}}) }
useHead(() => ({title:'讨论'}))
</script>
<template>
  <main class="community-page discuss-page">
    <header class="desktop-heading"><div><p>社区 / 讨论</p><h1>讨论</h1><span>{{ sectionIntro }}</span></div></header><h1 class="sr-only mobile-heading">社区讨论</h1>
    <CommunityTabs :items="tabs" :active="tab" label="讨论栏目" @change="setTab" />
    <div class="discussion-tools"><RouterLink :to="tab === 'create' ? filter === 'schemes' ? '/schemes/new' : '/publish' : '/new'" class="post-link">＋ {{ tab === 'create' ? filter === 'schemes' ? '发布方案' : '发布作品' : '发帖' }}</RouterLink><button class="all-topics" :class="{on:tab === 'all'}" @click="setTab('all')">全部讨论 ›</button></div>
    <div v-if="tab !== 'all'" class="filter-row" aria-label="分类筛选"><button v-for="item in filters" :key="item.key" :class="{on:filter === item.key}" :aria-pressed="filter === item.key" @click="setFilter(item.key)">{{ item.label }}</button></div>
    <CommunityTabs v-else :items="sorts" :active="sort" label="帖子排序" @change="setSort" />
    <p v-if="tab === 'casual' && filter === 'casual-chat'" class="feed-note">闲聊灌水不进首页，也不加积分</p>
    <p v-if="categoryError" class="category-error" role="alert">{{ categoryError }} <button class="community-button" @click="loadCategories">重试</button></p>
    <div v-if="tab === 'all' && sort === 'following' && !session.loggedIn" class="login-prompt"><p>登录后查看关注动态。</p><button class="community-button" @click="session.requireLogin()">登录</button></div>
    <template v-else>
      <RequestState :loading="loading" :error="error" @retry="load" />
      <template v-if="data">
        <div class="desktop-feed-heading"><h2>{{ sectionTitle }}</h2><span>共 {{ data.result.total.toLocaleString('zh-CN') }} 条</span></div>
        <div v-if="data.kind === 'packs'" class="pack-grid" :class="{single:['routine','reminder','task','goal','skill','expert'].includes(filter)}"><CommunityPackCard v-for="pack in data.result.items" :key="pack.id" :pack="pack" /></div>
        <div v-else-if="data.kind === 'schemes'" class="topic-feed"><RouterLink v-for="scheme in data.result.items" :key="scheme.id" :to="`/schemes/${scheme.id}`" class="scheme-card"><span class="scheme-kind">桌面方案 · {{ scheme.items.length }} 项</span><strong>{{ scheme.title }}</strong><p v-if="scheme.summary">{{ scheme.summary }}</p><span class="scheme-author"><UserAvatar :name="scheme.author.nickname || '社区用户'" :size="20" />{{ scheme.author.nickname || '社区用户' }}</span></RouterLink></div>
        <div v-else class="topic-feed" :class="{waterfall:tab === 'casual'}"><ResourceTopicRow v-for="topic in data.result.items" :key="topic.id" :topic="topic" /></div>
        <RequestState :empty="!data.result.items.length" /><PagerNav :page="data.result.page" :page-size="data.result.page_size" :total="data.result.total" @go="go" />
      </template>
    </template>
  </main>
</template>
<style scoped>
.desktop-heading,.desktop-feed-heading{display:none}.discussion-tools{display:flex;justify-content:space-between;align-items:center;padding:8px 12px;gap:12px}.post-link,.all-topics{display:inline-flex;align-items:center;min-height:44px;border:0;background:none;font-size:14px}.post-link{color:var(--brand);font-weight:600}.all-topics{color:var(--ink-3)}.all-topics.on{color:var(--brand)}.filter-row{display:flex;gap:8px;overflow-x:auto;padding:4px 12px 12px;scrollbar-width:none}.filter-row::-webkit-scrollbar{display:none}.filter-row button{flex:none;border:0;background:var(--surface);color:var(--ink-2);border-radius:24px;min-height:36px;padding:4px 14px;font-size:12px}.filter-row button.on{color:var(--brand);background:color-mix(in srgb,var(--brand) 9%,var(--surface));font-weight:600}.pack-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px;padding:4px 12px}.pack-grid.single{grid-template-columns:1fr}.topic-feed{display:grid;gap:8px;padding:4px 12px}.topic-feed.waterfall{grid-template-columns:repeat(2,minmax(0,1fr));align-items:start}.scheme-card{display:flex;flex-direction:column;gap:8px;padding:14px;border-radius:12px;background:var(--surface);font-size:15px}.scheme-kind,.scheme-card p,.scheme-author{font-size:12px;color:var(--ink-3);margin:0}.scheme-author{display:flex;align-items:center;gap:6px}.login-prompt{text-align:center;padding:24px;background:var(--surface);border-radius:12px;margin:4px 12px}.category-error,.feed-note{padding:8px 12px;color:var(--ink-3);font-size:12px;margin:0}@media(min-width:768px){.pack-grid{grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}.pack-grid.single{grid-template-columns:repeat(2,minmax(0,1fr))}.topic-feed.waterfall{grid-template-columns:repeat(3,minmax(0,1fr))}}
@media(min-width:1100px){
 .mobile-heading{display:none}
 .discuss-page{max-width:1120px}
 .desktop-heading{display:flex;align-items:end;justify-content:space-between;gap:24px;padding:26px 12px 22px}
 .desktop-heading p{margin:0 0 6px;color:var(--ink-3);font-size:12px}.desktop-heading h1{font-size:28px;line-height:1.2;margin:0 0 8px;letter-spacing:-.02em}.desktop-heading div>span{color:var(--ink-3);font-size:14px}.heading-section{color:var(--ink-3);font-size:13px}
 .desktop-feed-heading{display:flex;align-items:baseline;justify-content:space-between;padding:20px 12px 8px}.desktop-feed-heading h2{font-size:17px;margin:0}.desktop-feed-heading span{font-size:12px;color:var(--ink-3)}
 .discussion-tools{padding:10px 12px 2px}.filter-row{padding-bottom:4px}.pack-grid{gap:14px}.pack-grid.single{grid-template-columns:repeat(2,minmax(0,1fr))}
 .topic-feed{grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;align-items:start}.topic-feed.waterfall{grid-template-columns:repeat(3,minmax(0,1fr))}.topic-feed :deep(.topic-card){height:100%;padding:18px}.scheme-card{min-height:150px;padding:18px}.scheme-card strong{font-size:16px}.scheme-author{margin-top:auto}
}
</style>
