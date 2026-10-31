import { describe, expect, it, vi } from 'vitest';
import { watchEffect } from 'vue';
import { replyBlocks } from './replies.ts';
import { taskProgress } from './task-progress.ts';
import { createChatStore, type ChatDeps } from './chat-store.ts';
import type { ApprovalRequest, KernelEvent, Overview, QuestionRequest, SessionSummary, WireMessage } from './types.ts';

const summary = (id: string, title = '新对话', updatedAt = 1): SessionSummary => ({ id, title, createdAt: 1, updatedAt });
const overview = (patch: Partial<Overview> = {}): Overview => ({ configured: true, sessions: [], running: [], workspace: '/work', permissionMode: 'ask', hideToolCalls: false, ...patch });
const approval = (id: string, session = 's1'): ApprovalRequest => ({ id, session, tool: 'shell', risk: 'exec', arguments: { command: 'ls' }, tainted: false, canRemember: false });
const question = (id: string, session = 's1'): QuestionRequest => ({ id, session, question: '放哪里？', options: ['桌面', '文档'] });

type Responder = (args: Record<string, unknown> | undefined) => unknown;

it('does not turn ordinary unverified execution outcomes into warnings', async () => {
  const h = harness({chat_overview:()=>overview({sessions:[summary('s1')]}),chat_load_session:()=>[]});
  await h.store.init(); await h.store.select('s1');
  h.events('s1',{type:'tool_start',id:'write',name:'write_file',arguments:'{}'});
  h.events('s1',{type:'tool_end',id:'write',output:'saved',is_error:false});
  h.events('s1',{type:'done'});
  for (const status of ['unknown','succeeded']) h.emit('chat-task-outcome',{session:'s1',outcome:{runId:'r1',status,validator:'run_trace_only',evidence:'terminal=true; tool_errors=0'}});
  expect(h.store.current()?.items.filter(i=>i.kind==='notice')).toEqual([expect.objectContaining({text:'',tone:'info',detailsLabel:'执行详情'})]);
  h.store.dispose();
});

it('persists a workflow expert through sending, continuing and reopening the conversation',async()=>{
  const bound={plugin:'',agent:'',workflow:'health-report',name:'健康报告',profession:''};
  const session={...summary('workflow-session','报告'),expert:bound};
  const h=harness({chat_new_session:()=>session,chat_send:()=>session,chat_history:()=>[]});
  await h.store.init();h.store.pendingExpert={plugin:'',workflow:'health-report',name:'健康报告',profession:''};
  expect(await h.store.send('解读报告')).toBe(true);
  expect(h.calls.find(([c])=>c==='chat_new_session')?.[1]).toMatchObject({expert:{workflow:'health-report'}});
  expect(h.store.currentExpert).toMatchObject({workflow:'health-report',name:'健康报告',hasAvatar:false});
  h.events('workflow-session',{type:'done'});expect(await h.store.send('继续解释')).toBe(true);
  expect(h.calls.filter(([c])=>c==='chat_new_session')).toHaveLength(1);
  h.events('workflow-session',{type:'done'});await h.store.select(null);await h.store.select('workflow-session');
  expect(h.store.currentExpert?.workflow).toBe('health-report');h.store.dispose();
});

function harness(responders: Record<string, Responder> = {}, desktop = true) {
  const handlers = new Map<string, (payload: never) => void>();
  const stopped: string[] = [];
  const calls: [string, Record<string, unknown> | undefined][] = [];
  const table: Record<string, Responder> = { chat_overview: () => overview(), chat_context: () => ({ used: 0, limit: 256_000, canCompact: false }), ...responders };
  // The ring's numbers are read on their own schedule, so they are kept apart from the calls a test is about.
  const contextCalls: (Record<string, unknown> | undefined)[] = [];
  const deps: ChatDeps = {
    desktop,
    invoke: vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === 'chat_context') contextCalls.push(args); else calls.push([command, args]);
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
  return { store, deps, emit, events, calls, contextCalls, commands, handlers, stopped, table };
}

describe('revising a running task plan', () => {
  it('waits for cancellation to finish before sending the revision and keeps the original authorization mode', async () => {
    const h = harness({ chat_new_session: () => summary('s1'), chat_send: () => summary('s1'), chat_cancel: () => undefined });
    await h.store.init(); await h.store.send('制作汇报');
    const revised = h.store.revisePlan('改为面向管理层');
    await Promise.resolve(); await Promise.resolve();
    expect(h.calls.filter(([c]) => c === 'chat_send')).toHaveLength(1);
    h.events('s1', { type: 'cancelled' });
    expect(await revised).toBe(true);
    expect(h.calls.filter(([c]) => c === 'chat_send').at(-1)?.[1]?.text).toContain('改为面向管理层');
    expect(h.calls.filter(([c]) => c === 'chat_send').at(-1)?.[1]?.text).toContain('原有授权范围不变');
    expect(h.store.permissionMode).toBe('ask'); h.store.dispose();
  });

  it('retains the instruction on cancellation failure and never sends it to a different conversation', async () => {
    const h = harness({ chat_new_session: () => summary('s1'), chat_send: () => summary('s1'), chat_cancel: () => { throw new Error('无法停止'); } });
    await h.store.init(); await h.store.send('制作汇报');
    expect(await h.store.revisePlan('新要求')).toBe(false);
    expect(h.calls.filter(([c]) => c === 'chat_send')).toHaveLength(1);
    h.table.chat_cancel = () => { void h.store.select(null); h.events('s1', { type: 'cancelled' }); };
    expect(await h.store.revisePlan('仍属于 s1 的要求')).toBe(false);
    expect(h.calls.filter(([c]) => c === 'chat_send')).toHaveLength(1); h.store.dispose();
  });

  it('does not send the revision when native work never acknowledges cancellation', async () => {
    vi.useFakeTimers();
    const h = harness({ chat_new_session: () => summary('s1'), chat_send: () => summary('s1'), chat_cancel: () => undefined });
    try {
      await h.store.init(); await h.store.send('制作汇报');
      const revised = h.store.revisePlan('改计划');
      await vi.advanceTimersByTimeAsync(5001);
      expect(await revised).toBe(false);
      expect(h.calls.filter(([c]) => c === 'chat_send')).toHaveLength(1);
      expect(h.store.current()?.running).toBe(true);
    } finally { h.store.dispose(); vi.useRealTimers(); }
  });
});

describe('an answer cut off by a crash', () => {
  it('says so after the history loads, naming what may already have run', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1')] }),
      chat_load_session: () => { h.emit('chat-run-interrupted', { session: 's1', runId: 'r1', tools: ['shell', 'write_file', 'shell'] }); return [{ role: 'user', parts: [{ type: 'text', text: '整理文件' }] }]; },
    });
    await h.store.init();
    await h.store.select('s1');
    const items = h.store.views.s1.items;
    expect(items.map(i => i.kind)).toEqual(['user', 'notice']);
    const notice = items[1];
    expect(notice.kind === 'notice' && notice.text).toContain('可能已经执行：运行命令、写入文件');
    expect(notice.kind === 'notice' && notice.canContinue).toBe(true);
  });
});

