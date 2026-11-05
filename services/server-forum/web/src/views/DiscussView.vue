<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { contentApi, type PackItem } from '@/api/content'
import type { TopicItem } from '@/api/types'
import type { CommunityScheme } from '@/api/schemes'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
import { discussionTabs, discussionFilters, discussionFilter, canonicalCategory, discussionCategory, sectionScope, sectionCategories, sectionLeaves, categoryDestination } from '@/lib/communityStructure'
import MasonryGrid from '@/components/community/MasonryGrid.vue'
import FilterChips from '@/components/community/FilterChips.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import ResourceTopicRow from '@/components/community/ResourceTopicRow.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import '@/components/community/community.css'

const route = useRoute(), router = useRouter(), session = useSession()
const meta = useMeta()
const DISCUSSION_TABS = computed(()=>discussionTabs(meta.categories))
const DISCUSSION_BOARD_TOPICS = computed(()=>sectionCategories(meta.categories,'discuss').filter(c=>c.parent_id !== null).map(c=>({key:c.slug,label:c.name})))
const ideaCategories = computed(()=>sectionLeaves(meta.categories,'ideas'))
const siteRoots = computed(()=>sectionCategories(meta.categories,'site').filter(c=>c.parent_id === null))
const ideaFilter = computed(()=>ideaCategories.value.some(c=>c.slug===query('category')) ? query('category') : '')
const ideaKind = computed(()=>ideaFilter.value.replace(/^create-/,''))
const packKinds = new Set(['game','widget','wallpaper','effect','pet','skill','expert','workflow','reminder','routine','task','goal'])
const ideas = computed(() => route.name === 'ideas')
const query = (key:string) => typeof route.query[key] === 'string' ? route.query[key] : ''
const tab = computed(() => {
  if (ideas.value) return 'create'
  const raw = query('tab')
  const key = raw === 'casual' ? canonicalCategory(query('filter')) : raw
  return DISCUSSION_TABS.value.some(item => item.key === key) ? key : 'all'
})
const board = computed(() => {
  if (ideas.value) return ''
  const rawTab = query('tab')
  const legacyTab = rawTab === 'casual' ? query('filter') : rawTab
  const key = canonicalCategory(query('board') || legacyTab)
  return key !== tab.value && DISCUSSION_BOARD_TOPICS.value.some(item => item.key === key) ? key : ''
})
const tag = computed(() => query('tag'))
const subFilters = computed(() => discussionFilters(meta.categories,tab.value))
const filter = computed(() => discussionFilter(meta.categories,tab.value,query('filter')))
const following = computed(() => query('mode') === 'following' || query('sort') === 'following' || query('tab') === 'following')
const page = computed(() => Math.min(1000, Math.max(1, Math.floor(Number(query('page')) || 1))))
const category = computed(() => ideas.value ? ideaFilter.value || sectionScope(meta.categories,'ideas') : board.value || discussionCategory(tab.value, filter.value, meta.categories))
const scope = computed(() => ({category:category.value, tag:tag.value || undefined, active_only:true, home:category.value !== 'casual-chat'}))
const topicLabel = computed(() => tag.value || DISCUSSION_BOARD_TOPICS.value.find(item => item.key === board.value)?.label || '')

type Entry = {kind:'topic'; value:TopicItem} | {kind:'pack'; value:PackItem} | {kind:'scheme'; value:CommunityScheme}
function date(entry:Entry) {
  return Date.parse(entry.kind === 'pack' ? entry.value.updated_at : entry.value.created_at) || 0
}
const { data, loading, error, load } = useCommunityResource(async () => {
  const current = page.value
  if (following.value && !session.loggedIn) return { entries:[] as Entry[], hasMore:false }
  const [topics, packs, schemes] = await Promise.all([
    following.value ? api.followingTopics(current, scope.value) : api.topics({...scope.value, sort:'new', page:current}),
    ideas.value && (!ideaKind.value || packKinds.has(ideaKind.value)) ? contentApi.packs({ kind:ideaKind.value || undefined, tag:tag.value || undefined, sort:'new', page:current, following:following.value}) : null,
    ideas.value && !tag.value && (!ideaFilter.value || ideaFilter.value === 'create-schemes') ? contentApi.schemes(current, following.value) : null,
  ])
  const packTopics = new Set(packs?.items.map(item => item.topic_id))
  const entries:Entry[] = [
    ...(topics?.items.filter(item => !packTopics.has(item.id)).map(value => ({kind:'topic' as const,value})) ?? []),
    ...(packs?.items.map(value => ({kind:'pack' as const,value})) ?? []),
    ...(schemes?.items.map(value => ({kind:'scheme' as const,value})) ?? []),
  ]
  return { entries:entries.sort((a,b) => date(b)-date(a)), hasMore:[topics,packs,schemes].some(result => result && result.page * result.page_size < result.total) }
})
watch([() => meta.categories,ideaFilter,tab,board,tag,filter,following,page,() => session.user?.uid,() => session.loggedIn],load,{immediate:true})

