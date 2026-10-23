import type { AttachmentCard, PendingAttachment } from '../ui/chat/attachments.ts';
import { reactive, watch } from 'vue';
import type { ChatStore } from '../ui/chat/chat-store.ts';
import type { AsrStore, TranscriptContext } from '../ui/asr/store.ts';
import type { SpeechStore, SpeechStream } from '../ui/speech/store.ts';
import { isFiller, isPlaybackEcho, isSpeechText } from './voice-guard.ts';
import { ACK_PHRASES, ackDelay, ackKind, chooseAck, type AckKind } from '../voice/ack.ts';
import { interruptionNote, spokenNote } from '../voice/interruption.ts';
import type { ApprovalDecision, ApprovalRequest, ChatItem, QuestionRequest, SessionSummary } from '../ui/chat/types.ts';

export type OpenMode = 'voice' | 'type';
export type ReadMode = 'voice' | 'always' | 'never';
export type SettingsTarget = 'ai' | 'assistant';

export type Phase = 'idle' | 'listening' | 'thinking' | 'answering' | 'speaking' | 'attention';

/** Only the assistant window uses this pointer; conversation history remains with the host. */
export const SESSION_KEY = 'xiaoban.assistant.session.v1';
/** Voice text is sent this long after the last words were recognised, in case more follows. */
export const SEND_GRACE_MS = 300;
/** The same when the speaker was judged to have finished: nothing is expected to follow, so there is little to wait for. */
export const SETTLED_GRACE_MS = 100;
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
  /** The console's theme colour, as the host reports it. */
  onAccent?(value: string): void;
  setTimer?(callback: () => void, ms: number): unknown;
  clearTimer?(id: unknown): void;
}

/** A line shown beside the ball while the capsule is away: an answer finished, or something needs the user. */
export interface BallNotice { kind: 'done' | 'attention' | 'error'; text: string }

/**
 * Where things sit in the assistant's window (the stage), in CSS pixels from its top-left corner. `ball` is the
 * ball's square; it is null for a capsule opened without the ball. `right`: the ball is on the right half of its
 * screen, so the capsule and notices open to its left.
 */
export interface StageLayout { ball: { x: number; y: number; size: number } | null; right: boolean }

export interface AssistantStore {
  /** The capsule is showing. */
  open: boolean;
  /** The small ball is showing instead. */
  ball: boolean;
  /** What the capsule does when it opens, when answers are read aloud, and whether resting the pointer on the ball
   * opens it. Set in the assistant's settings page. */
  prefs: { open: OpenMode; read: ReadMode; hover: boolean };
  layout: StageLayout;
  /** The capsule opened because the pointer rested on the ball: it has not taken the keyboard focus, and folds
   * away again when the pointer leaves unless the user starts using it. */
  peek: boolean;
  /** The pointer is over something of the assistant's (reported by the host, which sees it leave). */
  hovering: boolean;
  /** A re-anchor in progress: the host waits for the ball to fade out before moving the window. 0 when none. */
  reanchoring: number;
  reanchorReady(id: number): void;
  /** 实时对话: a hands-free conversation that keeps listening after every answer until the user ends it. Every
   * answer is read aloud and can be talked over. */
  live: boolean;
  /** In a live conversation, the microphone is off until the user switches it back on. */
  muted: boolean;
  /** When the live conversation started, for its clock. 0 when none. */
  liveSince: number;
  /** One or two words for where a live conversation is: 正在连接…, 在听, 在想, 在说, 已静音. Empty when not live. */
  readonly liveStatus: string;
  startLive(): Promise<void>;
  endLive(): void;
  /** Another window took over the microphone and the voice: stop listening and speaking here. */
  yieldVoice(): void;
  toggleMute(): Promise<void>;
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
  /** Voice preparation and temporary errors, separate from the text input placeholder. */
  readonly voiceStatus: string;
  readonly question: string;
  readonly questionImages: string[];
  readonly questionFiles?: AttachmentCard[];
  readonly answer: string;
  readonly working: string;
  readonly failure: string;
  readonly approval: ApprovalRequest | null;
  readonly ask: QuestionRequest | null;
  /** Everything in the open conversation, for the multi-turn card. */
  readonly items: ChatItem[];
  /** The open conversation, if any. */
  readonly currentId: string | null;
  /** The open conversation's title; empty for a conversation not started yet. */
  readonly title: string;
  /** The latest conversations, newest first, for the switcher in the card. */
  readonly recent: SessionSummary[];
  /** An answer finished (or failed) while only the ball was showing. Cleared when the capsule opens. */
  unread: boolean;
  /** What the ball says beside itself, if anything. */
  notice: BallNotice | null;
  dismissNotice(): void;
  /** Opens another recent conversation in the card. */
  switchSession(id: string): Promise<void>;
  /** Asks the last question again. */
  retry(): Promise<void>;
  init(): Promise<void>;
  opened(peek?: boolean): Promise<void>;
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
  /** Short-lived voice notices leave the text input and conversation card available. */
  noteVoice(message: string): void;
  /** Push to talk: the shortcut went down (`true`, start listening) or up (`false`, send what was heard). */
  pushToTalk(down: boolean): Promise<void>;
  hide(): Promise<void>;
  /** The ball was clicked (or, with `peek`, the pointer rested on it): open the capsule. */
  unfold(peek?: boolean): Promise<void>;
  /** A capsule opened on hover is being used: it takes the keyboard focus and stays. */
  engage(): void;
  /** "存到资料库" under an answer: kept in 资料库 → 收藏 with a way back to this conversation. */
  saveAnswer(text: string): Promise<boolean>;
  /** Carries the conversation over to the full chat page. */
  openChat(): Promise<void>;
  openSettings(tab: SettingsTarget): Promise<void>;
  dispose(): void;
}

