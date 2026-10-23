/**
 * 实时对话 for the main window's conversation: a hands-free loop that listens, sends what was said once the user
 * pauses, reads the answer aloud sentence by sentence while it is written, and listens again, until the user ends
 * it. Talking over an answer stops it. It runs on the local cascade (recognition on this computer → 小伴 → read
 * aloud); a cloud realtime engine can take its place later without changing the composer.
 */
import { reactive, watch } from 'vue';
import type { AsrStore, TranscriptContext } from '../ui/asr/store.ts';
import type { SpeechStore, SpeechStream } from '../ui/speech/store.ts';
import type { ChatItem } from '../ui/chat/types.ts';
import { isFiller, isPlaybackEcho, isSpeechText } from '../assistant/voice-guard.ts';
import { interruptionNote } from './interruption.ts';
import { ACK_DELAY_MS } from './ack.ts';

/** How long a pause lasts before what was heard is sent, and after the recogniser marked the sentence finished. */
export const LIVE_SEND_GRACE_MS = 300;
export const LIVE_SETTLED_GRACE_MS = 100;

export type LiveStatus = '' | '正在连接…' | '在听' | '在想' | '在说' | '已静音' | '等你确认';

interface LiveView { running: boolean; items: ChatItem[]; approvals: unknown[]; questions: unknown[] }
export interface LiveChat {
  currentId: string | null;
  current(): LiveView | null;
  send(text: string, attachments?: never[], note?: string): Promise<boolean>;
  cancel(): Promise<void>;
}
export interface LiveTalkDeps {
  chat: LiveChat;
  asr: AsrStore;
  speech: SpeechStore;
  notify(message: string, type?: 'success' | 'error' | 'info'): void;
  now?(): number;
  setTimer?(run: () => void, ms: number): unknown;
  clearTimer?(id: unknown): void;
}

export interface LiveTalk {
  active: boolean;
  muted: boolean;
  /** When it started, for the clock. 0 when not active. */
  since: number;
  /** What the recogniser has of the sentence being spoken. */
  partial: string;
  /** What was heard and not sent yet. */
  heard: string;
  readonly status: LiveStatus;
  start(): Promise<void>;
  end(): void;
  toggleMute(): Promise<void>;
  /** The chat store's onReply: the answer to read, unless the user talked over it. */
  replied(session: string, text: string, key?: string): void;
  /** The chat store's onActivity. Returns true when the live conversation took care of it, so the caller must not
   * switch off the microphone or the voice itself. */
  activity(): boolean;
  dispose(): void;
}

