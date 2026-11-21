<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import DetailLayout from '@/components/community/DetailLayout.vue'
import PosterDialog from '@/components/PosterDialog.vue'
import { ApiError } from '@/api/forum'
import ContentSocialActions from '@/components/ContentSocialActions.vue'
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

const route = useRoute(), router = useRouter()
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
function back() {
  if (window.history.state?.back) router.back()
  else void router.push('/create')
}

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
  <DetailLayout :key="String(route.params.id)" :ready="!!pack" :title="pack?.title"
    :author="pack?.author?.nickname || '社区作品'"
    :author-to="pack?.author ? '/u/' + pack.author.uid : undefined"
    :images="postImage ? [postImage] : undefined" :badge="pack ? kindLabel(pack.kind) : undefined"
    shareable @back="back">
    <template #state>
      <p v-if="missing" class="state">这个内容不存在或已下架。</p>
      <p v-else-if="loadError" class="state" role="alert">{{ loadError }} <button class="btn" type="button" @click="reload++">重试</button></p>
      <p v-else class="state" role="status">正在读取作品…</p>
    </template>

    <template v-if="pack" #actions>
      <ContentSocialActions v-slot="social" kind="pack" :id="pack.id" :enabled="pack.status === 'published'">
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
                <a v-if="postImage" class="action-item" :href="postImage" target="_blank" rel="noopener noreferrer">查看作品原图</a>
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
        <p v-if="social.message" class="post-feedback" role="status">{{ social.message }}</p>
        <p v-if="social.error" class="post-feedback work-error" role="alert">{{ social.error }}
          <button class="retry-link" type="button" :disabled="social.busy" @click="social.reload">重试</button>
        </p>
      </ContentSocialActions>
    </template>

    <template v-if="pack" #default>
      <div v-if="pack.summary" class="xhs-prose summary">{{ pack.summary }}</div>
      <div v-if="pack.tags.length" class="xhs-hashtags">
        <RouterLink v-for="tag in pack.tags" :key="tag" :to="`/tag/${encodeURIComponent(tag)}`" class="tag">#{{ tagLabel(tag) }}</RouterLink>
      </div>
      <RouterLink v-if="runLocation" class="xhs-info-strip" :to="runLocation">
        <span class="xhs-strip-tag"><AppIcon name="play" :size="12" /> 体验</span>
        <span class="xhs-strip-text">{{ pack.kind === 'game' ? '试玩游戏' : '打开应用' }}</span><AppIcon name="arrow" :size="13" />
      </RouterLink>
      <template v-else>
        <a v-if="clientLink" class="xhs-info-strip desktop-only" :href="clientLink" @click="track('apply_click', pack.kind)">
          <span class="xhs-strip-tag"><AppIcon name="monitor" :size="12" /> 打开</span>
          <span class="xhs-strip-text">在小伴中打开</span><AppIcon name="arrow" :size="13" />
        </a>
        <button class="xhs-info-strip" :class="{ 'mobile-only': clientLink }" type="button" :disabled="busy" @click="download">
          <span class="xhs-strip-tag"><AppIcon name="download" :size="12" /> 下载</span>
          <span class="xhs-strip-text">{{ busy ? '正在准备…' : '下载 ZIP' }}</span><AppIcon name="arrow" :size="13" />
        </button>
      </template>
      <div v-if="usageNote" class="xhs-info-strip is-static pack-note">
        <span class="xhs-strip-tag is-warm">说明</span><span class="xhs-strip-text">{{ usageNote }}</span>
      </div>
      <div class="xhs-meta-row pack-meta">
        <time :datetime="pack.created_at" :title="formatDateTime(pack.created_at)">{{ timeAgo(pack.created_at) }}发布</time>
        <span>{{ kindLabel(pack.kind) }}</span>
        <span v-if="latest">{{ latest.label }}<template v-if="!runLocation"> · {{ fmtSize(latest.size) }}</template></span>
        <span v-if="pack.kind !== 'game' && pack.kind !== 'app'">{{ trustLabel(pack.trust_level) }}</span>
        <span v-if="pack.pure_data">纯数据</span>
        <span v-if="pack.apply_count">{{ pack.apply_count }} 次使用</span>
      </div>
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

    </template>
    <template v-if="pack?.status === 'published'" #comments>
      <WorkDiscussion :id="pack.id" :linked-topic-id="pack.topic_id" />
    </template>
    <template #dialogs>
      <ClientCreationPrompt v-model:show="creationOpen" :kind="pack ? kindLabel(pack.kind) : undefined" />
      <PosterDialog :open="posterOpen" :poster="posterInput" :name="pack?.title ?? ''" @close="posterOpen = false" />
    </template>
  </DetailLayout>
</template>

<style scoped>
.state {
  padding: 48px 12px;
  text-align: center;
  color: var(--ink-2);
}
.post-controls {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  margin-inline-start: auto;
}
.icon-action {
  display: grid;
  place-items: center;
  inline-size: 34px;
  block-size: 34px;
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
  margin: 0;
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
.summary { white-space: pre-line; }
.pack-note .xhs-strip-text { white-space: normal; }
.tag:hover { text-decoration: underline; }
.pack-meta { justify-content: flex-start; flex-wrap: wrap; }
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
.mobile-only { display: none; }
@media (max-width: 767px) {
  .desktop-only { display: none; }
  .mobile-only { display: flex; }
}
</style>