describe('starting up', () => {
  it('listens once, reads the list and the saved settings', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1', '第一个')], workspace: '/Users/me/work', configured: true }) });
    await Promise.all([h.store.init(), h.store.init()]);
    await h.store.init();
    expect([...h.handlers.keys()].sort()).toEqual(['chat-approval', 'chat-checkpoint', 'chat-compacting', 'chat-event', 'chat-hide-tool-calls-changed', 'chat-pending-changed', 'chat-permission-mode-changed', 'chat-question', 'chat-queue-changed', 'chat-run-interrupted', 'chat-sessions-changed', 'chat-task-outcome']);
    expect(h.deps.listen).toHaveBeenCalledTimes(12);
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

  it('loads history and preserves events for an answer that is being written', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')], running: ['s1'] }), chat_load_session: () => history });
    await h.store.init();
    h.events('s1', { type: 'text_delta', text: '写到一半' });
    await h.store.select('s1');
    expect(h.commands()).toContain('chat_load_session');
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

  describe('names and groups', () => {
    const work = { id: 'g1', name: '工作' };
    const listing = (sessions: SessionSummary[], groups = [work]) => ({ sessions, groups });

    it('reads the groups with the first overview', async () => {
      const h = harness({ chat_overview: () => ({ ...overview({ sessions: [{ ...summary('s1'), group: 'g1' }] }), groups: [work] }) });
      await h.store.init();
      expect(h.store.groups).toEqual([work]);
      expect(h.store.sessions[0].group).toBe('g1');
      const old = harness({ chat_overview: () => overview() });
      await old.store.init();
      expect(old.store.groups).toEqual([]);
    });

    it('renames a conversation and takes the list the host sends back', async () => {
      const h = harness({ chat_overview: () => overview({ sessions: [summary('s1', '旧名字')] }), chat_rename_session: () => listing([summary('s1', '新名字')], []) });
      await h.store.init();
      expect(await h.store.rename('s1', '新名字')).toBe(true);
      expect(h.calls.at(-1)).toEqual(['chat_rename_session', { sessionId: 's1', title: '新名字' }]);
      expect(h.store.sessions[0].title).toBe('新名字');
    });

    it('moves a conversation into a group and back out with a null group', async () => {
      const h = harness({
        chat_overview: () => overview({ sessions: [summary('s1')] }),
        chat_move_session: args => listing([{ ...summary('s1'), group: (args?.groupId as string | null) ?? '' }]),
      });
      await h.store.init();
      expect(await h.store.move('s1', 'g1')).toBe(true);
      expect(h.calls.at(-1)).toEqual(['chat_move_session', { sessionId: 's1', groupId: 'g1' }]);
      expect(h.store.sessions[0].group).toBe('g1');
      expect(await h.store.move('s1', null)).toBe(true);
      expect(h.calls.at(-1)).toEqual(['chat_move_session', { sessionId: 's1', groupId: null }]);
      expect(h.store.sessions[0].group).toBe('');
    });

    it('creates a group, optionally moving a conversation into it in the same call', async () => {
      const h = harness({
        chat_overview: () => overview({ sessions: [summary('s1')] }),
        chat_create_group: args => listing([{ ...summary('s1'), group: args?.sessionId ? 'g1' : '' }]),
      });
      await h.store.init();
      expect(await h.store.createGroup('工作')).toBe(true);
      expect(h.calls.at(-1)).toEqual(['chat_create_group', { name: '工作' }]);
      expect(h.store.groups).toEqual([work]);
      expect(h.store.sessions[0].group).toBe('');
      await h.store.createGroup('工作', 's1');
      expect(h.calls.at(-1)).toEqual(['chat_create_group', { name: '工作', sessionId: 's1' }]);
      expect(h.store.sessions[0].group).toBe('g1');
    });

    it('renames and deletes a group; deleting hands back the conversations that were inside it', async () => {
      const h = harness({
        chat_overview: () => ({ ...overview({ sessions: [{ ...summary('s1'), group: 'g1' }] }), groups: [work] }),
        chat_rename_group: () => listing([{ ...summary('s1'), group: 'g1' }], [{ id: 'g1', name: '项目' }]),
        chat_delete_group: () => listing([summary('s1')], []),
      });
      await h.store.init();
      expect(await h.store.renameGroup('g1', '项目')).toBe(true);
      expect(h.calls.at(-1)).toEqual(['chat_rename_group', { groupId: 'g1', name: '项目' }]);
      expect(h.store.groups[0].name).toBe('项目');
      expect(await h.store.removeGroup('g1')).toBe(true);
      expect(h.calls.at(-1)).toEqual(['chat_delete_group', { groupId: 'g1' }]);
      expect(h.store.groups).toEqual([]);
      expect(h.store.sessions.map(s => s.id)).toEqual(['s1']);
    });

    it('reports the host\'s reason and changes nothing when an edit is refused', async () => {
      const h = harness({
        chat_overview: () => ({ ...overview({ sessions: [summary('s1', '原名')] }), groups: [work] }),
        chat_rename_session: () => { throw new Error('对话名称不能为空'); },
        chat_create_group: () => { throw new Error('已经有同名的分组'); },
      });
      await h.store.init();
      expect(await h.store.rename('s1', ' ')).toBe(false);
      expect(await h.store.createGroup('工作')).toBe(false);
      expect(h.deps.notify).toHaveBeenCalledTimes(2);
      expect(h.deps.notify).toHaveBeenNthCalledWith(1, expect.objectContaining({ message: '对话名称不能为空' }), 'error');
      expect(h.store.sessions[0].title).toBe('原名');
      expect(h.store.groups).toEqual([work]);
    });

    it('does nothing in the browser preview', async () => {
      const h = harness({}, false);
      await h.store.init();
      expect(await h.store.rename('s1', 'x')).toBe(false);
      expect(await h.store.createGroup('x')).toBe(false);
      expect(h.deps.invoke).not.toHaveBeenCalled();
    });
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

  it('puts a card back when the answer could not be delivered, so the run is not left waiting', async () => {
    const h = harness({ chat_answer_approval: () => { throw '连接断开'; }, chat_answer_question: () => { throw '连接断开'; } });
    await h.store.init();
    h.emit('chat-approval', approval('a1'));
    h.emit('chat-question', question('q1'));
    await h.store.answerApproval(approval('a1'), 'once');
    await h.store.answerQuestion(question('q1'), '桌面');
    expect(h.store.views.s1.approvals.map(a => a.id)).toEqual(['a1']);
    expect(h.store.views.s1.questions.map(q => q.id)).toEqual(['q1']);
    expect(h.store.attention).toBe(2);
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
    expect(h.stopped.sort()).toEqual(['chat-approval', 'chat-checkpoint', 'chat-compacting', 'chat-event', 'chat-hide-tool-calls-changed', 'chat-pending-changed', 'chat-permission-mode-changed', 'chat-question', 'chat-queue-changed', 'chat-run-interrupted', 'chat-sessions-changed', 'chat-task-outcome']);
    h.events('s1', { type: 'text_delta', text: 'late' });
    h.emit('chat-approval', approval('a1'));
    expect(h.store.views.s1).toBeUndefined();
  });

  it('lets go of listeners that were still being set up', async () => {
    const h = harness();
    const starting = h.store.init();
    h.store.dispose();
    await starting;
    expect(h.stopped.sort()).toEqual(['chat-approval', 'chat-checkpoint', 'chat-compacting', 'chat-event', 'chat-hide-tool-calls-changed', 'chat-pending-changed', 'chat-permission-mode-changed', 'chat-question', 'chat-queue-changed', 'chat-run-interrupted', 'chat-sessions-changed', 'chat-task-outcome']);
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

describe('the context of the open conversation and compacting it', () => {
  const info = (used: number, canCompact = true) => ({ used, limit: 256_000, canCompact });
  const say = (value: string) => [{ type: 'text' as const, text: value }];
  const history: WireMessage[] = [{ role: 'user', parts: say('早') }, { role: 'assistant', parts: say('早上好') }];
  const withSummary: WireMessage[] = [...history, { role: 'user', synthetic: true, context_summary: { keep_from: 2, text: '摘要正文' } }];
  const tick = () => new Promise(resolve => setTimeout(resolve, 0));

  it('reads the numbers with the first overview, and again when a conversation is opened', async () => {
    let used = 1_000;
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => history, chat_context: () => info(used) });
    await h.store.init();
    await vi.waitFor(() => expect(h.store.context.used).toBe(1_000));
    expect(h.contextCalls[0]).toEqual({ sessionId: null });

    used = 7_000;
    await h.store.select('s1');
    await vi.waitFor(() => expect(h.store.context.used).toBe(7_000));
    expect(h.contextCalls.at(-1)).toEqual({ sessionId: 's1' });
    expect(h.commands()).not.toContain('chat_context');
  });

  it('ignores a late answer about a conversation that is no longer open', async () => {
    let release!: (value: unknown) => void;
    const slow = new Promise(resolve => { release = resolve; });
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1'), summary('s2')] }),
      chat_load_session: () => history,
      chat_context: args => (args?.sessionId === 's1' ? slow : info(500)),
    });
    await h.store.init();
    await h.store.select('s1');
    await h.store.select('s2');
    await vi.waitFor(() => expect(h.store.context.used).toBe(500));
    release(info(9_000));
    await tick();
    expect(h.store.context.used).toBe(500);
  });

  it('reads the numbers again when the open conversation finishes, is stopped or fails, but not for others', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1'), summary('s2')] }), chat_load_session: () => history });
    await h.store.init();
    await h.store.select('s1');
    await tick();
    for (const event of [{ type: 'done' }, { type: 'cancelled' }, { type: 'error', message: '坏了' }] as KernelEvent[]) {
      const before = h.contextCalls.length;
      h.events('s1', event);
      await vi.waitFor(() => expect(h.contextCalls.length).toBe(before + 1));
    }
    const before = h.contextCalls.length;
    h.events('s2', { type: 'done' });
    await tick();
    expect(h.contextCalls.length).toBe(before);
  });

  it('keeps the last numbers when reading them fails', async () => {
    let fail = false;
    const h = harness({ chat_overview: () => overview(), chat_context: () => { if (fail) throw new Error('读取失败'); return info(3_000); } });
    await h.store.init();
    await vi.waitFor(() => expect(h.store.context.used).toBe(3_000));
    fail = true;
    await h.store.refreshContext();
    expect(h.store.context.used).toBe(3_000);
    expect(h.deps.notify).not.toHaveBeenCalled();
  });

  it('compacts, takes the new numbers and shows the summary where it now sits', async () => {
    let compacted = false;
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1')] }),
      chat_load_session: () => (compacted ? withSummary : history),
      chat_context: () => info(compacted ? 2_000 : 40_000),
      chat_compact: () => { compacted = true; return info(2_000, false); },
    });
    await h.store.init();
    await h.store.select('s1');
    await vi.waitFor(() => expect(h.store.context.used).toBe(40_000));

    const pending = h.store.compact();
    expect(h.store.compactingId).toBe('s1');
    expect(await pending).toBe(true);
    expect(h.calls.at(-2)).toEqual(['chat_compact', { sessionId: 's1' }]);
    expect(h.calls.at(-1)).toEqual(['chat_load_session', { sessionId: 's1' }]);
    expect(h.store.compactingId).toBeNull();
    expect(h.store.context).toEqual(info(2_000, false));
    expect(h.store.current()!.items.map(item => item.kind)).toEqual(['user', 'assistant', 'notice']);
    expect(h.deps.notify).toHaveBeenCalledWith('较早的对话已压缩成摘要', 'success');
  });

  it('still counts as done when only reloading the conversation afterwards fails', async () => {
    let loads = 0;
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1')] }),
      chat_load_session: () => { if (++loads > 1) throw new Error('读不出来'); return history; },
      chat_compact: () => info(2_000, false),
    });
    await h.store.init();
    await h.store.select('s1');
    expect(await h.store.compact()).toBe(true);
    expect(h.store.context.used).toBe(2_000);
    expect(h.deps.notify).not.toHaveBeenCalledWith(expect.anything(), 'error');
  });

  it('says why when the host refuses, and changes nothing', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1')] }),
      chat_load_session: () => history,
      chat_context: () => info(40_000),
      chat_compact: () => { throw new Error('摘要没有比原文短多少，已保留原对话'); },
    });
    await h.store.init();
    await h.store.select('s1');
    await vi.waitFor(() => expect(h.store.context.used).toBe(40_000));
    expect(await h.store.compact()).toBe(false);
    expect(h.deps.notify).toHaveBeenCalledWith(expect.objectContaining({ message: '摘要没有比原文短多少，已保留原对话' }), 'error');
    expect(h.store.compactingId).toBeNull();
    expect(h.store.context.used).toBe(40_000);
    expect(h.calls.filter(([command]) => command === 'chat_load_session')).toHaveLength(1);
  });

  it('does nothing with no open conversation, while it answers, or while another compaction is under way', async () => {
    let release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; });
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1'), summary('s2')], running: ['s2'] }),
      chat_load_session: () => history,
      chat_compact: async () => { await gate; return info(1_000, false); },
    });
    await h.store.init();
    expect(await h.store.compact()).toBe(false);

    await h.store.select('s2');
    expect(await h.store.compact()).toBe(false);

    await h.store.select('s1');
    const first = h.store.compact();
    expect(await h.store.compact()).toBe(false);
    release();
    expect(await first).toBe(true);
    expect(h.calls.filter(([command]) => command === 'chat_compact')).toHaveLength(1);
  });

  it('is idle in the browser preview', async () => {
    const h = harness({}, false);
    await h.store.init();
    await h.store.refreshContext();
    expect(await h.store.compact()).toBe(false);
    expect(h.deps.invoke).not.toHaveBeenCalled();
  });
});

