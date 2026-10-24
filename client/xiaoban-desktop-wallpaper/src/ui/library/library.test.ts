// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { reactive } from 'vue';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { folderPath, folderTree, formatWhen, guessPlatform, inView, pathOf, type LibraryData, type MemoryOverview } from './library-store.ts';
import LibraryPage from '../pages/LibraryPage.vue';

const invoke = vi.fn();
vi.mock('../../bridge.ts', async importOriginal => ({ ...(await importOriginal<typeof import('../../bridge.ts')>()), invoke: (...args: unknown[]) => invoke(...args), listen: vi.fn(async () => () => {}) }));

const NOW = new Date(2026, 9, 23, 15, 0).getTime();
const data = (): LibraryData => ({
  folders: [
    { id: 'work', name: '工作', parent: null, createdMs: 1 }, { id: 'fin', name: '财务', parent: 'work', createdMs: 1 },
    { id: 'me', name: '个人', parent: null, createdMs: 1 },
  ],
  docs: [
    { id: 'a', name: '预算.xlsx', folder: 'fin', ext: 'xlsx', size: 286_000, importedMs: NOW, origin: '/Users/me/预算.xlsx', status: 'ready', note: '', preview: '全年预算 1,280 万', tags: ['财务'] },
    { id: 'b', name: '需求.pdf', folder: null, ext: 'pdf', size: 2_400_000, importedMs: NOW - 1000, origin: '', status: 'reading', note: '', preview: '', tags: [] },
    { id: 'c', name: '品牌规范.md', folder: 'work', ext: 'md', size: 12_000, importedMs: NOW - 2000, origin: '', status: 'ready', note: '', preview: '主色小伴红', tags: [] },
    { id: 'd', name: '发票.pdf', folder: 'me', ext: 'pdf', size: 1_100_000, importedMs: NOW - 3000, origin: '', status: 'failed', note: '里面没有可读的文字，可能是扫描件', preview: '', tags: [] },
  ],
  links: [{ id: 'l1', url: 'https://pan.baidu.com/s/1?pwd=x7k2', title: '设计素材', platform: 'baidu', kind: '网盘分享', code: 'x7k2', folder: 'work', addedMs: NOW }],
  saved: [{ id: 's1', kind: 'answer', title: '下周计划', text: '1. 周一评审', sessionId: 'chat-1', sessionTitle: '整理计划', source: 'assistant', savedMs: NOW }],
});
const memory = (): MemoryOverview => ({
  remembering: true,
  notes: [{ name: 'about-me', preview: '称呼我小王', text: '称呼我小王', bytes: 16 }],
  candidates: [{ id: 7, kind: 'procedure', content: '周五下午写周报', evidence: '出现两次', createdMs: NOW }],
  approved: [{ id: 3, kind: 'failure', content: '先看表格再看结论', evidence: '用户纠正', createdMs: NOW }],
  goals: [{ sessionId: 'chat-1', outcome: '11 月 7 日前上线官网', state: 'running', updatedAtMs: NOW }],
});

describe('library helpers', () => {
  it('nests folders with counts that include everything inside them', () => {
    const d = data();
    expect(folderTree(d).map(r => `${r.depth}:${r.name}:${r.count}`)).toEqual(['0:全部文档:4', '0:未分组:1', '0:个人:1', '0:工作:2', '1:财务:1']);
    expect(inView(d.folders, d.docs[0], 'work')).toBe(true);
    expect(inView(d.folders, d.docs[0], 'me')).toBe(false);
    expect(pathOf(d.folders, 'fin').map(f => f.name)).toEqual(['工作', '财务']);
    expect(folderPath(d.folders, 'fin')).toBe('工作 / 财务');
    expect(folderPath(d.folders, null)).toBe('未分组');
  });
  it('recognises cloud links as the host does, and not lookalikes', () => {
    expect(guessPlatform('https://x.feishu.cn/docx/1')).toBe('feishu');
    expect(guessPlatform('https://docs.qq.com/sheet/1')).toBe('tencent');
    expect(guessPlatform('https://feishu.cn.evil.example/docx')).toBe('other');
    expect(guessPlatform('飞书文档')).toBeNull();
  });
  it('says when things happened the way people say it', () => {
    expect(formatWhen(NOW, NOW)).toBe('今天 15:00');
    expect(formatWhen(NOW - 86_400_000, NOW)).toBe('昨天');
    expect(formatWhen(new Date(2026, 0, 5).getTime(), NOW)).toBe('1月5日');
  });
});

