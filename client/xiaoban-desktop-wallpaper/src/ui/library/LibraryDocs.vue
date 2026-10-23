<script setup lang="ts">
import { useConfirm } from './confirm.ts';
import { computed, ref, watch } from 'vue';
import { AtSign, FileText, Folder, FolderOpen, FolderPlus, LayoutList, Pencil, Plus, Trash2, X } from 'lucide-vue-next';
import ChatNameDialog from '../components/ChatNameDialog.vue';
import {
  DOC_FILTERS, DOC_STATUS, folderPath, folderTree, formatSize, formatWhen, inView, matchesFilter, pathOf, searchText,
  type FolderView, type LibraryDoc, type LibraryStore,
} from './library-store.ts';

const props = defineProps<{ library: LibraryStore; query: string; desktop: boolean }>();
const emit = defineEmits<{ ask: [doc: LibraryDoc]; notify: [message: string, type?: 'success' | 'error' | 'info'] }>();
const confirm = useConfirm();
const isMac = /Mac/i.test(navigator.userAgent);

const view = ref<FolderView>('all');
const filter = ref('全部');
const sort = ref<'recent' | 'name'>('recent');
const selected = ref<string | null>(null);
const data = computed(() => props.library.data);
const folders = computed(() => data.value.folders);
const realFolder = computed(() => folders.value.some(f => f.id === view.value) ? view.value : null);
// The folder that was open went away (deleted elsewhere): back to everything.
watch(folders, list => { if (view.value !== 'all' && view.value !== 'none' && !list.some(f => f.id === view.value)) view.value = 'all'; });

const tree = computed(() => folderTree(data.value));
const searching = computed(() => props.query.trim() !== '');
const crumbs = computed(() => [{ id: view.value === 'none' ? 'none' : 'all', name: view.value === 'none' ? '未分组' : '全部文档' }, ...pathOf(folders.value, realFolder.value).map(f => ({ id: f.id, name: f.name }))]);
const subfolders = computed(() => searching.value || !realFolder.value ? [] : folders.value.filter(f => f.parent === realFolder.value).sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'))
  .map(f => ({ ...f, count: data.value.docs.filter(d => inView(folders.value, d, f.id)).length })));
/** In a folder: the files directly in it. In the two views, and while searching, every matching file. */
const docs = computed(() => {
  const list = data.value.docs.filter(d => (searching.value ? true : realFolder.value ? d.folder === realFolder.value : inView(folders.value, d, view.value))
    && matchesFilter(d, filter.value) && searchText(props.query, d.name, d.preview, d.tags.join(' ')));
  return sort.value === 'name' ? list.sort((a, b) => a.name.localeCompare(b.name, 'zh-CN')) : list.sort((a, b) => b.importedMs - a.importedMs);
});
const current = computed(() => docs.value.find(d => d.id === selected.value) ?? docs.value[0] ?? null);
const showWhere = computed(() => searching.value || !realFolder.value);
const folderOptions = computed(() => [{ id: '', name: '未分组' }, ...tree.value.filter(r => r.folder).map(r => ({ id: r.id, name: folderPath(folders.value, r.id) }))]);
const totalBytes = computed(() => data.value.docs.reduce((sum, d) => sum + d.size, 0));
const empty = computed(() => !data.value.docs.length && !folders.value.length);

const KINDS: Record<string, [string, string, string]> = {
  pdf: ['PDF', '#FDECEC', '#C4323A'], docx: ['DOC', '#E8F0FD', '#2F6FDB'], doc: ['DOC', '#E8F0FD', '#2F6FDB'],
  xlsx: ['XLS', '#E6F5EC', '#23885A'], xls: ['XLS', '#E6F5EC', '#23885A'], csv: ['CSV', '#E6F5EC', '#23885A'],
  pptx: ['PPT', '#FDF0E6', '#D2691E'], ppt: ['PPT', '#FDF0E6', '#D2691E'],
  png: ['IMG', '#F1ECFD', '#6A4BD6'], jpg: ['IMG', '#F1ECFD', '#6A4BD6'], jpeg: ['IMG', '#F1ECFD', '#6A4BD6'], gif: ['IMG', '#F1ECFD', '#6A4BD6'], webp: ['IMG', '#F1ECFD', '#6A4BD6'],
};
const kind = (doc: LibraryDoc) => KINDS[doc.ext] ?? [(doc.ext || 'FILE').slice(0, 4).toUpperCase(), '#EFEFF2', '#4A4A50'];
const status = (doc: LibraryDoc) => DOC_STATUS[doc.status] ?? DOC_STATUS.unsupported;

