<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { discoverApi } from '@/api/discover'
import AppIcon from '@/components/AppIcon.vue'
import CommentSection from '@/components/comments/CommentSection.vue'
import type { CommentAdapter } from '@/components/comments/types'
import { webLink } from '@/components/community/contract'
import IdeaWorkGallery from '@/components/community/IdeaWorkGallery.vue'
import { useCommunityResource } from '@/components/community/resource'
import RequestState from '@/components/community/RequestState.vue'
import { useHead } from '@/lib/head'
import { loadFeaturedIdeas } from '@/lib/ideaCatalog'
import { ideaPlayerId, ideaRunLocation } from '@/lib/ideaPlayers'
import { galleryTab } from '@/lib/microGallery'
import '@/components/community/community.css'

const route = useRoute()
const { data, loading, error, load } = useCommunityResource(loadFeaturedIdeas)
void load()
const work = computed(() => data.value?.find(item => item.id === route.params.slug && (item.kind === 'work' || item.kind === 'exhibit')))
const images = computed(() => {
  const item = work.value
  if (!item) return []
  return [...new Set([item.cover, ...(item.images ?? [])].map(webLink).filter((url): url is string => Boolean(url)))]
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
useHead(() => ({ title: work.value?.title ?? '创意作品' }))
</script>

<template>
  <section class="community-page idea-work">
    <RouterLink :to="{ name: 'home' }" class="back"><AppIcon name="back" :size="16" /> 创意</RouterLink>
    <RequestState v-if="loading || error" :loading="loading" :error="error" @retry="load" />
    <template v-else-if="work">
      <article class="work" :class="{ 'has-cover': images.length }">
        <IdeaWorkGallery v-if="images.length" :key="work.id" :images="images" :title="work.title" />
        <div class="copy">
          <p v-if="work.label || work.kind === 'exhibit'" class="work-type">{{ work.kind === 'exhibit' ? '3D 模型' : work.label }}</p>
          <h1>{{ work.title }}</h1>
          <p class="introduction">{{ work.introduction || work.description }}</p>
          <dl class="credits">
            <div><dt>作者</dt><dd>{{ work.author }}</dd></div>
            <div v-if="work.kind === 'exhibit'">
              <dt>来源</dt><dd><RouterLink :to="{ name: 'home', query: { tab: galleryTab.key } }">微观展厅</RouterLink></dd>
            </div>
            <div v-else-if="site && run"><dt>来源</dt><dd><a :href="site">作品官网</a></dd></div>
            <div v-if="repository"><dt>源码</dt><dd><a :href="repository">查看源码</a></dd></div>
            <div v-if="desktopDownload"><dt>桌面</dt><dd><a :href="desktopDownload">下载 V2 作品包</a></dd></div>
          </dl>
          <div v-if="run || site" class="launch">
            <RouterLink v-if="run" class="launch-link" :to="run">
              {{ work.kind === 'exhibit' ? '体验 3D 模型' : '立即试玩' }} <AppIcon name="arrow" :size="16" />
            </RouterLink>
            <a v-else :href="site" class="launch-link">访问官网 <AppIcon name="arrow" :size="16" /></a>
            <p v-if="run && work.controls" class="input-hint">
              <template v-if="work.controls.desktop && work.controls.mobile">
                <span class="desktop-hint">{{ work.controls.desktop }}</span>
                <span class="mobile-hint">{{ work.controls.mobile }}</span>
              </template>
              <template v-else>{{ work.controls.desktop || work.controls.mobile }}</template>
            </p>
          </div>
        </div>
      </article>
      <CommentSection v-if="comments" :scope="'idea:' + work.id" :adapter="comments" />
    </template>
    <p v-else class="missing" role="status">找不到这个作品。</p>
  </section>
</template>

<style scoped>
.idea-work { max-width: none; --comments-bleed: 0px; }
.back { display: inline-flex; align-items: center; gap: 8px; min-height: 48px; padding-inline: 16px; color: var(--ink-2); font-size: 15px; font-weight: 600; }
.work { background: var(--surface); }
.copy { padding: 16px; }
.work-type { margin: 0 0 4px; color: var(--ink-3); font-size: 13px; line-height: 18px; }
h1 { margin: 0; font-size: 22px; line-height: 30px; font-weight: 700; overflow-wrap: anywhere; }
.introduction { margin: 8px 0 0; color: var(--ink-2); font-size: 15px; line-height: 22px; }
.credits { display: grid; gap: 10px; margin: 16px 0 0; padding-top: 12px; border-top: 1px solid var(--line); }
.credits div { display: flex; flex-wrap: wrap; align-items: baseline; gap: 8px 16px; font-size: 13px; line-height: 18px; }
.credits dt { color: var(--ink-3); }
.credits dd { margin: 0; overflow-wrap: anywhere; }
.credits a { text-decoration: underline; text-underline-offset: 3px; }
.launch { margin-top: 16px; }
.launch-link { display: inline-flex; align-items: center; justify-content: center; gap: 8px; width: 100%; min-height: 48px; padding: 10px 24px; border-radius: 24px; background: var(--surface-2); color: var(--ink); font-size: 15px; font-weight: 650; }
.launch-link:hover { background: color-mix(in srgb, var(--ink) 10%, var(--surface)); }
.input-hint { margin: 8px 0 0; color: var(--ink-3); font-size: 12px; line-height: 18px; text-align: center; }
.desktop-hint { display: none; }
.missing { padding: 0 16px; color: var(--ink-3); }
@container page (min-width: 690px) {
  .work.has-cover { display: grid; grid-template-columns: minmax(0, 1.18fr) minmax(0, 1fr); align-items: center; }
  .copy { padding: 32px clamp(24px, 4cqi, 46px); }
  .work-type { margin-bottom: 8px; }
  h1 { font-size: clamp(26px, 3.35cqi, 38px); line-height: 1.23; }
  .introduction { margin-top: 16px; font-size: 16px; line-height: 1.7; }
  .credits { margin-top: 24px; padding-top: 18px; }
  .launch { margin-top: 24px; }
  .launch-link { width: auto; }
  .input-hint { text-align: start; }
}
@media (min-width: 768px) and (pointer: fine) {
  .desktop-hint { display: inline; }
  .mobile-hint { display: none; }
}
</style>
