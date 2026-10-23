import { nextTick, reactive } from 'vue';
import { flushPromises } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { createAssistantStore, LISTEN_IDLE_MS, SESSION_KEY, SEND_GRACE_MS, SETTLED_GRACE_MS } from './assistant-store.ts';
import { ACK_DELAY_MS, ACK_PHRASES } from '../voice/ack.ts';
import { createChatStore } from '../ui/chat/chat-store.ts';
import type { AsrActivationOptions, TranscriptContext } from '../ui/asr/store.ts';
import { createAsrStore } from '../ui/asr/store.ts';
import { createSpeechStore, speechDefaults } from '../ui/speech/store.ts';

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
  let heard: (text: string, context?: TranscriptContext) => void = () => {};
  let captureOptions: AsrActivationOptions = {};
  const asr = reactive({
    enabled: options.voice ?? true, microphoneSupported: true, microphoneError: '', warming: false, listening: false, recognizing: false, preparing: false, recordingRequested: false, level: 0, error: '',
    init: vi.fn(async () => {}), prepare: vi.fn(async () => true),
    activate: vi.fn(async (callback: typeof heard, options?: AsrActivationOptions) => { asr.recordingRequested = true; asr.listening = true; heard = callback; captureOptions = options ?? {}; }),
    finish: vi.fn(async () => { asr.recordingRequested = false; asr.listening = false; }),
    deactivate: vi.fn(() => { asr.recordingRequested = false; asr.listening = false; }),
  });
  const speech = reactive({
    status: 'idle' as 'idle' | 'synthesizing' | 'playing', settings: { autoRead: false },
    init: vi.fn(async () => {}),
    speak: vi.fn(async () => { speech.status = 'playing'; }),
    stop: vi.fn(async () => { speech.status = 'idle'; }),
    ack: vi.fn(() => true), ackReady: vi.fn(() => false), prepareAck: vi.fn(async () => {}),
    stream: undefined as any,
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
  return { store, chat, asr, speech, invoke, advance, say: (text: string, context?: TranscriptContext) => heard(text, context), preview: (text: string, playbackText = captureOptions.playbackText?.() ?? '') => captureOptions.onPreview?.(text, {playbackText}), playback: () => captureOptions.playbackText?.() ?? '', emit: (event: string, payload?: unknown) => handlers.get(event)?.(payload), appearance };
}
const finishAnswer = (chat: ReturnType<typeof setup>['chat'], text = '好的，已经处理完了。') => {
  chat.views.s1.items.push({ kind: 'assistant', id: 'a', text, streaming: false });
  chat.views.s1.running = false;
};

describe('truthful voice prompts', () => {
  it('only announces listening after capture starts and returns to typing after stopping', async () => {
    const {store,asr}=setup({prefs:{open:'type',read:'never'}});await store.init();await store.opened();
    expect(store.status).toBe('问点什么…');
    let captured!:()=>void;const pending=new Promise<void>(resolve=>{captured=resolve;});
    asr.activate.mockImplementation(async()=>{asr.recordingRequested=true;await pending;asr.listening=true;});
    const starting=store.toggleListening();await flushPromises();
    expect(store.phase).toBe('idle');expect(store.status).toBe('问点什么…');expect(store.voiceStatus).toBe('正在启动麦克风…');
    captured();await starting;
    expect(store.phase).toBe('listening');expect(store.status).toBe('正在聆听…');
    await store.toggleListening();expect(store.status).toBe('问点什么…');store.dispose();
  });
  it('keeps preparation separate and cancels pending voice activation when the user types',async()=>{
    const {store,asr}=setup({prefs:{open:'type',read:'never'}});await store.init();await store.opened();
    let ready!:(value:boolean)=>void;const pending=new Promise<boolean>(resolve=>{ready=resolve;});
    asr.prepare.mockImplementation(async()=>{asr.preparing=true;return pending;});
    const starting=store.toggleListening();await flushPromises();
    expect(store.status).toBe('问点什么…');expect(store.phase).toBe('idle');expect(store.voiceStatus).toContain('正在准备');
    store.input='继续用文字提问';store.inputStarted();asr.preparing=false;ready(true);await starting;
    expect(asr.activate).not.toHaveBeenCalled();expect(store.input).toBe('继续用文字提问');store.dispose();
  });
  it('expires microphone failures without replacing the input or model hint',async()=>{
    const {store,asr,advance}=setup({prefs:{open:'type',read:'never'}});await store.init();await store.opened();
    store.input='保留我的草稿';store.note('模型设置提示','ai');asr.microphoneError='麦克风不可用';
    expect(store.voiceStatus).toBe('麦克风不可用');expect(store.status).toBe('问点什么…');
    advance(5001);expect(store.voiceStatus).toBe('');expect(store.input).toBe('保留我的草稿');expect(store.hint).toBe('模型设置提示');store.dispose();
  });
  it('keeps unsupported devices in typing mode with a temporary explanation',async()=>{
    const {store,asr,advance}=setup({prefs:{open:'type',read:'never'}});asr.microphoneSupported=false;await store.opened();await store.toggleListening();
    expect(store.status).toBe('问点什么…');expect(store.voiceStatus).toContain('不支持语音输入');expect(asr.activate).not.toHaveBeenCalled();
    advance(5001);expect(store.voiceStatus).toBe('');store.dispose();
  });
});

describe('assistant barge-in', () => {
  it('integrates microphone, partial ASR, echo rejection and immediate audio pause',async()=>{
    const h=setup();h.store.dispose();
    let feed!:(audio:Float32Array)=>void, transcript='山上有座庙';const releaseMic=vi.fn();
    const invoke=vi.fn(async(command:string)=>{
      if(command==='assistant_prefs')return {open:'type',read:'always'};
      if(command==='asr_get_status')return {enabled:true,modelReady:true,engineReady:true};
      if(command==='asr_transcribe')return transcript;
      if(command==='tts_get_settings')return {...speechDefaults};
      if(command==='tts_synthesize'||command==='tts_synthesize_part')return [1,2,3];
      return undefined;
    });
    const notify=vi.fn(), listen=async()=>()=>{};
    const asr=createAsrStore({desktop:true,invoke:invoke as never,listen,notify,capture:async cb=>{feed=cb;return {stop:releaseMic};}});
    const audio={play:vi.fn(async()=>{}),pause:vi.fn(),src:'',onended:null,onerror:null};
    const speech=createSpeechStore({desktop:true,invoke:invoke as never,notify,audio:()=>audio,url:()=> 'blob:test',revoke:vi.fn()});
    const store=createAssistantStore({desktop:true,invoke:invoke as never,listen,chat:h.chat as never,asr,speech,setTimer:vi.fn(),clearTimer:vi.fn()});
    await store.init();await store.opened();await store.submit('讲个故事',true);finishAnswer(h.chat,story);store.replied('s1',story,'a');
    await vi.waitFor(()=>{expect(speech.status).toBe('playing');expect(asr.listening).toBe(true);});
    const voice=Float32Array.from({length:1600},(_,i)=>.08*Math.sin(2*Math.PI*220*i/16000));
    for(let i=0;i<8;i++)feed(voice);await flushPromises();
    expect(speech.status).toBe('playing');expect(store.input).toBe('');
    transcript='等等，换一个科幻故事';
    for(let i=0;i<8;i++)feed(voice);await flushPromises();
    expect(speech.status).toBe('idle');expect(audio.pause).toHaveBeenCalledOnce();expect(asr.listening).toBe(true);
    for(let i=0;i<9;i++)feed(new Float32Array(1600));await flushPromises();
    expect(store.input).toBe(transcript);expect(releaseMic).not.toHaveBeenCalled();
    store.inputStarted();expect(asr.recordingRequested).toBe(false);expect(releaseMic).toHaveBeenCalledOnce();
    store.dispose();asr.dispose();speech.dispose();
  });
  const story = '从前有座山，山上有座庙，庙里有人讲故事。';
  async function reading() {
    const h=setup({prefs:{open:'type',read:'always'}});
    await h.store.init(); await h.store.opened(); await h.store.submit('讲一个故事',true);
    finishAnswer(h.chat,story); h.store.replied('s1',story,'a');
    await vi.waitFor(()=>expect(h.asr.recordingRequested).toBe(true));
    h.speech.stop.mockClear(); h.asr.deactivate.mockClear();
    return h;
  }
  it('keeps reading when partial and final recognition contain its own readout',async()=>{
    const h=await reading(); expect(h.playback()).toBe(story);
    h.preview('山上有座庙'); h.say('庙里有人讲故事',{playbackText:story});
    h.advance(SEND_GRACE_MS+1);await nextTick();
    expect(h.store.phase).toBe('speaking');expect(h.speech.stop).not.toHaveBeenCalled();
    expect(h.store.input).toBe('');expect(h.chat.send).toHaveBeenCalledTimes(1);h.store.dispose();
  });
  it('stops on new speech without discarding the utterance, then sends the whole question',async()=>{
    const h=await reading();
    h.preview('等等，我想换一个问题');await nextTick();
    expect(h.speech.stop).toHaveBeenCalledOnce();expect(h.asr.deactivate).not.toHaveBeenCalled();
    expect(h.store.phase).toBe('listening');expect(h.store.input).toBe('');
    h.say('等等，我想换一个问题，明天会下雨吗',{playbackText:story});
    h.advance(SEND_GRACE_MS+1);await Promise.resolve();
    // The model is told, out of the user's sight, that the answer was cut off.
    expect(h.chat.send).toHaveBeenLastCalledWith('等等，我想换一个问题，明天会下雨吗', [], expect.stringContaining('【语音打断】'));h.store.dispose();
  });
  it('filters a delayed echo result after playback ends while retaining new speech',async()=>{
    const h=await reading(); h.speech.status='idle';await nextTick();
    h.say('山上有座庙',{playbackText:story});expect(h.store.input).toBe('');
    h.say('帮我改成一个科幻故事',{playbackText:story});
    expect(h.store.input).toBe('帮我改成一个科幻故事');h.store.dispose();
  });
  it('manual editing preserves readout and the draft, then sending stops readout',async()=>{
    const h=await reading();h.store.input='新的问题';h.store.inputStarted();await nextTick();
    expect(h.speech.stop).not.toHaveBeenCalled();expect(h.asr.recordingRequested).toBe(false);
    h.say('旧识别结果');h.advance(LISTEN_IDLE_MS*2);await nextTick();
    expect(h.store.input).toBe('新的问题');expect(h.store.phase).toBe('speaking');
    expect(h.chat.send).toHaveBeenCalledTimes(1);
    await h.store.submit();expect(h.speech.stop).toHaveBeenCalledOnce();expect(h.chat.send).toHaveBeenLastCalledWith('新的问题');h.store.dispose();
  });
  it('typing cancels delayed microphone preparation but lets synthesis continue',async()=>{
    const h=setup({prefs:{open:'type',read:'always'}});await h.store.init();await h.store.opened();await h.store.submit('你好');finishAnswer(h.chat);
    let resolve!:(value:boolean)=>void;h.asr.prepare.mockImplementationOnce(()=>new Promise<boolean>(r=>{resolve=r;}));
    h.speech.speak.mockImplementationOnce(async()=>{h.speech.status='synthesizing';});
    h.store.replied('s1','你好呀','a');await nextTick();
    h.store.inputStarted();resolve(true);await nextTick();await nextTick();
    expect(h.asr.activate).not.toHaveBeenCalled();expect(h.speech.status).toBe('synthesizing');
    h.store.input='新的问题';await h.store.submit();expect(h.speech.status).toBe('idle');h.store.dispose();
  });
  it('does not open the microphone for readout when voice is disabled',async()=>{
    const h=setup({voice:false,prefs:{open:'type',read:'always'}});await h.store.init();await h.store.opened();await h.store.submit('你好');finishAnswer(h.chat);
    h.store.replied('s1','你好呀','a');await nextTick();expect(h.speech.status).toBe('playing');
    expect(h.asr.activate).not.toHaveBeenCalled();h.store.inputStarted();expect(h.speech.status).toBe('playing');
    h.store.input='新的问题';await h.store.submit();expect(h.speech.status).toBe('idle');h.store.dispose();
  });
  it('does not open a late monitoring microphone after a typed answer finishes',async()=>{
    const h=setup({prefs:{open:'type',read:'always'}});await h.store.init();await h.store.opened();await h.store.submit('你好');finishAnswer(h.chat);
    let resolve!:(value:boolean)=>void;h.asr.prepare.mockImplementationOnce(()=>new Promise<boolean>(r=>{resolve=r;}));
    h.store.replied('s1','你好呀','a');await nextTick();h.speech.status='idle';await nextTick();
    resolve(true);await nextTick();await nextTick();expect(h.asr.activate).not.toHaveBeenCalled();h.store.dispose();
  });
  it('releases the monitoring microphone when a typed readout finishes normally',async()=>{
    const h=setup({prefs:{open:'type',read:'always'}});await h.store.init();await h.store.opened();await h.store.submit('你好');finishAnswer(h.chat);
    h.store.replied('s1','你好呀','a');await vi.waitFor(()=>expect(h.asr.listening).toBe(true));
    h.speech.status='idle';await nextTick();expect(h.asr.finish).toHaveBeenCalledOnce();expect(h.asr.recordingRequested).toBe(false);h.store.dispose();
  });
});

describe('floating assistant', () => {
  it('silently falls back to typing when microphone capture is unsupported', async () => {
    const { store, asr } = setup();
    asr.microphoneSupported = false;
    await store.opened();
    await store.toggleListening();
    expect(store.hint).toBe('');
    expect(store.voiceAvailable).toBe(false);
    expect(store.status).toBe('问点什么…');
    expect(asr.prepare).not.toHaveBeenCalled();
    expect(asr.activate).not.toHaveBeenCalled();
    store.dispose();
  });
  it('starts a new conversation when opened, and listens when voice is on', async () => {
    const { store, chat, asr } = setup();
    await store.opened();
    expect(store.open).toBe(true);
    expect(chat.select).toHaveBeenCalledWith(null);
    expect(asr.activate).toHaveBeenCalledTimes(1);
    expect(store.phase).toBe('listening');
    expect(store.status).toBe('正在聆听…');
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
    expect(noVoice.store.hint).toBe('');
    expect(noVoice.invoke).toHaveBeenCalledWith('assistant_open_settings', { tab: 'assistant' });
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
    expect(store.voiceStatus).toContain('没听到声音');
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

  it('preserves the open draft when the host re-announces the capsule after showing its window', async () => {
    const { store, emit, asr } = setup({ prefs: { open: 'type', read: 'never' } });
    await store.init();
    emit('assistant-ball', true);
    emit('assistant-open');
    await flushPromises();
    expect(store.open).toBe(true);
    store.input = '还没有发送的消息';
    const deactivations = asr.deactivate.mock.calls.length;
    emit('assistant-open');
    await flushPromises();
    expect(store.open).toBe(true);
    expect(store.input).toBe('还没有发送的消息');
    expect(asr.deactivate.mock.calls.length).toBe(deactivations);
    emit('assistant-ball', true);
    emit('assistant-close');
    expect(store.ball).toBe(true);
    expect(store.open).toBe(false);
  });

  it('tells the host to open the capsule when the ball is clicked', async () => {
    const { store, invoke } = setup({ voice: false });
    await store.unfold();
    expect(invoke).toHaveBeenCalledWith('assistant_unfold', { peek: false });
    await store.unfold(true);
    expect(invoke).toHaveBeenCalledWith('assistant_unfold', { peek: true });
  });

  it('saves an answer to 资料库 with the conversation it came from, and says so when it cannot', async () => {
    const { store, invoke } = setup({ voice: false });
    await store.init();
    expect(await store.saveAnswer('结论在前')).toBe(true);
    expect(invoke).toHaveBeenCalledWith('library_save', { item: expect.objectContaining({ kind: 'answer', text: '结论在前' }) });
    invoke.mockImplementationOnce(async () => { throw new Error('资料库无法保存'); });
    expect(await store.saveAnswer('再存一次')).toBe(false);
  });

  it('opens for a glance without listening, and takes the focus once used', async () => {
    const { store, asr, invoke, emit } = setup();
    await store.init();
    emit('assistant-stage', { ball: { x: 408, y: 520, size: 72 }, right: true });
    expect(store.layout).toEqual({ ball: { x: 408, y: 520, size: 72 }, right: true });
    emit('assistant-open', { peek: true });
    await vi.waitFor(() => expect(store.open).toBe(true));
    expect(store.peek).toBe(true);
    await flushPromises();
    expect(asr.activate).not.toHaveBeenCalled();
    store.engage();
    expect(store.peek).toBe(false);
    expect(invoke).toHaveBeenCalledWith('assistant_focus');
    emit('assistant-close', null);
    expect(store.peek).toBe(false);
  });

  it('passes on where the pointer is and asks before the window moves under the ball', async () => {
    const { store, invoke, emit } = setup();
    await store.init();
    emit('assistant-hover', true);
    expect(store.hovering).toBe(true);
    emit('assistant-reanchor', { id: 3 });
    expect(store.reanchoring).toBe(3);
    store.reanchorReady(2);
    expect(invoke).not.toHaveBeenCalledWith('assistant_reanchor_ready', { id: 2 });
    store.reanchorReady(3);
    expect(invoke).toHaveBeenCalledWith('assistant_reanchor_ready', { id: 3 });
    emit('assistant-reanchor-done', { id: 3 });
    expect(store.reanchoring).toBe(0);
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
    expect(store.prefs).toEqual({ open: 'voice', read: 'voice', hover: true });
    emit('settings-changed', { assistantOpen: 'type', assistantRead: 'always', assistantHover: false, other: 1 });
    expect(store.prefs).toEqual({ open: 'type', read: 'always', hover: false });
    emit('settings-changed', { assistantOpen: 'nonsense', assistantRead: undefined });
    expect(store.prefs).toEqual({ open: 'type', read: 'always', hover: false });
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
  it('keeps the image draft if a model is missing, and across folding away and back', async () => {
    const { store, chat } = setup({ voice: false, configured: false }); await store.opened(); store.attachments = ['image']; store.input = '看看这个';
    await store.submit(); expect(chat.send).not.toHaveBeenCalled(); expect(store.attachments).toEqual(['image']);
    store.closed(); expect(store.attachments).toEqual(['image']); expect(store.input).toBe('看看这个');
    await store.opened(); expect(store.attachments).toEqual(['image']); expect(store.input).toBe('看看这个'); store.dispose();
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

describe('interrupting assistant output',()=>{
  it('stops simultaneous speech and generation, preserves text and draft, and suppresses late reading',async()=>{
    const {store,chat,speech,asr}=setup({prefs:{open:'type',read:'always'}});await store.init();await store.opened();await store.submit('你好');
    chat.views.s1.items.push({kind:'assistant',id:'a',text:'已经生成的文字'});
    store.replied('s1','已经生成的文字','a');await nextTick();expect(store.phase).toBe('speaking');
    store.input='下一句草稿';speech.stop.mockClear();asr.activate.mockClear();store.interrupt();await nextTick();
    expect(chat.cancel).toHaveBeenCalledOnce();expect(speech.stop).toHaveBeenCalledOnce();expect(store.answer).toBe('已经生成的文字');expect(store.input).toBe('下一句草稿');
    expect(asr.activate).not.toHaveBeenCalled();speech.speak.mockClear();store.replied('s1','延迟结束的回复','late');expect(speech.speak).not.toHaveBeenCalled();
    chat.views.s1.running=false;await store.submit('继续');finishAnswer(chat,'新一轮');store.replied('s1','新一轮','b');expect(speech.speak).toHaveBeenCalledWith('新一轮','b',undefined,true);
  });
  it('stopping voice playback returns to input without automatically reopening the microphone',async()=>{
    const {store,chat,speech,asr}=setup();await store.opened();await store.submit('讲个故事',true);finishAnswer(chat,'从前有座山');store.replied('s1','从前有座山','a');await nextTick();asr.activate.mockClear();
    store.interrupt();await nextTick();expect(store.phase).toBe('idle');expect(asr.activate).not.toHaveBeenCalled();expect(speech.status).toBe('idle');
  });
});

describe('assistant voice settings synchronization',()=>{
  it('cancels the pending automatic send when settings disables voice, preserving the draft',async()=>{
    const {store,asr,chat,say,advance}=setup();await store.opened();say('还没有发送的话');expect(store.input).toBe('还没有发送的话');
    asr.enabled=false;advance(SEND_GRACE_MS+1);await nextTick();
    expect(chat.send).not.toHaveBeenCalled();expect(asr.recordingRequested).toBe(false);expect(store.phase).toBe('idle');expect(store.input).toBe('还没有发送的话');
  });
});

describe('while only the ball shows', () => {
  it('marks a finished answer unread and says what it was', async () => {
    const { store, chat } = setup({ voice: false }); await store.init(); await store.opened();
    await store.submit('写个周报'); store.closed();
    finishAnswer(chat, '# 周报初稿\n\n本周完成了三件事'); store.replied('s1', '周报初稿');
    expect(store.unread).toBe(true); expect(store.notice).toEqual({ kind: 'done', text: '周报初稿' });
    await store.opened(); expect(store.unread).toBe(false); expect(store.notice).toBeNull(); store.dispose();
  });
  it('says when something waits for the user', async () => {
    const { store, chat } = setup({ voice: false }); await store.init(); await store.opened();
    await store.submit('整理桌面'); store.closed();
    chat.views.s1.approvals.push({ id: 'p1', session: 's1', tool: 'move_files', arguments: {} }); await nextTick();
    expect(store.notice).toEqual({ kind: 'attention', text: 'move_files 需要你许可' }); store.dispose();
  });
  it('lists every turn and the latest conversations, and switches between them', async () => {
    const { store, chat } = setup({ voice: false }); await store.init(); await store.opened();
    await store.submit('你好'); finishAnswer(chat);
    expect(store.items.map(item => item.kind)).toEqual(['user', 'assistant']);
    chat.sessions = [{ id: 's1', title: '打招呼', updatedAt: 1 }, { id: 's2', title: '周报', updatedAt: 5 }];
    expect(store.recent.map(session => session.id)).toEqual(['s2', 's1']); expect(store.title).toBe('打招呼');
    await store.switchSession('s2'); expect(chat.select).toHaveBeenLastCalledWith('s2'); expect(store.currentId).toBe('s2'); store.dispose();
  });
});

describe('push to talk', () => {
  it('listens while the shortcut is held and sends what was heard on release, never on a pause', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } }); await h.store.init(); await h.store.opened();
    expect(h.asr.activate).not.toHaveBeenCalled();
    await h.store.pushToTalk(true); expect(h.asr.activate).toHaveBeenCalledOnce();
    expect(h.store.voiceStatus).toBe('正在听，松开快捷键发送');
    h.say('明天十点提醒我开会'); h.advance(SEND_GRACE_MS * 4); await Promise.resolve();
    expect(h.chat.send).not.toHaveBeenCalled();
    await h.store.pushToTalk(false);
    expect(h.asr.finish).toHaveBeenCalled(); expect(h.chat.send).toHaveBeenCalledWith('明天十点提醒我开会'); h.store.dispose();
  });
  it('starts once the capsule has opened when the shortcut is held from the ball', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } }); await h.store.init();
    await h.store.pushToTalk(true); expect(h.asr.activate).not.toHaveBeenCalled();
    await h.store.opened(); expect(h.asr.activate).toHaveBeenCalledOnce(); h.store.dispose();
  });
  it('says so when nothing was heard, and sends nothing', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } }); await h.store.init(); await h.store.opened();
    await h.store.pushToTalk(true); await h.store.pushToTalk(false);
    expect(h.chat.send).not.toHaveBeenCalled(); expect(h.store.voiceStatus).toContain('没听清'); h.store.dispose();
  });
});