function setup(overrides: Record<string, unknown> = {}) {
  let library = data(), mem = memory();
  invoke.mockImplementation(async (command: string, args: Record<string, unknown> = {}) => {
    if (command === 'library_state') return library;
    if (command === 'chat_memory_overview') return mem;
    if (command === 'library_move') { library = { ...library, docs: library.docs.map(d => (args.ids as string[]).includes(d.id) ? { ...d, folder: args.folder as string | null } : d) }; return library; }
    if (command === 'library_add_link') { library = { ...library, links: [{ id: 'l2', url: args.url as string, title: args.title as string || '文档', platform: 'feishu', kind: '文档', code: null, folder: args.folder as string | null, addedMs: NOW }, ...library.links] }; return library; }
    if (command === 'library_attachment') return { name: '预算.xlsx', size: 286_000, mime: 'application/vnd', dataUrl: 'data:application/vnd;base64,AA==' };
    if (command === 'chat_review_experience') { mem = { ...mem, candidates: [], approved: [...mem.approved, { ...mem.candidates[0] }] }; return {}; }
    if (command === 'chat_memory_set_remembering') { mem = { ...mem, remembering: Boolean(args.enabled) }; return; }
    return undefined;
  });
  const chat = reactive({ sessions: [{ id: 'chat-1', title: '整理计划' }], views: {} as Record<string, { items: unknown[] }>, select: vi.fn(async () => {}) });
  const store = reactive({ desktop: true, notify: vi.fn(), navigate: vi.fn(), chat, creationDraft: null as string | null, creationAttachments: null as unknown[] | null, ...overrides });
  const wrapper = mount(LibraryPage, { attachTo: document.body, props: { store: store as never } });
  return { wrapper, store, chat };
}
const tab = async (wrapper: ReturnType<typeof setup>['wrapper'], id: string) => { await wrapper.get(`[data-testid="library-tab-${id}"]`).trigger('click'); await flushPromises(); };

