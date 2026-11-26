<script setup lang="ts">
// 精选项目工作台：4 步发布流引导条 + 236px 项目列表 + 分组表单 + 粘性 16:9 实时预览。
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { openSourceAdmin, privateAssetPreview, rasterAsset, type AssetCandidate, type OpenSourceWorkspace } from '@/api/openSourceAdmin'
import OpenSourceCover from '@/components/community/OpenSourceCover.vue'
import '@/components/community/open-source.css'
import type { OpenSourceCatalog, OpenSourceCollection } from '@/lib/openSource'
import { downloadCover, exportOpenSourceCover } from '@/lib/openSourceCoverExport'
import { openSourceCoverRules, resolveOpenSourceCover } from '@/lib/openSourceCovers'
import AdminIcon from './AdminIcon.vue'

const workspace = ref<OpenSourceWorkspace | null>(null)
const draft = ref<OpenSourceCatalog | null>(null)
const saved = ref('')
const selected = ref(0)
const search = ref('')
const repository = ref('')
const busy = ref(false)
const loading = ref(true)
const error = ref('')
const message = ref('')
const previewedRevision = ref(-1)
const candidates = ref<AssetCandidate[]>([])
const assetSources = ref<Record<string, string>>({})

const revision = computed(() => workspace.value?.revision || 0)
const dirty = computed(() => !!draft.value && JSON.stringify(draft.value) !== saved.value)
const project = computed(() => draft.value?.projects[selected.value])
const listed = computed(
  () =>
    draft.value?.projects
      .map((proj, index) => ({ project: proj, index }))
      .filter(
        item =>
          !search.value.trim() ||
          `${item.project.name} ${item.project.full_name} ${item.project.title}`
            .toLowerCase()
            .includes(search.value.trim().toLowerCase()),
      ) || [],
)
const preview = computed(() =>
  project.value
    ? resolveOpenSourceCover({
        ...project.value,
        icon_url: assetSources.value[project.value.icon_url || ''] || project.value.icon_url,
        cover_url: assetSources.value[project.value.cover_url] || project.value.cover_url,
      })
    : null,
)
const canPublish = computed(
  () =>
    !!workspace.value &&
    !dirty.value &&
    revision.value > workspace.value.published_revision &&
    previewedRevision.value === revision.value,
)
const activeTemplate = computed(() =>
  preview.value
    ? openSourceCoverRules.templates.find(
        item => item.style === preview.value!.cover_style && item.layout === preview.value!.cover_layout,
      )?.label
    : '',
)

/** 4 步进度条当前阶段：1 编辑草稿 / 2 已保存草稿 / 3 已校验可发布 / 4 已与线上同步 */
const workflowStep = computed(() => {
  if (!workspace.value) return 1
  if (dirty.value) return 1
  if (revision.value <= workspace.value.published_revision) return 4
  if (previewedRevision.value === revision.value) return 3
  return 2
})

const workflowSteps = computed(() => [
  '编辑目录草稿',
  `保存版本（v${revision.value}）`,
  '校验结构与素材',
  `发布上线（线上 v${workspace.value?.published_revision ?? 0}）`,
])