describe('reading along while the answer is written', () => {
  function withStream(h: ReturnType<typeof setup>) {
    const pushed: string[] = []; const end = vi.fn(async () => {}); let events: { onStart?(): void } = {};
    (h.speech as Record<string, unknown>).stream = vi.fn((_key: string, _options: unknown, e: { onStart?(): void }) => { events = e; return { push: (t: string) => pushed.push(t), end, spoken: '', active: true }; });
    return { pushed, end, start: () => events.onStart?.() };
  }
  it('reads the first sentence before the answer is complete and finishes the rest after it', async () => {
    const h = setup({ prefs: { open: 'type', read: 'always' } }); const s = withStream(h);
    await h.store.init(); await h.store.opened(); await h.store.submit('讲个故事');
    h.chat.views.s1.items.push({ kind: 'assistant', id: 'a', text: '从前有座山。', streaming: true }); await nextTick();
    expect(s.pushed.at(-1)).toBe('从前有座山。');
    s.start(); expect(h.store.phase).toBe('speaking');
    await vi.waitFor(() => expect(h.asr.activate).toHaveBeenCalled());
    h.chat.views.s1.items[1].text = '从前有座山。山上有座庙。'; h.chat.views.s1.running = false;
    h.store.replied('s1', '从前有座山。山上有座庙。');
    expect(s.end).toHaveBeenCalledWith('从前有座山。山上有座庙。'); expect(h.speech.speak).not.toHaveBeenCalled(); h.store.dispose();
  });
  it('stops the answer being written when the user talks over it, keeping the microphone that heard them', async () => {
    const h = setup({ prefs: { open: 'type', read: 'always' } }); const s = withStream(h);
    await h.store.init(); await h.store.opened(); await h.store.submit('讲个故事');
    h.chat.views.s1.items.push({ kind: 'assistant', id: 'a', text: '从前有座山。', streaming: true }); await nextTick();
    s.start(); await vi.waitFor(() => expect(h.asr.activate).toHaveBeenCalled());
    h.say('等一下，换一个');
    expect(h.speech.stop).toHaveBeenCalled(); expect(h.chat.cancel).toHaveBeenCalledOnce();
    h.asr.deactivate.mockClear(); h.store.stopped();
    expect(h.asr.deactivate).not.toHaveBeenCalled(); expect(h.asr.recordingRequested).toBe(true);
    expect(h.store.input).toBe('等一下，换一个'); h.store.dispose();
  });
});