/** One short line of an answer, for the notice beside the ball. */
export function noticeLine(text: string, max = 28): string {
  const line = text.replace(/[#>*`~]/g, '').split('\n').map(part => part.trim()).find(Boolean) ?? '';
  return line.length > max ? `${line.slice(0, max)}…` : line;
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
  let listenTimer: unknown, sendTimer: unknown, voiceNoticeTimer: unknown, ackTimer: unknown;
  /** Words recognised so far in this utterance; sent together when speaking pauses. */
  let heard = '';
  /** The last question was spoken, so the answer is read aloud and listening resumes afterwards. */
  let voiceTurn = false;
  /** Reactive, because the phase shows `speaking` from the moment a reply is handed to the voice until it ends. */
  const flags = reactive({ awaitingSpeech: false, submitting: false, changingSession: false, voiceNotice: '', ptt: false, partial: '' });
  /** The shortcut went down before the capsule had opened; start talking once it has. */
  let pendingPtt = false;
  /** The answer being read aloud while it is still being written. */
  let reading: SpeechStream | null = null;
  /** A spoken interruption cancelled the answer: keep the microphone that heard it. */
  let keepListening = false;
  let draftVersion = 0, speechVersion = 0, listenVersion = 0;
  let turnInterrupted = false;
  let readingText = '', echoTailUntil = 0;
  /** How far the answer got when the user talked over it; goes to the model with their next message. */
  let interruption: { session: string | null; note: string } | null = null;
  const now = deps.now ?? Date.now;

  const view = () => (chat.currentId ? chat.views[chat.currentId] ?? null : null);
  let takingCompanion = false;
  async function adoptCompanionSession(): Promise<boolean> {
    if (takingCompanion) return true;
    takingCompanion = true;
    let ownsSessionChange = false;
    try {
      const session = await deps.invoke<string | null>('assistant_take_companion_session');
      if (typeof session !== 'string' || !session) return false;
      // A companion must not erase another conversation's unsent draft or interrupt its run.
      if (flags.changingSession || flags.submitting || view()?.running || store.input.trim() || store.attachments.length) {
        await deps.invoke('assistant_open_chat', { session });
        return true;
      }
      ownsSessionChange = true; flags.changingSession = true; draftVersion++; speechVersion++; clearTimers(); asr.deactivate(); interruption = null;
      heard = ''; voiceTurn = false; flags.awaitingSpeech = false; flags.ptt = false; pendingPtt = false; reading = null; turnInterrupted = true;
      await speech.stop();
      const overview = await deps.invoke<{ sessions: SessionSummary[] }>('chat_overview');
      if (disposed || !store.open) return true;
      if (overview?.sessions) chat.sessions = overview.sessions;
      await chat.select(session); store.note('');
      return true;
    } catch (error) { store.note(error instanceof Error ? error.message : String(error)); return true; }
    finally { takingCompanion = false; if (ownsSessionChange) flags.changingSession = false; }
  }
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

  function readPrefs(open?: string, read?: string, hover?: boolean) {
    if (open === 'voice' || open === 'type') store.prefs.open = open;
    if (read === 'voice' || read === 'always' || read === 'never') store.prefs.read = read;
    if (typeof hover === 'boolean') store.prefs.hover = hover;
  }
  function clearTimers() { clearTimer(listenTimer); clearTimer(sendTimer); clearTimer(ackTimer); listenTimer = sendTimer = ackTimer = undefined; flags.partial = ''; }
  function noteVoice(message: string) {
    clearTimer(voiceNoticeTimer); voiceNoticeTimer = undefined; flags.voiceNotice = message;
    if (message) voiceNoticeTimer = setTimer(() => { flags.voiceNotice = ''; voiceNoticeTimer = undefined; }, 5000);
  }

  function armListenTimer() {
    clearTimer(listenTimer);
    // A live conversation waits as long as the user likes; only 结束 ends it.
    if (store.live) return;
    listenTimer = setTimer(() => {
      if (disposed || !store.open || heard || !asr.recordingRequested || flags.awaitingSpeech || flags.ptt) return;
      if (asr.speechActive || asr.recognizing) { armListenTimer(); return; }
      asr.deactivate(); voiceTurn = false;
      noteVoice('没听到声音，点击生命球可重新开始。');
    }, LISTEN_IDLE_MS);
  }
  /** The reading is cut off: remember how far the user got. */
  function noteInterruption() {
    if (!reading && !flags.awaitingSpeech) return;
    interruption = { session: chat.currentId, note: interruptionNote(reading ? reading.spoken : null) };
  }
  let lastAck: string | undefined;
  /** What was already in the box when recording started: spoken words are added after it. */
  let inputBase = '';
  /** What is said first, fitted to what the user said: a greeting at once, a request or question after a short wait. */
  function scheduleAck(kind: AckKind, phrase: string) {
    clearTimer(ackTimer);
    const ticket = speechVersion;
    ackTimer = setTimer(() => {
      ackTimer = undefined;
      if (ticket !== speechVersion || disposed || !store.open || flags.awaitingSpeech || !view()?.running || turn().answer) return;
      speech.ack?.(phrase);
    }, ackDelay(kind));
  }
  function stopReading() {
    noteInterruption();
    turnInterrupted = true; speechVersion++;
    flags.awaitingSpeech = false; reading = null;
    echoTailUntil = now() + 500;
    void speech.stop();
  }
  /** The user talks over an answer that is still being written: stop it too, or the next question would wait. */
  function bargeIn() {
    stopReading();
    if (view()?.running) { keepListening = true; void chat.cancel(); }
  }
  /** Reads the answer of this turn sentence by sentence while it is written. */
  function beginReading() {
    if (!speech.stream) return;
    const ticket = speechVersion;
    readingText = ''; echoTailUntil = 0;
    reading = speech.stream(`assistant:${chat.currentId ?? ''}`, undefined, {
      onStart: () => {
        if (ticket !== speechVersion || !store.open) return;
        clearTimer(ackTimer); ackTimer = undefined;
        flags.awaitingSpeech = true;
        // Listen while reading so the user can talk over it.
        if (asr.enabled && asr.microphoneSupported !== false && !asr.recordingRequested && !store.muted) void startListening(true);
      },
      onSentence: text => { if (ticket === speechVersion) readingText += text; },
    });
    if (store.answer) reading.push(store.answer);
  }
  async function beginTalk() {
    pendingPtt = false; flags.ptt = true; noteVoice('');
    if (flags.awaitingSpeech || reading) bargeIn();
    clearTimers(); heard = '';
    if (asr.recordingRequested) { voiceTurn = true; return; }
    voiceTurn = await startListening();
    if (!voiceTurn) flags.ptt = false;
  }
  function externalSpeech(text: string, context?: TranscriptContext): boolean {
    return isSpeechText(text) && !isFiller(text) && !(context?.playbackText && isPlaybackEcho(text, context.playbackText));
  }
  async function startListening(interruption = false): Promise<boolean> {
    const version=draftVersion, ticket=++listenVersion;
    const current = () => !disposed && version === draftVersion && ticket === listenVersion;
    await asr.init();
    if(!current())return false;
    if (asr.microphoneSupported === false) { noteVoice('当前设备不支持语音输入，可以继续打字。'); return false; }
    if (!asr.enabled) { await store.openSettings('assistant'); return false; }
    const prepared = await asr.prepare();
    if (!store.open || !current()) return false;
    if (!prepared) { noteVoice(asr.error || '语音识别还没准备好，可以继续打字。'); return false; }
    noteVoice(''); heard = ''; inputBase = store.input;
    await asr.activate((text, context)=>{if(current())onHeard(text, context);}, {
      noiseGuard: true,
      onPartial: text => { if (current() && store.open && !flags.awaitingSpeech) flags.partial = text; },
      playbackText: () => (flags.awaitingSpeech && speech.status === 'playing') || now() < echoTailUntil ? readingText : '',
      onPreview: (text, context) => {
        if (!current() || !store.open || !flags.awaitingSpeech || !externalSpeech(text, context)) return;
        // Keep this microphone and its full utterance buffer: stopping TTS must
        // not discard the beginning of the user's interruption.
        clearTimers(); voiceTurn = true; bargeIn(); armListenTimer();
      },
    });
    if (!current()) return false;
    if (!asr.recordingRequested) return false;
    if (!interruption || !flags.awaitingSpeech) armListenTimer();
    return true;
  }
  function onHeard(text: string, context?: TranscriptContext) {
    flags.partial = '';
    if (!externalSpeech(text, context)) return;
    if (flags.awaitingSpeech) bargeIn();
    voiceTurn = true;
    clearTimer(listenTimer);
    heard = heard ? `${heard} ${text}` : text;
    // Recording outside a live conversation only fills the box: the user reads it and sends it. A pause never sends.
    if (!store.live && !flags.ptt) { store.input = inputBase && !/\s$/.test(inputBase) ? `${inputBase} ${heard}` : inputBase + heard; voiceTurn = false; return; }
    store.input = heard;
    // Push to talk sends on release, never on a pause.
    if (flags.ptt) return;
    clearTimer(sendTimer);
    sendTimer = setTimer(function flush() {
      if (!store.open) return;
      if (asr.recognizing || asr.speechActive) { sendTimer = setTimer(flush, 300); return; }
      void store.submit(heard, true);
    }, context?.settled ? SETTLED_GRACE_MS : SEND_GRACE_MS);
  }
  /** What happens once an answer is complete and has been read: listen again if the user is talking. */
  function afterTurn() {
    if (!store.open || !asr.enabled) return;
    if (store.live) {
      if (store.muted) return;
      voiceTurn = true;
      if (!asr.recordingRequested) void startListening();
      return;
    }
    // Only a live conversation carries on listening; otherwise the voice is done once the answer has been read.
    if (asr.recordingRequested) {
      // A typed turn does not leave hands-free recording on. Flush any speech
      // that overlapped the end of playback before releasing the microphone.
      void asr.finish();
    } else listenVersion++; // Do not open a late monitoring microphone after a typed readout ends.
  }

  /** Ends a live conversation: the microphone and the voice stop; a task it started carries on and its answer
   * stays in the card. */
  function endLive() {
    if (!store.live) return;
    store.live = false; store.muted = false; store.liveSince = 0;
    listenVersion++; speechVersion++; reading = null;
    clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false; store.input = '';
    asr.deactivate(); void speech.stop();
  }

  /** Another window (the console's conversation) took over the microphone and the voice. The answer being read here is
   * left unread, so it does not start talking again once the other window is done. */
  function yieldVoice() {
    if (store.live) { endLive(); return; }
    turnInterrupted = true; speechVersion++; listenVersion++; reading = null;
    clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false; flags.ptt = false; pendingPtt = false; flags.partial = '';
    asr.deactivate(); void speech.stop();
  }

  const store: AssistantStore = reactive({
    open: false, ball: false, prefs: { open: 'voice', read: 'voice', hover: true }, input: '', attachments: [], hint: '', hintTab: '',
    layout: { ball: null, right: true }, peek: false, hovering: false, reanchoring: 0,
    reanchorReady(id) { if (store.reanchoring === id) void deps.invoke('assistant_reanchor_ready', { id }).catch(() => {}); },
    live: false, muted: false, liveSince: 0,
    get liveStatus() {
      if (!store.live) return '';
      if (store.muted) return '已静音';
      switch (store.phase) {
        case 'speaking': return '在说';
        case 'thinking': case 'answering': return '在想';
        case 'attention': return '等你确认';
      }
      if (asr.preparing || asr.warming || (asr.recordingRequested && !asr.listening) || !asr.recordingRequested) return '正在连接…';
      return '在听';
    },
    async startLive() {
      if (store.live || flags.changingSession) return;
      if (!asr.enabled || asr.microphoneSupported === false) { await startListening(); return; }
      store.live = true; store.muted = false; store.liveSince = now();
      draftVersion++; clearTimers(); heard = '';
      if (!view()?.running) { await speech.stop(); flags.awaitingSpeech = false; }
      voiceTurn = await startListening();
      if (!voiceTurn && !store.muted) endLive();
    },
    endLive,
    yieldVoice,
    async toggleMute() {
      if (!store.live) return;
      if (store.muted) {
        store.muted = false;
        voiceTurn = await startListening(flags.awaitingSpeech);
        return;
      }
      store.muted = true;
      listenVersion++; clearTimers(); heard = ''; store.input = '';
      asr.deactivate();
    },
    unread: false, notice: null,
    get items() { return view()?.items ?? []; },
    get currentId() { return chat.currentId; },
    get title() { const id = chat.currentId; return (id && chat.sessions.find(session => session.id === id)?.title) || ''; },
    get recent() { return [...chat.sessions].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 6); },
    dismissNotice() { store.notice = null; },
    async switchSession(id) {
      if (flags.changingSession || flags.submitting || id === chat.currentId || !chat.sessions.some(session => session.id === id)) return;
      endLive(); noteVoice(''); flags.changingSession = true; draftVersion++;
      clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false; asr.deactivate(); interruption = null;
      try { await speech.stop(); await chat.select(id); store.note(''); }
      catch (error) { store.note(error instanceof Error ? error.message : String(error)); }
      finally { flags.changingSession = false; }
    },
    async retry() {
      if (flags.submitting || flags.changingSession || view()?.running) return;
      turnInterrupted = false; voiceTurn = false;
      await chat.retry();
    },
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
      if (asr.listening) return 'listening';
      return 'idle';
    },
    get status() {
      switch (store.phase) {
        case 'attention': return '需要你确认一下';
        case 'thinking': return store.working || '思考中…';
        case 'answering': return store.working || '正在回答…';
        case 'speaking': return asr.listening ? '正在朗读，说话或发送可打断' : '正在朗读，发送可打断';
        case 'listening': return store.live ? '正在聆听…' : '正在录音，再点球结束';
        default: return '问点什么…';
      }
    },
    get voiceStatus() {
      if (flags.voiceNotice) return flags.voiceNotice;
      if (flags.ptt) return !asr.listening ? '正在启动麦克风…' : flags.partial ? `${flags.partial}… · 松开发送` : '正在听，松开快捷键发送';
      if (flags.partial && asr.listening) return `${flags.partial}…`;
      if (asr.listening && !store.live) return '录音中，文字会留在输入框，再点一下球结束，由你发送';
      if (asr.enabled && (asr.preparing || asr.warming)) return '正在准备语音引擎…';
      if (asr.recordingRequested && !asr.listening) return '正在启动麦克风…';
      return '';
    },
    init() {
      starting ??= (async () => {
        if (!deps.desktop) return;
        unsubscribers.push(
          await deps.listen<{ peek?: boolean } | null>('assistant-open', value => { void store.opened(Boolean(value?.peek)); }),
          await deps.listen<StageLayout>('assistant-stage', value => { if (value) store.layout = { ball: value.ball ?? null, right: Boolean(value.right) }; }),
          await deps.listen<boolean>('assistant-hover', inside => { store.hovering = Boolean(inside); }),
          await deps.listen<{ id: number }>('assistant-reanchor', value => { store.reanchoring = value?.id ?? 0; }),
          await deps.listen<{ id: number }>('assistant-reanchor-done', value => { if (!value || value.id === store.reanchoring) store.reanchoring = 0; }),
          await deps.listen('assistant-companion-session', () => { if (store.open) void adoptCompanionSession(); else void store.opened(); }),
          await deps.listen('assistant-close', () => store.closed()),
          await deps.listen<string>('assistant-ptt', state => { void store.pushToTalk(state === 'start'); }),
          await deps.listen('ai-config-changed', () => { if (!disposed) void refreshModel(); }),
          await deps.listen<boolean>('assistant-ball', shown => { store.ball = Boolean(shown); }),
          // The settings page lives in another window; its choices arrive with every settings change.
          await deps.listen<{ assistantOpen?: string; assistantRead?: string; assistantHover?: boolean; appearance?: string; accent?: string }>('settings-changed', value => { readPrefs(value.assistantOpen, value.assistantRead, value.assistantHover); if (value.appearance) deps.onAppearance?.(value.appearance); if (value.accent) deps.onAccent?.(value.accent); }),
        );
        try {
          const saved = await deps.invoke<{ open?: string; read?: string; hover?: boolean; appearance?: string; accent?: string }>('assistant_prefs');
          readPrefs(saved?.open, saved?.read, saved?.hover);
          if (saved?.appearance) deps.onAppearance?.(saved.appearance);
          if (saved?.accent) deps.onAccent?.(saved.accent);
        } catch { /* the defaults stand */ }
        await Promise.all([chat.init(), asr.init(), speech.init()]);
        await deps.invoke('assistant_ready');
      })();
      return starting;
    },
    async opened(peek = false) {
      // The host re-announces state after showing the WebView.
      // A duplicate event must never erase a draft or restart voice capture.
      if (disposed) return;
      if (store.open) { if (!peek) store.peek = false; return; }
      store.peek = peek;
      noteVoice('');
      clearTimers();
      // A draft and its attachments wait while the capsule is folded away; only voice capture starts afresh.
      draftVersion++;
      store.open = true; store.ball = false; store.unread = false; store.notice = null; store.note(''); heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
      await Promise.all([chat.init(), asr.init()]);
      await refreshModel();
      if (disposed || !store.open || flags.changingSession) return;
      if (store.prefs.read !== 'never') void speech.prepareAck?.();
      if (chat.loadError) { store.note('无法加载上次会话，请稍后重新打开小伴。'); return; }
      if (await adoptCompanionSession()) return;
      if (!chat.configured) { store.note(MODEL_REQUIRED, 'ai'); return; }
      const id = chat.currentId ?? readSession();
      if (id && chat.sessions.some(session => session.id === id)) {
        if(chat.currentId !== id) await chat.select(id);
      } else {
        rememberSession(null);
        await chat.select(null);
      }
      if (pendingPtt) { await beginTalk(); return; }
      // Typing first: the capsule waits, and a tap on the ball starts listening. A glance on hover never listens.
      if (store.open && !store.peek && asr.enabled && store.prefs.open === 'voice' && !store.input && !store.attachments.length) { voiceTurn = await startListening(); }
    },
    closed() {
      noteVoice('');
      pendingPtt = false; flags.ptt = false; reading = null;
      draftVersion++;
      clearTimers();
      asr.deactivate(); void speech.stop(); interruption = null;
      store.live = false; store.muted = false; store.liveSince = 0;
      store.open = false; store.peek = false; store.note(''); heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
    },
    async newConversation() {
      if (flags.changingSession || flags.submitting) return;
      endLive(); noteVoice('');
      flags.changingSession = true;draftVersion++;
      clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false; asr.deactivate(); interruption = null;
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
        // What is said first has to be decided before sending, so the model can be told not to say it again.
        const kind = spoken && (store.live || store.prefs.read !== 'never') ? ackKind(value) : null;
        const phrase = kind ? chooseAck(ACK_PHRASES[kind].filter(item => speech.ackReady?.(item)), lastAck) : undefined;
        const note = (interruption && interruption.session === chat.currentId ? interruption.note : '') + (phrase ? spokenNote(phrase) : '');
        const sent = note ? await chat.send(value, images, note) : images.length ? await chat.send(value, images) : await chat.send(value);
        if (sent) interruption = null;
        if (sent && store.open && (store.live || store.prefs.read === 'always' || (store.prefs.read === 'voice' && spoken))) beginReading();
        if (sent && store.open && kind) { void speech.prepareAck?.(); if (phrase) { lastAck = phrase; scheduleAck(kind, phrase); } }
        if (!sent && version === draftVersion) {
          if (!store.input) store.input = value;
          store.attachments = [...images, ...store.attachments].slice(0, 8);
        }
      } finally { flags.submitting = false; }
    },
    async toggleListening() {
      if (asr.listening || asr.recordingRequested) {
        // Ending a recording: the last words are still being recognised, let them reach the box before it closes.
        if (!store.live && !flags.ptt) await asr.finish();
        asr.deactivate(); voiceTurn = false; heard = ''; flags.partial = '';
        return;
      }
      await speech.stop(); flags.awaitingSpeech = false;
      voiceTurn = await startListening();
    },
    inputStarted() {
      draftVersion++; clearTimers(); heard = ''; voiceTurn = false;
      asr.deactivate();
    },
    interrupt() {
      // Stop both channels, including a reply that finishes while cancellation is in flight.
      noteInterruption();
      turnInterrupted = true; speechVersion++; listenVersion++; reading = null; flags.ptt = false;
      clearTimers(); heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
      asr.deactivate(); void speech.stop();
      if (view()?.running) void chat.cancel();
    },
    async decide(decision) { const request = store.approval; if (request) await chat.answerApproval(request, decision); },
    async reply(answer) { const request = store.ask; if (request) await chat.answerQuestion(request, answer); },
    replied(session, text, key) {
      // Both windows hear every answer; the one that was asked reads it, the other only shows it.
      if (chat.sentHere && !chat.sentHere(session)) return;
      if (!store.open && session === chat.currentId && !turnInterrupted) {
        store.unread = true;
        store.notice = { kind: 'done', text: noticeLine(text) || '回答好了' };
        return;
      }
      if (!store.open || session !== chat.currentId || turnInterrupted) return;
      if (reading) {
        // Already reading along: read the rest, then carry on as after any readout.
        const stream = reading, version = speechVersion; reading = null;
        void stream.end(text).then(() => {
          if (version !== speechVersion) return;
          if (flags.awaitingSpeech) { flags.awaitingSpeech = false; afterTurn(); }
          else if (!stream.spoken) afterTurn();
        });
        return;
      }
      const version = ++speechVersion;
      if (store.live || store.prefs.read === 'always' || (store.prefs.read === 'voice' && voiceTurn)) {
        readingText = text; echoTailUntil = 0;
        flags.awaitingSpeech = true;
        if (asr.enabled && asr.microphoneSupported !== false && !store.muted) void startListening(true);
        void speech.speak(text, key ?? `assistant:${session}`, undefined, true).then(() => {
          if (version === speechVersion && flags.awaitingSpeech && speech.status === 'idle') { flags.awaitingSpeech = false; afterTurn(); }
        });
      } else afterTurn();
    },
    stopped() {
      const failure = turn().failure;
      if (!store.open && failure && !turnInterrupted) { store.unread = true; store.notice = { kind: 'error', text: noticeLine(failure) }; }
      reading = null;
      // Cancelled because the user started talking: the microphone that heard them stays on.
      if (keepListening) { keepListening = false; speechVersion++; flags.awaitingSpeech = false; void speech.stop(); return; }
      speechVersion++; listenVersion++; flags.awaitingSpeech = false; clearTimers(); asr.deactivate(); void speech.stop();
      if (store.live && !store.muted && store.open) void startListening().then(ok => { voiceTurn = ok; });
    },
    note(message, tab = '') { store.hint = message; store.hintTab = message ? tab : ''; },
    noteVoice,
    async pushToTalk(down) {
      if (down) {
        if (!store.open) { pendingPtt = true; return; }
        await beginTalk();
        return;
      }
      pendingPtt = false;
      if (!flags.ptt) return;
      flags.ptt = false; clearTimers();
      // The last words are still being recognised: let them arrive before sending.
      if (asr.recordingRequested || asr.listening) await asr.finish();
      listenVersion++;
      if (asr.recordingRequested) asr.deactivate();
      const text = heard.trim(); heard = '';
      if (text) await store.submit(text, true);
      else noteVoice('没听清，按住快捷键再说一次。');
    },
    hide() { return deps.invoke<void>('assistant_hide').catch(() => {}); },
    async unfold(peek = false) { await deps.invoke('assistant_unfold', { peek }); },
    engage() { if (!store.peek) return; store.peek = false; void deps.invoke('assistant_focus').catch(() => {}); },
    async saveAnswer(text) {
      try { await deps.invoke('library_save', { item: { kind: 'answer', title: '', text, sessionId: chat.currentId, sessionTitle: store.title } }); return true; }
      catch (error) { store.note(error instanceof Error ? error.message : String(error)); return false; }
    },
    async openChat() { await deps.invoke('assistant_open_chat', { session: chat.currentId }); },
    async openSettings(tab) { await deps.invoke('assistant_open_settings', { tab }); },
    dispose() {
      disposed = true; clearTimers(); noteVoice(''); stopWatch(); stopSessionWatch(); stopVoiceWatch(); stopVoiceErrorWatch(); stopAttentionWatch(); stopReadAlongWatch();
      unsubscribers.splice(0).forEach(off => off());
      asr.deactivate(); void speech.stop();
    },
  });
  const stopReadAlongWatch = watch(() => turn().answer, text => { if (reading && text) reading.push(text); });
  const stopWatch = watch(() => speech.status, (status, before) => {
    if (flags.awaitingSpeech && status === 'idle' && before !== 'idle') {
      echoTailUntil = now() + 500; flags.awaitingSpeech = false; afterTurn();
    }
  });
  const stopVoiceWatch = watch(() => asr.enabled, enabled => {
    if (!enabled) { endLive(); clearTimers(); heard = ''; voiceTurn = false; asr.deactivate(); }
  }, { flush: 'sync' });
  const stopVoiceErrorWatch = watch(() => [asr.error, asr.microphoneError], (errors, before) => {
    if (!store.open || disposed) return;
    const message = errors.find((error, i) => error && error !== before[i]);
    if (message) noteVoice(message);
  }, { flush: 'sync' });
  // Something waits for the user while only the ball shows: say so beside it.
  const stopAttentionWatch = watch(() => store.phase === 'attention' && !store.open, waiting => {
    if (!waiting || disposed) return;
    const approval = store.approval, ask = store.ask;
    store.notice = { kind: 'attention', text: noticeLine(ask?.question ?? (approval ? `${approval.tool} 需要你许可` : '需要你确认一下')) };
  });
  const stopSessionWatch = watch(() => chat.currentId, id => {
    if(id && !disposed) rememberSession(id);
  }, { flush: 'sync' });
  return store;
}


