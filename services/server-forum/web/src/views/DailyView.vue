<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import CommunityBack from '@/components/community/CommunityBack.vue'
import NewsRow from '@/components/community/NewsRow.vue'
import NewsComments from '@/components/community/NewsComments.vue'
import RequestState from '@/components/community/RequestState.vue'
import PagerNav from '@/components/PagerNav.vue'
import { useCommunityResource } from '@/components/community/resource'
import { displayDate, webLink } from '@/components/community/contract'
import { useHead } from '@/lib/head'
import '@/components/community/community.css'
const route = useRoute(), router = useRouter()
const key = computed(() => String(route.params.key || ''))
const page = computed(() => Math.max(1, Math.floor(Number(route.query.page) || 1)))
const article = useCommunityResource(() => discoverApi.article(key.value))
const history = useCommunityResource(() => discoverApi.daily(page.value))
watch([key,page], () => { if (key.value) void article.load(); else void history.load() }, { immediate:true })
function go(page:number) { void router.replace({ query:{ page } }) }
useHead(() => ({ title:key.value ? article.data.value?.title || '日报' : '往期日报' }))
</script>
<template><main class="community-page"><template v-if="!key"><div class="community-detail"><RouterLink to="/" class="community-back">← 社区首页</RouterLink><h1>往期日报</h1></div><RequestState :loading="history.loading.value" :error="history.error.value" :empty="!!history.data.value && !history.data.value.items.length" @retry="history.load" /><template v-if="history.data.value"><div class="daily-history"><NewsRow v-for="item in history.data.value.items" :key="item.key" :item="item" /></div><PagerNav :page="history.data.value.page" :page-size="history.data.value.page_size" :total="history.data.value.total" @go="go" /></template></template><template v-else><CommunityBack fallback="/daily" label="往期日报" /><RequestState :loading="article.loading.value" :error="article.error.value" @retry="article.load" /><template v-if="article.data.value"><article class="community-detail daily-article"><span v-if="article.data.value.sample" class="community-sample">示例数据</span><h1>{{ article.data.value.title }}</h1><p class="community-meta">{{ article.data.value.source }} · {{ displayDate(article.data.value.source_published_at || article.data.value.published_at) }}<span v-if="article.data.value.time_basis === 'collected'">（采集时间）</span></p><img v-if="webLink(article.data.value.cover_url)" class="daily-cover" :src="webLink(article.data.value.cover_url)" alt="" @error="($event.target as HTMLImageElement).style.display = 'none'"><p v-if="article.data.value.summary" class="daily-summary">{{ article.data.value.summary }}</p><div v-if="article.data.value.body_html" class="prose" v-html="article.data.value.body_html" /><p v-else class="daily-empty">这篇内容暂未整理站内正文。</p><a v-if="webLink(article.data.value.url)" :href="webLink(article.data.value.url)" class="community-button" target="_blank" rel="noopener noreferrer">阅读来源原文 ↗</a></article><NewsComments :key="key" :article-key="key" /></template></template></main></template>

<style scoped>
.daily-history { display:grid; gap:8px; margin-top:12px; }
.community-page .daily-article { max-width:760px; margin:0 auto; padding:24px 22px 28px; }
.community-page .daily-article h1 { font-size:26px; line-height:1.4; font-weight:700; margin:0 0 14px; }
.daily-cover { display:block; width:100%; max-height:320px; aspect-ratio:16 / 9; object-fit:cover; border-radius:8px; margin:22px 0 0; }
.community-page .daily-summary { margin:24px 0; padding:16px; border-radius:12px; background:var(--surface-2); color:var(--ink-2); font-size:15px; line-height:1.7; white-space:pre-line; }
.daily-article .prose { margin-top:24px; }.daily-empty { color:var(--ink-3); font-size:14px; }.daily-article .community-button { margin-top:24px; }
@media(max-width:767px) { .community-page .daily-article { padding:20px 22px 28px; }.community-page .daily-article h1 { font-size:24px; } }
</style>
