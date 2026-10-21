import { reactive } from 'vue';
import { UtteranceSegmenter } from './audio.ts';
import { captureMicrophone, MICROPHONE_UNSUPPORTED, supportsMicrophone, type Capture } from './capture.ts';

interface HostStatus { enabled: boolean; modelReady: boolean; engineReady: boolean; warming?: boolean; microphoneError?: string }
interface Progress { requestId: string; phase: 'downloading' | 'loading'; file: string; downloaded: number; total: number }
export interface TranscriptContext { playbackText: string }
export interface AsrActivationOptions {
  noiseGuard?: boolean;
  /** Snapshot at capture time, so delayed results retain their echo reference. */
  playbackText?(): string;
  /** Early recognition only; never inserted as a final utterance. */
  onPreview?(text: string, context: TranscriptContext): void;
}
export interface AsrDeps {
  desktop: boolean;
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  listen<T>(event: string, callback: (payload: T) => void): Promise<() => void>;
  notify(message: unknown, type?: 'success' | 'error' | 'info'): void;
  capture?: typeof captureMicrophone;
}
export interface AsrStore extends HostStatus {
  microphoneSupported: boolean;
  /** Device diagnostics appear only in sound settings, shared across native windows. */
  microphoneError: string;
  ready: boolean; saving: boolean; preparing: boolean; listening: boolean; recognizing: boolean;
  /** Loudness of the microphone, 0..1, refreshed about fifteen times a second while listening. */
  level: number;
  speechActive: boolean;
  /** The host is fetching or loading the model on its own, in the background. */
  warming: boolean;
  /** Temporary user intent for the current conversation; never persisted. */
  recordingRequested: boolean;
  phase: 'idle' | 'downloading' | 'loading'; downloaded: number; total: number; error: string;
  init(): Promise<void>; setEnabled(enabled: boolean): Promise<void>; prepare(): Promise<boolean>;
  activate(onText: (text: string, context?: TranscriptContext) => void, options?: AsrActivationOptions): Promise<void>; finish(): Promise<void>; deactivate(): void; retry(): Promise<void>; dispose(): void;
}
/** Root-mean-square loudness mapped so ordinary speech lands around 0.4 to 0.9. */
function loudness(frame: Float32Array): number {
  if (!frame.length) return 0;
  let sum = 0;
  for (let i = 0; i < frame.length; i++) sum += frame[i] * frame[i];
  return Math.min(1, Math.sqrt(sum / frame.length) * 9);
}
const message = (error: unknown) => error instanceof Error ? error.message : String(error);
export function createAsrStore(deps: AsrDeps): AsrStore {
  const microphoneSupported = Boolean(deps.capture) || supportsMicrophone();
  let initialization: Promise<void> | undefined;
  let preparing: Promise<boolean> | undefined;
  let prepareGeneration = -1;
  let unlisten: (() => void) | undefined;
  let disposed = false, generation = 0, recording = 0, hostRevision = 0, request = '';
  let capture: Capture | undefined;
  let receiver: ((text: string, context?: TranscriptContext) => void) | undefined;
  let options: AsrActivationOptions = {};
  let segmenter: UtteranceSegmenter | undefined;
  let queue: { audio: Float32Array; context?: TranscriptContext; preview?: boolean }[] = [];
  let playbackText = '', previewSamples = 0, previewTicket: number | undefined;
  let draining: Promise<void> | undefined;
  let starting: number | undefined;
  function stopCapture() {
    recording++; starting = undefined; capture?.stop(); capture = undefined; queue = []; segmenter = undefined;
    playbackText = ''; previewSamples = 0; previewTicket = undefined;
    store.recordingRequested = false; store.listening = false; store.recognizing = false; store.level = 0; store.speechActive = false;
  }
  function fail(error: unknown) { stopCapture(); store.error = message(error); deps.notify(store.error, 'error'); }
  function reportMicrophone(error: string) {
    store.microphoneError = error;
    void deps.invoke('asr_report_microphone', { error }).catch(() => {});
  }
  function captureFailed(error: unknown) { stopCapture(); reportMicrophone(message(error)); }
  async function drain(ticket: number): Promise<void> {
    if (draining) {
      await draining;
      if (queue.length && ticket === recording && store.enabled && !disposed) await drain(ticket);
      return;
    }
    const pending = (async () => {
      try {
        while (queue.length && ticket === recording && !disposed) {
          const item = queue.shift()!;
          store.recognizing = true;
          try {
            const text = await deps.invoke<string>('asr_transcribe', { samples: Array.from(item.audio) });
            if (ticket === recording && store.enabled && !disposed && text.trim()) {
              if (item.preview) options.onPreview?.(text.trim(), item.context!);
              else if (item.context) receiver?.(text.trim(), item.context);
              else receiver?.(text.trim());
            }
          } finally { if (item.preview && previewTicket === ticket) previewTicket = undefined; }
        }
      } catch (e) { if (ticket === recording && !disposed) fail(e); }
      finally {
        if (ticket === recording) store.recognizing = false;
      }
    })();
    draining = pending;
    try { await pending; }
    finally { if (draining === pending) draining = undefined; }
    if (queue.length && ticket === recording && store.enabled && !disposed) await drain(ticket);
  }
  async function start() {
    if (!receiver || !store.recordingRequested || disposed || !store.enabled || !store.engineReady || capture || starting !== undefined) return;
    const ticket = recording;
    starting = ticket;
    const activeSegmenter = new UtteranceSegmenter(options.noiseGuard);
    segmenter = activeSegmenter;
    try {
      let levelAt = 0;
      const microphone = await (deps.capture ?? captureMicrophone)(frame => {
        if (ticket !== recording || disposed) return;
        const now = Date.now();
        if (now - levelAt >= 66) { levelAt = now; store.level = loudness(frame); }
        const reference = options.playbackText?.() ?? '';
        if (reference) playbackText = reference;
        const audio = activeSegmenter.push(frame);
        store.speechActive = activeSegmenter.active;
        previewSamples += frame.length;
        if (!audio) {
          if (!activeSegmenter.pending) playbackText = '';
          if (reference && options.onPreview && previewTicket === undefined && previewSamples >= 12800) {
            const preview = activeSegmenter.preview();
            if (preview) {
              previewSamples = 0; previewTicket = ticket;
              queue.push({ audio: preview, context: { playbackText: reference }, preview: true });
              void drain(ticket);
            }
          }
          return;
        }
        if (queue.length >= 5) { fail('语音处理暂时跟不上，请稍后点击重试'); return; }
        queue.push({ audio, ...(playbackText ? { context: { playbackText } } : {}) });
        playbackText = ''; previewSamples = 0;
        void drain(ticket);
      }, error => { if (ticket === recording && !disposed) captureFailed(error); });
      if (ticket !== recording || disposed || !receiver || !store.enabled || !store.recordingRequested) { microphone.stop(); return; }
      capture = microphone; store.listening = true; store.error = ''; reportMicrophone('');
    } catch (e) { if (ticket === recording && !disposed) captureFailed(e); }
    finally { if (starting === ticket) starting = undefined; }
  }
  const store: AsrStore = reactive({
    microphoneSupported, microphoneError: '',
    ready: false, enabled: false, modelReady: false, engineReady: false, saving: false, preparing: false, recordingRequested: false,
    listening: false, recognizing: false, level: 0, speechActive: false, warming: false, phase: 'idle', downloaded: 0, total: 239549735, error: '',
    init() {
      initialization ??= (async () => {
        try {
          if (!deps.desktop) return;
          const off = await deps.listen<Progress>('asr-progress', value => {
            if (disposed || !store.enabled || value.requestId !== request) return;
            store.phase = value.phase; store.downloaded = value.downloaded; store.total = value.total;
          });
          // Settings and the floating assistant have separate stores/windows. The host
          // owns the saved switch; enabling it never requests this window's microphone.
          const offState = await deps.listen<HostStatus>('asr-state', value => {
            if (disposed) return;
            hostRevision++;
            if (store.enabled !== value.enabled) {
              generation++;
              store.saving = false;
              store.error = '';
              if (!value.enabled) {
                store.deactivate(); request = '';
                store.preparing = false; store.phase = 'idle';
              }
            }
            Object.assign(store, value, { warming: Boolean(value.warming) });
          });
          if (disposed) { off(); offState(); return; } unlisten = () => { off(); offState(); };
          const revision = hostRevision;
          const value = await deps.invoke<HostStatus>('asr_get_status');
          if (!disposed && revision === hostRevision) Object.assign(store, value);
        } catch (e) { if (!disposed) store.error = message(e); }
        finally { if (!disposed) store.ready = true; }
      })();
      return initialization;
    },
    async setEnabled(enabled) {
      if (!deps.desktop || disposed || store.saving) return;
      await store.init();
      if (disposed || store.saving) return;
      const ticket = ++generation;
      if (!enabled) store.deactivate();
      store.error = ''; store.saving = true;
      // Flip immediately so closing a download cancels its UI and never restarts the microphone.
      const before = store.enabled; store.enabled = enabled;
      try {
        const value = await deps.invoke<HostStatus>('asr_set_enabled', { enabled });
        if (ticket !== generation || disposed) return;
        Object.assign(store, value);
      } catch (e) { if (ticket === generation && !disposed) { store.enabled = before; fail(e); } }
      finally { if (ticket === generation) store.saving = false; }
      if (enabled && store.enabled && ticket === generation && !store.error) await store.prepare();
    },
    prepare() {
      if (preparing) return prepareGeneration === generation ? preparing : preparing.then(() => store.prepare());
      if (!deps.desktop || !store.enabled || disposed) return Promise.resolve(false);
      if (store.engineReady) return Promise.resolve(true);
      const ticket = generation;
      prepareGeneration = ticket;
      request = `asr-${Date.now()}-${ticket}`;
      const requestId = request;
      store.preparing = true; store.phase = store.modelReady ? 'loading' : 'downloading'; store.downloaded = 0; store.error = '';
      if (!store.modelReady) deps.notify('正在加载语音识别引擎，请稍后。首次使用需从魔搭下载约 229 MiB 模型。', 'info');
      preparing = (async () => {
        try {
          const value = await deps.invoke<HostStatus>('asr_prepare', { requestId });
          if (ticket !== generation || disposed || !store.enabled) return false;
          Object.assign(store, value); return value.engineReady;
        } catch (e) { if (ticket === generation && !disposed && store.enabled) fail(e); return false; }
        finally {
          if (request === requestId) { store.preparing = false; store.phase = 'idle'; }
          preparing = undefined;
        }
      })();
      return preparing;
    },
    async activate(onText, activationOptions = {}) {
      if (!deps.desktop || disposed) return;
      if (!store.microphoneSupported) { reportMicrophone(MICROPHONE_UNSUPPORTED); return; }
      store.deactivate(); store.error = ''; receiver = onText; store.recordingRequested = true;
      options = activationOptions;
      const ticket = recording;
      await store.init();
      if (ticket !== recording || disposed) return;
      if (!store.enabled) await store.setEnabled(true);
      if (ticket !== recording || disposed || !store.enabled || store.saving) return;
      if (await store.prepare()) { if (ticket === recording && !disposed) await start(); }
    },
    async finish() {
      if (disposed) return;
      const ticket = recording;
      store.listening = false;
      store.recordingRequested = false;
      store.speechActive = false;
      const activeMic = capture;
      capture = undefined;
      activeMic?.stop();
      if (segmenter) {
        const remaining = segmenter.flush();
        segmenter = undefined;
        if (remaining && remaining.length >= 1600) {
          queue.push({ audio: remaining, ...(playbackText ? { context: { playbackText } } : {}) });
        }
      }
      await drain(ticket);
      if (ticket === recording) {
        store.recognizing = false;
      }
    },
    deactivate() { receiver = undefined; segmenter = undefined; stopCapture(); },
    async retry() {
      store.error = '';
      if (receiver) await store.activate(receiver, options);
      else await store.prepare();
    },
    dispose() { disposed = true; generation++; receiver = undefined; stopCapture(); unlisten?.(); },
  });
  return store;
}


