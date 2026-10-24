<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NButton, NInput, NModal, NSelect } from 'naive-ui';
import { BookmarkPlus, FilePlus2, FolderUp, Link2, NotebookPen, Search, Upload, X } from 'lucide-vue-next';
import { invoke, listen } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { createLibraryStore, folderPath, folderTree, searchText, type LibraryDoc, type LibrarySaved } from '../library/library-store.ts';
import LibraryDocs from '../library/LibraryDocs.vue';
import LibraryCloud from '../library/LibraryCloud.vue';
import LibrarySavedTab from '../library/LibrarySaved.vue';
import LibraryMemory from '../library/LibraryMemory.vue';
import '../library/library.css';

const props = defineProps<{ store: ControlStore }>();
const notify = (message: unknown, type: 'success' | 'error' | 'info' = 'success') => props.store.notify(message, type);
const library = createLibraryStore({ desktop: props.store.desktop, invoke, listen, notify });
onMounted(() => { void library.init(); });
onBeforeUnmount(() => library.dispose());
/** A note the assistant has just saved shows up in 记忆 without leaving and re-entering the page. */
const notesSaved = computed(() => Object.values(props.store.chat?.views ?? {}).reduce((sum, view) => sum + view.items.filter(item => item.kind === 'tool' && item.name === 'memory_write' && item.status === 'done').length, 0));
watch(notesSaved, () => { void library.refreshMemory(); });

type Tab = 'docs' | 'cloud' | 'saved' | 'memory';
const TAB_KEY = 'xiaoban.library-tab';
const saved = (() => { try { return localStorage.getItem(TAB_KEY) as Tab | null; } catch { return null; } })();
const tab = ref<Tab>(saved && ['docs', 'cloud', 'saved', 'memory'].includes(saved) ? saved : 'docs');
function pick(next: Tab) { tab.value = next; try { localStorage.setItem(TAB_KEY, next); } catch { /* only a convenience */ } }
const query = ref('');

/** Each tab shows how many items match the search, so a search finds things in the other tabs too. */
const counts = computed(() => {
  const q = query.value, d = library.data, m = library.memory;
  return {
    docs: d.docs.filter(x => searchText(q, x.name, x.preview, x.tags.join(' '))).length,
    cloud: d.links.filter(x => searchText(q, x.title, x.url, x.kind)).length,
    saved: d.saved.filter(x => searchText(q, x.title, x.text, x.sessionTitle)).length,
    memory: m ? m.notes.filter(n => searchText(q, n.text)).length + m.approved.filter(e => searchText(q, e.content)).length + m.candidates.filter(e => searchText(q, e.content)).length + m.goals.filter(g => searchText(q, g.outcome)).length : 0,
  };
});
const TABS: { id: Tab; label: string }[] = [{ id: 'docs', label: '我的文档' }, { id: 'cloud', label: '云文档' }, { id: 'saved', label: '收藏' }, { id: 'memory', label: '记忆' }];
function tabKey(event: KeyboardEvent) {
  const n = TABS.findIndex(t => t.id === tab.value);
  if (event.key === 'ArrowRight') pick(TABS[(n + 1) % TABS.length].id);
  else if (event.key === 'ArrowLeft') pick(TABS[(n + TABS.length - 1) % TABS.length].id);
  else return;
  event.preventDefault();
  document.getElementById(`library-tab-${tab.value}`)?.focus();
}

