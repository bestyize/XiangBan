<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import ForumShell from '@/components/ForumShell.vue'
import PosterDialog from '@/components/PosterDialog.vue'
import { ApiError } from '@/api/forum'
import ContentSocialActions from '@/components/ContentSocialActions.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import ClientCreationPrompt from '@/components/creation/ClientCreationPrompt.vue'
import WorkDiscussion from '@/components/creation/WorkDiscussion.vue'
import { webLink } from '@/components/community/contract'
import { contentApi, gameSandboxUrl, type PackDetail } from '@/api/content'
import { appRunLocation, canTrialPack } from '@/lib/appRuntime'
import { applyLink, fmtSize, kindLabel, trustLabel } from '@/lib/content'
import { formatDateTime, tagLabel, timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { track } from '@/lib/analytics'
import { useSession } from '@/stores/session'
import { usePackPublisher } from '@/stores/packPublisher'
import { useToast } from '@/stores/toast'

const route = useRoute()
const toast = useToast()
const session = useSession()
const publisher = usePackPublisher()
const pack = ref<PackDetail | null>(null)
const missing = ref(false)
const loadError = ref('')
const reload = ref(0)
const busy = ref(false)
const posterOpen = ref(false)
const creationOpen = ref(false)
const versionsOpen = ref(false)
const actionsMenu = useTemplateRef<HTMLDetailsElement>('actionsMenu')
const actionsTrigger = useTemplateRef<HTMLElement>('actionsTrigger')
const versionsPanel = useTemplateRef<HTMLElement>('versionsPanel')
let loadSequence = 0

useHead(() => ({ title: pack.value?.title ?? '内容详情' }))
watch(() => publisher.saved, saved => { if (saved?.id === Number(route.params.id)) reload.value++ })

watch(
  () => [route.params.id, reload.value] as const,
  async ([id]) => {
    const current = ++loadSequence
    pack.value = null
    missing.value = false
    loadError.value = ''
    versionsOpen.value = false
    try {
      const result = await contentApi.pack(Number(id), true)
      if (current !== loadSequence) return
      pack.value = result
      track('view_pack', result.kind)
    } catch (e) {
      if (current !== loadSequence) return
      if (e instanceof ApiError && e.status === 404) missing.value = true
      else loadError.value = e instanceof Error ? e.message : '加载失败'
    }
  },
  { immediate: true },
)

const isAuthor = computed(() => !!pack.value?.author && pack.value.author.uid === session.user?.uid)
const latest = computed(() => pack.value?.versions[0])
const trialVersion = computed(() => {
  const work = pack.value, selected = latest.value
  if (work && selected && canTrialPack(work.kind, work.status, work.trust_level, selected.manifest.runtimeProtocol)) {
    return selected
  }
})
const runLocation = computed(() => {
  const work = pack.value, selected = trialVersion.value
  if (!work || !selected) return null
  return selected.manifest.runtimeProtocol === 2 || gameSandboxUrl ? appRunLocation(work.id, selected.version) : null
})
const manualOnly = computed(() => latest.value?.manifest.manualOnly === true)
const desktopResource = computed(() => pack.value?.desktop_resource === true)
const clientLink = computed(() => {
  const work = pack.value, version = latest.value
  return work && version && !manualOnly.value && !desktopResource.value ? applyLink(work, version.version, version.sha256) : ''
})
const usageNote = computed(() => {
  const work = pack.value
  if (!work) return ''
  if (desktopResource.value) return '下载后在小伴桌面的「应用 → 导入作品」中打开。'
  if (manualOnly.value) return '下载并解压后，按包内的 README 使用。'
  if (!runLocation.value && trialVersion.value) return '请在小伴客户端打开此作品。'
  if (work.kind === 'game' && work.trust_level > 1 && latest.value?.manifest.runtimeProtocol !== 2) return '此作品暂不支持网页试玩。'
  if (!work.one_click && work.kind !== 'game' && work.kind !== 'app') return '在小伴桌面端确认内容和所需权限后再安装。'
  return ''
})
const postImage = computed(() => webLink(pack.value?.cover_url))
const attachmentIcon = computed(() => {
  switch (pack.value?.kind) {
    case 'game': return 'keyboard'
    case 'wallpaper': return 'image'
    case 'pet': return 'smile'
    case 'effect': return 'ai'
    default: return 'grid'
  }
})

function closeActions(restoreFocus = false) {
  if (!actionsMenu.value?.open) return
  actionsMenu.value.open = false
  if (restoreFocus) actionsTrigger.value?.focus()
}

function dismissActions(event: Event) {
  const target = event instanceof FocusEvent ? event.relatedTarget : event.target
  if (target instanceof Node && !actionsMenu.value?.contains(target)) closeActions()
}

async function toggleVersions() {
  versionsOpen.value = !versionsOpen.value
  closeActions(!versionsOpen.value)
  if (!versionsOpen.value) return
  await nextTick()
  versionsPanel.value?.focus()
}

onMounted(() => document.addEventListener('pointerdown', dismissActions))
onUnmounted(() => document.removeEventListener('pointerdown', dismissActions))

async function download() {
  if (!pack.value || busy.value) return
  track('download_click', pack.value.kind)
  busy.value = true
  try {
    const info = await contentApi.download(pack.value.id)
    window.location.href = info.url
  } catch (e) {
    toast.error(e)
  } finally {
    busy.value = false
  }
}

const posterInput = computed(() =>
  pack.value
    ? { tag: kindLabel(pack.value.kind), title: pack.value.title, summary: pack.value.summary || (desktopResource.value ? '下载后导入小伴桌面' : '在小伴里一键应用'), cover: pack.value.cover_url, url: `${window.location.origin}/packs/${pack.value.id}`, seed: pack.value.id }
    : null,
)

async function copyLink() {
  track('share_copy_link', 'pack')
  try {
    await navigator.clipboard.writeText(window.location.href)
    toast.show('链接已复制')
  } catch {
    toast.show('复制失败，请手动复制地址栏')
  }
}
</script>

<template>
  <ForumShell class="topic-shell" :rail="false">
    <p v-if="missing" class="state">这个内容不存在或已下架。</p>
    <p v-else-if="loadError" class="state" role="alert">{{ loadError }} <button class="btn" type="button" @click="reload++">重试</button></p>
    <p v-else-if="!pack" class="state" role="status">正在读取作品…</p>

    <article v-else :key="pack.id" class="topic" aria-labelledby="pack-title">
      <h1 id="pack-title" class="title">{{ pack.title }}</h1>

      <ContentSocialActions v-slot="social" kind="pack" :id="pack.id" :enabled="pack.status === 'published'">
        <div class="byline">
          <RouterLink v-if="pack.author" :to="`/u/${pack.author.uid}`" class="who">
            <UserAvatar :name="pack.author.nickname || '社区用户'" :size="28" />
            <span class="name">{{ pack.author.nickname || '社区用户' }}</span>
          </RouterLink>
          <span v-else class="who">
            <UserAvatar name="社区作品" :size="28" />
            <span class="name">社区作品</span>
          </span>
          <time :datetime="pack.created_at" :title="formatDateTime(pack.created_at)">{{ timeAgo(pack.created_at) }}发布</time>
          <span v-if="pack.kind !== 'game' && pack.kind !== 'app'">{{ trustLabel(pack.trust_level) }}</span>

          <div class="post-controls">
            <button v-if="pack.status === 'published'" class="icon-action" type="button"
              :aria-label="social.bookmarked ? '取消收藏' : '收藏'" :aria-pressed="social.bookmarked"
              :title="social.bookmarked ? '取消收藏' : '收藏'" :disabled="social.bookmarkDisabled" @click="social.bookmark">
              <AppIcon name="bookmark" :size="19" />
            </button>
            <details ref="actionsMenu" class="more-actions" @focusout="dismissActions" @keydown.esc.stop.prevent="closeActions(true)">
              <summary ref="actionsTrigger" class="icon-action" aria-label="更多作品操作" title="更多作品操作"><AppIcon name="more" :size="22" /></summary>
              <div class="actions-panel" @click="closeActions(true)">
                <a v-if="runLocation && clientLink" class="action-item desktop-only" :href="clientLink" @click="track('apply_click', pack.kind)">在小伴中打开</a>
                <button v-if="runLocation || clientLink" class="action-item" :class="{ 'desktop-only': !runLocation }" type="button" :disabled="busy" @click="download">下载压缩包</button>
                <button v-if="latest" class="action-item" type="button" :aria-expanded="versionsOpen" aria-controls="pack-versions" @click.stop="toggleVersions">
                  {{ versionsOpen ? '收起版本记录' : '版本记录' }}
                </button>
                <button class="action-item" type="button" @click="copyLink">复制链接</button>
                <button class="action-item" type="button" @click="posterOpen = true">生成海报</button>
                <button class="action-item" type="button" @click="creationOpen = true">在客户端创作或修改</button>
                <RouterLink v-if="pack.topic_id" class="action-item" :to="`/t/${pack.topic_id}`">查看关联帖子</RouterLink>
                <button v-if="isAuthor" class="action-item" type="button" aria-haspopup="dialog" @click="publisher.open(pack.id)">发布新版本</button>
                <template v-if="pack.status === 'published'">
                  <RouterLink class="action-item" to="/me/content-bookmarks">我的作品收藏</RouterLink>
                  <button class="action-item report-action" type="button" aria-haspopup="dialog" :disabled="social.busy" @click="social.report">举报作品</button>
                </template>
              </div>
            </details>
          </div>
        </div>
        <p v-if="social.message" class="post-feedback" role="status">{{ social.message }}</p>
        <p v-if="social.error" class="post-feedback work-error" role="alert">{{ social.error }}
          <button class="retry-link" type="button" :disabled="social.busy" @click="social.reload">重试</button>
        </p>
      </ContentSocialActions>

      <div v-if="pack.summary || postImage" class="prose post-body">
        <p v-if="pack.summary" class="summary">{{ pack.summary }}</p>
        <a v-if="postImage" class="post-image" :href="postImage" target="_blank" rel="noopener noreferrer"
          :aria-label="`查看${pack.title}的作品原图`">
          <img :src="postImage" :alt="`${pack.title}的作品图片`" decoding="async" />
        </a>
      </div>
      <div v-if="pack.tags.length" class="tags">
        <RouterLink v-for="tag in pack.tags" :key="tag" :to="`/tag/${encodeURIComponent(tag)}`" class="tag">#{{ tagLabel(tag) }}</RouterLink>
      </div>

      <section class="work-card" :aria-label="runLocation ? '试玩卡片' : '作品附件'">
        <div class="work-card-row">
          <div class="work-card-info">
            <span class="work-icon"><AppIcon :name="attachmentIcon" :size="22" /></span>
            <div>
              <strong class="work-title">{{ pack.title }}</strong>
              <p class="work-meta">
                {{ kindLabel(pack.kind) }}
                <template v-if="latest"> · {{ latest.label }}</template>
                <template v-if="latest && !runLocation"> · {{ fmtSize(latest.size) }}</template>
                <template v-if="pack.pure_data"> · 纯数据</template>
                <template v-if="pack.apply_count"> · {{ pack.apply_count }} 次使用</template>
              </p>
            </div>
          </div>
          <RouterLink v-if="runLocation" class="btn btn-primary launch-action" :to="runLocation">
            <AppIcon name="play" :size="18" />
            {{ pack.kind === 'game' ? '试玩游戏' : '打开应用' }}
          </RouterLink>
          <template v-else>
            <a v-if="clientLink" class="btn btn-primary launch-action desktop-only" :href="clientLink" @click="track('apply_click', pack.kind)">
              <AppIcon name="monitor" :size="18" /> 在小伴中打开
            </a>
            <button class="btn btn-primary launch-action" :class="{ 'mobile-only': clientLink }" type="button" :disabled="busy" @click="download">
              <AppIcon name="download" :size="18" /> {{ busy ? '正在准备…' : '下载 ZIP' }}
            </button>
          </template>
        </div>
        <p v-if="usageNote" class="work-note">{{ usageNote }}</p>
      </section>

      <section v-if="latest" v-show="versionsOpen" id="pack-versions" ref="versionsPanel" class="versions" tabindex="-1" aria-labelledby="pack-versions-title">
        <h2 id="pack-versions-title">版本记录</h2>
        <ul>
          <li v-for="version in pack.versions" :key="version.version">
            <div class="version-line">
              <strong>{{ version.label }}</strong>
              <span>{{ fmtSize(version.size) }} · <time :datetime="version.created_at">{{ timeAgo(version.created_at) }}</time></span>
            </div>
            <p v-if="version.changelog">{{ version.changelog }}</p>
          </li>
        </ul>
      </section>

      <div v-if="pack.status === 'published'" class="work-comments">
        <WorkDiscussion :id="pack.id" />
      </div>
    </article>

    <ClientCreationPrompt v-model:show="creationOpen" :kind="pack ? kindLabel(pack.kind) : undefined" />
    <PosterDialog :open="posterOpen" :poster="posterInput" :name="pack?.title ?? ''" @close="posterOpen = false" />
  </ForumShell>
</template>

<style scoped src="../styles/forum-post.css"></style>

<style scoped>
.topic-shell { max-width: 748px; }
.topic-shell :deep(.center) { padding-block-start: 28px; }
.topic { max-width: 660px; }
.title {
  margin-bottom: 6px;
  font-size: clamp(24px, 4vw, 34px);
  line-height: 1.25;
  letter-spacing: -0.02em;
  text-wrap: balance;
}
.byline {
  margin-bottom: 18px;
  color: var(--ink-2);
}
.state {
  padding: 48px 12px;
  text-align: center;
  color: var(--ink-2);
}
.who { min-width: 0; }
.who > .name { overflow-wrap: anywhere; }
.post-controls {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  margin-inline-start: auto;
}
.icon-action {
  display: grid;
  place-items: center;
  inline-size: 42px;
  block-size: 42px;
  padding: 0;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink-2);
  cursor: pointer;
}
.icon-action:hover,
.more-actions[open] > summary { background: var(--surface-2); }
.icon-action:disabled { opacity: 0.55; cursor: not-allowed; }
.icon-action[aria-pressed="true"] { color: var(--brand); }
.icon-action[aria-pressed="true"] :deep(svg) { fill: currentColor; }
.more-actions { position: relative; }
summary { list-style: none; }
summary::-webkit-details-marker { display: none; }
.actions-panel {
  position: absolute;
  z-index: 4;
  inset: calc(100% + 4px) 0 auto auto;
  inline-size: 216px;
  max-height: min(410px, 55dvh);
  overflow-y: auto;
  padding: 6px;
  border-radius: 12px;
  background: var(--surface);
  box-shadow: 0 10px 30px rgb(24 20 22 / 24%);
}
.action-item {
  display: flex;
  align-items: center;
  width: 100%;
  min-height: 42px;
  padding: 6px 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  text-align: start;
  font-size: var(--fs-sm);
}
.action-item:hover { background: var(--surface-2); }
.action-item:disabled { opacity: 0.58; cursor: not-allowed; }
.report-action,
.work-error { color: var(--danger); }
.post-feedback {
  margin: -8px 0 14px;
  font-size: var(--fs-sm);
  overflow-wrap: anywhere;
}
.retry-link {
  padding: 0;
  border: 0;
  background: none;
  text-decoration: underline;
  text-underline-offset: 0.16em;
}
.summary {
  max-width: 67ch;
  color: var(--ink-2);
  line-height: 1.75;
  text-wrap: pretty;
  white-space: pre-line;
}
.post-image {
  display: block;
  border-radius: 10px;
  cursor: zoom-in;
}
.post-image img {
  display: block;
  width: 100%;
  height: auto;
  max-height: min(540px, 66dvh);
  object-fit: contain;
  border-radius: inherit;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
  margin-top: 10px;
}
.tag {
  color: var(--ink-2);
  font-size: var(--fs-xs);
  overflow-wrap: anywhere;
}
.tag:hover { color: var(--ink); text-decoration: underline; text-underline-offset: 3px; }
.work-card {
  margin-top: 18px;
  padding: 10px 12px;
  border: 1px solid var(--line);
  border-radius: 12px;
}
.work-card-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
}
.work-card-info {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.work-card-info > div { min-width: 0; }
.work-icon {
  display: grid;
  place-items: center;
  flex: none;
  inline-size: 40px;
  block-size: 40px;
  border-radius: 9px;
  background: var(--brand-soft);
  color: var(--brand);
}
.work-title {
  display: block;
  font-size: 15px;
  font-weight: 650;
}
.work-title,
.work-meta { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.work-meta,
.work-note {
  margin: 2px 0 0;
  color: var(--ink-2);
  font-size: var(--fs-xs);
}
.work-meta { font-variant-numeric: tabular-nums; }
.work-note { margin-top: 10px; }
.launch-action {
  min-height: 44px;
  gap: 6px;
  padding-inline: 13px;
  border-color: var(--ink);
  border-radius: 7px;
  background: var(--ink);
  color: var(--surface);
}
.launch-action:not(:disabled):hover {
  border-color: color-mix(in srgb, var(--ink) 84%, var(--surface));
  background: color-mix(in srgb, var(--ink) 84%, var(--surface));
}
.versions {
  margin-top: 16px;
  scroll-margin-top: calc(var(--header-h) + 12px);
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.versions h2 { margin: 0 0 8px; color: var(--ink); font-size: var(--fs-sm); }
.versions ul {
  margin: 0;
  padding: 0;
  list-style: none;
}
.versions li {
  padding-block: 8px;
  border-top: 1px solid var(--line);
  overflow-wrap: anywhere;
}
.version-line {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 10px;
  font-variant-numeric: tabular-nums;
}
.versions li p {
  margin: 4px 0 0;
  white-space: pre-line;
}
.work-comments { margin-top: 24px; }
.mobile-only { display: none; }
@media (max-width: 767px) {
  .topic-shell :deep(.center) { margin-inline: -12px; padding-block-start: 20px; }
  .icon-action { inline-size: 44px; block-size: 44px; }
  .launch-action { padding-inline: 10px; }
  .desktop-only { display: none; }
  .mobile-only { display: inline-flex; }
}
</style>
