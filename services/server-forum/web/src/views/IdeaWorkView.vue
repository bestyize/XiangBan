<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import AppIcon from '@/components/AppIcon.vue'
import CommentSection from '@/components/comments/CommentSection.vue'
import type { CommentAdapter } from '@/components/comments/types'
import { webLink } from '@/components/community/contract'
import IdeaWorkGallery from '@/components/community/IdeaWorkGallery.vue'
import { useCommunityResource } from '@/components/community/resource'
import RequestState from '@/components/community/RequestState.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { useHead } from '@/lib/head'
import { ideaCoverFallback, loadFeaturedIdeas } from '@/lib/ideaCatalog'
import { ideaPlayerId, ideaRunLocation } from '@/lib/ideaPlayers'
import { galleryTab } from '@/lib/microGallery'
import { useToast } from '@/stores/toast'
import '@/components/community/community.css'

const route = useRoute(), router = useRouter(), toast = useToast()
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
function backToList() {
  if (window.history.state?.back) router.back()
  else void router.push({ name: 'home', query: { tab: work.value?.kind === 'exhibit' ? galleryTab.key : undefined } })
}
async function shareWork() {
  try {
    if (navigator.share) await navigator.share({ title: work.value?.title, url: location.href })
    else {
      await navigator.clipboard.writeText(location.href)
      toast.show('链接已复制')
    }
  } catch (error) {
    if (!(error instanceof Error && error.name === 'AbortError')) toast.show('暂时无法分享，请复制地址栏中的链接')
  }
}
useHead(() => ({ title: work.value?.title ?? '创意作品' }))
</script>

<template>
  <section class="community-page idea-work">
    <header class="work-header">
      <button type="button" aria-label="返回上一页" @click="backToList"><AppIcon name="back" :size="18" /></button>
      <strong>{{ work?.title ?? '创意作品' }}</strong>
      <button type="button" aria-label="分享作品" :disabled="!work" @click="shareWork"><AppIcon name="share" :size="19" /></button>
    </header>
    <div class="work-content">
      <button class="work-back" type="button" @click="backToList"><AppIcon name="back" :size="15" /> 返回列表</button>
      <RequestState v-if="loading || error" :loading="loading" :error="error" @retry="load" />
      <template v-else-if="work">
        <article class="work">
          <IdeaWorkGallery v-if="images.length" :key="work.id" :images="images" :title="work.title" :fallback-cover="ideaCoverFallback(work.id)" />
          <span v-if="work.label || work.kind === 'exhibit'" class="work-type">{{ work.kind === 'exhibit' ? '3D 模型' : work.label }}</span>
          <h1>{{ work.title }}</h1>
          <div class="work-byline">
            <UserAvatar :name="work.author.replace(/^@/, '')" :size="24" />
            <strong>{{ work.author }}</strong>
            <time :datetime="work.date">{{ work.date.slice(0, 10) }}</time>
          </div>
          <p class="introduction">{{ work.introduction || work.description }}</p>
          <dl v-if="work.kind === 'exhibit' || (site && run) || repository || desktopDownload" class="credits">
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
        </article>
        <div v-if="comments" class="work-comments"><CommentSection :scope="'idea:' + work.id" :adapter="comments" /></div>
      </template>
      <p v-else class="missing" role="status">找不到这个作品。</p>
    </div>
  </section>
</template>

<style scoped>
.idea-work { max-width: none; --comments-bleed: 0px; }
.work-header { position: sticky; z-index: 30; top: 0; display: grid; grid-template-columns: 44px minmax(0, 1fr) 44px; align-items: center; height: calc(48px + env(safe-area-inset-top)); padding: env(safe-area-inset-top) 6px 0; border-bottom: 1px solid var(--line); background: var(--mobile-nav-surface); }
.work-header strong { overflow: hidden; font-size: 15px; text-align: center; text-overflow: ellipsis; white-space: nowrap; }
.work-header button { display: grid; place-items: center; min-height: 44px; padding: 0; border: 0; background: none; }
.work-header button:disabled { opacity: .45; }
.work-content { max-width: 876px; margin-inline: auto; padding: 8px 6px 24px; }
.work-back { display: none; }
.work, .work-comments { padding: 14px; border-radius: 16px; background: var(--surface); box-shadow: var(--shadow-card); }
.work :deep(.work-gallery) { overflow: hidden; margin-bottom: 16px; border-radius: 13px; }
.work :deep(.work-gallery .cover), .work :deep(.work-gallery .slide:first-child img) { aspect-ratio: 16 / 9; object-fit: contain; }
.work-type { display: inline-flex; margin-bottom: 8px; padding: 2px 9px; border-radius: 999px; background: var(--brand-soft); color: var(--brand); font-size: 12px; font-weight: 650; }
h1 { margin: 0; font-size: 20px; line-height: 1.35; font-weight: 700; overflow-wrap: anywhere; }
.work-byline { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 8px; margin-top: 10px; color: var(--ink-2); font-size: 13px; }
.work-byline time { color: var(--ink-3); }
.introduction { max-width: 70ch; margin: 14px 0 0; color: var(--ink-2); font-size: 15px; line-height: 1.8; }
.credits { display: grid; gap: 8px; margin: 16px 0 0; padding-top: 12px; border-top: 1px solid var(--line); }
.credits div { display: flex; flex-wrap: wrap; align-items: baseline; gap: 8px 16px; font-size: 13px; line-height: 18px; }
.credits dt { color: var(--ink-3); }
.credits dd { margin: 0; overflow-wrap: anywhere; }
.credits a { text-decoration: underline; text-underline-offset: 3px; }
.launch { margin-top: 16px; }
.launch-link { display: inline-flex; align-items: center; justify-content: center; gap: 8px; min-height: 44px; padding: 8px 17px; border-radius: 999px; background: var(--brand-fill); color: var(--brand-ink); font-size: 14px; font-weight: 650; }
.launch-link:hover { filter: brightness(.96); }
.input-hint { margin: 10px 0 0; padding: 8px 11px; border-radius: 8px; background: var(--surface-2); color: var(--ink-2); font-size: 12px; line-height: 18px; }
.desktop-hint { display: none; }
.work-comments { margin-top: 12px; }
.missing { padding: 0 16px; color: var(--ink-3); }
@media (min-width: 1024px) {
  .work-header { display: none; }
  .work-content { padding: 22px 28px 40px; }
  .work-back { display: inline-flex; align-items: center; gap: 5px; min-height: 34px; margin-bottom: 16px; padding: 0 12px; border: 1px solid var(--line); border-radius: 999px; background: var(--surface); color: var(--ink-2); font-size: 13px; }
  .work-back:hover { color: var(--brand); }
  .work, .work-comments { padding: 22px 26px; }
  h1 { font-size: 24px; }
}
@media (min-width: 768px) and (pointer: fine) {
  .desktop-hint { display: inline; }
  .mobile-hint { display: none; }
}
</style>