describe('the context limit and the host compacting by itself', () => {
  const info = (used: number, limit = 256_000, canCompact = true) => ({ used, limit, canCompact });
  const say = (value: string) => [{ type: 'text' as const, text: value }];
  const history: WireMessage[] = [{ role: 'user', parts: say('早') }, { role: 'assistant', parts: say('早上好') }];
  const tick = () => new Promise(resolve => setTimeout(resolve, 0));

  it('shows a new limit at once, saves it, and then reads the numbers back from the host', async () => {
    let limit = 256_000;
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [], chat_context: () => info(100_000, limit), chat_set_session_options: args => { limit = Number(args?.contextTokens); return undefined; } });
    await h.store.init();
    await h.store.select('s1');
    await vi.waitFor(() => expect(h.store.context.limit).toBe(256_000));

    const pending = h.store.setContextSize(512_000);
    expect(h.store.context.limit).toBe(512_000);
    expect(await pending).toBe(true);
    expect(h.calls.at(-1)).toEqual(['chat_set_session_options', { sessionId: 's1', contextTokens: 512_000 }]);
    expect(h.store.context).toEqual(info(100_000, 512_000));
  });

  it('puts the old limit back and says why when the host refuses', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [], chat_context: () => info(100_000), chat_set_session_options: () => { throw new Error('无法保存上下文大小：磁盘已满'); } });
    await h.store.init();
    await h.store.select('s1');
    await vi.waitFor(() => expect(h.store.context.used).toBe(100_000));
    expect(await h.store.setContextSize(900_000)).toBe(false);
    expect(h.store.context.limit).toBe(256_000);
    expect(h.deps.notify).toHaveBeenCalledWith(expect.objectContaining({ message: '无法保存上下文大小：磁盘已满' }), 'error');
  });

  it('does nothing about the limit in the browser preview', async () => {
    const h = harness({}, false);
    await h.store.init();
    expect(await h.store.setContextSize(512_000)).toBe(false);
    expect(h.deps.invoke).not.toHaveBeenCalled();
  });

  it('marks a conversation as being compacted while the host does it, then shows the summary there', async () => {
    let used = 240_000;
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => history, chat_context: () => info(used) });
    await h.store.init();
    await h.store.select('s1');
    await vi.waitFor(() => expect(h.store.context.used).toBe(240_000));
    const before = h.store.current()!.items.length;

    h.emit('chat-compacting', { session: 's1', state: 'start' });
    expect(h.store.compactingId).toBe('s1');

    used = 30_000;
    h.emit('chat-compacting', { session: 's1', state: 'done', summary: '【用户目标】整理文件' });
    expect(h.store.compactingId).toBeNull();
    const items = h.store.current()!.items;
    expect(items).toHaveLength(before + 1);
    const notice = items.at(-1) as Extract<(typeof items)[number], { kind: 'notice' }>;
    expect([notice.kind, notice.detailsLabel, notice.details]).toEqual(['notice', '查看摘要', '【用户目标】整理文件']);
    await vi.waitFor(() => expect(h.store.context.used).toBe(30_000));
  });

  it('says so, once, when the host could not compact', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => history });
    await h.store.init();
    await h.store.select('s1');
    h.emit('chat-compacting', { session: 's1', state: 'start' });
    h.emit('chat-compacting', { session: 's1', state: 'failed', message: '额度已用完，请兑换额度后继续使用' });
    expect(h.store.compactingId).toBeNull();
    expect(h.deps.notify).toHaveBeenCalledTimes(1);
    expect(h.deps.notify).toHaveBeenCalledWith(expect.stringContaining('额度已用完'), 'info');
    expect(h.store.current()!.items.some(item => item.kind === 'notice')).toBe(false);
  });

  it('leaves the open conversation alone when another one is compacted, and a manual compaction of its own still works', async () => {
    let compacted = false;
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1'), summary('s2')] }),
      chat_load_session: () => history,
      chat_compact: () => { compacted = true; return info(1_000, 256_000, false); },
    });
    await h.store.init();
    await h.store.select('s1');
    h.emit('chat-compacting', { session: 's2', state: 'start' });
    expect(h.store.compactingId).toBe('s2');
    expect(await h.store.compact()).toBe(true);
    expect(compacted).toBe(true);

    const calls = h.contextCalls.length;
    h.emit('chat-compacting', { session: 's2', state: 'done', summary: '别的对话' });
    await tick();
    expect(h.contextCalls.length).toBe(calls);
    expect(h.store.current()!.items.some(item => item.kind === 'notice' && item.details === '别的对话')).toBe(false);
  });
});


