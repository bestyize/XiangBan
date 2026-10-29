// @vitest-environment happy-dom
import { reactive } from 'vue';
import { flushPromises } from '@vue/test-utils';
import { describe, it, expect, vi } from 'vitest';
import { createAssistantStore } from './assistant-store.ts';

function setup(configured = true) {
  let pending: string | null = null;
  const listeners = new Map<string, () => void>();
  const chat = reactive({ configured, sessions: [] as any[], views: {} as Record<string, any>, currentId: null as string | null, permissionMode: 'ask', loadError: '',
    init: vi.fn(async () => {}), refresh: vi.fn(async () => {}), select: vi.fn(async (id: string | null) => { chat.currentId = id; }),
    send: vi.fn(), cancel: vi.fn(), answerApproval: vi.fn(), answerQuestion: vi.fn() });
  const asr = reactive({ enabled: true, init: vi.fn(async () => {}), deactivate: vi.fn(), activate: vi.fn(), recordingRequested: false, listening: false });
  const speech = reactive({ settings: {}, status: 'idle', init: vi.fn(async () => {}), stop: vi.fn(async () => {}), speak: vi.fn() });
  const invoke = vi.fn(async (command: string) => {
    if (command === 'assistant_prefs') return { open: 'type', read: 'never' };
    if (command === 'assistant_take_companion_session') { const result = pending; pending = null; return result; }
    if (command === 'chat_overview') return { sessions: [{ id: 'pet-a', title: '小葵', companionId: 'pet:a' }] };
  });
  const store = createAssistantStore({ desktop: true, invoke: invoke as never, chat: chat as never, asr: asr as never, speech: speech as never,
    listen: async (event, fn) => { listeners.set(event, fn as () => void); return () => {}; }, setTimer: vi.fn(), clearTimer: vi.fn() });
  return { store, chat, asr, invoke, pending: (id: string) => { pending = id; }, emit: () => listeners.get('assistant-companion-session')?.() };
}
describe('opening a companion conversation', () => {
  it('loads the seeded session on first open, even without a configured model', async () => {
    const h = setup(false); h.pending('pet-a'); await h.store.init(); await h.store.opened();
    expect(h.chat.select).toHaveBeenCalledWith('pet-a'); expect(h.asr.activate).not.toHaveBeenCalled(); h.store.dispose();
  });
  it('selects a new companion session while the assistant is already open', async () => {
    const h = setup(); await h.store.init(); await h.store.opened(); h.pending('pet-a'); h.emit(); await flushPromises();
    expect(h.chat.currentId).toBe('pet-a'); h.store.dispose();
  });
  it('opens the main chat while preserving an existing unsent draft', async () => {
    const h = setup(); await h.store.init(); await h.store.opened(); h.store.input = '还没写完'; h.pending('pet-a'); h.emit(); await flushPromises();
    expect(h.store.input).toBe('还没写完'); expect(h.invoke).toHaveBeenCalledWith('assistant_open_chat', { session: 'pet-a' }); h.store.dispose();
  });
  it('does not cancel or append to another running conversation', async () => {
    const h = setup(); await h.store.init(); await h.store.opened(); h.chat.currentId = 'work'; h.chat.views.work = { running: true, items: [], approvals: [], questions: [] }; h.pending('pet-a'); h.emit(); await flushPromises();
    expect(h.chat.currentId).toBe('work'); expect(h.chat.cancel).not.toHaveBeenCalled(); expect(h.invoke).toHaveBeenCalledWith('assistant_open_chat', { session: 'pet-a' }); h.store.dispose();
  });
});
