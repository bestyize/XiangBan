// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { NSlider } from 'naive-ui';
import { reactive } from 'vue';
import ChatPage from './ChatPage.vue';
import ChatNameDialog from '../components/ChatNameDialog.vue';
import ChatMoveDialog from '../components/ChatMoveDialog.vue';
import { createChatStore, CONTINUE_TEXT, type ChatDeps } from '../chat/chat-store.ts';
import type { ControlStore } from '../control-types.ts';
import type { ApprovalRequest, ChatGroup, KernelEvent, Overview, SessionSummary, WireMessage } from '../chat/types.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
import { invoke as bridgeInvoke } from '../../bridge.ts';

const summary = (id: string, title: string, updatedAt = Math.floor(Date.now() / 1000)): SessionSummary => ({ id, title, createdAt: updatedAt, updatedAt });
const overview = (patch: Partial<Overview> = {}): Overview => ({ configured: true, sessions: [], running: [], workspace: '/Users/me/小伴/工作区', permissionMode: 'ask', hideToolCalls: false, ...patch });

type Responder = (args: Record<string, unknown> | undefined) => unknown;
let wrapper: VueWrapper | undefined;

async function open(responders: Record<string, Responder> = {}, options: { desktop?: boolean } = {}) {
  const handlers = new Map<string, (payload: never) => void>();
  const calls: [string, Record<string, unknown> | undefined][] = [];
  const table: Record<string, Responder> = { chat_overview: () => overview(), chat_context: () => ({ used: 0, limit: 256_000, canCompact: false }), ...responders };
  // The ring's numbers are read on their own schedule, so they are kept apart from the calls a test is about.
  const contextCalls: (Record<string, unknown> | undefined)[] = [];
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === 'chat_context') contextCalls.push(args); else if (command !== 'get_ai_config') calls.push([command, args]);
    if (command === 'get_ai_config' && !table[command]) return { model: 'coding-model', activeProvider: 'demo', providers: [{ id: 'demo', name: '演示模型', models: [{ id: 'coding-model', name: 'Coding Model' }] }] } as never;
    const respond = table[command];
    if (!respond) throw new Error(`unexpected ${command}`);
    return respond(args) as never;
  });
  const notify = vi.fn();
  const desktop = options.desktop ?? true;
  const deps: ChatDeps = { desktop, invoke, notify, listen: async (event, callback) => { handlers.set(event, callback as never); return () => {}; } };
  vi.mocked(bridgeInvoke).mockImplementation(invoke as never);
  const chat = createChatStore(deps);
  const store = reactive({ desktop, chat, settingsTab: 'general', page: 'chat', navigate: vi.fn(), notify }) as unknown as ControlStore;
  wrapper = mount(ChatPage, { props: { store }, attachTo: document.body });
  await flushPromises();
  const emit = (event: string, payload: unknown) => handlers.get(event)!(payload as never);
  const events = async (session: string, ...list: KernelEvent[]) => { list.forEach(event => emit('chat-event', { session, event })); await flushPromises(); };
  return { store, chat, calls, contextCalls, invoke, notify, emit, events, commands: () => calls.map(([c]) => c) };
}

const q = <T extends Element = HTMLElement>(id: string) => wrapper!.element.querySelector(`[data-testid="${id}"]`) as T | null;
const all = (selector: string) => [...wrapper!.element.querySelectorAll(selector)] as HTMLElement[];
const textarea = () => wrapper!.element.querySelector('[data-testid="chat-input"] textarea') as HTMLTextAreaElement;
async function click(id: string) { q<HTMLButtonElement>(id)!.click(); await flushPromises(); }
// Menus and dialogs are teleported out of the page, so they are looked up on the whole document.
const inBody = <T extends HTMLElement = HTMLElement>(id: string) => document.body.querySelector(`[data-testid="${id}"]`) as T | null;
async function clickBody(id: string) { const el = inBody(id); if (!el) throw new Error(`missing ${id}`); el.click(); await flushPromises(); }
async function openMenu(id: string) { await click(`chat-menu-${id}`); await flushPromises(); }
async function openGroupMenu(id: string) { await click(`chat-group-menu-${id}`); await flushPromises(); }
async function typeName(value: string) { const input = inBody('chat-name-input')!.querySelector('input')!; input.value = value; input.dispatchEvent(new Event('input')); await flushPromises(); }
async function submitName() { inBody('chat-name-ok')!.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true })); await flushPromises(); }
const nameInput = () => inBody('chat-name-input')?.querySelector('input')?.value;
// A closed modal stays in the document until its leave transition ends, which never happens here, so ask the component.
const nameDialogOpen = () => wrapper!.findComponent(ChatNameDialog).props('show');
const moveDialogOpen = () => wrapper!.findComponent(ChatMoveDialog).props('show');
async function type(value: string) { const t = textarea(); t.value = value; t.dispatchEvent(new Event('input', { bubbles: true })); await flushPromises(); }
async function press(init: KeyboardEventInit & { keyCode?: number }) {
  const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
  if (init.keyCode !== undefined) Object.defineProperty(event, 'keyCode', { value: init.keyCode });
  textarea().dispatchEvent(event);
  await flushPromises();
  return event;
}
const startChat: Record<string, Responder> = { chat_new_session: () => summary('s1', '新对话'), chat_send: () => summary('s1', '你好') };
const approval = (patch: Partial<ApprovalRequest> = {}): ApprovalRequest => ({ id: 'a1', session: 's1', tool: 'shell', risk: 'exec', arguments: { command: 'ls -la' }, tainted: false, canRemember: false, ...patch });

