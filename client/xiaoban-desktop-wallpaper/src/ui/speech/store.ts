import { reactive } from 'vue';
import { speechText } from './text.ts';
export interface VoiceOptions { voice: string; rate: number; volume: number; pitch: number }
export interface SpeechSettings extends VoiceOptions { autoRead: boolean }
export interface Voice { shortName: string; gender: string; locale: string; friendlyName: string }
export const speechDefaults: SpeechSettings = { autoRead: false, voice: 'zh-CN-XiaoxiaoNeural', rate: 0, volume: 100, pitch: 0 };
export const defaultVoices: Voice[] = [
  { shortName: 'zh-CN-XiaoxiaoNeural', friendlyName: '晓晓 · 女声', gender: 'Female', locale: 'zh-CN' },
  { shortName: 'zh-CN-XiaoyiNeural', friendlyName: '晓伊 · 女声', gender: 'Female', locale: 'zh-CN' },
  { shortName: 'zh-CN-YunxiNeural', friendlyName: '云希 · 男声', gender: 'Male', locale: 'zh-CN' },
  { shortName: 'zh-CN-YunyangNeural', friendlyName: '云扬 · 男声', gender: 'Male', locale: 'zh-CN' },
];
type AudioHandle = Pick<HTMLAudioElement, 'play' | 'pause' | 'src' | 'onended' | 'onerror'>;

export interface SpeechDeps {
  desktop: boolean;
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  notify(message: unknown, type?: 'success' | 'error' | 'info'): void;
  audio?(url: string): AudioHandle;
  url?(bytes: Blob): string;
  revoke?(url: string): void;
}
export interface SpeechStore {
  ready: boolean; saving: boolean; settings: SpeechSettings; voices: Voice[];
  loadingVoices: boolean; status: 'idle' | 'synthesizing' | 'playing'; currentKey: string; error: string;
  init(): Promise<void>; save(patch: Partial<SpeechSettings>): Promise<void>; loadVoices(): Promise<void>;
  speak(source: string, key: string, options?: VoiceOptions, automatic?: boolean): Promise<void>;
  stop(): Promise<void>; dispose(): void;
}
export function createSpeechStore(deps: SpeechDeps): SpeechStore {
  let initialized: Promise<void> | null = null;
  let generation = 0, disposed = false, pendingSaves = 0;
  let audio: AudioHandle | null = null, objectUrl = '';
  let cancelQueue = Promise.resolve(), saveQueue = Promise.resolve();
  const createUrl = deps.url ?? (blob => URL.createObjectURL(blob));
  const revoke = deps.revoke ?? (url => URL.revokeObjectURL(url));
  function release() {
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
        objectUrl = createUrl(new Blob([data], { type: 'audio/mpeg' }));
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
    stop() {
      generation++; release();
      if (deps.desktop) cancelQueue = cancelQueue.then(() => deps.invoke<void>('tts_cancel')).catch(() => {});
      return cancelQueue;
    },
    dispose() { disposed = true; void store.stop(); },
  });
  return store;
}