let assetGeneration = 0
function revokeAssets() {
  for (const url of Object.values(assetSources.value)) if (url.startsWith('blob:')) URL.revokeObjectURL(url)
  assetSources.value = {}
}
async function loadAssets() {
  const generation = ++assetGeneration
  const urls = [project.value?.icon_url, project.value?.cover_url].filter(
    (url): url is string => !!url && url.startsWith('/api/open-source/assets/'),
  )
  for (const [key, local] of Object.entries(assetSources.value)) {
    if (!urls.includes(key)) {
      if (local.startsWith('blob:')) URL.revokeObjectURL(local)
      delete assetSources.value[key]
    }
  }
  await Promise.all(
    urls.map(async url => {
      if (assetSources.value[url]) return
      try {
        const local = await privateAssetPreview(url)
        if (generation === assetGeneration) assetSources.value = { ...assetSources.value, [url]: local }
        else URL.revokeObjectURL(local)
      } catch (e) {
        if (generation === assetGeneration) error.value = e instanceof Error ? e.message : '素材预览加载失败'
      }
    }),
  )
}
watch(() => [project.value?.icon_url, project.value?.cover_url], loadAssets)
watch(dirty, value => {
  if (value) previewedRevision.value = -1
})
function adopt(value: OpenSourceWorkspace) {
  const name = project.value?.full_name
  workspace.value = value
  draft.value = structuredClone(value.draft)
  saved.value = JSON.stringify(draft.value)
  selected.value = Math.max(0, value.draft.projects.findIndex(item => item.full_name === name))
  previewedRevision.value = -1
}
async function act(work: () => Promise<void>) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  message.value = ''
  try {
    await work()
  } catch (e) {
    error.value = e instanceof Error ? e.message : '操作失败'
  } finally {
    busy.value = false
  }
}
async function read() {
  if (dirty.value && !window.confirm('放弃当前未保存修改并重新读取？')) return
  loading.value = true
  await act(async () => {
    adopt(await openSourceAdmin.read())
    await loadAssets()
  })
  loading.value = false
}
void read()
function pick(index: number) {
  if (busy.value) return
  selected.value = index
  candidates.value = []
  error.value = ''
  message.value = ''
}
function changeList(key: 'tags' | 'related_categories' | 'highlights' | 'notes' | 'cover_lines', value: string) {
  if (project.value) project.value[key] = value.split('\n').map(line => line.trim()).filter(Boolean)
}
function create() {
  if (!draft.value) return
  draft.value.projects.push({
    full_name: '',
    name: '',
    title: '',
    summary: '',
    category: 'dev',
    related_categories: [],
    tags: [],
    effort: '',
    cover_url: '',
    cover_caption: '',
    icon_url: '',
    homepage_url: '',
    source_url: '',
    highlights: [],
    audience: '',
    getting_started: '',
    notes: [],
    featured: true,
    order: draft.value.projects.length + 1,
    cover_template: 'auto',
    cover_variant: 'balanced',
    cover_lines: [],
  })
  pick(draft.value.projects.length - 1)
}
async function importProject() {
  if (!draft.value || !repository.value.trim()) return
  await act(async () => {
    const result = await openSourceAdmin.import(repository.value.trim())
    const exists = draft.value!.projects.findIndex(item => item.full_name.toLowerCase() === result.project.full_name.toLowerCase())
    if (exists >= 0) {
      selected.value = exists
      message.value = '项目已在目录中，保留已有内容并列出本次找到的素材。'
    } else {
      result.project.order = draft.value!.projects.length + 1
      draft.value!.projects.push(result.project)
      selected.value = draft.value!.projects.length - 1
      message.value = result.notes.join(' ')
    }
    candidates.value = result.assets
  })
}
function useCandidate(candidate: AssetCandidate, field: 'icon_url' | 'cover_url') {
  if (!project.value) return
  project.value[field] = candidate.url
  if (field === 'icon_url') project.value.icon_source_url = candidate.url
  else project.value.cover_caption = '项目 README 展示图'
}
async function cacheAsset(field: 'icon_url' | 'cover_url') {
  const item = project.value
  if (!item || !item[field] || !item.full_name) return
  await act(async () => {
    const asset = await openSourceAdmin.collectAsset(item.full_name, item[field]!)
    item[field] = asset.url
    if (field === 'icon_url') item.icon_source_url = asset.source_url
    message.value = '素材已存入精选素材库，保存并发布目录后对外可见。'
    await loadAssets()
  })
}
async function upload(event: Event, field: 'icon_url' | 'cover_url') {
  const inputEl = event.target as HTMLInputElement
  const file = inputEl.files?.[0]
  const item = project.value
  inputEl.value = ''
  if (!file || !item) return
  await act(async () => {
    if (!item.full_name) throw new Error('请先填写仓库标识')
    const source = field === 'icon_url' ? item.icon_source_url || item.source_url : item.source_url
    const asset = await openSourceAdmin.upload(item.full_name, source, await rasterAsset(file, field === 'icon_url'))
    item[field] = asset.url
    message.value = '素材已上传；未发布的素材仅管理员可预览。'
    await loadAssets()
  })
}
async function save() {
  if (draft.value) {
    await act(async () => {
      adopt(await openSourceAdmin.save(revision.value, draft.value!))
      message.value = '草稿已保存。请点击「校验预览」确认无误后再发布。'
    })
  }
}
async function validate() {
  if (draft.value) {
    await act(async () => {
      await openSourceAdmin.preview(draft.value!)
      previewedRevision.value = dirty.value ? -1 : revision.value
      message.value = dirty.value ? '校验通过，请先保存草稿。' : `版本 ${revision.value} 校验通过，可以发布。`
    })
  }
}
async function publish() {
  if (canPublish.value) {
    await act(async () => {
      adopt(await openSourceAdmin.publish(revision.value))
      message.value = '精选目录已发布，网页和手机读取同一版本。'
    })
  }
}
async function exportCover() {
  if (preview.value) {
    await act(async () => {
      downloadCover(await exportOpenSourceCover(preview.value!), preview.value!.name)
      message.value = '已导出 1280 × 720 PNG 封面。'
    })
  }
}
function remove() {
  if (!draft.value || !project.value || !window.confirm('将这个项目移出草稿？已发布内容在下次发布前不受影响。')) return
  draft.value.projects.splice(selected.value, 1)
  selected.value = Math.max(0, selected.value - 1)
}
function restoreLive() {
  if (!workspace.value || !window.confirm('用当前线上目录覆盖本地草稿？仍需保存后才会更新草稿版本。')) return
  draft.value = JSON.parse(JSON.stringify(workspace.value.live))
  selected.value = 0
  previewedRevision.value = -1
}
function addCollection() {
  draft.value?.collections.push({
    key: `collection-${Date.now()}`,
    title: '新专题',
    summary: '',
    description: '',
    projects: [],
    notes: [],
  })
}
function collectionMembers(collection: OpenSourceCollection, event: Event) {
  const previous = new Map(collection.projects.map((name, index) => [name, collection.notes[index] || '']))
  collection.projects = [...(event.target as HTMLSelectElement).selectedOptions].map(option => option.value)
  collection.notes = collection.projects.map(name => previous.get(name) || '')
}
function beforeUnload(event: BeforeUnloadEvent) {
  if (dirty.value) {
    event.preventDefault()
    event.returnValue = ''
  }
}
window.addEventListener('beforeunload', beforeUnload)
onBeforeRouteLeave(() => !dirty.value || window.confirm('精选草稿尚未保存，离开此页？'))
onBeforeUnmount(() => {
  ++assetGeneration
  revokeAssets()
  window.removeEventListener('beforeunload', beforeUnload)
})
</script>

