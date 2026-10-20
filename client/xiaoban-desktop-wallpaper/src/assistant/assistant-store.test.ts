import { nextTick, reactive } from 'vue';
import { describe, expect, it, vi } from 'vitest';
import { createAssistantStore, LISTEN_IDLE_MS, SESSION_KEY, SEND_GRACE_MS } from './assistant-store.ts';
import { createChatStore } from '../ui/chat/chat-store.ts';

function setup(options: { configured?: boolean; voice?: boolean; prefs?: { open: string; read: string; appearance?: string }; storage?: Pick<Storage,'getItem'|'setItem'|'removeItem'> } = {}) {
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
    sessions: [] as any[], loadError:'', configured: options.configured ?? true, currentId: null as string | null, views: {} as Record<string, any>, permissionMode: 'ask' as 'ask' | 'auto',
    init: vi.fn(async () => {}),
    refresh: vi.fn(async () => {}),
    select: vi.fn(async (id: string | null) => { chat.currentId = id; }),
    send: vi.fn(async (text: string) => {
      chat.currentId ??= 's1';
      const id=chat.currentId;if(!chat.sessions.some(s=>s.id===id))chat.sessions.push({id,permissionMode:'auto'});
      chat.views[id] = { id, items: [{ kind: 'user', id: 'u', text, images: [] }], running: true, approvals: [], questions: [] };
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
  const handlers = new Map<string, (payload: unknown) => void>();
  let fresh=0;
  const invoke = vi.fn(async (command: string) => (command === 'assistant_prefs' ? options.prefs ?? { open: 'voice', read: 'voice' } : command==='chat_new_session'?{id:'fresh-'+(++fresh),permissionMode:'auto'}:undefined) as never);
  const appearance = vi.fn();
  const store = createAssistantStore({
    storage:options.storage,onAppearance: appearance, desktop: true, invoke, listen: async (event, callback) => { handlers.set(event, callback as (payload: unknown) => void); return () => {}; },
    chat: chat as never, asr: asr as never, speech: speech as never,
    now: () => clock,
    setTimer: (run, ms) => { const timer = { id: nextId++, at: clock + ms, run }; timers.push(timer); return timer.id; },
    clearTimer: id => { const at = timers.findIndex(timer => timer.id === id); if (at >= 0) timers.splice(at, 1); },
  });
  return { store, chat, asr, speech, invoke, advance, say: (text: string) => heard(text), emit: (event: string, payload?: unknown) => handlers.get(event)?.(payload), appearance };
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

  it.each([true, false])('creates capsule conversations in full trust (spoken: %s)', async spoken => {
    const { asr, speech } = setup();
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === 'chat_new_session' || command === 'chat_send') {
        return { id: 'trusted', permissionMode: command === 'chat_new_session' ? args?.permissionMode : 'auto' };
      }
      return undefined;
    });
    const chat = createChatStore({ desktop: true, invoke: invoke as never, listen: async () => () => {}, notify: vi.fn() });
    chat.configured = true;
    const store = createAssistantStore({ desktop: true, invoke: invoke as never, listen: async () => () => {}, chat, asr: asr as never, speech: speech as never, setTimer: vi.fn(), clearTimer: vi.fn() });
    await store.submit('帮我换个壁纸', spoken);
    expect(invoke).toHaveBeenCalledWith('chat_new_session', { permissionMode: 'auto' });
    expect(invoke).toHaveBeenCalledWith('chat_send', { sessionId: 'trusted', text: '帮我换个壁纸' });
    expect(chat.sessions[0].permissionMode).toBe('auto');
    store.dispose(); chat.dispose();
  });

  it('cancels a spoken turn before its first output without hiding or clearing the next draft',async()=>{
    const {store,chat}=setup();await store.opened();await store.submit('这个问题需要思考',true);
    expect(store.phase).toBe('thinking');store.input='下一句草稿';store.interrupt();
    expect(chat.cancel).toHaveBeenCalledOnce();expect(store.input).toBe('下一句草稿');expect(store.open).toBe(true);
    chat.views.s1.running=false;await nextTick();expect(store.phase).toBe('idle');store.dispose();
  });

  it('keeps the saved permission mode when continuing a conversation', async () => {
    const { store, chat, advance } = setup({ voice: false });
    await store.opened();
    await store.submit('你好');
    expect(chat.permissionMode).toBe('auto');
    finishAnswer(chat);
    chat.permissionMode = 'ask';
    store.closed();
    advance(1000);
    await store.opened();
    await store.submit('继续');
    expect(chat.permissionMode).toBe('ask');
    store.closed();
    advance(30*24*60*60*1000);
    await store.opened();
    await store.submit('继续旧会话');
    expect(chat.permissionMode).toBe('ask');
    store.dispose();
  });

  it('continues the assistant conversation regardless of time away', async () => {
    const { store, chat, advance } = setup({ voice: false });
    await store.opened();
    await store.submit('你好');
    store.closed();
    chat.select.mockClear();
    advance(1000);
    await store.opened();
    expect(chat.select).not.toHaveBeenCalled();
    store.closed();
    advance(30*24*60*60*1000);
    await store.opened();
    expect(chat.select).not.toHaveBeenCalled();
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
    expect(noVoice.store.hintTab).toBe('assistant');
    expect(noVoice.asr.activate).not.toHaveBeenCalled();
    await noVoice.store.openSettings('assistant');
    expect(noVoice.invoke).toHaveBeenCalledWith('assistant_open_settings', { tab: 'assistant' });
  });

  it('recovers from the cached unconfigured overview through the real chat store', async () => {
    let configured = false;
    const handlers = new Map<string, (payload: never) => void>();
    const listen = async (event: string, callback: (payload: never) => void) => { handlers.set(event, callback); return () => { handlers.delete(event); }; };
    const invoke = vi.fn(async (command: string) => {
      if (command === 'chat_overview') return { configured, sessions: [], running: [], workspace: '/work', permissionMode: 'ask' };
      if (command === 'chat_new_session' || command === 'chat_send') return { id: 's1', permissionMode: 'auto' };
      return undefined;
    });
    const { asr, speech } = setup({ voice: false });
    const chat = createChatStore({ desktop: true, invoke: invoke as never, listen, notify: vi.fn() });
    const store = createAssistantStore({ desktop: true, invoke: invoke as never, listen, chat, asr: asr as never, speech: speech as never, setTimer: vi.fn(), clearTimer: vi.fn() });
    await store.init(); await store.opened();
    expect(store.configured).toBe(false);
    expect(store.hintTab).toBe('ai');
    configured = true;
    // init is intentionally cached: the event must fetch a fresh overview instead.
    await chat.init();
    expect(store.configured).toBe(false);
    handlers.get('ai-config-changed')?.(undefined as never);
    await vi.waitFor(() => expect(store.configured).toBe(true));
    expect(store.hint).toBe('');
    await store.submit('模型配置好了', true);
    expect(invoke).toHaveBeenCalledWith('chat_new_session', { permissionMode: 'auto' });
    expect(invoke).toHaveBeenCalledWith('chat_send', { sessionId: 's1', text: '模型配置好了' });
    store.dispose(); chat.dispose();
  });

  it('refreshes after model setup and clears only the stale model hint without opening the microphone', async () => {
    const { store, chat, asr, emit } = setup({ configured: false });
    await store.init();
    await store.opened();
    expect(store.hintTab).toBe('ai');
    store.input = '先保留这句话';
    chat.refresh.mockImplementation(async () => { chat.configured = true; });
    emit('ai-config-changed');
    await vi.waitFor(() => expect(store.configured).toBe(true));
    expect(store.hint).toBe('');
    expect(store.hintTab).toBe('');
    expect(store.input).toBe('先保留这句话');
    expect(asr.activate).not.toHaveBeenCalled();
    await store.submit();
    expect(chat.send).toHaveBeenCalledWith('先保留这句话');
    store.note('语音识别暂时不可用');
    emit('ai-config-changed');
    await Promise.resolve();
    expect(store.hint).toBe('语音识别暂时不可用');
    store.dispose();
  });

  it('reads the latest model status whenever reopened even if the change event was missed', async () => {
    const { store, chat, asr } = setup({ configured: false });
    await store.opened();
    expect(store.configured).toBe(false);
    store.closed();
    chat.refresh.mockImplementation(async () => { chat.configured = true; });
    await store.opened();
    expect(chat.refresh).toHaveBeenCalledTimes(2);
    expect(store.configured).toBe(true);
    expect(store.hint).toBe('');
    expect(asr.activate).toHaveBeenCalledTimes(1);
    store.dispose();
  });

  it('preserves the conversation, draft and permission mode across configuration changes', async () => {
    const { store, chat, emit } = setup({ voice: false });
    await store.init(); await store.opened();
    await store.submit('你好');
    store.input = '下一句';
    const view = chat.views.s1;
    chat.refresh.mockImplementation(async () => { chat.permissionMode = 'ask'; });
    emit('ai-config-changed');
    await Promise.resolve();
    expect(chat.currentId).toBe('s1');
    expect(chat.permissionMode).toBe('auto');
    expect(chat.views.s1).toBe(view);
    expect(store.input).toBe('下一句');
    store.dispose();
  });

  it('keeps the question and shows model settings when the configuration is removed', async () => {
    const { store, chat, emit } = setup({ voice: false });
    await store.init(); await store.opened();
    chat.refresh.mockImplementation(async () => { chat.configured = false; });
    emit('ai-config-changed');
    await vi.waitFor(() => expect(store.configured).toBe(false));
    await store.submit('不要丢掉这个问题', true);
    expect(chat.send).not.toHaveBeenCalled();
    expect(store.input).toBe('不要丢掉这个问题');
    expect(store.hintTab).toBe('ai');
    store.dispose();
  });

  it('ignores configuration notifications after disposal', async () => {
    const { store, chat, emit } = setup();
    await store.init();
    store.dispose();
    emit('ai-config-changed');
    expect(chat.refresh).not.toHaveBeenCalled();
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
    await store.openChat();
    expect(invoke).toHaveBeenCalledWith('assistant_open_chat', { session: expect.anything() });
  });

  it('shows the ball when the host says so, and the capsule replaces it', async () => {
    const { store, emit } = setup({ voice: false });
    await store.init();
    expect(store.ball).toBe(false);
    emit('assistant-ball', true);
    expect(store.ball).toBe(true);
    emit('assistant-open');
    await vi.waitFor(() => expect(store.open).toBe(true));
    expect(store.ball).toBe(false);
    emit('assistant-close');
    expect(store.open).toBe(false);
    emit('assistant-ball', true);
    expect(store.ball).toBe(true);
    emit('assistant-ball', false);
    expect(store.ball).toBe(false);
  });

  it('tells the host to open the capsule when the ball is clicked', async () => {
    const { store, invoke } = setup({ voice: false });
    await store.unfold();
    expect(invoke).toHaveBeenCalledWith('assistant_unfold');
  });

  it('waits for typing when the user chose to type first, and listens when they tap the ball', async () => {
    const { store, asr } = setup({ prefs: { open: 'type', read: 'voice' } });
    await store.init();
    await store.opened();
    expect(asr.activate).not.toHaveBeenCalled();
    expect(store.phase).toBe('idle');
    await store.toggleListening();
    expect(asr.activate).toHaveBeenCalledTimes(1);
  });

  it('follows a change of the open and read choices made in the settings window', async () => {
    const { store, asr, emit } = setup();
    await store.init();
    expect(store.prefs).toEqual({ open: 'voice', read: 'voice' });
    emit('settings-changed', { assistantOpen: 'type', assistantRead: 'always', other: 1 });
    expect(store.prefs).toEqual({ open: 'type', read: 'always' });
    emit('settings-changed', { assistantOpen: 'nonsense', assistantRead: undefined });
    expect(store.prefs).toEqual({ open: 'type', read: 'always' });
    await store.opened();
    expect(asr.activate).not.toHaveBeenCalled();
  });

  it('reads answers aloud always, only after spoken questions, or never, as chosen', async () => {
    for (const [read, typedReads, spokenReads] of [['always', true, true], ['voice', false, true], ['never', false, false]] as const) {
      const typed = setup({ voice: false, prefs: { open: 'type', read } });
      await typed.store.init(); await typed.store.opened();
      typed.store.input = '打字问'; await typed.store.submit();
      finishAnswer(typed.chat); typed.store.replied('s1', '回答', 'a');
      expect(typed.speech.speak.mock.calls.length > 0, `typed, ${read}`).toBe(typedReads);

      const spoken = setup({ prefs: { open: 'voice', read } });
      await spoken.store.init(); await spoken.store.opened();
      spoken.say('说着问'); spoken.advance(SEND_GRACE_MS + 1);
      await vi.waitFor(() => expect(spoken.chat.send).toHaveBeenCalled());
      finishAnswer(spoken.chat); spoken.store.replied('s1', '回答', 'a');
      expect(spoken.speech.speak.mock.calls.length > 0, `spoken, ${read}`).toBe(spokenReads);
    }
  });

  it('takes the console\'s colour scheme from the host at start and follows every change, since it cannot read the console\'s storage', async () => {
    const { store, emit, appearance } = setup({ prefs: { open: 'voice', read: 'voice', appearance: 'dark' } });
    await store.init();
    expect(appearance).toHaveBeenLastCalledWith('dark');
    emit('settings-changed', { appearance: 'light' });
    expect(appearance).toHaveBeenLastCalledWith('light');
    emit('settings-changed', { assistantOpen: 'type' });
    expect(appearance).toHaveBeenCalledTimes(2);
  });
});

