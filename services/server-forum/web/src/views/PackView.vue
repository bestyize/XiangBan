<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import { ApiError } from '@/api/forum'
import { contentApi, type PackDetail } from '@/api/content'
import { applyLink, coverStyle, fmtSize, kindLabel, trustLabel } from '@/lib/content'
import { timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

const route = useRoute()
const toast = useToast()
const session = useSession()
const pack = ref<PackDetail | null>(null)
const missing = ref(false)
const busy = ref(false)

useHead(() => ({ title: pack.value?.title ?? '内容详情' }))

watch(
  () => route.params.id,
  async id => {
    pack.value = null
    missing.value = false
    try {
      pack.value = await contentApi.pack(Number(id))
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) missing.value = true
      else toast.error(e)
    }
  },
  { immediate: true },
)

const isAuthor = computed(() => !!pack.value?.author && pack.value.author.uid === session.user?.uid)
const latest = computed(() => pack.value?.versions[0])
const open = computed(() => (pack.value && latest.value ? applyLink(pack.value, latest.value.version, latest.value.sha256) : ''))

async function download() {
  if (!pack.value || busy.value) return
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

async function copyLink() {
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
    <p v-else-if="!pack" class="state">加载中</p>
    <article v-else class="pack">
      <div class="cover" :style="coverStyle(pack)" role="img" :aria-label="`${pack.title} 封面`" />
      <div class="head">
        <h1>{{ pack.title }}</h1>
        <p class="meta">
          <span>{{ kindLabel(pack.kind) }}</span>
          <span :class="{ warn: pack.trust_level === 2 }">{{ trustLabel(pack.trust_level) }}</span>
          <span v-if="pack.author">{{ pack.author.nickname }}</span>
          <span v-if="latest">版本 {{ latest.label }}</span>
          <span>{{ timeAgo(pack.updated_at) }}更新</span>
          <span v-if="pack.apply_count">{{ pack.apply_count }} 次使用</span>
        </p>
        <p v-if="pack.summary" class="summary">{{ pack.summary }}</p>
        <p v-if="pack.tags.length" class="tags">
          <RouterLink v-for="t in pack.tags" :key="t" :to="`/tag/${encodeURIComponent(t)}`">{{ t }}</RouterLink>
        </p>
        <p v-if="!pack.one_click" class="notice">
          {{ pack.trust_level === 2 ? '这是用户发布、尚未审核的内容，不支持一键应用；' : '' }}在小伴桌面端会先展示它会做什么、需要哪些权限，由你确认后才安装。
        </p>
        <div class="actions">
          <a class="btn btn-primary open-app" :href="open">在小伴中打开</a>
          <button class="btn" type="button" :disabled="busy" @click="download">下载压缩包</button>
          <button class="btn" type="button" @click="copyLink">复制链接</button>
          <RouterLink v-if="pack.topic_id" class="btn" :to="`/t/${pack.topic_id}`">看讨论</RouterLink>
          <RouterLink v-if="isAuthor" class="btn" :to="`/publish?pack=${pack.id}`">发布新版本</RouterLink>
        </div>
        <p class="hint">没有安装小伴？在桌面端“社区”页里也能找到它。</p>
        <p class="hint only-phone">在手机上请先复制链接，到电脑上打开即可应用。</p>
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
.head {
  padding: 14px 16px 0;
}
h1 {
  margin: 0;
  font-size: var(--fs-xl);
  font-weight: 600;
}
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
