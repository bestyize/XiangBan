<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import AppIcon from '@/components/AppIcon.vue'
import DetailLayout from '@/components/community/DetailLayout.vue'
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
const article = useCommunityResource(() => discoverApi.article(key.value))
const history = useCommunityResource(() => discoverApi.daily(page.value))
const item = computed(() => article.data.value)
const cover = computed(() => webLink(item.value?.cover_url))
const sourceUrl = computed(() => webLink(item.value?.url))
const share = computed<SharePayload | undefined>(() => {
  const current = item.value
  if (!current) return
  return {
    title: current.title,
    url: new URL(route.path, location.origin).href,
    badge: 'AI 日报',
    author: current.source,
    meta: [current.source, displayDate(current.source_published_at || current.published_at)].filter(Boolean).join(' · '),
    excerpt: current.summary,
    cover: cover.value,
    icon: 'news',
    extra: current.body_html ? copyFull(current.title, current.body_html) : undefined,
  }
})
watch([key, page], () => { if (key.value) void article.load(); else void history.load() }, { immediate: true })
function go(page: number) { void router.replace({ query: { page } }) }
function back() {
  if (window.history.state?.back) router.back()
  else void router.push('/daily')
}
useHead(() => ({ title: key.value ? item.value?.title || '日报' : '往期日报' }))
</script>
<template>
  <main v-if="!key" class="community-page">
    <div class="community-detail"><RouterLink :to="newsHomePath" class="community-back">← 返回资讯</RouterLink><h1>往期日报</h1></div>
    <RequestState :loading="history.loading.value" :error="history.error.value" :empty="!!history.data.value && !history.data.value.items.length" @retry="history.load" />
    <template v-if="history.data.value">
      <div class="daily-history"><NewsRow v-for="entry in history.data.value.items" :key="entry.key" :item="entry" /></div>
      <PagerNav :page="history.data.value.page" :page-size="history.data.value.page_size" :total="history.data.value.total" @go="go" />
    </template>
  </main>
  <DetailLayout v-else :key="key" :ready="!!item" :title="item?.title" :author="item?.source || 'AI 日报'"
    :images="cover ? [cover] : undefined" badge="AI 日报" :share="share" @back="back">
    <template #state>
      <RequestState :loading="article.loading.value" :error="article.error.value" @retry="article.load" />
    </template>
    <template v-if="item && sourceUrl" #cta>
      <a class="xhs-primary-cta" :href="sourceUrl" target="_blank" rel="noopener noreferrer">
        <span class="xhs-primary-cta-icon"><AppIcon name="external" :size="15" /></span>
        <span class="xhs-primary-cta-label">阅读来源原文</span>
      </a>
    </template>
    <template v-if="item" #default>
      <span v-if="item.sample" class="community-sample daily-sample">示例数据</span>
      <p v-if="item.summary" class="daily-summary">{{ item.summary }}</p>
      <div v-if="item.body_html" class="xhs-prose prose daily-prose" v-html="item.body_html" />
      <p v-else class="daily-empty">这篇内容暂未整理站内正文。</p>
      <div class="xhs-meta-row">
        <span>{{ item.source }} · {{ displayDate(item.source_published_at || item.published_at) }}<template v-if="item.time_basis === 'collected'">（采集时间）</template></span>
      </div>
    </template>
    <template v-if="item" #comments>
      <NewsComments :key="key" :article-key="key" detail />
    </template>
  </DetailLayout>
</template>

<style scoped>
.daily-history { display: grid; gap: 8px; margin-top: 12px; }
.daily-sample { margin-bottom: 10px; }
.daily-summary { margin: 0 0 16px; padding: 12px 14px; border-radius: 12px; background: var(--bg-subtle); color: var(--ink-2); font-size: 14.5px; line-height: 1.65; white-space: pre-line; }
.daily-prose :deep(img) { max-width: 100%; height: auto; border-radius: 8px; }
.daily-prose :deep(pre), .daily-prose :deep(table) { max-width: 100%; overflow: auto; }
.daily-empty { margin: 0; color: var(--ink-3); font-size: 14px; }
</style>
