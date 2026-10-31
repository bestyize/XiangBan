<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { AdminApiError } from '@/api/admin'
import { collectionApi, type CollectionBatch, type CollectionRun, type CollectionSource } from '@/api/collection'
import { errorText, fmtTime, notify } from './shared'

const sources = ref<CollectionSource[]>([])
const sourceLoading = ref(false)
const sourceError = ref('')
const forms = reactive<Record<string, { key: string; enabled: boolean; interval: number; repositories: string; feedIds: string[] }>>({})
const busy = reactive<Record<string, string>>({})
const actionErrors = reactive<Record<string, string>>({})
const accepted = reactive<Record<string, string>>({})
const filter = ref('')
const runs = ref<CollectionRun[]>([])
const page = ref(1)
const pageSize = ref(30)
const total = ref(0)
const runLoading = ref(false)
const runError = ref('')
const polling = ref(false)
const refreshNotice = ref('')
const previewRun = ref('')
const batch = ref<CollectionBatch | null>(null)
const previewLoading = ref(false)
const previewError = ref('')
const approving = ref(false)
const read = ref<string[]>([])
const acknowledged = ref(false)
let sourceSeq = 0
let runSeq = 0
let previewSeq = 0
let disposed = false
let timer: ReturnType<typeof setTimeout> | undefined
let pollsLeft = 0

const STATUS: Record<string, string> = { queued: '排队中', running: '采集中', succeeded: '采集完成', failed: '采集失败' }
const SOURCE_NAMES: Record<string, string> = { artificial_analysis: 'Artificial Analysis · 能力榜', openrouter: 'OpenRouter · 调用量榜', github_repos: 'GitHub · 开源热榜', official_news: '官方 AI 日报' }
const FEEDS = [
  { id: 'openai_news', title: 'OpenAI', url: 'https://openai.com/news/rss.xml' },
  { id: 'google_deepmind', title: 'Google DeepMind', url: 'https://deepmind.google/blog/rss.xml' },
  { id: 'google_research', title: 'Google Research', url: 'https://research.google/blog/rss/' },
] as const
const needsCredential = (s: CollectionSource) => s.credential_required !== false
function ready(s: CollectionSource) {
  if (needsCredential(s) && !s.configured) return false
  if (s.id === 'github_repos') return !!s.config?.repositories?.length
  if (s.id === 'official_news') return !!s.config?.feed_ids?.length
  return true
}
function readinessHint(s: CollectionSource) {
  if (needsCredential(s) && !s.configured) return '请先配置有效密钥。'
  return s.id === 'github_repos' ? '请先保存至少一个候选仓库。' : '请先保存至少一个官方来源。'
}
const savedFeeds = (s: CollectionSource) => (s.config?.feed_ids ?? []).map(id => FEEDS.find(f => f.id === id)?.title ?? id).join('、') || '尚未选择'
const sourceName = (id: string) => sources.value.find(s => s.id === id)?.title || SOURCE_NAMES[id] || id
const active = (id: string) => !!accepted[id] || runs.value.some(r => r.source === id && ['queued', 'running'].includes(r.status))
const isAuthError = (e: unknown) => e instanceof AdminApiError && e.status === 401
const itemId = (kind: string, key: string) => JSON.stringify([kind, key])
const canReview = computed(() => !!batch.value && batch.value.status === 'succeeded' && !batch.value.reviewed_at && batch.value.items.length > 0 && batch.value.items.every(i => !i.conflict && i.draft !== null && ['draft', 'published'].includes(i.draft.status)))
const allRead = computed(() => !!batch.value?.items.length && batch.value.items.every(i => read.value.includes(itemId(i.kind, i.key))))

function clearPreview() {
  ++previewSeq
  previewRun.value = ''
  batch.value = null
  previewLoading.value = false
  previewError.value = ''
  read.value = []
  acknowledged.value = false
}

