import type { AttachmentCard, PendingAttachment } from '../ui/chat/attachments.ts';
import { reactive, watch } from 'vue';
import type { ChatStore } from '../ui/chat/chat-store.ts';
import type { AsrStore, TranscriptContext } from '../ui/asr/store.ts';
import type { SpeechStore } from '../ui/speech/store.ts';
import { isPlaybackEcho, isSpeechText } from './voice-guard.ts';
import type { ApprovalDecision, ApprovalRequest, ChatItem, QuestionRequest, SessionSummary } from '../ui/chat/types.ts';

export type OpenMode = 'voice' | 'type';
export type ReadMode = 'voice' | 'always' | 'never';
export type SettingsTarget = 'ai' | 'assistant';

export type Phase = 'idle' | 'listening' | 'thinking' | 'answering' | 'speaking' | 'attention';

/** Only the assistant window uses this pointer; conversation history remains with the host. */
export const SESSION_KEY = 'xiaoban.assistant.session.v1';
/** Voice text is sent this long after the last words were recognised, in case more follows. */
export const SEND_GRACE_MS = 600;
/** The microphone is released when nothing was heard for this long. The capsule itself stays until it is dismissed. */
export const LISTEN_IDLE_MS = 15_000;
const MODEL_REQUIRED = '还没有配置模型。先到设置里添加一个，我才能回答。';

export interface AssistantDeps {
  desktop: boolean;
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  listen<T>(event: string, callback: (payload: T) => void): Promise<() => void>;
  chat: ChatStore;
  asr: AsrStore;
  speech: SpeechStore;
  now?(): number;
  storage?: Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;
  /** The console's colour scheme, as the host reports it (at start and whenever it changes). */
  onAppearance?(value: string): void;
  setTimer?(callback: () => void, ms: number): unknown;
  clearTimer?(id: unknown): void;
}

export interface AssistantStore {
  /** The capsule is showing. */
  open: boolean;
  /** The small ball is showing instead. */
  ball: boolean;
  /** What the capsule does when it opens, and when answers are read aloud. Set in the assistant's settings page. */
  prefs: { open: OpenMode; read: ReadMode };
  /** The text box. Voice fills it as words are recognised, then sends it. */
  input: string;
  /** Images staged for the next question, as data URLs. */
  attachments: PendingAttachment[];
  /** Pause voice auto-send while choosing or dropping images. */
  prepareImages(): void;
  /** Why voice or the model is not ready, in a sentence. Empty when nothing is wrong. */
  hint: string;
  /** The settings tab that fixes the hint, if one does. */
  hintTab: '' | SettingsTarget;
  readonly phase: Phase;
  readonly configured: boolean;
  readonly voiceAvailable: boolean;
  readonly status: string;
  readonly question: string;
  readonly questionImages: string[];
  readonly questionFiles?: AttachmentCard[];
  readonly answer: string;
  readonly working: string;
  readonly failure: string;
  readonly approval: ApprovalRequest | null;
  readonly ask: QuestionRequest | null;
  init(): Promise<void>;
  opened(): Promise<void>;
  closed(): void;
  submit(text?: string, spoken?: boolean): Promise<void>;
  readonly changingSession: boolean;
  newConversation(): Promise<void>;
  toggleListening(): Promise<void>;
  /** Pause voice input for an edit/IME composition without stopping readout. */
  inputStarted(): void;
  /** Stops speaking, or cancels the answer being written. */
  interrupt(): void;
  decide(decision: ApprovalDecision): Promise<void>;
  reply(answer: string | null): Promise<void>;
  /** Called when a reply to the open conversation is complete. */
  replied(session: string, text: string, key?: string): void;
  /** Called when the open conversation was cancelled or failed. */
  stopped(): void;
  /** Shows a line in the card, optionally with a way to the settings tab that fixes it. */
  note(message: string, tab?: '' | SettingsTarget): void;
  hide(): Promise<void>;
  /** The ball was clicked: open the capsule. */
  unfold(): Promise<void>;
  /** Carries the conversation over to the full chat page. */
  openChat(): Promise<void>;
  openSettings(tab: SettingsTarget): Promise<void>;
  dispose(): void;
}

