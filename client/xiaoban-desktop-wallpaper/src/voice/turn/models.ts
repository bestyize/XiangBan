/**
 * Endpointing models: Silero VAD and Smart Turn v3, run by the local inference engine (`turn` capability) through the host.
 * The page only streams audio to them; audio crosses the IPC as 16-bit samples.
 */
type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/** "Is someone speaking": one probability per 512-sample window at 16 kHz. */
export interface Vad {
  process(samples: Float32Array): Promise<number[]>;
  reset(): void;
}
/** "Have they finished": the probability that the turn is complete. */
export interface TurnJudge { predict(audio: Float32Array): Promise<number> }
export interface TurnModels { vad: Vad; turn: TurnJudge }

export function pcm16(samples: Float32Array): number[] {
  const out = new Array<number>(samples.length);
  for (let i = 0; i < samples.length; i++) out[i] = Math.round(Math.max(-1, Math.min(1, samples[i])) * 32767);
  return out;
}

let streams = 0;
class NativeVad implements Vad {
  private stream = `vad-${Date.now().toString(36)}-${++streams}`;
  constructor(private invoke: Invoke) {}
  process(samples: Float32Array) { return this.invoke<number[]>('voice_vad', { stream: this.stream, pcm: pcm16(samples) }); }
  reset() {
    void this.invoke('voice_vad_reset', { stream: this.stream }).catch(() => {});
    this.stream = `vad-${Date.now().toString(36)}-${++streams}`;
  }
}

let ready: Promise<boolean> | undefined;
/**
 * Loads the models on the host once. Resolves to null when they are unavailable (not a desktop build, missing
 * resources); the caller then keeps loudness endpointing. Each call gets its own VAD stream.
 */
export async function loadTurnModels(invoke: Invoke): Promise<TurnModels | null> {
  ready ??= invoke<boolean>('voice_turn_ready').catch(() => false).then(ok => { if (!ok) ready = undefined; return ok; });
  if (!(await ready)) return null;
  return {
    vad: new NativeVad(invoke),
    turn: { predict: audio => invoke<number>('voice_turn_complete', { pcm: pcm16(audio) }) },
  };
}