describe('answering faster', () => {
  it('sends sooner when the speaker was judged to have finished', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } });
    await h.store.init(); await h.store.opened(); await h.store.toggleListening();
    h.say('明天会下雨吗', { playbackText: '', settled: true });
    h.advance(SETTLED_GRACE_MS + 1); await flushPromises();
    expect(h.chat.send).toHaveBeenCalledWith('明天会下雨吗');
    expect(SETTLED_GRACE_MS).toBeLessThan(SEND_GRACE_MS);
    h.store.dispose();
  });
  it('still waits the full grace when the pause was only a long silence', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } });
    await h.store.init(); await h.store.opened(); await h.store.toggleListening();
    h.say('明天会下雨吗');
    h.advance(SETTLED_GRACE_MS + 1); await flushPromises();
    expect(h.chat.send).not.toHaveBeenCalled();
    h.advance(SEND_GRACE_MS); await flushPromises();
    expect(h.chat.send).toHaveBeenCalledWith('明天会下雨吗');
    h.store.dispose();
  });
  it('does not take a listening sound for a question', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } });
    await h.store.init(); await h.store.opened(); await h.store.toggleListening();
    h.say('嗯'); h.say('哈哈哈');
    h.advance(SEND_GRACE_MS + 1); await flushPromises();
    expect(h.store.input).toBe(''); expect(h.chat.send).not.toHaveBeenCalled();
    h.store.dispose();
  });
});

