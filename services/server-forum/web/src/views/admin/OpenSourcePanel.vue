<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { openSourceAdmin, privateAssetPreview, rasterAsset, type AssetCandidate, type OpenSourceWorkspace } from '@/api/openSourceAdmin'
import { openSourceCoverRules, resolveOpenSourceCover } from '@/lib/openSourceCovers'
import type { OpenSourceCatalog, OpenSourceProject, OpenSourceCollection } from '@/lib/openSource'
import { exportOpenSourceCover, downloadCover } from '@/lib/openSourceCoverExport'
import { loadOpenSourceCatalog } from '@/stores/openSource'
import OpenSourceCover from '@/components/community/OpenSourceCover.vue'
import '@/components/community/open-source.css'

const workspace = ref<OpenSourceWorkspace | null>(null), draft = ref<OpenSourceCatalog | null>(null)
const saved = ref(''), selected = ref(0), search = ref(''), repository = ref('')
const busy = ref(false), loading = ref(true), error = ref(''), message = ref(''), previewedRevision = ref(-1)
const candidates = ref<AssetCandidate[]>([]), assetSources = ref<Record<string, string>>({})
const revision = computed(() => workspace.value?.revision || 0)
const dirty = computed(() => !!draft.value && JSON.stringify(draft.value) !== saved.value)
const project = computed(() => draft.value?.projects[selected.value])
const listed = computed(() => draft.value?.projects.map((project, index) => ({ project, index })).filter(item => !search.value.trim() || `${item.project.name} ${item.project.full_name} ${item.project.title}`.toLowerCase().includes(search.value.trim().toLowerCase())) || [])
const preview = computed(() => project.value ? resolveOpenSourceCover({ ...project.value, icon_url: assetSources.value[project.value.icon_url || ''] || project.value.icon_url, cover_url: assetSources.value[project.value.cover_url] || project.value.cover_url }) : null)
const canPublish = computed(() => !!workspace.value && !dirty.value && revision.value > workspace.value.published_revision && previewedRevision.value === revision.value)
const activeTemplate = computed(() => preview.value ? openSourceCoverRules.templates.find(item => item.style === preview.value!.cover_style && item.layout === preview.value!.cover_layout)?.label : '')
const badge = computed(() => dirty.value ? '有未保存修改' : revision.value ? `草稿版本 ${revision.value}` : '尚未保存')
let assetGeneration = 0
function revokeAssets() { for (const url of Object.values(assetSources.value)) if (url.startsWith('blob:')) URL.revokeObjectURL(url); assetSources.value = {} }
async function loadAssets() {
  const generation = ++assetGeneration
  const urls = [project.value?.icon_url, project.value?.cover_url].filter((url): url is string => !!url && url.startsWith('/api/open-source/assets/'))
  for (const [key, local] of Object.entries(assetSources.value)) { if (!urls.includes(key)) { if (local.startsWith('blob:')) URL.revokeObjectURL(local); delete assetSources.value[key] } }
  await Promise.all(urls.map(async url => {
    if (assetSources.value[url]) return
    try { const local = await privateAssetPreview(url); if (generation === assetGeneration) assetSources.value = { ...assetSources.value, [url]: local }; else URL.revokeObjectURL(local) }
    catch (e) { if (generation === assetGeneration) error.value = e instanceof Error ? e.message : '素材预览加载失败' }
  }))
}
watch(() => [project.value?.icon_url, project.value?.cover_url], loadAssets)
watch(dirty, value => { if (value) previewedRevision.value = -1 })
function adopt(value: OpenSourceWorkspace) {
  const name = project.value?.full_name
  workspace.value = value; draft.value = structuredClone(value.draft); saved.value = JSON.stringify(draft.value)
  selected.value = Math.max(0, value.draft.projects.findIndex(item => item.full_name === name)); previewedRevision.value = -1
}
async function act(work: () => Promise<void>) {
  if (busy.value) return
  busy.value = true; error.value = ''; message.value = ''
  try { await work() } catch (e) { error.value = e instanceof Error ? e.message : '操作失败' } finally { busy.value = false }
}
async function read() {
  if (dirty.value && !window.confirm('放弃当前未保存修改并重新读取？')) return
  loading.value = true
  await act(async () => { adopt(await openSourceAdmin.read()); await loadAssets() })
  loading.value = false
}
void read()
function pick(index: number) { if (busy.value) return; selected.value = index; candidates.value = []; error.value = ''; message.value = '' }
function changeList(key: 'tags' | 'related_categories' | 'highlights' | 'notes' | 'cover_lines', value: string) {
  if (project.value) project.value[key] = value.split('\n').map(line => line.trim()).filter(Boolean)
}
function create() {
  if (!draft.value) return
  draft.value.projects.push({ full_name: '', name: '', title: '', summary: '', category: 'dev', related_categories: [], tags: [], effort: '', cover_url: '', cover_caption: '', icon_url: '', homepage_url: '', source_url: '', highlights: [], audience: '', getting_started: '', notes: [], featured: true, order: draft.value.projects.length + 1, cover_template: 'auto', cover_variant: 'balanced', cover_lines: [] })
  pick(draft.value.projects.length - 1)
}
async function importProject() {
  if (!draft.value || !repository.value.trim()) return
  await act(async () => {
    const result = await openSourceAdmin.import(repository.value.trim())
    const exists = draft.value!.projects.findIndex(item => item.full_name.toLowerCase() === result.project.full_name.toLowerCase())
    if (exists >= 0) { selected.value = exists; message.value = '项目已在目录中，保留已有内容并列出本次找到的素材。' }
    else { result.project.order = draft.value!.projects.length + 1; draft.value!.projects.push(result.project); selected.value = draft.value!.projects.length - 1; message.value = result.notes.join(' ') }
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
  const item = project.value; if (!item || !item[field] || !item.full_name) return
  await act(async () => {
    const asset = await openSourceAdmin.collectAsset(item.full_name, item[field]!)
    item[field] = asset.url
    if (field === 'icon_url') item.icon_source_url = asset.source_url
    message.value = '素材已存入精选素材库，保存并发布目录后对外可见。'; await loadAssets()
  })
}
async function upload(event: Event, field: 'icon_url' | 'cover_url') {
  const input = event.target as HTMLInputElement, file = input.files?.[0], item = project.value
  input.value = ''
  if (!file || !item) return
  await act(async () => {
    if (!item.full_name) throw new Error('请先填写仓库标识')
    const source = field === 'icon_url' ? item.icon_source_url || item.source_url : item.source_url
    const asset = await openSourceAdmin.upload(item.full_name, source, await rasterAsset(file, field === 'icon_url'))
    item[field] = asset.url; message.value = '素材已上传；未发布的素材仅管理员可预览。'; await loadAssets()
  })
}
async function save() { if (draft.value) await act(async () => { adopt(await openSourceAdmin.save(revision.value, draft.value!)); message.value = '草稿已保存。确认预览后再发布。' }) }
async function validate() { if (draft.value) await act(async () => { await openSourceAdmin.preview(draft.value!); previewedRevision.value = dirty.value ? -1 : revision.value; message.value = dirty.value ? '校验通过，请先保存草稿。' : `版本 ${revision.value} 校验通过，可以发布。` }) }
async function publish() { if (canPublish.value) await act(async () => { adopt(await openSourceAdmin.publish(revision.value)); message.value = '精选目录已发布，网页和手机读取同一版本。'; await loadOpenSourceCatalog(true) }) }
async function exportCover() { if (preview.value) await act(async () => { downloadCover(await exportOpenSourceCover(preview.value!), preview.value!.name); message.value = '已导出 1280 × 720 PNG 封面。' }) }
function remove() {
  if (!draft.value || !project.value || !window.confirm('将这个项目移出草稿？已发布内容在下次发布前不受影响。')) return
  draft.value.projects.splice(selected.value, 1); selected.value = Math.max(0, selected.value - 1)
}
function restoreLive() {
  if (!workspace.value || !window.confirm('用当前线上目录覆盖本地草稿？仍需保存后才会更新草稿版本。')) return
  draft.value = JSON.parse(JSON.stringify(workspace.value.live)); selected.value = 0; previewedRevision.value = -1
}
function addCollection() { draft.value?.collections.push({ key: `collection-${Date.now()}`, title: '新专题', summary: '', description: '', projects: [], notes: [] }) }
function collectionMembers(collection: OpenSourceCollection, event: Event) {
  const previous = new Map(collection.projects.map((name, index) => [name, collection.notes[index] || '']))
  collection.projects = [...(event.target as HTMLSelectElement).selectedOptions].map(option => option.value)
  collection.notes = collection.projects.map(name => previous.get(name) || '')
}
function beforeUnload(event: BeforeUnloadEvent) { if (dirty.value) { event.preventDefault(); event.returnValue = '' } }
window.addEventListener('beforeunload', beforeUnload)
onBeforeRouteLeave(() => !dirty.value || window.confirm('精选草稿尚未保存，离开此页？'))
onBeforeUnmount(() => { ++assetGeneration; revokeAssets(); window.removeEventListener('beforeunload', beforeUnload) })
</script>

<template>
  <section class="curation-panel">
    <div class="cur-toolbar"><div><span class="cur-badge">仅精选板块</span><span class="muted">{{ badge }}<template v-if="workspace"> · 线上版本 {{ workspace.published_revision }}</template></span></div><div class="cur-actions"><button class="btn btn-ghost" :disabled="busy" @click="read">重新读取</button><button class="btn" :disabled="busy || !draft" @click="save">保存草稿</button><button class="btn" :disabled="busy || !draft" @click="validate">校验预览</button><button class="btn btn-primary" :disabled="busy || !canPublish" @click="publish">发布此版本</button></div></div>
    <p class="cur-scope muted">固定模板与自动匹配只用于精选项目，不改变资讯、作品和讨论的封面规则。保存草稿后，校验预览并发布，网页与手机才会更新。</p>
    <p v-if="error" class="cur-error" role="alert">{{ error }}</p><p v-if="message" class="cur-message" role="status">{{ message }}</p>
    <p v-if="loading" class="muted">正在读取精选工作台…</p>
    <template v-else-if="draft">
      <div class="cur-import"><input v-model="repository" class="input" placeholder="GitHub 仓库地址或 owner/repo" aria-label="导入 GitHub 仓库" :disabled="busy"><button class="btn" :disabled="busy || !repository.trim()" @click="importProject">获取项目资料</button><button class="btn btn-ghost" :disabled="busy" @click="create">手工新增</button></div>
      <div class="cur-layout"><aside class="cur-list"><input v-model="search" class="input" placeholder="查找项目" aria-label="查找草稿项目"><button v-for="item in listed" :key="item.index" type="button" :class="{on:selected===item.index}" :disabled="busy" @click="pick(item.index)"><strong>{{ item.project.name || '未命名项目' }}</strong><small>{{ item.project.full_name || '待填写仓库' }}</small></button><p v-if="!listed.length" class="muted">暂无项目</p></aside>
        <div v-if="project" class="cur-editor"><fieldset :disabled="busy"><div class="cur-section-head"><h2>项目内容</h2><button class="btn btn-ghost btn-sm" type="button" @click="remove">移出草稿</button></div>
          <div class="cur-fields"><label>仓库标识<input v-model="project.full_name" class="input" placeholder="owner/repo" maxlength="140"></label><label>项目名称<input v-model="project.name" class="input" maxlength="100"></label><label class="full">中文推荐标题<input v-model="project.title" class="input" maxlength="200"></label><label>主分类<select v-model="project.category" class="input"><option v-for="category in draft.categories" :key="category.key" :value="category.key">{{ category.label }}</option></select></label><label>排序<input v-model.number="project.order" class="input" type="number" min="0" max="100000"></label><label class="full cur-check"><input v-model="project.featured" type="checkbox">出现在推荐页</label><label>标签（每行一个）<textarea class="input" rows="3" :value="project.tags.join('\n')" @input="changeList('tags',($event.target as HTMLTextAreaElement).value)" /></label><label>关联分类（最多两个）<select v-model="project.related_categories" class="input" multiple size="4"><option v-for="category in draft.categories.filter(c=>c.key!==project!.category)" :key="category.key" :value="category.key">{{ category.label }}</option></select></label><label class="full">简短介绍（详情与搜索使用）<textarea v-model="project.summary" class="input" rows="2" maxlength="1000" /></label><label>项目官网<input v-model="project.homepage_url" class="input" type="url"></label><label>资料来源<input v-model="project.source_url" class="input" type="url" placeholder="https://github.com/owner/repo#readme"></label></div>
          <details class="cur-details"><summary>编辑完整导读</summary><div class="cur-fields"><label class="full">推荐理由（每行一条）<textarea class="input" rows="3" :value="project.highlights.join('\n')" @input="changeList('highlights',($event.target as HTMLTextAreaElement).value)" /></label><label class="full">适合谁<textarea v-model="project.audience" class="input" rows="2" /></label><label class="full">从哪里开始<textarea v-model="project.getting_started" class="input" rows="2" /></label><label>使用门槛（仅详情展示）<input v-model="project.effort" class="input"></label><label>注意事项（每行一条）<textarea class="input" rows="3" :value="project.notes.join('\n')" @input="changeList('notes',($event.target as HTMLTextAreaElement).value)" /></label></div></details>
          <h2>封面与素材</h2><div class="cur-fields"><label>封面模板<select v-model="project.cover_template" class="input"><option value="auto">自动匹配</option><option v-for="preset in openSourceCoverRules.templates" :key="preset.id" :value="preset.id">{{ preset.label }}</option></select></label><label>构图<select v-model="project.cover_variant" class="input"><option v-for="variant in openSourceCoverRules.variants" :key="variant.id" :value="variant.id">{{ variant.label }}</option></select></label><label>品牌点缀色<input :value="project.cover_accent || ''" class="input" placeholder="留空自动匹配" @input="project.cover_accent=($event.target as HTMLInputElement).value"></label><label>图标裁切<select v-model="project.icon_fit" class="input"><option value="contain">完整显示</option><option value="mark-left">左侧标志（横向 Logo）</option></select></label><label class="full">示意内容（最多三行）<textarea class="input" rows="3" :value="(project.cover_lines||[]).join('\n')" @input="changeList('cover_lines',($event.target as HTMLTextAreaElement).value)" /></label><label>照片水平焦点<input v-model.number="project.cover_focus_x" class="input" type="range" min="0" max="100"></label><label>照片垂直焦点<input v-model.number="project.cover_focus_y" class="input" type="range" min="0" max="100"></label>
          <div class="full cur-asset"><label>项目 Logo<input v-model="project.icon_url" class="input" placeholder="站内素材或公开 HTTPS 图片"></label><label>Logo 来源<input v-model="project.icon_source_url" class="input" type="url"></label><div class="cur-actions"><label class="btn cur-upload">上传 Logo<input type="file" accept="image/png,image/jpeg,image/webp,image/svg+xml,image/x-icon" @change="upload($event,'icon_url')"></label><button type="button" class="btn btn-ghost" :disabled="!project.icon_url?.startsWith('https://')" @click="cacheAsset('icon_url')">采集为站内素材</button></div></div>
          <div class="full cur-asset"><label>截图或实物照片<input v-model="project.cover_url" class="input" placeholder="没有图片时使用示意模板"></label><label>配图说明<input v-model="project.cover_caption" class="input"></label><div class="cur-actions"><label class="btn cur-upload">上传配图<input type="file" accept="image/png,image/jpeg,image/webp" @change="upload($event,'cover_url')"></label><button type="button" class="btn btn-ghost" :disabled="!project.cover_url.startsWith('https://')" @click="cacheAsset('cover_url')">采集为站内素材</button></div></div></div>
          <section v-if="candidates.length" class="cur-candidates"><h3>README 中找到的素材</h3><div v-for="candidate in candidates" :key="candidate.url"><span>{{ candidate.label || candidate.role }}<small>{{ candidate.url }}</small></span><button type="button" class="btn btn-sm" @click="useCandidate(candidate,'icon_url')">用作 Logo</button><button type="button" class="btn btn-sm" @click="useCandidate(candidate,'cover_url')">用作配图</button></div></section>
        </fieldset></div><p v-else class="muted">新增或选择一个项目开始编辑。</p>
        <aside v-if="preview" class="cur-preview"><div class="cur-section-head"><h2>实时预览</h2><span class="muted">{{ activeTemplate }}</span></div><article class="os-project-card"><OpenSourceCover :project="preview" /><div class="os-card-copy"><h3>{{ preview.title || preview.name || '项目标题' }}</h3><div class="os-tags"><span v-for="tag in preview.tags.slice(0,2)" :key="tag">{{ tag }}</span></div></div></article><p class="muted">1280 × 720 · 固定模板排版，无需模型生图。照片、截图保留素材身份，示意明确标注。</p><button class="btn" :disabled="busy" @click="exportCover">导出 PNG 封面</button><details><summary>仅查看图标和配图</summary><div class="cur-raw"><img v-if="preview.icon_url" :src="preview.icon_url" alt="Logo 原素材"><img v-if="preview.cover_url" :src="preview.cover_url" alt="配图原素材"></div></details></aside>
      </div>
      <details class="cur-details"><summary>专题编排（{{ draft.collections.length }}）</summary><fieldset :disabled="busy" class="cur-collections"><section v-for="(collection,index) in draft.collections" :key="index"><div class="cur-section-head"><h3>{{ collection.title }}</h3><button class="btn btn-ghost btn-sm" @click="draft.collections.splice(index,1)">移出草稿</button></div><div class="cur-fields"><label>标识<input v-model="collection.key" class="input"></label><label>标题<input v-model="collection.title" class="input"></label><label class="full">简短说明<input v-model="collection.summary" class="input"></label><label class="full">专题导语<textarea v-model="collection.description" class="input" rows="3" /></label><label class="full">包含项目<select class="input" multiple size="5" :value="collection.projects" @change="collectionMembers(collection,$event)"><option v-for="item in draft.projects" :key="item.full_name" :value="item.full_name">{{ item.name }} · {{ item.full_name }}</option></select></label><label v-for="(name,noteIndex) in collection.projects" :key="name" class="full">{{ name }} 的点评<input v-model="collection.notes[noteIndex]" class="input"></label></div></section><button class="btn" @click="addCollection">新增专题</button></fieldset></details>
      <div class="cur-footer"><button class="btn btn-ghost" :disabled="busy" @click="restoreLive">以线上目录覆盖本地草稿</button><span class="muted">本工作台仅更新精选项目、专题与封面。</span></div>
    </template>
  </section>
</template>

<style scoped>
.cur-toolbar,.cur-actions,.cur-section-head,.cur-footer{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap}.cur-toolbar>div{display:flex;align-items:center;gap:12px;flex-wrap:wrap}.cur-badge{font-size:12px;background:var(--surface-2);padding:6px 10px;border-radius:99px}.cur-scope{font-size:13px;line-height:1.7;margin:12px 0 20px}.cur-import{display:flex;gap:8px;margin-bottom:20px}.cur-import input{flex:1;min-width:0}.cur-layout{display:grid;grid-template-columns:150px minmax(0,1fr) 280px;gap:16px;align-items:start}.cur-list{display:grid;gap:6px;max-height:75vh;overflow:auto;position:sticky;top:16px}.cur-list button{display:grid;gap:4px;min-height:64px;text-align:left;border:1px solid transparent;border-radius:8px;padding:12px;background:none;color:var(--ink);cursor:pointer}.cur-list button.on{border-color:var(--line-strong);background:var(--surface-2)}.cur-list strong{font-size:14px;font-weight:500}.cur-list small{font-size:12px;color:var(--ink-3);overflow-wrap:anywhere}.cur-editor fieldset,.cur-collections{border:0;margin:0;padding:0;min-width:0}.cur-section-head h2,.cur-editor h2{font-size:16px;margin:12px 0 16px}.cur-fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}.cur-fields label{display:grid;gap:6px;font-size:13px;min-width:0}.cur-fields .full{grid-column:1/-1}.cur-check{display:flex!important;align-items:center}.cur-check input{width:auto}.cur-details{border-top:1px solid var(--line);margin-top:24px;padding-top:16px}.cur-details summary{min-height:44px;cursor:pointer;font-size:14px}.cur-asset{display:grid;gap:8px;padding:16px;background:var(--surface-2);border-radius:8px}.cur-upload{position:relative;overflow:hidden;display:inline-flex!important}.cur-upload input{position:absolute;inset:0;opacity:0;cursor:pointer;width:100%}.cur-preview{position:sticky;top:16px;min-width:0}.cur-preview>p{font-size:12px;line-height:1.8}.cur-preview>.btn{width:100%}.cur-preview details{margin-top:16px;font-size:13px}.cur-raw{display:grid;gap:12px;padding-top:12px}.cur-raw img{max-width:100%;max-height:220px;object-fit:contain;background:var(--surface-2)}.cur-candidates{margin-top:20px}.cur-candidates h3{font-size:14px}.cur-candidates>div{display:flex;align-items:center;gap:8px;padding:12px 0;border-top:1px solid var(--line)}.cur-candidates>div>span{flex:1;min-width:0;font-size:13px}.cur-candidates small{display:block;overflow-wrap:anywhere;color:var(--ink-3);font-size:12px;margin-top:4px}.cur-candidates button{flex:none}.cur-error{color:var(--danger);padding:12px;background:var(--surface-2);border-radius:8px;font-size:14px}.cur-message{padding:12px;background:var(--surface-2);border-radius:8px;font-size:14px}.cur-collections{display:grid;gap:20px}.cur-collections>section{padding:16px;border:1px solid var(--line);border-radius:12px}.cur-footer{margin-top:24px;border-top:1px solid var(--line);padding-top:16px;font-size:12px}.cur-actions{justify-content:flex-start;gap:8px}@media(max-width:1100px){.cur-layout{grid-template-columns:140px minmax(0,1fr)}.cur-list{grid-row:1/span 2}.cur-preview{grid-column:2;grid-row:1;position:static;max-width:520px}.cur-editor{grid-column:2;grid-row:2}}@media(max-width:760px){.cur-layout{grid-template-columns:minmax(0,1fr)}.cur-list{position:static;max-height:220px;grid-row:1}.cur-preview{grid-column:1;grid-row:2}.cur-editor{grid-column:1;grid-row:3}.cur-import{flex-wrap:wrap}.cur-import input{flex-basis:100%}.cur-fields{grid-template-columns:1fr}.cur-toolbar .btn{font-size:12px}.cur-candidates>div{flex-wrap:wrap}}
</style>
