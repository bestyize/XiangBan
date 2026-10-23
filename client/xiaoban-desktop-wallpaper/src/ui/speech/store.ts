import { reactive } from 'vue';
import { speechText } from './text.ts';
import { createStreamPlayer, type StreamPlayer } from '../../voice/stream-player.ts';
import { ALL_ACK_PHRASES } from '../../voice/ack.ts';
export interface VoiceOptions { voice: string; rate: number; volume: number; pitch: number }
/** Where speech comes from; online voices fall back to the system voice when they fail. */
export type SpeechProvider = 'edge' | 'gateway' | 'system';
export interface SpeechSettings extends VoiceOptions { autoRead: boolean; provider: SpeechProvider }
export interface Voice { shortName: string; gender: string; locale: string; friendlyName: string }
export const speechDefaults: SpeechSettings = { autoRead: false, provider: 'edge', voice: 'zh-CN-XiaoxiaoNeural', rate: 0, volume: 100, pitch: 0 };
export const defaultVoices: Voice[] = [
  { shortName: 'zh-CN-XiaoxiaoNeural', friendlyName: '晓晓 · 女声', gender: 'Female', locale: 'zh-CN' },
  { shortName: 'zh-CN-XiaoyiNeural', friendlyName: '晓伊 · 女声', gender: 'Female', locale: 'zh-CN' },
  { shortName: 'zh-CN-YunxiNeural', friendlyName: '云希 · 男声', gender: 'Male', locale: 'zh-CN' },
  { shortName: 'zh-CN-YunyangNeural', friendlyName: '云扬 · 男声', gender: 'Male', locale: 'zh-CN' },
];
/** MP3 from the online voices, WAV from the system voice: tell them apart by their header. */
export function audioType(data: ArrayBuffer | Uint8Array): string {
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 ? 'audio/wav' : 'audio/mpeg';
}
type AudioHandle = Pick<HTMLAudioElement, 'play' | 'pause' | 'src' | 'onended' | 'onerror'>;

