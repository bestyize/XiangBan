<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { contentApi, type PackItem } from '@/api/content'
import type { TopicItem } from '@/api/types'
import type { CommunityScheme } from '@/api/schemes'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
import { PACK_KINDS } from '@/lib/content'
import { DISCUSSION_TABS, DISCUSSION_BOARD_TOPICS, LEARN_FILTERS, canonicalCategory, discussionCategory } from '@/lib/communityStructure'
import ModalDialog from '@/components/ModalDialog.vue'
import MasonryGrid from '@/components/community/MasonryGrid.vue'
import CommunityTabs from '@/components/community/CommunityTabs.vue'
import FilterChips from '@/components/community/FilterChips.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import ResourceTopicRow from '@/components/community/ResourceTopicRow.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import '@/components/community/community.css'

const route = useRoute(), router = useRouter(), session = useSession()
const modes = [{key:'recommend',label:'推荐'}, {key:'following',label:'关注'}]
const workFilters = [{key:'',label:'全部'}, ...PACK_KINDS, {key:'schemes',label:'方案'}]
const query = (key:string) => typeof route.query[key] === 'string' ? route.query[key] : ''
const tab = computed(() => {
  const raw = query('tab')
  const key = raw === 'casual' ? canonicalCategory(query('filter')) : raw
  return DISCUSSION_TABS.some(item => item.key === key) ? key : 'all'
})
const board = computed(() => {
  const key = canonicalCategory(query('board') || (query('tab') === 'casual' ? query('filter') : ''))
  return DISCUSSION_BOARD_TOPICS.some(item => item.key === key) ? key : ''
})
const tag = computed(() => query('tag'))
const filter = computed(() => {
  const options = tab.value === 'create' ? workFilters : tab.value === 'learn' ? LEARN_FILTERS : []
  return options.some(item => item.key === query('filter')) ? query('filter') : ''
})
const following = computed(() => query('mode') === 'following' || query('sort') === 'following' || query('tab') === 'following')
const page = computed(() => Math.min(1000, Math.max(1, Math.floor(Number(query('page')) || 1))))
const category = computed(() => board.value || discussionCategory(tab.value, filter.value === 'schemes' ? '' : filter.value))
const scope = computed(() => ({category:category.value, tag:tag.value || undefined, home:board.value !== 'casual-chat'}))
const topicLabel = computed(() => tag.value || DISCUSSION_BOARD_TOPICS.find(item => item.key === board.value)?.label || '')

type Entry = {kind:'topic'; value:TopicItem} | {kind:'pack'; value:PackItem} | {kind:'scheme'; value:CommunityScheme}
function date(entry:Entry) {
  return Date.parse(entry.kind === 'pack' ? entry.value.updated_at : entry.value.created_at) || 0
}
const { data, loading, error, load } = useCommunityResource(async () => {
  const current = page.value
  const wantsWorks = !board.value && ['all','create'].includes(tab.value)
  if (following.value && !session.loggedIn) return { entries:[] as Entry[], hasMore:false }
  const [topics, packs, schemes] = await Promise.all([
    filter.value === 'schemes' ? null : following.value ? api.followingTopics(current, scope.value) : api.topics({...scope.value, sort:'new', page:current}),
    wantsWorks && filter.value !== 'schemes' ? contentApi.packs({kind:filter.value || undefined, tag:tag.value || undefined, sort:'new', page:current, following:following.value}) : null,
    wantsWorks && !tag.value && (!filter.value || filter.value === 'schemes') ? contentApi.schemes(current, following.value) : null,
  ])
  const packTopics = new Set(packs?.items.map(item => item.topic_id))
  const entries:Entry[] = [
    ...(topics?.items.filter(item => !packTopics.has(item.id)).map(value => ({kind:'topic' as const,value})) ?? []),
    ...(packs?.items.map(value => ({kind:'pack' as const,value})) ?? []),
    ...(schemes?.items.map(value => ({kind:'scheme' as const,value})) ?? []),
  ]
  return { entries:entries.sort((a,b) => date(b)-date(a)), hasMore:[topics,packs,schemes].some(result => result && result.page * result.page_size < result.total) }
})
watch([tab,board,tag,filter,following,page,() => session.user?.uid,() => session.loggedIn],load,{immediate:true})

function navigate(values:Record<string,string|undefined>) {
  void router.replace({query:{tab:tab.value, board:board.value || undefined, tag:tag.value || undefined,
    filter:filter.value || undefined, mode:following.value ? 'following' : undefined, ...values}})
}
function setMode(key:string) { navigate({mode:key === 'following' ? key : undefined}) }
function go(value:number) { navigate({page:String(value)}) }

