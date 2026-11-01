<script setup lang="ts">
import { useConfirm } from './confirm.ts';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NButton, NInput, NModal } from 'naive-ui';
import { ArrowLeft, Copy, ExternalLink, FileText, Folder, KeyRound, Link2, MessageCircle, Pencil, RefreshCw, Search, Trash2 } from 'lucide-vue-next';
import { invoke, listen } from '../../bridge.ts';
import { openStoreWindow } from '../ai-store/store-window.ts';
import { openCloudDocument } from './cloud-document.ts';
import { createCloudBrowser, type CloudEntry, type CloudSource } from './cloud-browser.ts';
import type { LibraryData } from './library-store.ts';
import ChatNameDialog from '../components/ChatNameDialog.vue';
import { folderPath, folderTree, formatWhen, guessPlatform, PLATFORMS, searchText, type LibraryLink, type LibraryStore, type Platform } from './library-store.ts';

const props = defineProps<{ library: LibraryStore; query: string; desktop: boolean; openUrl: (url: string) => void }>();
const emit = defineEmits<{ notify: [message: string, type?: 'success' | 'error' | 'info']; ask: [draft: string] }>();
const confirm = useConfirm();
const browser = createCloudBrowser(invoke);
const activeSource = computed(() => browser.sources.find(source => source.provider === browser.active));
let disposed = false, off: (() => void) | undefined, refreshTimer: ReturnType<typeof setTimeout> | undefined;
async function refreshSources() {
  if (!props.desktop || disposed) return;
  await browser.refreshSources();
  clearTimeout(refreshTimer);
  if (!disposed && browser.sources.some(source => source.status === 'connecting')) refreshTimer = setTimeout(refreshSources, 1500);
}
onMounted(async () => { void refreshSources(); if (props.desktop) { const stop = await listen('ai-store-changed', () => void refreshSources()); if (disposed) stop(); else off = stop; } });
onBeforeUnmount(() => { disposed = true; browser.close(); clearTimeout(refreshTimer); off?.(); });
const managing = ref(''), importing = ref('');
async function manage(source: CloudSource) {
  if (managing.value) return;
  managing.value = source.provider;
  try {
    if (source.connectorId) await openStoreWindow({ view: 'connector', kind: 'connectors', source: 'market', item: { slug: source.connectorId, name: source.name, description: '' } });
    else {
      const slug = source.provider === 'tencent' ? 'tencent-docs' : 'baidu-netdisk';
      const result = await invoke<{ results: { slug: string; name: string; description: string; namespace: string; version: string }[] }>('ai_market_browse', { kind: 'connectors', keyword: source.name, page: 1, category: '', source: '', sortBy: '', variant: '' });
      const item = result.results.find(item => item.slug === slug);
      if (!item) throw new Error('市场暂未找到此连接器，请在 AI 插件中搜索后连接。');
      await openStoreWindow({ view: 'registry', kind: 'connectors', source: 'market', item });
    }
  } catch (e) { emit('notify', String(e), 'error'); }
  finally { managing.value = ''; }
}
function alreadyAdded(entry: CloudEntry) { return props.library.data.links.some(link => link.url === entry.url || (link.remote?.provider === browser.active && link.remote.id === entry.id)); }
async function importEntry(entry: CloudEntry) {
  if (!browser.active || importing.value || alreadyAdded(entry)) return;
  importing.value = entry.id;
  try { props.library.data = await invoke<LibraryData>('library_cloud_add', { platform: browser.active, entry, folder: folder.value || null }); const link = props.library.data.links.find(link => link.remote?.provider === browser.active && link.remote.id === entry.id); if (link) await preview(link); else emit('notify', '已加入资料库'); }
  catch (e) { emit('notify', String(e), 'error'); }
  finally { importing.value = ''; }
}

async function preview(link: LibraryLink) { try { await openCloudDocument(link.id); } catch(e) { emit('notify', String(e), 'error'); } }