it('sends a named document with an image and retains the attachment card', async () => {
  const h = harness({chat_new_session:()=>summary('s1'),chat_send:()=>summary('s1')});
  await h.store.init();
  const doc = {name:'预算.csv',size:9,mime:'text/csv',dataUrl:'data:text/csv;base64,YW1vdW50CjEy'};
  expect(await h.store.send('',[doc,'data:image/png;base64,AA=='])).toBe(true);
  expect(h.calls.find(([name])=>name==='chat_send')?.[1]).toEqual({sessionId:'s1',text:'',images:['data:image/png;base64,AA=='],attachments:[{name:doc.name,dataUrl:doc.dataUrl}]});
  expect(h.store.current()?.items[0]).toMatchObject({kind:'user',files:[{name:'预算.csv',size:9}]});
});

describe('undoing an answer\'s file changes', () => {
  it('offers undo after an answer changed files, puts them back once, and reports what could not be restored', async () => {
    let report = { restored: ['notes.md'], removed: ['new.md'], failed: [] as [string, string][] };
    const h = harness({ chat_undo_run: () => report });
    await h.store.init();
    h.emit('chat-checkpoint', { session: 's1', runId: 'r1', files: ['notes.md', 'new.md'] });
    const notice = h.store.views.s1.items.at(-1)!;
    expect(notice).toMatchObject({ kind: 'notice', undoRun: 'r1', details: 'notes.md\nnew.md' });
    expect(await h.store.undoRun('s1', notice.id)).toBe(true);
    expect(h.calls.at(-1)).toEqual(['chat_undo_run', { sessionId: 's1', runId: 'r1' }]);
    expect(notice).toMatchObject({ undoRun: undefined, text: '这轮回答的 2 个文件改动已撤销。' });
    expect(await h.store.undoRun('s1', notice.id), 'undone once').toBe(false);

    report = { restored: [], removed: [], failed: [['big.bin', '文件太大，当时没有备份']] };
    h.emit('chat-checkpoint', { session: 's1', runId: 'r2', files: ['big.bin'] });
    const second = h.store.views.s1.items.at(-1)!;
    await h.store.undoRun('s1', second.id);
    expect(second).toMatchObject({ tone: 'warning', details: 'big.bin：文件太大，当时没有备份' });
  });

  it('ignores an answer that changed nothing and keeps the button when the host refuses', async () => {
    const h = harness({ chat_undo_run: () => { throw '这个对话还在回答'; } });
    await h.store.init();
    h.emit('chat-checkpoint', { session: 's1', runId: 'r0', files: [] });
    expect(h.store.views.s1?.items ?? []).toHaveLength(0);
    h.emit('chat-checkpoint', { session: 's1', runId: 'r1', files: ['a.txt'] });
    const notice = h.store.views.s1.items.at(-1)!;
    expect(await h.store.undoRun('s1', notice.id)).toBe(false);
    expect(notice).toMatchObject({ undoRun: 'r1' });
    expect(h.deps.notify).toHaveBeenCalledWith('这个对话还在回答', 'error');
  });
});

