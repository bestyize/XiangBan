import type * as Ort from 'onnxruntime-web';
import { FRAMES, N_MELS, lastEightSeconds, logMelFeatures } from './whisper-features.ts';

/** The pieces of onnxruntime this module uses, so tests can hand in a Node build. */
export type OrtModule = Pick<typeof Ort, 'InferenceSession' | 'Tensor' | 'env'>;

/** Silero VAD v5 at 16 kHz: 512-sample windows with 64 samples of context and a recurrent state. */
export class SileroVad {
  static readonly WINDOW = 512;
  private static readonly CONTEXT = 64;
  private state: Float32Array = new Float32Array(2 * 128);
  private context = new Float32Array(SileroVad.CONTEXT);
  private pending: number[] = [];
  /** Speech probability of the most recent window. */
  probability = 0;
  constructor(private ort: OrtModule, private session: Ort.InferenceSession) {}

  reset() { this.state = new Float32Array(2 * 128); this.context = new Float32Array(SileroVad.CONTEXT); this.pending = []; this.probability = 0; }

  /** Feeds any number of samples; returns the probability of each complete window. */
  async process(samples: Float32Array): Promise<number[]> {
    for (let i = 0; i < samples.length; i++) this.pending.push(samples[i]);
    const out: number[] = [];
    while (this.pending.length >= SileroVad.WINDOW) {
      const window = Float32Array.from(this.pending.splice(0, SileroVad.WINDOW));
      const input = new Float32Array(SileroVad.CONTEXT + SileroVad.WINDOW);
      input.set(this.context); input.set(window, SileroVad.CONTEXT);
      const result = await this.session.run({
        input: new this.ort.Tensor('float32', input, [1, input.length]),
        state: new this.ort.Tensor('float32', this.state, [2, 1, 128]),
        sr: new this.ort.Tensor('int64', BigInt64Array.from([16000n]), []),
      });
      this.state = Float32Array.from(result.stateN.data as Float32Array);
      this.context = input.slice(-SileroVad.CONTEXT);
      this.probability = (result.output.data as Float32Array)[0];
      out.push(this.probability);
    }
    return out;
  }
}

/** Smart Turn v3: the probability that the speaker has finished their turn, from the last 8 s of audio. */
export class SmartTurn {
  constructor(private ort: OrtModule, private session: Ort.InferenceSession) {}
  async predict(audio: Float32Array): Promise<number> {
    const features = logMelFeatures(lastEightSeconds(audio));
    const result = await this.session.run({ input_features: new this.ort.Tensor('float32', features, [1, N_MELS, FRAMES]) });
    const output = result[this.session.outputNames[0]];
    return (output.data as Float32Array)[0];
  }
}

export interface TurnModels { vad: SileroVad; turn: SmartTurn }

export interface ModelSources {
  /** Loads onnxruntime (the WebAssembly build in the app). */
  ort(): Promise<OrtModule>;
  silero: string;
  smartTurn: string;
}

/** Where the app finds the models and the runtime: bundled with the page, loaded on first use. */
export const bundledSources: ModelSources = {
  async ort() {
    const [ort, wasm] = await Promise.all([
      import('onnxruntime-web/wasm'),
      import('onnxruntime-web/ort-wasm-simd-threaded.wasm?url'),
    ]);
    // One thread: the desktop WebViews are not cross-origin isolated, and these models are small.
    ort.env.wasm.numThreads = 1;
    ort.env.wasm.wasmPaths = { wasm: wasm.default };
    return ort;
  },
  silero: '/voice/silero_vad.onnx',
  smartTurn: '/voice/smart-turn-v3.2.onnx',
};

let loading: Promise<{ ort: OrtModule; silero: Ort.InferenceSession; turn: Ort.InferenceSession } | null> | undefined;
/**
 * Loads both models once per window. Resolves to null when they cannot run here (old WebView, missing
 * files); the caller then falls back to plain energy endpointing.
 */
export async function loadTurnModels(sources: ModelSources = bundledSources): Promise<TurnModels | null> {
  loading ??= (async () => {
    try {
      const ort = await sources.ort();
      const options = { executionProviders: ['wasm'], graphOptimizationLevel: 'all' as const };
      const [silero, turn] = await Promise.all([
        ort.InferenceSession.create(sources.silero, options),
        ort.InferenceSession.create(sources.smartTurn, options),
      ]);
      return { ort, silero, turn };
    } catch (error) {
      console.warn('语音判停模型加载失败，改用音量判停', error);
      loading = undefined;
      return null;
    }
  })();
  const loaded = await loading;
  // Each listener gets its own VAD state; the sessions are shared.
  return loaded ? { vad: new SileroVad(loaded.ort, loaded.silero), turn: new SmartTurn(loaded.ort, loaded.turn) } : null;
}