async function loadSources() {
  const mine = ++sourceSeq
  sourceLoading.value = true
  sourceError.value = ''
  try {
    const result = await collectionApi.sources()
    if (disposed || mine !== sourceSeq) return
    sources.value = result.items
    for (const s of result.items) {
      if (!forms[s.id]) forms[s.id] = { key: '', enabled: s.enabled, interval: s.interval_minutes, repositories: (s.config?.repositories ?? []).join('\n'), feedIds: [...(s.config?.feed_ids ?? [])] }
    }
  } catch (e) {
    if (!disposed && mine === sourceSeq && !isAuthError(e)) sourceError.value = errorText(e)
  } finally {
    if (!disposed && mine === sourceSeq) sourceLoading.value = false
  }
}
function reconcile(items: CollectionRun[]) {
  for (const r of items) if (accepted[r.source] === r.id && !['queued', 'running'].includes(r.status)) delete accepted[r.source]
}
async function loadRuns() {
  const mine = ++runSeq
  runLoading.value = true
  runError.value = ''
  try {
    const result = await collectionApi.runs(filter.value, page.value)
    if (disposed || mine !== runSeq) return
    reconcile(result.items)
    runs.value = result.items
    total.value = result.total
    pageSize.value = result.page_size
  } catch (e) {
    if (!disposed && mine === runSeq && !isAuthError(e)) runError.value = errorText(e)
  } finally {
    if (!disposed && mine === runSeq) runLoading.value = false
  }
}
function stopPolling() {
  clearTimeout(timer)
  timer = undefined
  pollsLeft = 0
  polling.value = false
}
function pollAgain() {
  if (disposed || pollsLeft <= 0 || !Object.keys(accepted).length) {
    const pending = Object.keys(accepted).length > 0
    stopPolling()
    refreshNotice.value = pending ? '自动刷新已暂停，任务仍在后台执行。请稍后刷新查看结果，不要重复启动。' : '当前任务已结束，请查看运行记录并审核待审内容。'
    return
  }
  timer = setTimeout(async () => {
    --pollsLeft
    await Promise.all([loadSources(), loadRuns()])
    if (disposed) return
    // 筛选或翻页时，也检查刚提交任务的状态，防止按钮永久停留在执行中。
    if (Object.keys(accepted).length && (filter.value || page.value !== 1)) {
      try { reconcile((await collectionApi.runs()).items) } catch { /* 列表已有错误提示；不重复弹窗 */ }
    }
    if (sourceError.value || runError.value) {
      stopPolling()
      refreshNotice.value = '自动刷新遇到错误已暂停，请手动重试查看任务状态。'
      return
    }
    pollAgain()
  }, 3000)
}
async function refresh() {
  clearPreview()
  await Promise.all([loadSources(), loadRuns()])
}
async function mutate(id: string, operation: string, fn: () => Promise<void>) {
  if (busy[id]) return
  busy[id] = operation
  actionErrors[id] = ''
  try { await fn() } catch (e) {
    if (!disposed && !isAuthError(e)) actionErrors[id] = errorText(e)
  } finally { if (!disposed) delete busy[id] }
}
async function saveKey(s: CollectionSource) {
  const key = forms[s.id].key.trim()
  if (!key) { actionErrors[s.id] = '请输入 API key'; return }
  await mutate(s.id, 'key', async () => {
    await collectionApi.credential(s.id, key)
    forms[s.id].key = ''
    notify('密钥验证通过并已保存')
    await loadSources()
  })
}
async function configure(s: CollectionSource) {
  const form = forms[s.id]
  if (!Number.isInteger(form.interval) || form.interval < 60 || form.interval > 10080) {
    actionErrors[s.id] = '采集周期须为 60–10080 分钟之间的整数'
    return
  }
  if (form.enabled && !ready(s)) { actionErrors[s.id] = readinessHint(s); return }
  await mutate(s.id, 'config', async () => {
    await collectionApi.configure(s.id, { enabled: form.enabled, interval_minutes: form.interval })
    notify('采集设置已保存')
    await loadSources()
  })
}
async function saveSourceConfig(s: CollectionSource) {
  const form = forms[s.id]
  let config: { repositories?: string[]; feed_ids?: string[] }
  if (s.id === 'github_repos') {
    const repositories = form.repositories.split(/\r?\n/).map(r => r.trim().toLowerCase()).filter(Boolean)
    const valid = (r: string) => {
      const parts = r.split('/')
      return parts.length === 2 && /^[a-z0-9](?:[a-z0-9-]{0,37}[a-z0-9])?$/.test(parts[0]) && /^[a-z0-9_.-]{1,100}$/.test(parts[1]) && !['.', '..'].includes(parts[1])
    }
    if (repositories.length > 30 || repositories.some(r => !valid(r)) || new Set(repositories).size !== repositories.length) {
      actionErrors[s.id] = '每行填写一个 owner/repo，最多 30 个，不可重复；请勿填写 URL'
      return
    }
    config = { repositories }
  } else if (s.id === 'official_news') {
    config = { feed_ids: [...form.feedIds] }
  } else return
  await mutate(s.id, 'source-config', async () => {
    // 独立保存来源候选，避免覆盖尚未提交的周期/开关设置。
    await collectionApi.configure(s.id, { config })
    if (config.repositories) form.repositories = config.repositories.join('\n')
    notify('来源配置已保存')
    await loadSources()
  })
}
async function run(s: CollectionSource) {
  if (active(s.id)) return
  await mutate(s.id, 'run', async () => {
    const result = await collectionApi.run(s.id)
    accepted[s.id] = result.run_id
    clearPreview()
    filter.value = ''
    page.value = 1
    refreshNotice.value = '任务已受理，结果先进入待审。接下来最多自动刷新 30 秒。'
    notify('已启动采集，完成后请预览并人工审核')
    await Promise.all([loadSources(), loadRuns()])
    stopPolling()
    pollsLeft = 10
    polling.value = true
    pollAgain()
  })
}
function search() { page.value = 1; clearPreview(); void loadRuns() }
function goto(p: number) { page.value = p; clearPreview(); void loadRuns() }
async function preview(runId: string) {
  stopPolling()
  clearPreview()
  previewRun.value = runId
  previewLoading.value = true
  const mine = ++previewSeq
  try {
    const result = await collectionApi.drafts(runId)
    if (disposed || mine !== previewSeq) return
    batch.value = result
  } catch (e) {
    if (!disposed && mine === previewSeq && !isAuthError(e)) previewError.value = errorText(e)
  } finally { if (!disposed && mine === previewSeq) previewLoading.value = false }
}
function markRead(event: Event, kind: string, key: string) {
  if ((event.target as HTMLDetailsElement).open) {
    const id = itemId(kind, key)
    if (!read.value.includes(id)) read.value.push(id)
  }
}
async function review() {
  const current = batch.value
  if (!current || !canReview.value || !allRead.value || !acknowledged.value || approving.value) return
  if (!window.confirm(`确认已人工审核全部 ${current.items.length} 条内容，立即整批发布？`)) return
  approving.value = true
  acknowledged.value = false
  try {
    const result = await collectionApi.review(current.run_id, current.items)
    notify(`已整批发布 ${result.item_count} 条内容`)
    clearPreview()
    await Promise.all([loadSources(), loadRuns()])
  } catch (e) {
    // 无论是版本冲突还是网络错误，均须重读完整批次；不能沿用旧确认重试发布。
    batch.value = null
    read.value = []
    if (!isAuthError(e)) previewError.value = `${errorText(e)}。请重新读取本次草稿，核对最新版本后再审核。`
  } finally { if (!disposed) approving.value = false }
}
onMounted(() => { void refresh() })
onUnmounted(() => {
  disposed = true
  ++sourceSeq; ++runSeq; ++previewSeq
  stopPolling()
  for (const form of Object.values(forms)) form.key = ''
})
</script>