describe('forking a conversation from one message', () => {
  const said = (role: 'user' | 'assistant', value: string, extra: Record<string, unknown> = {}): WireMessage => ({ role, parts: [{ type: 'text', text: value }], ...extra });
  const history = (): WireMessage[] => [said('user', '一'), said('assistant', '答一'), said('user', '二', { parts: [{ type: 'text', text: '二' }, { type: 'image', url: 'data:image/png;base64,AA==' }] }), said('assistant', '答二')];
  async function opened(responders: Record<string, Responder> = {}) {
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1', '原来的')] }),
      chat_load_session: args => (args?.sessionId === 's1' ? history() : history().slice(0, 2)),
      chat_fork_session: () => summary('s2', '原来的（分支）'),
      ...responders,
    });
    await h.store.init(); await h.store.select('s1');
    return h;
  }

  it('copies up to just before the edited message, opens the copy and leaves the text and pictures to be sent again', async () => {
    const h = await opened();
    const second = h.store.current()!.items.filter(i => i.kind === 'user')[1];
    expect(await h.store.fork('s1', second.id, true)).toBe('s2');
    expect(h.calls.find(([c]) => c === 'chat_fork_session')?.[1]).toEqual({ sessionId: 's1', beforeTurn: 1 });
    expect(h.store.currentId).toBe('s2');
    expect(h.store.sessions.map(s => s.id)).toEqual(['s2', 's1']);
    expect(h.store.draftFor('s2')).toMatchObject({ text: '二', attachments: ['data:image/png;base64,AA=='] });
    expect(h.store.views.s1.items, 'the original is untouched').toHaveLength(4);
    h.store.dispose();
  });

  it('copies through the end of a reply when asked to continue from it, with nothing put in the input box', async () => {
    const h = await opened();
    const replies = h.store.current()!.items.filter(i => i.kind === 'assistant');
    expect(await h.store.fork('s1', replies[0].id, false)).toBe('s2');
    expect(h.calls.find(([c]) => c === 'chat_fork_session')?.[1]).toEqual({ sessionId: 's1', beforeTurn: 1 });
    expect(h.store.draftFor('s2').text).toBe('');
    await h.store.select('s1');
    await h.store.fork('s1', replies[1].id, false);
    expect(h.calls.filter(([c]) => c === 'chat_fork_session').at(-1)?.[1]).toEqual({ sessionId: 's1', beforeTurn: 2 });
    h.store.dispose();
  });

  it('does nothing while the conversation is answering or when the item is not a message to edit', async () => {
    const h = await opened();
    const [first] = h.store.current()!.items;
    const reply = h.store.current()!.items.find(i => i.kind === 'assistant')!;
    expect(await h.store.fork('s1', reply.id, true), 'only a message of the user can be edited').toBeNull();
    expect(await h.store.fork('s1', 'missing', false)).toBeNull();
    h.store.current()!.running = true;
    expect(await h.store.fork('s1', first.id, true)).toBeNull();
    expect(h.calls.some(([c]) => c === 'chat_fork_session')).toBe(false);
    h.store.dispose();
  });

  it('stays in the original and tells the user why when the host refuses', async () => {
    const h = await opened({ chat_fork_session: () => { throw '陪伴对话不能分叉'; } });
    const [first] = h.store.current()!.items;
    expect(await h.store.fork('s1', first.id, true)).toBeNull();
    expect(h.store.currentId).toBe('s1');
    expect(h.deps.notify).toHaveBeenCalledWith('陪伴对话不能分叉', 'error');
    h.store.dispose();
  });

  it('says that file attachments are not carried back into the input box', async () => {
    const withFile = [said('user', '看附件', { structured_content: { attachments: [{ name: '预算.csv', size: 9, mime: 'text/csv' }] } }), said('assistant', '好')];
    const h = await opened({ chat_load_session: () => withFile });
    const [first] = h.store.current()!.items;
    await h.store.fork('s1', first.id, true);
    expect(h.deps.notify).toHaveBeenCalledWith('文件附件没有带回输入框，需要的话请重新添加。', 'info');
    h.store.dispose();
  });
});