describe('assistant image messages', () => {
  it('sends image-only questions through the existing image chat transport', async () => {
    const { store, chat } = setup({ voice: false }); await store.opened();
    store.attachments = ['data:image/png;base64,AA==']; await store.submit();
    expect(chat.send).toHaveBeenCalledWith('', ['data:image/png;base64,AA==']);
    expect(store.attachments).toEqual([]); store.dispose();
  });
  it('restores text and images if the host refuses the message', async () => {
    const { store, chat } = setup({ voice: false }); await store.opened();
    chat.send.mockResolvedValueOnce(false); store.input = '看看这个'; store.attachments = ['image'];
    await store.submit(); expect(store.input).toBe('看看这个'); expect(store.attachments).toEqual(['image']); store.dispose();
  });
  it('keeps the image draft if a model is missing and clears it on close', async () => {
    const { store, chat } = setup({ voice: false, configured: false }); await store.opened(); store.attachments = ['image'];
    await store.submit(); expect(chat.send).not.toHaveBeenCalled(); expect(store.attachments).toEqual(['image']);
    store.closed(); expect(store.attachments).toEqual([]); store.dispose();
  });
  it('pauses voice auto-send when the user starts attaching an image', async () => {
    const { store, chat, asr, say, advance } = setup(); await store.opened(); say('看看这张图');
    store.prepareImages(); advance(SEND_GRACE_MS + 1); await Promise.resolve();
    expect(chat.send).not.toHaveBeenCalled(); expect(asr.recordingRequested).toBe(false); expect(store.input).toBe('看看这张图'); store.dispose();
  });
  it('ignores a second send while the first image message is being submitted', async () => {
    const { store, chat, speech } = setup({ voice: false }); await store.opened();
    let finish!: () => void; speech.stop.mockImplementationOnce(() => new Promise<void>(resolve => { finish = resolve; }));
    store.attachments = ['image']; const sent = store.submit(); await store.submit('duplicate'); finish(); await sent;
    expect(chat.send).toHaveBeenCalledTimes(1); store.dispose();
  });
});

