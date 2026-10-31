<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { challengeEditorial, type EditorialChallenge, type ChallengeDetail, type ChallengeEntry } from '@/api/communityEditorial'
import { fmtTime } from './shared'
const labels: Record<string, string> = { draft: '待审核草稿', upcoming: '即将开始', open: '投稿中', ended: '已截止', closed: '已关闭' }
const rows = ref<EditorialChallenge[]>([])
const total = ref(0)
const page = ref(1)
const listLoading = ref(false)
const listError = ref('')
const detail = ref<ChallengeDetail | null>(null)
const detailLoading = ref(false)
const entryPage = ref(1)
const error = ref('')
const busy = ref(false)
const message = ref('')
const editing = ref(false)
const originalKey = ref('')
const form = reactive({ key: '', title: '', description: '', starts_at: '', ends_at: '', submission_tag: '', topic_id: '', revision: 0 })
const isDraft = computed(() => !detail.value || detail.value.challenge.publication_status === 'draft')
const dirty = computed(() => {
  if (!detail.value) return editing.value
  const row = detail.value.challenge
  return form.key !== row.key || form.title.trim() !== row.title || form.description.trim() !== row.description || form.starts_at !== localTime(row.starts_at) || form.ends_at !== localTime(row.ends_at) || form.submission_tag.trim() !== row.submission_tag || form.topic_id.trim() !== (row.topic_id?.toString() || '')
})
const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone
function localTime(iso: string) { const date = new Date(iso); return new Date(date.getTime() - date.getTimezoneOffset() * 60000).toISOString().slice(0, 16) }
function apply(data: ChallengeDetail) {
  detail.value = data; editing.value = true; originalKey.value = data.challenge.key
  const challenge = data.challenge
  Object.assign(form, { key: challenge.key, title: challenge.title, description: challenge.description, starts_at: localTime(challenge.starts_at), ends_at: localTime(challenge.ends_at), submission_tag: challenge.submission_tag, topic_id: challenge.topic_id?.toString() || '', revision: challenge.revision })
}
async function loadList(next = page.value) {
  listLoading.value = true; listError.value = ''
  try { const data = await challengeEditorial.list(next); rows.value = data.items; total.value = data.total; page.value = data.page }
  catch (e) { listError.value = e instanceof Error ? e.message : '挑战列表加载失败' }
  finally { listLoading.value = false }
}
async function open(key: string, next = 1) {
  detailLoading.value = true; error.value = ''; message.value = ''
  try { const data = await challengeEditorial.detail(key, next); apply(data); entryPage.value = data.entries.page }
  catch (e) { error.value = e instanceof Error ? e.message : '挑战加载失败' }
  finally { detailLoading.value = false }
}
function create() {
  detail.value = null; editing.value = true; originalKey.value = ''; error.value = ''; message.value = ''; entryPage.value = 1
  Object.assign(form, { key: '', title: '', description: '', starts_at: localTime(new Date().toISOString()), ends_at: localTime(new Date(Date.now() + 7 * 86400000).toISOString()), submission_tag: '', topic_id: '', revision: 0 })
}
async function save() {
  busy.value = true; error.value = ''; message.value = ''
  try {
    if (!/^[a-z0-9_-]{2,48}$/.test(form.key)) throw new Error('挑战标识须为 2–48 位小写字母、数字、连字符或下划线')
    const start = new Date(form.starts_at), end = new Date(form.ends_at)
    if (!Number.isFinite(start.getTime()) || !Number.isFinite(end.getTime()) || end <= start) throw new Error('请选择有效的开始和截止时间')
    const topic = form.topic_id.trim() ? Number(form.topic_id) : null
    if (topic !== null && (!Number.isSafeInteger(topic) || topic <= 0)) throw new Error('讨论帖编号须为正整数')
    apply(await challengeEditorial.save(form.key, { title: form.title.trim(), description: form.description.trim(), starts_at: start.toISOString(), ends_at: end.toISOString(), submission_tag: form.submission_tag.trim(), topic_id: topic, expected_revision: form.revision }))
    message.value = '草稿已保存，请检查全部内容后审核发布。'; await loadList()
  } catch (e) { error.value = e instanceof Error ? e.message : '保存失败' }
  finally { busy.value = false }
}
async function action(name: 'publish' | 'unpublish' | 'close') {
  if (!detail.value) return
  const prompt = name === 'publish' ? '确认已审核标题、说明、时间和投稿标签，向社区发布这个挑战？' : name === 'unpublish' ? '撤回后公开页面和投稿入口会暂时关闭，已有投稿保留。确认撤回重新编辑？' : '关闭后立即停止投稿，已有作品仍可查看。确认关闭挑战？'
  if (!window.confirm(prompt)) return
  busy.value = true; error.value = ''; message.value = ''
  try { apply(await challengeEditorial.action(detail.value.challenge.key, name, detail.value.challenge.revision)); message.value = name === 'publish' ? '挑战已发布。' : name === 'close' ? '挑战已关闭。' : '已撤回为草稿。'; await loadList() }
  catch (e) { error.value = e instanceof Error ? e.message : '操作失败，请重新加载后重试' }
  finally { busy.value = false }
}
async function feature(entry: ChallengeEntry, featured = !entry.featured) {
  if (!detail.value) return
  busy.value = true; error.value = ''
  try { await challengeEditorial.feature(detail.value.challenge.key, entry, featured); await open(detail.value.challenge.key, entryPage.value); message.value = '投稿精选已更新。' }
  catch (e) { error.value = e instanceof Error ? e.message : '精选操作失败' }
  finally { busy.value = false }
}
onMounted(() => loadList())
</script>
<template>
  <section class="panel adm-card challenges-panel">
    <div class="adm-toolbar"><h3>主题挑战</h3><button class="btn" :disabled="busy || listLoading || detailLoading" @click="loadList()">刷新列表</button><button class="btn btn-primary" :disabled="busy || detailLoading" @click="create">新建草稿</button></div>
    <p class="adm-sub">挑战先保存草稿，再由管理员明确审核发布。投稿只收用户自己的公开作品；点赞沿用作品讨论帖，挑战不发积分。</p>
    <p v-if="listError" role="alert">{{ listError }} <button class="btn" @click="loadList()">重试</button></p>
    <p v-if="listLoading" role="status">正在加载挑战…</p>
    <p v-else-if="!listError && !rows.length" class="adm-sub">尚未创建挑战。</p>
    <ul class="challenge-list"><li v-for="row in rows" :key="row.key"><button class="btn" :disabled="busy || detailLoading" @click="open(row.key)">{{ row.title }}</button><span>{{ labels[row.status] }} · {{ row.entry_count }} 件公开投稿 · 截止 {{ fmtTime(row.ends_at) }}</span></li></ul>
    <div v-if="total > 25" class="adm-toolbar"><button class="btn" :disabled="page === 1 || listLoading" @click="loadList(page - 1)">上一页</button><span>{{ page }} / {{ Math.ceil(total / 25) }}</span><button class="btn" :disabled="page * 25 >= total || listLoading" @click="loadList(page + 1)">下一页</button></div>
    <p v-if="error" role="alert">{{ error }} <button v-if="originalKey" class="btn" :disabled="busy || detailLoading" @click="open(originalKey, entryPage)">重新加载当前挑战</button></p>
    <p v-if="message" role="status">{{ message }}</p><p v-if="detailLoading" role="status">正在读取挑战与投稿…</p>
    <form v-if="editing" class="adm-form" @submit.prevent="save">
      <h4>{{ detail ? labels[detail.challenge.status] : '新挑战草稿' }}</h4>
      <p v-if="!isDraft" class="adm-sub">已发布的正文已锁定。需要修改时先撤回，保存后重新审核发布。</p>
      <div class="adm-form-grid"><label class="field"><span>稳定标识</span><input v-model="form.key" class="input" required maxlength="48" pattern="[a-z0-9_-]{2,48}" :disabled="!!originalKey || busy"></label><label class="field"><span>标题</span><input v-model="form.title" class="input" required maxlength="80" :disabled="!isDraft || busy"></label><label class="field"><span>开始时间（{{ timezone }}）</span><input v-model="form.starts_at" class="input" type="datetime-local" required :disabled="!isDraft || busy"></label><label class="field"><span>截止时间（{{ timezone }}）</span><input v-model="form.ends_at" class="input" type="datetime-local" required :disabled="!isDraft || busy"></label><label class="field"><span>投稿标签</span><input v-model="form.submission_tag" class="input" required maxlength="40" placeholder="desktop-challenge" :disabled="!isDraft || busy"></label><label class="field"><span>公开讨论帖编号（选填）</span><input v-model="form.topic_id" class="input" inputmode="numeric" :disabled="!isDraft || busy"></label></div>
      <label class="field"><span>挑战说明（纯文本）</span><textarea v-model="form.description" class="input" rows="6" required maxlength="3000" :disabled="!isDraft || busy" /></label>
      <div class="adm-toolbar"><button v-if="isDraft" class="btn btn-primary" :disabled="busy || detailLoading">保存草稿</button><button v-if="detail?.challenge.publication_status === 'draft'" class="btn" type="button" :disabled="busy || detailLoading || dirty" @click="action('publish')">审核并发布已保存草稿</button><button v-if="detail?.challenge.publication_status === 'published'" class="btn" type="button" :disabled="busy || detailLoading" @click="action('unpublish')">撤回重新编辑</button><button v-if="detail?.challenge.publication_status === 'published'" class="btn" type="button" :disabled="busy || detailLoading" @click="action('close')">关闭挑战</button></div>
      <p v-if="detail?.challenge.publication_status === 'draft' && dirty" class="adm-sub">有尚未保存的修改，请先保存，再审核当前版本。</p>
    </form>
    <section v-if="detail">
      <h4>投稿作品</h4><p class="adm-sub">精选仅改变展示顺序；点赞来自作品原有讨论帖。下架和作者被封禁的作品不会出现在公开列表。</p>
      <p v-if="!detail.entries.items.length" class="adm-sub">暂无投稿作品。</p>
      <ul class="challenge-list"><li v-for="entry in detail.entries.items" :key="entry.pack_id"><span>{{ entry.pack.title }} · {{ entry.visible ? '公开可见' : '当前不可公开' }} · {{ entry.likes === null ? '未关联可点赞讨论' : `${entry.likes} 个讨论点赞` }}</span><span><label>顺序 <input v-model.number="entry.sort_order" class="input entry-order" type="number" :disabled="busy"></label> <button class="btn" :disabled="busy || detailLoading || detail.challenge.publication_status === 'draft' || (!entry.visible && !entry.featured)" @click="feature(entry)">{{ entry.featured ? '取消精选' : '设为精选' }}</button> <button class="btn" :disabled="busy || detailLoading || detail.challenge.publication_status === 'draft'" @click="feature(entry, entry.featured)">保存顺序</button></span></li></ul>
      <div v-if="detail.entries.total > 50" class="adm-toolbar"><button class="btn" :disabled="entryPage === 1 || busy || detailLoading" @click="open(detail.challenge.key, entryPage - 1)">上一页投稿</button><span>{{ entryPage }} / {{ Math.ceil(detail.entries.total / 50) }}</span><button class="btn" :disabled="entryPage * 50 >= detail.entries.total || busy || detailLoading" @click="open(detail.challenge.key, entryPage + 1)">下一页投稿</button></div>
    </section>
  </section>
</template>
<style scoped>
.challenges-panel { margin-top: 20px; display: grid; gap: 12px; }
.challenges-panel h3, .challenges-panel h4 { margin: 0; margin-right: auto; }
.challenge-list { margin: 0; padding: 0; list-style: none; }
.challenge-list li { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding: 8px 0; flex-wrap: wrap; }
.entry-order { width: 90px; }
</style>