describe('each conversation works in its own folder', () => {
  const folders = [{ path: '/p/site', name: 'site', exists: true, wide: false }, { path: '/p/gone', name: 'gone', exists: false, wide: false }];
  const withHistory: WireMessage[] = [{ role: 'user', parts: [{ type: 'text', text: '改一下首页' }] }];

  it('a new conversation starts in the folder chosen before its first message', async () => {
    const h = harness({
      chat_overview: () => overview({ workspace: '/default', recentWorkspaces: folders }),
      chat_new_session: args => ({ ...summary('s1'), workspace: String(args?.workspace ?? '/p/site') }),
      chat_send: () => ({ ...summary('s1', 'x', 2), workspace: '/p/other' }),
    });
    await h.store.init();
    expect(h.store.currentWorkspace).toBe('/p/site'); // the folder used last that still exists
    expect(await h.store.switchWorkspace('/p/other')).toBe('done');
    expect(h.store.currentWorkspace).toBe('/p/other');
    await h.store.send('你好');
    expect(h.calls.find(([c]) => c === 'chat_new_session')?.[1]).toEqual({ permissionMode: 'ask', workspace: '/p/other' });
    expect(h.store.pendingWorkspace).toBe('');
    expect(h.store.currentWorkspace).toBe('/p/other');
  });

  it('an empty conversation switches at once, one with messages asks first', async () => {
    const result = (id: string, workspace: string) => ({ session: { ...summary(id), workspace }, wide: false, recentWorkspaces: folders });
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('empty'), { ...summary('used'), workspace: '/p/site' }], workspace: '/default' }),
      chat_load_session: args => (args?.sessionId === 'used' ? withHistory : []),
      chat_set_workspace: args => result(String(args?.sessionId), String(args?.workspace)),
    });
    await h.store.init();
    await h.store.select('empty');
    expect(h.store.currentWorkspace).toBe('/default'); // an old conversation uses the default folder
    expect(await h.store.switchWorkspace('/p/b')).toBe('done');
    expect(h.store.currentWorkspace).toBe('/p/b');
    expect(h.store.views.empty.items).toEqual([]);

    await h.store.select('used');
    expect(await h.store.switchWorkspace('/p/b')).toBe('ask');
    expect(h.calls.filter(([c]) => c === 'chat_set_workspace')).toHaveLength(1); // nothing changed yet
    expect(await h.store.switchWorkspace('/p/b', 'here')).toBe('done');
    expect(h.store.currentWorkspace).toBe('/p/b');
    const last = h.store.views.used.items.at(-1);
    expect(last?.kind === 'notice' && last.text).toContain('工作区已切换到 /p/b');
  });

  it('the sidebar moves any conversation to a folder, not only the open one', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('open'), { ...summary('other'), workspace: '/p/site' }], workspace: '/default' }),
      chat_load_session: () => withHistory,
      chat_set_workspace: args => ({ session: { ...summary(String(args?.sessionId)), workspace: String(args?.workspace) }, wide: false, recentWorkspaces: [] }),
    });
    await h.store.init();
    await h.store.select('open');
    expect(await h.store.moveToWorkspace('other', '/p/b')).toBe(true);
    expect(h.calls.find(([c]) => c === 'chat_set_workspace')?.[1]).toEqual({ sessionId: 'other', workspace: '/p/b' });
    expect(h.store.sessions.find(s => s.id === 'other')?.workspace).toBe('/p/b');
    expect(h.store.currentId).toBe('open'); // the open conversation stays open
    expect(await h.store.moveToWorkspace('open', '/p/c')).toBe(true);
    const last = h.store.views.open.items.at(-1);
    expect(last?.kind === 'notice' && last.text).toContain('工作区已切换到 /p/c');
    expect(await h.store.moveToWorkspace('open', '/p/c')).toBe(true); // already there: nothing to do
    expect(h.calls.filter(([c]) => c === 'chat_set_workspace')).toHaveLength(2);
  });

  it('moves the open conversation into an isolated copy and back, telling the page each time', async () => {
    const link = { origin: '/p/site', path: '/data/worktrees/site-1', branch: 'xiaoban/1', base: 'abc' };
    const h = harness({
      chat_overview: () => overview({ sessions: [{ ...summary('used'), workspace: '/p/site' }] }),
      chat_load_session: () => withHistory,
      chat_worktree_status: () => ({ active: false, available: true }),
      chat_worktree_start: () => ({ session: { ...summary('used'), workspace: '/data/worktrees/site-1', worktree: link } }),
      chat_worktree_finish: args => ({ session: { ...summary('used'), workspace: '/p/site', worktree: null }, message: args?.action === 'merge' ? '已合并' : '已保留' }),
    });
    await h.store.init();
    await h.store.select('used');
    expect(h.store.currentWorktree).toBeNull();
    expect(await h.store.worktreeInfo()).toEqual({ active: false, available: true });
    expect(await h.store.startWorktree()).toBe(true);
    expect(h.calls.find(([c]) => c === 'chat_worktree_start')?.[1]).toEqual({ sessionId: 'used' });
    expect(h.store.currentWorktree).toEqual(link);
    expect(h.store.currentWorkspace).toBe('/data/worktrees/site-1');
    let last = h.store.views.used.items.at(-1);
    expect(last?.kind === 'notice' && last.text).toContain('xiaoban/1');
    expect(await h.store.finishWorktree('merge')).toBe(true);
    expect(h.calls.find(([c]) => c === 'chat_worktree_finish')?.[1]).toEqual({ sessionId: 'used', action: 'merge' });
    expect(h.store.currentWorktree).toBeNull();
    expect(h.store.currentWorkspace).toBe('/p/site');
    last = h.store.views.used.items.at(-1);
    expect(last?.kind === 'notice' && last.text).toContain('已合并');
  });

  it('does not touch the copy while the conversation is answering, and keeps the state when the host refuses', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [{ ...summary('used'), workspace: '/p/site' }] }),
      chat_load_session: () => withHistory,
      chat_worktree_start: () => { throw new Error('这个仓库还没有任何提交'); },
    });
    await h.store.init();
    await h.store.select('used');
    expect(await h.store.startWorktree()).toBe(false);
    expect(h.store.currentWorktree).toBeNull();
    expect(h.store.sessions.find(s => s.id === 'used')?.workspace).toBe('/p/site');
    h.store.views.used.running = true;
    expect(await h.store.finishWorktree('discard')).toBe(false);
    expect(await h.store.startWorktree()).toBe(false);
    expect(h.calls.filter(([c]) => c === 'chat_worktree_start')).toHaveLength(1);
    expect(h.calls.some(([c]) => c === 'chat_worktree_finish')).toBe(false);
  });

  it('“新开对话” leaves the old conversation where it was', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [{ ...summary('used'), workspace: '/p/site' }] }),
      chat_load_session: () => withHistory,
      chat_new_session: args => ({ ...summary('fresh', '新对话', 5), workspace: String(args?.workspace) }),
    });
    await h.store.init();
    await h.store.select('used');
    expect(await h.store.switchWorkspace('/p/b', 'new')).toBe('done');
    expect(h.store.currentId).toBe('fresh');
    expect(h.store.currentWorkspace).toBe('/p/b');
    expect(h.store.sessions.find(s => s.id === 'used')?.workspace).toBe('/p/site');
    expect(h.calls.some(([c]) => c === 'chat_set_workspace')).toBe(false);
  });

  it('a running conversation is not moved, and a refusal is shown', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1')], running: ['s1'] }),
      chat_load_session: () => withHistory,
    });
    await h.store.init();
    await h.store.select('s1');
    expect(await h.store.switchWorkspace('/p/b', 'here')).toBe('failed');
    expect(h.deps.notify).toHaveBeenCalled();
  });

  it('the note saved when a folder changed is shown as a notice when the conversation is opened again', async () => {
    const h = harness({
      chat_overview: () => overview({ sessions: [summary('s1')] }),
      chat_load_session: () => [...withHistory, { role: 'user', synthetic: true, parts: [{ type: 'text', text: '【工作区已切换】从现在起这个对话在“/p/b”里工作（之前是“/p/a”）。' }] }],
    });
    await h.store.init();
    await h.store.select('s1');
    const notice = h.store.views.s1.items[1];
    expect(notice.kind === 'notice' && notice.text).toContain('工作区已切换到 /p/b');
  });
});


describe('sequenced session snapshots', () => {
  it('keeps history, buffers concurrent deltas and deduplicates repeated sequences', async () => {
    let resolve!: (value: unknown) => void;
    const h = harness({ chat_overview: () => ({ ...overview({ sessions: [summary('s1')], running: ['s1'] }), snapshotProtocol: 1 }),
      chat_session_snapshot: () => new Promise(r => { resolve = r; }) });
    await h.store.init();
    const selected = h.store.select('s1');
    const delta = { session: 's1', runId: 'r1', sequence: 3, event: { type: 'text_delta', text: ' world' } };
    h.emit('chat-event', delta); h.emit('chat-event', delta);
    resolve({ messages: [{ role: 'user', parts: [{ type: 'text', text: 'question' }] }], runId: 'r1', sequence: 2,
      events: [{ session: 's1', runId: 'r1', sequence: 2, event: { type: 'text_delta', text: 'hello' } }], running: true });
    await selected;
    expect(h.store.current()!.items.map(item => 'text' in item ? item.text : '')).toEqual(['question', 'hello world']);
    h.emit('chat-event', delta);
    expect(h.store.current()!.items.at(-1)).toMatchObject({ text: 'hello world' });
  });
});