const topicPicker = ref(false), search = ref('')
const tagFeed = useCommunityResource(api.tags)
const topicChoices = computed(() => [
  ...DISCUSSION_BOARD_TOPICS.map(item => ({...item,board:true})),
  ...(tagFeed.data.value ?? []).map(item => ({key:item.name,label:item.name,board:false})),
].filter(item => item.label.toLowerCase().includes(search.value.trim().toLowerCase())))
function openTopics() { search.value = ''; topicPicker.value = true; if (!tagFeed.data.value) void tagFeed.load() }
function chooseTopic(key:string, isBoard:boolean) {
  topicPicker.value = false
  navigate({tab:'all',filter:undefined,board:isBoard ? key : undefined,tag:isBoard ? undefined : key})
}
useHead(() => ({title:'讨论'}))
</script>

<template>
  <main class="community-page discuss-page">
    <h1 class="sr-only">社区讨论</h1>
    <CommunityTabs :items="modes" :active="following ? 'following' : 'recommend'" label="讨论信息流" @change="setMode">
      <template #actions><button type="button" class="topic-picker" @click="openTopics">找话题</button></template>
    </CommunityTabs>
    <FilterChips :items="DISCUSSION_TABS" :active="tab" label="讨论分类"
      @change="key => navigate({tab:key,filter:undefined,board:undefined,tag:undefined})" />
    <FilterChips v-if="tab === 'create' || tab === 'learn'" :items="tab === 'create' ? workFilters : LEARN_FILTERS"
      :active="filter" :label="tab === 'create' ? '作品类型' : '教程主题'" @change="key => navigate({filter:key || undefined})" />
    <button v-if="topicLabel" class="active-topic" type="button" @click="navigate({tag:undefined,board:undefined})"># {{ topicLabel }} ×</button>
    <p v-if="board === 'casual-chat'" class="feed-note">闲聊灌水不进首页，也不加积分</p>
    <div v-if="following && !session.loggedIn" class="login-prompt">
      <p>登录后查看关注动态。</p><button class="community-button" @click="session.requireLogin()">登录</button>
    </div>
    <template v-else>
      <RequestState :loading="loading" :error="error" @retry="load" />
      <MasonryGrid v-if="data?.entries.length" class="discussion-grid" :columns="2">
        <template v-for="entry in data.entries" :key="entry.kind + ':' + entry.value.id">
          <ResourceTopicRow v-if="entry.kind === 'topic'" :topic="entry.value" layout="masonry" show-category note-cover @select-tag="chooseTopic($event,false)" />
          <CommunityPackCard v-else-if="entry.kind === 'pack'" :pack="entry.value" layout="masonry" discussion />
          <RouterLink v-else :to="'/schemes/' + entry.value.id" class="scheme-card">
            <span>创作 · 桌面方案</span><strong>{{ entry.value.title }}</strong><p>{{ entry.value.summary }}</p>
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
    <ModalDialog :open="topicPicker" title="找话题" @close="topicPicker = false">
      <input v-model="search" class="topic-search" type="search" aria-label="搜索话题" placeholder="搜索话题标签">
      <FilterChips :items="topicChoices" :active="board || tag" label="话题" @change="key => chooseTopic(key, topicChoices.find(item => item.key === key)?.board ?? false)" />
      <RequestState :loading="tagFeed.loading.value" :error="tagFeed.error.value" :empty="!topicChoices.length" @retry="tagFeed.load" />
    </ModalDialog>
  </main>
</template>

<style scoped>
.discussion-grid{margin:4px 12px}.topic-picker{border:0;background:none;color:var(--ink-2);white-space:nowrap;font-size:12px;padding:8px 2px}
.active-topic{margin:0 12px 8px;padding:6px 10px;border:0;border-radius:14px;background:var(--brand-soft);color:var(--brand);font-size:12px}
.scheme-card{display:flex;flex-direction:column;gap:8px;border:1px solid var(--line);border-radius:12px;background:var(--surface);padding:14px;color:var(--ink);overflow-wrap:anywhere}
.scheme-card span,.scheme-card p,.scheme-card small,.feed-note{font-size:12px;color:var(--ink-3);margin:0}
.feed-note{padding:0 12px 8px}.login-prompt{text-align:center;padding:24px}.discussion-pager{display:flex;align-items:center;justify-content:center;gap:12px;padding:16px;font-size:12px}
.topic-search{width:100%;padding:10px;border:1px solid var(--line);border-radius:8px;margin-bottom:8px}
@media(min-width:1100px){.discuss-page{max-width:1120px}.discussion-grid{margin-inline:0}}
</style>
