// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest';
import { createChatStore, type ChatDeps } from './chat-store.ts';
import type { SessionModel, SessionSummary } from './types.ts';
import { sessionModelConfig } from './session-model.ts';
import type { AiConfigView } from '../../bridge.ts';

const model = (id: string): SessionModel => ({ providerId: id, model: `${id}-model`, replyPreference: 'medium' });
const session = (id: string, permissionMode: 'ask' | 'auto' = 'ask'): SessionSummary => ({
  id, title: id, createdAt: 1, updatedAt: 1, permissionMode, model: model(id), contextTokens: 256_000, connectors: [],
});
function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
function setup(overrides: Record<string, (args?: Record<string, unknown>) => unknown> = {}) {
  const handlers = new Map<string, (payload: never) => void>();
  const list = [session('a', 'auto'), session('b')];
  const table: typeof overrides = {
    chat_overview: () => ({ configured: true, sessions: list.map(s => ({ ...s })), groups: [], running: [], workspace: '/work', permissionMode: 'ask' }),
    chat_load_session: () => [],
    chat_context: () => ({ used: 100, limit: 256_000, canCompact: false }),
    chat_set_model: args => args!.selection,
    chat_set_session_options: () => undefined,
    chat_set_permission_mode: () => undefined,
    chat_send: args => session(String(args!.sessionId)),
    ...overrides,
  };
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => table[command]?.(args));
  const notify = vi.fn();
  const store = createChatStore({ desktop: true, invoke: invoke as ChatDeps['invoke'], notify,
    listen: async (event, callback) => { handlers.set(event, callback as never); return () => {}; } });
  return { store, invoke, notify, handlers, list };
}