const FILTERS: { label: string; match: (p: Platform) => boolean }[] = [
  { label: '全部', match: () => true },
  { label: '飞书', match: p => p === 'feishu' },
  { label: '腾讯文档', match: p => p === 'tencent' },
  { label: '百度网盘', match: p => p === 'baidu' },
  { label: '其他', match: p => !['feishu', 'tencent', 'baidu'].includes(p) },
];
const filter = ref('全部');
const folders = computed(() => props.library.data.folders);
const links = computed(() => props.library.data.links.filter(l => (FILTERS.find(f => f.label === filter.value)?.match ?? (() => true))(l.platform)
  && searchText(props.query, l.title, l.url, l.kind, l.content?.preview, PLATFORMS[l.platform]?.name)));
const info = (link: LibraryLink) => PLATFORMS[link.platform] ?? PLATFORMS.other;

/* ── 添加链接 ── */
const adding = ref(false), url = ref(''), title = ref(''), folder = ref('');
const urlInput = ref<InstanceType<typeof NInput> | null>(null);
const guessed = computed(() => guessPlatform(url.value));
const folderOptions = computed(() => [{ id: '', name: '未分组' }, ...folderTree(props.library.data).filter(r => r.folder).map(r => ({ id: r.id, name: folderPath(folders.value, r.id) }))]);
function openAdd() { adding.value = true; url.value = ''; title.value = ''; folder.value = ''; void nextTick(() => urlInput.value?.focus()); }
watch(adding, open => { if (open) void nextTick(() => urlInput.value?.focus()); });
async function add() {
  if (!guessed.value) return;
  const ok = await props.library.addLink(url.value, title.value, folder.value || null);
  if (ok) { adding.value = false; emit('notify', '已添加到云文档'); }
}
/* ── Cards ── */
const renaming = ref<LibraryLink | null>(null);
async function copy(text: string, what: string) {
  try { await navigator.clipboard.writeText(text); emit('notify', `已复制${what}`); } catch { emit('notify', '复制失败，请重试', 'error'); }
}
function remove(link: LibraryLink) {
  confirm({ title: `移除「${link.title}」？`, content: '只从资料库移除这个链接，云端的文档不受影响。', positiveText: '移除', negativeText: '取消', onPositiveClick: () => props.library.remove([link.id]) });
}
async function rename(name: string) { if (renaming.value) await props.library.renameLink(renaming.value.id, name); renaming.value = null; }
defineExpose({ openAdd });
</script>

