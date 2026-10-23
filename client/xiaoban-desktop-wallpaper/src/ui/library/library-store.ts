import { reactive } from 'vue';

/** 资料库 as the host keeps it (see `data_library.rs`). */
export interface LibraryFolder { id: string; name: string; parent: string | null; createdMs: number }
export type DocStatus = 'reading' | 'ready' | 'failed' | 'unsupported' | 'tooLarge';
export interface LibraryDoc {
  id: string; name: string; folder: string | null; ext: string; size: number; importedMs: number;
  origin: string; status: DocStatus; note: string; preview: string; tags: string[];
}
export type Platform = 'feishu' | 'tencent' | 'baidu' | 'shimo' | 'yuque' | 'wps' | 'other';
export interface LibraryLink { id: string; url: string; title: string; platform: Platform; kind: string; code: string | null; folder: string | null; addedMs: number }
export type SavedKind = 'answer' | 'file' | 'image' | 'shot';
export interface LibrarySaved { id: string; kind: SavedKind; title: string; text: string; sessionId: string | null; sessionTitle: string; source: 'main' | 'assistant'; savedMs: number }
export interface LibraryData { folders: LibraryFolder[]; docs: LibraryDoc[]; links: LibraryLink[]; saved: LibrarySaved[] }

export interface Experience { id: number; kind: string; content: string; evidence: string; createdMs: number }
export interface MemoryNote { name: string; preview: string; text: string; bytes: number }
export interface MemoryGoal { sessionId: string; outcome: string; state: string; updatedAtMs: number }
export interface MemoryOverview { remembering: boolean; notes: MemoryNote[]; candidates: Experience[]; approved: Experience[]; goals: MemoryGoal[] }

type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
type Listen = <T>(event: string, handler: (payload: T) => void) => Promise<() => void>;
export interface LibraryDeps { desktop: boolean; invoke: Invoke; listen: Listen; notify: (message: unknown, type?: 'success' | 'error' | 'info') => void }

export const emptyLibrary = (): LibraryData => ({ folders: [], docs: [], links: [], saved: [] });
/** Whatever the host sent, with every list present. */
const asLibrary = (value: Partial<LibraryData> | null | undefined): LibraryData => ({ ...emptyLibrary(), ...(value ?? {}) });
const asMemory = (value: Partial<MemoryOverview> | null | undefined): MemoryOverview | null => value
  ? { remembering: value.remembering ?? true, notes: value.notes ?? [], candidates: value.candidates ?? [], approved: value.approved ?? [], goals: value.goals ?? [] } : null;

/* ── What the page shows, worked out from the data ─────────────────────── */

/** A folder view: every document, those in no folder, or one folder. */
export type FolderView = 'all' | 'none' | string;

/** `folder` and every folder inside it. */
export function within(folders: LibraryFolder[], id: string): string[] {
  const out = [id];
  for (let i = 0; i < out.length; i++) out.push(...folders.filter(f => f.parent === out[i]).map(f => f.id));
  return out;
}

export function pathOf(folders: LibraryFolder[], id: string | null): LibraryFolder[] {
  const out: LibraryFolder[] = [];
  let current = id ? folders.find(f => f.id === id) : undefined;
  while (current && out.length < 64) { out.unshift(current); current = current.parent ? folders.find(f => f.id === current!.parent) : undefined; }
  return out;
}

export const folderPath = (folders: LibraryFolder[], id: string | null) => id ? pathOf(folders, id).map(f => f.name).join(' / ') || '未分组' : '未分组';

export function inView(folders: LibraryFolder[], doc: { folder: string | null }, view: FolderView) {
  if (view === 'all') return true;
  if (view === 'none') return doc.folder === null;
  return doc.folder !== null && within(folders, view).includes(doc.folder);
}

