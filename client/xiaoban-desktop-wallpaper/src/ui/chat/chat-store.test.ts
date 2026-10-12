import { describe, expect, it, vi } from 'vitest';
import { watchEffect } from 'vue';
import { createChatStore, type ChatDeps } from './chat-store.ts';
import type { ApprovalRequest, KernelEvent, Overview, QuestionRequest, SessionSummary, WireMessage } from './types.ts';

const summary = (id: string, title = '新对话', updatedAt = 1): SessionSummary => ({ id, title, createdAt: 1, updatedAt });
const overview = (patch: Partial<Overview> = {}): Overview => ({ configured: true, sessions: [], running: [], workspace: '/work', permissionMode: 'ask', hideToolCalls: false, ...patch });
const approval = (id: string, session = 's1'): ApprovalRequest => ({ id, session, tool: 'shell', risk: 'exec', arguments: { command: 'ls' }, tainted: false, canRemember: false });
const question = (id: string, session = 's1'): QuestionRequest => ({ id, session, question: '放哪里？', options: ['桌面', '文档'] });

type Responder = (args: Record<string, unknown> | undefined) => unknown;

function harness(responders: Record<string, Responder> = {}, desktop = true) {
  const handlers = new Map<string, (payload: never) => void>();
  const stopped: string[] = [];
  const calls: [string, Record<string, unknown> | undefined][] = [];
  const table: Record<string, Responder> = { chat_overview: () => overview(), ...responders };
  const deps: ChatDeps = {
    desktop,
    invoke: vi.fn(async (command: string, args?: Record<string, unknown>) => {
      calls.push([command, args]);
      const respond = table[command];
      if (!respond) throw new Error(`unexpected ${command}`);
      return respond(args) as never;
    }),
    listen: vi.fn(async (event: string, callback: (payload: never) => void) => { handlers.set(event, callback); return () => { stopped.push(event); }; }),
    notify: vi.fn(),
  };
  const store = createChatStore(deps);
  const emit = (event: string, payload: unknown) => handlers.get(event)!(payload as never);
  const events = (session: string, ...list: KernelEvent[]) => list.forEach(event => emit('chat-event', { session, event }));
  const commands = () => calls.map(([command]) => command);
  return { store, deps, emit, events, calls, commands, handlers, stopped, table };
}

describe('starting up', () => {
  it('listens once, reads the list and the saved settings', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1', '第一个')], workspace: '/Users/me/work', configured: true }) });
    await Promise.all([h.store.init(), h.store.init()]);
    await h.store.init();
    expect([...h.handlers.keys()].sort()).toEqual(['chat-approval', 'chat-event', 'chat-hide-tool-calls-changed', 'chat-permission-mode-changed', 'chat-question', 'chat-sessions-changed']);
    expect(h.deps.listen).toHaveBeenCalledTimes(6);
    expect(h.commands()).toEqual(['chat_overview']);
    expect(h.store).toMatchObject({ ready: true, configured: true, workspace: '/Users/me/work' });
    expect(h.store.sessions.map(s => s.title)).toEqual(['第一个']);
  });

  it('knows when no model is set up yet', async () => {
    const h = harness({ chat_overview: () => overview({ configured: false }) });
    await h.store.init();
    expect(h.store.configured).toBe(false);
  });

  it('is ready but idle in the browser preview', async () => {
    const h = harness({}, false);
    await h.store.init();
    expect(h.store.ready).toBe(true);
    expect(h.deps.listen).not.toHaveBeenCalled();
    expect(h.deps.invoke).not.toHaveBeenCalled();
    expect(await h.store.send('hi')).toBe(false);
    expect(h.deps.notify).toHaveBeenCalledWith('请在桌面应用中使用聊天', 'error');
  });

  it('keeps going and explains when the list cannot be read, and recovers on refresh', async () => {
    let fail = true;
    const h = harness({ chat_overview: () => { if (fail) throw new Error('磁盘不可读'); return overview({ sessions: [summary('s1')] }); } });
    await h.store.init();
    expect(h.store).toMatchObject({ ready: true, loadError: '磁盘不可读' });
    fail = false;
    await h.store.refresh();
    expect(h.store).toMatchObject({ loadError: '', sessions: [summary('s1')] });
  });

  it('shows an answer that was already running when the page started', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')], running: ['s1'] }) });
    await h.store.init();
    expect(h.store.views.s1.running).toBe(true);
  });

  it('shows nothing from an overview the host did not fill in', async () => {
    const h = harness({ chat_overview: () => undefined });
    await h.store.init();
    expect(h.store.ready).toBe(true);
    expect(h.store.sessions).toEqual([]);
  });
});