<template>
  <section class="curation-panel">
    <div class="adm-toolbar is-spread" style="margin: 0">
      <div class="adm-tags">
        <span class="adm-tag brand">仅精选板块</span>
        <span class="adm-tag" :class="dirty ? 'suspect' : 'valid'">
          {{ dirty ? '有未保存修改' : revision ? `草稿版本 v${revision}` : '尚未保存' }}
        </span>
        <span v-if="workspace" class="adm-tag line">线上版本 v{{ workspace.published_revision }}</span>
      </div>
      <div class="adm-actions">
        <button class="btn btn-sm btn-ghost" type="button" :disabled="busy" @click="read">
          <AdminIcon name="refresh" :size="13" /> 重新读取
        </button>
        <button class="btn btn-sm" type="button" :disabled="busy || !draft" @click="save">保存草稿</button>
        <button class="btn btn-sm" type="button" :disabled="busy || !draft" @click="validate">校验预览</button>
        <button class="btn btn-sm btn-primary" type="button" :disabled="busy || !canPublish" @click="publish">发布此版本</button>
      </div>
    </div>

    <div class="panel adm-stepper">
      <template v-for="(label, idx) in workflowSteps" :key="label">
        <div class="adm-step" :class="{ 'is-done': workflowStep > idx + 1 || (workflowStep === 4 && idx === 3), 'is-current': workflowStep === idx + 1 && workflowStep < 4 }">
          <span class="adm-step-num">{{ workflowStep > idx + 1 || (workflowStep === 4 && idx === 3) ? '✓' : idx + 1 }}</span>
          <span>{{ label }}</span>
        </div>
        <div v-if="idx < workflowSteps.length - 1" class="adm-step-sep" />
      </template>
    </div>

    <p v-if="error" class="cur-error" role="alert">{{ error }}</p>
    <p v-if="message" class="cur-message" role="status">{{ message }}</p>
    <p v-if="loading" class="muted">正在读取精选工作台…</p>

    <template v-else-if="draft">
      <div class="cur-import">
        <input v-model="repository" class="input" placeholder="输入 GitHub 仓库地址或 owner/repo 获取项目资料与 README 素材" aria-label="导入 GitHub 仓库" :disabled="busy" />
        <button class="btn" type="button" :disabled="busy || !repository.trim()" @click="importProject">获取项目资料</button>
        <button class="btn btn-ghost" type="button" :disabled="busy" @click="create">
          <AdminIcon name="plus" :size="14" /> 手工新增
        </button>
      </div>

      <div class="cur-layout">
        <aside class="panel adm-card cur-list">
          <input v-model="search" class="input is-sm" placeholder="查找草稿项目…" aria-label="查找草稿项目" />
          <div class="cur-list-scroll">
            <button
              v-for="item in listed"
              :key="item.index"
              type="button"
              :class="{ on: selected === item.index }"
              :disabled="busy"
              @click="pick(item.index)"
            >
              <span class="cur-avatar" :style="{ background: item.project.cover_accent || 'var(--brand-fill)' }">
                {{ (item.project.name || item.project.full_name || '?').slice(0, 2) }}
              </span>
              <span class="cur-list-copy">
                <strong>{{ item.project.name || '未命名项目' }}<em v-if="item.project.featured"> ★</em></strong>
                <small>{{ item.project.full_name || '待填写仓库' }}</small>
              </span>
            </button>
            <p v-if="!listed.length" class="adm-empty" style="padding: 20px">暂无项目</p>
          </div>
        </aside>

        <div v-if="project" class="cur-editor">
          <fieldset :disabled="busy" style="display: grid; gap: 14px">
            <section class="panel adm-card">
              <div class="adm-card-head">
                <h3>项目内容</h3>
                <button class="btn btn-ghost btn-sm btn-danger" type="button" @click="remove">移出草稿</button>
              </div>
              <div class="adm-card-body cur-fields">
                <label class="field"><span>仓库标识</span><input v-model="project.full_name" class="input adm-mono" placeholder="owner/repo" maxlength="140" /></label>
                <label class="field"><span>项目名称</span><input v-model="project.name" class="input" maxlength="100" /></label>
                <label class="field full"><span>中文推荐标题</span><input v-model="project.title" class="input" maxlength="200" /></label>
                <label class="field">
                  <span>主分类</span>
                  <select v-model="project.category" class="select" style="width: 100%">
                    <option v-for="category in draft.categories" :key="category.key" :value="category.key">{{ category.label }}</option>
                  </select>
                </label>
                <label class="field"><span>排序</span><input v-model.number="project.order" class="input" type="number" min="0" max="100000" /></label>
                <label class="full cur-check"><input v-model="project.featured" type="checkbox" /> 出现在推荐页</label>
                <label class="field"><span>标签（每行一个）</span><textarea class="textarea" rows="3" :value="project.tags.join('\n')" @input="changeList('tags', ($event.target as HTMLTextAreaElement).value)" /></label>
                <label class="field">
                  <span>关联分类（最多两个）</span>
                  <select v-model="project.related_categories" class="select" style="width: 100%; height: auto" multiple size="4">
                    <option v-for="category in draft.categories.filter(c => c.key !== project!.category)" :key="category.key" :value="category.key">{{ category.label }}</option>
                  </select>
                </label>
                <label class="field full"><span>简短介绍（详情与搜索使用）</span><textarea v-model="project.summary" class="textarea" rows="2" maxlength="1000" /></label>
                <label class="field"><span>项目官网</span><input v-model="project.homepage_url" class="input" type="url" /></label>
                <label class="field"><span>资料来源</span><input v-model="project.source_url" class="input" type="url" placeholder="https://github.com/owner/repo#readme" /></label>

                <details class="full adm-advanced">
                  <summary>展开编辑完整导读（推荐理由 / 适合谁 / 入门方式 / 注意事项）</summary>
                  <div class="cur-fields" style="margin-top: 12px">
                    <label class="field full"><span>推荐理由（每行一条）</span><textarea class="textarea" rows="3" :value="project.highlights.join('\n')" @input="changeList('highlights', ($event.target as HTMLTextAreaElement).value)" /></label>
                    <label class="field full"><span>适合谁</span><textarea v-model="project.audience" class="textarea" rows="2" /></label>
                    <label class="field full"><span>从哪里开始</span><textarea v-model="project.getting_started" class="textarea" rows="2" /></label>
                    <label class="field"><span>使用门槛（仅详情展示）</span><input v-model="project.effort" class="input" /></label>
                    <label class="field full"><span>注意事项（每行一条）</span><textarea class="textarea" rows="3" :value="project.notes.join('\n')" @input="changeList('notes', ($event.target as HTMLTextAreaElement).value)" /></label>
                  </div>
                </details>
              </div>
            </section>

            <section class="panel adm-card">
              <div class="adm-card-head">
                <h3>封面模板与素材</h3>
                <span class="adm-tag line">1280 × 720</span>
              </div>
              <div class="adm-card-body cur-fields">
                <label class="field">
                  <span>封面模板</span>
                  <select v-model="project.cover_template" class="select" style="width: 100%">
                    <option value="auto">自动匹配</option>
                    <option v-for="preset in openSourceCoverRules.templates" :key="preset.id" :value="preset.id">{{ preset.label }}</option>
                  </select>
                </label>
                <label class="field">
                  <span>构图</span>
                  <select v-model="project.cover_variant" class="select" style="width: 100%">
                    <option v-for="variant in openSourceCoverRules.variants" :key="variant.id" :value="variant.id">{{ variant.label }}</option>
                  </select>
                </label>
                <label class="field"><span>品牌点缀色</span><input :value="project.cover_accent || ''" class="input adm-mono" placeholder="留空自动匹配" @input="project.cover_accent = ($event.target as HTMLInputElement).value" /></label>
                <label class="field">
                  <span>图标裁切</span>
                  <select v-model="project.icon_fit" class="select" style="width: 100%">
                    <option value="contain">完整显示</option>
                    <option value="mark-left">左侧标志（横向 Logo）</option>
                  </select>
                </label>
                <label class="field full"><span>示意内容（最多三行）</span><textarea class="textarea" rows="3" :value="(project.cover_lines || []).join('\n')" @input="changeList('cover_lines', ($event.target as HTMLTextAreaElement).value)" /></label>
                <label class="field"><span>照片水平焦点</span><input v-model.number="project.cover_focus_x" type="range" min="0" max="100" /></label>
                <label class="field"><span>照片垂直焦点</span><input v-model.number="project.cover_focus_y" type="range" min="0" max="100" /></label>

                <div class="full cur-asset">
                  <label class="field"><span>项目 Logo</span><input v-model="project.icon_url" class="input" placeholder="站内素材或公开 HTTPS 图片" /></label>
                  <label class="field"><span>Logo 来源</span><input v-model="project.icon_source_url" class="input" type="url" /></label>
                  <div class="adm-actions">
                    <label class="btn btn-sm cur-upload">上传 Logo<input type="file" accept="image/png,image/jpeg,image/webp,image/svg+xml,image/x-icon" @change="upload($event, 'icon_url')" /></label>
                    <button type="button" class="btn btn-sm btn-ghost" :disabled="!project.icon_url?.startsWith('https://')" @click="cacheAsset('icon_url')">采集为站内素材</button>
                  </div>
                </div>

                <div class="full cur-asset">
                  <label class="field"><span>截图或实物照片</span><input v-model="project.cover_url" class="input" placeholder="没有图片时使用示意模板" /></label>
                  <label class="field"><span>配图说明</span><input v-model="project.cover_caption" class="input" /></label>
                  <div class="adm-actions">
                    <label class="btn btn-sm cur-upload">上传配图<input type="file" accept="image/png,image/jpeg,image/webp" @change="upload($event, 'cover_url')" /></label>
                    <button type="button" class="btn btn-sm btn-ghost" :disabled="!project.cover_url.startsWith('https://')" @click="cacheAsset('cover_url')">采集为站内素材</button>
                  </div>
                </div>

                <section v-if="candidates.length" class="full cur-candidates">
                  <h3>README 中找到的候选素材</h3>
                  <div v-for="candidate in candidates" :key="candidate.url">
                    <span>{{ candidate.label || candidate.role }}<small>{{ candidate.url }}</small></span>
                    <button type="button" class="btn btn-sm" @click="useCandidate(candidate, 'icon_url')">用作 Logo</button>
                    <button type="button" class="btn btn-sm" @click="useCandidate(candidate, 'cover_url')">用作配图</button>
                  </div>
                </section>
              </div>
            </section>
          </fieldset>
        </div>
        <p v-else class="panel adm-card adm-empty">新增或选择一个项目开始编辑。</p>

        <aside v-if="preview" class="panel adm-card cur-preview">
          <div class="adm-card-head">
            <h3>实时预览</h3>
            <span class="adm-tag valid">{{ activeTemplate || '自动' }}</span>
          </div>
          <div class="adm-card-body">
            <article class="os-project-card">
              <OpenSourceCover :project="preview" />
              <div class="os-card-copy">
                <h3>{{ preview.title || preview.name || '项目标题' }}</h3>
                <div class="os-tags"><span v-for="tag in preview.tags.slice(0, 2)" :key="tag">{{ tag }}</span></div>
              </div>
            </article>
            <p class="adm-sub" style="margin: 10px 0 14px">1280 × 720 固定模板排版，照片与截图保留素材身份。</p>
            <button class="btn" type="button" style="width: 100%" :disabled="busy" @click="exportCover">导出 PNG 封面</button>
            <details style="margin-top: 12px; font-size: 12.5px">
              <summary style="cursor: pointer; color: var(--adm-ink-2)">仅查看图标和配图原素材</summary>
              <div class="cur-raw">
                <img v-if="preview.icon_url" :src="preview.icon_url" alt="Logo 原素材" />
                <img v-if="preview.cover_url" :src="preview.cover_url" alt="配图原素材" />
              </div>
            </details>
          </div>
        </aside>
      </div>

      <details class="panel adm-card cur-details">
        <summary>专题编排（{{ draft.collections.length }}）</summary>
        <fieldset :disabled="busy" class="cur-collections">
          <section v-for="(collection, index) in draft.collections" :key="index">
            <div class="cur-section-head">
              <h3>{{ collection.title }}</h3>
              <button class="btn btn-ghost btn-sm btn-danger" type="button" @click="draft.collections.splice(index, 1)">移出草稿</button>
            </div>
            <div class="cur-fields">
              <label class="field"><span>标识</span><input v-model="collection.key" class="input adm-mono" /></label>
              <label class="field"><span>标题</span><input v-model="collection.title" class="input" /></label>
              <label class="field full"><span>简短说明</span><input v-model="collection.summary" class="input" /></label>
              <label class="field full"><span>专题导语</span><textarea v-model="collection.description" class="textarea" rows="3" /></label>
              <label class="field full">
                <span>包含项目</span>
                <select class="select" style="width: 100%; height: auto" multiple size="5" :value="collection.projects" @change="collectionMembers(collection, $event)">
                  <option v-for="item in draft.projects" :key="item.full_name" :value="item.full_name">{{ item.name }} · {{ item.full_name }}</option>
                </select>
              </label>
              <label v-for="(name, noteIndex) in collection.projects" :key="name" class="field full">
                <span>{{ name }} 的点评</span>
                <input v-model="collection.notes[noteIndex]" class="input" />
              </label>
            </div>
          </section>
          <div><button class="btn" type="button" @click="addCollection">新增专题</button></div>
        </fieldset>
      </details>

      <div class="cur-footer">
        <button class="btn btn-ghost btn-sm" type="button" :disabled="busy" @click="restoreLive">以线上目录覆盖本地草稿</button>
        <span class="muted">本工作台仅更新精选项目、专题与封面，不影响其他社区板块。</span>
      </div>
    </template>
  </section>