function navigate(values:Record<string,string|undefined>) {
  void router.replace({query:{tab:ideas.value ? undefined : tab.value, category:ideas.value ? ideaFilter.value || undefined : undefined, board:board.value || undefined, tag:tag.value || undefined,
    filter:filter.value || undefined, mode:following.value ? 'following' : undefined, ...values}})
}
function go(value:number) { navigate({page:String(value)}) }

useHead(() => ({title:ideas.value ? '创意' : '讨论'}))
</script>

<template>
  <main class="community-page discuss-page">
    <h1 class="sr-only">{{ ideas ? '社区创意' : '社区讨论' }}</h1>
    <FilterChips v-if="!ideas" :items="DISCUSSION_TABS" :active="tab" label="讨论分类"
      @change="key => navigate({tab:key,filter:undefined,board:undefined,tag:undefined})" />
    <FilterChips v-if="subFilters.length" :items="subFilters"
      :active="filter" label="分类主题" @change="key => navigate({filter:key || undefined})" />
    <FilterChips v-if="ideas" :items="[{key:'',label:'全部'},...ideaCategories.map(c=>({key:c.slug,label:c.name}))]" :active="ideaFilter" label="创意分类" @change="key=>navigate({category:key || undefined,page:undefined})" />
    <nav v-if="!ideas" class="discussion-extra" aria-label="其他板块"><RouterLink v-for="c in siteRoots" :key="c.id" :to="categoryDestination(c,meta.categories)">{{ c.name }}</RouterLink></nav>
    <button v-if="topicLabel" class="active-topic" type="button" @click="navigate({tag:undefined,board:undefined})"># {{ topicLabel }} ×</button>
    <p v-if="category === 'casual-chat'" class="feed-note">闲聊灌水不进首页，也不加积分</p>
    <div v-if="following && !session.loggedIn" class="login-prompt">
      <p>登录后查看关注动态。</p><button class="community-button" @click="session.requireLogin()">登录</button>
    </div>
    <template v-else>
      <RequestState :loading="loading" :error="error" @retry="load" />
      <MasonryGrid v-if="data?.entries.length" class="discussion-grid" :columns="2">
        <template v-for="entry in data.entries" :key="entry.kind + ':' + entry.value.id">
          <ResourceTopicRow v-if="entry.kind === 'topic'" :topic="entry.value" layout="masonry" show-category note-cover @select-tag="navigate({tab:ideas ? undefined : 'all',filter:undefined,board:undefined,tag:$event})" />
          <CommunityPackCard v-else-if="entry.kind === 'pack'" :pack="entry.value" layout="masonry" discussion />
          <RouterLink v-else :to="'/schemes/' + entry.value.id" class="scheme-card">
            <span>创意 · 桌面方案</span><strong>{{ entry.value.title }}</strong><p>{{ entry.value.summary }}</p>
            <small>{{ entry.value.author.nickname || '社区作品' }} · {{ entry.value.items.length }} 项 · 查看导入</small>
          </RouterLink>
        </template>
      </MasonryGrid>
      <RequestState v-if="data && !data.entries.length" empty />
      <div v-if="data && (page > 1 || data.hasMore)" class="discussion-pager">
        <button class="community-button" :disabled="page <= 1" @click="go(page - 1)">上一页</button>
        <span>第 {{ page }} 页</span>
        <button class="community-button" :disabled="!data.hasMore || page >= 1000" @click="go(page + 1)">下一页</button>
      </div>
    </template>
  </main>
</template>

<style scoped>
.discussion-grid{margin:4px 12px}
@container page (min-width: 720px) {
  .discuss-page .discussion-grid.two-columns { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
@container page (min-width: 960px) {
  .discuss-page .discussion-grid.two-columns { grid-template-columns: repeat(4, minmax(0, 1fr)); }
}
.active-topic{margin:0 12px 8px;padding:6px 10px;border:0;border-radius:14px;background:var(--brand-soft);color:var(--brand);font-size:12px}
.scheme-card{display:flex;flex-direction:column;gap:8px;border:1px solid var(--line);border-radius:12px;background:var(--surface);padding:14px;color:var(--ink);overflow-wrap:anywhere}
.scheme-card span,.scheme-card p,.scheme-card small,.feed-note{font-size:12px;color:var(--ink-3);margin:0}
.feed-note{padding:0 12px 8px}.login-prompt{text-align:center;padding:24px}.discussion-pager{display:flex;align-items:center;justify-content:center;gap:12px;padding:16px;font-size:12px}
@media(min-width:1100px){.discuss-page{max-width:1120px}.discussion-grid{margin-inline:0}}
</style>
