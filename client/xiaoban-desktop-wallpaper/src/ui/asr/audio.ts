/** Streaming box filter; keeps fractional sample boundaries between worklet frames. */
export class PcmResampler {
  private remaining: number;
  private sum = 0;
  private readonly ratio: number;
  constructor(sourceRate: number) {
    if (!Number.isFinite(sourceRate) || sourceRate < 16000) throw new Error('麦克风采样率不支持');
    this.ratio = sourceRate / 16000;
    this.remaining = this.ratio;
  }
  push(input: Float32Array): Float32Array {
    const output: number[] = [];
    for (const sample of input) {
      let weight = 1;
      while (weight > 1e-8) {
        const used = Math.min(weight, this.remaining);
        this.sum += Math.max(-1, Math.min(1, sample)) * used;
        this.remaining -= used; weight -= used;
        if (this.remaining < 1e-8) { output.push(this.sum / this.ratio); this.sum = 0; this.remaining = this.ratio; }
      }
    }
    return Float32Array.from(output);
  }
}

/** Extra conservative gate for the floating assistant's hands-free microphone.
 * Suppress low rumble, hiss and energy near the learned quiet-room floor.
 * This complements the device's echo cancellation/noise suppression; it is not speaker identification.
 */
class AdaptiveVoiceGate {
  private input = 0;
  private filtered = 0;
  private noise = .001;
  voiced(frame: Float32Array, continuing: boolean): boolean {
    let raw = 0, energy = 0, crossings = 0;
    for (const sample of frame) {
      const next = .969 * (this.filtered + sample - this.input); // 80 Hz at 16 kHz
      if ((next >= 0) !== (this.filtered >= 0)) crossings++;
      this.input = sample; this.filtered = next;
      raw += sample * sample; energy += next * next;
    }
    const rms = Math.sqrt(energy / frame.length);
    const speechBand = energy >= raw * .3 && crossings / frame.length < .35;
    const voiced = speechBand && rms >= Math.max(.005, this.noise * (continuing ? 1.8 : 3));
    if (!voiced) {
      const weight = 1 - Math.exp(-frame.length / 19200);
      this.noise += (Math.min(.015, rms) - this.noise) * weight;
    }
    return voiced;
  }
}

/** Silence (16 kHz samples, 0.6 s) that ends an utterance. */
export const END_SILENCE = 9600;
/** Energy endpointing with pre-roll, minimum speech and bounded utterances. No audio is persisted. */
export class UtteranceSegmenter {
  private pre: number[] = [];
  private audio: number[] = [];
  private speech = 0;
  private silence = 0;
  private speechRun = 0;
  private longestRun = 0;
  private gate?: AdaptiveVoiceGate;
  constructor(noiseGuard = false) { if (noiseGuard) this.gate = new AdaptiveVoiceGate(); }
  get pending(): boolean { return this.audio.length > 0; }
  get active(): boolean { return this.speech >= 1600 && this.silence < END_SILENCE; }
  /** Bounded early recognition while TTS is playing; final segmentation stays intact. */
  preview(): Float32Array | null {
    if (this.speech < 9600 || this.silence > 3200 || (this.gate && this.longestRun < 3200)) return null;
    return Float32Array.from(this.audio.slice(-24000));
  }
  push(frame: Float32Array): Float32Array | null {
    if (!frame.length) return null;
    const rms = Math.sqrt(frame.reduce((sum, s) => sum + s * s, 0) / frame.length);
    const voiced = this.gate ? this.gate.voiced(frame, this.speechRun > 0) : rms >= 0.005;
    if (!this.audio.length && !voiced) {
      this.pre.push(...frame); this.pre = this.pre.slice(-4000); return null;
    }
    if (!this.audio.length) { this.audio = this.pre; this.pre = []; }
    // Split at 20 seconds, keeping the overflow as the start of the next segment.
    const available = 320000 - this.audio.length;
    this.audio.push(...frame.subarray(0, available));
    if (voiced) {
      this.speech += frame.length; this.silence = 0; this.speechRun += frame.length;
      this.longestRun = Math.max(this.longestRun, this.speechRun);
    } else {
      this.silence += frame.length;
      if (this.silence > 640) this.speechRun = 0;
    }
    if (this.audio.length < 320000 && this.silence < END_SILENCE) return null;
    const result = this.speech >= 2880 && (!this.gate || this.longestRun >= 3200) ? Float32Array.from(this.audio) : null;
    this.audio = Array.from(frame.subarray(available)); this.speech = this.audio.length && voiced ? this.audio.length : 0; this.silence = 0;
    this.speechRun = this.longestRun = this.speech;
    return result;
  }
  flush(): Float32Array | null {
    if (!this.audio.length) return null;
    const result = this.speech >= 1600 && (!this.gate || this.longestRun >= 3200) ? Float32Array.from(this.audio) : null;
    this.audio = []; this.pre = []; this.speech = 0; this.silence = 0;
    this.speechRun = this.longestRun = 0;
    return result;
  }
}