describe('sending', () => {
  it('starts a conversation on the first message and shows it at once', async () => {
    const h = harness({ chat_new_session: () => summary('s1'), chat_send: () => summary('s1', '整理下载文件夹', 5) });
    await h.store.init();
    expect(await h.store.send('整理下载文件夹')).toBe(true);
    expect(h.commands()).toEqual(['chat_overview', 'chat_new_session', 'chat_send']);
    expect(h.calls[2][1]).toEqual({ sessionId: 's1', text: '整理下载文件夹' });
    expect(h.store.currentId).toBe('s1');
    expect(h.store.sessions).toEqual([summary('s1', '整理下载文件夹', 5)]);
    expect(h.store.current()).toMatchObject({ running: true, loaded: true });
    expect(h.store.current()!.items).toMatchObject([{ kind: 'user', text: '整理下载文件夹' }]);
  });

  it('creates a new conversation in the mode the top bar shows, so it is not shown as trusted but run as asking', async () => {
    const trusted = { ...summary('s1'), permissionMode: 'auto' as const };
    const h = harness({
      chat_overview: () => overview({ sessions: [trusted], permissionMode: 'auto' }), chat_load_session: () => [],
      chat_set_permission_mode: () => 'auto', chat_new_session: () => ({ ...summary('s2'), permissionMode: 'auto' }), chat_send: () => summary('s2'),
    });
    await h.store.init();
    await h.store.select('s1');
    expect(h.store.permissionMode).toBe('auto');
    // "新对话" leaves the id empty and the bar keeps showing the mode of the conversation before it.
    await h.store.select(null);
    expect(h.store.permissionMode).toBe('auto');
    await h.store.send('接着做');
    const created = h.calls.find(([command]) => command === 'chat_new_session');
    expect(created?.[1]).toEqual({ permissionMode: 'auto' });

    // Choosing a mode before the first message of a draft counts as well.
    await h.store.select(null);
    await h.store.setPermissionMode('ask');
    await h.store.send('再来一个');
    const calls = h.calls.filter(([command]) => command === 'chat_new_session');
    expect(calls[calls.length - 1][1]).toEqual({ permissionMode: 'ask' });
  });

  it('adds to the open conversation without making a new one', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [], chat_send: () => summary('s1', 'x', 9) });
    await h.store.init();
    await h.store.select('s1');
    await h.store.send('第二句');
    expect(h.commands()).toEqual(['chat_overview', 'chat_load_session', 'chat_send']);
    expect(h.store.sessions[0].updatedAt).toBe(9);
  });

  it('follows an answer from its events to its end, and the list refreshes afterwards', async () => {
    const h = harness({ chat_new_session: () => summary('s1'), chat_send: () => summary('s1') });
    await h.store.init();
    await h.store.send('hi');
    h.events('s1', { type: 'text_delta', text: '你' }, { type: 'text_delta', text: '好' }, { type: 'done' });
    h.emit('chat-sessions-changed', [summary('s1', 'hi', 20)]);
    const view = h.store.current()!;
    expect(view.running).toBe(false);
    expect(view.items.map(i => i.kind === 'assistant' && i.text)).toEqual([false, '你好']);
    expect(h.store.sessions[0].title).toBe('hi');
  });

  it('copes with the first events arriving before the send call returns', async () => {
    let release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; });
    const h = harness({
      chat_new_session: () => summary('s1'),
      chat_send: async () => { await gate; return summary('s1'); },
    });
    await h.store.init();
    const sending = h.store.send('hi');
    await vi.waitFor(() => expect(h.store.currentId).toBe('s1'));
    h.events('s1', { type: 'text_delta', text: '很快' }, { type: 'done' });
    release();
    expect(await sending).toBe(true);
    const view = h.store.current()!;
    expect(view.running, 'a finished answer must not look like it is still running').toBe(false);
    expect(view.items.map(i => i.kind)).toEqual(['user', 'assistant']);
  });

  it('gives the text back and leaves no trace when the first message is refused', async () => {
    const h = harness({
      chat_new_session: () => summary('s1'),
      chat_send: () => { throw '还没有配置模型'; },
      chat_delete_session: () => [],
    });
    await h.store.init();
    expect(await h.store.send('hi')).toBe(false);
    expect(h.deps.notify).toHaveBeenCalledWith('还没有配置模型', 'error');
    await vi.waitFor(() => expect(h.commands()).toContain('chat_delete_session'));
    expect(h.store.currentId).toBeNull();
    expect(h.store.views.s1).toBeUndefined();
  });

  it('keeps an earlier conversation intact when a later message is refused', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [{ role: 'user', parts: [{ type: 'text', text: '之前' }] }], chat_send: () => { throw new Error('这个对话还在回答'); } });
    await h.store.init();
    await h.store.select('s1');
    expect(await h.store.send('再来')).toBe(false);
    const view = h.store.views.s1;
    expect(view.running).toBe(false);
    expect(view.items).toMatchObject([{ kind: 'user', text: '之前' }]);
    expect(h.commands()).not.toContain('chat_delete_session');
  });

  it('does not send while an answer is still coming, and does not make a conversation it cannot use', async () => {
    const h = harness({ chat_new_session: () => { throw new Error('对话最多保存 500 个'); } });
    await h.store.init();
    expect(await h.store.send('hi')).toBe(false);
    expect(h.deps.notify).toHaveBeenCalledWith(expect.objectContaining({ message: '对话最多保存 500 个' }), 'error');
    expect(h.store.currentId).toBeNull();

    const busy = harness({ chat_overview: () => overview({ sessions: [summary('s1')], running: ['s1'] }), chat_load_session: () => [] });
    await busy.store.init();
    await busy.store.select('s1');
    expect(await busy.store.send('again')).toBe(false);
    expect(busy.commands()).not.toContain('chat_send');
  });

  it('keeps answers to different conversations apart', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('a'), summary('b')], running: ['a', 'b'] }) });
    await h.store.init();
    h.events('a', { type: 'text_delta', text: '给 A' });
    h.events('b', { type: 'text_delta', text: '给 B' }, { type: 'done' });
    expect(h.store.views.a.running).toBe(true);
    expect(h.store.views.b.running).toBe(false);
    expect(JSON.stringify(h.store.views.a.items)).toContain('给 A');
    expect(JSON.stringify(h.store.views.a.items)).not.toContain('给 B');
  });
});

