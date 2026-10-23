<script setup lang="ts">
import { computed, nextTick, onActivated, onBeforeUnmount, onDeactivated, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Icon from '../components/Icon.vue'
import { downloadPlugin, fetchJson, fetchPlugins, filtersFromQuery, installPrompt, pluginPath,
  queryFromFilters, releasePath, sorts, type Filters, type PluginDetail, type PluginEntry, type PluginPage } from '../plugins/api'

const route = useRoute()
const router = useRouter()
const filters = computed(() => filtersFromQuery(route.query))
const keyword = ref(filters.value.keyword)
const categories = ref<{ key: string; name: string }[]>([])
const categoryError = ref('')
const feed = ref<PluginPage | null>(null)
const loading = ref(false)
const error = ref('')
const brokenIcons = ref(new Set<string>())
const dialog = ref<HTMLDialogElement | null>(null)
const detail = ref<PluginDetail | null>(null)
const detailLoading = ref(false)
const detailError = ref('')
const readme = ref('')
const readmeError = ref('')
const readmeLoading = ref(false)
const actionError = ref('')
const downloading = ref(false)
const copied = ref(false)
const manualPrompt = ref('')
let searchTimer: ReturnType<typeof setTimeout> | undefined
let copyTimer: ReturnType<typeof setTimeout> | undefined
let feedRequest: AbortController | undefined
let detailRequest: AbortController | undefined
let categoryRequest: AbortController | undefined
let paused = false
const selectedSlug = computed(() => route.meta.tab === 'plugins' && typeof route.query.plugin === 'string' ? route.query.plugin : '')
const selectedNamespace = computed(() => typeof route.query.namespace === 'string' ? route.query.namespace : '')
const total = computed(() => feed.value?.total.toLocaleString('zh-CN'))
const currentCategory = computed(() => categories.value.find(c => c.key === filters.value.category)?.name ?? '全部技能')
const number = (n: number) => n >= 10000 ? `${(n / 10000).toFixed(1)}万` : n.toLocaleString('zh-CN')
const icon = (entry: PluginEntry) => entry.iconUrl && !brokenIcons.value.has(entry.iconUrl) ? entry.iconUrl : null
const message = (reason: unknown) => reason instanceof Error ? reason.message : String(reason)

function updateFilters(patch: Partial<Filters>) {
  clearTimeout(searchTimer)
  void router.push({ path: '/plugins', query: queryFromFilters({ ...filters.value, page: 1, ...patch }) })
}
function search() { updateFilters({ keyword: keyword.value }) }
watch(keyword, () => {
  clearTimeout(searchTimer)
  if (keyword.value === filters.value.keyword) return
  searchTimer = setTimeout(search, 300)
})
watch(() => filters.value.keyword, value => { keyword.value = value })

async function loadCategories() {
  categoryRequest?.abort()
  const request = categoryRequest = new AbortController()
  categoryError.value = ''
  try {
    const result = await fetchJson<{ results: { key: string; name: string }[] }>('/api/v1/ai/plugins/categories', request.signal)
    if (!request.signal.aborted) categories.value = result.results
  } catch (reason) { if (!request.signal.aborted) categoryError.value = message(reason) }
}
async function loadFeed() {
  if (route.meta.tab !== 'plugins' || paused) return
  feedRequest?.abort()
  const request = feedRequest = new AbortController()
  loading.value = true
  error.value = ''
  try {
    const result = await fetchPlugins(filters.value, request.signal)
    if (!request.signal.aborted) feed.value = result
  } catch (reason) {
    if (!request.signal.aborted) { feed.value = null; error.value = message(reason) }
  } finally { if (!request.signal.aborted) loading.value = false }
}
watch(() => JSON.stringify(filters.value), () => void loadFeed(), { immediate: true })
void loadCategories()

function open(entry: PluginEntry) {
  clearTimeout(searchTimer)
  void router.push({ path: '/plugins', query: { ...route.query, plugin: entry.slug, namespace: entry.namespace || undefined } })
}
function close() {
  const query = { ...route.query }
  delete query.plugin
  delete query.namespace
  void router.push({ path: '/plugins', query })
}
function dialogClick(event: MouseEvent) {
  if (event.target === event.currentTarget) close()
}
async function loadDetail() {
  detailRequest?.abort()
  const request = detailRequest = new AbortController()
  detail.value = null
  readme.value = ''
  detailError.value = readmeError.value = actionError.value = manualPrompt.value = ''
  copied.value = false
  downloading.value = false
  readmeLoading.value = false
  const slug = selectedSlug.value
  if (!slug || paused) { detailLoading.value = false; dialog.value?.close(); return }
  detailLoading.value = true
  await nextTick()
  if (request.signal.aborted) return
  if (!dialog.value?.open) dialog.value?.showModal()
  try {
    const result = await fetchJson<PluginDetail>(pluginPath(slug, selectedNamespace.value), request.signal)
    if (request.signal.aborted) return
    detail.value = result
    detailLoading.value = false
    if (!result.entry.version) return
    readmeLoading.value = true
    try {
      const body = await fetchJson<{ content: string }>(releasePath(result.entry, 'readme'), request.signal)
      if (!request.signal.aborted) readme.value = body.content
    } catch (reason) { if (!request.signal.aborted) readmeError.value = message(reason) }
    finally { if (!request.signal.aborted) readmeLoading.value = false }
  } catch (reason) { if (!request.signal.aborted) detailError.value = message(reason) }
  finally { if (!request.signal.aborted) detailLoading.value = false }
}
watch([selectedSlug, selectedNamespace], () => void loadDetail(), { immediate: true })
async function copyPrompt() {
  if (!detail.value) return
  const prompt = installPrompt(detail.value.entry)
  try {
    await navigator.clipboard.writeText(prompt)
    copied.value = true
    clearTimeout(copyTimer)
    copyTimer = setTimeout(() => { copied.value = false }, 2000)
  } catch { manualPrompt.value = prompt }
}
async function download() {
  if (!detail.value || downloading.value) return
  const current = detail.value
  downloading.value = true
  actionError.value = ''
  try { await downloadPlugin(current.entry) }
  catch (reason) { if (detail.value === current) actionError.value = message(reason) }
  finally { if (detail.value === current) downloading.value = false }
}
onDeactivated(() => {
  paused = true
  clearTimeout(searchTimer)
  feedRequest?.abort()
  detailRequest?.abort()
  categoryRequest?.abort()
  dialog.value?.close()
})
onActivated(() => {
  if (!paused) return
  paused = false
  if (!categories.value.length) void loadCategories()
  void loadFeed()
  void loadDetail()
})
onBeforeUnmount(() => {
  clearTimeout(searchTimer)
  clearTimeout(copyTimer)
  feedRequest?.abort()
  detailRequest?.abort()
  categoryRequest?.abort()
})
</script>

<template>
  <main class="page plugin-market">
    <header class="market-intro">
      <span class="eyebrow"><Icon name="sparkles" :size="15" /> 小伴 AI 插件</span>
      <h1>给小伴，添一点本领。</h1>
      <p>发现写作、编程、办公与生活技能，把想做的事交给小伴。</p>
      <form class="market-search" role="search" @submit.prevent="search">
        <Icon name="search" :size="20" />
        <input v-model="keyword" maxlength="80" aria-label="搜索 AI 技能" placeholder="搜索技能，或试试你想完成的任务…">
        <button v-if="keyword" type="button" aria-label="清空搜索" @click="keyword = ''; updateFilters({ keyword: '' })"><Icon name="close" :size="17" /></button>
        <button type="submit" class="search-submit">搜索</button>
      </form>
    </header>

    <div class="market-controls">
      <div class="category-rail hide-scrollbar" aria-label="技能分类">
        <button class="category" :aria-pressed="!filters.category" @click="updateFilters({ category: '' })">全部</button>
        <button v-for="item in categories" :key="item.key" class="category" :aria-pressed="filters.category === item.key" @click="updateFilters({ category: item.key })">{{ item.name }}</button>
      </div>
      <div class="list-toolbar">
        <div class="list-title"><h2>{{ currentCategory }}</h2><span v-if="total && !loading">{{ total }} 项匹配技能</span></div>
        <div class="selectors">
          <select :value="filters.source" aria-label="技能来源" @change="updateFilters({ source: ($event.target as HTMLSelectElement).value })">
            <option value="">全部来源</option><option value="official">官方发布</option><option value="community">社区发布</option><option value="enterprise">企业发布</option>
          </select>
          <select :value="filters.sortBy" aria-label="技能排序" @change="updateFilters({ sortBy: ($event.target as HTMLSelectElement).value })">
            <option v-for="sort in sorts" :key="sort.value" :value="sort.value">{{ sort.label }}</option>
          </select>
        </div>
      </div>
    </div>
    <p v-if="categoryError" class="notice" role="status">分类暂不可用，仍可搜索技能。<button @click="loadCategories">重试分类</button></p>
    <div v-if="error" class="empty-state" role="alert"><Icon name="network" :size="30" /><h2>暂时无法加载技能</h2><p>{{ error }}</p><button class="market-button" @click="loadFeed">重新加载</button></div>
    <div v-else-if="loading" class="plugin-grid" aria-busy="true" aria-label="正在加载技能"><div v-for="n in 9" :key="n" class="plugin-skeleton"><span /><i /><i /></div></div>
    <template v-else-if="feed?.results.length">
      <div class="plugin-grid">
        <article v-for="entry in feed.results" :key="`${entry.namespace}/${entry.slug}`" class="plugin-card">
          <button class="card-open" :aria-label="`查看 ${entry.name} 详情`" @click="open(entry)">
            <div class="card-top"><div class="plugin-icon"><img v-if="icon(entry)" :src="icon(entry)!" alt="" loading="lazy" referrerpolicy="no-referrer" @error="brokenIcons.add(entry.iconUrl!)"><Icon v-else name="sparkles" :size="24" /></div><span v-if="entry.source === 'official'" class="official"><Icon name="check" :size="11" /> 官方发布</span></div>
            <h3>{{ entry.name || entry.slug }}</h3>
            <p class="plugin-description">{{ entry.description || '查看技能说明与使用方法' }}</p>
          </button>
          <footer><span class="publisher" :title="entry.namespace">{{ entry.namespace || 'SkillHub' }}</span><span :aria-label="`${entry.downloads} 次下载`"><Icon name="download" :size="13" />{{ number(entry.downloads) }}</span><button :aria-label="`查看 ${entry.name} 详情`" @click="open(entry)"><Icon name="plus" :size="15" /> 查看</button></footer>
        </article>
      </div>
      <nav class="pagination" aria-label="技能分页"><button :disabled="filters.page <= 1" aria-label="上一页" @click="updateFilters({ page: filters.page - 1 })"><Icon name="chevron-left" :size="16" /></button><span>第 {{ filters.page }} 页</span><button :disabled="!feed.hasMore" aria-label="下一页" @click="updateFilters({ page: filters.page + 1 })"><Icon name="chevron-right" :size="16" /></button></nav>
    </template>
    <div v-else class="empty-state"><Icon name="search" :size="30" /><h2>没有找到匹配技能</h2><p>换一个关键词，或试试其他分类。</p><button class="market-button" @click="keyword = ''; updateFilters({ keyword: '', category: '', source: '' })">查看全部技能</button></div>
    <footer class="market-footer"><span>技能内容与安装包来自 SkillHub</span><a href="https://skillhub.cn" target="_blank" rel="noopener noreferrer">访问来源社区 <Icon name="external" :size="12" /></a></footer>

    <dialog ref="dialog" class="plugin-dialog" aria-labelledby="plugin-title" @cancel.prevent="close" @click="dialogClick">
      <div class="dialog-body">
        <button class="dialog-close" aria-label="关闭插件详情" @click="close"><Icon name="close" :size="20" /></button>
        <h2 v-if="!detail" id="plugin-title">{{ detailLoading ? '正在加载技能…' : '技能详情' }}</h2>
        <div v-if="detailError" class="detail-error" role="alert"><p>{{ detailError }}</p><button class="market-button" @click="loadDetail">重试</button></div>
        <template v-if="detail">
          <div class="detail-heading"><div class="plugin-icon"><img v-if="icon(detail.entry)" :src="icon(detail.entry)!" alt="" referrerpolicy="no-referrer" @error="brokenIcons.add(detail.entry.iconUrl!)"><Icon v-else name="sparkles" :size="26" /></div><div><h2 id="plugin-title">{{ detail.entry.name }}</h2><p>{{ detail.publisher }}<span v-if="detail.entry.version"> · v{{ detail.entry.version }}</span></p></div></div>
          <p class="detail-summary">{{ detail.entry.description }}</p>
          <div class="detail-labels"><span><Icon name="download" :size="14" />{{ number(detail.entry.downloads) }} 次下载</span><span>{{ detail.entry.source === 'official' ? '官方发布' : detail.entry.source === 'enterprise' ? '企业发布' : detail.entry.source === 'community' ? '社区发布' : '其他来源' }}</span><span v-if="detail.entry.labels?.requires_api_key === 'true'">需 API Key</span><span v-if="detail.entry.labels?.pricing_type === 'paid'">付费服务</span></div>
          <div class="detail-actions"><button class="market-button primary" :disabled="!detail.entry.version" @click="copyPrompt"><Icon :name="copied ? 'check' : 'plus'" :size="16" />{{ copied ? '已复制' : '复制安装指令' }}</button><button class="market-button" :disabled="!detail.entry.version || downloading" @click="download"><Icon name="download" :size="16" />{{ downloading ? '正在下载…' : '下载技能包' }}</button><a :href="detail.sourceUrl" target="_blank" rel="noopener noreferrer">来源页面 <Icon name="external" :size="13" /></a></div>
          <p class="install-help">将安装指令粘贴到小伴桌面的聊天中，审查后确认安装。</p>
          <p v-if="copied" class="sr-status" role="status">安装指令已复制</p>
          <p v-if="!detail.entry.version" class="notice">此技能暂未提供可用版本。</p>
          <p v-if="actionError" class="notice" role="alert">{{ actionError }}</p>
          <label v-if="manualPrompt" class="manual-copy">请手动复制安装指令<textarea readonly :value="manualPrompt" @focus="($event.target as HTMLTextAreaElement).select()" /></label>
          <section class="readme"><h3>技能说明</h3><p v-if="readmeLoading" role="status">正在读取 SKILL.md…</p><p v-else-if="readmeError" class="notice">{{ readmeError }}，可到来源页面查看说明。</p><pre v-else-if="readme">{{ readme }}</pre><p v-else>暂未提供说明。</p></section>
          <section v-if="detail.changelog" class="changelog"><h3>版本更新</h3><p>{{ detail.changelog }}</p></section>
        </template>
      </div>
    </dialog>
  </main>
</template>

<style scoped>
.plugin-market{max-width:1240px;--accent:#cc3455;--accent-soft:#fff0f3}
.market-intro{padding:52px 0 34px}.eyebrow{display:flex;align-items:center;gap:7px;color:var(--accent);font-size:13px;font-weight:600;letter-spacing:.04em}
.market-intro h1{font-size:clamp(27px,4vw,39px);font-weight:650;letter-spacing:-.04em;margin:15px 0 14px;color:var(--text)}
.market-intro>p{color:var(--text-2);font-size:15px}.market-search{display:flex;align-items:center;gap:12px;max-width:640px;margin-top:28px;padding:7px 8px 7px 17px;border:1px solid var(--line-strong);background:var(--surface);border-radius:15px;color:var(--text-2);box-shadow:0 4px 18px #50273605}
.market-search:focus-within{border-color:var(--accent)}.market-search input{min-width:0;flex:1;outline:none;background:none;border:0;font-size:14px;height:35px}.market-search input::placeholder{color:var(--text-2)}.market-search>button{display:flex;align-items:center;justify-content:center;padding:5px}.market-search .search-submit{border-radius:9px;background:var(--accent);color:white;padding:9px 18px;font-size:13px}
.market-controls{position:sticky;top:calc(var(--header-h) + var(--safe-top));z-index:5;background:var(--bg);padding:8px 0 14px}.category-rail{display:flex;gap:7px;overflow-x:auto;padding:4px 1px 17px}.category{white-space:nowrap;border:1px solid transparent;padding:7px 13px;border-radius:9px;color:var(--text-2);font-size:13px}.category:hover{background:var(--surface)}.category[aria-pressed=true]{background:var(--accent-soft);border-color:#eecad3;color:var(--accent);font-weight:600}
.list-toolbar{display:flex;justify-content:space-between;align-items:center;gap:16px;margin-top:10px}.list-title{display:flex;align-items:baseline;gap:14px}.list-title h2{font-size:18px;font-weight:600}.list-title span{font-size:12px;color:var(--text-2)}.selectors{display:flex;gap:8px}.selectors select{font-size:12px;background:var(--surface);border:1px solid var(--line);padding:7px 10px;border-radius:8px;max-width:140px}
.plugin-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}.plugin-card{min-width:0;background:var(--surface);border:1px solid var(--line);border-radius:17px;transition:border-color .18s,box-shadow .18s;overflow:hidden}.plugin-card:hover{border-color:#e1b9c4;box-shadow:0 5px 20px #50273607}.card-open{display:block;text-align:left;width:100%;padding:21px 21px 16px}.card-top{display:flex;align-items:center;justify-content:space-between;margin-bottom:16px;gap:12px}.plugin-icon{display:grid;place-items:center;flex-shrink:0;width:46px;height:46px;border:1px solid var(--line);border-radius:13px;color:var(--accent);background:var(--accent-soft);overflow:hidden}.plugin-icon img{width:100%;height:100%;object-fit:contain;background:white}.official{display:flex;align-items:center;gap:4px;font-size:11px;color:#327c66;background:#eff8f3;border-radius:5px;padding:3px 6px}.card-open h3{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:15px;font-weight:600;margin-bottom:10px}.plugin-description{display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;color:var(--text-2);font-size:13px;line-height:1.7;min-height:44px;overflow-wrap:anywhere}.plugin-card footer{margin:0 21px;padding:12px 0 16px;border-top:1px solid var(--line);display:flex;align-items:center;gap:12px;font-size:11px;color:var(--text-2)}.plugin-card footer span{display:flex;align-items:center;gap:4px;white-space:nowrap}.plugin-card footer .publisher{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;display:block}.plugin-card footer button{display:flex;align-items:center;gap:3px;color:var(--accent);font-size:12px;flex-shrink:0}
.plugin-skeleton{height:227px;border:1px solid var(--line);border-radius:17px;padding:22px;background:var(--surface)}.plugin-skeleton span{display:block;width:46px;height:46px;border-radius:12px;background:var(--surface-2);margin-bottom:25px}.plugin-skeleton i{display:block;height:12px;background:var(--surface-2);border-radius:5px;margin-bottom:15px}.plugin-skeleton i:last-child{width:75%}.empty-state{min-height:280px;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;gap:14px;color:var(--text-2)}.empty-state h2{color:var(--text);font-size:20px}.empty-state p{font-size:13px}.pagination{display:flex;align-items:center;justify-content:center;gap:18px;padding:32px 0;font-size:12px;color:var(--text-2)}.pagination button{width:32px;height:32px;border:1px solid var(--line);border-radius:9px;display:grid;place-items:center;background:var(--surface)}.pagination button:disabled{opacity:.35;cursor:default}.market-footer{display:flex;justify-content:space-between;gap:12px;border-top:1px solid var(--line);padding:20px 0;font-size:11px;color:var(--text-2)}.market-footer a{display:flex;align-items:center;gap:5px}
.plugin-dialog{position:fixed;margin:auto;width:min(760px,calc(100vw - 32px));max-height:85dvh;padding:0;border:1px solid var(--line);border-radius:23px;background:var(--surface);color:var(--text);box-shadow:0 24px 100px #30131d25;overflow:auto;overscroll-behavior:contain}.plugin-dialog::backdrop{background:#30242a55;backdrop-filter:blur(5px)}.dialog-body{padding:32px;position:relative;min-height:150px}.dialog-close{position:absolute;right:18px;top:18px;border-radius:50%;padding:6px;color:var(--text-2);display:grid;place-items:center}.dialog-close:hover{background:var(--surface-2)}.detail-heading{display:flex;align-items:center;gap:15px;padding-right:30px}.detail-heading h2{font-size:23px;font-weight:600;overflow-wrap:anywhere}.detail-heading p{font-size:12px;color:var(--text-2);margin-top:6px}.detail-summary{font-size:14px;line-height:1.9;margin-top:23px;white-space:pre-wrap;overflow-wrap:anywhere}.detail-labels{display:flex;gap:10px;flex-wrap:wrap;margin:18px 0 24px}.detail-labels span{display:flex;align-items:center;gap:4px;background:var(--bg);padding:4px 8px;border-radius:6px;font-size:11px;color:var(--text-2)}.detail-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.market-button{display:inline-flex;align-items:center;justify-content:center;gap:6px;padding:10px 15px;border:1px solid var(--line-strong);border-radius:10px;font-size:13px;background:var(--surface)}.market-button.primary{background:var(--accent);color:white;border-color:var(--accent)}.market-button:disabled{opacity:.5;cursor:default}.detail-actions>a{display:flex;align-items:center;gap:4px;font-size:12px;color:var(--text-2);margin-left:auto}.install-help{font-size:12px;color:var(--text-2);margin:12px 0 26px}.readme,.changelog{border-top:1px solid var(--line);padding-top:22px;margin-top:24px}.readme h3,.changelog h3{font-size:14px;font-weight:600;margin-bottom:15px}.readme pre{font-family:var(--font-ui);font-size:13px;line-height:1.9;white-space:pre-wrap;overflow-wrap:anywhere;max-height:400px;overflow:auto;padding:18px;border-radius:12px;background:var(--bg);color:var(--text-2)}.readme>p,.changelog>p{font-size:13px;color:var(--text-2);white-space:pre-wrap;overflow-wrap:anywhere}.notice{font-size:12px;color:#a34635;padding:10px 0}.notice button{color:var(--accent);margin-left:8px;text-decoration:underline}.detail-error{padding:18px 0;font-size:13px}.detail-error button{margin-top:16px}.manual-copy{display:block;font-size:12px;margin-top:16px}.manual-copy textarea{width:100%;height:110px;margin-top:8px;padding:10px;background:var(--bg);color:var(--text);border:1px solid var(--line);border-radius:8px;font:inherit}.sr-status{position:absolute;clip-path:inset(50%);width:1px;height:1px;overflow:hidden}
@media(max-width:960px){.plugin-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
@media(max-width:768px){.market-intro{padding-top:28px}.market-intro>p{font-size:13px}.market-controls{top:0}.list-title{gap:5px;flex-direction:column}.selectors{gap:5px}.selectors select{font-size:11px;padding:6px;max-width:108px}.market-footer{flex-direction:column}.dialog-body{padding:24px 20px}.detail-actions>a{margin-left:0}.plugin-dialog{max-height:90dvh}}
@media(max-width:520px){.plugin-grid{grid-template-columns:minmax(0,1fr);gap:12px}.card-open{padding-bottom:12px}.plugin-description{min-height:0}.market-search{gap:8px}.market-search input{font-size:12px}.market-search .search-submit{padding:8px 13px}.detail-heading h2{font-size:20px}}
</style>