export interface TreeRow { id: string; name: string; depth: number; count: number; folder: boolean }
/** The folder list down the side: the two views, then the folders, nested, each with how many documents it holds. */
export function folderTree(data: LibraryData): TreeRow[] {
  const rows: TreeRow[] = [
    { id: 'all', name: '全部文档', depth: 0, count: data.docs.length, folder: false },
    { id: 'none', name: '未分组', depth: 0, count: data.docs.filter(d => d.folder === null).length, folder: false },
  ];
  const sorted = [...data.folders].sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'));
  const walk = (parent: string | null, depth: number) => {
    for (const f of sorted.filter(x => x.parent === parent)) {
      rows.push({ id: f.id, name: f.name, depth, count: data.docs.filter(d => inView(data.folders, d, f.id)).length, folder: true });
      walk(f.id, depth + 1);
    }
  };
  walk(null, 0);
  return rows;
}

const MATCH: Record<string, string[]> = { PDF: ['pdf'], 文档: ['docx', 'doc'], 表格: ['xlsx', 'xls', 'csv', 'tsv'], 演示: ['pptx', 'ppt'], 图片: ['png', 'jpg', 'jpeg', 'gif', 'webp'], 文本: ['md', 'txt', 'json', 'log'] };
export const DOC_FILTERS = ['全部', ...Object.keys(MATCH)];
export const matchesFilter = (doc: LibraryDoc, filter: string) => filter === '全部' || (MATCH[filter] ?? []).includes(doc.ext);

export const searchText = (query: string, ...fields: (string | null | undefined)[]) => {
  const q = query.trim().toLowerCase();
  return !q || fields.some(f => (f ?? '').toLowerCase().includes(q));
};

export const DOC_STATUS: Record<DocStatus, { label: string; tone: 'ok' | 'busy' | 'muted' }> = {
  ready: { label: '小伴可参考', tone: 'ok' },
  reading: { label: '正在读取', tone: 'busy' },
  failed: { label: '读不出文字', tone: 'muted' },
  unsupported: { label: '暂不支持', tone: 'muted' },
  tooLarge: { label: '文件太大', tone: 'muted' },
};