describe('opening and removing conversations', () => {
  const history: WireMessage[] = [{ role: 'user', parts: [{ type: 'text', text: '早' }] }, { role: 'assistant', parts: [{ type: 'text', text: '早上好' }] }];

  it('reads a conversation once and shows a new one on null', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => history });
    await h.store.init();
    await h.store.select('s1');
    await h.store.select(null);
    expect(h.store.current()).toBeNull();
    await h.store.select('s1');
    expect(h.commands().filter(c => c === 'chat_load_session')).toHaveLength(1);
    expect(h.store.current()!.items.map(i => i.kind)).toEqual(['user', 'assistant']);
  });

  it('does not reload over an answer that is being written', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')], running: ['s1'] }), chat_load_session: () => history });
    await h.store.init();
    h.events('s1', { type: 'text_delta', text: '写到一半' });
    await h.store.select('s1');
    expect(h.commands()).not.toContain('chat_load_session');
    expect(JSON.stringify(h.store.current()!.items)).toContain('写到一半');
  });

  it('says so when a conversation cannot be read', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => { throw new Error('文件损坏'); } });
    await h.store.init();
    await h.store.select('s1');
    expect(h.deps.notify).toHaveBeenCalled();
    expect(h.store.currentId).toBe('s1');
    expect(h.store.current()!.loaded).toBe(false);
  });

  it('removes a conversation and goes back to a new one, or keeps it when that fails', async () => {
    let fail = true;
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1'), summary('s2')] }),
      chat_load_session: () => history,
      chat_delete_session: (args) => { if (fail) throw new Error('这个对话还在回答'); return [summary('s2')].filter(s => s.id !== args?.sessionId); },
    });
    await h.store.init();
    await h.store.select('s1');
    await h.store.remove('s1');
    expect(h.store.sessions).toHaveLength(2);
    expect(h.store.currentId).toBe('s1');
    expect(h.deps.notify).toHaveBeenCalled();
    fail = false;
    await h.store.remove('s1');
    expect(h.store.sessions.map(s => s.id)).toEqual(['s2']);
    expect(h.store.views.s1).toBeUndefined();
    expect(h.store.currentId).toBeNull();
  });

  it('stops the open conversation', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [], chat_cancel: () => undefined });
    await h.store.init();
    await h.store.cancel();
    expect(h.commands()).not.toContain('chat_cancel');
    await h.store.select('s1');
    await h.store.cancel();
    expect(h.calls.at(-1)).toEqual(['chat_cancel', { sessionId: 's1' }]);
  });
});

