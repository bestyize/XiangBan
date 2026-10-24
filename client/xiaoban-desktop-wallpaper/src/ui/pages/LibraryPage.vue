<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { FolderUp, Plus, Search, Upload } from 'lucide-vue-next';
import { invoke, listen } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { createLibraryStore, searchText, type LibraryDoc, type LibrarySaved } from '../library/library-store.ts';
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
const searchBox = ref<HTMLInputElement | null>(null);
function focusSearch(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') { event.preventDefault(); searchBox.value?.focus(); }
}
onMounted(() => window.addEventListener('keydown', focusSearch));
onBeforeUnmount(() => window.removeEventListener('keydown', focusSearch));

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
        <label class="xb-search library-search">
          <Search :size="14" aria-hidden="true" />
          <input ref="searchBox" v-model="query" type="search" maxlength="80" aria-label="搜索资料库" placeholder="搜索全部资料" data-testid="library-search" />
          <kbd v-if="!query">⌘K</kbd>
        </label>
        <template v-if="tab === 'docs'">
          <button type="button" class="xb-btn" :disabled="!store.desktop" data-testid="library-import-folder" @click="docsTab?.importFrom(true)"><FolderUp :size="14" />导入文件夹</button>
          <button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="library-import" @click="docsTab?.importFrom(false)"><Upload :size="14" />导入文档</button>
        </template>
        <button v-else-if="tab === 'cloud'" type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="library-add-link" @click="cloudTab?.openAdd()"><Plus :size="14" />添加链接</button>
        <button v-else-if="tab === 'memory'" type="button" class="xb-btn-primary" :disabled="!store.desktop || !library.memory" data-testid="memory-write" @click="memoryTab?.write()"><Plus :size="14" />写一条</button>
      </div>
    </header>
    <div :id="`library-panel-${tab}`" class="library-body" role="tabpanel" :aria-labelledby="`library-tab-${tab}`">
      <LibraryDocs v-if="tab === 'docs'" ref="docsTab" :library="library" :query="query" :desktop="store.desktop" @ask="askWith" @notify="notify" />
      <LibraryCloud v-else-if="tab === 'cloud'" ref="cloudTab" :library="library" :query="query" :desktop="store.desktop" :open-url="openUrl" @notify="notify" />
      <LibrarySavedTab v-else-if="tab === 'saved'" :library="library" :query="query" @open="item => item.sessionId && openSession(item.sessionId)" @ask="askAbout" @notify="notify" />
      <LibraryMemory v-else ref="memoryTab" :library="library" :query="query" :desktop="store.desktop" :session-title="sessionTitle" @open-session="openSession" />
    </div>
  </section>
</template>

<style scoped>
.library-page{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column;padding-bottom:0;color:var(--ui-text-primary);background:var(--ui-surface)}
.library-head{flex:none}
.xb-tabs-lg>button{display:inline-flex;align-items:baseline;gap:6px}
.count{font-size:12px;font-weight:500;color:var(--ui-text-tertiary)}
.library-search{width:240px;height:32px;padding:0 10px;gap:8px;border-radius:9px;background:var(--ui-control);box-sizing:border-box}
.library-search input{flex:1;min-width:0}
.library-search kbd{font:11px -apple-system,sans-serif;color:var(--ui-text-tertiary)}
.library-body{flex:1;min-height:0;display:flex;flex-direction:column}
</style>