describe('assistant conversation persistence',()=>{
  const memory=()=>{const data=new Map<string,string>();return {getItem:(k:string)=>data.get(k)??null,setItem:(k:string,v:string)=>{data.set(k,v);},removeItem:(k:string)=>{data.delete(k);}};};
  it('restores the assistant conversation after recreating its window store',async()=>{
    const storage=memory(),first=setup({voice:false,storage});await first.store.opened();await first.store.submit('记住这个上下文');
    expect(storage.getItem(SESSION_KEY)).toBe('s1');first.store.dispose();
    const next=setup({voice:false,storage});next.chat.sessions=[...first.chat.sessions];next.chat.views=first.chat.views;
    await next.store.opened();expect(next.chat.select).toHaveBeenCalledWith('s1');expect(next.store.question).toBe('记住这个上下文');next.store.dispose();
  });
  it('starts a new persistent conversation explicitly and keeps the old history',async()=>{
    const storage=memory(),h=setup({voice:false,storage});await h.store.opened();await h.store.submit('旧问题');
    h.store.input='旧草稿';h.store.attachments=['image'];await h.store.newConversation();
    expect(h.chat.cancel).toHaveBeenCalledOnce();expect(h.chat.currentId).toBe('fresh-1');expect(storage.getItem(SESSION_KEY)).toBe('fresh-1');
    expect(h.chat.views.s1.items[0].text).toBe('旧问题');expect(h.store.input).toBe('');expect(h.store.attachments).toEqual([]);
    await h.store.submit('新问题');expect(h.chat.currentId).toBe('fresh-1');h.store.closed();await h.store.opened();expect(h.chat.currentId).toBe('fresh-1');h.store.dispose();
  });
  it('keeps the current conversation and draft if creation fails',async()=>{
    const storage=memory(),h=setup({voice:false,storage});await h.store.opened();await h.store.submit('旧问题');h.store.input='保留';h.store.attachments=['image'];
    h.invoke.mockRejectedValueOnce(new Error('创建失败'));await h.store.newConversation();
    expect(h.chat.currentId).toBe('s1');expect(storage.getItem(SESSION_KEY)).toBe('s1');expect(h.store.input).toBe('保留');expect(h.store.attachments).toEqual(['image']);expect(h.store.changingSession).toBe(false);h.store.dispose();
  });
  it('drops a pointer to a deleted conversation without choosing another chat',async()=>{
    const storage=memory();storage.setItem(SESSION_KEY,'deleted');const h=setup({voice:false,storage});h.chat.sessions=[{id:'unrelated'}];
    await h.store.opened();expect(h.chat.currentId).toBeNull();expect(storage.getItem(SESSION_KEY)).toBeNull();expect(h.invoke).not.toHaveBeenCalledWith('chat_new_session',expect.anything());h.store.dispose();
  });
});
