import { reactive, watch } from 'vue';
import type { ChatStore } from '../ui/chat/chat-store.ts';
import type { AsrStore } from '../ui/asr/store.ts';
import type { SpeechStore } from '../ui/speech/store.ts';
import type { ApprovalDecision, ApprovalRequest, ChatItem, QuestionRequest } from '../ui/chat/types.ts';

export type Phase = 'idle' | 'listening' | 'thinking' | 'answering' | 'speaking' | 'attention';

/** A conversation the capsule left less than this long ago is continued; an older one starts over. */
export const RESUME_MS = 5 * 60 * 1000;
/** Voice text is sent this long after the last words were recognised, in case more follows. */
export const SEND_GRACE_MS = 600;
/** The capsule puts itself away when nothing was said or typed for this long after it opened. */
export const QUIET_CLOSE_MS = 12_000;
/** The same, once an answer has been shown (and read). */
export const DONE_CLOSE_MS = 30_000;
const RECHECK_MS = 3_000;
/** A focus change this soon after opening is the system settling, not the user leaving. */
const SETTLE_MS = 500;

export interface AssistantDeps {
  desktop: boolean;
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  listen<T>(event: string, callback: (payload: T) => void): Promise<() => void>;
  chat: ChatStore;
  asr: AsrStore;
  speech: SpeechStore;
  now?(): number;
  setTimer?(callback: () => void, ms: number): unknown;
  clearTimer?(id: unknown): void;
}

