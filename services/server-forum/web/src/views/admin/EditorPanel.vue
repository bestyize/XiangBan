<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { editorApi, type Author, type Draft, type Detail } from '@/api/editor'
import type { AdminCategory } from '@/api/admin'
import { readBundle, exampleMarkdown, type ImportRow } from '@/lib/postBundle'
import { fmtTime } from './shared'
const authors = ref<Author[]>([]), categories = ref<AdminCategory[]>([]), author = ref(''), categoryOverride = ref(''), overwrite = ref(false)
const imports = ref<ImportRow[]>([]), drafts = ref<Draft[]>([]), checked = ref<string[]>([]), busy = ref(false), error = ref(''), progress = ref('')
const page = ref(1), total = ref(0), published = ref(false), detail = ref<Detail | null>(null), preview = ref(''), editTitle = ref(''), editCategory = ref(''), editTags = ref(''), editBody = ref('')
const results = ref<{ title: string; ok: boolean; message: string; url?: string }[]>([])
const leaves = computed(() => categories.value.filter(c => !categories.value.some(k => k.parent_id === c.id)))
async function loadDrafts() { const r = await editorApi.drafts(page.value, published.value); drafts.value = r.items; total.value = r.total; checked.value = [] }
async function load() { const [a, c] = await Promise.all([editorApi.authors(), editorApi.context()]); authors.value = a.items; categories.value = c.categories; author.value ||= a.items[0]?.uid || ''; await loadDrafts() }
async function run(fn: () => Promise<void>) { busy.value = true; error.value = ''; try { await fn() } catch (e) { error.value = e instanceof Error ? e.message : String(e) } finally { busy.value = false } }
async function choose(ev: Event) { const input = ev.target as HTMLInputElement; const file = input.files?.[0]; if (file) await run(async () => { imports.value = await readBundle(file); progress.value = `已解析 ${imports.value.length} 篇，可勾选导入` }); input.value = '' }
async function importPosts() {
  await run(async () => {
    if (!author.value) throw new Error('请先选择编辑账号')
    const selected = imports.value.filter(r => r.selected && r.data && !r.error)
    for (let i = 0; i < selected.length; i++) {
      const row = selected[i]!; progress.value = `正在导入 ${i + 1} / ${selected.length}`
      try { await editorApi.save({ ...row.data!, author_uid: author.value, category: categoryOverride.value || row.data!.category, overwrite: overwrite.value }); row.result = '已导入'; row.selected = false }
      catch (e) { row.result = e instanceof Error ? e.message : String(e) }
    }
    progress.value = '导入结束；失败的稿件可单独重试'; await loadDrafts()
  })
}
async function open(d: Draft) { await run(async () => { const r = await editorApi.detail(d.id); detail.value = r; editTitle.value = r.draft.title; editCategory.value = r.category; editTags.value = r.draft.tags.join(', '); editBody.value = r.body; preview.value = r.html }) }
async function save() {
  const d = detail.value; if (!d) return
  await run(async () => {
    await editorApi.save({ author_uid: d.draft.author_uid, external_id: d.draft.external_id, title: editTitle.value, category: editCategory.value, tags: editTags.value.split(/[,，]/).map(v => v.trim()).filter(Boolean), body: editBody.value, images: d.images, overwrite: true, expected_updated_at: d.draft.updated_at })
    const next = await editorApi.detail(d.draft.id); detail.value = next; preview.value = next.html; await loadDrafts(); progress.value = '已保存并更新预览'
  })
}
async function publishSelected() {
  await run(async () => {
    results.value = []
    for (const id of [...checked.value]) {
      const title = drafts.value.find(d => d.id === id)?.title || id
      try { const r = await editorApi.publish(id); results.value.push({ title, ok: true, message: r.already_published ? '此前已发布' : '发布成功', url: r.url }) }
      catch (e) { results.value.push({ title, ok: false, message: e instanceof Error ? e.message : String(e) }) }
    }
    const failed = results.value.filter(r => !r.ok).map(r => drafts.value.find(d => d.title === r.title)?.id).filter((v): v is string => !!v)
    await loadDrafts(); checked.value = failed; detail.value = null
  })
}
async function remove(d: Draft) { await run(async () => { await editorApi.remove(d.id); if (detail.value?.draft.id === d.id) detail.value = null; await loadDrafts() }) }
async function switchPage(delta: number) { page.value += delta; await run(loadDrafts) }
function sample() { const url = URL.createObjectURL(new Blob([exampleMarkdown], { type: 'text/markdown;charset=utf-8' })); const a = document.createElement('a'); a.href = url; a.download = 'example-post.md'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000) }
onMounted(() => run(load))
</script>
<template>
  <p class="muted">上传 ZIP 图文包，或单个纯文字 Markdown。每篇顶部填写 id、title、category、tags；图片使用相对路径。<button class="btn btn-ghost btn-sm" @click="sample">下载格式示例</button><RouterLink to="/admin/editor-accounts">管理编辑账号与本地令牌</RouterLink></p>
  <p v-if="error" class="error-text" role="alert">{{ error }}</p><p v-if="progress" role="status">{{ progress }}</p>
  <section class="panel import-box">
    <div class="adm-toolbar"><label>发布作者 <select v-model="author" class="select"><option value="">请选择编辑账号</option><option v-for="a in authors.filter(a => a.status === 'active')" :key="a.uid" :value="a.uid">{{ a.nickname }}</option></select></label><label class="btn">{{ busy ? '处理中…' : '选择 ZIP / Markdown' }}<input type="file" accept=".zip,.md" :disabled="busy" class="file-input" @change="choose" /></label></div>
    <p class="muted">每批 1–50 篇，ZIP 最大 100 MB、解压后最大 80 MB；每篇最多 20 张图、合计 20 MB。图文稿请用 ZIP 一起上传。</p>
    <template v-if="imports.length">
      <div class="adm-toolbar"><label>统一板块 <select v-model="categoryOverride" class="select"><option value="">沿用每篇元信息</option><option v-for="c in leaves" :key="c.id" :value="c.slug">{{ c.name }} · {{ c.slug }}</option></select></label><label><input v-model="overwrite" type="checkbox" /> 更新同编号的待发布稿</label></div>
      <table class="adm-table"><thead><tr><th>选择</th><th>标题 / 文件</th><th>板块</th><th>图片</th><th>状态</th></tr></thead><tbody><tr v-for="row in imports" :key="row.file"><td><input v-model="row.selected" type="checkbox" :disabled="busy || !!row.error" :aria-label="`选择 ${row.file}`" /></td><td>{{ row.data?.title || row.file }}<div class="adm-sub">{{ row.file }}</div></td><td>{{ categoryOverride || row.data?.category || '—' }}</td><td>{{ row.data?.images.length || 0 }}</td><td :class="{ 'error-text': row.error || (row.result && row.result !== '已导入') }">{{ row.error || row.result || '待导入' }}</td></tr></tbody></table>
      <button class="btn btn-primary" :disabled="busy || !author || !imports.some(r => r.selected && r.data && !r.error)" @click="importPosts">导入选中的稿件</button>
    </template>
  </section>
  <div class="adm-toolbar"><h2>稿件列表</h2><select v-model="published" class="select" :disabled="busy" @change="page = 1; run(loadDrafts)"><option :value="false">待发布</option><option :value="true">已发布</option></select><button v-if="!published" class="btn btn-primary" :disabled="busy || !checked.length" @click="publishSelected">发布选中（{{ checked.length }}）</button><button class="btn" :disabled="busy" @click="run(loadDrafts)">刷新</button></div>
  <div class="panel adm-card"><table class="adm-table"><thead><tr><th v-if="!published"><input type="checkbox" :checked="drafts.length > 0 && checked.length === drafts.length" aria-label="选择当前页全部稿件" :disabled="busy" @change="checked = ($event.target as HTMLInputElement).checked ? drafts.map(d => d.id) : []" /></th><th>标题</th><th>作者 / 板块</th><th>时间</th><th>操作</th></tr></thead><tbody><tr v-for="d in drafts" :key="d.id"><td v-if="!published"><input v-model="checked" type="checkbox" :value="d.id" :disabled="busy" :aria-label="`选择 ${d.title}`" /></td><td>{{ d.title }}<div class="adm-sub">{{ d.external_id }}</div></td><td>{{ authors.find(a => a.uid === d.author_uid)?.nickname || d.author_uid }}<div class="adm-sub">{{ categories.find(c => c.id === d.category_id)?.name }}</div></td><td>{{ fmtTime(d.updated_at) }}</td><td><button class="btn btn-sm" :disabled="busy" @click="open(d)">{{ published ? '查看' : '编辑预览' }}</button><a v-if="d.topic_id" class="btn btn-sm" :href="`/t/${d.topic_id}`" target="_blank" rel="noopener">打开帖子</a><button v-else class="btn btn-ghost btn-sm" :disabled="busy" @click="remove(d)">删除稿件</button></td></tr></tbody></table><p v-if="!drafts.length" class="adm-empty">暂无稿件</p></div>
  <div class="adm-toolbar"><span>共 {{ total }} 篇 · 第 {{ page }} 页</span><button class="btn btn-sm" :disabled="busy || page <= 1" @click="switchPage(-1)">上一页</button><button class="btn btn-sm" :disabled="busy || page * 30 >= total" @click="switchPage(1)">下一页</button></div>
  <div v-if="results.length" class="panel import-box"><p v-for="(r, i) in results" :key="i" :class="{ 'error-text': !r.ok }">{{ r.title }}：{{ r.message }} <a v-if="r.url" :href="r.url" target="_blank" rel="noopener">打开帖子</a></p></div>
  <section v-if="detail" class="panel import-box">
    <div class="adm-toolbar"><h2>{{ detail.draft.topic_id ? '已发布稿件' : '编辑与预览' }}</h2><button class="btn btn-sm" :disabled="busy" @click="detail = null">关闭</button></div>
    <div class="edit-fields"><label>标题<input v-model="editTitle" class="input" :disabled="!!detail.draft.topic_id" maxlength="80" /></label><label>板块<select v-model="editCategory" class="select" :disabled="!!detail.draft.topic_id"><option v-for="c in leaves" :key="c.id" :value="c.slug">{{ c.name }} · {{ c.slug }}</option></select></label><label>标签（逗号分隔）<input v-model="editTags" class="input" :disabled="!!detail.draft.topic_id" /></label></div>
    <div class="edit-grid"><label>Markdown<textarea v-model="editBody" class="input source" :readonly="!!detail.draft.topic_id" /></label><div><span>保存后的论坛预览</span><article class="prose draft-preview" v-html="preview" /></div></div>
    <p class="muted">已导入的图片随稿件保存；预览不会公开待发布图片。修改正文后点击保存更新预览。</p><button v-if="!detail.draft.topic_id" class="btn btn-primary" :disabled="busy" @click="save">保存并更新预览</button>
  </section>
</template>
<style scoped>
.import-box{padding:20px;margin:16px 0;overflow-x:auto}.adm-toolbar{flex-wrap:wrap;gap:12px}.adm-toolbar label{display:flex;align-items:center;gap:8px}.file-input{position:absolute;width:1px;height:1px;opacity:0}.edit-fields{display:grid;gap:12px;margin:16px 0}.edit-fields label,.edit-grid label{display:grid;gap:6px}.edit-grid{display:grid;grid-template-columns:1fr 1fr;gap:20px}.source{min-height:420px;resize:vertical;font-family:monospace;font-size:13px}.draft-preview{margin-top:6px;min-height:420px;padding:16px;border:1px solid var(--line);border-radius:8px;overflow-wrap:anywhere}.draft-preview :deep(img){max-width:100%;height:auto}.draft-preview :deep(pre){overflow:auto}.draft-preview :deep(table){display:block;overflow:auto}.muted{line-height:1.8}h2{font-size:18px;margin:0}.adm-table td{overflow-wrap:anywhere}@media(max-width:850px){.edit-grid{grid-template-columns:1fr}.source{min-height:280px}}
</style>