</template>

<style scoped>
.curation-panel { display: grid; gap: 14px; }
.cur-import { display: flex; gap: 8px; flex-wrap: wrap; }
.cur-import input { flex: 1; min-width: 240px; }
.cur-layout { display: grid; grid-template-columns: 236px minmax(0, 1fr) 308px; gap: 16px; align-items: start; }
.cur-list { padding: 10px; display: grid; gap: 8px; position: sticky; top: 68px; }
.cur-list-scroll { display: grid; gap: 2px; max-height: 68vh; overflow: auto; }
.cur-list button { display: flex; align-items: center; gap: 9px; text-align: left; border: 0; border-radius: 8px; padding: 9px 10px; background: transparent; color: var(--adm-ink); cursor: pointer; }
.cur-list button:hover { background: var(--adm-subtle); }
.cur-list button.on { background: var(--adm-brand-soft); }
.cur-avatar { width: 32px; height: 32px; border-radius: 8px; flex: none; display: grid; place-items: center; font-weight: 700; font-size: 12px; color: #fff; }
.cur-list-copy { min-width: 0; flex: 1; display: grid; gap: 2px; }
.cur-list-copy strong { font-size: 13px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cur-list-copy strong em { font-style: normal; color: var(--adm-warn); }
.cur-list-copy small { font-size: 11px; color: var(--adm-ink-3); font-family: var(--font-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cur-editor fieldset, .cur-collections { border: 0; margin: 0; padding: 0; min-width: 0; }
.cur-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.cur-fields .full { grid-column: 1 / -1; }
.cur-check { display: inline-flex; align-items: center; gap: 8px; font-size: 13px; }
.cur-asset { display: grid; gap: 10px; padding: 14px; background: var(--adm-subtle); border: 1px solid var(--adm-line); border-radius: 10px; }
.cur-upload { position: relative; overflow: hidden; display: inline-flex; }
.cur-upload input { position: absolute; inset: 0; opacity: 0; cursor: pointer; width: 100%; }
.cur-preview { position: sticky; top: 68px; min-width: 0; }
.cur-raw { display: grid; gap: 10px; padding-top: 10px; }
.cur-raw img { max-width: 100%; max-height: 200px; object-fit: contain; background: var(--adm-subtle); border-radius: 8px; }
.cur-candidates h3 { font-size: 13.5px; margin: 0 0 8px; }
.cur-candidates > div { display: flex; align-items: center; gap: 8px; padding: 10px 0; border-top: 1px solid var(--adm-line-subtle); }
.cur-candidates > div > span { flex: 1; min-width: 0; font-size: 12.5px; }
.cur-candidates small { display: block; overflow-wrap: anywhere; color: var(--adm-ink-3); font-size: 11.5px; margin-top: 3px; }
.cur-error { color: var(--adm-danger); padding: 12px 14px; background: var(--adm-danger-soft); border-radius: 10px; margin: 0; font-size: 13px; }
.cur-message { color: var(--adm-ok); padding: 12px 14px; background: var(--adm-ok-soft); border-radius: 10px; margin: 0; font-size: 13px; }
.cur-details { padding: 14px 18px; }
.cur-details summary { cursor: pointer; font-weight: 600; font-size: 14px; }
.cur-collections { display: grid; gap: 16px; margin-top: 14px; }
.cur-collections > section { padding: 16px; border: 1px solid var(--adm-line); border-radius: 10px; }
.cur-section-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.cur-section-head h3 { margin: 0; font-size: 14px; }
.cur-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; padding-top: 8px; font-size: 12px; }

@media (max-width: 1200px) {
  .cur-layout { grid-template-columns: 210px minmax(0, 1fr); }
  .cur-preview { grid-column: 2; position: static; max-width: 480px; }
}
@media (max-width: 760px) {
  .cur-layout { grid-template-columns: minmax(0, 1fr); }
  .cur-list { position: static; }
  .cur-list-scroll { max-height: 210px; }
  .cur-preview { grid-column: 1; }
  .cur-fields { grid-template-columns: 1fr; }
}
</style>
