<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import PosterDialog from '@/components/PosterDialog.vue'
import { ApiError } from '@/api/forum'
import ContentSocialActions from '@/components/ContentSocialActions.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import ClientCreationPrompt from '@/components/creation/ClientCreationPrompt.vue'
import WorkDiscussion from '@/components/creation/WorkDiscussion.vue'
import CommunityGameTrial from '@/components/creation/CommunityGameTrial.vue'
import { contentApi, type PackDetail } from '@/api/content'
import { applyLink, coverStyle, fmtSize, kindLabel, trustLabel } from '@/lib/content'
import { timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { track } from '@/lib/analytics'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

const route = useRoute()
const toast = useToast()
const session = useSession()
const pack = ref<PackDetail | null>(null)
const missing = ref(false)
const loadError = ref('')
const reload = ref(0)
const busy = ref(false)
const posterOpen = ref(false)
const creationOpen = ref(false)
let loadSequence = 0

useHead(() => ({ title: pack.value?.title ?? '内容详情' }))

watch(
  () => [route.params.id, reload.value] as const,
  async ([id]) => {
    const current = ++loadSequence
    pack.value = null
    missing.value = false
    loadError.value = ''
    try {
      const result = await contentApi.pack(Number(id))
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
const manualOnly = computed(() => latest.value?.manifest.manualOnly === true)
const open = computed(() => (pack.value && latest.value ? applyLink(pack.value, latest.value.version, latest.value.sha256) : ''))

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
    ? { tag: kindLabel(pack.value.kind), title: pack.value.title, summary: pack.value.summary || '在小伴里一键应用', cover: pack.value.cover_url, url: `${window.location.origin}/packs/${pack.value.id}`, seed: pack.value.id }
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
  <ContentShell>
    <p v-if="missing" class="state">这个内容不存在或已下架。</p>
    <p v-else-if="loadError" class="state" role="alert">{{ loadError }} <button class="btn" type="button" @click="reload++">重试</button></p>
    <p v-else-if="!pack" class="state">加载中</p>
    <article v-else class="pack">
      <div v-if="pack.cover_url" class="cover" :class="{ 'game-cover': pack.kind === 'game' || pack.kind === 'widget' }" :style="coverStyle(pack)" role="img" :aria-label="`${pack.title} 封面`" />
      <div class="head">
        <h1>{{ pack.title }}</h1>
        <p class="meta">
          <span>{{ kindLabel(pack.kind) }}</span>
          <span :class="{ warn: pack.trust_level === 2 }">{{ trustLabel(pack.trust_level) }}</span>
          <span v-if="pack.pure_data">纯数据</span>
          <span v-if="pack.apply_count">{{ pack.apply_count }} 次使用</span>
        </p>
        <div class="author-row">
          <UserAvatar :name="pack.author?.nickname || '社区作品'" :size="36" />
          <div><RouterLink v-if="pack.author" :to="`/u/${pack.author.uid}`" class="author-name">{{ pack.author.nickname || '社区用户' }}</RouterLink><span v-else class="author-name">社区作品</span><p class="author-meta"><span v-if="latest">{{ latest.label }} · {{ fmtSize(latest.size) }} · </span>{{ timeAgo(pack.updated_at) }}更新</p></div>
        </div>
        <p v-if="pack.summary" class="summary">{{ pack.summary }}</p>
        <p v-if="pack.tags.length" class="tags">
          <RouterLink v-for="t in pack.tags" :key="t" :to="`/tag/${encodeURIComponent(t)}`">{{ t }}</RouterLink>
        </p>
        <CommunityGameTrial v-if="latest" :key="`${pack.id}-${latest.version}`" :pack="pack" :version="latest.version" />
        <p v-if="pack.kind === 'game' && pack.trust_level > 1" class="notice">此作品尚未审核，暂不支持网页试玩。</p>
        <p v-if="manualOnly" class="notice">这是手动使用的模板包。下载并解压后，按 README 中的步骤使用；它不会自动创建任务或运行流程。</p>
        <p v-else-if="!pack.one_click" class="notice">
          {{ pack.trust_level === 2 ? '这是用户发布、尚未审核的内容，不支持一键应用；' : '' }}在小伴桌面端会先展示它会做什么、需要哪些权限，由你确认后才安装。
        </p>
        <ContentSocialActions v-if="pack.status === 'published'" kind="pack" :id="pack.id" />
        <WorkDiscussion v-if="pack.status === 'published'" :id="pack.id" />
        <div class="actions">
          <button class="btn" type="button" @click="creationOpen = true">在客户端创作或修改</button>
          <a v-if="!manualOnly" class="btn btn-primary open-app" :href="open" @click="track('apply_click', pack.kind)">在小伴中打开</a>
          <button class="btn" type="button" :disabled="busy" @click="download">下载压缩包</button>
          <button class="btn" type="button" @click="copyLink">复制链接</button>
          <button class="btn" type="button" @click="posterOpen = true">生成海报</button>
          <RouterLink v-if="pack.topic_id" class="btn" :to="`/t/${pack.topic_id}`">看讨论</RouterLink>
          <RouterLink v-if="isAuthor" class="btn" :to="`/publish?pack=${pack.id}`">发布新版本</RouterLink>
        </div>
        <p v-if="!manualOnly" class="hint">没有安装小伴？在桌面端“社区”页里也能找到它。</p>
        <p class="hint only-phone">{{ manualOnly ? '可以复制链接，在电脑上下载并阅读使用说明。' : pack.kind === 'game' ? '支持的游戏可在网页试玩；持续体验或创作请使用客户端。' : '桌面作品请在 Windows 或 macOS 客户端使用。' }}</p>
      </div>

      <section v-if="pack.versions.length" class="versions">
        <h2>版本记录</h2>
        <ul>
          <li v-for="v in pack.versions" :key="v.version">
            <strong>{{ v.label }}</strong>
            <span class="muted">{{ fmtSize(v.size) }} · {{ timeAgo(v.created_at) }}</span>
            <p v-if="v.changelog">{{ v.changelog }}</p>
          </li>
        </ul>
      </section>
    </article>
  </ContentShell>
  <ClientCreationPrompt v-model:show="creationOpen" :kind="pack ? kindLabel(pack.kind) : undefined" />
  <PosterDialog :open="posterOpen" :poster="posterInput" :name="pack?.title ?? ''" @close="posterOpen = false" />
</template>

<style scoped>
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.pack {
  padding: 0 0 8px;
}
.cover {
  aspect-ratio: 16 / 9;
  background-size: cover;
  background-position: center;
}
.cover.game-cover { background-size:contain; background-repeat:no-repeat; background-color:var(--surface-2); }
.head {
  padding: 14px 16px 0;
}
h1 {
  margin: 0;
  font-size: 22px;
  line-height:1.35;
  overflow-wrap:anywhere;
  font-weight: 600;
}
.author-row { display:flex; align-items:center; gap:10px; margin-top:16px; min-width:0; }
.author-row > div { min-width:0; }
.author-name { font-size:14px; font-weight:600; overflow-wrap:anywhere; }
.author-meta { margin:2px 0 0; font-size:12px; color:var(--ink-3); }
.meta {
  margin: 6px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
.meta .warn {
  color: var(--brand);
}
.summary {
  margin: 12px 0 0;
  color: var(--ink-2);
  white-space: pre-line;
  overflow-wrap:anywhere;
  line-height:1.75;
}
.tags {
  margin: 8px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
.notice {
  margin: 12px 0 0;
  padding: 8px 10px;
  background: var(--surface-2);
  border-radius: 6px;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.actions {
  margin-top: 14px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.actions .btn { min-height:44px; }
.only-phone {
  display: none;
}
/* 手机不能应用内容包，只留下载与复制链接。 */
@media (max-width: 767px) {
  .open-app {
    display: none;
  }
  .only-phone {
    display: block;
  }
}
.hint {
  margin: 8px 0 0;
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
.versions {
  margin: 20px 16px 0;
  padding-top: 8px;
  border-top: 1px solid var(--line);
}
.versions h2 {
  font-size: var(--fs-md);
  margin: 8px 0;
}
.versions ul {
  list-style: none;
  margin: 0;
  padding: 0;
}
.versions li {
  padding: 8px 0;
  border-bottom: 1px solid var(--line);
  font-size: var(--fs-sm);
}
.versions li p {
  margin: 4px 0 0;
  color: var(--ink-2);
}
.muted {
  margin-left: 8px;
  color: var(--ink-3);
}
</style>
