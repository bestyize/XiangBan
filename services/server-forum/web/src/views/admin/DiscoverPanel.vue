<script setup lang="ts">
// 发现页编排：左侧 12 类频道导航 + 右侧草稿/线上双视图列表 + 右侧宽抽屉（结构化表单 / Diff 对照 / 高级 JSON）。
import { computed, onMounted, reactive, ref, watch } from 'vue'
import {
  adminApi,
  AdminApiError,
  type DiscoverDraft,
  type DiscoverDraftInput,
  type DiscoverRow,
  type DiscoverToken,
} from '@/api/admin'
import AdminDrawer from './AdminDrawer.vue'
import AdminIcon from './AdminIcon.vue'
import AdminModal from './AdminModal.vue'
import DiscoverFieldsEditor from './DiscoverFieldsEditor.vue'
import { fmtTime, notify, perform } from './shared'

const KINDS = [
  { key: 'banner', label: '首页头图', sample: '{"tag":"本周原创","title":"标题","byline":"小伴编辑部 · 8 分钟读完","link":"topic:12"}' },
  { key: 'frontier', label: '小伴前沿', sample: '{"title":"标题","source":"来源","url":"https://example.com/a","cover_url":"","summary":"文章摘要","body":"文章正文"}' },
  { key: 'news', label: 'AI 日报', sample: '{"title":"标题","source":"来源","url":"https://example.com/a","cover_url":"","summary":"文章摘要","body":"## 小标题\\n\\n站内文章正文，支持 Markdown 和图片。"}' },
  { key: 'repo_issue', label: '开源热榜（按期）', sample: '{"issue_no":1,"week_start":"10.23","week_end":"10.29","total_gain":86420,"wow_pct":12,"daily_gain":[1,2,3,4,5,6,7],"ranks":[{"full_name":"owner/repo","description":"简介","language":"Python","language_color":"#3572A5","star_total":12400,"star_gain":3214,"spark":[1,2,3,4,5,6,7]}]}' },
  { key: 'vendor', label: '模型厂商', sample: '{"name":"厂商名","brand_color":"#E3203C","tint":"#FFE4E8"}' },
  { key: 'model', label: '模型榜', sample: '{"name":"模型名","vendor":"厂商 key","iq":70,"code":68,"math":66,"agent":69,"price":3.4,"speed":90,"latency":0.9,"context":"200K","open":false,"reasoning":true,"cn":false}' },
  { key: 'nav', label: 'AI 导航', sample: '{"category":"models","group":"大模型","region":"cn","summary":"一句话用途","name":"名称","vendor":"厂商","url":"https://example.com","brand_color":"#D97757","tint":"#FBE9E1"}' },
  { key: 'doc', label: '资料导读', sample: '{"product":"Claude","title":"标题","subtitle":"说明","label":"guide","url":"https://example.com"}' },
  { key: 'tool', label: 'AI 工具', sample: '{"name":"名称","developer":"开发者","type_label":"桌面应用","license":"open","platforms":["Windows","macOS"],"description":"简介","homepage_url":"https://github.com/owner/repo","discussion_tag":"tag","releases":[{"version":"v1.0.0","date":"10-20","items":[{"kind":"new","text":"内容"}]}],"artifacts":[{"os":"windows","desc":".msi 安装包","size":12582912,"sha256":"","mirror_url":"","github_url":"https://github.com/owner/repo/releases"}]}' },
  { key: 'hotword', label: '热门搜索', sample: '{"text":"桌面特效","tag":"热"}' },
  { key: 'model_board', label: '真实模型能力榜', sample: '{}' },
  { key: 'model_usage', label: 'OpenRouter 调用量榜', sample: '{}' },
] as const

const kind = ref<string>(KINDS[0].key)
const view = ref<'live' | 'drafts'>('drafts')
const status = ref('')
const rows = ref<DiscoverRow[]>([])
const drafts = ref<DiscoverDraft[]>([])
const page = ref(1)
const total = ref(0)
const editing = ref<string | null>(null)
const editorTab = ref<'form' | 'diff' | 'json'>('form')
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
const tokenForm = reactive({ name: '', days: 30, kinds: ['news'] as string[], publish_kinds: [] as string[] })
const confirmAction = ref<{ title: string; body: string; danger?: boolean; run: () => Promise<void> } | null>(null)