describe('conversation state isolation', () => {
  it('pins a model and reasoning change to its original conversation while another is opened', async () => {
    const gate = deferred<SessionModel>();
    const h = setup({ chat_set_model: () => gate.promise });
    await h.store.init(); await h.store.select('a');
    const changed = h.store.setModel({ ...model('replacement'), replyPreference: 'deeper' });
    await h.store.select('b'); gate.resolve({ ...model('replacement'), replyPreference: 'deeper' });
    expect(await changed).toBe(true);
    expect(h.store.model).toEqual(model('b'));
    expect(h.store.sessions.find(s => s.id === 'a')!.model?.replyPreference).toBe('deeper');
    expect(h.invoke).toHaveBeenCalledWith('chat_set_model', { sessionId: 'a', selection: { ...model('replacement'), replyPreference: 'deeper' } });
    expect(h.invoke.mock.calls.some(([command]) => ['save_ai_provider', 'use_ai_gateway'].includes(command))).toBe(false);
    h.store.dispose();
  });

  it('keeps pending model, context and connector choices with the newly created session', async () => {
    const h = setup({ chat_new_session: args => ({ ...session('new'), model: args!.model, connectors: args!.connectors }) });
    await h.store.init();
    await h.store.setModel(model('pending'));
    await h.store.setContextSize(512_000);
    h.store.draft.connectors.push({ kind: 'connector', id: 'repo', name: '仓库', description: '' });
    const original = h.store.draft;
    expect(await h.store.send('开始')).toBe(true);
    expect(h.invoke).toHaveBeenCalledWith('chat_new_session', expect.objectContaining({ model: model('pending'), contextTokens: 512_000, connectors: ['repo'], permissionMode: 'ask' }));
    expect(h.store.draft).toBe(original);
    expect(h.store.draft.connectors[0].id).toBe('repo');
    expect(h.invoke.mock.calls.some(([command]) => command === 'set_context_tokens')).toBe(false);
    h.store.dispose();
  });

  it('does not overwrite session permission when configuration is refreshed or a default event arrives', async () => {
    const h = setup(); await h.store.init(); await h.store.select('a');
    await h.store.refresh(); expect(h.store.permissionMode).toBe('auto');
    h.handlers.get('chat-permission-mode-changed')!({ mode: 'ask' } as never);
    expect(h.store.permissionMode).toBe('auto');
    await h.store.select(null); expect(h.store.permissionMode).toBe('ask');
    await h.store.setPermissionMode('auto');
    expect(h.invoke.mock.calls.some(([command]) => command === 'chat_set_permission_mode')).toBe(false);
    await h.store.select('b'); expect(h.store.permissionMode).toBe('ask');
    h.store.dispose();
  });

  it('rolls a failed permission save back on its owner without changing the newly selected session', async () => {
    const gate = deferred<void>(); const h = setup({ chat_set_permission_mode: () => gate.promise });
    await h.store.init(); await h.store.select('a');
    const saving = h.store.setPermissionMode('ask'); await h.store.select('b');
    gate.reject(new Error('磁盘不可写')); await saving;
    expect(h.store.permissionMode).toBe('ask');
    expect(h.store.sessions.find(s => s.id === 'a')!.permissionMode).toBe('auto');
    h.store.dispose();
  });

  it('restores text, attachments, skills and connector choices separately for both conversations', async () => {
    const h = setup(); await h.store.init(); await h.store.select('a');
    h.store.draft.text = 'A 的草稿'; h.store.draft.attachments = ['data:image/png;base64,YQ=='];
    h.store.draft.skills = [{ name: 'check', title: '检查' }];
    h.store.draft.connectors = [{ kind: 'connector', id: 'repo', name: '仓库', description: '' }];
    await h.store.select('b'); expect(h.store.draft.text).toBe(''); expect(h.store.draft.connectors).toEqual([]);
    h.store.draft.text = 'B 的草稿'; await h.store.select('a');
    expect(h.store.draft).toMatchObject({ text: 'A 的草稿', attachments: ['data:image/png;base64,YQ=='], skills: [{ name: 'check' }], connectors: [{ id: 'repo' }] });
    await vi.waitFor(() => expect(h.invoke).toHaveBeenCalledWith('chat_set_session_options', { sessionId: 'a', connectors: ['repo'] }));
    await h.store.select('b'); expect(h.store.draft.text).toBe('B 的草稿'); h.store.dispose();
  });

  it('restores persisted connector choices after the window is recreated', async () => {
    const h = setup(); h.list[0].connectors = ['repo']; await h.store.init(); await h.store.select('a');
    expect(h.store.draft.connectors).toMatchObject([{ kind: 'connector', id: 'repo' }]); h.store.dispose();
  });

  it('never steals focus when session creation finishes after a switch, including a failed first send', async () => {
    const created = deferred<SessionSummary>(); const sent = deferred<SessionSummary>();
    const h = setup({ chat_new_session: () => created.promise, chat_send: () => sent.promise, chat_delete_session: () => [] });
    await h.store.init(); const sending = h.store.send('新任务'); expect(h.store.sending).toBe(true);
    await h.store.select('b'); created.resolve(session('new'));
    await vi.waitFor(() => expect(h.invoke).toHaveBeenCalledWith('chat_send', expect.objectContaining({ sessionId: 'new' })));
    expect(h.store.currentId).toBe('b'); sent.reject(new Error('服务不可用')); expect(await sending).toBe(false);
    expect(h.store.currentId).toBe('b'); h.store.dispose();
  });

  it('allows only one first-message creation while its request is pending', async () => {
    const created = deferred<SessionSummary>(); const h = setup({ chat_new_session: () => created.promise });
    await h.store.init(); const first = h.store.send('第一次'); expect(await h.store.send('重复点击')).toBe(false);
    created.resolve(session('new')); await first;
    expect(h.invoke.mock.calls.filter(([command]) => command === 'chat_new_session')).toHaveLength(1); h.store.dispose();
  });

  it('saves a delayed context change to the captured owner and leaves the current ring alone', async () => {
    const gate = deferred<void>(); const h = setup({ chat_set_session_options: () => gate.promise });
    await h.store.init(); await h.store.select('a');
    const saved = h.store.setContextSize(512_000, 'a'); await h.store.select('b');
    gate.resolve(); await saved;
    expect(h.invoke).toHaveBeenCalledWith('chat_set_session_options', { sessionId: 'a', contextTokens: 512_000 });
    expect(h.store.context.limit).toBe(256_000); expect(h.store.sessions.find(s => s.id === 'b')!.contextTokens).toBe(256_000); h.store.dispose();
  });

  it('overlays a session model without modifying shared provider defaults', () => {
    const config = { activeProvider: 'a', model: 'a-default', providers: [{ id: 'a', model: 'a-default', replyPreference: 'faster' }] } as AiConfigView;
    const view = sessionModelConfig(config, { ...model('a'), replyPreference: 'deeper' });
    expect(view?.model).toBe('a-model'); expect(view?.providers![0].replyPreference).toBe('deeper');
    expect(config.model).toBe('a-default'); expect(config.providers![0].replyPreference).toBe('faster');
  });
});
