<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import { contentApi, type PackItem } from '@/api/content'
import type { TopicItem } from '@/api/types'
import type { CommunityScheme } from '@/api/schemes'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useTopicComposer } from '@/stores/topicComposer'
import { galleryTab } from '@/lib/microGallery'
import { discussionTabs, discussionTab, discussionCategory, discussionTopicTag, sectionScope, categoryLabel } from '@/lib/communityStructure'
import type { IdeaCardItem } from '@/lib/ideaCards'
import { ideaCoverFallback, loadFeaturedIdeas } from '@/lib/ideaCatalog'
import { kindLabel } from '@/lib/content'
import CommunityTabs from '@/components/community/CommunityTabs.vue'
import IdeaCard from '@/components/community/IdeaCard.vue'
import MicroGallery from '@/components/community/MicroGallery.vue'
import ResourceTopicRow from '@/components/community/ResourceTopicRow.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'

const { ideas = false } = defineProps<{ ideas?: boolean }>()
const route = useRoute(), router = useRouter(), session = useSession()
const meta = useMeta()
const composer = useTopicComposer()
const query = (key:string) => typeof route.query[key] === 'string' ? route.query[key] : ''
const showGallery = computed(() => ideas && query('tab') === galleryTab.key)
const tab = computed(() => ideas ? '' : discussionTab(meta.categories, query('tab'), query('filter'), query('board')))
const discussionOptions = computed(() => discussionTabs(meta.categories))
const tag = computed(() => query('tag'))
const following = computed(() => query('mode') === 'following' || query('sort') === 'following' || query('tab') === 'following')
const page = computed(() => Math.min(1000, Math.max(1, Math.floor(Number(query('page')) || 1))))
const category = computed(() => ideas ? sectionScope(meta.categories, 'ideas') : discussionCategory(tab.value, meta.categories))
const scope = computed(() => ({
  category:category.value,
  tag:(!ideas ? discussionTopicTag(tab.value) : undefined) || tag.value || undefined,
  active_only:true,
  home:!ideas && (tab.value === 'all' || Boolean(discussionTopicTag(tab.value))),
}))

type Entry = {kind:'topic'; value:TopicItem & { images?: string[] }} | {kind:'pack'; value:PackItem} | {kind:'scheme'; value:CommunityScheme}
function date(entry:Entry) {
  return Date.parse(entry.kind === 'pack' ? entry.value.updated_at : entry.value.created_at) || 0
}
const { data, loading, error, load, refresh } = useCommunityResource(async () => {
  const current = page.value
  if (showGallery.value || (following.value && !session.loggedIn) || !meta.categories.length) return null
  const [topics, packs, schemes] = await Promise.all([
    following.value ? api.followingTopics(current, scope.value) : api.topics({...scope.value, sort:'new', page:current}),
    ideas ? contentApi.packs({ tag:tag.value || undefined, sort:'new', page:current, following:following.value}) : null,
    ideas && !tag.value ? contentApi.schemes(current, following.value) : null,
  ])
  const packTopics = new Set(packs?.items.map(item => item.topic_id))
  const entries:Entry[] = [
    ...(topics?.items.filter(item => !packTopics.has(item.id)).map(value => ({kind:'topic' as const,value})) ?? []),
    ...(packs?.items.map(value => ({kind:'pack' as const,value})) ?? []),
    ...(schemes?.items.map(value => ({kind:'scheme' as const,value})) ?? []),
  ]
  return { entries:entries.sort((a,b) => date(b)-date(a)), hasMore:[topics,packs,schemes].some(result => result && result.page * result.page_size < result.total) }
})
const showFeaturedIdeas = computed(() => ideas && !showGallery.value && !tag.value && !following.value && page.value === 1)
const featuredCatalog = useCommunityResource(loadFeaturedIdeas)
watch(showFeaturedIdeas, show => { if (show) void featuredCatalog.refresh() }, { immediate: true })
const featuredIdeas = computed(() => showFeaturedIdeas.value ? featuredCatalog.data.value ?? [] : [])

function communityIdeaCard(entry: Entry): IdeaCardItem {
  if (entry.kind === 'topic') {
    const topic = entry.value
    return {
      id: 'topic-' + topic.id, to: { name: 'topic', params: { id: topic.id } },
      title: topic.title, description: topic.excerpt, cover: topic.images?.[0],
      label: categoryLabel(topic.category), author: topic.author.nickname, date: topic.created_at,
    }
  }
  if (entry.kind === 'pack') {
    const pack = entry.value
    return {
      id: 'pack-' + pack.id, to: { name: 'pack', params: { id: pack.id } },
      title: pack.title, description: pack.summary, cover: pack.cover_url,
      label: kindLabel(pack.kind), author: pack.author?.nickname, date: pack.created_at,
    }
  }
  const scheme = entry.value
  return {
    id: 'scheme-' + scheme.id, to: { name: 'scheme', params: { id: scheme.id } },
    title: scheme.title, description: scheme.summary, label: '桌面方案',
    author: scheme.author.nickname, date: scheme.created_at,
  }
}