export const PLATFORMS: Record<Platform, { name: string; letter: string; hue: string }> = {
  feishu: { name: '飞书', letter: '飞', hue: '#2F6FDB' },
  tencent: { name: '腾讯文档', letter: '腾', hue: '#1A7FB8' },
  baidu: { name: '百度网盘', letter: '百', hue: '#3D5AD6' },
  shimo: { name: '石墨文档', letter: '石', hue: '#4A4A50' },
  yuque: { name: '语雀', letter: '语', hue: '#23885A' },
  wps: { name: 'WPS 云文档', letter: 'W', hue: '#C4323A' },
  other: { name: '网页', letter: '网', hue: '#6E6E73' },
};
/** Recognises a pasted link the same way the host does, to show it before it is added. */
export function guessPlatform(url: string): Platform | null {
  const match = /^https?:\/\/([^/?#]+)/i.exec(url.trim());
  if (!match) return null;
  const host = match[1].split('@').pop()!.split(':')[0].toLowerCase();
  const ends = (d: string) => host === d || host.endsWith('.' + d);
  if (ends('feishu.cn') || ends('larksuite.com') || ends('larkoffice.com')) return 'feishu';
  if (ends('docs.qq.com')) return 'tencent';
  if (ends('pan.baidu.com')) return 'baidu';
  if (ends('shimo.im')) return 'shimo';
  if (ends('yuque.com')) return 'yuque';
  if (ends('kdocs.cn') || ends('wps.cn')) return 'wps';
  return host.includes('.') ? 'other' : null;
}

export const SAVED_KINDS: Record<SavedKind, string> = { answer: '回答', file: '文件', image: '图片', shot: '截图' };

export function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
export function formatWhen(ms: number, now = Date.now()) {
  const date = new Date(ms), today = new Date(now);
  const sameDay = (a: Date, b: Date) => a.toDateString() === b.toDateString();
  const time = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
  if (sameDay(date, today)) return `今天 ${time}`;
  if (sameDay(date, new Date(now - 86_400_000))) return '昨天';
  return date.getFullYear() === today.getFullYear() ? `${date.getMonth() + 1}月${date.getDate()}日` : `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
}

/* ── The store ──────────────────────────────────────────────────────────── */

export function createLibraryStore(deps: LibraryDeps) {
  let off: (() => void) | undefined;
  const store = reactive({
    data: emptyLibrary(),
    memory: null as MemoryOverview | null,
    loaded: false,
    busy: false,
    async init() {
      if (!deps.desktop) { store.loaded = true; return; }
      off ??= await deps.listen('library-changed', () => { void store.refresh(); });
      await Promise.all([store.refresh(), store.refreshMemory()]);
      store.loaded = true;
    },
    dispose() { off?.(); off = undefined; },
    async refresh() { await run(async () => { store.data = asLibrary(await deps.invoke<LibraryData>('library_state')); }); },
    async refreshMemory() { await run(async () => { store.memory = asMemory(await deps.invoke<MemoryOverview>('chat_memory_overview')); }); },
    async pickImport(folder: string | null, directory: boolean) {
      await run(async () => {
        const result = await deps.invoke<{ library: LibraryData; imported: number; errors: string[] }>('library_pick_import', { folder, directory });
        store.data = asLibrary(result?.library);
        if (!result) return;
        if (result.errors.length) deps.notify(`${result.errors.length} 个没有导入：${result.errors.slice(0, 3).join('；')}`, 'error');
        else if (result.imported) deps.notify(`已导入 ${result.imported} 个文件`);
      });
    },
    async importFile(name: string, dataUrl: string, folder: string | null) {
      await run(async () => { store.data = asLibrary(await deps.invoke<LibraryData>('library_import_data', { name, dataUrl, folder })); });
    },
    async createFolder(name: string, parent: string | null) {
      let id: string | null = null;
      await run(async () => { const result = await deps.invoke<{ id: string; library: LibraryData }>('library_create_folder', { name, parent }); store.data = asLibrary(result?.library); id = result?.id ?? null; });
      return id;
    },
    renameFolder: (id: string, name: string) => update('library_rename_folder', { id, name }),
    deleteFolder: (id: string) => update('library_delete_folder', { id }),
    move: (ids: string[], folder: string | null) => update('library_move', { ids, folder }),
    renameDoc: (id: string, name: string) => update('library_update_doc', { id, patch: { name } }),
    setTags: (id: string, tags: string[]) => update('library_update_doc', { id, patch: { tags } }),
    remove: (ids: string[]) => update('library_remove', { ids }),
    reveal: (id: string) => run(() => deps.invoke('library_reveal', { id })),
    addLink: (url: string, title: string, folder: string | null) => update('library_add_link', { url, title, folder }),
    renameLink: (id: string, title: string) => update('library_rename_link', { id, title }),
    /** The document as a chat attachment: a data URL for an image, a file record otherwise. */
    async attachment(id: string) {
      try {
        const value = await deps.invoke<{ image?: string; name?: string; size?: number; mime?: string; dataUrl?: string }>('library_attachment', { id });
        return value.image ?? { name: value.name!, size: value.size!, mime: value.mime!, dataUrl: value.dataUrl! };
      } catch (error) { deps.notify(error, 'error'); return null; }
    },
    setRemembering: (enabled: boolean) => memory('chat_memory_set_remembering', { enabled }),
    review: (experienceId: number, approve: boolean) => memory('chat_review_experience', { experienceId, approve }),
    editExperience: (id: number, content: string) => memory('chat_memory_edit_experience', { id, content }),
    forgetExperience: (id: number) => memory('chat_memory_forget_experience', { id }),
    writeNote: (name: string | null, text: string) => memory('chat_memory_write_note', { name, text }),
    deleteNote: (name: string) => memory('chat_memory_delete_note', { name }),
    clearMemory: () => memory('chat_memory_clear'),
  });
  async function run(action: () => Promise<unknown>) {
    store.busy = true;
    try { await action(); return true; }
    catch (error) { deps.notify(error, 'error'); return false; }
    finally { store.busy = false; }
  }
  const update = (command: string, args: Record<string, unknown>) => run(async () => { store.data = asLibrary(await deps.invoke<LibraryData>(command, args)); });
  const memory = async (command: string, args: Record<string, unknown> = {}) => {
    const ok = await run(() => deps.invoke(command, args));
    await store.refreshMemory();
    return ok;
  };
  return store;
}
export type LibraryStore = ReturnType<typeof createLibraryStore>;