describe('saying something first', () => {
  async function asked(text = '帮我整理一下下载文件夹', spoken = true) {
    const h = setup({ prefs: { open: 'type', read: 'voice' } });
    h.speech.ackReady.mockReturnValue(true); // the phrases are prepared
    await h.store.init(); await h.store.opened();
    await h.store.submit(text, spoken);
    return h;
  }
  const said = (h: Awaited<ReturnType<typeof asked>>) => (h.speech.ack.mock.calls as unknown as string[][]).map(call => call[0]);
  const noteOf = (h: Awaited<ReturnType<typeof asked>>) => (h.chat.send.mock.calls.at(-1) as unknown as unknown[])[2] as string | undefined;

  it('takes up a request after a short wait, and tells the model what was said', async () => {
    const h = await asked();
    expect(h.speech.prepareAck).toHaveBeenCalled();
    h.advance(ACK_DELAY_MS - 1); expect(h.speech.ack).not.toHaveBeenCalled();
    h.advance(2); expect(said(h)).toHaveLength(1); expect(ACK_PHRASES.task).toContain(said(h)[0]);
    expect(noteOf(h)).toContain(`「${said(h)[0]}」`); expect(noteOf(h)).toContain('不要重复');
    h.store.dispose();
  });
  it('answers a question with "I\'ll think", and a greeting at once with a greeting', async () => {
    const question = await asked('明天北京会下雨吗');
    question.advance(ACK_DELAY_MS + 1); expect(ACK_PHRASES.question).toContain(said(question)[0]); question.store.dispose();
    for (const [text, kind] of [['你好', 'hello'], ['谢谢你', 'thanks'], ['晚安', 'goodnight']] as const) {
      const social = await asked(text);
      social.advance(0); // no wait: a greeting is answered at once
      expect(ACK_PHRASES[kind]).toContain(said(social)[0]);
      expect(noteOf(social)).toContain(`「${said(social)[0]}」`);
      social.store.dispose();
    }
  });
  it('says nothing to a feeling, a plain statement or bare agreement, and sends no note', async () => {
    for (const text of ['今天好累', '今天天气不错', '好的']) {
      const h = await asked(text);
      h.advance(ACK_DELAY_MS + 1); expect(h.speech.ack).not.toHaveBeenCalled(); expect(h.chat.send).toHaveBeenLastCalledWith(text); h.store.dispose();
    }
  });
  it('says nothing, and claims nothing, when the phrase is not ready to be played', async () => {
    const h = setup({ prefs: { open: 'type', read: 'voice' } });
    await h.store.init(); await h.store.opened(); await h.store.submit('你好', true);
    h.advance(ACK_DELAY_MS + 1); expect(h.speech.ack).not.toHaveBeenCalled(); expect(h.chat.send).toHaveBeenLastCalledWith('你好');
    h.store.dispose();
  });
  it('says nothing when the answer has already started', async () => {
    const h = await asked();
    h.chat.views.s1.items.push({ kind: 'assistant', id: 'a', text: '好，我先看一下', streaming: true });
    h.advance(ACK_DELAY_MS + 1);
    expect(h.speech.ack).not.toHaveBeenCalled();
    h.store.dispose();
  });
  it('says nothing for a typed question, once the answer is done, or after being interrupted', async () => {
    const typed = await asked('你好', false);
    typed.advance(ACK_DELAY_MS + 1); expect(typed.speech.ack).not.toHaveBeenCalled(); typed.store.dispose();
    const done = await asked();
    finishAnswer(done.chat); done.advance(ACK_DELAY_MS + 1); expect(done.speech.ack).not.toHaveBeenCalled(); done.store.dispose();
    const stopped = await asked();
    stopped.store.interrupt(); stopped.advance(ACK_DELAY_MS + 1); expect(stopped.speech.ack).not.toHaveBeenCalled(); stopped.store.dispose();
  });
  it('says nothing when reading is off', async () => {
    const h = setup({ prefs: { open: 'type', read: 'never' } });
    h.speech.ackReady.mockReturnValue(true);
    await h.store.init(); await h.store.opened(); await h.store.submit('你好', true);
    h.advance(ACK_DELAY_MS + 1); expect(h.speech.ack).not.toHaveBeenCalled(); expect(h.speech.prepareAck).not.toHaveBeenCalled();
    h.store.dispose();
  });
});