beforeEach(() => { vi.resetAllMocks(); localStorage.clear(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; });

describe('chat workbench composer actions', () => {
  it('keeps capability menus in the composer and uses the desktop AI Store inventories', async () => {
    const h = await open({
      ai_store_experts_installed: () => [{ slug: 'reviewer', name: '代码专家', description: '代码审阅', entrySkill: 'code-review', ready: true }],
      ai_store_installed: () => [{ slug: 'check', name: '质量检查', description: '分析测试' }],
      ai_store_connector_status: () => [{ id: 'github', name: 'GitHub', connected: true, enabled: true, toolCount: 8 }],
    });
    await click('chat-add-menu');
    expect(q('chat-add-screenshot')).not.toBeNull();
    expect(q('chat-pick-image')).not.toBeNull();
    expect(q('chat-add-experts')).not.toBeNull();
    await click('chat-add-experts');
    expect(q('chat-installed-reviewer')?.textContent).toContain('代码专家');
    await click('chat-installed-reviewer');
    expect(textarea().value).toContain('load_skill');
    expect(textarea().value).toContain('code-review');
    await click('chat-add-menu');
    await click('chat-add-skills');
    await click('chat-installed-check');
    expect(textarea().value).toContain('质量检查');
    await click('chat-add-menu');
    await click('chat-add-connectors');
    await click('chat-installed-github');
    expect(textarea().value).toContain('GitHub');
    expect(h.commands()).toContain('ai_store_connector_status');
  });

  it('uses a dedicated control-window screenshot command and adds the result without sending', async () => {
    const shot = 'data:image/png;base64,iVBORw0KGgo=';
    const h = await open({ chat_screenshot: () => shot });
    await click('chat-add-menu');
    await click('chat-add-screenshot');
    expect(h.commands()).toContain('chat_screenshot');
    expect(q('chat-attachment-0')).not.toBeNull();
    expect(h.commands()).not.toContain('chat_send');
  });

  it('lists configured providers and their models in a compact composer menu', async () => {
    const config = { model: 'coding-model', activeProvider: 'demo', providers: [{
      id: 'demo', name: '演示供应商', protocol: 'chat', baseUrl: 'https://example.com', model: 'coding-model',
      hasKey: true, isCustom: true, models: [{ id: 'coding-model', name: 'Coding Model' }, { id: 'reasoning', name: '推理模型' }],
    }] };
    const h = await open({ get_ai_config: () => config, refresh_ai_providers: () => config,
      save_ai_provider: () => undefined });
    expect(q('chat-workbench-model')!.textContent).toContain('Coding Model');
    await click('chat-workbench-model');
    expect(wrapper!.element.textContent).toContain('演示供应商');
    await click('chat-model-demo-reasoning');
    expect(h.commands()).toContain('save_ai_provider');
  });
});

describe('the chat page before a conversation', () => {
  it('sends people to the model settings when no model is set up', async () => {
    const h = await open({ chat_overview: () => overview({ configured: false }) });
    expect(q('chat-unconfigured')).not.toBeNull();
    expect(textarea().disabled).toBe(true);
    expect(textarea().placeholder).toContain('先到设置里配置模型');
    await click('chat-go-settings');
    expect(h.store.settingsTab).toBe('ai');
    expect(h.store.navigate).toHaveBeenCalledWith('settings');
  });

  it('offers things to try and fills the box with the one picked', async () => {
    await open();
    expect(q('chat-welcome')).not.toBeNull();
    expect(all('.examples button')).toHaveLength(4);
    await click('chat-example-2');
    expect(textarea().value).toBe('每天下午三点提醒我喝水');
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
  });

  it('receives a creative prompt in a fresh conversation without sending it', async () => {
    const h = await open();
    h.store.creationDraft = '帮我创作一个桌面特效：';
    await flushPromises();
    expect(textarea().value).toBe('帮我创作一个桌面特效：');
    expect(h.store.creationDraft).toBeNull();
    expect(h.commands()).toEqual(['chat_overview']);
  });

  it('only says it is a preview in the browser', async () => {
    const h = await open({}, { desktop: false });
    expect(q('chat-preview-only')).not.toBeNull();
    expect(h.invoke).not.toHaveBeenCalled();
    expect(textarea().disabled).toBe(true);
  });

  it('has an AI model settings button in the header that opens the AI tab', async () => {
    const h = await open();
    expect(q('chat-ai-config')!.textContent).toContain('AI 模型配置');
    expect(q('chat-workspace')?.textContent).toContain('工作区');
    await click('chat-ai-config');
    expect(h.store.settingsTab).toBe('ai');
  });

  it('opens the capability guide from the header or welcome screen', async () => {
    await open();
    expect(q('chat-open-guide')).not.toBeNull();
    await click('chat-open-guide');
    expect(document.body.textContent).toContain('小伴桌面 · AI 玩法指南');
  });
});

describe('talking', () => {
  it('sends with the button, shows the question at once, streams the answer and shows rich text safely', async () => {
    const h = await open(startChat);
    await type('你好');
    await click('chat-send');
    expect(h.commands()).toEqual(['chat_overview', 'chat_new_session', 'chat_send']);
    expect(textarea().value).toBe('');
    expect(all('[data-testid="msg-user"]').map(m => m.textContent)).toEqual(['你好']);
    expect(q('chat-stop')).not.toBeNull();
    expect(q('chat-status')!.textContent).toContain('思考中');
    expect(q('chat-title')!.textContent).toBe('你好');

    await h.events('s1', { type: 'text_delta', text: '你好！**我在**' }, { type: 'text_delta', text: '<img src=x onerror=alert(1)>' });
    expect(q('chat-status')).toBeNull();
    expect(wrapper!.find('.answer strong').text()).toBe('我在');
    expect(wrapper!.find('.cursor').exists()).toBe(true);
    expect(wrapper!.find('.answer img').exists()).toBe(false);
    expect(wrapper!.find('.answer').text()).toContain('<img src=x onerror=alert(1)>');

    await h.events('s1', { type: 'done' });
    expect(wrapper!.find('.cursor').exists()).toBe(false);
    expect(q('chat-stop')).toBeNull();
    expect(q('chat-send')).not.toBeNull();
  });

  it('sends with Enter, keeps Shift+Enter for new lines and ignores Enter that picks an input-method candidate', async () => {
    const h = await open(startChat);
    await type('第一行');
    expect((await press({ key: 'Enter', shiftKey: true })).defaultPrevented).toBe(false);
    expect(h.commands()).not.toContain('chat_send');

    await press({ key: 'Enter', isComposing: true });
    await press({ key: 'Enter', keyCode: 229 });
    textarea().dispatchEvent(new Event('compositionstart', { bubbles: true }));
    await press({ key: 'Enter' });
    expect(h.commands()).not.toContain('chat_send');
    textarea().dispatchEvent(new Event('compositionend', { bubbles: true }));

    const sent = await press({ key: 'Enter', keyCode: 13 });
    expect(sent.defaultPrevented).toBe(true);
    expect(h.calls.find(([c]) => c === 'chat_send')![1]).toEqual({ sessionId: 's1', text: '第一行' });
  });

  it('does not send an empty message, and gives the text back when the send is refused', async () => {
    const h = await open({ chat_new_session: () => summary('s1', '新对话'), chat_send: () => { throw new Error('网络不通'); }, chat_delete_session: () => [] });
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(true);
    await type('   ');
    await press({ key: 'Enter' });
    expect(h.commands()).toEqual(['chat_overview']);
    await type('帮我查一下');
    await click('chat-send');
    expect(textarea().value).toBe('帮我查一下');
    expect(h.notify).toHaveBeenCalled();
    expect(all('[data-testid="msg-user"]')).toHaveLength(0);
  });

  it('stops an answer with the stop button and shows that it stopped', async () => {
    const h = await open({ ...startChat, chat_cancel: () => undefined });
    await type('写一篇长文');
    await click('chat-send');
    await click('chat-stop');
    expect(h.calls.at(-1)).toEqual(['chat_cancel', { sessionId: 's1' }]);
    await h.events('s1', { type: 'cancelled' });
    expect(q('notice-info')!.textContent).toContain('已停止');
    expect(q('chat-send')).not.toBeNull();
  });

  it('keeps live tool activity compact and opens the command/result history on request', async () => {
    const h = await open(startChat);
    await type('看看文件');
    await click('chat-send');
    await h.events('s1', { type: 'tool_start', id: 't1', name: 'shell', arguments: '{"command":"ls -la"}' });
    expect(q('activity-live')!.textContent).toContain('ls -la');
    expect(q('activity-history')).toBeNull();
    await click('activity-toggle');
    const card = () => q<HTMLDetailsElement>('tool-t1')!;
    expect(card().querySelector('summary')!.textContent).toContain('运行命令');
    expect(card().querySelector('summary')!.textContent).toContain('ls -la');
    expect(card().classList.contains('running')).toBe(true);
    await h.events('s1', { type: 'tool_end', id: 't1', output: 'a.txt\nb.txt', is_error: false });
    expect(card().classList.contains('done')).toBe(true);
    expect(card().querySelectorAll('pre')[1].textContent).toBe('a.txt\nb.txt');
    await h.events('s1', { type: 'tool_start', id: 't2', name: 'shell', arguments: '{"command":"boom"}' }, { type: 'tool_end', id: 't2', output: '命令失败', is_error: true });
    expect(q('tool-t2')!.classList.contains('error')).toBe(true);
  });

  it('places a turn’s tool history below its final reply and folds it when finished', async () => {
    const h = await open(startChat);
    await type('读取浏览器');
    await click('chat-send');
    await h.events('s1', { type: 'text_delta', text: '我先读取页面。' });
    for (let n = 0; n < 12; n++) {
      await h.events('s1', { type: 'tool_start', id: `t${n}`, name: 'get_app_state', arguments: '{}' }, { type: 'tool_end', id: `t${n}`, output: '窗口状态', is_error: false });
    }
    expect(all('[data-testid="chat-activity"]')).toHaveLength(1);
    expect(all('[data-testid^="tool-"]')).toHaveLength(1);
    expect(q('activity-toggle')!.textContent).toContain('12 步');
    await h.events('s1', { type: 'text_delta', text: '页面读取完成。' }, { type: 'done' });
    expect(q('activity-live')).toBeNull();
    expect(q('activity-history')).toBeNull();
    expect(q('activity-toggle')!.getAttribute('aria-expanded')).toBe('false');
    const messages = all('[data-testid="msg-assistant"]');
    expect(messages.at(-1)!.textContent).toContain('页面读取完成');
    expect(messages.at(-1)!.compareDocumentPosition(q('chat-activity')!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    await click('activity-toggle');
    expect(all('[data-testid^="tool-"]')).toHaveLength(12);
  });

  it('offers to continue a cut-off answer', async () => {
    const h = await open({ ...startChat, chat_send: () => summary('s1', 'x') });
    await type('写');
    await click('chat-send');
    await h.events('s1', { type: 'text_delta', text: '写到一半' }, { type: 'truncated' }, { type: 'done' });
    expect(q('notice-warning')!.textContent).toContain('被截断');
    await click('chat-continue');
    expect(h.calls.at(-1)).toEqual(['chat_send', { sessionId: 's1', text: CONTINUE_TEXT }]);
    expect(q('chat-continue'), 'hidden while the new answer runs').toBeNull();
  });

  it('resumes saved work after a no-progress pause without replaying previous actions', async () => {
    const h = await open({ ...startChat, chat_send: () => summary('s1', 'x') });
    await type('继续完成花果山');
    await click('chat-send');
    await h.events('s1',
      { type: 'text_delta', text: '现在增加夜间灯光：' },
      { type: 'continuing', reason: 'announced_action' },
      { type: 'error', message: 'run paused by continuation guard: no_progress' });
    expect(q('notice-warning')!.textContent).toContain('未取得新进展');
    expect(q('notice-error')).toBeNull();
    await click('chat-continue');
    expect(h.calls.at(-1)).toEqual(['chat_send', { sessionId: 's1', text: CONTINUE_TEXT }]);
    expect(q('chat-continue')).toBeNull();
    await h.events('s1', { type: 'text_delta', text: '已经完成。' }, { type: 'done' });
    expect(q('chat-send')).not.toBeNull();
  });

  it('shows an error from the model as an alert', async () => {
    const h = await open(startChat);
    await type('hi');
    await click('chat-send');
    await h.events('s1', { type: 'error', message: '模型不可用：HTTP 401' });
    expect(q('notice-error')!.getAttribute('role')).toBe('alert');
    expect(q('notice-error')!.textContent).toContain('HTTP 401');
  });
});

describe('asking the user', () => {
  async function running() {
    const h = await open({ ...startChat, chat_answer_approval: () => undefined, chat_answer_question: () => undefined });
    await type('干活');
    await click('chat-send');
    return h;
  }

  it('shows the command itself and offers only choices that mean something', async () => {
    const h = await running();
    h.emit('chat-approval', approval());
    await flushPromises();
    expect(q('approval-preview')!.textContent).toBe('ls -la');
    expect(q('chat-approval')!.textContent).toContain('运行命令');
    expect(q('approval-always'), 'a command is asked every time').toBeNull();
    expect(q('approval-tainted')).toBeNull();
    expect(q('chat-status')!.textContent).toContain('等你确认');
    await click('approval-once');
    expect(h.calls.at(-1)).toEqual(['chat_answer_approval', { id: 'a1', decision: 'once' }]);
    expect(q('chat-approval')).toBeNull();
  });

  it('says why a dangerous operation is asked even in full trust, and never offers to remember it', async () => {
    const h = await running();
    h.emit('chat-approval', approval({ id: 'a9', arguments: { command: 'sudo rm -rf /tmp/x' }, danger: '以管理员权限运行', canRemember: false }));
    await flushPromises();
    expect(q('approval-danger')!.textContent).toContain('以管理员权限运行');
    expect(q('approval-danger')!.textContent).toContain('完全信任');
    expect(q('chat-approval')!.className).toContain('dangerous');
    expect(q('approval-always')).toBeNull();
    expect(q('approval-tainted')).toBeNull();
    await click('approval-deny');
    expect(h.calls.at(-1)).toEqual(['chat_answer_approval', { id: 'a9', decision: 'deny' }]);
    h.emit('chat-approval', approval({ id: 'a10' }));
    await flushPromises();
    expect(q('approval-danger'), 'an ordinary request carries no warning').toBeNull();
  });

  it('offers to remember an ordinary change, explains why an outside-content run is asked every time, and lets the user refuse', async () => {
    const h = await running();
    h.emit('chat-approval', approval({ id: 'a2', tool: 'write_file', risk: 'write', canRemember: true, tainted: true, arguments: { path: 'notes.md', content: '# 今天' } }));
    await flushPromises();
    expect(q('approval-preview')!.textContent).toBe('notes.md\n\n# 今天');
    expect(q('approval-tainted')!.textContent).toContain('外部内容');
    await click('approval-always');
    expect(h.calls.at(-1)).toEqual(['chat_answer_approval', { id: 'a2', decision: 'always' }]);
    h.emit('chat-approval', approval({ id: 'a3' }));
    await flushPromises();
    await click('approval-deny');
    expect(h.calls.at(-1)).toEqual(['chat_answer_approval', { id: 'a3', decision: 'deny' }]);
  });

  it('shows a question with its choices, takes a typed answer, or skips', async () => {
    const h = await running();
    h.emit('chat-question', { id: 'q1', session: 's1', question: '文件放哪里？', options: ['桌面', '文档'] });
    await flushPromises();
    expect(q('chat-question')!.textContent).toContain('文件放哪里？');
    expect(q<HTMLButtonElement>('question-submit')!.disabled).toBe(true);
    await click('question-option-1');
    expect(h.calls.at(-1)).toEqual(['chat_answer_question', { id: 'q1', answer: '文档' }]);

    h.emit('chat-question', { id: 'q2', session: 's1', question: '叫什么名字？', options: [] });
    await flushPromises();
    const input = q('question-input')!.querySelector('input')!;
    input.value = '周报';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await flushPromises();
    await click('question-submit');
    expect(h.calls.at(-1)).toEqual(['chat_answer_question', { id: 'q2', answer: '周报' }]);

    h.emit('chat-question', { id: 'q3', session: 's1', question: '继续吗？', options: [] });
    await flushPromises();
    await click('question-skip');
    expect(h.calls.at(-1)).toEqual(['chat_answer_question', { id: 'q3', answer: null }]);
  });

  it('removes the cards when the answer ends without waiting for them', async () => {
    const h = await running();
    h.emit('chat-approval', approval());
    await flushPromises();
    await h.events('s1', { type: 'cancelled' });
    expect(q('chat-approval')).toBeNull();
  });
});

describe('the list of conversations', () => {
  const history: WireMessage[] = [{ role: 'user', parts: [{ type: 'text', text: '早' }] }, { role: 'assistant', parts: [{ type: 'text', text: '早上好，**今天**天气不错' }] }];
  const listed = { chat_overview: () => overview({ sessions: [summary('s1', '整理文件'), summary('s2', '写周报', 1)] }), chat_load_session: () => history };

  it('lists them with titles and times, newest as the server sent them, and opens one', async () => {
    const h = await open(listed);
    expect(all('.session-title').map(t => t.textContent)).toEqual(['整理文件', '写周报']);
    expect(q('chat-session-s1')!.textContent).toContain('刚刚');
    await click('chat-session-s1');
    expect(h.calls.at(-1)).toEqual(['chat_load_session', { sessionId: 's1' }]);
    expect(q('chat-title')!.textContent).toBe('整理文件');
    expect(wrapper!.find('.answer strong').text()).toBe('今天');
    expect(q('chat-session-s1')!.getAttribute('aria-current')).toBe('true');
  });

  it('goes back to an empty conversation with the new button', async () => {
    await open(listed);
    expect(q<HTMLButtonElement>('chat-new')!.disabled).toBe(true);
    await click('chat-session-s1');
    await click('chat-new');
    expect(q('chat-welcome')).not.toBeNull();
    expect(q('chat-title')!.textContent).toBe('新对话');
  });

  it('keeps rename, move and delete in a menu on each row, and asks before deleting', async () => {
    const h = await open({ ...listed, chat_delete_session: () => [summary('s2', '写周报', 1)] });
    expect(q('chat-delete-s1')).toBeNull();
    await openMenu('s1');
    expect(['chat-menu-rename', 'chat-menu-move', 'chat-menu-delete'].map(id => inBody(id)?.textContent?.trim())).toEqual(['重命名', '移动', '删除']);
    await clickBody('chat-menu-delete');
    expect(q('chat-delete-confirm')).not.toBeNull();
    expect(h.commands()).not.toContain('chat_delete_session');
    await click('chat-delete-yes-s1');
    expect(h.calls.at(-1)).toEqual(['chat_delete_session', { sessionId: 's1' }]);
    expect(all('.session-title').map(t => t.textContent)).toEqual(['写周报']);
  });

  it('marks a conversation that is waiting for the user, and one that is still answering', async () => {
    const h = await open({ chat_overview: () => overview({ sessions: [summary('s1', '整理文件'), summary('s2', '写周报')], running: ['s2'] }) });
    expect(q('chat-session-s2')!.querySelector('.spin')).not.toBeNull();
    await openMenu('s2');
    expect(inBody('chat-menu-delete')!.classList.contains('n-dropdown-option-body--disabled')).toBe(true);
    expect(inBody('chat-menu-rename')!.classList.contains('n-dropdown-option-body--disabled')).toBe(false);
    h.emit('chat-approval', approval({ session: 's1' }));
    await flushPromises();
    expect(q('chat-attention-s1')).not.toBeNull();
    expect(q('chat-attention-s2')).toBeNull();
  });

  it('says so when there is nothing yet', async () => {
    await open();
    expect(wrapper!.find('.empty-list').text()).toBe('还没有对话');
  });
});

/** A host that keeps the list in memory and answers the way the real one does, errors included. */
function hostList(sessions: SessionSummary[], groups: ChatGroup[] = []) {
  const state = { sessions, groups, next: groups.length };
  const library = () => ({ sessions: state.sessions.map(s => ({ ...s })), groups: state.groups.map(g => ({ ...g })) });
  const edit = (id: string, change: (s: SessionSummary) => void) => { const s = state.sessions.find(item => item.id === id); if (!s) throw new Error('这个对话已经不存在'); change(s); };
  const responders: Record<string, Responder> = {
    chat_overview: () => ({ ...overview({ sessions: library().sessions }), groups: library().groups }),
    chat_rename_session: a => {
      const title = String(a?.title).split(/\s+/).filter(Boolean).join(' ');
      if (!title) throw new Error('对话名称不能为空');
      edit(String(a?.sessionId), s => { s.title = title; });
      return library();
    },
    chat_move_session: a => { edit(String(a?.sessionId), s => { s.group = (a?.groupId as string | null) ?? ''; }); return library(); },
    chat_create_group: a => {
      const name = String(a?.name).trim();
      if (state.groups.some(g => g.name === name)) throw new Error('已经有同名的分组');
      const group = { id: `g${++state.next}`, name };
      state.groups.push(group);
      if (a?.sessionId) edit(String(a.sessionId), s => { s.group = group.id; });
      return library();
    },
    chat_rename_group: a => { const g = state.groups.find(item => item.id === a?.groupId)!; g.name = String(a?.name).trim(); return library(); },
    chat_delete_group: a => {
      state.groups = state.groups.filter(g => g.id !== a?.groupId);
      state.sessions.forEach(s => { if (s.group === a?.groupId) s.group = ''; });
      return library();
    },
  };
  return { state, responders };
}
const titles = () => all('.session-title').map(t => t.textContent);
const groupRows = () => all('.group-name').map(t => t.textContent);

describe('renaming conversations', () => {
  const two = () => hostList([summary('s1', '整理文件'), summary('s2', '写周报', 1)]);

  it('opens a dialog filled with the current name and saves the new one', async () => {
    const host = two();
    const h = await open(host.responders);
    await openMenu('s1');
    await clickBody('chat-menu-rename');
    expect(nameDialogOpen()).toBe(true);
    expect(nameInput()).toBe('整理文件');
    await typeName('  周报   第三周 ');
    await submitName();
    expect(h.calls.at(-1)).toEqual(['chat_rename_session', { sessionId: 's1', title: '  周报   第三周 ' }]);
    expect(titles()).toEqual(['周报 第三周', '写周报']);
    expect(nameDialogOpen()).toBe(false);
  });

  it('shows the new name in the header of the open conversation', async () => {
    const host = hostList([summary('s1', '整理文件')]);
    await open({ ...host.responders, chat_load_session: () => [] });
    await click('chat-session-s1');
    expect(q('chat-title')!.textContent).toBe('整理文件');
    await openMenu('s1');
    await clickBody('chat-menu-rename');
    await typeName('新标题');
    await submitName();
    expect(q('chat-title')!.textContent).toBe('新标题');
  });

  it('keeps the dialog open with the text when the host refuses, and does not save an empty name', async () => {
    const host = two();
    const h = await open({ ...host.responders, chat_rename_session: () => { throw new Error('对话名称最多 60 个字'); } });
    await openMenu('s1');
    await clickBody('chat-menu-rename');
    await typeName('太长的名字');
    await submitName();
    expect(nameDialogOpen()).toBe(true);
    expect(nameInput()).toBe('太长的名字');
    expect(h.notify).toHaveBeenCalledWith(expect.objectContaining({ message: '对话名称最多 60 个字' }), 'error');
    expect(titles()).toEqual(['整理文件', '写周报']);

    await typeName('   ');
    expect(inBody<HTMLButtonElement>('chat-name-ok')!.disabled).toBe(true);
    const before = h.commands().length;
    await submitName();
    expect(h.commands().length).toBe(before);
  });

  it('closes without asking the host when the name did not change, and cancel leaves everything alone', async () => {
    const host = two();
    const h = await open(host.responders);
    await openMenu('s1');
    await clickBody('chat-menu-rename');
    await submitName();
    expect(nameDialogOpen()).toBe(false);
    expect(h.commands()).not.toContain('chat_rename_session');

    await openMenu('s1');
    await clickBody('chat-menu-rename');
    await typeName('不保存');
    await clickBody('chat-name-cancel');
    expect(nameDialogOpen()).toBe(false);
    expect(h.commands()).not.toContain('chat_rename_session');
    expect(titles()).toEqual(['整理文件', '写周报']);
  });
});

describe('groups of conversations', () => {
  const work = { id: 'g1', name: '工作' };
  const inGroup = (id: string, title: string, group: string, at = Math.floor(Date.now() / 1000)): SessionSummary => ({ ...summary(id, title, at), group });

  it('shows no group furniture until there is a group', async () => {
    const host = hostList([summary('s1', '整理文件')]);
    await open(host.responders);
    expect(groupRows()).toEqual([]);
    expect(all('.list-label')).toHaveLength(0);
    expect(q('chat-new-group')).not.toBeNull();
  });

  it('lays groups out as folders above the conversations that are in none', async () => {
    const host = hostList([inGroup('s1', '季度报告', 'g1'), summary('s2', '随便聊聊'), inGroup('s3', '会议纪要', 'g1')], [work]);
    await open(host.responders);
    expect(groupRows()).toEqual(['工作']);
    expect(q('chat-group-g1')!.textContent).toContain('2');
    expect(titles()).toEqual(['季度报告', '会议纪要', '随便聊聊']);
    expect(all('.list-label').map(l => l.textContent)).toEqual(['未分组']);
    expect(q('chat-session-s1')!.closest('li')!.classList.contains('grouped')).toBe(true);
    expect(q('chat-session-s2')!.closest('li')!.classList.contains('grouped')).toBe(false);
  });

  it('folds a group away, remembers it, and opens it again', async () => {
    const host = hostList([inGroup('s1', '季度报告', 'g1'), summary('s2', '随便聊聊')], [work]);
    await open(host.responders);
    expect(q('chat-group-g1')!.getAttribute('aria-expanded')).toBe('true');
    await click('chat-group-g1');
    expect(titles()).toEqual(['随便聊聊']);
    expect(q('chat-group-g1')!.getAttribute('aria-expanded')).toBe('false');
    expect(q('chat-group-g1')!.textContent).toContain('1');
    expect(JSON.parse(localStorage.getItem('xiaoban.chat.collapsed-groups')!)).toEqual(['g1']);

    wrapper!.unmount();
    await open(host.responders);
    expect(titles()).toEqual(['随便聊聊']);
    await click('chat-group-g1');
    expect(titles()).toEqual(['季度报告', '随便聊聊']);
  });

  it('creates a group from the button above the list', async () => {
    const host = hostList([summary('s1', '整理文件')]);
    const h = await open(host.responders);
    await click('chat-new-group');
    expect(nameInput()).toBe('');
    expect(inBody('chat-name-dialog')!.textContent).toContain('新建分组');
    await typeName(' 生活 ');
    await submitName();
    expect(h.calls.at(-1)).toEqual(['chat_create_group', { name: ' 生活 ' }]);
    expect(groupRows()).toEqual(['生活']);
    expect(wrapper!.find('.group-empty').exists()).toBe(true);
    expect(nameDialogOpen()).toBe(false);
  });

  it('keeps the dialog open when the group name is taken', async () => {
    const host = hostList([summary('s1', '整理文件')], [work]);
    await open(host.responders);
    await click('chat-new-group');
    await typeName('工作');
    await submitName();
    expect(nameDialogOpen()).toBe(true);
    expect(groupRows()).toEqual(['工作']);
  });

  it('moves a conversation into a group from the menu, and back out', async () => {
    const host = hostList([summary('s1', '整理文件'), summary('s2', '写周报', 1)], [work]);
    const h = await open(host.responders);
    await openMenu('s1');
    await clickBody('chat-menu-move');
    expect(moveDialogOpen()).toBe(true);
    expect(inBody('chat-move-to-none')!.getAttribute('aria-checked')).toBe('true');
    expect(inBody('chat-move-to-g1')!.getAttribute('aria-checked')).toBe('false');
    await clickBody('chat-move-to-g1');
    expect(h.calls.at(-1)).toEqual(['chat_move_session', { sessionId: 's1', groupId: 'g1' }]);
    expect(moveDialogOpen()).toBe(false);
    expect(titles()).toEqual(['整理文件', '写周报']);
    expect(q('chat-session-s1')!.closest('li')!.classList.contains('grouped')).toBe(true);

    await openMenu('s1');
    await clickBody('chat-menu-move');
    expect(inBody('chat-move-to-g1')!.getAttribute('aria-checked')).toBe('true');
    await clickBody('chat-move-to-none');
    expect(h.calls.at(-1)).toEqual(['chat_move_session', { sessionId: 's1', groupId: null }]);
    expect(q('chat-session-s1')!.closest('li')!.classList.contains('grouped')).toBe(false);
  });

  it('does not call the host when the conversation is already where it is picked, and opens a folded target', async () => {
    const host = hostList([inGroup('s1', '季度报告', 'g1'), summary('s2', '随便聊聊')], [work]);
    localStorage.setItem('xiaoban.chat.collapsed-groups', JSON.stringify(['g1']));
    const h = await open(host.responders);
    expect(titles()).toEqual(['随便聊聊']);
    await openMenu('s2');
    await clickBody('chat-menu-move');
    await clickBody('chat-move-to-none');
    expect(h.commands()).not.toContain('chat_move_session');
    expect(moveDialogOpen()).toBe(false);

    await openMenu('s2');
    await clickBody('chat-menu-move');
    await clickBody('chat-move-to-g1');
    // The folded group opens so the move is visible.
    expect(titles()).toEqual(['季度报告', '随便聊聊']);
  });

  it('creates a group while moving, and the conversation lands in it', async () => {
    const host = hostList([summary('s1', '整理文件')]);
    const h = await open(host.responders);
    await openMenu('s1');
    await clickBody('chat-menu-move');
    expect(inBody('chat-move-dialog')!.querySelectorAll('[role="radio"]')).toHaveLength(1);
    await clickBody('chat-move-new-group');
    expect(moveDialogOpen()).toBe(false);
    await typeName('归档');
    await submitName();
    expect(h.calls.at(-1)).toEqual(['chat_create_group', { name: '归档', sessionId: 's1' }]);
    expect(groupRows()).toEqual(['归档']);
    expect(q('chat-session-s1')!.closest('li')!.classList.contains('grouped')).toBe(true);
  });

  it('renames a group from its menu', async () => {
    const host = hostList([inGroup('s1', '季度报告', 'g1')], [work]);
    const h = await open(host.responders);
    await openGroupMenu('g1');
    await clickBody('chat-group-menu-rename');
    expect(nameInput()).toBe('工作');
    expect(inBody('chat-name-dialog')!.textContent).toContain('重命名分组');
    await typeName('项目');
    await submitName();
    expect(h.calls.at(-1)).toEqual(['chat_rename_group', { groupId: 'g1', name: '项目' }]);
    expect(groupRows()).toEqual(['项目']);
  });

  it('asks before deleting a group and keeps its conversations', async () => {
    const host = hostList([inGroup('s1', '季度报告', 'g1'), summary('s2', '随便聊聊')], [work]);
    const h = await open(host.responders);
    await openGroupMenu('g1');
    await clickBody('chat-group-menu-delete');
    expect(q('chat-group-delete-confirm')!.textContent).toContain('里面的对话会保留');
    expect(h.commands()).not.toContain('chat_delete_group');
    await click('chat-group-delete-yes-g1');
    expect(h.calls.at(-1)).toEqual(['chat_delete_group', { groupId: 'g1' }]);
    expect(groupRows()).toEqual([]);
    expect(titles()).toEqual(['季度报告', '随便聊聊']);
  });
});

describe('links in answers', () => {
  async function withLink() {
    const h = await open({ chat_overview: () => overview({ sessions: [summary('s1', '链接')] }), chat_load_session: () => [{ role: 'assistant', parts: [{ type: 'text', text: '看 [文档](https://example.com/doc#part) 吧' }] }] as WireMessage[], open_download_url: () => undefined });
    await click('chat-session-s1');
    return h;
  }

  it('opens a link in the system browser, without the part after #', async () => {
    const h = await withLink();
    wrapper!.find('a.md-link').element.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    await flushPromises();
    expect(h.calls.at(-1)).toEqual(['open_download_url', { url: 'https://example.com/doc' }]);
  });

  it('copies the address instead when the app will not open it', async () => {
    const h = await withLink();
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    h.invoke.mockImplementationOnce(async () => { throw new Error('只支持 https'); });
    wrapper!.find('a.md-link').element.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    await flushPromises();
    expect(writeText).toHaveBeenCalledWith('https://example.com/doc#part');
    expect(h.notify).toHaveBeenCalledWith('这个地址不能直接打开，已复制到剪贴板', 'info');
  });
});

describe('permission mode toggle', () => {
  it('shows the current permission mode and switches between default and full trust', async () => {
    const h = await open({
      chat_overview: () => overview({ permissionMode: 'ask' }),
      chat_set_permission_mode: ({ mode }: any) => mode,
    });
    expect(q('perm-ask')?.classList.contains('active')).toBe(true);
    await click('chat-permission-toggle');
    expect(q('chat-permission-toggle')?.getAttribute('aria-expanded')).toBe('true');
    expect(q('perm-auto')?.classList.contains('active')).toBe(false);

    await click('perm-auto');
    expect(h.calls.at(-1)).toEqual(['chat_set_permission_mode', { mode: 'auto' }]);
    expect(q('perm-auto')?.classList.contains('active')).toBe(true);
    expect(q('perm-ask')?.classList.contains('active')).toBe(false);

    await click('perm-ask');
    expect(h.calls.at(-1)).toEqual(['chat_set_permission_mode', { mode: 'ask' }]);
    expect(q('perm-ask')?.classList.contains('active')).toBe(true);
  });
});

describe('hiding tool calls', () => {
  it('hides tool call details when hideToolCalls is true in store', async () => {
    const h = await open({ ...startChat });
    await type('查看文件');
    await click('chat-send');
    await h.events('s1', { type: 'tool_start', id: 't1', name: 'shell', arguments: '{"command":"ls"}' });
    expect(q('tool-t1')).not.toBeNull();

    h.chat.hideToolCalls = true;
    await flushPromises();
    expect(q('tool-t1')).toBeNull();

    h.chat.hideToolCalls = false;
    await flushPromises();
    expect(q('tool-t1')).not.toBeNull();
  });
});

/** A real one-pixel PNG, so FileReader produces the same `data:` URL the host will accept. */
const PNG_BASE64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';
const pngFile = (name = 'shot.png') => new File([Uint8Array.from(atob(PNG_BASE64), c => c.charCodeAt(0))], name, { type: 'image/png' });

/** FileReader finishes on its own schedule in happy-dom, so tests wait for the effect. */
async function settleAttachments(count?: number) {
  for (let attempt = 0; attempt < 40; attempt++) {
    await flushPromises();
    await new Promise(resolve => setTimeout(resolve, 5));
    const shown = all('[data-testid^="chat-attachment-"]').filter(el => !el.dataset.testid!.includes('remove')).length;
    if (count === undefined ? true : shown >= count) {
      await flushPromises();
      return;
    }
  }
  await flushPromises();
}

/** Puts files on the hidden picker, the way the system dialog would. */
async function pickFiles(files: File[]) {
  const input = q<HTMLInputElement>('chat-image-input')!;
  Object.defineProperty(input, 'files', { value: files, configurable: true });
  input.dispatchEvent(new Event('change', { bubbles: true }));
  await settleAttachments(files.length);
}

/** Pastes files, which is how a screenshot arrives from the clipboard. */
async function pasteFiles(files: File[]) {
  const event = new Event('paste', { bubbles: true, cancelable: true });
  Object.defineProperty(event, 'clipboardData', { value: { files }, configurable: true });
  q('chat-input')!.closest('footer')!.dispatchEvent(event);
  await settleAttachments(files.length);
}

describe('sending images', () => {
  it('adds a pasted image, sends it with the text, and shows it in the conversation', async () => {
    const h = await open({ ...startChat, chat_send: () => summary('s1', '看这个截图') });
    await pasteFiles([pngFile()]);
    expect(q('chat-attachment-0')).not.toBeNull();
    expect(q('chat-attachments')!.textContent).toContain('1/8');

    await type('这个报错是什么原因');
    await click('chat-send');

    const sent = h.calls.find(([command]) => command === 'chat_send')!;
    expect(sent[1]).toMatchObject({ text: '这个报错是什么原因' });
    expect(sent[1]!.images).toHaveLength(1);
    expect(String((sent[1]!.images as string[])[0])).toMatch(/^data:image\/png;base64,/);
    // The composer is cleared, and the image is now shown on the sent message.
    expect(q('chat-attachments')).toBeNull();
    expect(q('msg-user-image-0')).not.toBeNull();
  });

  it('sends an image with no text at all', async () => {
    const h = await open({ ...startChat });
    await pickFiles([pngFile()]);
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);

    await click('chat-send');
    const sent = h.calls.find(([command]) => command === 'chat_send')!;
    expect(sent[1]).toMatchObject({ text: '' });
    expect(sent[1]!.images).toHaveLength(1);
  });

  it('lets one image be removed before sending, and closes the picker at the limit', async () => {
    const h = await open({ ...startChat });
    await pickFiles([pngFile('a.png'), pngFile('b.png')]);
    expect(q('chat-attachment-0')).not.toBeNull();
    expect(q('chat-attachment-1')).not.toBeNull();

    await click('chat-attachment-remove-0');
    expect(q('chat-attachment-1')).toBeNull();
    expect(q('chat-attachment-0')).not.toBeNull();

    await pickFiles(Array.from({ length: 8 }, (_, n) => pngFile(`f${n}.png`)));
    expect(h.notify).toHaveBeenCalledWith('一次最多添加 8 个附件', 'info');
    expect(q<HTMLButtonElement>('chat-pick-image')!.disabled).toBe(true);

    await type('还是有文字');
    await click('chat-send');
    expect((h.calls.find(([c]) => c === 'chat_send')![1]!.images as string[])).toHaveLength(8);
  });

  it('refuses a type or a size the host would reject, and says why', async () => {
    const h = await open({ ...startChat });
    await pickFiles([new File(['<svg/>'], 'icon.svg', { type: 'image/svg+xml' })]);
    expect(h.notify).toHaveBeenCalledWith('暂不支持这种文件，请选择图片、PDF、Office 文档或文本文件', 'error');
    expect(q('chat-attachments')).toBeNull();

    const huge = new File([new Uint8Array(3 * 1024 * 1024 + 1)], 'big.png', { type: 'image/png' });
    await pickFiles([huge]);
    expect(h.notify).toHaveBeenCalledWith('这张图片太大了，请换一张小于 3MB 的', 'error');
    expect(q('chat-attachments')).toBeNull();
  });

  it('hands the image back when the host refuses the message', async () => {
    const h = await open({ ...startChat, chat_send: () => { throw new Error('这个对话还在回答'); } });
    await pickFiles([pngFile()]);
    await type('看看');
    await click('chat-send');

    expect(h.notify).toHaveBeenCalledWith(expect.anything(), 'error');
    expect(q('chat-attachment-0')).not.toBeNull();
    expect(textarea().value).toBe('看看');
  });

  it('leaves plain text untouched when nothing is pasted', async () => {
    const h = await open({ ...startChat });
    await type('只是文字');
    await click('chat-send');
    expect(h.calls.find(([c]) => c === 'chat_send')![1]).toEqual({ sessionId: 's1', text: '只是文字' });
  });

  it('shows images that were sent earlier and opens one larger when clicked', async () => {
    const history: WireMessage[] = [
      { role: 'user', parts: [{ type: 'image', url: 'data:image/png;base64,AA' }, { type: 'text', text: '看这张图' }] },
      { role: 'assistant', parts: [{ type: 'text', text: '看到了' }] },
    ];
    await open({ chat_overview: () => overview({ sessions: [summary('s1', '看这张图')] }), chat_load_session: () => history });
    await click('chat-session-s1');

    const shown = q<HTMLImageElement>('msg-user-image-0');
    expect(shown).not.toBeNull();
    expect(shown!.getAttribute('src')).toBe('data:image/png;base64,AA');
    expect(q('msg-user')!.textContent).toContain('看这张图');

    shown!.click();
    await flushPromises();
    expect(q('chat-image-preview')).not.toBeNull();
    await click('chat-image-preview-close');
    expect(q('chat-image-preview')).toBeNull();
  });
});

describe('the context ring in the composer', () => {
  const info = (used: number, canCompact = true) => ({ used, limit: 256_000, canCompact });
  const say = (value: string) => [{ type: 'text' as const, text: value }];
  const history: WireMessage[] = [{ role: 'user', parts: say('早') }, { role: 'assistant', parts: say('早上好') }];
  const summarized: WireMessage[] = [...history, { role: 'user', synthetic: true, context_summary: { keep_from: 2, text: '摘要正文' } }];
  const listed = { chat_overview: () => overview({ sessions: [summary('s1', '整理文件')] }), chat_load_session: () => history };

  it('shows the ring next to the send button, with the numbers from the host in its popover', async () => {
    await open({ ...listed, chat_context: () => info(12_300) });
    await flushPromises();
    expect(q('chat-context')).not.toBeNull();
    expect(q('chat-context')!.compareDocumentPosition(q('chat-workbench-model')!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(q('chat-workbench-model')!.compareDocumentPosition(q('chat-send')!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    await click('chat-context');
    expect(inBody('ctx-numbers')!.textContent).toBe('12.3K / 256K');
  });

  it('is not there in the browser preview', async () => {
    await open({}, { desktop: false });
    expect(q('chat-context')).toBeNull();
  });

  it('reads the numbers of the conversation that is opened', async () => {
    const h = await open({ ...listed, chat_context: args => (args?.sessionId === 's1' ? info(30_000) : info(3_000)) });
    await flushPromises();
    await click('chat-context');
    expect(inBody('ctx-numbers')!.textContent).toBe('3K / 256K');
    await click('chat-session-s1');
    await vi.waitFor(() => expect(inBody('ctx-numbers')!.textContent).toBe('30K / 256K'));
    expect(h.contextCalls.at(-1)).toEqual({ sessionId: 's1' });
  });

  it('compacts from the popover and then shows the summary in the conversation', async () => {
    let compacted = false;
    const h = await open({
      ...listed,
      chat_load_session: () => (compacted ? summarized : history),
      chat_context: () => (compacted ? info(2_000, false) : info(40_000)),
      chat_compact: () => { compacted = true; return info(2_000, false); },
    });
    await click('chat-session-s1');
    await click('chat-context');
    await vi.waitFor(() => expect(inBody('ctx-numbers')!.textContent).toBe('40K / 256K'));
    await clickBody('ctx-compact');
    await vi.waitFor(() => expect(inBody('ctx-numbers')!.textContent).toBe('2K / 256K'));
    expect(h.calls.map(([command]) => command)).toContain('chat_compact');
    const notice = q('notice-info')!;
    expect(notice.textContent).toContain('压缩成摘要');
    expect(notice.querySelector('summary')!.textContent).toBe('查看摘要');
    expect(notice.querySelector('.notice-details')!.textContent).toBe('摘要正文');
    expect(inBody<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
  });

  it('cannot send while the conversation is being compacted, and can again afterwards', async () => {
    let release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; });
    await open({ ...listed, chat_context: () => info(40_000), chat_compact: async () => { await gate; return info(2_000, false); } });
    await click('chat-session-s1');
    await type('接着聊');
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
    await click('chat-context');
    await clickBody('ctx-compact');
    expect(inBody('ctx-compact')!.textContent).toContain('正在压缩…');
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(true);
    release();
    await flushPromises();
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
  });

  it('cannot compact while the conversation is answering', async () => {
    await open({ chat_overview: () => overview({ sessions: [summary('s1', '整理文件')], running: ['s1'] }), chat_load_session: () => history, chat_context: () => info(40_000) });
    await click('chat-session-s1');
    await click('chat-context');
    expect(inBody<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
    expect(inBody('ctx-compact')!.parentElement!.getAttribute('title')).toBe('回答结束后才能压缩');
  });

  it('keeps the compact button off for a conversation that has not started', async () => {
    await open({ ...listed, chat_context: () => info(2_500, true) });
    await flushPromises();
    await click('chat-context');
    expect(inBody<HTMLButtonElement>('ctx-compact')!.disabled).toBe(true);
  });
});

describe('the limit slider and automatic compaction in the composer', () => {
  const info = (used: number, limit = 256_000, canCompact = true) => ({ used, limit, canCompact });
  const say = (value: string) => [{ type: 'text' as const, text: value }];
  const history: WireMessage[] = [{ role: 'user', parts: say('早') }, { role: 'assistant', parts: say('早上好') }];
  const listed = { chat_overview: () => overview({ sessions: [summary('s1', '整理文件')] }), chat_load_session: () => history };

  it('saves the limit a moment after the slider stops and shows the host\'s number', async () => {
    let limit = 256_000;
    const h = await open({ ...listed, chat_context: () => info(64_000, limit), set_context_tokens: a => { limit = Number(a?.tokens); return undefined; } });
    await flushPromises();
    await click('chat-context');
    wrapper!.findComponent(NSlider).vm.$emit('update:value', 640_000);
    await flushPromises();
    expect(inBody('ctx-numbers')!.textContent).toBe('64K / 640K');
    await vi.waitFor(() => expect(h.calls.at(-1)).toEqual(['set_context_tokens', { tokens: 640_000 }]));
    await vi.waitFor(() => expect(h.chat.context.limit).toBe(640_000));
    expect(inBody('ctx-numbers')!.textContent).toBe('64K / 640K');
  });

  it('shows "正在压缩…" and blocks sending while the host compacts a full conversation by itself', async () => {
    const h = await open({ ...listed, chat_context: () => info(240_000) });
    await click('chat-session-s1');
    await type('接着聊');
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
    await click('chat-context');

    h.emit('chat-compacting', { session: 's1', state: 'start' });
    await flushPromises();
    expect(inBody('ctx-compact')!.textContent).toContain('正在压缩…');
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(true);

    h.emit('chat-compacting', { session: 's1', state: 'done', summary: '摘要正文' });
    await flushPromises();
    expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
    const notice = q('notice-info')!;
    expect(notice.querySelector('summary')!.textContent).toBe('查看摘要');
    expect(notice.querySelector('.notice-details')!.textContent).toBe('摘要正文');
  });
});


it('picks and sends a document-only question with its named attachment card', async () => {
  const h=await open({...startChat});
  await pickFiles([new File(['amount\n12'],'预算.csv',{type:'text/csv'})]);
  expect(q('chat-attachments')?.textContent).toContain('预算.csv');
  await click('chat-send');
  const sent=h.calls.find(([command])=>command==='chat_send')?.[1];
  expect(sent).toMatchObject({text:'',attachments:[{name:'预算.csv',dataUrl:expect.stringMatching(/^data:text\/csv;base64,/)}]});
  expect(q('msg-user')?.textContent).toContain('预算.csv');
});