export interface SpeechDeps {
  desktop: boolean;
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  notify(message: unknown, type?: 'success' | 'error' | 'info'): void;
  audio?(url: string): AudioHandle;
  url?(bytes: Blob): string;
  revoke?(url: string): void;
}
export interface SpeechStreamEvents { onStart?(): void; onSentence?(text: string): void }
/** A reading that follows a streamed answer: push the text so far, end it when the answer is complete. */
export type SpeechStream = Pick<StreamPlayer, 'push' | 'end' | 'spoken' | 'active'>;
export interface SpeechStore {
  ready: boolean; saving: boolean; settings: SpeechSettings; voices: Voice[];
  loadingVoices: boolean; status: 'idle' | 'synthesizing' | 'playing'; currentKey: string; error: string;
  init(): Promise<void>; save(patch: Partial<SpeechSettings>): Promise<void>; loadVoices(): Promise<void>;
  speak(source: string, key: string, options?: VoiceOptions, automatic?: boolean): Promise<void>;
  /** Reads an answer while it is still being written, sentence by sentence. `stop()` ends it too. */
  stream?(key: string, options?: VoiceOptions, events?: SpeechStreamEvents): SpeechStream;
  /** Synthesises the short acknowledgements for the current voice, once; later calls with the same voice do nothing. */
  prepareAck?(): Promise<void>;
  /** Whether this opening phrase has been synthesised and can be played now. */
  ackReady?(phrase: string): boolean;
  /** Plays an opening phrase ("你好呀，我在") unless something is being read already. Returns whether it started. */
  ack?(phrase: string): boolean;
  stop(): Promise<void>; dispose(): void;
}
export function createSpeechStore(deps: SpeechDeps): SpeechStore {
  let initialized: Promise<void> | null = null;
  let generation = 0, disposed = false, pendingSaves = 0;
  let audio: AudioHandle | null = null, objectUrl = '';
  let streaming: StreamPlayer | null = null;
  let cancelQueue = Promise.resolve(), saveQueue = Promise.resolve();
  /** Synthesised acknowledgements, by voice. */
  const acks = new Map<string, Uint8Array | ArrayBuffer>();
  let ackBusy: Promise<void> | null = null, ackClip: { done: Promise<void>; stop(): void } | null = null;
  const ackKey = (phrase: string) => `${store.settings.provider}|${store.settings.voice}|${store.settings.rate}|${phrase}`;
  const playable = () => store.settings.provider !== 'system';
  const createUrl = deps.url ?? (blob => URL.createObjectURL(blob));
  const revoke = deps.revoke ?? (url => URL.revokeObjectURL(url));
  function stopAck() { ackClip?.stop(); ackClip = null; }
  function release() {
    stopAck();
    streaming?.stop(); streaming = null;
    if (audio) { audio.pause(); audio.onended = null; audio.onerror = null; audio.src = ''; audio = null; }
    if (objectUrl) { revoke(objectUrl); objectUrl = ''; }
    store.currentKey = ''; store.status = 'idle';
  }
  const store: SpeechStore = reactive({
    ready: false, saving: false, settings: { ...speechDefaults }, voices: [...defaultVoices],
    loadingVoices: false, status: 'idle' as SpeechStore['status'], currentKey: '', error: '',
    init() {
      initialized ??= (async () => {
        try {
          if (deps.desktop) { const settings = await deps.invoke<SpeechSettings>('tts_get_settings'); if (!disposed && settings) store.settings = settings; }
        } catch (e) { store.error = String(e instanceof Error ? e.message : e); }
        finally { if (!disposed) store.ready = true; }
      })();
      return initialized;
    },
    save(patch) {
      pendingSaves++; store.saving = true;
      const task = saveQueue.then(async () => {
        if (!deps.desktop || disposed) return;
        await store.stop();
        const settings = { ...store.settings, ...patch };
        const saved = await deps.invoke<SpeechSettings>('tts_save_settings', { settings });
        if (!disposed) { store.settings = saved; store.error = ''; }
      }).catch(e => { if (!disposed) { store.error = String(e instanceof Error ? e.message : e); deps.notify(store.error, 'error'); } })
        .finally(() => { store.saving = --pendingSaves > 0; });
      saveQueue = task; return task;
    },
    async loadVoices() {
      if (!deps.desktop || store.loadingVoices || disposed) return;
      store.loadingVoices = true;
      try {
        const voices = await deps.invoke<Voice[]>('tts_list_voices');
        if (!disposed) { store.voices = voices.length ? voices : [...defaultVoices]; store.error = ''; }
      } catch (e) { if (!disposed) { store.error = String(e instanceof Error ? e.message : e); deps.notify(store.error, 'error'); } }
      finally { store.loadingVoices = false; }
    },
    async speak(source, key, options, automatic = false) {
      if (!deps.desktop || disposed) { if (!automatic) deps.notify('请在桌面应用中使用朗读', 'info'); return; }
      const text = speechText(source);
      if (!text) { if (!automatic) deps.notify('这条消息没有可朗读的文字', 'info'); return; }
      const stopping = store.stop(), ticket = generation;
      await stopping;
      if (ticket !== generation || disposed) return;
      store.status = 'synthesizing'; store.currentKey = key; store.error = '';
      try {
        const bytes = await deps.invoke<ArrayBuffer | number[]>('tts_synthesize', { text, options: options ?? null });
        if (ticket !== generation || disposed) return;
        const data = bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes);
        if (!data.byteLength) throw new Error('语音服务没有返回音频');
        objectUrl = createUrl(new Blob([data], { type: audioType(data) }));
        audio = deps.audio ? deps.audio(objectUrl) : new Audio(objectUrl);
        audio.onended = () => { if (ticket === generation) release(); };
        audio.onerror = () => { if (ticket === generation) { store.error = '音频播放失败，请重试'; release(); deps.notify(store.error, 'error'); } };
        await audio.play();
        if (ticket === generation && !disposed && audio) store.status = 'playing';
      } catch (e) {
        if (ticket !== generation || disposed) return;
        release(); store.error = String(e instanceof Error ? e.message : e);
        deps.notify(automatic ? '语音暂不可用，文字回复已保留；可点击消息旁的朗读重试。' : store.error, automatic ? 'info' : 'error');
      }
    },
    stream(key, options, events = {}) {
      void store.stop();
      const ticket = generation;
      store.currentKey = key; store.error = '';
      const player = createStreamPlayer({
        // The first sentence waits for an acknowledgement that is still playing instead of talking over it.
        beforeFirst: () => ackClip?.done,
        async synthesize(text) {
          // A reading stopped just before must not cancel this one's sentences on the host.
          await cancelQueue;
          if (ticket !== generation || disposed) throw new Error('朗读已停止');
          if (store.status === 'idle') store.status = 'synthesizing';
          const bytes = await deps.invoke<ArrayBuffer | number[]>('tts_synthesize_part', { text, options: options ?? null });
          const data = bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes);
          if (!data.byteLength) throw new Error('语音服务没有返回音频');
          return data;
        },
        async play(data) {
          const url = createUrl(new Blob([data as BlobPart], { type: audioType(data) }));
          const clip = deps.audio ? deps.audio(url) : new Audio(url);
          let settle!: () => void;
          const done = new Promise<void>(resolve => { settle = resolve; });
          const end = () => { clip.onended = null; clip.onerror = null; clip.pause(); revoke(url); settle(); };
          clip.onended = end; clip.onerror = end;
          await clip.play();
          return { done, stop: end };
        },
      }, {
        onStart() { if (ticket === generation) store.status = 'playing'; events.onStart?.(); },
        onSentence(text) { if (ticket === generation) store.status = 'playing'; events.onSentence?.(text); },
        onError(error) { if (ticket === generation) store.error = String(error instanceof Error ? error.message : error); },
      });
      streaming = player;
      return {
        push: full => player.push(full),
        end: full => player.end(full).then(() => { if (ticket === generation && streaming === player) { streaming = null; release(); } }),
        get spoken() { return player.spoken; },
        get active() { return player.active; },
      };
    },
    prepareAck() {
      if (!deps.desktop || disposed || !playable()) return Promise.resolve();
      const missing = ALL_ACK_PHRASES.filter(phrase => !acks.has(ackKey(phrase)));
      if (!missing.length) return Promise.resolve();
      return ackBusy ??= (async () => {
        for (const phrase of missing) {
          const key = ackKey(phrase);
          try {
            const bytes = await deps.invoke<ArrayBuffer | number[]>('tts_synthesize_part', { text: phrase, options: null });
            const data = bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes);
            if (data.byteLength && !disposed) acks.set(key, data);
          } catch { /* the acknowledgement is a courtesy; the answer is read regardless */ }
        }
      })().finally(() => { ackBusy = null; });
    },
    ackReady(phrase) { return deps.desktop && !disposed && playable() && acks.has(ackKey(phrase)); },
    ack(phrase) {
      if (!deps.desktop || disposed || !playable() || store.status !== 'idle' || ackClip) return false;
      const data = acks.get(ackKey(phrase));
      if (!data) return false;
      const url = createUrl(new Blob([data as BlobPart], { type: audioType(data) }));
      const element = deps.audio ? deps.audio(url) : new Audio(url);
      let settle!: () => void;
      const done = new Promise<void>(resolve => { settle = resolve; });
      const clip = { done, stop: () => { element.onended = null; element.onerror = null; element.pause(); revoke(url); if (ackClip === clip) ackClip = null; settle(); } };
      element.onended = clip.stop; element.onerror = clip.stop;
      ackClip = clip;
      void Promise.resolve(element.play()).catch(clip.stop);
      return true;
    },
    stop() {
      generation++; release();
      if (deps.desktop) cancelQueue = cancelQueue.then(() => deps.invoke<void>('tts_cancel')).catch(() => {});
      return cancelQueue;
    },
    dispose() { disposed = true; void store.stop(); },
  });
  return store;
}