<template>
  <div class="cloud" data-testid="library-cloud">
    <div v-if="desktop" class="cloud-sources" aria-label="云文档账号">
      <article v-for="source in browser.sources" :key="source.provider" class="cloud-source" :data-testid="`cloud-source-${source.provider}`"><div><b>{{ source.name }}</b><span class="lib-muted">{{ source.connected ? '已连接' : source.status === 'connecting' ? '连接中…' : '未连接' }}</span></div><button v-if="source.connected && (source.browse || source.search)" class="xb-btn" @click="browser.open(source)">{{ source.browse ? '浏览文件' : '搜索文件' }}</button><button class="xb-btn" :disabled="!!managing" @click="manage(source)">{{ source.connected ? '管理' : '连接' }}</button></article>
    </div>
    <p v-if="browser.sourceError" role="alert" class="cloud-error">{{ browser.sourceError }} <button class="lib-link" @click="refreshSources">重试连接状态</button></p>
    <section v-if="browser.active" class="cloud-browser" aria-label="账号文件浏览">
      <div class="cloud-browser-heading"><button class="lib-icon-btn" aria-label="返回云文档" @click="browser.close"><ArrowLeft :size="16" /></button><h3>{{ activeSource?.name }}</h3><button class="xb-btn" :disabled="browser.loading || !browser.history.length" @click="browser.back">上一级</button><button class="lib-icon-btn" aria-label="刷新账号文件" :disabled="browser.loading" @click="browser.load(browser.page)"><RefreshCw :size="15" /></button></div>
      <form v-if="activeSource?.search" class="cloud-search" @submit.prevent="browser.load()"><Search :size="16" /><select v-model="browser.searchMode" aria-label="云文档搜索范围"><option value="title">文件名</option><option value="content">{{ browser.active === 'baidu' ? '语义搜索' : '正文内容' }}</option></select><input v-model="browser.query" maxlength="80" :placeholder="`搜索${activeSource?.name}文件`" aria-label="搜索账号文件" /><button class="xb-btn" :disabled="browser.loading || (!browser.query.trim() && !activeSource?.browse)" type="submit">搜索</button></form>
      <div class="cloud-browser-location"><span :title="browser.locationLabel">{{ browser.locationLabel || (browser.active === 'tencent' ? '选择一个空间' : '全部文件') }}</span><label>加入到<select v-model="folder" aria-label="云文档保存位置"><option v-for="option in folderOptions" :key="option.id" :value="option.id">{{ option.name }}</option></select></label></div>
      <p v-if="browser.notice" class="lib-muted" role="status">{{ browser.notice }}</p>
      <p v-if="browser.loading" role="status" class="lib-muted">正在读取云端文件…</p>
      <div v-else-if="browser.confirmationRequired" class="cloud-read-confirm"><p>读取{{ activeSource?.name }}的文件列表？本次只查询文件名与位置。</p><button class="xb-btn-primary" @click="browser.load(browser.page, true)">确认读取</button></div>
      <div v-else-if="browser.error" role="alert" class="cloud-error">{{ browser.error }}<button class="lib-link" @click="browser.load(browser.page)">重试</button></div>
      <p v-else-if="!browser.items.length" class="lib-muted nothing">{{ !activeSource?.browse && !browser.query ? '输入关键词搜索账号文件' : '没有找到文件' }}</p>
      <div v-else class="cloud-file-list"><div v-for="entry in browser.items" :key="entry.id" class="cloud-file"><Folder v-if="entry.directory" :size="18" /><FileText v-else :size="18" /><button v-if="entry.directory" class="cloud-file-name" @click="browser.enter(entry)">{{ entry.name }}</button><span v-else class="cloud-file-name" :title="entry.path || entry.name">{{ entry.name }}</span><button v-if="!entry.directory" class="xb-btn" :disabled="!!importing || alreadyAdded(entry)" @click="importEntry(entry)">{{ alreadyAdded(entry) ? '已加入' : importing === entry.id ? '加入中…' : '读取并加入' }}</button><button v-else class="xb-btn" @click="browser.enter(entry)">打开</button></div></div>
      <div class="cloud-pagination"><button class="xb-btn" :disabled="browser.loading || browser.page <= 1" @click="browser.load(browser.page - 1)">上一页</button><span>第 {{ browser.page }} 页</span><button class="xb-btn" :disabled="browser.loading || !browser.hasMore" @click="browser.load(browser.page + 1)">下一页</button></div>
    </section>
    <template v-else>
    <div class="lib-toolbar">
      <div class="lib-chips" aria-label="按平台筛选">
        <button v-for="f in FILTERS" :key="f.label" type="button" :class="{ active: filter === f.label }" :aria-pressed="filter === f.label" @click="filter = f.label">{{ f.label }}</button>
      </div>
      <span class="lib-muted right">已保存 {{ library.data.links.length }} 份云端引用</span>
    </div>
    <div class="lib-scroll grid-wrap">
      <div v-if="!library.data.links.length" class="xb-empty">
        <span class="xb-empty-icon"><Link2 :size="24" /></span>
        <h3>从云端选择资料</h3>
        <p>连接腾讯文档或百度网盘，浏览文件并加入资料库。飞书、石墨、语雀、WPS 等平台可以添加链接。</p>
        <button type="button" class="xb-btn-primary" :disabled="!desktop" @click="openAdd">添加链接</button>
      </div>
      <p v-else-if="!links.length" class="lib-muted nothing">没有找到相关链接</p>
      <div v-else class="grid">
        <article v-for="link in links" :key="link.id" class="lib-card card" :data-testid="`library-link-${link.id}`">
          <div class="card-head">
            <span class="lib-badge badge" :style="{ background: info(link).hue }">{{ info(link).letter }}</span>
            <span class="card-title"><button v-if="link.remote" class="cloud-title-button" :title="link.title" @click="preview(link)">{{ link.title }}</button><b v-else :title="link.title">{{ link.title }}</b><span class="lib-muted">{{ info(link).name }} · {{ link.kind }}</span></span>
          </div>
          <span class="lib-muted note">{{ folderPath(folders, link.folder) }} · {{ formatWhen(link.addedMs) }}添加</span>
          <span v-if="link.code" class="code"><KeyRound :size="12" />提取码 <b>{{ link.code }}</b><button type="button" class="lib-link" @click="copy(link.code!, '提取码')">复制</button></span>
          <div class="card-foot">
            <span class="lib-pill">{{ link.content ? link.content.coverage === 'full' ? '正文已保存' : link.content.coverage === 'partial' ? '摘要／部分内容' : '暂无正文' : link.remote ? '待读取正文' : '仅保存链接' }}</span>
            <button v-if="link.remote" type="button" class="lib-icon-btn" aria-label="预览云文档与提问" title="预览云文档与提问" @click="preview(link)"><MessageCircle :size="14" /></button>
            <button type="button" class="lib-icon-btn push" aria-label="重命名" @click="renaming = link"><Pencil :size="14" /></button>
            <button type="button" class="lib-icon-btn" aria-label="复制链接" @click="copy(link.url, '链接')"><Copy :size="14" /></button>
            <button type="button" class="lib-icon-btn" aria-label="在浏览器打开" :data-testid="`library-open-${link.id}`" @click="openUrl(link.url)"><ExternalLink :size="14" /></button>
            <button type="button" class="lib-icon-btn" aria-label="移除链接" @click="remove(link)"><Trash2 :size="14" /></button>
          </div>
        </article>
      </div>
    </div>
    </template>

    <NModal :show="adding" preset="card" title="添加云文档链接" style="max-width: 480px; width: 92vw;" data-testid="library-link-dialog" @update:show="value => { if (!value) adding = false; }">
      <form class="lib-form" @submit.prevent="add">
        <label>链接<NInput ref="urlInput" v-model:value="url" placeholder="粘贴飞书、腾讯文档、百度网盘等链接" clearable data-testid="library-link-url" /></label>
        <div v-if="url.trim()" class="guess" :class="{ bad: !guessed }" data-testid="library-link-guess">
          <template v-if="guessed"><span class="lib-badge badge sm" :style="{ background: PLATFORMS[guessed].hue }">{{ PLATFORMS[guessed].letter }}</span>识别为 {{ PLATFORMS[guessed].name }}</template>
          <template v-else>这不像一个网址，请粘贴以 http:// 或 https:// 开头的链接</template>
        </div>
        <label>标题<NInput v-model:value="title" placeholder="可不填，之后也能改" :maxlength="120" /></label>
        <label>放到<select v-model="folder" class="folder-select" aria-label="放到文件夹"><option v-for="o in folderOptions" :key="o.id" :value="o.id">{{ o.name }}</option></select></label>
        <p class="lib-muted">腾讯文档与百度网盘复用 MCP 连接，凭据仅保存在本机。其他平台目前保存链接。</p>
        <div class="lib-form-actions">
          <NButton @click="adding = false">取消</NButton>
          <NButton type="primary" attr-type="submit" :loading="library.busy" :disabled="!guessed" data-testid="library-link-add">添加</NButton>
        </div>
      </form>
    </NModal>
    <ChatNameDialog :show="!!renaming" title="重命名链接" placeholder="标题" :initial="renaming?.title ?? ''" :max-length="120" confirm-text="确定" :busy="library.busy" @close="renaming = null" @submit="rename" />
  </div>
