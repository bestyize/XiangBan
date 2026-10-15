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

/** Energy endpointing with pre-roll, minimum speech and bounded utterances. No audio is persisted. */
export class UtteranceSegmenter {
  private pre: number[] = [];
  private audio: number[] = [];
  private speech = 0;
  private silence = 0;
  push(frame: Float32Array): Float32Array | null {
    if (!frame.length) return null;
    const rms = Math.sqrt(frame.reduce((sum, s) => sum + s * s, 0) / frame.length);
    const voiced = rms >= 0.005;
    if (!this.audio.length && !voiced) {
      this.pre.push(...frame); this.pre = this.pre.slice(-4000); return null;
    }
    if (!this.audio.length) { this.audio = this.pre; this.pre = []; }
    // Split at 20 seconds, keeping the overflow as the start of the next segment.
    const available = 320000 - this.audio.length;
    this.audio.push(...frame.subarray(0, available));
    if (voiced) { this.speech += frame.length; this.silence = 0; } else this.silence += frame.length;
    if (this.audio.length < 320000 && this.silence < 14400) return null;
    const result = this.speech >= 2880 ? Float32Array.from(this.audio) : null;
    this.audio = Array.from(frame.subarray(available)); this.speech = this.audio.length && voiced ? this.audio.length : 0; this.silence = 0;
    return result;
  }
  flush(): Float32Array | null {
    if (!this.audio.length) return null;
    const result = this.speech >= 1600 ? Float32Array.from(this.audio) : null;
    this.audio = []; this.pre = []; this.speech = 0; this.silence = 0;
    return result;
  }
}