function open(id: FolderView) { view.value = id; selected.value = null; }

/* ── Folders ── */
const naming = ref<null | { mode: 'create' | 'rename-folder' | 'rename-doc'; id?: string; initial: string }>(null);
const dialogTitle = computed(() => naming.value?.mode === 'create' ? (realFolder.value ? `在「${folders.value.find(f => f.id === realFolder.value)?.name}」里新建文件夹` : '新建文件夹') : naming.value?.mode === 'rename-doc' ? '重命名文档' : '重命名文件夹');
async function submitName(name: string) {
  const job = naming.value;
  if (!job) return;
  if (job.mode === 'create') { const id = await props.library.createFolder(name, realFolder.value); if (id) open(id); }
  else if (job.mode === 'rename-folder' && job.id) await props.library.renameFolder(job.id, name);
  else if (job.mode === 'rename-doc' && job.id) await props.library.renameDoc(job.id, name);
  naming.value = null;
}
function deleteFolder(id: string, name: string) {
  confirm({
    title: `删除文件夹「${name}」？`, content: '里面的文档和文件夹不会删除，会移到上一级。', positiveText: '删除文件夹', negativeText: '取消',
    onPositiveClick: async () => { if (view.value === id) open(folders.value.find(f => f.id === id)?.parent ?? 'all'); await props.library.deleteFolder(id); },
  });
}
function removeDoc(doc: LibraryDoc) {
  confirm({
    title: `从资料库删除「${doc.name}」？`, content: '只删除资料库里的这份副本，原文件不受影响。', positiveText: '删除', negativeText: '取消',
    onPositiveClick: async () => { await props.library.remove([doc.id]); selected.value = null; },
  });
}

/* ── Tags ── */
const tagging = ref(false), tagText = ref('');
async function addTag(doc: LibraryDoc) {
  const tag = tagText.value.trim();
  tagging.value = false; tagText.value = '';
  if (tag && !doc.tags.includes(tag)) await props.library.setTags(doc.id, [...doc.tags, tag]);
}

/* ── Moving by drag, importing by drop ── */
const DRAG = 'application/x-xiaoban-library-doc';
const dropTarget = ref<string | null>(null), droppingFiles = ref(false);
function dragDoc(event: DragEvent, doc: LibraryDoc) { event.dataTransfer?.setData(DRAG, doc.id); if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move'; }
function overFolder(event: DragEvent, id: string) {
  if (id === 'all' || !event.dataTransfer?.types.includes(DRAG)) return;
  event.preventDefault(); dropTarget.value = id;
}
async function dropOnFolder(event: DragEvent, id: string) {
  const doc = event.dataTransfer?.getData(DRAG);
  dropTarget.value = null;
  if (!doc || id === 'all') return;
  event.preventDefault();
  await props.library.move([doc], id === 'none' ? null : id);
}
const MAX_DROP = 50 * 1024 * 1024;
function overList(event: DragEvent) { if (event.dataTransfer?.types.includes('Files')) { event.preventDefault(); droppingFiles.value = true; } }
async function dropFiles(event: DragEvent) {
  droppingFiles.value = false;
  const files = [...(event.dataTransfer?.files ?? [])];
  if (!files.length) return;
  event.preventDefault();
  for (const file of files) {
    if (file.size > MAX_DROP) { emit('notify', `${file.name} 超过 50MB，请用「导入文档」选择`, 'error'); continue; }
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => typeof reader.result === 'string' ? resolve(reader.result) : reject(new Error('读取失败'));
      reader.onerror = () => reject(new Error('读取失败'));
      reader.readAsDataURL(file);
    }).catch(() => null);
    if (dataUrl) await props.library.importFile(file.name, dataUrl, realFolder.value);
    else emit('notify', `${file.name} 读取失败`, 'error');
  }
}

