import { reactive } from 'vue';
import { isFiller, isSpeechText } from '../assistant/voice-guard.ts';
import type { AsrStore } from '../ui/asr/store.ts';

/**
 * 听写：说中文，识别出的文字直接出现在另一个应用（Claude、ChatGPT、Codex 的桌面版或终端）的输入框里。
 * 只上屏，不发送；说“发送”才按回车。不经过小伴自己的对话，也不会让小伴回答。
 */
export interface InjectRequest { target: string; text: string; press_enter: boolean }
export interface InjectResult { ok: boolean; error?: string | null; window_title?: string | null }

export interface DictationDeps {
  asr: Pick<AsrStore, 'init' | 'prepare' | 'activate' | 'finish' | 'deactivate' | 'enabled' | 'microphoneSupported' | 'recordingRequested' | 'error'>;
  inject(request: InjectRequest): Promise<InjectResult>;
  notify(message: string): void;
  /** Nothing said for this long ends dictation, so the microphone is not left open. */
  idleMs?: number;
  setTimer?(run: () => void, ms: number): unknown;
  clearTimer?(id: unknown): void;
}

export interface Dictation {
  readonly active: boolean;
  readonly starting: boolean;
  target: string;
  /** What is being said right now, before it is final. */
  readonly partial: string;
  /** The last thing put into the other app. */
  readonly last: string;
  start(target?: string): Promise<boolean>;
  stop(reason?: string): Promise<void>;
  /** The microphone was taken away (another window, or the recogniser stopped) without asking dictation to stop. */
  micLost(): void;
}

export const DICTATION_IDLE_MS = 5 * 60_000;
const SEND_WORDS = new Set(['发送', '发出去', '发出', '回车', '确认发送', '发送吧']);
const STOP_WORDS = new Set(['停止听写', '结束听写', '关闭听写', '不听写了', '听写结束']);
const strip = (text: string) => text.replace(/[\s，。！？、；：,.!?;:“”"'‘’…]/g, '');
const ASCII_WORD = /[A-Za-z0-9]/;

export function createDictation(deps: DictationDeps): Dictation {
  const { asr } = deps;
  const setTimer = deps.setTimer ?? ((run, ms) => window.setTimeout(run, ms));
  const clearTimer = deps.clearTimer ?? (id => window.clearTimeout(id as number));
  const idleMs = deps.idleMs ?? DICTATION_IDLE_MS;
  const state = reactive({ active: false, starting: false, target: 'Claude', partial: '', last: '' });
  let chain: Promise<void> = Promise.resolve();
  let idleTimer: unknown;
  let tail = '';
  let failures = 0;
  let version = 0;

  const armIdle = () => {
    clearTimer(idleTimer);
    idleTimer = setTimer(() => { if (state.active) void dictation.stop('很久没说话，已结束听写'); }, idleMs);
  };

  /** Utterances are pasted one after another, in the order they were said. */
  function enqueue(text: string, pressEnter: boolean) {
    const ticket = version;
    chain = chain.then(async () => {
      if (ticket !== version || !state.active) return;
      let result: InjectResult;
      try {
        result = await deps.inject({ target: state.target, text, press_enter: pressEnter });
      } catch (error) {
        result = { ok: false, error: error instanceof Error ? error.message : String(error) };
      }
      if (ticket !== version) return;
      if (result.ok) {
        failures = 0;
        if (text) { state.last = text; tail = text.slice(-1); }
        return;
      }
      failures++;
      deps.notify(result.error || `没能把文字送到 ${state.target}`);
      // A target that is not there fails every time; stop instead of listening to nothing.
      if (failures >= 2) void dictation.stop(`连续两次没能送到 ${state.target}，已结束听写`);
    });
  }

  function heard(text: string) {
    state.partial = '';
    const value = text.trim();
    if (!state.active || !isSpeechText(value) || isFiller(value)) return;
    armIdle();
    const command = strip(value);
    if (STOP_WORDS.has(command)) { void dictation.stop(); return; }
    if (SEND_WORDS.has(command)) { enqueue('', true); return; }
    // Two Latin words in a row need the space the recogniser leaves out between utterances; Chinese does not.
    const spaced = tail && ASCII_WORD.test(tail) && ASCII_WORD.test(value[0] ?? '') ? ` ${value}` : value;
    enqueue(spaced, false);
  }

  const dictation: Dictation = reactive({
    get active() { return state.active; },
    get starting() { return state.starting; },
    get target() { return state.target; },
    set target(value: string) { state.target = value.trim() || 'Claude'; },
    get partial() { return state.partial; },
    get last() { return state.last; },
    async start(target?: string) {
      if (state.active || state.starting) return state.active;
      if (target !== undefined) state.target = target.trim() || 'Claude';
      state.starting = true; state.last = ''; state.partial = ''; tail = ''; failures = 0;
      const ticket = ++version;
      try {
        await asr.init();
        if (ticket !== version) return false;
        if (asr.microphoneSupported === false) { deps.notify('当前设备不支持语音输入'); return false; }
        if (!asr.enabled) { deps.notify('请先在设置 → 声音里开启语音识别'); return false; }
        if (!(await asr.prepare())) { deps.notify(asr.error || '语音识别还没准备好'); return false; }
        if (ticket !== version) return false;
        state.active = true;
        await asr.activate(text => { if (ticket === version) heard(text); }, {
          noiseGuard: true,
          onPartial: text => { if (ticket === version && state.active) state.partial = text; },
        });
        if (ticket !== version) return false;
        if (!asr.recordingRequested) { state.active = false; return false; }
        armIdle();
        return true;
      } catch (error) {
        state.active = false;
        deps.notify(error instanceof Error ? error.message : String(error));
        return false;
      } finally { state.starting = false; }
    },
    async stop(reason?: string) {
      const wasActive = state.active;
      version++;
      state.active = false; state.partial = '';
      clearTimer(idleTimer); idleTimer = undefined;
      if (wasActive) {
        try { await asr.finish(); } catch { /* the microphone is already closed */ }
        asr.deactivate();
        if (reason) deps.notify(reason);
      }
    },
    micLost() {
      if (!state.active) return;
      version++;
      state.active = false; state.partial = '';
      clearTimer(idleTimer); idleTimer = undefined;
    },
  });
  return dictation;
}