export function describeTool(name: string): string {
  const n = name.toLowerCase();
  if (/search|fetch|web|http|browse/.test(n)) return '正在联网查找';
  if (/read|glob|grep|list|find|ls\b/.test(n)) return '正在查看文件';
  if (/write|edit|patch|create/.test(n)) return '正在修改文件';
  if (/shell|bash|exec|run|command/.test(n)) return '正在执行命令';
  return '正在处理';
}

export function createAssistantStore(deps: AssistantDeps): AssistantStore {
  const { chat, asr, speech } = deps;
  let storage = deps.storage;
  try { storage ??= typeof window !== 'undefined' ? window.localStorage : undefined; } catch { /* persistence may be unavailable */ }
  const readSession = () => { try { return storage?.getItem(SESSION_KEY) || null; } catch { return null; } };
  const rememberSession = (id: string | null) => { try { if(id)storage?.setItem(SESSION_KEY,id);else storage?.removeItem(SESSION_KEY); } catch { /* the live conversation still works */ } };
  const setTimer = deps.setTimer ?? ((callback, ms) => window.setTimeout(callback, ms));
  const clearTimer = deps.clearTimer ?? (id => window.clearTimeout(id as number));
  const unsubscribers: (() => void)[] = [];
  let starting: Promise<void> | null = null;
  let disposed = false;
  let listenTimer: unknown, sendTimer: unknown;
  /** Words recognised so far in this utterance; sent together when speaking pauses. */
  let heard = '';
  /** The last question was spoken, so the answer is read aloud and listening resumes afterwards. */
  let voiceTurn = false;
  /** Reactive, because the phase shows `speaking` from the moment a reply is handed to the voice until it ends. */
  const flags = reactive({ awaitingSpeech: false, submitting: false, changingSession: false });
  let draftVersion = 0, speechVersion = 0, listenVersion = 0;
  let turnInterrupted = false;
  let readingText = '', echoTailUntil = 0;
  const now = deps.now ?? Date.now;

  const view = () => (chat.currentId ? chat.views[chat.currentId] ?? null : null);
  function turn() {
    const items = view()?.items ?? [];
    let at = items.length - 1;
    while (at >= 0 && items[at].kind !== 'user') at--;
    const question = at >= 0 ? (items[at] as Extract<ChatItem, { kind: 'user' }>).text : '';
    const rest = items.slice(at + 1);
    const answer = rest.filter((item): item is Extract<ChatItem, { kind: 'assistant' }> => item.kind === 'assistant').map(item => item.text.trim()).filter(Boolean).join('\n\n');
    const tool = [...rest].reverse().find((item): item is Extract<ChatItem, { kind: 'tool' }> => item.kind === 'tool' && item.status === 'running');
    const notice = [...rest].reverse().find((item): item is Extract<ChatItem, { kind: 'notice' }> => item.kind === 'notice' && item.tone === 'error');
    const questionImages = at >= 0 ? (items[at] as Extract<ChatItem, { kind: 'user' }>).images : [];
    const questionFiles = at >= 0 ? (items[at] as Extract<ChatItem, { kind: 'user' }>).files ?? [] : [];
    return { question, questionImages, questionFiles, answer, working: tool ? describeTool(tool.name) : '', failure: notice?.text ?? '' };
  }

  async function refreshModel() {
    const id = chat.currentId, mode = chat.permissionMode;
    await chat.refresh();
    // The overview's mode belongs to the host, not necessarily this capsule's conversation.
    if (chat.currentId === id) chat.permissionMode = mode;
    if (disposed) return;
    if (chat.configured) {
      if (store.hint === MODEL_REQUIRED) store.note('');
    } else if (store.open && !store.hint) store.note(MODEL_REQUIRED, 'ai');
  }

  function readPrefs(open?: string, read?: string) {
    if (open === 'voice' || open === 'type') store.prefs.open = open;
    if (read === 'voice' || read === 'always' || read === 'never') store.prefs.read = read;
  }
  function clearTimers() { clearTimer(listenTimer); clearTimer(sendTimer); listenTimer = sendTimer = undefined; }

  function armListenTimer() {
    clearTimer(listenTimer);
    listenTimer = setTimer(() => {
      if (disposed || !store.open || heard || !asr.recordingRequested || flags.awaitingSpeech) return;
      if (asr.speechActive || asr.recognizing) { armListenTimer(); return; }
      asr.deactivate(); voiceTurn = false;
      store.note('没听到声音，点光球可以继续说话。');
    }, LISTEN_IDLE_MS);
  }
  function stopReading() {
    turnInterrupted = true; speechVersion++;
    flags.awaitingSpeech = false;
    echoTailUntil = now() + 500;
    void speech.stop();
  }
  function externalSpeech(text: string, context?: TranscriptContext): boolean {
    return isSpeechText(text) && !(context?.playbackText && isPlaybackEcho(text, context.playbackText));
  }
  async function startListening(interruption = false): Promise<boolean> {
    const version=draftVersion, ticket=++listenVersion;
    const current = () => !disposed && version === draftVersion && ticket === listenVersion;
    await asr.init();
    if(!current())return false;
    if (asr.microphoneSupported === false) return false;
    if (!asr.enabled) { store.note('语音对话还没开启。在超级小伴设置里打开后，就能直接说话。', 'assistant'); return false; }
    const prepared = await asr.prepare();
    if (!store.open || !current()) return false;
    if (!prepared) { store.note(asr.error || '语音识别还没准备好，请稍后再试。'); return false; }
    store.note(''); heard = '';
    await asr.activate((text, context)=>{if(current())onHeard(text, context);}, {
      noiseGuard: true,
      playbackText: () => (flags.awaitingSpeech && speech.status === 'playing') || now() < echoTailUntil ? readingText : '',
      onPreview: (text, context) => {
        if (!current() || !store.open || !flags.awaitingSpeech || !externalSpeech(text, context)) return;
        // Keep this microphone and its full utterance buffer: stopping TTS must
        // not discard the beginning of the user's interruption.
        clearTimers(); voiceTurn = true; stopReading(); armListenTimer();
      },
    });
    if (!current()) return false;
    if (!asr.recordingRequested) return false;
    if (!interruption || !flags.awaitingSpeech) armListenTimer();
    return true;
  }
  function onHeard(text: string, context?: TranscriptContext) {
    if (!externalSpeech(text, context)) return;
    if (flags.awaitingSpeech) stopReading();
    voiceTurn = true;
    clearTimer(listenTimer);
    heard = heard ? `${heard} ${text}` : text;
    store.input = heard;
    clearTimer(sendTimer);
    sendTimer = setTimer(function flush() {
      if (!store.open) return;
      if (asr.recognizing || asr.speechActive) { sendTimer = setTimer(flush, 300); return; }
      void store.submit(heard, true);
    }, SEND_GRACE_MS);
  }
  /** What happens once an answer is complete and has been read: listen again if the user is talking. */
  function afterTurn() {
    if (!store.open || !asr.enabled) return;
    if (voiceTurn) {
      if (asr.recordingRequested) armListenTimer();
      else void startListening();
    } else if (asr.recordingRequested) {
      // A typed turn does not leave hands-free recording on. Flush any speech
      // that overlapped the end of playback before releasing the microphone.
      void asr.finish();
    } else listenVersion++; // Do not open a late monitoring microphone after a typed readout ends.
  }

  const store: AssistantStore = reactive({
    open: false, ball: false, prefs: { open: 'voice', read: 'voice' }, input: '', attachments: [], hint: '', hintTab: '',
    get configured() { return chat.configured; },
    get changingSession() { return flags.changingSession; },
    get voiceAvailable() { return asr.enabled && asr.microphoneSupported !== false; },
    get question() { return turn().question; },
    get questionImages() { return turn().questionImages; },
    prepareImages() {
      store.inputStarted();
    },
    get questionFiles() { return turn().questionFiles; },
    get answer() { return turn().answer; },
    get working() { return turn().working; },
    get failure() { return turn().failure; },
    get approval() { return view()?.approvals[0] ?? null; },
    get ask() { return view()?.questions[0] ?? null; },
    get phase(): Phase {
      const current = view();
      if (current && (current.approvals.length || current.questions.length)) return 'attention';
      if (flags.awaitingSpeech) return 'speaking';
      if (current?.running) return turn().answer ? 'answering' : 'thinking';
      if (asr.listening || asr.recognizing || asr.preparing || asr.recordingRequested) return 'listening';
      return 'idle';
    },
    get status() {
      switch (store.phase) {
        case 'attention': return '需要你确认一下';
        case 'thinking': return store.working || '思考中…';
        case 'answering': return store.working || '正在回答…';
        case 'speaking': return asr.recordingRequested ? '正在朗读，说话或发送可打断' : '正在朗读，发送可打断';
        case 'listening': return asr.preparing ? '正在准备语音识别…' : asr.recognizing ? '正在识别…' : '我在听，请说…';
        default: return store.voiceAvailable ? '问点什么，或直接说话' : '问点什么';
      }
    },
    init() {
      starting ??= (async () => {
        if (!deps.desktop) return;
        unsubscribers.push(
          await deps.listen('assistant-open', () => { void store.opened(); }),
          await deps.listen('assistant-close', () => store.closed()),
          await deps.listen('ai-config-changed', () => { if (!disposed) void refreshModel(); }),
          await deps.listen<boolean>('assistant-ball', shown => { store.ball = Boolean(shown); }),
          // The settings page lives in another window; its choices arrive with every settings change.
          await deps.listen<{ assistantOpen?: string; assistantRead?: string; appearance?: string }>('settings-changed', value => { readPrefs(value.assistantOpen, value.assistantRead); if (value.appearance) deps.onAppearance?.(value.appearance); }),
        );
        try {
          const saved = await deps.invoke<{ open?: string; read?: string; appearance?: string }>('assistant_prefs');
          readPrefs(saved?.open, saved?.read);
          if (saved?.appearance) deps.onAppearance?.(saved.appearance);
        } catch { /* the defaults stand */ }
        await Promise.all([chat.init(), asr.init(), speech.init()]);
        await deps.invoke('assistant_ready');
      })();
      return starting;
    },
    async opened() {
      // Native transitions re-announce state after showing the WebView.
      // A duplicate event must never erase a draft or restart voice capture.
      if (disposed || store.open) return;
      clearTimers();
      draftVersion++; store.attachments = [];
      store.open = true; store.ball = false; store.note(''); store.input = ''; heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
      await Promise.all([chat.init(), asr.init()]);
      await refreshModel();
      if (disposed || !store.open || flags.changingSession) return;
      if (chat.loadError) { store.note('无法加载上次会话，请稍后重新打开小伴。'); return; }
      if (!chat.configured) { store.note(MODEL_REQUIRED, 'ai'); return; }
      const id = chat.currentId ?? readSession();
      if (id && chat.sessions.some(session => session.id === id)) {
        if(chat.currentId !== id) await chat.select(id);
      } else {
        rememberSession(null);
        await chat.select(null);
      }
      // Typing first: the capsule waits, and a tap on the ball starts listening.
      if (store.open && asr.enabled && store.prefs.open === 'voice') { voiceTurn = await startListening(); }
    },
    closed() {
      draftVersion++; store.attachments = [];
      clearTimers();
      asr.deactivate(); void speech.stop();
      store.open = false; store.input = ''; store.note(''); heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
    },
    async newConversation() {
      if (flags.changingSession || flags.submitting) return;
      flags.changingSession = true;draftVersion++;
      clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false; asr.deactivate();
      try {
        await speech.stop();
        if(view()?.running) await chat.cancel();
        const session = await deps.invoke<SessionSummary>('chat_new_session', { permissionMode: 'auto' });
        if(!session?.id) throw new Error('无法创建新会话，请重试。');
        if(!chat.sessions.some(item => item.id === session.id)) chat.sessions.unshift(session);
        await chat.select(session.id);
        draftVersion++; store.input=''; store.attachments=[]; store.note('');
        if(store.open && asr.enabled && store.prefs.open === 'voice') voiceTurn = await startListening();
      } catch(error) { store.note(error instanceof Error ? error.message : String(error)); }
      finally { flags.changingSession = false; }
    },
    async submit(text, spoken = false) {
      const value = (text ?? store.input).trim();
      if ((!value && !store.attachments.length) || flags.submitting || flags.changingSession || view()?.running || store.ask || store.approval) return;
      if (!chat.configured) {
        store.input = value;
        store.note(MODEL_REQUIRED, 'ai');
        return;
      }
      const images = [...store.attachments], version = draftVersion;
      flags.submitting = true; turnInterrupted = false; speechVersion++; listenVersion++;
      flags.awaitingSpeech = false;
      clearTimers(); heard = '';
      asr.deactivate();
      store.input = ''; store.attachments = []; store.note('');
      try {
        await speech.stop();
        if (version !== draftVersion) return;
        voiceTurn = spoken;
        // New capsule conversations start in full trust; resumed ones keep their saved mode.
        if (!chat.currentId) chat.permissionMode = 'auto';
        const sent = images.length ? await chat.send(value, images) : await chat.send(value);
        if (!sent && version === draftVersion) {
          if (!store.input) store.input = value;
          store.attachments = [...images, ...store.attachments].slice(0, 8);
        }
      } finally { flags.submitting = false; }
    },
    async toggleListening() {
      if (asr.listening || asr.recordingRequested) { asr.deactivate(); voiceTurn = false; heard = ''; return; }
      await speech.stop(); flags.awaitingSpeech = false;
      voiceTurn = await startListening();
    },
    inputStarted() {
      draftVersion++; clearTimers(); heard = ''; voiceTurn = false;
      asr.deactivate();
    },
    interrupt() {
      // Stop both channels, including a reply that finishes while cancellation is in flight.
      turnInterrupted = true; speechVersion++; listenVersion++;
      clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
      asr.deactivate(); void speech.stop();
      if (view()?.running) void chat.cancel();
    },
    async decide(decision) { const request = store.approval; if (request) await chat.answerApproval(request, decision); },
    async reply(answer) { const request = store.ask; if (request) await chat.answerQuestion(request, answer); },
    replied(session, text, key) {
      if (!store.open || session !== chat.currentId || turnInterrupted) return;
      const version = ++speechVersion;
      if (store.prefs.read === 'always' || (store.prefs.read === 'voice' && voiceTurn)) {
        readingText = text; echoTailUntil = 0;
        flags.awaitingSpeech = true;
        if (asr.enabled && asr.microphoneSupported !== false) void startListening(true);
        void speech.speak(text, key ?? `assistant:${session}`, undefined, true).then(() => {
          if (version === speechVersion && flags.awaitingSpeech && speech.status === 'idle') { flags.awaitingSpeech = false; afterTurn(); }
        });
      } else afterTurn();
    },
    stopped() {
      speechVersion++; listenVersion++; flags.awaitingSpeech = false; clearTimers(); asr.deactivate(); void speech.stop();
    },
    note(message, tab = '') { store.hint = message; store.hintTab = message ? tab : ''; },
    hide() { return deps.invoke<void>('assistant_hide').catch(() => {}); },
    async unfold() { await deps.invoke('assistant_unfold'); },
    async openChat() { await deps.invoke('assistant_open_chat', { session: chat.currentId }); },
    async openSettings(tab) { await deps.invoke('assistant_open_settings', { tab }); },
    dispose() {
      disposed = true; clearTimers(); stopWatch(); stopSessionWatch(); stopVoiceWatch();
      unsubscribers.splice(0).forEach(off => off());
      asr.deactivate(); void speech.stop();
    },
  });
  const stopWatch = watch(() => speech.status, (status, before) => {
    if (flags.awaitingSpeech && status === 'idle' && before !== 'idle') {
      echoTailUntil = now() + 500; flags.awaitingSpeech = false; afterTurn();
    }
  });
  const stopVoiceWatch = watch(() => asr.enabled, enabled => {
    if (!enabled) { clearTimers(); heard = ''; voiceTurn = false; asr.deactivate(); }
    else if (store.hint === '语音对话还没开启。在超级小伴设置里打开后，就能直接说话。') store.note('');
  }, { flush: 'sync' });
  const stopSessionWatch = watch(() => chat.currentId, id => {
    if(id && !disposed) rememberSession(id);
  }, { flush: 'sync' });
  return store;
}