async function importFrom(directory: boolean) { await props.library.pickImport(realFolder.value, directory); }
defineExpose({ importFrom, newFolder: () => { naming.value = { mode: 'create', initial: '' }; } });
</script>

<template>
  <div class="docs" data-testid="library-docs">
    <nav class="tree" aria-label="文件夹">
      <div v-for="row in tree" :key="row.id" class="tree-row" :class="{ current: view === row.id, target: dropTarget === row.id }" :style="{ paddingLeft: `${8 + row.depth * 16}px` }"
        @dragover="overFolder($event, row.id)" @dragleave="dropTarget = null" @drop="dropOnFolder($event, row.id)">
        <button type="button" class="tree-open" :aria-current="view === row.id ? 'true' : undefined" :data-testid="`library-folder-${row.id}`" @click="open(row.id)">
          <LayoutList v-if="row.id === 'all'" :size="15" /><FileText v-else-if="row.id === 'none'" :size="15" /><FolderOpen v-else-if="view === row.id" :size="15" /><Folder v-else :size="15" />
          <span class="tree-name">{{ row.name }}</span><span class="tree-count">{{ row.count }}</span>
        </button>
        <span v-if="row.folder" class="tree-actions">
          <button type="button" class="lib-icon-btn" :aria-label="`重命名文件夹 ${row.name}`" @click="naming = { mode: 'rename-folder', id: row.id, initial: row.name }"><Pencil :size="12" /></button>
          <button type="button" class="lib-icon-btn" :aria-label="`删除文件夹 ${row.name}`" @click="deleteFolder(row.id, row.name)"><Trash2 :size="12" /></button>
        </span>
      </div>
      <button type="button" class="tree-new" :disabled="!desktop" data-testid="library-new-folder" @click="naming = { mode: 'create', initial: '' }"><FolderPlus :size="15" />新建文件夹</button>
      <p class="tree-foot">共 {{ data.docs.length }} 份 · {{ formatSize(totalBytes) }}<br>文件复制存在本机，原文件移走也不影响</p>
    </nav>

    <div class="main">
      <div class="crumbs">
        <template v-if="!searching">
          <template v-for="(crumb, n) in crumbs" :key="crumb.id">
            <button type="button" class="lib-link" :class="{ muted: n < crumbs.length - 1, here: n === crumbs.length - 1 }" @click="open(crumb.id)">{{ crumb.name }}</button>
            <span v-if="n < crumbs.length - 1" class="sep" aria-hidden="true">/</span>
          </template>
        </template>
        <span v-else class="lib-muted">搜索「{{ query.trim() }}」，在全部文件夹里找到 {{ docs.length }} 份</span>
        <div class="seg" role="group" aria-label="排序">
          <button type="button" :class="{ on: sort === 'recent' }" :aria-pressed="sort === 'recent'" @click="sort = 'recent'">最近</button>
          <button type="button" :class="{ on: sort === 'name' }" :aria-pressed="sort === 'name'" @click="sort = 'name'">名称</button>
        </div>
      </div>
      <div class="lib-toolbar">
        <div class="lib-chips" aria-label="按类型筛选">
          <button v-for="f in DOC_FILTERS" :key="f" type="button" :class="{ active: filter === f }" :aria-pressed="filter === f" @click="filter = f">{{ f }}</button>
        </div>
      </div>

      <div class="list lib-scroll" :class="{ 'lib-dropping': droppingFiles }" role="list" aria-label="文档" data-testid="library-doc-list" @dragover="overList" @dragleave="droppingFiles = false" @drop="dropFiles">
        <div v-if="empty" class="xb-empty first">
          <span class="xb-empty-icon"><FileText :size="24" /></span>
          <h3>把常用文件放进来</h3>
          <p>拖入文件，或导入一个文件夹（会保留原来的层级）。资料库里存的是副本，小伴读完后就能在回答里参考。</p>
          <div class="first-actions"><button type="button" class="xb-btn" :disabled="!desktop" @click="importFrom(true)">导入文件夹</button><button type="button" class="xb-btn-primary" :disabled="!desktop" @click="importFrom(false)">导入文档</button></div>
        </div>
        <template v-else>
          <button v-for="f in subfolders" :key="f.id" type="button" role="listitem" class="row" :data-testid="`library-subfolder-${f.id}`" @click="open(f.id)">
            <span class="row-main"><span class="folder-icon"><Folder :size="24" :stroke-width="1.5" /></span>
              <span class="row-text"><b>{{ f.name }}</b><span class="lib-muted">{{ f.count }} 份 · 文件夹</span></span></span>
            <span class="lib-muted">打开 ›</span>
          </button>
          <button v-for="doc in docs" :key="doc.id" type="button" role="listitem" class="row" :class="{ selected: current?.id === doc.id }" :aria-pressed="current?.id === doc.id" draggable="true" :data-testid="`library-doc-${doc.id}`"
            @click="selected = doc.id" @dragstart="dragDoc($event, doc)">
            <span class="row-main">
              <span class="file-icon" :style="{ background: kind(doc)[1], color: kind(doc)[2] }">{{ kind(doc)[0] }}</span>
              <span class="row-text"><b>{{ doc.name }}</b><span class="lib-muted">{{ kind(doc)[0] }} · {{ formatSize(doc.size) }} · {{ formatWhen(doc.importedMs) }}<template v-if="showWhere"> · {{ folderPath(folders, doc.folder) }}</template></span></span>
            </span>
            <span class="lib-pill" :class="status(doc).tone"><i></i>{{ status(doc).label }}</span>
          </button>
          <p v-if="!docs.length && !subfolders.length" class="lib-muted nothing">{{ searching ? '没有找到相关文档' : '这个文件夹是空的，把文件拖进来就会放到这里' }}</p>
          <p class="lib-muted hint">把文件拖到左侧的文件夹即可移动；导入整个文件夹时会保留原来的层级</p>
        </template>
      </div>
    </div>

    <aside v-if="current" class="detail" aria-label="文档详情" data-testid="library-doc-detail">
      <div class="thumb"><span class="sheet"><i :style="{ background: kind(current)[1] }"></i><i></i><i></i><i class="short"></i><i></i></span><span class="thumb-kind" :style="{ color: kind(current)[2] }">{{ kind(current)[0] }}</span></div>
      <div class="title-row"><b class="doc-title">{{ current.name }}</b><button type="button" class="lib-icon-btn" aria-label="重命名" @click="naming = { mode: 'rename-doc', id: current.id, initial: current.name }"><Pencil :size="13" /></button></div>
      <span class="lib-muted">{{ kind(current)[0] }} · {{ formatSize(current.size) }} · 导入于 {{ formatWhen(current.importedMs) }}</span>
      <div class="excerpt" :class="{ quiet: current.status !== 'ready' || !current.preview }">
        <span class="excerpt-head">{{ current.status === 'ready' && current.preview ? '内容开头' : status(current).label }}</span>
        <span class="excerpt-text">{{ current.status === 'ready' ? (current.preview || '这是一张图片，提问时小伴会直接看图。') : current.status === 'reading' ? '正在读取文字，读完后就能参考。' : current.note }}</span>
      </div>
      <dl class="facts">
        <dt>文件夹</dt>
        <dd><select class="move" :value="current.folder ?? ''" aria-label="移动到文件夹" data-testid="library-move" @change="library.move([current.id], ($event.target as HTMLSelectElement).value || null)"><option v-for="o in folderOptions" :key="o.id" :value="o.id">{{ o.name }}</option></select></dd>
        <dt>标签</dt>
        <dd class="tags">
          <span v-for="tag in current.tags" :key="tag" class="lib-tag">{{ tag }}<button type="button" class="untag" :aria-label="`去掉标签 ${tag}`" @click="library.setTags(current.id, current.tags.filter(t => t !== tag))"><X :size="10" /></button></span>
          <input v-if="tagging" v-model="tagText" class="tag-input" maxlength="20" aria-label="新标签" placeholder="回车添加" @keydown.enter.prevent="addTag(current)" @blur="addTag(current)" />
          <button v-else type="button" class="lib-tag add" @click="tagging = true"><Plus :size="11" />标签</button>
        </dd>
        <template v-if="current.origin"><dt>原文件</dt><dd class="origin">{{ current.origin }}</dd></template>
      </dl>
      <div class="detail-actions">
        <button type="button" class="xb-btn-primary" :disabled="!desktop || !['ready', 'reading'].includes(current.status)" :title="['ready', 'reading'].includes(current.status) ? '' : current.note" data-testid="library-ask" @click="emit('ask', current)"><AtSign :size="14" />用这份资料提问</button>
        <div class="pair">
          <button type="button" class="xb-btn" :disabled="!desktop" @click="library.reveal(current.id)"><FolderOpen :size="14" />{{ isMac ? '在访达中显示' : '在文件夹中显示' }}</button>
          <button type="button" class="xb-btn is-danger" aria-label="从资料库删除" @click="removeDoc(current)"><Trash2 :size="14" /></button>
        </div>
      </div>
    </aside>

    <ChatNameDialog :show="!!naming" :title="dialogTitle" placeholder="名称" :initial="naming?.initial ?? ''" :max-length="120" confirm-text="确定" :busy="library.busy" @close="naming = null" @submit="submitName" />
  </div>