export interface AssistantStore {
  open: boolean;
  /** The text box. Voice fills it as words are recognised, then sends it. */
  input: string;
  /** Why voice or the model is not ready, in a sentence. Empty when nothing is wrong. */
  hint: string;
  /** The settings tab that fixes the hint, if one does. */
  hintTab: '' | 'ai' | 'voice';
  /** The pointer is over the capsule, so it will not put itself away. */
  hovering: boolean;
  readonly phase: Phase;
  readonly configured: boolean;
  readonly voiceAvailable: boolean;
  readonly status: string;
  readonly question: string;
  readonly answer: string;
  readonly working: string;
  readonly failure: string;
  readonly approval: ApprovalRequest | null;
  readonly ask: QuestionRequest | null;
  init(): Promise<void>;
  opened(): Promise<void>;
  closed(): void;
  submit(text?: string, spoken?: boolean): Promise<void>;
  toggleListening(): Promise<void>;
  /** Stops speaking, or cancels the answer being written. */
  interrupt(): void;
  decide(decision: ApprovalDecision): Promise<void>;
  reply(answer: string | null): Promise<void>;
  /** Called when a reply to the open conversation is complete. */
  replied(session: string, text: string, key?: string): void;
  /** Called when the open conversation was cancelled or failed. */
  stopped(): void;
  blurred(): void;
  touch(): void;
  /** Shows a line in the card, optionally with a way to the settings tab that fixes it. */
  note(message: string, tab?: '' | 'ai' | 'voice'): void;
  hide(): Promise<void>;
  expand(): Promise<void>;
  openSettings(tab: 'ai' | 'voice'): Promise<void>;
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
  const now = deps.now ?? (() => Date.now());
  const setTimer = deps.setTimer ?? ((callback, ms) => window.setTimeout(callback, ms));
  const clearTimer = deps.clearTimer ?? (id => window.clearTimeout(id as number));
  const unsubscribers: (() => void)[] = [];
  let starting: Promise<void> | null = null;
  let disposed = false;
  let openedAt = 0, lastActivity = 0;
  let closeTimer: unknown, sendTimer: unknown;
  /** Words recognised so far in this utterance; sent together when speaking pauses. */
  let heard = '';
  /** The last question was spoken, so the answer is read aloud and listening resumes afterwards. */
  let voiceTurn = false;
  /** Reactive, because the phase shows `speaking` from the moment a reply is handed to the voice until it ends. */
  const flags = reactive({ awaitingSpeech: false });

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
    return { question, answer, working: tool ? describeTool(tool.name) : '', failure: notice?.text ?? '' };
  }

  function clearTimers() { clearTimer(closeTimer); clearTimer(sendTimer); closeTimer = sendTimer = undefined; }
  function busy() { return store.phase !== 'idle' && store.phase !== 'listening'; }
  /** Puts the capsule away after `ms` of nothing happening; it waits while the user is reading, typing or being answered. */
  function arm(ms: number) {
    clearTimer(closeTimer);
    closeTimer = setTimer(() => {
      if (disposed || !store.open) return;
      if (store.hovering || busy() || store.input.trim()) { arm(RECHECK_MS); return; }
      void store.hide();
    }, ms);
  }

  async function startListening(): Promise<boolean> {
    await asr.init();
    if (!asr.enabled) { store.note('语音还没开启。在设置的“声音”里打开后，就能直接说话。', 'voice'); return false; }
    if (!(await asr.prepare())) { store.note(asr.error || '语音识别还没准备好，请稍后再试。'); return false; }
    if (!store.open) return false;
    store.note(''); heard = '';
    await asr.activate(onHeard);
    return asr.recordingRequested;
  }
  function onHeard(text: string) {
    heard = heard ? `${heard} ${text}` : text;
    store.input = heard;
    clearTimer(sendTimer);
    sendTimer = setTimer(function flush() {
      if (!store.open) return;
      if (asr.recognizing) { sendTimer = setTimer(flush, 300); return; }
      void store.submit(heard, true);
    }, SEND_GRACE_MS);
  }
  /** What happens once an answer is complete and has been read: listen again if the user is talking, and arm the timer. */
  function afterTurn() {
    if (!store.open) return;
    if (voiceTurn && asr.enabled) void startListening();
    arm(DONE_CLOSE_MS);
  }

  const store: AssistantStore = reactive({
    open: false, input: '', hint: '', hintTab: '', hovering: false,
    get configured() { return chat.configured; },
    get voiceAvailable() { return asr.enabled; },
    get question() { return turn().question; },
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
        case 'speaking': return '正在朗读，点一下光球可以停止';
        case 'listening': return asr.preparing ? '正在准备语音识别…' : asr.recognizing ? '正在识别…' : '我在听，请说…';
        default: return asr.enabled ? '问点什么，或直接说话' : '问点什么';
      }
    },
    init() {
      starting ??= (async () => {
        if (!deps.desktop) return;
        unsubscribers.push(
          await deps.listen('assistant-open', () => { void store.opened(); }),
          await deps.listen('assistant-close', () => store.closed()),
        );
        await Promise.all([chat.init(), asr.init(), speech.init()]);
        await deps.invoke('assistant_ready');
      })();
      return starting;
    },
    async opened() {
      if (disposed) return;
      clearTimers();
      openedAt = now(); store.open = true; store.note(''); store.input = ''; heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
      await Promise.all([chat.init(), asr.init()]);
      if (!store.open) return;
      if (!chat.configured) { store.note('还没有配置模型。先到设置里添加一个，我才能回答。', 'ai'); arm(QUIET_CLOSE_MS * 2); return; }
      if (!chat.currentId || now() - lastActivity > RESUME_MS) await chat.select(null);
      arm(QUIET_CLOSE_MS);
      if (store.open && asr.enabled) { voiceTurn = await startListening(); }
    },
    closed() {
      clearTimers();
      asr.deactivate(); void speech.stop();
      store.open = false; store.input = ''; store.note(''); store.hovering = false; heard = ''; voiceTurn = false; flags.awaitingSpeech = false;
    },
    async submit(text, spoken = false) {
      const value = (text ?? store.input).trim();
      if (!value || !chat.configured) return;
      clearTimers(); heard = '';
      asr.deactivate(); await speech.stop();
      store.input = ''; store.note('');
      voiceTurn = spoken; lastActivity = now();
      if (!(await chat.send(value))) { store.input = value; arm(DONE_CLOSE_MS); }
    },
    async toggleListening() {
      store.touch();
      if (asr.listening || asr.recordingRequested) { asr.deactivate(); voiceTurn = false; heard = ''; return; }
      await speech.stop(); flags.awaitingSpeech = false;
      voiceTurn = await startListening();
    },
    interrupt() {
      if (store.phase === 'speaking') { flags.awaitingSpeech = false; void speech.stop(); afterTurn(); return; }
      if (view()?.running) void chat.cancel();
    },
    async decide(decision) { const request = store.approval; if (request) await chat.answerApproval(request, decision); },
    async reply(answer) { const request = store.ask; if (request) await chat.answerQuestion(request, answer); },
    replied(session, text, key) {
      if (!store.open || session !== chat.currentId) return;
      lastActivity = now();
      if (voiceTurn || speech.settings.autoRead) {
        flags.awaitingSpeech = true;
        void speech.speak(text, key ?? `assistant:${session}`, undefined, true).then(() => {
          if (flags.awaitingSpeech && speech.status === 'idle') { flags.awaitingSpeech = false; afterTurn(); }
        });
      } else afterTurn();
    },
    stopped() {
      flags.awaitingSpeech = false; void speech.stop();
      if (store.open) arm(DONE_CLOSE_MS);
    },
    blurred() {
      if (!store.open || now() - openedAt < SETTLE_MS) return;
      if (busy() || store.input.trim()) return;
      void store.hide();
    },
    note(message, tab = '') { store.hint = message; store.hintTab = message ? tab : ''; },
    touch() { if (store.open) arm(busy() ? DONE_CLOSE_MS : QUIET_CLOSE_MS * 2); },
    hide() { return deps.invoke<void>('assistant_hide').catch(() => {}); },
    async expand() { await deps.invoke('assistant_open_chat', { session: chat.currentId }); },
    async openSettings(tab) { await deps.invoke('assistant_open_settings', { tab }); },
    dispose() {
      disposed = true; clearTimers(); stopWatch();
      unsubscribers.splice(0).forEach(off => off());
      asr.deactivate(); void speech.stop();
    },
  });
  const stopWatch = watch(() => speech.status, (status, before) => {
    if (flags.awaitingSpeech && status === 'idle' && before !== 'idle') { flags.awaitingSpeech = false; afterTurn(); }
  });
  return store;
}