const visibleRows = computed(() => (view.value === 'live' ? rows.value : drafts.value))
const structuredKind = computed(() => ['banner', 'news', 'nav', 'doc', 'hotword', 'vendor'].includes(kind.value))
const snapshotKind = computed(() => kind.value === 'model_board' || kind.value === 'model_usage')
const currentKindLabel = computed(() => KINDS.find(k => k.key === kind.value)?.label ?? kind.value)
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
      if (mine === loadSeq) {
        drafts.value = result.items
        total.value = result.total
      }
    }
  } catch (e) {
    if (mine === loadSeq && !(e instanceof AdminApiError && e.status === 401)) {
      loadError.value = e instanceof Error ? e.message : '读取失败，请刷新重试'
    }
  } finally {
    if (mine === loadSeq) loading.value = false
  }
}

function isDraft(r: DiscoverRow | DiscoverDraft): r is DiscoverDraft {
  return 'revision' in r
}
function title(r: { data: Record<string, unknown> }) {
  const d = r.data
  return String(
    d.title ?? d.name ?? d.text ?? (d.source ? `${d.source}${d.date ? ` · ${d.date}` : ''}` : d.issue_no ? `第 ${d.issue_no} 期` : ''),
  )
}
function localTime(iso: string | null) {
  return iso ? fmtTime(iso).replace(' ', 'T') : ''
}
function fill(r?: DiscoverRow | DiscoverDraft) {
  editing.value = r?.key ?? ''
  editorTab.value = structuredKind.value ? 'form' : 'json'
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
  if (!r) {
    fill()
    return
  }
  busy.value = true
  const selectedKind = kind.value
  try {
    const detail = await adminApi.discoverDraft(selectedKind, r.key)
    if (selectedKind !== kind.value) return
    fill(detail.draft)
    liveData.value = detail.live?.data ?? null
  } catch (e) {
    if (selectedKind !== kind.value) return
    if (e instanceof AdminApiError && e.status === 404) {
      fill(r)
      liveData.value = r.data
    } else {
      notify(e instanceof Error ? e.message : '读取草稿失败', 'error')
    }
  } finally {
    busy.value = false
  }
}
function input(): DiscoverDraftInput {
  return {
    data: JSON.parse(form.json),
    sort_order: form.sort_order,
    sample: form.sample,
    expected_revision: form.expected_revision,
    source_name: form.source_name,
    source_url: form.source_url,
    ...(form.published_at ? { published_at: new Date(form.published_at).toISOString() } : {}),
  }
}
async function showSavedDrafts() {
  const filtersChanged = view.value !== 'drafts' || status.value !== 'draft' || page.value !== 1
  view.value = 'drafts'
  status.value = 'draft'
  page.value = 1
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
  const ok = await perform(async () => {
    previewData.value = (await adminApi.previewDiscover(kind.value, { data: JSON.parse(form.json), sample: form.sample })).data
  })
  busy.value = false
  if (ok) editorTab.value = 'diff'
}
function askReview() {
  const d = current.value
  if (!d || d.status !== 'draft' || !previewData.value) return
  confirmAction.value = {
    title: publishAfter.value ? '确认定时发布？' : '确认审核通过并立即发布？',
    body: publishAfter.value
      ? `确认已审核 ${d.key}（版本 ${d.revision}），将于 ${publishAfter.value.replace('T', ' ')} 自动上线？`
      : `确认已审核 ${d.key}（版本 ${d.revision}），立即发布到线上发现页？`,
    run: async () => {
      busy.value = true
      const ok = await perform(
        () => adminApi.reviewDiscover(d.kind, d.key, d.revision, publishAfter.value ? new Date(publishAfter.value).toISOString() : undefined),
        publishAfter.value ? '已安排定时发布' : '已发布上线',
      )
      busy.value = false
      if (ok) {
        editing.value = null
        await load()
      }
    },
  }
}
function askRemove(r: DiscoverRow | DiscoverDraft) {
  confirmAction.value = {
    title: isDraft(r) ? '丢弃编辑副本？' : '删除线上内容？',
    body: isDraft(r) ? `确定丢弃 ${r.key} 的草稿副本？未发布的改动将无法找回。` : `确定删除线上发现页内容 ${r.key}？删除后前台将不再展示。`,
    danger: true,
    run: async () => {
      busy.value = true
      const ok = await perform(
        () => (isDraft(r) ? adminApi.discardDiscover(kind.value, r.key, r.revision) : adminApi.deleteDiscover(kind.value, r.key)),
        '已删除',
      )
      busy.value = false
      if (ok) {
        editing.value = null
        await load()
      }
    },
  }
}
async function cancel(r: DiscoverDraft) {
  busy.value = true
  const ok = await perform(() => adminApi.cancelDiscover(kind.value, r.key, r.revision), '定时发布已取消，需重新审核')
  busy.value = false
  if (ok) {
    editing.value = null
    await load()
  }
}
async function importBatch() {
  busy.value = true
  const ok = await perform(async () => {
    const items = JSON.parse(importJson.value)
    if (!Array.isArray(items)) throw new Error('请填 JSON 数组')
    await adminApi.importDiscoverDrafts(items)
  }, '整批已导入草稿，等待人工审核')
  busy.value = false
  if (ok) {
    importOpen.value = false
    await showSavedDrafts()
  }
}
async function loadTokens() {
  await perform(async () => {
    tokens.value = (await adminApi.discoverTokens()).items
  })
}
async function createToken() {
  busy.value = true
  await perform(async () => {
    issuedToken.value = (
      await adminApi.createDiscoverToken({
        ...tokenForm,
        publish_kinds: tokenForm.publish_kinds.filter(k => tokenForm.kinds.includes(k)),
      })
    ).token
    await loadTokens()
  }, '令牌已创建，仅此一次显示')
  busy.value = false
}
function askRevokeToken(t: DiscoverToken) {
  confirmAction.value = {
    title: '撤销采集令牌？',
    body: `确定撤销「${t.name}」的全部采集和发布权限？撤销后无法恢复。`,
    danger: true,
    run: async () => {
      busy.value = true
      if (await perform(() => adminApi.revokeDiscoverToken(t.id), '已撤销')) {
        issuedToken.value = ''
        await loadTokens()
      }
      busy.value = false
    },
  }
}
async function copyToken() {
  await perform(() => navigator.clipboard.writeText(issuedToken.value), '已复制')
}
async function runConfirm() {
  const act = confirmAction.value
  confirmAction.value = null
  if (act) await act.run()
}

