import { nextTick, reactive } from 'vue';
import { describe, expect, it, vi } from 'vitest';
import { createAssistantStore, LISTEN_IDLE_MS, RESUME_MS, SEND_GRACE_MS } from './assistant-store.ts';

function setup(options: { configured?: boolean; voice?: boolean } = {}) {
  let clock = 1_000_000;
  const timers: { id: number; at: number; run: () => void }[] = [];
  let nextId = 1;
  const advance = (ms: number) => {
    clock += ms;
    for (;;) {
      const due = timers.filter(timer => timer.at <= clock).sort((a, b) => a.at - b.at)[0];
      if (!due) break;
      timers.splice(timers.indexOf(due), 1);
      due.run();
    }
  };
  const chat = reactive({
    configured: options.configured ?? true, currentId: null as string | null, views: {} as Record<string, any>,
    init: vi.fn(async () => {}),
    select: vi.fn(async (id: string | null) => { chat.currentId = id; }),
    send: vi.fn(async (text: string) => {
      chat.currentId = 's1';
      chat.views.s1 = { id: 's1', items: [{ kind: 'user', id: 'u', text, images: [] }], running: true, approvals: [], questions: [] };
      return true;
    }),
    cancel: vi.fn(async () => {}), answerApproval: vi.fn(async () => {}), answerQuestion: vi.fn(async () => {}),
  });
  let heard: (text: string) => void = () => {};
  const asr = reactive({
    enabled: options.voice ?? true, listening: false, recognizing: false, preparing: false, recordingRequested: false, level: 0, error: '',
    init: vi.fn(async () => {}), prepare: vi.fn(async () => true),
    activate: vi.fn(async (callback: (text: string) => void) => { asr.recordingRequested = true; asr.listening = true; heard = callback; }),
    deactivate: vi.fn(() => { asr.recordingRequested = false; asr.listening = false; }),
  });
  const speech = reactive({
    status: 'idle' as 'idle' | 'synthesizing' | 'playing', settings: { autoRead: false },
    init: vi.fn(async () => {}),
    speak: vi.fn(async () => { speech.status = 'playing'; }),
    stop: vi.fn(async () => { speech.status = 'idle'; }),
  });
  const invoke = vi.fn(async () => undefined as never);
  const store = createAssistantStore({
    desktop: true, invoke, listen: async () => () => {},
    chat: chat as never, asr: asr as never, speech: speech as never,
    now: () => clock,
    setTimer: (run, ms) => { const timer = { id: nextId++, at: clock + ms, run }; timers.push(timer); return timer.id; },
    clearTimer: id => { const at = timers.findIndex(timer => timer.id === id); if (at >= 0) timers.splice(at, 1); },
  });
  return { store, chat, asr, speech, invoke, advance, say: (text: string) => heard(text) };
}
const finishAnswer = (chat: ReturnType<typeof setup>['chat'], text = '好的，已经处理完了。') => {
  chat.views.s1.items.push({ kind: 'assistant', id: 'a', text, streaming: false });
  chat.views.s1.running = false;
};

