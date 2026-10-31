<script setup lang="ts">
// 手机端社区的发现页内容：头图、日报、开源热榜、模型榜、追踪、导航、资料导读和 AI 工具。
// 内容形状在服务端校验，这里用 JSON 编辑；草稿不对外，标成示例的条目手机端会显示「示例数据」。
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { adminApi, AdminApiError, type DiscoverRow, type DiscoverDraft, type DiscoverToken, type DiscoverDraftInput } from '@/api/admin'
import { perform, fmtTime, notify } from './shared'

const KINDS = [
  { key: 'banner', label: '首页头图', sample: '{"tag":"本周原创","title":"标题","byline":"小伴编辑部 · 8 分钟读完","link":"topic:12"}' },
  { key: 'news', label: 'AI 日报', sample: '{"title":"标题","source":"来源","url":"https://example.com/a","cover_url":""}' },
  { key: 'repo_issue', label: '开源热榜（按期）', sample: '{"issue_no":1,"week_start":"10.23","week_end":"10.29","total_gain":86420,"wow_pct":12,"daily_gain":[1,2,3,4,5,6,7],"ranks":[{"full_name":"owner/repo","description":"简介","language":"Python","language_color":"#3572A5","star_total":12400,"star_gain":3214,"spark":[1,2,3,4,5,6,7]}]}' },
  { key: 'vendor', label: '模型厂商', sample: '{"name":"厂商名","brand_color":"#E3203C","tint":"#FFE4E8"}' },
  { key: 'model', label: '模型榜', sample: '{"name":"模型名","vendor":"厂商 key","iq":70,"code":68,"math":66,"agent":69,"price":3.4,"speed":90,"latency":0.9,"context":"200K","open":false,"reasoning":true,"cn":false}' },
  { key: 'quota', label: 'Codex 额度重置', sample: '{"progress_pct":68,"next_reset_at":"2026-11-01T10:00:00Z","recent_intervals":[5,5,4.5,5,5.5,5,5]}' },
  { key: 'launch', label: '模型上新', sample: '{"title":"标题","tags":["API","编程"]}' },
  { key: 'promo', label: '限时免费', sample: '{"text":"说明","started_at":"2026-10-28T00:00:00Z","ends_at":"2026-11-04T00:00:00Z"}' },
  { key: 'nav', label: '导航', sample: '{"group":"海外","name":"名称","vendor":"厂商","url":"https://example.com","brand_color":"#D97757","tint":"#FBE9E1"}' },
  { key: 'doc', label: '资料导读', sample: '{"product":"Claude","title":"标题","subtitle":"说明","label":"guide","url":"https://example.com"}' },
  { key: 'tool', label: 'AI 工具', sample: '{"name":"名称","developer":"开发者","type_label":"桌面应用","license":"open","platforms":["Windows","macOS"],"description":"简介","homepage_url":"https://github.com/owner/repo","discussion_tag":"tag","releases":[{"version":"v1.0.0","date":"10-20","items":[{"kind":"new","text":"内容"}]}],"artifacts":[{"os":"windows","desc":".msi 安装包","size":12582912,"sha256":"","mirror_url":"","github_url":"https://github.com/owner/repo/releases"}]}' },
  { key: 'hotword', label: '热门搜索', sample: '{"text":"桌面特效","tag":"热"}' },
] as const

const kind = ref<string>(KINDS[0].key)
const view = ref<'live' | 'drafts'>('drafts')
const status = ref('')
const rows = ref<DiscoverRow[]>([])
const drafts = ref<DiscoverDraft[]>([])
const page = ref(1)
const total = ref(0)
const editing = ref<string | null>(null)
const current = ref<DiscoverDraft | null>(null)
const liveData = ref<Record<string, unknown> | null>(null)
const busy = ref(false)
const loading = ref(false)
const loadError = ref('')
const form = reactive({ key: '', sort_order: 0, sample: false, published_at: '', source_name: '', source_url: '', json: '', expected_revision: 0 })
const previewData = ref<Record<string, unknown> | null>(null)
const publishAfter = ref('')
const importOpen = ref(false)
const importJson = ref('[\n  {"kind":"news","key":"source-url-hash","expected_revision":0,"data":{"title":"标题","source":"官方来源","url":"https://example.com/news"}}\n]')
const tokensOpen = ref(false)
const tokens = ref<DiscoverToken[]>([])
const issuedToken = ref('')
const tokenForm = reactive({ name: '', days: 30, kinds: ['news'] as string[] })
const visibleRows = computed(() => view.value === 'live' ? rows.value : drafts.value)
let loadSeq = 0