<template>
  <div class="collection-panel">
    <div class="adm-toolbar">
      <button class="btn" :disabled="sourceLoading || runLoading || approving" @click="refresh">刷新状态</button>
      <RouterLink class="btn" to="/admin/discover">发现页编辑与审核</RouterLink>
    </div>
    <p class="muted">采集只生成待审草稿，不会自动发布。请预览本次完整批次，人工终审后再发布；失败会保留上一版线上内容。</p>
    <div v-if="sourceError" class="collection-error" role="alert">来源配置读取失败：{{ sourceError }} <button class="btn btn-sm" :disabled="sourceLoading" @click="loadSources">重试</button></div>
    <p v-if="sourceLoading && !sources.length" role="status">正在读取采集来源…</p>
    <p v-else-if="!sources.length && !sourceError" class="muted">暂无可用采集来源。</p>
    <div class="collection-sources" :aria-busy="sourceLoading">
      <section v-for="s in sources" :key="s.id" class="panel adm-card collection-source">
        <div class="collection-heading">
          <h2>{{ sourceName(s.id) }}</h2>
          <span class="adm-tag" :class="!needsCredential(s) || s.configured ? 'valid' : 'suspect'">{{ !needsCredential(s) ? '公开来源 · 无需密钥' : s.configured ? '密钥已配置' : '未配置密钥' }}</span>
        </div>
        <p v-if="s.id === 'openrouter'" class="muted">全平台每日输入 + 输出 Token 用量榜，仅代表 OpenRouter 平台流量，数值不是请求次数。</p>
        <p v-else-if="s.id === 'artificial_analysis'" class="muted">采集模型能力、价格等官方指标；缺失指标保留为空，不以零代替。</p>
        <p v-if="s.hint" class="muted">{{ s.hint }}</p>
        <form v-if="needsCredential(s)" class="adm-form collection-key" @submit.prevent="saveKey(s)">
          <label class="field"><span>{{ s.configured ? '更新 API key' : '配置 API key' }}</span><input v-model="forms[s.id].key" class="input" type="password" autocomplete="off" spellcheck="false" :disabled="!!busy[s.id] || approving" :aria-label="`${sourceName(s.id)} API key`" placeholder="只写入，不回显已保存密钥" /></label>
          <button class="btn" type="submit" :disabled="!!busy[s.id] || approving || !forms[s.id].key.trim()">{{ busy[s.id] === 'key' ? '验证并保存中…' : '验证并保存密钥' }}</button>
        </form>
        <small v-if="needsCredential(s)" class="muted">验证成功后更新已保存密钥，并清空输入；密钥不会保存到浏览器本地存储。</small>
        <form v-if="s.id === 'github_repos'" class="adm-form collection-source-config" @submit.prevent="saveSourceConfig(s)">
          <label class="field"><span>候选仓库（每行 owner/repo，最多 30 个）</span><textarea v-model="forms[s.id].repositories" class="input" rows="5" spellcheck="false" :disabled="!!busy[s.id] || approving" placeholder="例如 rust-lang/rust&#10;vuejs/core" /></label>
          <small class="muted">只跟踪明确选择的仓库，每日记录 star 快照；首日为基线，不会伪造七日增长。仓库名保存时统一为小写。</small>
          <button class="btn" type="submit" :disabled="!!busy[s.id] || approving">{{ busy[s.id] === 'source-config' ? '保存中…' : '保存候选仓库' }}</button>
          <p class="collection-saved muted">已保存 {{ s.config?.repositories?.length ?? 0 }} 个：{{ s.config?.repositories?.join('、') || '暂无候选' }}</p>
        </form>
        <form v-else-if="s.id === 'official_news'" class="adm-form collection-source-config" @submit.prevent="saveSourceConfig(s)">
          <fieldset class="collection-feed-options" :disabled="!!busy[s.id] || approving"><legend>官方 AI 日报来源</legend><label v-for="feed in FEEDS" :key="feed.id" class="collection-check"><input v-model="forms[s.id].feedIds" type="checkbox" :value="feed.id" /> {{ feed.title }} <a :href="feed.url" target="_blank" rel="noopener noreferrer">查看官方 RSS</a></label></fieldset>
          <small class="muted">只采集官方标题、链接与来源时间，生成待审内容；不抓取文章全文，不接受自定义地址。</small>
          <button class="btn" type="submit" :disabled="!!busy[s.id] || approving">{{ busy[s.id] === 'source-config' ? '保存中…' : '保存日报来源' }}</button>
          <p class="collection-saved muted">已保存来源：{{ savedFeeds(s) }}</p>
        </form>
        <form class="collection-config" @submit.prevent="configure(s)">
          <label class="collection-check"><input v-model="forms[s.id].enabled" type="checkbox" :disabled="!!busy[s.id] || approving" /> 启用自动采集</label>
          <label class="field"><span>周期（分钟）</span><input v-model.number="forms[s.id].interval" class="input" type="number" min="60" max="10080" step="1" required :disabled="!!busy[s.id] || approving" :aria-label="`${sourceName(s.id)}采集周期`" /></label>
          <button class="btn" type="submit" :disabled="!!busy[s.id] || approving">{{ busy[s.id] === 'config' ? '保存中…' : '保存采集设置' }}</button>
        </form>
        <dl class="collection-meta">
          <div><dt>已保存状态</dt><dd>{{ s.enabled ? '自动采集已启用' : '自动采集已停用' }} · 每 {{ s.interval_minutes }} 分钟</dd></div>
          <div><dt>最后成功</dt><dd>{{ fmtTime(s.last_success_at) }}</dd></div>
          <div><dt>下次执行</dt><dd>{{ s.enabled ? fmtTime(s.next_run_at) : '已停用' }}</dd></div>
        </dl>
        <p v-if="s.last_error" class="collection-error">最近失败：{{ s.last_error }}</p>
        <p v-if="actionErrors[s.id]" class="collection-error" role="alert">{{ actionErrors[s.id] }}</p>
        <button class="btn btn-primary" :disabled="!!busy[s.id] || approving || !ready(s) || active(s.id)" @click="run(s)">{{ active(s.id) || busy[s.id] === 'run' ? '任务已受理 / 执行中' : '立即采集到待审' }}</button>
        <small v-if="!ready(s)" class="muted"> {{ readinessHint(s) }}</small>
      </section>
    </div>

    <section class="panel adm-card collection-runs">
      <div class="collection-heading"><h2>采集运行记录</h2><button class="btn btn-sm" :disabled="runLoading || approving" @click="clearPreview(); loadRuns()">刷新记录</button></div>
      <div class="adm-toolbar"><label>来源 <select v-model="filter" class="select" :disabled="approving" @change="search"><option value="">全部来源</option><option v-for="s in sources" :key="s.id" :value="s.id">{{ sourceName(s.id) }}</option></select></label><span v-if="polling" class="muted" role="status">正在短暂自动刷新…</span></div>
      <p v-if="refreshNotice" class="muted" role="status">{{ refreshNotice }}</p>
      <div v-if="runError" class="collection-error" role="alert">运行记录读取失败：{{ runError }} <button class="btn btn-sm" :disabled="runLoading" @click="clearPreview(); loadRuns()">重试</button></div>
      <p v-if="runLoading && !runs.length" role="status">正在读取运行记录…</p>
      <p v-else-if="!runs.length && !runError" class="muted">暂无运行记录，配置来源后可手动执行一次。</p>
      <div v-if="runs.length" class="collection-table" :aria-busy="runLoading">
        <table class="adm-table"><thead><tr><th>来源 / 任务</th><th>状态</th><th>开始 / 结束</th><th>条目</th><th>结果与审核</th></tr></thead><tbody>
          <tr v-for="r in runs" :key="r.id"><td>{{ sourceName(r.source) }}<small class="collection-run-id">{{ r.id }}</small></td><td><span class="adm-tag" :class="r.status === 'failed' ? 'danger' : r.status === 'succeeded' ? 'valid' : 'suspect'">{{ STATUS[r.status] || r.status }}</span></td><td>{{ fmtTime(r.started_at) }}<br /><span class="muted">{{ fmtTime(r.finished_at) }}</span></td><td>{{ r.item_count }}</td><td><p v-if="r.error" class="collection-error">{{ r.error }}</p><button v-if="r.status === 'succeeded' && r.item_count > 0" class="btn btn-sm" :disabled="previewLoading || approving" @click="preview(r.id)">预览本次草稿</button><span v-else-if="r.status === 'succeeded'" class="muted">{{ r.source === 'github_repos' ? '已记录基线，正在积累每日快照；本次没有新增待审内容' : '没有新增待审内容' }}</span></td></tr>
        </tbody></table>
      </div>
      <div v-if="total > pageSize" class="collection-pagination"><button class="btn btn-sm" :disabled="page <= 1 || runLoading || approving" @click="goto(page - 1)">上一页</button><span>第 {{ page }} 页 · 共 {{ total }} 次</span><button class="btn btn-sm" :disabled="page * pageSize >= total || runLoading || approving" @click="goto(page + 1)">下一页</button></div>
    </section>

    <section v-if="previewRun" class="panel adm-card collection-preview" :aria-busy="previewLoading">
      <div class="collection-heading"><h2>本次采集完整批次</h2><button class="btn btn-sm" :disabled="approving || previewLoading" @click="preview(previewRun)">重新读取草稿</button><button class="btn btn-sm" :disabled="approving" @click="clearPreview">关闭</button></div>
      <p class="muted">任务 {{ previewRun }}。逐条展开核对候选内容与线上版本；发布时将核对完整批次及每条版本，冲突不会覆盖线上内容。</p>
      <p v-if="previewLoading" role="status">正在读取完整批次…</p>
      <p v-if="previewError" class="collection-error" role="alert">{{ previewError }}</p>
      <template v-if="batch">
        <p v-if="batch.reviewed_at" class="muted">本批次已于 {{ fmtTime(batch.reviewed_at) }} 终审发布。</p>
        <p v-else-if="!batch.items.length" class="muted">本次没有新增草稿。</p>
        <p v-else-if="!canReview" class="collection-error" role="alert">本批次存在版本冲突或不可审核状态。请到发现页核对内容并重新采集，不能使用旧版本覆盖。</p>
        <details v-for="item in batch.items" :key="itemId(item.kind, item.key)" class="collection-draft" @toggle="markRead($event, item.kind, item.key)">
          <summary>{{ item.kind }} / {{ item.key }} · 版本 {{ item.draft?.revision ?? '已删除' }} <span v-if="item.conflict" class="collection-error">（与采集版本 {{ item.expected_revision }} 冲突）</span><span v-if="read.includes(itemId(item.kind, item.key))" class="muted"> · 已展开</span></summary>
          <div class="collection-diff"><div><h3>待审内容</h3><pre v-if="item.draft">{{ JSON.stringify(item.draft.data, null, 2) }}</pre><p v-else class="collection-error">草稿已删除，不能审核此批次。</p></div><div><h3>当前线上内容</h3><pre v-if="item.live">{{ JSON.stringify(item.live.data, null, 2) }}</pre><p v-else class="muted">尚无线上内容</p></div></div>
        </details>
        <template v-if="canReview">
          <label class="collection-check collection-ack"><input v-model="acknowledged" type="checkbox" :disabled="!allRead || approving" /> 我已核对全部 {{ batch.items.length }} 条候选及其线上版本，确认整批立即发布</label>
          <small v-if="!allRead" class="muted">请先逐条展开查看本次完整批次。</small>
          <button class="btn btn-primary" :disabled="!acknowledged || !allRead || approving || previewLoading" @click="review">{{ approving ? '整批发布中…' : '人工终审，整批发布' }}</button>
        </template>
      </template>
    </section>
  </div>