describe('recoverable pending interactions',()=>{it('restores cards from snapshot and ignores stale revisions',async()=>{
 const h=harness({chat_overview:()=>overview({sessions:[summary('s1')],snapshotProtocol:1,pendingProtocol:1}),chat_session_snapshot:()=>({messages:[],ready:true,running:true,pending:{revision:4,approvals:[approval('a1')],questions:[question('q1')]}})});
 await h.store.init();await h.store.select('s1');expect(h.store.views.s1.approvals).toHaveLength(1);expect(h.store.views.s1.questions).toHaveLength(1);
 h.emit('chat-pending-changed',{session:'s1',pending:{revision:3,approvals:[],questions:[]}});expect(h.store.views.s1.approvals).toHaveLength(1);
 h.emit('chat-pending-changed',{session:'s1',pending:{revision:5,approvals:[],questions:[]}});expect(h.store.views.s1.approvals).toHaveLength(0);
});});
describe('which window asked', () => {
  it('knows the question was asked here until its answer is done, and not for a question the other window asked', async () => {
    const onReply = vi.fn();
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [], chat_send: () => summary('s1') });
    const store = createChatStore({ ...h.deps, onReply });
    await store.init(); await store.select('s1');
    expect(store.sentHere('s1')).toBe(false);
    await store.send('你好');
    expect(store.sentHere('s1')).toBe(true);
    const emit = (event: KernelEvent) => (h.deps.listen as ReturnType<typeof vi.fn>).mock.calls.filter(([name]) => name === 'chat-event').forEach(([, callback]) => (callback as (payload: unknown) => void)({ session: 's1', event }));
    emit({ type: 'text_delta', text: '你好呀' } as KernelEvent);
    emit({ type: 'done' } as KernelEvent);
    expect(onReply).toHaveBeenCalledWith('s1', '你好呀', expect.any(String));
    expect(store.sentHere('s1')).toBe(true); // still true while the reply is handed over…
    await Promise.resolve();
    expect(store.sentHere('s1')).toBe(false); // …and over once it is done
    store.dispose();
  });
  it('forgets the question when the host refuses it', async () => {
    const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')] }), chat_load_session: () => [], chat_send: () => { throw new Error('busy'); } });
    await h.store.init(); await h.store.select('s1');
    expect(await h.store.send('你好')).toBe(false);
    expect(h.store.sentHere('s1')).toBe(false);
  });
});

it('retains backend partial outcomes after an unrelated successful tool',async()=>{
  const h=harness({chat_new_session:()=>summary('s1'),chat_send:()=>summary('s1')});
  await h.store.init();await h.store.send('生成文件');
  h.events('s1',{type:'tool_start',id:'a',name:'write_file',arguments:'{"path":"a.txt"}'},{type:'tool_end',id:'a',output:'failed',is_error:true},
    {type:'tool_start',id:'b',name:'read_file',arguments:'{"path":"b.txt"}'},{type:'tool_end',id:'b',output:'ok',is_error:false},{type:'done'});
  h.emit('chat-task-outcome',{session:'s1',outcome:{runId:'r1',status:'partial',validator:'run_trace_only',evidence:'unresolved_tool_errors=1'}});
  expect(h.store.current()?.items).toContainEqual(expect.objectContaining({kind:'notice',taskStatus:'partial',tone:'warning',canContinue:true}));
  h.store.dispose();
});

it('updates verification in place and does not let a delayed trace outcome undo it', async () => {
  const h = harness(); await h.store.init();
  h.events('s1', { type: 'tool_start', id: 't1', name: 'todo', arguments: '{}' }, { type: 'tool_end', id: 't1', output: 'Todo list updated:', is_error: false });
  const outcome = { runId: 'r1', sessionId: 's1', lastToolCallId: 't1', status: 'unknown', validator: 'run_trace_only', evidence: '' };
  h.emit('chat-task-outcome', { session: 's1', outcome });
  h.emit('chat-task-outcome', { session: 's1', outcome: { ...outcome, status: 'succeeded', validator: 'user_confirmation', verifiedAtMs: 100, evidence: '已核实' } });
  h.emit('chat-task-outcome', { session: 's1', outcome });
  const notices = h.store.views.s1.items.filter(i => i.id === 'outcome:r1');
  expect(notices).toHaveLength(1); expect(notices[0]).toMatchObject({ taskStatus: 'succeeded', details: '已核实' });
  h.store.dispose();
});

it('restores each saved outcome to its own reply rather than the latest task', async () => {
  const h = harness({ chat_overview: () => overview({ sessions: [summary('s1')], snapshotProtocol: 1 }),
    chat_session_snapshot: () => ({ runId: 'r2', running: false, ready: true, messages: [
      { role: 'user', parts: [{ type: 'text', text: '旧任务' }] },
      { role: 'assistant', tool_calls: [{ id: 'old-tool', name: 'todo', arguments: JSON.stringify({ todos: [{ content: '检查结果', status: 'completed' }] }) }] },
      { role: 'tool', tool_call_id: 'old-tool', parts: [{ type: 'text', text: 'Todo list updated:' }] },
      { role: 'user', parts: [{ type: 'text', text: '新任务' }] },
      { role: 'assistant', tool_calls: [{ id: 'new-tool', name: 'todo', arguments: JSON.stringify({ todos: [{ content: '检查结果', status: 'completed' }] }) }] },
      { role: 'tool', tool_call_id: 'new-tool', parts: [{ type: 'text', text: 'Todo list updated:' }] },
    ], outcomes: [
      { runId: 'r1', sessionId: 's1', lastToolCallId: 'old-tool', status: 'unknown', validator: 'run_trace_only', evidence: 'old' },
      { runId: 'r2', sessionId: 's1', lastToolCallId: 'new-tool', status: 'succeeded', validator: 'user_confirmation', evidence: 'new', verifiedAtMs: 100 },
    ] }) });
  await h.store.init(); await h.store.select('s1');
  const items = h.store.views.s1.items;
  const nextUser = items.findIndex(i => i.kind === 'user' && i.text === '新任务');
  expect(items.findIndex(i => i.id === 'outcome:r1')).toBeLessThan(nextUser);
  expect(items.findIndex(i => i.id === 'outcome:r2')).toBeGreaterThan(nextUser);
  expect(items.find(i => i.id === 'outcome:r1')).toMatchObject({ taskStatus: 'unknown' });
  expect(items.find(i => i.id === 'outcome:r2')).toMatchObject({ taskStatus: 'succeeded' });
  const replies = replyBlocks(items).filter(block => block.calls.length);
  expect(taskProgress(replies[0], false)?.state).toBe('completed');
  expect(taskProgress(replies[1], false)?.state).toBe('completed'); h.store.dispose();
});