</template>

<style scoped>
.docs{flex:1;min-height:0;display:flex;margin-top:14px}
.tree{width:188px;flex:none;display:flex;flex-direction:column;gap:1px;padding:2px 12px 16px 0;border-right:1px solid var(--ui-line);overflow-y:auto}
.tree-row{position:relative;display:flex;align-items:center;border-radius:8px}
.tree-row:hover{background:var(--ui-control)}
.tree-row.current{background:var(--ui-soft)}
.tree-row.target{box-shadow:inset 0 0 0 1.5px var(--ui-btn)}
.tree-open{flex:1;min-width:0;display:flex;align-items:center;gap:7px;height:32px;padding:0 8px 0 0;border:0;background:transparent;color:var(--ui-text-primary);font:inherit;font-size:13px;text-align:left;cursor:pointer}
.tree-row.current .tree-open{color:var(--ui-ink);font-weight:600}
.tree-name{flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.tree-count{font-size:11.5px;color:var(--ui-text-tertiary);font-weight:400}
.tree-actions{display:none;position:absolute;right:2px;background:inherit;border-radius:8px}
.tree-row:hover .tree-actions{display:flex}
.tree-row:hover .tree-count{visibility:hidden}
.tree-actions .lib-icon-btn{width:24px;height:24px}
.tree-new{display:flex;align-items:center;gap:7px;height:32px;margin-top:4px;padding:0 8px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:13px;cursor:pointer}
.tree-new:hover:not(:disabled){background:var(--ui-control)}
.tree-foot{margin:auto 0 0;padding:12px 8px 0;font-size:11.5px;line-height:1.6;color:var(--ui-text-tertiary)}
.main{flex:1;min-width:0;display:flex;flex-direction:column;padding-left:18px}
.crumbs{display:flex;align-items:center;gap:6px;min-height:32px;font-size:13px}
.crumbs .lib-link{font-size:13px}
.crumbs .lib-link.here{color:var(--ui-text-primary);font-weight:600}
.crumbs .sep{color:var(--ui-text-disabled,#C7C7CC)}
.seg{margin-left:auto;display:inline-flex;padding:2px;border-radius:8px;background:var(--ui-control)}
.seg button{height:26px;padding:0 12px;border:0;border-radius:6px;background:transparent;font:inherit;font-size:12px;color:var(--ui-text-secondary);cursor:pointer}
.seg button.on{background:var(--ui-surface);color:var(--ui-text-primary);font-weight:500;box-shadow:0 1px 2px rgba(0,0,0,.08)}
.list{padding:6px 0 16px}
.row{width:100%;display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;align-items:center;padding:8px 10px;border:0;border-radius:10px;background:transparent;font:inherit;text-align:left;cursor:pointer;color:inherit}
.row:hover{background:var(--ui-surface-hover,var(--ui-control))}
.row.selected{background:var(--ui-soft)}
.row-main{display:flex;align-items:center;gap:10px;min-width:0}
.row-text{display:flex;flex-direction:column;gap:2px;min-width:0}
.row-text b{font-size:13.5px;font-weight:500;color:var(--ui-text-primary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.row-text .lib-muted{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.file-icon{width:32px;height:38px;flex:none;border-radius:6px;display:grid;place-items:end center;padding-bottom:5px;box-sizing:border-box;font-size:9px;font-weight:700;letter-spacing:.02em;box-shadow:inset 0 0 0 1px rgba(0,0,0,.05)}
.folder-icon{width:32px;height:38px;flex:none;display:grid;place-items:center;color:var(--ui-btn)}
.folder-icon :deep(svg){fill:var(--ui-soft)}
.nothing{padding:48px 0;text-align:center;font-size:13px}
.hint{padding:14px 10px 0}
.first{padding-top:64px}
.first-actions{display:flex;gap:10px;margin-top:6px}
.detail{width:276px;flex:none;margin-left:16px;padding:4px 2px 18px 18px;border-left:1px solid var(--ui-line);display:flex;flex-direction:column;gap:12px;overflow-y:auto}
.thumb{position:relative;height:130px;border-radius:12px;background:var(--ui-surface-sunken,var(--ui-control));box-shadow:inset 0 0 0 1px var(--ui-line);display:grid;place-items:center}
.sheet{width:76px;height:94px;border-radius:8px;background:var(--ui-surface);box-shadow:0 6px 18px rgba(0,0,0,.08),0 0 0 1px var(--ui-line);display:flex;flex-direction:column;gap:6px;padding:14px 12px;box-sizing:border-box}
.sheet i{height:4px;border-radius:2px;background:var(--ui-control)}
.sheet i:first-child{height:6px;width:70%}
.sheet i.short{width:60%}
.thumb-kind{position:absolute;right:12px;bottom:10px;font-size:11px;font-weight:700}
.title-row{display:flex;align-items:flex-start;gap:4px}
.doc-title{flex:1;min-width:0;font-size:15px;font-weight:600;line-height:1.4;word-break:break-all}
.excerpt{display:flex;flex-direction:column;gap:6px;padding:12px;border-radius:10px;background:var(--ui-surface-sunken,var(--ui-control));box-shadow:inset 0 0 0 1px var(--ui-line)}
.excerpt-head{font-size:12px;font-weight:600;color:var(--ui-ink)}
.excerpt.quiet .excerpt-head{color:var(--ui-text-secondary)}
.excerpt-text{font-size:12.5px;line-height:1.65;color:var(--ui-text-secondary);display:-webkit-box;-webkit-line-clamp:6;-webkit-box-orient:vertical;overflow:hidden}
.facts{margin:0;display:grid;grid-template-columns:52px 1fr;gap:8px;align-items:center;font-size:12px}
.facts dt{color:var(--ui-text-tertiary)}
.facts dd{margin:0;min-width:0;color:var(--ui-text-secondary)}
.move{width:100%;height:28px;border:0;border-radius:8px;padding:0 8px;background:var(--ui-control);color:var(--ui-text-primary);font:inherit;font-size:12px}
.tags{display:flex;flex-wrap:wrap;gap:6px}
.untag{display:inline-grid;place-items:center;margin-left:3px;padding:0;border:0;background:transparent;color:inherit;cursor:pointer}
.lib-tag.add{cursor:pointer;color:var(--ui-text-tertiary);gap:2px}
.tag-input{width:84px;height:22px;border:0;border-radius:6px;padding:0 6px;background:var(--ui-control);font:inherit;font-size:11.5px;color:var(--ui-text-primary);outline:none}
.origin{word-break:break-all}
.detail-actions{display:flex;flex-direction:column;gap:8px;margin-top:auto}
.pair{display:flex;gap:8px}
.pair .xb-btn:first-child{flex:1}
</style>
