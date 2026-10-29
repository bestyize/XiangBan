import type { TurnJudge, Vad } from './models.ts';

/** What the speech recogniser needs from an endpointer; the energy `UtteranceSegmenter` has the same shape. */
export interface Segmenter {
  /** Takes a 16 kHz frame; returns a finished utterance when the speaker is done. */
  push(frame: Float32Array): Float32Array | null;
  /** Ends listening: whatever was said so far. */
  flush(): Float32Array | null;
  /** Recent speech for early recognition while the assistant is talking (barge-in). */
  preview(): Float32Array | null;
  /** The utterance so far, for live captions while the user is still talking. */
  partial?(): Float32Array | null;
  /**
   * Once per pause, as soon as the pause is long enough to ask whether the speaker has finished: the utterance
   * so far, to be recognised while that question is still being answered. `mark` identifies the utterance.
   */
  speculative?(): { audio: Float32Array; mark: string } | null;
  /** What the last returned utterance was: its `mark`, and whether the speaker was judged to have finished. */
  readonly lastCut?: { mark: string; settled: boolean } | null;
  /** Set while the assistant is talking: its own voice leaking into the microphone must not count as speech. */
  strict?: boolean;
  readonly pending: boolean;
  readonly active: boolean;
}

const RATE = 16000;
/** Speech probability at which Silero counts a window as voiced. */
const VOICED = 0.5;
/** The same, while the assistant is talking: only clearly voiced windows count, so its echo is not mistaken for the user. */
const VOICED_STRICT = 0.75;
/** Audio kept from before speech starts, so the first syllable is not clipped. */
const PRE_ROLL = 4000;
/** A pause this long asks Smart Turn whether the speaker has finished. */
const ASK_AFTER = 0.15 * RATE;
/** Smart Turn said "not finished": ask once more after this much silence. */
const ASK_AGAIN_AFTER = 0.8 * RATE;
/** Silence that ends the turn whatever Smart Turn thinks, and the fallback when it cannot answer. */
const GIVE_UP_AFTER = 1.6 * RATE;
/** Speech shorter than this is a cough or a click, not a turn. */
const MIN_SPEECH = 0.25 * RATE;
/** The same, while the assistant is talking. */
const MIN_SPEECH_STRICT = 0.4 * RATE;
const MAX_UTTERANCE = 20 * RATE;

/**
 * Endpointing with Silero VAD for "is someone speaking" and Smart Turn v3 for "have they finished".
 * A short pause is enough when the sentence sounds complete; a pause mid-thought ("我想要…") is waited out.
 * Model calls are asynchronous; their verdicts are applied on the next frame, which arrives every ~40 ms.
 */
export class SmartSegmenter implements Segmenter {
  private frames: { at: number; data: Float32Array }[] = [];
  /** Samples received, and samples the VAD has looked at. */
  private total = 0;
  private analysed = 0;
  private speechStart: number | null = null;
  private lastVoiced = 0;
  private voicedSamples = 0;
  private asked = 0;
  private verdict: 'none' | 'pending' | 'complete' | 'incomplete' = 'none';
  private ticket = 0;
  private ready = false;
  private chain: Promise<void> = Promise.resolve();
  private broken = false;
  private speculated = '';
  strict = false;
  lastCut: { mark: string; settled: boolean } | null = null;

  constructor(private vad: Vad, private judge: TurnJudge, private onError: (error: unknown) => void = () => {}) {}

  get pending() { return this.speechStart !== null; }
  get active() { return this.speechStart !== null && this.silence < GIVE_UP_AFTER; }
  private get silence() { return this.analysed - this.lastVoiced; }
  private get minSpeech() { return this.strict ? MIN_SPEECH_STRICT : MIN_SPEECH; }
  private get mark() { return `${this.speechStart}:${this.lastVoiced}`; }

  push(frame: Float32Array): Float32Array | null {
    if (!frame.length) return null;
    this.frames.push({ at: this.total, data: frame });
    this.total += frame.length;
    if (!this.broken) this.chain = this.chain.then(() => this.analyse(frame)).catch(error => { this.broken = true; this.onError(error); });
    this.prune();
    if (this.speechStart !== null && this.total - this.speechStart >= MAX_UTTERANCE) return this.cut(this.total, true);
    if (this.ready) return this.cut(this.analysed);
    return null;
  }