describe('durable message queues', () => {
  const queued = (revision = 1) => ({revision,paused:false,reason:'',enqueuedId:'q1',items:[{id:'q1',text:'next',mode:'queue' as const,dispatching:false,images:0,files:[]}]});
  it('keeps a pending enqueue and its attachments bound to the original session after switching', async () => {
    let finish!: (value: unknown) => void;
    const h = harness({chat_overview:()=>overview({sessions:[summary('s1'),summary('s2')]}),chat_load_session:()=>[],chat_enqueue:()=>new Promise(resolve=>{finish=resolve;})});
    await h.store.init();await h.store.select('s1');h.events('s1',{type:'text_delta',text:'working'});
    const task=h.store.enqueue('next',[{name:'report.txt',size:1,mime:'text/plain',dataUrl:'data:text/plain;base64,YQ=='}],[]);
    expect(await h.store.enqueue('duplicate',[],[])).toBe(false);
    await h.store.select('s2');finish(queued());expect(await task).toBe(true);
    expect(h.store.currentId).toBe('s2');expect(h.store.queue).toBeNull();expect(h.store.queues.s1.items[0]?.text).toBe('next');
    expect(h.calls.find(([c])=>c==='chat_enqueue')?.[1]).toMatchObject({sessionId:'s1',attachments:[{name:'report.txt',dataUrl:'data:text/plain;base64,YQ=='}]});
    expect(h.calls.filter(([c])=>c==='chat_enqueue')).toHaveLength(1);h.store.dispose();
  });
  it('reuses the submission identity after a failed enqueue and supports older WebViews without randomUUID', async () => {
    let fail=true;
    const h=harness({chat_overview:()=>overview({sessions:[summary('s1')]}),chat_load_session:()=>[],chat_enqueue:()=>{if(fail)throw Error('lost response');return queued();}});
    await h.store.init();await h.store.select('s1');
    vi.stubGlobal('crypto', {});
    try {
      expect(await h.store.enqueue('next',[],[])).toBe(false);fail=false;expect(await h.store.enqueue('next',[],[])).toBe(true);
      const calls=h.calls.filter(([c])=>c==='chat_enqueue');expect(calls[0]?.[1]?.requestId).toBe(calls[1]?.[1]?.requestId);expect(calls[0]?.[1]?.requestId).toMatch(/^q-[a-z0-9-]+$/);
      expect(h.store.sending).toBe(false);
    } finally {vi.unstubAllGlobals();h.store.dispose();}
  });
  it('ignores stale queue events and keeps another session isolated', async () => {
    const h=harness({chat_overview:()=>overview({sessions:[summary('s1')]}),chat_load_session:()=>[]});await h.store.init();await h.store.select('s1');
    h.emit('chat-queue-changed',{session:'s1',queue:queued(5)});
    h.emit('chat-queue-changed',{session:'s1',queue:{...queued(4),items:[]}});
    h.emit('chat-queue-changed',{session:'s2',queue:{...queued(9),paused:true}});
    expect(h.store.queue?.revision).toBe(5);expect(h.store.queue?.items).toHaveLength(1);expect(h.store.queue?.paused).toBe(false);h.store.dispose();
  });
  it('uses the accepted message id for stop-and-send even if another window queued later', async () => {
    const value=queued();value.items.push({...value.items[0]!,id:'other',text:'another window'});
    const h=harness({chat_overview:()=>overview({sessions:[summary('s1')]}),chat_load_session:()=>[],chat_enqueue:()=>value,chat_queue_update:()=>queued(2)});await h.store.init();await h.store.select('s1');
    expect(await h.store.enqueue('next',[],[],'replace')).toBe(true);
    expect(h.calls.find(([c])=>c==='chat_enqueue')?.[1]).toMatchObject({sessionId:'s1',stopNow:true});
    expect(h.calls.filter(([c])=>c==='chat_queue_update')).toHaveLength(0);h.store.dispose();
  });
});

it('keeps the design selected at submission in both direct and queued messages',async()=>{
 const h=harness({chat_new_session:()=>summary('s1'),chat_send:()=>summary('s1'),chat_enqueue:()=>({revision:1,paused:false,reason:'',items:[]})});await h.store.init();
 const first='design-012345678901234567890123',second='design-000000000000000000000000';
 expect(await h.store.send('紧凑一点',[],undefined,[],first)).toBe(true);h.events('s1',{type:'done'});
 expect(await h.store.enqueue('改首页',[],[],'queue',second)).toBe(true);
 expect(h.calls.find(([c])=>c==='chat_send')?.[1]).toMatchObject({text:'紧凑一点',designId:first});
 expect(h.calls.find(([c])=>c==='chat_enqueue')?.[1]).toMatchObject({text:'改首页',designId:second});h.store.dispose();
});

it('uses the failed operation from the correct reply and preserves uncertainty guards', async () => {
  const h = harness(); await h.store.init();
  h.events('s1', { type: 'tool_start', id: 'write', name: 'write_file', arguments: '{"path":"settings.html"}' },
    { type: 'tool_end', id: 'write', output: 'Permission denied', is_error: true }, { type: 'done' });
  const outcome = { runId: 'r1', sessionId: 's1', lastToolCallId: 'write', status: 'partial', validator: 'run_trace_only', evidence: 'unresolved_tool_errors=1' };
  h.emit('chat-task-outcome', { session: 's1', outcome });
  expect(h.store.views.s1.items.find(item => item.id === 'outcome:r1')).toMatchObject({
    text: '部分完成：写入文件（settings.html）失败。', details: expect.stringContaining('Permission denied'), canContinue: true,
  });
  h.emit('chat-task-outcome', { session: 's1', outcome: { ...outcome, status: 'unknown', evidence: 'terminal=true; incomplete_tools=1' } });
  expect(h.store.views.s1.items.find(item => item.id === 'outcome:r1')).toMatchObject({ tone: 'warning', canContinue: false, text: expect.stringContaining('尚未返回结果') });
  h.store.dispose();
});

it('does not enable continuation after an outcome-needs-review guard', async () => {
  const h = harness(); await h.store.init();
  h.events('s1', { type: 'tool_start', id: 'external', name: 'shell', arguments: '{}' },
    { type: 'error', message: 'run paused by continuation guard: tool_outcome_needs_review' });
  h.emit('chat-task-outcome', { session: 's1', outcome: { runId: 'r1', lastToolCallId: 'external', status: 'failed', validator: 'run_trace_only', evidence: 'terminal=true' } });
  expect(h.store.views.s1.items.find(item => item.id === 'outcome:r1')).toMatchObject({ canContinue: false, text: expect.stringContaining('操作结果尚未确认') });
  h.store.dispose();
});