async function load() {
  const mine = ++loadSeq
  const selectedKind = kind.value
  loading.value = true
  loadError.value = ''
  try {
    if (view.value === 'live') {
      const result = await adminApi.discoverItems(selectedKind)
      if (mine === loadSeq) rows.value = result.items
    } else {
      const result = await adminApi.discoverDrafts({ kind: selectedKind, status: status.value || undefined, page: page.value })
      if (mine === loadSeq) { drafts.value = result.items; total.value = result.total }
    }
  } catch (e) {
    if (mine === loadSeq && !(e instanceof AdminApiError && e.status === 401)) {
      loadError.value = e instanceof Error ? e.message : '读取失败，请刷新重试'
    }
  } finally { if (mine === loadSeq) loading.value = false }
}

function isDraft(r: DiscoverRow | DiscoverDraft): r is DiscoverDraft { return 'revision' in r }
function title(r: { data: Record<string, unknown> }) {
  const d = r.data
  return String(d.title ?? d.name ?? d.text ?? (d.issue_no ? `第 ${d.issue_no} 期` : ''))
}
function localTime(iso: string | null) {
  return iso ? fmtTime(iso).replace(' ', 'T') : ''
}
function fill(r?: DiscoverRow | DiscoverDraft) {
  editing.value = r?.key ?? ''
  const draft = r && isDraft(r) ? r : null
  form.key = r?.key ?? ''
  form.sort_order = r?.sort_order ?? 0
  form.sample = !!r?.data.sample
  form.published_at = localTime(r?.published_at ?? null)
  form.source_name = draft?.source_name ?? ''
  form.source_url = draft?.source_url ?? ''
  form.expected_revision = draft?.revision ?? 0
  const data = { ...(r?.data ?? JSON.parse(KINDS.find(k => k.key === kind.value)!.sample)) }
  delete data.sample
  form.json = JSON.stringify(data, null, 2)
  publishAfter.value = ''
  previewData.value = null
  current.value = draft
}
async function edit(r?: DiscoverRow | DiscoverDraft) {
  liveData.value = null
  if (!r) { fill(); return }
  busy.value = true
  const selectedKind = kind.value
  try {
    const detail = await adminApi.discoverDraft(selectedKind, r.key)
    if (selectedKind !== kind.value) return
    fill(detail.draft)
    liveData.value = detail.live?.data ?? null
  } catch (e) {
    if (selectedKind !== kind.value) return
    if (e instanceof AdminApiError && e.status === 404) { fill(r); liveData.value = r.data }
    else notify(e instanceof Error ? e.message : '读取草稿失败', 'error')
  } finally { busy.value = false }
}
function input(): DiscoverDraftInput {
  return { data: JSON.parse(form.json), sort_order: form.sort_order, sample: form.sample,
    expected_revision: form.expected_revision, source_name: form.source_name, source_url: form.source_url,
    ...(form.published_at ? { published_at: new Date(form.published_at).toISOString() } : {}) }
}
async function showSavedDrafts() {
  const filtersChanged = view.value !== 'drafts' || status.value !== 'draft' || page.value !== 1
  view.value = 'drafts'
  status.value = 'draft'
  page.value = 1
  // 筛选改变由 watcher 加载；保留编辑器，避免刚保存的草稿被旧筛选隐藏。
  if (!filtersChanged) await load()
}
async function save() {
  busy.value = true
  const ok = await perform(async () => {
    const d = await adminApi.saveDiscoverDraft(kind.value, form.key.trim(), input())
    fill(d)
    await showSavedDrafts()
  }, '已存为草稿，等待人工审核')
  busy.value = false
  if (ok) await preview()
}
async function preview() {
  busy.value = true
  await perform(async () => { previewData.value = (await adminApi.previewDiscover(kind.value, { data: JSON.parse(form.json), sample: form.sample })).data })
  busy.value = false
}
async function review() {
  const d = current.value
  if (!d || d.status !== 'draft' || !previewData.value) return
  if (!window.confirm(publishAfter.value ? `确认已审核 ${d.key}，于 ${publishAfter.value.replace('T', ' ')} 发布？` : `确认已审核 ${d.key}，立即发布？`)) return
  busy.value = true
  const ok = await perform(() => adminApi.reviewDiscover(d.kind, d.key, d.revision, publishAfter.value ? new Date(publishAfter.value).toISOString() : undefined), publishAfter.value ? '已安排定时发布' : '已发布')
  busy.value = false
  if (ok) { editing.value = null; await load() }
}
async function remove(r: DiscoverRow | DiscoverDraft) {
  if (!window.confirm(isDraft(r) ? `丢弃 ${r.key} 的编辑副本？` : `删除线上内容 ${r.key}？`)) return
  busy.value = true
  const ok = await perform(() => isDraft(r) ? adminApi.discardDiscover(kind.value, r.key, r.revision) : adminApi.deleteDiscover(kind.value, r.key), '已删除')
  busy.value = false
  if (ok) { editing.value = null; await load() }
}
async function cancel(r: DiscoverDraft) {
  busy.value = true
  const ok = await perform(() => adminApi.cancelDiscover(kind.value, r.key, r.revision), '定时发布已取消，需重新审核')
  busy.value = false
  if (ok) { editing.value = null; await load() }
}
async function importBatch() {
  busy.value = true
  const ok = await perform(async () => {
    const items = JSON.parse(importJson.value)
    if (!Array.isArray(items)) throw new Error('请填 JSON 数组')
    await adminApi.importDiscoverDrafts(items)
  }, '整批已导入草稿，等待人工审核')
  busy.value = false
  if (ok) { importOpen.value = false; await showSavedDrafts() }
}
async function loadTokens() {
  await perform(async () => { tokens.value = (await adminApi.discoverTokens()).items })
}
async function createToken() {
  busy.value = true
  await perform(async () => { issuedToken.value = (await adminApi.createDiscoverToken(tokenForm)).token; await loadTokens() }, '令牌已创建，仅此一次显示')
  busy.value = false
}
async function revokeToken(t: DiscoverToken) {
  if (!window.confirm(`撤销 ${t.name} 的采集权限？`)) return
  busy.value = true
  if (await perform(() => adminApi.revokeDiscoverToken(t.id), '已撤销')) { issuedToken.value = ''; await loadTokens() }
  busy.value = false
}
async function copyToken() {
  await perform(() => navigator.clipboard.writeText(issuedToken.value), '已复制')
}
watch(form, () => { previewData.value = null; current.value = null }, { flush: 'sync' })
watch([kind, view, status, page], (value, previous) => {
  if (value[0] !== previous[0]) editing.value = null
  if (value.slice(0, 3).some((v, i) => v !== previous[i]) && page.value !== 1) {
    page.value = 1
    return
  }
  void load()
})
watch(tokensOpen, v => { if (v) void loadTokens(); else issuedToken.value = '' })
onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <select v-model="kind" class="select" aria-label="内容类型" :disabled="busy"><option v-for="k in KINDS" :key="k.key" :value="k.key">{{ k.label }}</option></select>
    <select v-model="view" class="select" aria-label="内容视图" :disabled="busy"><option value="drafts">编辑与审核</option><option value="live">线上内容</option></select>
    <select v-if="view === 'drafts'" v-model="status" class="select" aria-label="草稿状态" :disabled="busy"><option value="">全部状态</option><option value="draft">待审核</option><option value="scheduled">定时发布</option><option value="published">已发布副本</option></select>
    <button class="btn btn-primary" :disabled="busy" @click="edit()">新增草稿</button>
    <button class="btn" :disabled="busy" @click="importOpen = !importOpen">批量导入</button>
    <button class="btn" :disabled="busy" @click="tokensOpen = !tokensOpen">采集令牌</button>
    <button class="btn" :disabled="busy || loading" @click="load">刷新</button>
  </div>
  <p class="muted">采集和编辑先保存草稿，人工预览后发布。定时发布只执行已审核版本；再次编辑需重新审核。</p>

  <form v-if="importOpen" class="panel adm-card adm-form" @submit.prevent="importBatch">
    <label class="field"><span>JSON 数组（每批 1–100 条，可跨类型）</span><textarea v-model="importJson" class="input adm-mono" rows="8" spellcheck="false" :disabled="busy" /></label>
    <small>新条目 expected_revision 填 0；更新填草稿版本号。任一条校验失败或版本冲突，整批不写入。</small>
    <button class="btn btn-primary" :disabled="busy" type="submit">导入草稿</button>
  </form>

  <div v-if="tokensOpen" class="panel adm-card">
    <h3>采集令牌</h3><p class="muted">仅能读写勾选类型的草稿。令牌只显示一次，请保存到本地安全位置。</p>
    <form class="adm-form" @submit.prevent="createToken">
      <div class="adm-form-grid">
        <label class="field"><span>名称</span><input v-model="tokenForm.name" class="input" required maxlength="60" :disabled="busy" /></label>
        <label class="field"><span>有效天数</span><input v-model.number="tokenForm.days" class="input" type="number" min="1" max="365" required :disabled="busy" /></label>
        <div class="field wide"><span>允许的内容类型</span><label v-for="k in KINDS" :key="k.key" class="scope"><input v-model="tokenForm.kinds" type="checkbox" :value="k.key" :disabled="busy" /> {{ k.label }}</label></div>
      </div>
      <button class="btn" type="submit" :disabled="busy || !tokenForm.kinds.length">创建令牌</button>
    </form>
    <div v-if="issuedToken" class="field"><textarea class="input adm-mono" :value="issuedToken" readonly rows="3" aria-label="新建的采集令牌" /><button class="btn" @click="copyToken">复制令牌</button></div>
    <table v-if="tokens.length" class="adm-table"><thead><tr><th>名称</th><th>类型</th><th>到期</th><th>状态</th><th>操作</th></tr></thead><tbody><tr v-for="t in tokens" :key="t.id"><td>{{ t.name }}</td><td>{{ t.kinds.join(', ') }}</td><td>{{ fmtTime(t.expires_at) }}</td><td>{{ t.revoked_at ? '已撤销' : new Date(t.expires_at).getTime() <= Date.now() ? '已过期' : '有效' }}</td><td><button v-if="!t.revoked_at" class="btn btn-sm" :disabled="busy" @click="revokeToken(t)">撤销</button></td></tr></tbody></table>
  </div>

  <form v-if="editing !== null" class="panel adm-card adm-form" @submit.prevent="save">
    <fieldset :disabled="busy" class="editor-fields">
      <div class="adm-form-grid">
        <label class="field"><span>key</span><input v-model="form.key" class="input" required pattern="[a-z0-9_\-]{1,60}" :disabled="!!editing" /><small>来源与周期使用稳定 key，重复导入不会增加条目</small></label>
        <label class="field"><span>排序（小在前）</span><input v-model.number="form.sort_order" class="input" type="number" /></label>
        <label class="field"><span>内容日期</span><input v-model="form.published_at" class="input" type="datetime-local" /><small>日报、模型上新按此日期排序；留空沿用线上日期</small></label>
        <label class="field"><span>来源名称</span><input v-model="form.source_name" class="input" maxlength="100" /></label>
        <label class="field wide"><span>一手来源地址</span><input v-model="form.source_url" class="input" type="url" placeholder="https://" /></label>
        <label class="field"><span><input v-model="form.sample" type="checkbox" /> 示例数据</span></label>
        <label class="field wide"><span>内容（JSON）</span><textarea v-model="form.json" class="input adm-mono" rows="14" spellcheck="false" /></label>
      </div>
      <div class="adm-form-actions"><button class="btn btn-primary" type="submit">保存草稿</button><button class="btn" type="button" @click="preview">校验并预览</button><button class="btn btn-ghost" type="button" @click="editing = null">关闭</button></div>
    </fieldset>
    <div v-if="previewData" class="preview-card">
      <h3>{{ title({ data: previewData }) }}</h3><p v-if="previewData.sample">示例数据</p>
      <pre class="adm-mono">{{ JSON.stringify(previewData, null, 2) }}</pre>
      <details v-if="liveData"><summary>对照当前线上内容</summary><pre class="adm-mono">{{ JSON.stringify(liveData, null, 2) }}</pre></details>
      <div v-if="current?.status === 'draft'" class="field"><span>审核版本 {{ current.revision }} · 留空立即发布</span><input v-model="publishAfter" class="input" type="datetime-local" aria-label="审核后发布时间" :disabled="busy" /><button class="btn btn-primary" type="button" :disabled="busy" @click="review">{{ publishAfter ? '审核通过并定时发布' : '审核通过并立即发布' }}</button></div>
      <p v-else class="muted">先保存当前内容为草稿，再审核发布。</p>
    </div>
  </form>

  <div class="panel adm-card">
    <p v-if="loading" class="adm-empty">加载中…</p>
    <p v-else-if="loadError" class="adm-empty error" role="alert">{{ loadError }}<button class="btn btn-sm" :disabled="busy" @click="load">重新加载</button></p>
    <p v-else-if="!visibleRows.length" class="adm-empty">还没有内容</p>
    <table v-else class="adm-table"><thead><tr><th>key</th><th>标题</th><th>排序</th><th>状态</th><th>操作</th></tr></thead><tbody>
      <tr v-for="r in visibleRows" :key="r.key">
        <td data-label="key" class="adm-mono">{{ r.key }}</td>
        <td data-label="标题">{{ title(r) }}<span v-if="r.data.sample"> · 示例</span><small v-if="isDraft(r) && r.last_error" class="error">{{ r.last_error }}</small></td>
        <td data-label="排序">{{ r.sort_order }}</td>
        <td data-label="状态">{{ r.status === 'published' ? '已发布' : r.status === 'scheduled' ? '定时发布' : '待审核' }}<small v-if="isDraft(r) && r.publish_after">{{ fmtTime(r.publish_after) }}</small></td>
        <td data-label="操作" class="adm-actions"><button class="btn btn-sm" :disabled="busy" @click="edit(r)">编辑与预览</button><button v-if="isDraft(r) && r.status === 'scheduled'" class="btn btn-sm" :disabled="busy" @click="cancel(r)">取消定时</button><button class="btn btn-sm" :disabled="busy" @click="remove(r)">{{ isDraft(r) ? '丢弃副本' : '删除线上' }}</button></td>
      </tr>
    </tbody></table>
    <div v-if="view === 'drafts'" class="adm-toolbar"><button class="btn btn-sm" :disabled="busy || loading || page <= 1" @click="page--">上一页</button><span>第 {{ page }} 页 · 共 {{ total }} 条</span><button class="btn btn-sm" :disabled="busy || loading || page * 50 >= total" @click="page++">下一页</button></div>
  </div>
</template>

<style scoped>
.editor-fields { border: 0; padding: 0; margin: 0; min-width: 0; }
.preview-card { margin-top: 1rem; }
.preview-card pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 32rem; overflow: auto; }
.scope { display: inline-block; margin: .3rem .8rem .3rem 0; }
.adm-table small { display: block; }
</style>