describe('remembering how far the user listened', () => {
  async function interrupted() {
    const h = setup({ prefs: { open: 'type', read: 'voice' } });
    let started!: () => void;
    h.speech.stream = vi.fn((_key: string, _options: unknown, events: { onStart?: () => void }) => {
      started = () => events.onStart?.();
      return { push: vi.fn(), end: vi.fn(async () => {}), spoken: '从前有座山，山上有座庙。', active: true };
    });
    await h.store.init(); await h.store.opened(); await h.store.submit('讲个故事', true);
    started(); await flushPromises();
    return h;
  }
  it('tells the model, out of sight, what was heard when the user talks over the answer', async () => {
    const h = await interrupted();
    h.preview('等等，换一个'); await nextTick();
    h.chat.views.s1.running = false; // the cancelled answer has stopped
    h.say('等等，换一个科幻的', { playbackText: '' });
    h.advance(SEND_GRACE_MS + 1); await flushPromises();
    const [text, , note] = h.chat.send.mock.calls.at(-1) as unknown as [string, unknown[], string];
    expect(text).toBe('等等，换一个科幻的');
    expect(note).toContain('「从前有座山，山上有座庙。」');
    h.store.dispose();
  });
  it('tells it once, and only for the next question', async () => {
    const h = await interrupted();
    h.store.interrupt();
    finishAnswer(h.chat, '');
    await h.store.submit('换一个科幻的', true);
    expect((h.chat.send.mock.calls.at(-1) as unknown as unknown[])[2]).toContain('【语音打断】');
    finishAnswer(h.chat, '');
    await h.store.submit('再短一点', true);
    expect(h.chat.send).toHaveBeenLastCalledWith('再短一点');
    h.store.dispose();
  });
  it('does not mention an interruption when the answer was read to the end', async () => {
    const h = setup({ prefs: { open: 'type', read: 'voice' } });
    await h.store.init(); await h.store.opened(); await h.store.submit('讲个故事', true);
    finishAnswer(h.chat, '一个故事。');
    await h.store.submit('谢谢', true);
    expect(h.chat.send).toHaveBeenLastCalledWith('谢谢');
    h.store.dispose();
  });
});