describe('资料库 page', () => {
  beforeEach(() => { invoke.mockReset(); try { localStorage.clear(); } catch { /* not needed */ } });

  it('shows the four tabs with counts that follow the search', async () => {
    const { wrapper } = setup();
    await flushPromises();
    expect(wrapper.findAll('[role="tab"]').map(t => t.text())).toEqual(['我的文档4', '云文档1', '收藏1', '记忆4']);
    await wrapper.get('[data-testid="library-search"]').setValue('预算');
    expect(wrapper.get('[data-testid="library-tab-docs"]').text()).toBe('我的文档1');
  });

  it('refreshes 记忆 when the assistant saves a note, but not when the save failed', async () => {
    const { chat } = setup();
    await flushPromises();
    const reads = () => invoke.mock.calls.filter(([command]) => command === 'chat_memory_overview').length;
    const before = reads();
    chat.views['chat-1'] = { items: [{ kind: 'tool', id: 't1', name: 'memory_write', status: 'error' }] };
    await flushPromises();
    expect(reads()).toBe(before);
    chat.views['chat-1'].items.push({ kind: 'tool', id: 't2', name: 'memory_write', status: 'done' });
    await flushPromises();
    expect(reads()).toBe(before + 1);
  });

  it('opens folders, shows a folder’s subfolders first, and moves a document', async () => {
    const { wrapper } = setup();
    await flushPromises();
    await wrapper.get('[data-testid="library-folder-work"]').trigger('click');
    const list = wrapper.get('[data-testid="library-doc-list"]');
    expect(list.find('[data-testid="library-subfolder-fin"]').exists()).toBe(true);
    // Only what sits directly in 工作; 财务's file is inside its folder.
    expect(list.findAll('[data-testid^="library-doc-"]').map(r => r.attributes('data-testid'))).toEqual(['library-doc-c']);
    await wrapper.get('[data-testid="library-subfolder-fin"]').trigger('click');
    expect(wrapper.find('[data-testid="library-doc-a"]').exists()).toBe(true);
    expect(wrapper.get('[data-testid="library-doc-detail"]').text()).toContain('全年预算 1,280 万');
    await wrapper.get('[data-testid="library-move"]').setValue('me');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('library_move', { ids: ['a'], folder: 'me' });
    // Moved out of 财务, which is now empty.
    expect(wrapper.find('[data-testid="library-doc-a"]').exists()).toBe(false);
  });

  it('says why a scanned PDF cannot be used and does not offer to ask with it', async () => {
    const { wrapper } = setup();
    await flushPromises();
    await wrapper.get('[data-testid="library-doc-d"]').trigger('click');
    const detail = wrapper.get('[data-testid="library-doc-detail"]');
    expect(detail.text()).toContain('可能是扫描件');
    expect(detail.get('[data-testid="library-ask"]').attributes('disabled')).toBeDefined();
  });

  it('hands a document to a new task to ask with it', async () => {
    const { wrapper, store, chat } = setup();
    await flushPromises();
    await wrapper.get('[data-testid="library-doc-a"]').trigger('click');
    await wrapper.get('[data-testid="library-ask"]').trigger('click');
    await flushPromises();
    expect(chat.select).toHaveBeenCalledWith(null);
    expect(store.creationAttachments).toEqual([{ name: '预算.xlsx', size: 286_000, mime: 'application/vnd', dataUrl: 'data:application/vnd;base64,AA==' }]);
    expect(store.creationDraft).toContain('预算.xlsx');
    expect(store.navigate).toHaveBeenCalledWith('new-task');
  });

  it('adds a cloud link through the dialog after recognising it', async () => {
    const { wrapper } = setup();
    await flushPromises();
    await tab(wrapper, 'cloud');
    expect(wrapper.get('[data-testid="library-link-l1"]').text()).toContain('提取码 x7k2');
    await wrapper.get('[data-testid="library-add-link"]').trigger('click');
    await flushPromises();
    const dialog = document.body.querySelector('[data-testid="library-link-dialog"]')!;
    const url = dialog.querySelector<HTMLInputElement>('[data-testid="library-link-url"] input')!;
    url.value = 'not a link'; url.dispatchEvent(new Event('input')); await flushPromises();
    expect(dialog.querySelector('[data-testid="library-link-guess"]')!.textContent).toContain('不像一个网址');
    expect(dialog.querySelector<HTMLButtonElement>('[data-testid="library-link-add"]')!.disabled).toBe(true);
    url.value = 'https://x.feishu.cn/docx/okr'; url.dispatchEvent(new Event('input')); await flushPromises();
    expect(dialog.querySelector('[data-testid="library-link-guess"]')!.textContent).toContain('识别为 飞书');
    dialog.querySelector<HTMLButtonElement>('[data-testid="library-link-add"]')!.click();
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('library_add_link', { url: 'https://x.feishu.cn/docx/okr', title: '', folder: null });
    expect(wrapper.find('[data-testid="library-link-l2"]').exists()).toBe(true);
  });

  it('goes back to the conversation a saved answer came from', async () => {
    const { wrapper, store, chat } = setup();
    await flushPromises();
    await tab(wrapper, 'saved');
    const card = wrapper.get('[data-testid="library-saved-s1"]');
    expect(card.text()).toContain('来自「整理计划」· 超级小伴');
    await card.findAll('button').find(b => b.text().includes('回到对话'))!.trigger('click');
    await flushPromises();
    expect(chat.select).toHaveBeenCalledWith('chat-1');
    expect(store.navigate).toHaveBeenCalledWith('chat');
  });

  it('asks before remembering what 小伴 noticed, and can switch remembering off', async () => {
    const { wrapper } = setup();
    await flushPromises();
    await tab(wrapper, 'memory');
    expect(wrapper.get('[data-testid="memory-pending"]').text()).toContain('周五下午写周报');
    expect(wrapper.text()).toContain('称呼我小王');
    expect(wrapper.text()).toContain('11 月 7 日前上线官网');
    await wrapper.get('[data-testid="memory-keep-7"]').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_review_experience', { experienceId: 7, approve: true });
    expect(wrapper.find('[data-testid="memory-pending"]').exists()).toBe(false);
    await wrapper.get('[data-testid="memory-switch"]').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_memory_set_remembering', { enabled: false });
    expect(wrapper.text()).toContain('已关闭');
  });
});