const featuredCardPresentation = new Map<string, Pick<IdeaCardItem, 'avatarColor'> & { title?: string; label?: string }>([
  ['jiangshan-map', { avatarColor: '#A8640C' }],
  ['micro-gallery', { avatarColor: '#07C160', title: '微观展厅 · 龙鱼水族箱', label: '展厅' }],
  ['plush-rabbit', { avatarColor: '#D93A44', label: '3D 模型' }],
  ['starry-nightrider', { avatarColor: '#7152D8' }],
  ['little-prince', { avatarColor: '#2F6FDB' }],
  ['saltline', { avatarColor: '#2B2B30' }],
  ['oil-paint', { avatarColor: '#D93A44', title: 'Oil Paint · 实时油画漫步' }],
  ['whale-girl', { avatarColor: '#2F6FDB' }],
])
const featuredCards = computed<IdeaCardItem[]>(() => {
  const catalog = new Map(featuredIdeas.value.map(item => [item.id, item]))
  const order = new Set([...featuredCardPresentation.keys(), ...catalog.keys()])
  return [...order].flatMap(id => {
    const item = catalog.get(id)
    if (!item) return []
    const bundledCover = ideaCoverFallback(id)
    return [{
      ...item,
      ...featuredCardPresentation.get(id),
      cover: bundledCover ?? item.cover,
      fallbackCover: bundledCover ? item.cover : undefined,
      to: item.kind === 'gallery' ? item.href : { name: 'idea-work', params: { slug: id } },
    }]
  })
})
const spotlight = computed(() => featuredCards.value.find(card => card.id === 'whale-girl' && card.cover))
const ideaCards = computed(() => {
  if (!ideas || showGallery.value) return []
  const featuredPacks = new Set(featuredIdeas.value.flatMap(item => item.pack_slug ? [item.pack_slug] : []))
  const entries = data.value?.entries ?? []
  const community = entries
    .filter(entry => entry.kind !== 'pack' || !featuredPacks.has(entry.value.slug))
    .map(communityIdeaCard)
  return [...featuredCards.value, ...community]
})
watch([() => meta.categories,() => ideas,showGallery,tab,tag,following,page,() => session.user?.uid,() => session.loggedIn],load,{immediate:true})
watch(() => composer.saved, saved => { if (saved && !showGallery.value) void refresh() })

function navigate(values:Record<string,string|undefined>) {
  void router.replace({query:{
    tab: ideas ? (showGallery.value ? galleryTab.key : undefined) : tab.value,
    tag: tag.value || undefined, mode: following.value ? 'following' : undefined, ...values,
  }})
}
function changeDiscussionTab(key:string) {
  navigate({ tab:key, tag:undefined, page:undefined })
}
function go(value:number) { navigate({page:String(value)}) }
</script>

<template>
  <div class="community-post-feed">
    <CommunityTabs v-if="!ideas" class="page-section-tabs discussion-tabs" :items="discussionOptions" :active="tab" label="讨论板块" scrollable
      @change="changeDiscussionTab" />
    <button v-if="!showGallery && tag" class="active-topic" type="button" @click="navigate({tag:undefined,page:undefined})"># {{ tag }} ×</button>
    <p v-if="category === 'casual-chat'" class="feed-note">闲聊灌水不进首页，也不加积分</p>
    <RequestState v-if="showFeaturedIdeas" :loading="featuredCatalog.loading.value" :error="featuredCatalog.error.value"
      @retry="featuredCatalog.refresh" />
    <IdeaCard v-if="spotlight" class="ideas-spotlight" :card="spotlight" spotlight />
    <div v-if="ideaCards.length" class="ideas-grid">
      <IdeaCard v-for="card in ideaCards" :key="card.id" :card="card" />
    </div>
    <MicroGallery v-if="showGallery" />
    <RequestState v-else-if="!meta.categories.length && (meta.categoriesLoading || meta.categoriesError)"
      :loading="meta.categoriesLoading" :error="meta.categoriesError" @retry="meta.load()" />
    <div v-else-if="following && !session.loggedIn" class="login-prompt">
      <p>登录后查看关注动态。</p><button class="community-button" @click="session.requireLogin()">登录</button>
    </div>
    <template v-else>
      <RequestState :loading="loading" :error="error" @retry="load" />
      <div v-if="!ideas && data?.entries.length" class="discussion-grid">
        <template v-for="entry in data.entries" :key="entry.kind + ':' + entry.value.id">
          <ResourceTopicRow v-if="entry.kind === 'topic'" :topic="entry.value" mode="discussion"
            @select-tag="navigate({tab:'all',tag:$event,page:undefined})" />
        </template>
      </div>
      <RequestState v-if="data && !data.entries.length && !ideaCards.length && (!showFeaturedIdeas || (!featuredCatalog.loading.value && !featuredCatalog.error.value))" empty />
      <div v-if="data && (page > 1 || data.hasMore)" class="discussion-pager">
        <button class="community-button" :disabled="page <= 1" @click="go(page - 1)">上一页</button>
        <span>第 {{ page }} 页</span>
        <button class="community-button" :disabled="!data.hasMore || page >= 1000" @click="go(page + 1)">下一页</button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.discussion-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 12px;
  max-width: 880px;
}
.ideas-spotlight { margin-bottom: 8px; }
.ideas-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}
@media (min-width: 1024px) {
  .ideas-spotlight { margin-bottom: 18px; }
  .ideas-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 16px; }
}
.active-topic{margin:0 12px 8px;padding:6px 10px;border:0;border-radius:14px;background:var(--brand-soft);color:var(--brand);font-size:12px}
.feed-note{font-size:12px;color:var(--ink-3);margin:0}
.feed-note{padding:0 12px 8px}.login-prompt{text-align:center;padding:24px}.discussion-pager{display:flex;align-items:center;justify-content:center;gap:12px;padding:16px;font-size:12px}
</style>