</template>

<style scoped>
.cloud{flex:1;min-height:0;display:flex;flex-direction:column;margin-top:14px}
.cloud-sources{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;flex:none;margin-bottom:14px}.cloud-source{display:flex;align-items:center;gap:8px;padding:14px;background:var(--ui-control);border-radius:12px}.cloud-source>div{display:flex;flex:1;align-items:center;gap:10px;min-width:0}.cloud-source b{font-size:13px}.cloud-source .lib-muted{font-size:12px}.cloud-source .xb-btn{padding:4px 7px;white-space:nowrap}.cloud-browser{display:flex;flex-direction:column;min-height:0;flex:1;gap:12px}.cloud-browser-heading,.cloud-search,.cloud-browser-location,.cloud-file,.cloud-pagination{display:flex;align-items:center;gap:10px}.cloud-browser-heading h3{flex:1;margin:0;font-size:15px}.cloud-search{padding:5px 10px;background:var(--ui-control);border-radius:9px}.cloud-search select{border:0;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:12px}.cloud-search input,.cloud-error input{flex:1;min-width:0;border:0;background:transparent;color:var(--ui-text-primary);font:inherit;outline:none}.cloud-browser-location{justify-content:space-between;color:var(--ui-text-secondary);font-size:12px}.cloud-browser-location>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.cloud-browser-location label{display:flex;align-items:center;gap:8px;flex:none}.cloud-browser-location select{max-width:160px;padding:5px;border:0;border-radius:6px;background:var(--ui-control);color:var(--ui-text-primary)}.cloud-file-list{flex:1;min-height:0;overflow:auto}.cloud-file{padding:10px 4px;border-bottom:1px solid var(--ui-line);color:var(--ui-text-secondary)}.cloud-file-name{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;text-align:left;border:0;background:transparent;color:var(--ui-text-primary);font:inherit;font-size:13px}.cloud-file>button{cursor:pointer}.cloud-pagination{justify-content:center;font-size:12px}.cloud-error{font-size:13px;color:var(--ui-danger)}.cloud-error .lib-link{margin-left:10px}@media(max-width:760px){.cloud-sources{grid-template-columns:1fr}.cloud-source>div{flex-wrap:wrap}}
.right{margin-left:auto}
.grid-wrap{margin:0 -28px;padding:14px 28px 24px}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:14px}
.card{padding:14px;display:flex;flex-direction:column;gap:10px;min-width:0}
.card-head{display:flex;align-items:center;gap:10px;min-width:0}
.badge{width:32px;height:32px;border-radius:9px;font-size:12.5px}
.badge.sm{width:22px;height:22px;border-radius:6px;font-size:10.5px}
.card-title{display:flex;flex-direction:column;gap:1px;min-width:0}
.cloud-title-button{border:0;background:none;color:var(--ui-text-primary);text-align:left;cursor:pointer;padding:0}.card-title b,.cloud-title-button{font-size:13.5px;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.note{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.code{display:flex;align-items:center;gap:6px;font-size:12px;color:var(--ui-text-secondary)}
.code b{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--ui-text-primary)}
.card-foot{display:flex;align-items:center;gap:4px;padding-top:10px;border-top:1px solid var(--ui-line)}
.push{margin-left:auto}
.nothing{padding:64px 0;text-align:center;font-size:13px}
.guess{display:flex;align-items:center;gap:8px;margin-top:-6px;font-size:12.5px;color:var(--ui-text-secondary)}
.guess.bad{color:var(--ui-danger)}
.folder-select{height:34px;border:0;border-radius:8px;padding:0 10px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:13px}
.connect{display:flex;flex-direction:column;gap:8px;padding:12px;border-radius:10px;background:var(--ui-surface-sunken,var(--ui-control))}
.connect .lib-muted{line-height:1.6}
.connect-row{display:flex;align-items:center;gap:8px;font-size:13px;color:var(--ui-text-primary)}
.soon{margin-left:auto}
</style>