describe('实时对话', () => {
  it('keeps listening through silence, reads every answer even when reading is off, and listens again afterwards', async () => {
    const { store, chat, asr, speech, advance, say } = setup({ prefs: { open: 'type', read: 'never' } });
    await store.init(); await store.opened();
    await store.startLive();
    expect(store.live).toBe(true);
    expect(store.liveStatus).toBe('在听');
    advance(LISTEN_IDLE_MS + 1000);
    expect(asr.recordingRequested).toBe(true);
    say('明天几点开会');
    advance(SEND_GRACE_MS + 10); await flushPromises();
    expect(chat.send).toHaveBeenCalledWith('明天几点开会');
    expect(store.liveStatus).toBe('在想');
    finishAnswer(chat, '明天上午十点。');
    store.replied('s1', '明天上午十点。');
    expect(speech.speak).toHaveBeenCalled();
    expect(store.liveStatus).toBe('在说');
    speech.status = 'idle'; await nextTick(); await flushPromises();
    expect(asr.recordingRequested).toBe(true);
    expect(store.liveStatus).toBe('在听');
    store.dispose();
  });

  it('mutes the microphone without ending, and 结束 stops listening and speaking but not the task', async () => {
    const { store, chat, asr, speech, say, advance } = setup();
    await store.init(); await store.opened();
    await store.startLive();
    await store.toggleMute();
    expect(store.muted).toBe(true); expect(store.liveStatus).toBe('已静音');
    expect(asr.recordingRequested).toBe(false);
    await store.toggleMute();
    expect(store.muted).toBe(false); expect(asr.recordingRequested).toBe(true);
    say('帮我查一下天气'); advance(SEND_GRACE_MS + 10); await flushPromises();
    store.endLive();
    expect(store.live).toBe(false); expect(asr.recordingRequested).toBe(false);
    expect(speech.stop).toHaveBeenCalled();
    expect(chat.cancel).not.toHaveBeenCalled();
    finishAnswer(chat, '晴。');
    speech.speak.mockClear();
    store.replied('s1', '晴。');
    expect(speech.speak).not.toHaveBeenCalled();
    store.dispose();
  });

  it('ends when the capsule folds away', async () => {
    const { store, asr } = setup();
    await store.init(); await store.opened();
    await store.startLive();
    store.closed();
    expect(store.live).toBe(false); expect(asr.recordingRequested).toBe(false);
    store.dispose();
  });
});