</template>

<style scoped>
.collection-sources { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 380px), 1fr)); gap: 16px; margin-top: 18px; }
.collection-source, .collection-runs, .collection-preview { min-width: 0; }
.collection-heading { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; margin-bottom: 12px; }
.collection-heading h2 { margin: 0 auto 0 0; font-size: 17px; }
.collection-key, .collection-source-config { margin-top: 18px; }
.collection-feed-options { margin: 0; padding: 10px 12px; border: 1px solid var(--line); border-radius: 8px; }
.collection-feed-options a { font-size: 12px; margin-left: auto; }
.collection-saved { overflow-wrap: anywhere; margin: 0; }
.collection-source-config .btn { align-self: start; }
.collection-key .btn { align-self: start; }
.collection-config { display: flex; align-items: end; flex-wrap: wrap; gap: 12px; margin: 20px 0; }
.collection-config .field { flex: 1; min-width: 130px; }
.collection-config .input { width: 100%; }
.collection-check { display: flex; align-items: center; gap: 8px; min-height: 36px; }
.collection-meta { display: grid; gap: 8px; margin: 18px 0; font-size: 13px; }
.collection-meta > div { display: flex; gap: 12px; }
.collection-meta dt { min-width: 65px; color: var(--ink-3); }
.collection-meta dd { margin: 0; }
.collection-error { color: var(--danger); overflow-wrap: anywhere; white-space: pre-wrap; }
.collection-runs, .collection-preview { margin-top: 20px; }
.collection-table { overflow-x: auto; }
.collection-table td { vertical-align: top; }
.collection-run-id { display: block; font-size: 11px; color: var(--ink-3); overflow-wrap: anywhere; max-width: 280px; }
.collection-pagination { display: flex; align-items: center; justify-content: end; gap: 12px; margin-top: 16px; }
.collection-draft { border: 1px solid var(--line); padding: 12px; margin: 12px 0; border-radius: 8px; }
.collection-draft summary { cursor: pointer; overflow-wrap: anywhere; }
.collection-diff { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; }
.collection-diff h3 { font-size: 13px; }
.collection-diff pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 460px; overflow: auto; font-size: 12px; }
.collection-ack { margin: 18px 0 10px; }
.collection-preview > .btn-primary { margin-top: 12px; }
@media (max-width: 700px) { .collection-diff { grid-template-columns: 1fr; } }
</style>