const docsTab = ref<InstanceType<typeof LibraryDocs> | null>(null);
const cloudTab = ref<InstanceType<typeof LibraryCloud> | null>(null);
const memoryTab = ref<InstanceType<typeof LibraryMemory> | null>(null);
const savedTab = ref<InstanceType<typeof LibrarySavedTab> | null>(null);
const creating = ref<'doc' | 'saved' | null>(null), createTitle = ref(''), createText = ref(''), createFolder = ref(''), createFormat = ref('md');
const creatingBusy = ref(false), createError = ref('');
const createDialogTitle = ref('');
const createFolders = computed(() => [{ label: '未分组', value: '' }, ...folderTree(library.data).filter(row => row.folder).map(row => ({ label: folderPath(library.data.folders, row.id), value: row.id }))]);
function openCreate(kind: 'doc' | 'saved') {
  creating.value = kind; createTitle.value = ''; createText.value = ''; createError.value = ''; createFormat.value = 'md';
  createDialogTitle.value = kind === 'doc' ? '新建文档' : '添加收藏';
  createFolder.value = kind === 'doc' ? docsTab.value?.currentFolder() ?? '' : '';
}
async function saveCreated() {
  if (creatingBusy.value || !createText.value.trim() || (creating.value === 'doc' && !createTitle.value.trim())) return;
  const kind = creating.value;
  if (!kind) return;
  if (kind === 'doc' && /[\\/:*?"<>|\u0000-\u001f]/.test(createTitle.value)) { createError.value = '文档名称不能包含路径分隔符或特殊字符 \\ / : * ? " < > |'; return; }
  creatingBusy.value = true; createError.value = '';
  try {
    if (kind === 'doc') {
      const folder = createFolder.value || null;
      const name = `${createTitle.value.trim().replace(/\.(md|txt)$/i, '')}.${createFormat.value}`;
      const id = await library.createDocument(name, createText.value, folder);
      if (!id) { createError.value = '文档未创建成功，请检查错误提示后重试。'; return; }
      query.value = ''; docsTab.value?.showCreated(id, folder);
      notify('文档已创建');
    } else {
      if (!await library.createSaved(createTitle.value.trim(), createText.value)) { createError.value = '收藏未保存成功，请检查错误提示后重试。'; return; }
      query.value = ''; await nextTick(); savedTab.value?.showAll(); notify('已添加收藏');
    }
    creating.value = null;
  } finally { creatingBusy.value = false; }
}
const searchBox = ref<HTMLInputElement | null>(null);
const searchButton = ref<HTMLButtonElement | null>(null), searchOpen = ref(false);
async function openSearch() { searchOpen.value = true; await nextTick(); searchBox.value?.focus(); }
function closeSearch() { query.value = ''; searchOpen.value = false; searchButton.value?.focus(); }
function toggleSearch() { if (searchOpen.value) closeSearch(); else void openSearch(); }
function focusSearch(event: KeyboardEvent) {
  if (props.store.page && props.store.page !== 'library') return;
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') { event.preventDefault(); event.stopImmediatePropagation(); void openSearch(); }
}
onMounted(() => window.addEventListener('keydown', focusSearch, true));
onBeforeUnmount(() => window.removeEventListener('keydown', focusSearch, true));

/* ── Into a conversation ── */
async function startTask(draft: string, attachments: NonNullable<ControlStore['creationAttachments']> = []) {
  await props.store.chat.select(null);
  props.store.creationAttachments = attachments.length ? attachments : null;
  props.store.creationDraft = draft;
  props.store.navigate('new-task');
}
async function askWith(doc: LibraryDoc) {
  const attachment = await library.attachment(doc.id);
  if (attachment) await startTask(`请参考资料库里的「${doc.name}」：`, [attachment]);
}
async function openSession(id: string) {
  if (!props.store.chat.sessions.some(s => s.id === id)) { notify('原来的对话已经删除了', 'info'); return; }
  await props.store.chat.select(id);
  props.store.navigate('chat');
}
const askAbout = (item: LibrarySaved) => startTask(`参考我收藏的这段内容：\n\n${item.text.slice(0, 4000)}\n\n`);
const sessionTitle = (id: string) => props.store.chat.sessions.find(s => s.id === id)?.title ?? '';
const openUrl = (url: string) => { void invoke('open_download_url', { url }).catch(() => notify('这个链接打不开', 'error')); };
</script>

<template>
  <section class="library-page xb-page" data-testid="library-page" aria-label="资料库">
    <header class="xb-page-head library-head" data-tauri-drag-region>
      <div class="xb-tabs-lg" role="tablist" aria-label="资料库分类" @keydown="tabKey">
        <button v-for="t in TABS" :id="`library-tab-${t.id}`" :key="t.id" type="button" role="tab" :aria-selected="tab === t.id" :tabindex="tab === t.id ? 0 : -1" :data-testid="`library-tab-${t.id}`" @click="pick(t.id)">{{ t.label }}<span class="count">{{ counts[t.id] }}</span></button>
      </div>
      <div class="xb-page-actions">
        <button ref="searchButton" type="button" class="xb-icon-btn library-action" :class="{ 'is-active': searchOpen || !!query }" aria-label="搜索资料库" :title="searchOpen ? '收起搜索' : '搜索资料库'" :aria-expanded="searchOpen" aria-controls="library-search-field" data-testid="library-search-toggle" @click="toggleSearch"><Search :size="17" /></button>
        <label v-show="searchOpen" id="library-search-field" class="xb-search library-search">
          <input ref="searchBox" v-model="query" type="search" maxlength="80" aria-label="搜索资料库" placeholder="搜索全部资料" data-testid="library-search" @keydown.esc.prevent.stop="closeSearch" />
          <button v-if="query" type="button" class="xb-icon-btn is-sm" aria-label="清空搜索" title="清空搜索" @click="query = ''; searchBox?.focus()"><X :size="12" /></button>
        </label>
        <template v-if="tab === 'docs'">
          <button type="button" class="xb-icon-btn library-action" :disabled="!store.desktop" aria-label="导入文件夹" title="导入文件夹" data-testid="library-import-folder" @click="docsTab?.importFrom(true)"><FolderUp :size="17" /></button>
          <button type="button" class="xb-icon-btn library-action" :disabled="!store.desktop" aria-label="导入文档" title="导入文档" data-testid="library-import" @click="docsTab?.importFrom(false)"><Upload :size="17" /></button>
          <button type="button" class="xb-icon-btn library-action" :disabled="!store.desktop || library.busy" aria-label="新建文档" title="新建文档" data-testid="library-create-doc" @click="openCreate('doc')"><FilePlus2 :size="17" /></button>
        </template>
        <button v-else-if="tab === 'cloud'" type="button" class="xb-icon-btn library-action" :disabled="!store.desktop" aria-label="添加链接" title="添加链接" data-testid="library-add-link" @click="cloudTab?.openAdd()"><Link2 :size="17" /></button>
        <button v-else-if="tab === 'saved'" type="button" class="xb-icon-btn library-action" :disabled="!store.desktop || library.busy" aria-label="添加收藏" title="添加收藏" data-testid="library-create-saved" @click="openCreate('saved')"><BookmarkPlus :size="17" /></button>
        <button v-else-if="tab === 'memory'" type="button" class="xb-icon-btn library-action" :disabled="!store.desktop || !library.memory || library.busy" aria-label="新建记忆" title="新建记忆" data-testid="memory-write" @click="memoryTab?.write()"><NotebookPen :size="17" /></button>
      </div>
    </header>
    <div :id="`library-panel-${tab}`" class="library-body" role="tabpanel" :aria-labelledby="`library-tab-${tab}`">
      <LibraryDocs v-if="tab === 'docs'" ref="docsTab" :library="library" :query="query" :desktop="store.desktop" @ask="askWith" @notify="notify" />
      <LibraryCloud v-else-if="tab === 'cloud'" ref="cloudTab" :library="library" :query="query" :desktop="store.desktop" :open-url="openUrl" @notify="notify" />
      <LibrarySavedTab v-else-if="tab === 'saved'" ref="savedTab" :library="library" :query="query" @open="item => item.sessionId && openSession(item.sessionId)" @ask="askAbout" @notify="notify" />
      <LibraryMemory v-else ref="memoryTab" :library="library" :query="query" :desktop="store.desktop" :session-title="sessionTitle" @open-session="openSession" @created="query = ''" />
    </div>
    <NModal :show="!!creating" preset="card" :title="createDialogTitle" :mask-closable="!creatingBusy" :close-on-esc="!creatingBusy" :closable="!creatingBusy" style="width:560px;max-width:calc(100vw - 32px);max-height:calc(100dvh - 48px);overflow:auto" @update:show="show => { if (!show && !creatingBusy) creating = null; }">
      <form v-if="creating" class="lib-form creation-form" data-testid="library-create-form" @submit.prevent="saveCreated">
        <p class="lib-muted">{{ creating === 'doc' ? '直接编写内容，保存为资料库中的文本文件。' : '保存值得留住的文字、链接或想法，不必先发起对话。' }}</p>
        <label>{{ creating === 'doc' ? '文档名称' : '标题（选填）' }}<NInput v-model:value="createTitle" :maxlength="creating === 'doc' ? 100 : 80" :disabled="creatingBusy" :placeholder="creating === 'doc' ? '例如：本周工作计划' : '留空时使用正文首行'" data-testid="library-create-title" /></label>
        <div v-if="creating === 'doc'" class="creation-options"><label>保存位置<NSelect v-model:value="createFolder" :options="createFolders" :disabled="creatingBusy" data-testid="library-create-folder" /></label><label>文档格式<NSelect v-model:value="createFormat" :options="[{ label: 'Markdown（.md）', value: 'md' }, { label: '纯文本（.txt）', value: 'txt' }]" :disabled="creatingBusy" data-testid="library-create-format" /></label></div>
        <label>正文<NInput v-model:value="createText" type="textarea" :autosize="{ minRows: 7, maxRows: 12 }" :maxlength="creating === 'doc' ? 100000 : 50000" :disabled="creatingBusy" :placeholder="creating === 'doc' ? '在这里编写文档内容…' : '写下你想收藏的内容…'" data-testid="library-create-text" /></label>
        <p v-if="createError" role="alert" class="creation-error">{{ createError }}</p>
        <div class="lib-form-actions"><NButton :disabled="creatingBusy" @click="creating = null">取消</NButton><NButton type="primary" attr-type="submit" :loading="creatingBusy" :disabled="creatingBusy || !createText.trim() || (creating === 'doc' && !createTitle.trim())" data-testid="library-create-save">{{ creating === 'doc' ? '创建文档' : '添加收藏' }}</NButton></div>
      </form>
    </NModal>
  </section>
</template>

<style scoped>
.library-page{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column;padding-bottom:0;color:var(--ui-text-primary);background:var(--ui-surface)}
.library-head{flex:none;flex-wrap:wrap;row-gap:14px}
.xb-tabs-lg>button{display:inline-flex;align-items:baseline;gap:6px}
.count{font-size:12px;font-weight:500;color:var(--ui-text-tertiary)}
.library-action{width:32px;height:32px;border-radius:9px}.library-search{width:220px;height:32px;padding:0 10px;gap:6px;border-radius:9px;background:var(--ui-control);box-sizing:border-box}
.library-search input{flex:1;min-width:0}
.library-search input::-webkit-search-cancel-button{-webkit-appearance:none}.library-search .xb-icon-btn{flex:none}
.library-body{flex:1;min-height:0;display:flex;flex-direction:column}
.creation-form>p{margin:0;line-height:1.7}.creation-options{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:12px}.creation-options label{min-width:0}.creation-error{color:var(--ui-danger);font-size:12px}@media(max-width:480px){.creation-options{grid-template-columns:1fr}}
</style>
