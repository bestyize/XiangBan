import { describe, expect, it, vi } from 'vitest';
import { createChatStore, type ChatDeps } from './chat-store.ts';
import type { ChatEventPayload } from './types.ts';
describe('automatic speech events', () => {
  it('reads only a completed reply in the selected conversation and cancels on activity', async () => {
    const handlers = new Map<string, (payload: unknown) => void>();
    const onReply = vi.fn(), onActivity = vi.fn();
    const invoke: ChatDeps['invoke'] = async command => {
      if (command === 'chat_overview') return { configured: true, workspace: '', sessions: [], running: [] } as never;
      if (command === 'chat_load_session') return [] as never;
      return undefined as never;
    };
    const store = createChatStore({
      desktop: true, invoke, notify: vi.fn(), onReply, onActivity,
      listen: async (event, handler) => { handlers.set(event, handler as (payload: unknown) => void); return () => {}; },
    });
    await store.init(); await store.select('foreground');
    const emit = (payload: ChatEventPayload) => handlers.get('chat-event')!(payload);
    emit({ session: 'background', event: { type: 'text_delta', text: '后台回答' } });
    emit({ session: 'background', event: { type: 'done' } });
    expect(onReply).not.toHaveBeenCalled();
    emit({ session: 'foreground', event: { type: 'text_delta', text: '完整的回答' } });
    expect(onReply).not.toHaveBeenCalled();
    emit({ session: 'foreground', event: { type: 'done' } });
    expect(onReply).toHaveBeenCalledWith('foreground', '完整的回答', expect.any(String));
    await store.select(null); expect(onActivity).toHaveBeenCalledTimes(2);
    store.dispose();
  });
});