describe('confirmations and questions', () => {
  it('collects them per conversation, once each, and counts them for the badge', async () => {
    const h = harness();
    await h.store.init();
    h.emit('chat-approval', approval('a1', 's1'));
    h.emit('chat-approval', approval('a1', 's1'));
    h.emit('chat-question', question('q1', 's2'));
    expect(h.store.views.s1.approvals).toHaveLength(1);
    expect(h.store.views.s2.questions).toHaveLength(1);
    expect(h.store.attention).toBe(2);
  });

  it('removes a card as soon as it is answered and tells the host', async () => {
    const h = harness({ chat_answer_approval: () => undefined, chat_answer_question: () => undefined });
    await h.store.init();
    const a = approval('a1');
    const q = question('q1');
    h.emit('chat-approval', a);
    h.emit('chat-question', q);
    const answering = h.store.answerApproval(a, 'once');
    expect(h.store.views.s1.approvals).toEqual([]);
    await answering;
    await h.store.answerQuestion(q, '桌面');
    expect(h.calls.slice(-2)).toEqual([['chat_answer_approval', { id: 'a1', decision: 'once' }], ['chat_answer_question', { id: 'q1', answer: '桌面' }]]);
    expect(h.store.attention).toBe(0);
    expect(h.deps.notify).not.toHaveBeenCalled();
  });

  it('is quiet about an answer that came too late, but reports other failures', async () => {
    let message = '这个确认已经结束';
    const h = harness({ chat_answer_approval: () => { throw message; } });
    await h.store.init();
    h.emit('chat-approval', approval('a1'));
    await h.store.answerApproval(approval('a1'), 'deny');
    expect(h.deps.notify).not.toHaveBeenCalled();
    message = '连接断开';
    await h.store.answerApproval(approval('a2'), 'deny');
    expect(h.deps.notify).toHaveBeenCalledWith('连接断开', 'error');
  });

  it('drops them when the answer ends, however it ends', async () => {
    const h = harness();
    await h.store.init();
    h.emit('chat-approval', approval('a1'));
    h.emit('chat-question', question('q1'));
    h.events('s1', { type: 'cancelled' });
    expect(h.store.attention).toBe(0);
  });
});

describe('shutting down', () => {
  it('stops listening and ignores events that still arrive', async () => {
    const h = harness();
    await h.store.init();
    h.store.dispose();
    expect(h.stopped.sort()).toEqual(['chat-approval', 'chat-event', 'chat-hide-tool-calls-changed', 'chat-permission-mode-changed', 'chat-question', 'chat-sessions-changed']);
    h.events('s1', { type: 'text_delta', text: 'late' });
    h.emit('chat-approval', approval('a1'));
    expect(h.store.views.s1).toBeUndefined();
  });

  it('lets go of listeners that were still being set up', async () => {
    const h = harness();
    const starting = h.store.init();
    h.store.dispose();
    await starting;
    expect(h.stopped.sort()).toEqual(['chat-approval', 'chat-event', 'chat-hide-tool-calls-changed', 'chat-permission-mode-changed', 'chat-question', 'chat-sessions-changed']);
    expect(h.store.sessions).toEqual([]);
  });
});

describe('what the page can see', () => {
  /** Counts how often something that reads the view runs again: it must run again when the view changes. */
  function watched(read: () => unknown) {
    const seen: unknown[] = [];
    const stop = watchEffect(() => { seen.push(read()); }, { flush: 'sync' });
    return { seen, stop };
  }

  it('updates when a conversation opened from disk finishes loading', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [{ role: 'user', parts: [{ type: 'text', text: '早' }] }] });
    await h.store.init();
    const w = watched(() => h.store.views.s1?.items.length ?? -1);
    await h.store.select('s1');
    expect(w.seen.at(-1)).toBe(1);
    w.stop();
  });

  it('updates for the very first event of a conversation it had not seen', async () => {
    const h = harness();
    await h.store.init();
    const w = watched(() => JSON.stringify(h.store.views.live?.items ?? null));
    h.events('live', { type: 'text_delta', text: '第一个字' });
    expect(w.seen.at(-1)).toContain('第一个字');
    h.events('live', { type: 'text_delta', text: '第二个' });
    expect(w.seen.at(-1)).toContain('第一个字第二个');
    w.stop();
  });

  it('updates for confirmations and for the running state', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')], running: ['s1'] }) });
    await h.store.init();
    const w = watched(() => `${h.store.attention}|${h.store.views.s1.running}`);
    h.emit('chat-approval', approval('a1'));
    expect(w.seen.at(-1)).toBe('1|true');
    h.events('s1', { type: 'done' });
    expect(w.seen.at(-1)).toBe('0|false');
    w.stop();
  });
});