watch(form, () => {
  previewData.value = null
  current.value = null
}, { flush: 'sync' })
watch([kind, view, status, page], (value, previous) => {
  if (value[0] !== previous[0]) editing.value = null
  if (value.slice(0, 3).some((v, i) => v !== previous[i]) && page.value !== 1) {
    page.value = 1
    return
  }
  void load()
})
watch(tokensOpen, v => {
  if (v) void loadTokens()
  else issuedToken.value = ''
})
onMounted(load)
</script>

<template>
  <div class="adm-discover-grid">
    <aside class="panel adm-card adm-channels" aria-label="发现页频道">
      <button
        v-for="k in KINDS"
        :key="k.key"
        type="button"
        class="adm-channel-btn"
        :class="{ 'is-active': kind === k.key }"
        :disabled="busy"
        @click="kind = k.key"
      >
        <span>{{ k.label }}</span>
      </button>
    </aside>

    <div style="min-width: 0; display: grid; gap: 12px">
      <div class="adm-toolbar is-spread" style="margin: 0">
        <div class="adm-toolbar" style="margin: 0">
          <div class="adm-seg" role="group" aria-label="内容视图">
            <button type="button" :class="{ 'is-on': view === 'drafts' }" :disabled="busy" @click="view = 'drafts'">编辑与审核</button>
            <button type="button" :class="{ 'is-on': view === 'live' }" :disabled="busy" @click="view = 'live'">当前线上</button>
          </div>
          <select v-if="view === 'drafts'" v-model="status" class="select" aria-label="草稿状态" :disabled="busy">
            <option value="">全部状态</option>
            <option value="draft">待审核</option>
            <option value="scheduled">定时发布</option>
            <option value="published">已发布副本</option>
          </select>
        </div>
        <div class="adm-actions">
          <button class="btn btn-sm" type="button" :disabled="busy" @click="tokensOpen = true">采集令牌</button>
          <button class="btn btn-sm" type="button" :disabled="busy" @click="importOpen = true">批量导入</button>
          <button class="btn btn-sm" type="button" :disabled="busy || loading" @click="load">
            <AdminIcon name="refresh" :size="13" /> 刷新
          </button>
          <button class="btn btn-sm btn-primary" type="button" :disabled="busy || snapshotKind" @click="edit()">
            <AdminIcon name="plus" :size="13" /> 新增草稿
          </button>
        </div>
      </div>

      <p v-if="snapshotKind" class="adm-callout">
        客观榜单快照请从 <RouterLink class="adm-link" to="/admin/collection">采编与健康</RouterLink> 触发采集，再在此查看草稿或线上快照。
      </p>

      <div class="panel adm-card">
        <p v-if="loading" class="adm-empty">加载中…</p>
        <p v-else-if="loadError" class="adm-empty error" role="alert">
          {{ loadError }}
          <button class="btn btn-sm" type="button" :disabled="busy" @click="load">重新加载</button>
        </p>
        <p v-else-if="!visibleRows.length" class="adm-empty">当前频道还没有内容</p>
        <div v-else class="adm-table-scroll">
          <table class="adm-table">
            <thead>
              <tr>
                <th>条目标题与 Key</th>
                <th class="num">排序</th>
                <th>状态</th>
                <th class="num" style="width: 210px">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="r in visibleRows" :key="r.key">
                <td class="full">
                  <div class="adm-title">
                    {{ title(r) || r.key }}
                    <span v-if="r.data.sample" class="adm-tag line" style="margin-left: 6px">示例</span>
                  </div>
                  <div class="adm-sub">
                    <span class="adm-mono">{{ r.key }}</span>
                    <span v-if="isDraft(r)">rev.{{ r.revision }}</span>
                    <span v-if="isDraft(r) && r.source_name">{{ r.source_name }}</span>
                  </div>
                  <small v-if="isDraft(r) && r.last_error" class="error">{{ r.last_error }}</small>
                </td>
                <td data-label="排序" class="num">{{ r.sort_order }}</td>
                <td data-label="状态" class="shrink">
                  <span
                    class="adm-tag"
                    :class="r.status === 'published' ? 'valid' : r.status === 'scheduled' ? 'info' : 'suspect'"
                  >
                    {{ r.status === 'published' ? '已发布' : r.status === 'scheduled' ? '定时发布' : '待审核' }}
                  </span>
                  <small v-if="isDraft(r) && r.publish_after" class="adm-mono">{{ fmtTime(r.publish_after) }}</small>
                </td>
                <td class="num shrink">
                  <div class="adm-actions">
                    <button class="btn btn-sm btn-soft" type="button" :disabled="busy" @click="edit(r)">编辑与审核</button>
                    <button v-if="isDraft(r) && r.status === 'scheduled'" class="btn btn-sm" type="button" :disabled="busy" @click="cancel(r)">
                      取消定时
                    </button>
                    <button class="btn btn-sm btn-ghost" type="button" :disabled="busy" @click="askRemove(r)">
                      {{ isDraft(r) ? '丢弃' : '删除' }}
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-if="view === 'drafts'" class="adm-pager">
          <span>第 {{ page }} 页 · 共 {{ total }} 条</span>
          <div>
            <button class="btn btn-sm" type="button" :disabled="busy || loading || page <= 1" @click="page--">上一页</button>
            <button class="btn btn-sm" type="button" :disabled="busy || loading || page * 50 >= total" @click="page++">下一页</button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 编辑器右侧宽抽屉 -->
  <AdminDrawer
    :open="editing !== null"
    :title="`${editing ? '编辑与审核草稿' : '新增草稿'} · ${currentKindLabel}`"
    wide
    @close="editing = null"
  >
    <div class="adm-subtabs" style="margin-top: -6px">
      <button type="button" class="adm-subtab" :class="{ 'is-active': editorTab === 'form' }" @click="editorTab = 'form'">结构化表单</button>
      <button type="button" class="adm-subtab" :class="{ 'is-active': editorTab === 'diff' }" @click="editorTab = 'diff'">预览与线上对照</button>
      <button type="button" class="adm-subtab" :class="{ 'is-active': editorTab === 'json' }" @click="editorTab = 'json'">高级 JSON</button>
    </div>

    <form id="discover-drawer-form" @submit.prevent="save">
      <fieldset :disabled="busy" style="border: 0; padding: 0; margin: 0; min-width: 0">
        <div v-show="editorTab === 'form'" class="adm-form-grid">
          <label class="field">
            <span>稳定 key</span>
            <input v-model="form.key" class="input adm-mono" required pattern="[a-z0-9_\-]{1,60}" :disabled="!!editing" />
            <small>重复导入同 key 会更新而非新增条目</small>
          </label>
          <label class="field">
            <span>排序（小在前）</span>
            <input v-model.number="form.sort_order" class="input" type="number" />
          </label>
          <label class="field">
            <span>内容日期</span>
            <input v-model="form.published_at" class="input" type="datetime-local" />
            <small>留空沿用线上发布日期</small>
          </label>
          <label class="field">
            <span>来源名称</span>
            <input v-model="form.source_name" class="input" maxlength="100" />
          </label>
          <label class="field wide">
            <span>一手来源地址</span>
            <input v-model="form.source_url" class="input" type="url" placeholder="https://" />
          </label>
          <label class="field wide" style="display: flex; flex-direction: row; align-items: center; gap: 8px">
            <input v-model="form.sample" type="checkbox" />
            <span>标记为示例数据（客户端将显示「示例数据」提示）</span>
          </label>
          <DiscoverFieldsEditor v-model="form.json" :kind="kind" />
        </div>

        <div v-show="editorTab === 'json'" class="adm-form-grid">
          <label class="field wide">
            <span>内容 JSON（与结构化表单实时同步）</span>
            <textarea v-model="form.json" class="textarea adm-mono" rows="16" spellcheck="false" />
          </label>
        </div>

        <div v-show="editorTab === 'diff'" style="display: grid; gap: 14px">
          <div v-if="previewData" class="adm-diff-grid">
            <div>
              <div class="adm-eyebrow" style="margin-bottom: 6px; color: var(--brand)">
                校验通过的预览数据{{ current ? `（rev.${current.revision}）` : '' }}
              </div>
              <pre class="adm-code-block">{{ JSON.stringify(previewData, null, 2) }}</pre>
            </div>
            <div>
              <div class="adm-eyebrow" style="margin-bottom: 6px">当前线上版本</div>
              <pre v-if="liveData" class="adm-code-block">{{ JSON.stringify(liveData, null, 2) }}</pre>
              <p v-else class="adm-empty" style="border: 1px dashed var(--adm-line-strong); border-radius: 8px">尚无线上内容</p>
            </div>
          </div>
          <p v-else class="adm-callout">点击底部「校验并预览」或「保存草稿」后，可在此对照线上版本并执行审核发布。</p>

          <div v-if="previewData && current?.status === 'draft'" class="panel adm-card adm-card-body" style="display: grid; gap: 12px">
            <label class="field">
              <span>审核版本 {{ current.revision }} · 定时发布时间（留空立即发布）</span>
              <input v-model="publishAfter" class="input" type="datetime-local" aria-label="审核后发布时间" :disabled="busy" />
            </label>
            <div>
              <button class="btn btn-primary" type="button" :disabled="busy" @click="askReview">
                {{ publishAfter ? '审核通过并定时发布' : '审核通过并立即发布' }}
              </button>
            </div>
          </div>
          <p v-else-if="previewData" class="muted" style="margin: 0">请先点击「保存草稿」，再审核发布当前版本。</p>
        </div>
      </fieldset>
    </form>

    <template #footer>
      <button class="btn btn-ghost" type="button" @click="editing = null">关闭</button>
      <button class="btn" type="button" :disabled="busy" @click="preview">校验并预览</button>
      <button class="btn btn-primary" type="submit" form="discover-drawer-form" :disabled="busy">保存草稿</button>
    </template>
  </AdminDrawer>

  <!-- 批量导入抽屉 -->
  <AdminDrawer :open="importOpen" title="批量导入发现页草稿" @close="importOpen = false">
    <form id="discover-import-form" style="display: grid; gap: 12px" @submit.prevent="importBatch">
      <label class="field">
        <span>JSON 数组（每批 1–100 条，可跨类型）</span>
        <textarea v-model="importJson" class="textarea adm-mono" rows="12" spellcheck="false" :disabled="busy" />
        <small>新条目 expected_revision 填 0；更新填草稿版本号。任一条校验失败或版本冲突，整批不写入。</small>
      </label>
    </form>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="importOpen = false">取消</button>
      <button class="btn btn-primary" type="submit" form="discover-import-form" :disabled="busy">导入草稿</button>
    </template>
  </AdminDrawer>

  <!-- 采集令牌抽屉 -->
  <AdminDrawer :open="tokensOpen" title="发现页采集令牌" wide @close="tokensOpen = false">
    <p class="adm-callout" style="margin-bottom: 16px">
      默认只读写勾选类型的草稿；前沿与日报自动发布须单独勾选授权。令牌创建后仅展示一次，请立即保存到安全位置。
    </p>
    <form style="display: grid; gap: 14px; margin-bottom: 18px" @submit.prevent="createToken">
      <div class="adm-form-grid">
        <label class="field"><span>令牌名称</span><input v-model="tokenForm.name" class="input" required maxlength="60" :disabled="busy" /></label>
        <label class="field"><span>有效天数</span><input v-model.number="tokenForm.days" class="input" type="number" min="1" max="365" required :disabled="busy" /></label>
        <div class="field wide">
          <span>允许的内容类型</span>
          <div class="adm-checks">
            <label v-for="k in KINDS" :key="k.key"><input v-model="tokenForm.kinds" type="checkbox" :value="k.key" :disabled="busy" /> {{ k.label }}</label>
          </div>
        </div>
      </div>
      <div class="adm-checks">
        <label v-if="tokenForm.kinds.includes('frontier')"><input v-model="tokenForm.publish_kinds" type="checkbox" value="frontier" :disabled="busy" /> 允许小伴前沿自动发布已保存版本</label>
        <label v-if="tokenForm.kinds.includes('news')"><input v-model="tokenForm.publish_kinds" type="checkbox" value="news" :disabled="busy" /> 允许 AI 日报自动发布已保存版本</label>
      </div>
      <div>
        <button class="btn btn-primary" type="submit" :disabled="busy || !tokenForm.kinds.length">创建令牌</button>
      </div>
    </form>

    <div v-if="issuedToken" class="panel adm-card adm-card-body" style="margin-bottom: 16px; display: grid; gap: 8px">
      <span class="adm-eyebrow">新创建的采集令牌（仅此次显示）</span>
      <textarea class="textarea adm-mono" :value="issuedToken" readonly rows="3" aria-label="新建的采集令牌" />
      <div><button class="btn btn-sm btn-soft" type="button" @click="copyToken">复制令牌</button></div>
    </div>

    <div v-if="tokens.length" class="panel adm-card">
      <div class="adm-table-scroll">
        <table class="adm-table">
          <thead><tr><th>名称</th><th>类型</th><th>发布权限</th><th>到期</th><th>状态</th><th>操作</th></tr></thead>
          <tbody>
            <tr v-for="t in tokens" :key="t.id">
              <td data-label="名称"><strong>{{ t.name }}</strong></td>
              <td data-label="类型" class="adm-mono">{{ t.kinds.join(', ') }}</td>
              <td data-label="发布权限">{{ t.publish_kinds?.length ? `自动：${t.publish_kinds.join(', ')}` : '仅草稿' }}</td>
              <td data-label="到期" class="shrink">{{ fmtTime(t.expires_at) }}</td>
              <td data-label="状态">
                <span class="adm-tag" :class="t.revoked_at ? 'expired' : new Date(t.expires_at).getTime() <= Date.now() ? 'suspect' : 'valid'">
                  {{ t.revoked_at ? '已撤销' : new Date(t.expires_at).getTime() <= Date.now() ? '已过期' : '有效' }}
                </span>
              </td>
              <td data-label="操作">
                <button v-if="!t.revoked_at" class="btn btn-sm btn-danger" type="button" :disabled="busy" @click="askRevokeToken(t)">撤销</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <template #footer>
      <button class="btn" type="button" @click="tokensOpen = false">关闭</button>
    </template>
  </AdminDrawer>

  <AdminModal :open="!!confirmAction" :title="confirmAction?.title ?? ''" @close="confirmAction = null">
    <p v-if="confirmAction" style="margin: 0; color: var(--adm-ink-2)">{{ confirmAction.body }}</p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="confirmAction = null">取消</button>
      <button
        v-if="confirmAction"
        class="btn"
        :class="confirmAction.danger ? 'btn-danger-fill' : 'btn-primary'"
        type="button"
        :disabled="busy"
        @click="runConfirm"
      >
        确认执行
      </button>
    </template>
  </AdminModal>
</template>