  flush(): Float32Array | null {
    const start = this.speechStart;
    const audio = start !== null && this.voicedSamples >= 1600 ? this.slice(start - PRE_ROLL, this.total) : null;
    this.reset();
    return audio;
  }

  preview(): Float32Array | null {
    if (this.speechStart === null || this.voicedSamples < 9600 || this.silence > 3200) return null;
    return this.slice(this.total - 24000, this.total);
  }

  partial(): Float32Array | null {
    if (this.speechStart === null || this.voicedSamples < RATE * 0.6) return null;
    return this.slice(this.speechStart - PRE_ROLL, this.total);
  }

  speculative(): { audio: Float32Array; mark: string } | null {
    if (this.speechStart === null || this.ready || this.voicedSamples < this.minSpeech || this.silence < ASK_AFTER) return null;
    const mark = this.mark;
    if (mark === this.speculated) return null;
    this.speculated = mark;
    return { audio: this.slice(this.speechStart - PRE_ROLL, this.analysed), mark };
  }

  private async analyse(frame: Float32Array) {
    const probabilities = await this.vad.process(frame);
    for (const p of probabilities) {
      this.analysed += 512;
      if (p >= (this.strict ? VOICED_STRICT : VOICED)) {
        if (this.speechStart === null) { this.speechStart = Math.max(0, this.analysed - 512); this.voicedSamples = 0; }
        this.lastVoiced = this.analysed; this.voicedSamples += 512;
        // Talking again: any verdict about the pause before is stale.
        if (this.verdict !== 'none') { this.verdict = 'none'; this.asked = 0; this.ticket++; }
      }
    }
    this.decide();
  }

  private decide() {
    if (this.speechStart === null || this.ready) return;
    const silence = this.silence;
    if (silence >= GIVE_UP_AFTER) {
      if (this.voicedSamples >= this.minSpeech) this.ready = true; else this.reset(false);
      return;
    }
    if (this.voicedSamples < this.minSpeech || this.verdict === 'pending') return;
    if (this.verdict === 'complete') { this.ready = true; return; }
    const due = this.asked === 0 ? ASK_AFTER : this.asked === 1 ? ASK_AGAIN_AFTER : Infinity;
    if (silence < due) return;
    this.asked++;
    this.verdict = 'pending';
    const ticket = this.ticket, audio = this.slice(this.speechStart - PRE_ROLL, this.analysed);
    void this.judge.predict(audio).then(probability => {
      if (ticket !== this.ticket || this.speechStart === null) return;
      this.verdict = probability >= 0.5 ? 'complete' : 'incomplete';
      this.decide();
    }, error => {
      // No judge: fall back to waiting for the long pause.
      if (ticket === this.ticket) this.verdict = 'incomplete';
      this.onError(error);
    });
  }

  private cut(end: number, continuing = false): Float32Array | null {
    const start = this.speechStart!;
    const audio = this.slice(start - PRE_ROLL, end);
    this.lastCut = { mark: continuing ? '' : this.mark, settled: !continuing && this.verdict === 'complete' };
    this.reset(false);
    // A 20-second monologue goes on: the next utterance starts where this one was cut.
    if (continuing) { this.speechStart = end; this.lastVoiced = Math.max(this.lastVoiced, this.analysed); this.voicedSamples = MIN_SPEECH; }
    return audio;
  }

  private reset(all = true) {
    this.speechStart = null; this.voicedSamples = 0; this.asked = 0; this.verdict = 'none'; this.ready = false; this.ticket++;
    if (all) { this.frames = []; this.vad.reset(); this.chain = Promise.resolve(); this.total = this.analysed = this.lastVoiced = 0; this.speculated = ''; this.lastCut = null; }
  }

  /** Drops audio that can no longer belong to an utterance. */
  private prune() {
    const keepFrom = this.speechStart !== null ? this.speechStart - PRE_ROLL : this.total - PRE_ROLL - 8192;
    while (this.frames.length > 1 && this.frames[0].at + this.frames[0].data.length < keepFrom) this.frames.shift();
  }

  private slice(from: number, to: number): Float32Array {
    from = Math.max(0, from); to = Math.min(this.total, to);
    const out = new Float32Array(Math.max(0, to - from));
    for (const { at, data } of this.frames) {
      const a = Math.max(from, at), b = Math.min(to, at + data.length);
      if (a < b) out.set(data.subarray(a - at, b - at), a - from);
    }
    return out;
  }
}