describe('floating assistant', () => {
  it('starts a new conversation when opened, and listens when voice is on', async () => {
    const { store, chat, asr } = setup();
    await store.opened();
    expect(store.open).toBe(true);
    expect(chat.select).toHaveBeenCalledWith(null);
    expect(asr.activate).toHaveBeenCalledTimes(1);
    expect(store.phase).toBe('listening');
    expect(store.status).toBe('我在听，请说…');
  });

  it('continues a conversation left a moment ago, and starts over after a while', async () => {
    const { store, chat, advance } = setup({ voice: false });
    await store.opened();
    await store.submit('你好');
    store.closed();
    chat.select.mockClear();
    advance(RESUME_MS - 1000);
    await store.opened();
    expect(chat.select).not.toHaveBeenCalled();
    store.closed();
    advance(RESUME_MS + 1000);
    await store.opened();
    expect(chat.select).toHaveBeenCalledWith(null);
  });

  it('sends what was heard once speaking pauses, and stops the microphone while it answers', async () => {
    const { store, chat, asr, say, advance } = setup();
    await store.opened();
    say('今天天气'); advance(200); say('怎么样');
    expect(chat.send).not.toHaveBeenCalled();
    expect(store.input).toBe('今天天气 怎么样');
    advance(SEND_GRACE_MS + 1);
    await Promise.resolve();
    expect(chat.send).toHaveBeenCalledWith('今天天气 怎么样');
    expect(asr.deactivate).toHaveBeenCalled();
    expect(store.phase).toBe('thinking');
  });

  it('reads a spoken question\'s answer aloud and listens again afterwards, but stays quiet for typed ones', async () => {
    const spoken = setup();
    await spoken.store.opened();
    spoken.say('讲个笑话'); spoken.advance(SEND_GRACE_MS + 1); await Promise.resolve(); await Promise.resolve();
    finishAnswer(spoken.chat, '从前有座山。');
    spoken.asr.activate.mockClear();
    spoken.store.replied('s1', '从前有座山。', 'a');
    await nextTick();
    expect(spoken.speech.speak).toHaveBeenCalledWith('从前有座山。', 'a', undefined, true);
    expect(spoken.store.phase).toBe('speaking');
    spoken.speech.status = 'idle'; await nextTick();
    await vi.waitFor(() => expect(spoken.asr.activate).toHaveBeenCalledTimes(1));
    expect(spoken.store.phase).toBe('listening');

    const typed = setup({ voice: false });
    await typed.store.opened();
    typed.store.input = '你好';
    await typed.store.submit();
    finishAnswer(typed.chat);
    typed.store.replied('s1', '你好呀', 'a');
    expect(typed.speech.speak).not.toHaveBeenCalled();
  });

  it('shows the question and the answer of the latest turn, and where it is in it', async () => {
    const { store, chat } = setup({ voice: false });
    await store.opened();
    await store.submit('帮我查一下');
    expect(store.question).toBe('帮我查一下');
    expect(store.phase).toBe('thinking');
    chat.views.s1.items.push({ kind: 'tool', id: 't', name: 'web_search', arguments: '{}', output: '', status: 'running' });
    expect(store.working).toBe('正在联网查找');
    chat.views.s1.items.push({ kind: 'assistant', id: 'a', text: '找到了', streaming: true });
    expect(store.phase).toBe('answering');
    expect(store.answer).toBe('找到了');
    chat.views.s1.approvals.push({ id: 'p', session: 's1', tool: 'shell', risk: 'exec', arguments: {}, tainted: false, canRemember: false });
    expect(store.phase).toBe('attention');
    await store.decide('once');
    expect(chat.answerApproval).toHaveBeenCalledWith(chat.views.s1.approvals[0], 'once');
  });

  it('gives the text back when nothing could be sent', async () => {
    const { store, chat } = setup({ voice: false });
    await store.opened();
    chat.send.mockResolvedValueOnce(false);
    store.input = '再试一次';
    await store.submit();
    expect(store.input).toBe('再试一次');
  });

  it('asks for a model when none is set up, and for voice to be switched on when it is off', async () => {
    const noModel = setup({ configured: false });
    await noModel.store.opened();
    expect(noModel.store.hintTab).toBe('ai');
    expect(noModel.asr.activate).not.toHaveBeenCalled();
    await noModel.store.submit('hi');
    expect(noModel.chat.send).not.toHaveBeenCalled();

    const noVoice = setup({ voice: false });
    await noVoice.store.opened();
    expect(noVoice.store.hint).toBe('');
    await noVoice.store.toggleListening();
    expect(noVoice.store.hintTab).toBe('voice');
    expect(noVoice.asr.activate).not.toHaveBeenCalled();
    await noVoice.store.openSettings('voice');
    expect(noVoice.invoke).toHaveBeenCalledWith('assistant_open_settings', { tab: 'voice' });
  });

  it('stays where it is until it is dismissed, however long it sits and whatever else the user clicks', async () => {
    const { store, invoke, advance } = setup({ voice: false });
    await store.opened();
    advance(60 * 60 * 1000);
    expect(store.open).toBe(true);
    expect(invoke).not.toHaveBeenCalledWith('assistant_hide');

    const answering = setup({ voice: false });
    await answering.store.opened();
    await answering.store.submit('慢一点的问题');
    answering.advance(60 * 60 * 1000);
    expect(answering.store.open).toBe(true);
    expect(answering.invoke).not.toHaveBeenCalledWith('assistant_hide');
  });

  it('lets the microphone go when nobody talks, but keeps the capsule, and listens again on a tap', async () => {
    const { store, asr, advance } = setup();
    await store.opened();
    expect(asr.recordingRequested).toBe(true);
    advance(LISTEN_IDLE_MS - 1000);
    expect(asr.recordingRequested).toBe(true);
    advance(2000);
    expect(asr.recordingRequested).toBe(false);
    expect(store.open).toBe(true);
    expect(store.hint).toContain('没听到声音');
    await store.toggleListening();
    expect(asr.recordingRequested).toBe(true);
    expect(store.hint).toBe('');
  });

  it('keeps the microphone while the user is talking, and releases it once more after the next silence', async () => {
    const { store, asr, speech, say, advance, chat } = setup();
    await store.opened();
    advance(LISTEN_IDLE_MS - 1000);
    say('你好');
    advance(LISTEN_IDLE_MS * 2);
    // what was heard was sent, so the microphone is off while the answer is written
    await vi.waitFor(() => expect(chat.send).toHaveBeenCalledWith('你好'));
    expect(asr.recordingRequested).toBe(false);
    finishAnswer(chat, '你好呀');
    store.replied('s1', '你好呀', 'a');
    await nextTick();
    speech.status = 'idle'; await nextTick();
    await vi.waitFor(() => expect(asr.activate).toHaveBeenCalledTimes(2));
    advance(LISTEN_IDLE_MS + 1000);
    expect(asr.recordingRequested).toBe(false);
    expect(store.open).toBe(true);
  });

  it('stops listening and speaking when it closes, and can hand the conversation to the full chat', async () => {
    const { store, asr, speech, invoke } = setup();
    await store.opened();
    store.closed();
    expect(store.open).toBe(false);
    expect(asr.deactivate).toHaveBeenCalled();
    expect(speech.stop).toHaveBeenCalled();
    await store.submit('留下一个会话'); // closed: still sends nothing wrong
    await store.expand();
    expect(invoke).toHaveBeenCalledWith('assistant_open_chat', { session: expect.anything() });
  });
});
