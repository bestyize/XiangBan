// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { NSlider } from 'naive-ui';
import { reactive } from 'vue';
import ChatPage from './ChatPage.vue';
import ChatNameDialog from '../components/ChatNameDialog.vue';
import { createChatStore, CONTINUE_TEXT, type ChatDeps } from '../chat/chat-store.ts';
import type { ControlStore } from '../control-types.ts';
import type { ApprovalRequest, ChatGroup, KernelEvent, Overview, SessionSummary, WireMessage } from '../chat/types.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
import { invoke as bridgeInvoke } from '../../bridge.ts';

const summary = (id: string, title: string, updatedAt = Math.floor(Date.now() / 1000)): SessionSummary => ({ id, title, createdAt: updatedAt, updatedAt });
const overview = (patch: Partial<Overview> = {}): Overview => ({ configured: true, sessions: [], running: [], workspace: '/Users/me/小伴/工作区', permissionMode: 'ask', hideToolCalls: false, ...patch });

type Responder = (args: Record<string, unknown> | undefined) => unknown;
let wrapper: VueWrapper | undefined;
let currentStore: ControlStore | undefined;

async function open(responders: Record<string, Responder> = {}, options: { desktop?: boolean; extra?: Record<string, unknown> } = {}) {
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
  const store = reactive({ desktop, chat, settingsTab: 'general', page: 'chat', navigate: vi.fn(), notify, openSettings: vi.fn(), ...options.extra }) as unknown as ControlStore;
  vi.mocked(store.openSettings).mockImplementation(tab => { if (tab) store.settingsTab = tab; });
  currentStore = store;
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
/** Conversations are opened from the 空间 tree in the sidebar, which selects them in the chat store. */
async function openSession(id: string) { await currentStore!.chat.select(id); await flushPromises(); }
/** The title bar's ⋯ menu of the open conversation. */
async function openMenu(id: string) { await openSession(id); await click('chat-more'); }
async function typeName(value: string) { const input = inBody('chat-name-input')!.querySelector('input')!; input.value = value; input.dispatchEvent(new Event('input')); await flushPromises(); }
async function submitName() { inBody('chat-name-ok')!.closest('form')!.dispatchEvent(new Event('submit', { cancelable: true })); await flushPromises(); }
const nameInput = () => inBody('chat-name-input')?.querySelector('input')?.value;
// A closed modal stays in the document until its leave transition ends, which never happens here, so ask the component.
const nameDialogOpen = () => wrapper!.findComponent(ChatNameDialog).props('show');
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
      chat_invocable_skills: () => [{ name: 'check', displayName: '质量检查', description: '分析测试', source: 'installed' }],
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
    expect(textarea().value.startsWith('/check ')).toBe(true);
    await click('chat-add-menu');
    await click('chat-add-connectors');
    await click('chat-installed-github');
    expect(textarea().value).toContain('GitHub');
    expect(h.commands()).toContain('ai_store_connector_status');
  });

  it('offers the skills the user may start when a /name is typed, and inserts the chosen one', async () => {
    const h = await open({
      chat_invocable_skills: () => [
        { name: 'pptx', displayName: 'PPT 演示文稿', description: '做幻灯片', source: 'builtin' },
        { name: 'xlsx', displayName: 'Excel 表格', description: '处理表格', source: 'plugin', plugin: 'office-suite' },
        { name: 'pdf-tools', displayName: 'pdf-tools', description: '处理 PDF', source: 'installed' },
      ],
    });
    await type('先做 /');
    expect(h.commands()).toContain('chat_invocable_skills');
    expect(all('.slash-item').map(e => e.dataset.testid)).toEqual(['chat-slash-pdf-tools', 'chat-slash-pptx', 'chat-slash-xlsx']);
    expect(q('chat-slash-xlsx')?.textContent).toContain('office-suite');
    await type('先做 /p');
    expect(all('.slash-item').map(e => e.dataset.testid)).toEqual(['chat-slash-pdf-tools', 'chat-slash-pptx']);
    // Down moves the highlight, Enter inserts it instead of sending.
    expect((await press({ key: 'ArrowDown' })).defaultPrevented).toBe(true);
    expect((await press({ key: 'Enter' })).defaultPrevented).toBe(true);
    expect(textarea().value).toBe('先做 /pptx ');
    expect(q('chat-slash-menu')).toBeNull();
    expect(h.commands()).not.toContain('chat_send');
    // A finished name or a path opens nothing; Escape closes the menu and leaves the text alone.
    await type('看 /etc/hosts');
    expect(q('chat-slash-menu')).toBeNull();
    await type('/x');
    expect(q('chat-slash-menu')).not.toBeNull();
    await press({ key: 'Escape' });
    expect(q('chat-slash-menu')).toBeNull();
    expect(textarea().value).toBe('/x');
  });

  it('keeps the slash menu away when nothing matches or the app is not the desktop one', async () => {
    await open({ chat_invocable_skills: () => [{ name: 'pptx', displayName: 'PPT', description: '', source: 'builtin' }] });
    await type('/zzz');
    expect(q('chat-slash-menu')).toBeNull();
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

  it('shows a friendly gateway mode label and updates it after switching', async () => {
    const config = { activeProvider: 'xiaoban-ai', gatewayEnabled: true, gatewayTier: 'standard', model: 'standard', providers: [] };
    const tiers = [
      { tier: 'standard', displayName: '日常', available: true },
      { tier: 'advanced', displayName: '深入', available: true },
    ];
    const h = await open({ get_ai_config: () => config, refresh_ai_providers: () => config, ai_gateway_tiers: () => tiers,
      use_ai_gateway: args => { config.gatewayTier = config.model = args!.tier as string; return config; } });
    expect(q('chat-workbench-model')!.textContent).toContain('小伴 AI · 日常');
    await click('chat-workbench-model');
    await click('chat-model-xiaoban-ai-advanced');
    expect(h.calls).toContainEqual(['use_ai_gateway', { tier: 'advanced' }]);
    expect(q('chat-workbench-model')!.textContent).toContain('小伴 AI · 深入');
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
    expect(h.store.openSettings).toHaveBeenCalledWith('ai');
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

  it('keeps AI model settings in the title bar\'s more menu and opens the AI tab of the settings dialog', async () => {
    const h = await open();
    await click('chat-more');
    expect(q('chat-more')!.getAttribute('aria-expanded')).toBe('true');
    expect(q('chat-ai-config')!.textContent).toContain('AI 模型配置');
    await click('chat-ai-config');
    expect(h.store.settingsTab).toBe('ai');
    expect(h.store.openSettings).toHaveBeenCalledWith('ai');
    expect(q('chat-more')!.getAttribute('aria-expanded')).toBe('false');
  });

  it('shows where a new conversation will go in the title bar: 最近任务, or the folder chosen for it', async () => {
    const h = await open();
    expect(q('chat-workspace')!.textContent).toBe('最近任务');
    expect(q('chat-title')!.textContent).toBe('新对话');
    h.chat.pendingWorkspace = '/Users/me/官网改版';
    await flushPromises();
    expect(q('chat-workspace')!.textContent).toBe('官网改版');
  });

  it('opens the capability guide from the more menu or welcome screen', async () => {
    await open();
    await click('chat-more');
    await click('chat-open-guide');
    expect(document.body.textContent).toContain('小伴桌面 · AI 玩法指南');
  });

  it('opens long-term goals in a separate dialog and closes it without changing the chat', async () => {
    await open();
    expect(inBody('chat-goal-panel')).toBeNull();
    await click('chat-more');
    expect(q('chat-goal-toggle')!.getAttribute('aria-haspopup')).toBe('dialog');
    await click('chat-goal-toggle');
    const dialog = inBody('chat-goal-panel')!;
    expect(dialog.getAttribute('role')).toBe('dialog');
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    expect(dialog.textContent).toContain('想完成什么');
    expect(q('chat-goal-panel')).toBeNull();
    expect(q('chat-more')!.getAttribute('aria-expanded')).toBe('false');
    expect(q('chat-input')).not.toBeNull();
    await clickBody('goal-close');
    expect(inBody('chat-goal-panel')).toBeNull();
    expect(q('chat-input')).not.toBeNull();
  });
});

describe('实时对话', () => {
  it('sits beside send and turns the box into a voice bar', async () => {
    const asr = reactive({ enabled: true, ready: true, microphoneSupported: true, listening: false, recordingRequested: false, saving: false, init: vi.fn(async () => {}), deactivate: vi.fn() });
    const live = reactive({ active: false, muted: false, since: 0, partial: '', heard: '', status: '', start: vi.fn(async () => { live.active = true; live.status = '在听'; live.since = Date.now(); }), end: vi.fn(() => { live.active = false; }), toggleMute: vi.fn(async () => {}) });
    await open({}, { extra: { asr, live } });
    expect(q('chat-send')).not.toBeNull();
    await click('chat-live-start');
    expect(live.start).toHaveBeenCalledOnce();
    expect(q('chat-input')).toBeNull();
    expect(q('chat-live-status')!.textContent).toContain('在听');
    await click('chat-live-mute'); expect(live.toggleMute).toHaveBeenCalledOnce();
    await click('chat-live-end'); expect(live.end).toHaveBeenCalledOnce();
    await type('打字后两个按钮都在');
    expect(q('chat-live-start')).not.toBeNull(); expect(q<HTMLButtonElement>('chat-send')!.disabled).toBe(false);
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

  it('puts a turn’s tool history under the 小伴 heading, above its replies, and folds it to a summary when finished', async () => {
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
    expect(q('activity-toggle')!.textContent).toContain('已完成');
    expect(q('activity-toggle')!.textContent).toMatch(/用时 \d+ 秒/);
    const messages = all('[data-testid="msg-assistant"]');
    expect(messages.at(-1)!.textContent).toContain('页面读取完成');
    expect(q('chat-activity')!.compareDocumentPosition(messages[0]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(q('chat-activity')!.closest('[data-testid="chat-reply"]')!.querySelector('.reply-head')!.textContent).toContain('小伴');
    await click('activity-toggle');
    expect(all('[data-testid^="tool-"]')).toHaveLength(12);
  });

  it('offers to continue a cut-off answer', async () => {
    const h = await open({ ...startChat, chat_send: () => summary('s1', 'x') });
    await type('写');
    await click('chat-send');
    await h.events('s1', { type: 'text_delta', text: '写到一半' }, { type: 'truncated' }, { type: 'done' });
    expect(q('notice-warning')!.textContent).toContain('单次输出上限');
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

  it('undoes an answer\'s file changes only after a second, explicit confirmation', async () => {
    const h = await open({ ...startChat, chat_undo_run: () => ({ restored: ['a.md'], removed: [], failed: [] }) });
    await type('干活');
    await click('chat-send');
    await h.events('s1', { type: 'done' });
    h.emit('chat-checkpoint', { session: 's1', runId: 'r1', files: ['a.md'] });
    await flushPromises();
    await click('chat-undo-run');
    expect(h.calls.some(([command]) => command === 'chat_undo_run'), 'the first click only asks').toBe(false);
    await click('chat-undo-confirm');
    expect(h.calls.at(-1)).toEqual(['chat_undo_run', { sessionId: 's1', runId: 'r1' }]);
    expect(q('chat-undo-run')).toBeNull();
  });

  it('offers to remember one kind of command and names it', async () => {
    const h = await running();
    h.emit('chat-approval', approval({ id: 'a7', arguments: { command: 'npm test' }, canRemember: true, rememberAs: 'npm test' }));
    await flushPromises();
    expect(q('approval-always')!.textContent).toContain('npm test');
    await click('approval-always');
    expect(h.calls.at(-1)).toEqual(['chat_answer_approval', { id: 'a7', decision: 'always' }]);
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

describe('a conversation in the page', () => {
  const history: WireMessage[] = [{ role: 'user', parts: [{ type: 'text', text: '早' }] }, { role: 'assistant', parts: [{ type: 'text', text: '早上好，**今天**天气不错' }] }];
  const listed = { chat_overview: () => overview({ sessions: [summary('s1', '整理文件'), { ...summary('s2', '写周报', 1), workspace: '/Users/me/周报自动化' }] }), chat_load_session: () => history };

  it('has no list of its own: the sidebar opens one, and the title bar shows its space and title', async () => {
    const h = await open(listed);
    expect(q('chat-session-s1'), 'the 空间 tree in the sidebar lists conversations').toBeNull();
    expect(q('chat-new-group')).toBeNull();
    await openSession('s1');
    expect(h.calls.at(-1)).toEqual(['chat_load_session', { sessionId: 's1' }]);
    expect(q('chat-title')!.textContent).toBe('整理文件');
    expect(q('chat-workspace')!.textContent).toBe('最近任务');
    expect(wrapper!.find('.answer strong').text()).toBe('今天');
    await openSession('s2');
    expect(q('chat-workspace')!.textContent).toBe('周报自动化');
  });

  it('shows a reply with the 小伴 heading, a light user bubble and copy / regenerate actions', async () => {
    const h = await open({ ...listed, chat_send: () => summary('s1', '整理文件') });
    await openSession('s1');
    expect(q('msg-user')!.querySelector('.bubble')!.textContent).toBe('早');
    expect(q('chat-reply')!.querySelector('.reply-head')!.textContent).toContain('小伴');
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    await click('chat-copy-reply');
    expect(writeText).toHaveBeenCalledWith('早上好，**今天**天气不错');
    await click('chat-regenerate');
    expect(h.calls.at(-1)).toEqual(['chat_send', { sessionId: 's1', text: '早' }]);
  });

  it('saves a reply to 资料库 with the conversation it came from', async () => {
    const h = await open({ ...listed, library_save: () => 'saved-1' });
    await openSession('s1');
    await click('chat-save-reply');
    expect(h.calls.find(([command]) => command === 'library_save')).toEqual(['library_save', { item: { kind: 'answer', title: '', text: '早上好，**今天**天气不错', sessionId: 's1', sessionTitle: expect.any(String) } }]);
    expect(q('chat-save-reply')!.getAttribute('aria-label')).toBe('已存到资料库');
  });

  it('keeps delete in the title bar menu, asks first, then goes back to 新建任务', async () => {
    const h = await open({ ...listed, chat_delete_session: () => [summary('s2', '写周报', 1)] });
    await openMenu('s1');
    expect(['chat-menu-rename', 'chat-menu-delete'].map(id => q(id)?.textContent?.trim())).toEqual(['重命名', '删除对话']);
    await click('chat-menu-delete');
    expect(q('chat-delete-confirm')).not.toBeNull();
    expect(h.commands()).not.toContain('chat_delete_session');
    await click('chat-delete-yes-s1');
    expect(h.calls.at(-1)).toEqual(['chat_delete_session', { sessionId: 's1' }]);
    expect(q('chat-delete-confirm')).toBeNull();
    expect(h.store.navigate).toHaveBeenCalledWith('new-task');
  });

  it('does not offer to delete a conversation that is still answering, nor rename one that has not started', async () => {
    await open({ chat_overview: () => overview({ sessions: [summary('s1', '整理文件')], running: ['s1'] }), chat_load_session: () => history });
    expect(q<HTMLButtonElement>('chat-menu-rename')!.disabled).toBe(true);
    await openMenu('s1');
    expect(q<HTMLButtonElement>('chat-menu-delete')!.disabled).toBe(true);
    expect(q<HTMLButtonElement>('chat-menu-rename')!.disabled).toBe(false);
  });

  it('finds text in the conversation from the search button', async () => {
    await open(listed);
    await openSession('s1');
    await click('chat-search-toggle');
    const input = q<HTMLInputElement>('chat-search-input')!;
    input.value = '天气';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await flushPromises();
    expect(q('chat-search-count')!.textContent).toBe('1/1');
    expect(wrapper!.find('.answer').classes()).toContain('is-hit');
    input.value = '没有的词';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await flushPromises();
    expect(q('chat-search-count')!.textContent).toBe('无结果');
  });

  it('shows the execution panel with the steps of the conversation, and remembers it', async () => {
    const h = await open(startChat);
    expect(q('chat-exec-panel')).toBeNull();
    await click('chat-panel-toggle');
    expect(q('chat-exec-panel')!.textContent).toContain('还没有执行操作');
    await type('看看文件');
    await click('chat-send');
    await h.events('s1', { type: 'tool_start', id: 't1', name: 'write_file', arguments: '{"path":"/w/周计划.md","content":"x"}' }, { type: 'tool_end', id: 't1', output: 'ok', is_error: false }, { type: 'done' });
    expect(q('chat-exec-panel')!.textContent).toContain('写入文件');
    expect(q('chat-exec-panel')!.textContent).toContain('周计划.md');
    expect(q('chat-file-card')!.textContent).toContain('周计划.md');
    expect(localStorage.getItem('xiaoban.chat.exec-panel')).toBe('1');
  });

  it('collapses file changes by default, deduplicates paths and keeps the folder action available', async () => {
    const h = await open({ ...startChat, chat_reveal_workspace: () => undefined });
    await type('修改两个文件');
    await click('chat-send');
    await h.events('s1',
      { type: 'tool_start', id: 'f1', name: 'write_file', arguments: '{"path":"/w/index.html","content":"x"}' },
      { type: 'tool_end', id: 'f1', output: 'ok', is_error: false },
      { type: 'tool_start', id: 'f2', name: 'edit_file', arguments: '{"path":"/w/index.html","old_text":"x","new_text":"y"}' },
      { type: 'tool_end', id: 'f2', output: 'ok', is_error: false },
      { type: 'tool_start', id: 'f3', name: 'write_file', arguments: '{"path":"/w/check.mjs","content":"z"}' },
      { type: 'tool_end', id: 'f3', output: 'ok', is_error: false },
      { type: 'tool_start', id: 'f4', name: 'write_file', arguments: '{"path":"/w/failed.txt","content":"z"}' },
      { type: 'tool_end', id: 'f4', output: 'denied', is_error: true },
      { type: 'done' },
    );
    const disclosure = q<HTMLDetailsElement>('chat-files')!;
    expect(disclosure.open).toBe(false);
    expect(q('chat-files-toggle')!.querySelector('.file-count')!.textContent).toBe('2');
    await click('chat-files-toggle');
    expect(disclosure.open).toBe(true);
    const files = all('[data-testid="chat-file-card"]');
    expect(files).toHaveLength(2);
    expect(files[0].textContent).toContain('index.html');
    expect(files[0].textContent).toContain('已修改');
    expect(files[1].textContent).toContain('已写入');
    files[0].querySelector<HTMLButtonElement>('button[aria-label="打开文件夹"]')!.click();
    await flushPromises();
    expect(h.calls).toContainEqual(['chat_reveal_workspace', { sessionId: 's1' }]);
    await click('chat-files-toggle');
    expect(disclosure.open).toBe(false);
  });
});

/** A host that keeps the list in memory and answers the way the real one does, errors included. */
function hostList(sessions: SessionSummary[], groups: ChatGroup[] = []) {
  const state = { sessions, groups, next: groups.length };
  const library = () => ({ sessions: state.sessions.map(s => ({ ...s })), groups: state.groups.map(g => ({ ...g })) });
  const edit = (id: string, change: (s: SessionSummary) => void) => { const s = state.sessions.find(item => item.id === id); if (!s) throw new Error('这个对话已经不存在'); change(s); };
  const responders: Record<string, Responder> = {
    chat_overview: () => ({ ...overview({ sessions: library().sessions }), groups: library().groups }),
    chat_load_session: () => [],
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
const titles = () => currentStore!.chat.sessions.map(s => s.title);

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
    await openMenu('s1');
    expect(q('chat-title')!.textContent).toBe('整理文件');
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

describe('links in answers', () => {
  async function withLink() {
    const h = await open({ chat_overview: () => overview({ sessions: [summary('s1', '链接')] }), chat_load_session: () => [{ role: 'assistant', parts: [{ type: 'text', text: '看 [文档](https://example.com/doc#part) 吧' }] }] as WireMessage[], open_download_url: () => undefined });
    await openSession('s1');
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
    await openSession('s1');

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
    await openSession('s1');
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
    await openSession('s1');
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
    await openSession('s1');
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
    await openSession('s1');
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
    await openSession('s1');
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
