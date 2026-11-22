<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import CommunityBack from '@/components/community/CommunityBack.vue'
import DetailShareDialog from '@/components/community/DetailShareDialog.vue'
import { copyFull, type SharePayload } from '@/lib/detailShare'
import NewsRow from '@/components/community/NewsRow.vue'
import NewsComments from '@/components/community/NewsComments.vue'
import RequestState from '@/components/community/RequestState.vue'
import PagerNav from '@/components/PagerNav.vue'
import { useCommunityResource } from '@/components/community/resource'
import { displayDate, webLink } from '@/components/community/contract'
import { useHead } from '@/lib/head'
import { newsHomePath } from '@/lib/homeNavigation'
import '@/components/community/community.css'
const route = useRoute(), router = useRouter()
const key = computed(() => String(route.params.key || ''))
const page = computed(() => Math.max(1, Math.floor(Number(route.query.page) || 1)))
const article = useCommunityResource(() => discoverApi.frontierArticle(key.value))
const history = useCommunityResource(() => discoverApi.frontier(page.value))
const share = computed<SharePayload | undefined>(() => {
  const item = article.data.value
  if (!item) return
  return {
    title: item.title,
    url: new URL(route.path, location.origin).href,
    badge: '小伴前沿',
    author: item.source,
    meta: [item.source, displayDate(item.source_published_at || item.published_at)].filter(Boolean).join(' · '),
    excerpt: item.summary,
    cover: webLink(item.cover_url),
    icon: 'news',
    extra: item.body_html ? copyFull(item.title, item.body_html) : undefined,
  }
})
watch([key,page], () => { if (key.value) void article.load(); else void history.load() }, { immediate:true })
function go(page:number) { void router.replace({ query:{ page } }) }
useHead(() => ({ title:key.value ? article.data.value?.title || '小伴前沿' : '小伴前沿' }))
</script>
<template><main class="community-page frontier-page"><template v-if="!key"><div class="community-detail"><RouterLink :to="newsHomePath" class="community-back">← 返回资讯</RouterLink><h1>小伴前沿</h1></div><RequestState :loading="history.loading.value" :error="history.error.value" :empty="!!history.data.value && !history.data.value.items.length" @retry="history.load" /><template v-if="history.data.value"><div class="daily-history adaptive-card-grid"><NewsRow v-for="item in history.data.value.items" :key="item.key" :item="item" frontier /></div><PagerNav :page="history.data.value.page" :page-size="history.data.value.page_size" :total="history.data.value.total" @go="go" /></template></template><template v-else><CommunityBack fallback="/frontier" label="小伴前沿" /><RequestState :loading="article.loading.value" :error="article.error.value" @retry="article.load" /><template v-if="article.data.value"><article class="community-detail daily-article"><span v-if="article.data.value.sample" class="community-sample">示例数据</span><div class="daily-heading"><h1>{{ article.data.value.title }}</h1><DetailShareDialog v-if="share" :key="key" :payload="share" /></div><p class="community-meta">{{ article.data.value.source }} · {{ displayDate(article.data.value.source_published_at || article.data.value.published_at) }}<span v-if="article.data.value.time_basis === 'collected'">（采集时间）</span></p><img v-if="webLink(article.data.value.cover_url)" class="daily-cover" :src="webLink(article.data.value.cover_url)" alt="" @error="($event.target as HTMLImageElement).style.display = 'none'"><p v-if="article.data.value.summary" class="daily-summary">{{ article.data.value.summary }}</p><div v-if="article.data.value.body_html" class="prose" v-html="article.data.value.body_html" /><p v-else class="daily-empty">这篇内容暂未整理站内正文。</p><a v-if="webLink(article.data.value.url)" :href="webLink(article.data.value.url)" class="community-button" target="_blank" rel="noopener noreferrer">阅读来源原文 ↗</a></article><NewsComments :key="key" :article-key="key" frontier /></template></template></main></template>

<style scoped>
.daily-history { display:grid; gap:16px; margin-top:12px; }
@media(min-width:768px){.daily-history{grid-template-columns:repeat(var(--card-columns,2),minmax(0,1fr))}}
.community-page .daily-article { max-width:760px; margin:0 auto; padding:24px 22px 28px; }
.community-page .daily-article h1 { font-size:26px; line-height:1.4; font-weight:700; margin:0 0 14px; }
.daily-heading { display:flex; justify-content:space-between; align-items:flex-start; gap:12px; }
.daily-heading :deep(button) { flex:none; }
.daily-cover { display:block; width:100%; max-height:320px; aspect-ratio:16 / 9; object-fit:cover; border-radius:8px; margin:22px 0 0; }
.community-page .daily-summary { margin:24px 0; padding:16px; border-radius:12px; background:var(--surface-2); color:var(--ink-2); font-size:15px; line-height:1.7; white-space:pre-line; }
.daily-article .prose { margin-top:24px; }.daily-empty { color:var(--ink-3); font-size:14px; }.daily-article .community-button { margin-top:24px; }
@media(max-width:767px) { .community-page .daily-article { padding:20px 22px 28px; }.community-page .daily-article h1 { font-size:24px; } }
</style>