export function createLiveTalk(deps: LiveTalkDeps): LiveTalk {
  const { chat, asr, speech } = deps;
  const now = deps.now ?? Date.now;
  const setTimer = deps.setTimer ?? ((run, ms) => window.setTimeout(run, ms));
  const clearTimer = deps.clearTimer ?? (id => window.clearTimeout(id as number));
  let sendTimer: unknown, ackTimer: unknown;
  let listenVersion = 0, speechVersion = 0;
  let reading: SpeechStream | null = null, readingText = '', echoTailUntil = 0;
  const flags = reactive({ speaking: false });
  /** live.send is under way: the chat store's own onActivity for it must not stop the voice. */
  let sending = false;
  /** The user talked over a running answer: the cancel that follows keeps the microphone that heard them. */
  let keepListening = false;
  /** The current answer was talked over; when it arrives it is not read. */
  let interrupted = false;
  let interruption: { session: string | null; note: string } | null = null;

  const answerText = () => {
    const items = chat.current()?.items ?? [];
    let at = items.length - 1;
    while (at >= 0 && items[at].kind !== 'user') at--;
    return items.slice(at + 1).filter((item): item is Extract<ChatItem, { kind: 'assistant' }> => item.kind === 'assistant').map(item => item.text.trim()).filter(Boolean).join('\n\n');
  };
  function clearTimers() { clearTimer(sendTimer); clearTimer(ackTimer); sendTimer = ackTimer = undefined; live.partial = ''; }
  const external = (text: string, context?: TranscriptContext) =>
    isSpeechText(text) && !isFiller(text) && !(context?.playbackText && isPlaybackEcho(text, context.playbackText));

  /** Stops the answer being read, remembering how far it got for the next message. */
  function bargeIn() {
    if (reading || flags.speaking) interruption = { session: chat.currentId, note: interruptionNote(reading ? reading.spoken : null) };
    interrupted = true; speechVersion++; flags.speaking = false; reading = null;
    echoTailUntil = now() + 500;
    void speech.stop();
    if (chat.current()?.running) { keepListening = true; void chat.cancel(); }
  }

  async function listen(whileReading = false): Promise<boolean> {
    const ticket = ++listenVersion;
    const current = () => live.active && !live.muted && ticket === listenVersion;
    await asr.init();
    if (!current()) return false;
    if (asr.microphoneSupported === false) { deps.notify('当前设备不支持语音输入', 'info'); return false; }
    if (!asr.enabled) { deps.notify('请先在设置 → 声音里开启语音识别', 'info'); return false; }
    const prepared = await asr.prepare();
    if (!current()) return false;
    if (!prepared) { deps.notify(asr.error || '语音识别还没准备好', 'info'); return false; }
    await asr.activate((text, context) => { if (current()) heardSome(text, context); }, {
      noiseGuard: true,
      onPartial: text => { if (current() && !flags.speaking) live.partial = text; },
      playbackText: () => (flags.speaking && speech.status === 'playing') || now() < echoTailUntil ? readingText : '',
      onPreview: (text, context) => { if (current() && flags.speaking && external(text, context)) { clearTimers(); bargeIn(); } },
    });
    if (!current()) return false;
    return asr.recordingRequested || whileReading;
  }

  function heardSome(text: string, context?: TranscriptContext) {
    live.partial = '';
    if (!external(text, context)) return;
    if (flags.speaking || chat.current()?.running) bargeIn();
    live.heard = live.heard ? `${live.heard} ${text}` : text;
    clearTimer(sendTimer);
    sendTimer = setTimer(function flush() {
      if (!live.active) return;
      if (asr.recognizing || asr.speechActive || chat.current()?.running) { sendTimer = setTimer(flush, 300); return; }
      void send(live.heard);
    }, context?.settled ? LIVE_SETTLED_GRACE_MS : LIVE_SEND_GRACE_MS);
  }

  async function send(text: string) {
    const value = text.trim();
    live.heard = '';
    if (!value || !live.active) return;
    sending = true; listenVersion++; speechVersion++; clearTimers();
    asr.deactivate();
    let sent = false;
    try {
      await speech.stop();
      interrupted = false;
      const note = interruption && interruption.session === chat.currentId ? interruption.note : '';
      sent = note ? await chat.send(value, [], note) : await chat.send(value);
      if (sent) interruption = null;
    } finally { sending = false; }
    if (!live.active) return;
    if (!sent) { await resume(); return; }
    beginReading();
    scheduleAck();
    // Keep listening while 小伴 thinks, so the user can add to the question or take it back.
    void resume();
  }

  /** Reads the answer sentence by sentence while it is written, listening meanwhile so it can be talked over. */
  function beginReading() {
    if (!speech.stream) return;
    const ticket = speechVersion;
    readingText = ''; echoTailUntil = 0;
    reading = speech.stream(`live:${chat.currentId ?? ''}`, undefined, {
      onStart: () => {
        if (ticket !== speechVersion || !live.active) return;
        clearTimer(ackTimer); ackTimer = undefined;
        flags.speaking = true;
        if (!live.muted && !asr.recordingRequested) void listen(true);
      },
      onSentence: text => { if (ticket === speechVersion) readingText += text; },
    });
    const text = answerText();
    if (text) reading.push(text);
  }
  /** An answer that takes a moment gets a short "好的" first, so the wait is not silent. */
  function scheduleAck() {
    clearTimer(ackTimer);
    const ticket = speechVersion;
    ackTimer = setTimer(() => {
      ackTimer = undefined;
      if (ticket !== speechVersion || !live.active || flags.speaking || !chat.current()?.running || answerText()) return;
      speech.ack?.();
    }, ACK_DELAY_MS);
  }
  /** After an answer, or when one could not start: listen again. */
  async function resume() {
    if (!live.active || live.muted || asr.recordingRequested) return;
    await listen();
  }

  const live: LiveTalk = reactive({
    active: false, muted: false, since: 0, partial: '', heard: '',
    get status(): LiveStatus {
      if (!live.active) return '';
      if (live.muted) return '已静音';
      const view = chat.current();
      if (view && (view.approvals.length || view.questions.length)) return '等你确认';
      if (flags.speaking) return '在说';
      if (view?.running || sending) return '在想';
      if (asr.listening) return '在听';
      return '正在连接…';
    },
    async start() {
      if (live.active) return;
      if (asr.microphoneSupported === false) { deps.notify('当前设备不支持语音输入', 'info'); return; }
      if (!asr.enabled) { deps.notify('请先在设置 → 声音里开启语音识别', 'info'); return; }
      live.active = true; live.muted = false; live.since = now(); live.heard = ''; interrupted = false;
      void speech.prepareAck?.();
      if (!chat.current()?.running) { await speech.stop(); flags.speaking = false; }
      const listening = await listen();
      if (!listening && live.active && !live.muted) live.end();
    },
    end() {
      if (!live.active) return;
      live.active = false; live.muted = false; live.since = 0; live.heard = '';
      listenVersion++; speechVersion++; reading = null; flags.speaking = false; keepListening = false;
      clearTimers(); asr.deactivate(); void speech.stop();
    },
    async toggleMute() {
      if (!live.active) return;
      if (live.muted) { live.muted = false; await listen(flags.speaking); return; }
      live.muted = true; listenVersion++; clearTimers(); live.heard = '';
      asr.deactivate();
    },
    replied(session, text, key) {
      if (!live.active || session !== chat.currentId || interrupted) return;
      if (reading) {
        const stream = reading, version = speechVersion; reading = null;
        void stream.end(text).then(() => {
          if (version !== speechVersion) return;
          flags.speaking = false; void resume();
        });
        return;
      }
      const version = ++speechVersion;
      readingText = text; echoTailUntil = 0; flags.speaking = true;
      if (!live.muted && !asr.recordingRequested) void listen(true);
      void speech.speak(text, key ?? `live:${session}`, undefined, true).then(() => {
        if (version === speechVersion && flags.speaking && speech.status === 'idle') { flags.speaking = false; void resume(); }
      });
    },
    activity() {
      if (!live.active) return false;
      if (sending) return true;
      reading = null;
      if (keepListening) { keepListening = false; speechVersion++; flags.speaking = false; return true; }
      // Cancelled or failed from elsewhere: stop reading and carry on listening.
      speechVersion++; flags.speaking = false; void speech.stop();
      void resume();
      return true;
    },
    dispose() { live.end(); stopAnswerWatch(); stopSpeechWatch(); stopEnabledWatch(); },
  });
  const stopAnswerWatch = watch(answerText, text => { if (reading && text) reading.push(text); });
  const stopSpeechWatch = watch(() => speech.status, (status, before) => {
    if (flags.speaking && status === 'idle' && before !== 'idle') { echoTailUntil = now() + 500; flags.speaking = false; void resume(); }
  });
  const stopEnabledWatch = watch(() => asr.enabled, enabled => { if (!enabled) live.end(); });
  return live;
}
