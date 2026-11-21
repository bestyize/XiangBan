<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import AppIcon from '@/components/AppIcon.vue'
import CommentSection from '@/components/comments/CommentSection.vue'
import type { CommentAdapter } from '@/components/comments/types'
import { webLink } from '@/components/community/contract'
import DetailLayout from '@/components/community/DetailLayout.vue'
import { useCommunityResource } from '@/components/community/resource'
import RequestState from '@/components/community/RequestState.vue'
import { useHead } from '@/lib/head'
import { ideaCoverFallback, loadFeaturedIdeas } from '@/lib/ideaCatalog'
import { ideaPlayerId, ideaRunLocation } from '@/lib/ideaPlayers'
import { galleryTab } from '@/lib/microGallery'
import { MOBILE_SITE_QUERY, useNarrow } from '@/lib/viewport'

const route = useRoute(), router = useRouter()
const mobile = useNarrow(MOBILE_SITE_QUERY)
const { data, loading, error, load } = useCommunityResource(loadFeaturedIdeas)
void load()
const work = computed(() => data.value?.find(item => item.id === route.params.slug && (item.kind === 'work' || item.kind === 'exhibit')))
const images = computed(() => {
  const item = work.value
  if (!item) return []
  return [...new Set([item.cover, ...(item.images ?? [])].map(webLink).filter((url): url is string => Boolean(url)))]
})
const badge = computed(() => work.value?.kind === 'exhibit' ? '3D 模型' : work.value?.label)
const tags = computed(() => [...new Set([badge.value, '小伴创意', '交互作品'].filter((tag): tag is string => !!tag))])
const controls = computed(() => {
  const item = work.value?.controls
  return mobile.value ? item?.mobile || item?.desktop : item?.desktop || item?.mobile
})
const site = computed(() => webLink(work.value?.site))
const repository = computed(() => webLink(work.value?.repository))
const desktopDownload = computed(() => webLink(work.value?.desktop_download))
const run = computed(() => {
  const id = ideaPlayerId(work.value?.player)
  return id ? ideaRunLocation(id) : undefined
})
const comments = computed<CommentAdapter | undefined>(() => {
  const id = work.value?.id
  if (!id) return undefined
  return {
    list: page => discoverApi.ideaComments(id, page),
    post: (body, parent, images) => discoverApi.commentIdea(id, body, parent, images),
    like: (comment, on) => discoverApi.likeIdeaComment(id, comment, on),
    remove: comment => discoverApi.deleteIdeaComment(id, comment),
  }
})
function backToList() {
  if (window.history.state?.back) router.back()
  else void router.push({ name: 'home', query: { tab: work.value?.kind === 'exhibit' ? galleryTab.key : undefined } })
}
useHead(() => ({ title: work.value?.title ?? '创意作品' }))
</script>

<template>
  <DetailLayout :key="String(route.params.slug)" :ready="!!work" :title="work?.title" :author="work?.author"
    :images="images" :badge="badge" :fallback-cover="work ? ideaCoverFallback(work.id) : undefined"
    shareable @back="backToList">
    <template #state>
      <RequestState v-if="loading || error" :loading="loading" :error="error" @retry="load" />
      <p v-else class="missing" role="status">找不到这个作品。</p>
    </template>
    <template v-if="work" #default>
      <div class="xhs-prose work-introduction">{{ work.introduction || work.description }}</div>
      <div class="xhs-hashtags"><span v-for="tag in tags" :key="tag">#{{ tag }}</span></div>
      <RouterLink v-if="run" class="xhs-info-strip" :to="run">
        <span class="xhs-strip-tag"><AppIcon name="ai" :size="12" /> 体验</span>
        <span class="xhs-strip-text">{{ work.kind === 'exhibit' ? '体验 3D 模型' : '打开' + work.title }}</span>
        <AppIcon name="arrow" :size="13" />
      </RouterLink>
      <a v-else-if="site" class="xhs-info-strip" :href="site">
        <span class="xhs-strip-tag"><AppIcon name="ai" :size="12" /> 体验</span>
        <span class="xhs-strip-text">访问作品官网</span><AppIcon name="arrow" :size="13" />
      </a>
      <details v-if="work.kind === 'exhibit' || (site && run) || repository || desktopDownload" class="work-info">
        <summary class="xhs-info-strip">
          <span class="xhs-strip-tag is-warm">说明</span>
          <span class="xhs-strip-text">{{ controls || '来源与下载' }}</span>
          <AppIcon name="arrow" :size="13" />
        </summary>
        <dl class="credits">
          <div v-if="work.kind === 'exhibit'">
            <dt>来源</dt><dd><RouterLink :to="{ name: 'home', query: { tab: galleryTab.key } }">微观展厅</RouterLink></dd>
          </div>
          <div v-else-if="site && run"><dt>来源</dt><dd><a :href="site">作品官网</a></dd></div>
          <div v-if="repository"><dt>源码</dt><dd><a :href="repository">查看源码</a></dd></div>
          <div v-if="desktopDownload"><dt>桌面</dt><dd><a :href="desktopDownload">下载 V2 作品包</a></dd></div>
        </dl>
      </details>
      <div v-else-if="controls" class="xhs-info-strip is-static">
        <span class="xhs-strip-tag is-warm">说明</span><span class="xhs-strip-text">{{ controls }}</span>
      </div>
      <div class="xhs-meta-row"><time :datetime="work.date">收录于 {{ work.date.slice(0, 10) }} 社区</time></div>
    </template>
    <template v-if="work && comments" #comments>
      <CommentSection detail :scope="'idea:' + work.id" :adapter="comments" />
    </template>
  </DetailLayout>
</template>

<style scoped>
.work-introduction { white-space: pre-wrap; }
.work-info summary::-webkit-details-marker { display: none; }
.work-info[open] summary > :last-child { transform: rotate(90deg); }
.credits { display: grid; gap: 8px; margin: 10px 0 0; padding: 0 11px; }
.credits div { display: flex; align-items: baseline; gap: 8px 16px; font-size: 13px; }
.credits dt, .missing { color: var(--ink-3); }
.credits dd { margin: 0; overflow-wrap: anywhere; }
.credits a { text-decoration: underline; text-underline-offset: 3px; }
.missing { padding: 0 16px; }
</style>
