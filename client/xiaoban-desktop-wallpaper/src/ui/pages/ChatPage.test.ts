// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import ChatPage from './ChatPage.vue';
import { createChatStore, CONTINUE_TEXT, type ChatDeps } from '../chat/chat-store.ts';
import type { ControlStore } from '../control-types.ts';
import type { ApprovalRequest, KernelEvent, Overview, SessionSummary, WireMessage } from '../chat/types.ts';

vi.mock('../../bridge.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
import { invoke as bridgeInvoke } from '../../bridge.ts';

const summary = (id: string, title: string, updatedAt = Math.floor(Date.now() / 1000)): SessionSummary => ({ id, title, createdAt: updatedAt, updatedAt });
const overview = (patch: Partial<Overview> = {}): Overview => ({ configured: true, sessions: [], running: [], workspace: '/Users/me/小伴/工作区', permissionMode: 'ask', hideToolCalls: false, ...patch });

type Responder = (args: Record<string, unknown> | undefined) => unknown;
let wrapper: VueWrapper | undefined;

async function open(responders: Record<string, Responder> = {}, options: { desktop?: boolean } = {}) {
  const handlers = new Map<string, (payload: never) => void>();
  const calls: [string, Record<string, unknown> | undefined][] = [];
  const table: Record<string, Responder> = { chat_overview: () => overview(), ...responders };
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    calls.push([command, args]);
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
  return { store, chat, calls, invoke, notify, emit, events, commands: () => calls.map(([c]) => c) };
}

const q = <T extends Element = HTMLElement>(id: string) => wrapper!.element.querySelector(`[data-testid="${id}"]`) as T | null;
const all = (selector: string) => [...wrapper!.element.querySelectorAll(selector)] as HTMLElement[];
const textarea = () => wrapper!.element.querySelector('[data-testid="chat-input"] textarea') as HTMLTextAreaElement;
async function click(id: string) { q<HTMLButtonElement>(id)!.click(); await flushPromises(); }
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

beforeEach(() => { vi.resetAllMocks(); });
afterEach(() => { wrapper?.unmount(); wrapper = undefined; });

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

  it('only says it is a preview in the browser', async () => {
    const h = await open({}, { desktop: false });
    expect(q('chat-preview-only')).not.toBeNull();
    expect(h.invoke).not.toHaveBeenCalled();
    expect(textarea().disabled).toBe(true);
  });

  it('shows the working folder in the header and opens the model settings from it', async () => {
    const h = await open();
    expect(q('chat-workspace')!.textContent).toContain('小伴/工作区');
    expect(q('chat-workspace')!.getAttribute('title')).toContain('/Users/me/小伴/工作区');
    await click('chat-workspace');
    expect(h.store.settingsTab).toBe('ai');
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

  it('shows what the assistant did as cards that open to the details', async () => {
    const h = await open(startChat);
    await type('看看文件');
    await click('chat-send');
    await h.events('s1', { type: 'tool_start', id: 't1', name: 'shell', arguments: '{"command":"ls -la"}' });
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

  it('asks before deleting and then deletes', async () => {
    const h = await open({ ...listed, chat_delete_session: () => [summary('s2', '写周报', 1)] });
    await click('chat-delete-s1');
    expect(q('chat-delete-confirm')).not.toBeNull();
    expect(h.commands()).not.toContain('chat_delete_session');
    await click('chat-delete-yes-s1');
    expect(h.calls.at(-1)).toEqual(['chat_delete_session', { sessionId: 's1' }]);
    expect(all('.session-title').map(t => t.textContent)).toEqual(['写周报']);
  });

  it('marks a conversation that is waiting for the user, and one that is still answering', async () => {
    const h = await open({ chat_overview: () => overview({ sessions: [summary('s1', '整理文件'), summary('s2', '写周报')], running: ['s2'] }) });
    expect(q('chat-session-s2')!.querySelector('.spin')).not.toBeNull();
    expect(q<HTMLButtonElement>('chat-delete-s2')!.disabled).toBe(true);
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
    expect(h.notify).toHaveBeenCalledWith('一次最多发送 8 张图片，多余的没有加上', 'info');
    expect(q<HTMLButtonElement>('chat-pick-image')!.disabled).toBe(true);

    await type('还是有文字');
    await click('chat-send');
    expect((h.calls.find(([c]) => c === 'chat_send')![1]!.images as string[])).toHaveLength(8);
  });

  it('refuses a type or a size the host would reject, and says why', async () => {
    const h = await open({ ...startChat });
    await pickFiles([new File(['<svg/>'], 'icon.svg', { type: 'image/svg+xml' })]);
    expect(h.notify).toHaveBeenCalledWith('只能发送 PNG、JPEG、GIF 或 WebP 图片', 'error');
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
