import { describe, expect, it } from 'vitest';
import { PcmResampler, UtteranceSegmenter } from './audio.ts';
describe('microphone PCM', () => {
  it.each([44100, 48000])('converts %i Hz to 16 kHz across arbitrary frame boundaries', rate => {
    const input = Float32Array.from({ length: rate }, (_, i) => Math.sin(i / 20));
    const expected = new PcmResampler(rate).push(input);
    const stream = new PcmResampler(rate); const actual: number[] = [];
    for (let n = 0; n < input.length; n += 2048) actual.push(...stream.push(input.subarray(n, n + 2048)));
    expect(expected).toHaveLength(16000); expect(actual).toEqual(Array.from(expected));
  });
  it('does not recognize silence or a short noise click', () => {
    const segmenter = new UtteranceSegmenter();
    for (let i = 0; i < 20; i++) expect(segmenter.push(new Float32Array(1600))).toBeNull();
    expect(segmenter.push(new Float32Array(1600).fill(0.1))).toBeNull();
    for (let i = 0; i < 12; i++) expect(segmenter.push(new Float32Array(1600))).toBeNull();
  });
  it('ends an utterance after a pause and bounds continuous speech to 20 seconds', () => {
    const segmenter = new UtteranceSegmenter();
    for (let i = 0; i < 3; i++) expect(segmenter.push(new Float32Array(1600).fill(0.1))).toBeNull();
    let phrase: Float32Array | null = null;
    for (let i = 0; i < 9; i++) phrase = segmenter.push(new Float32Array(1600));
    expect(phrase?.length).toBe(19200);
    const continuous = new UtteranceSegmenter(); const parts: number[] = [];
    for (let i = 0; i < 400; i++) { const part = continuous.push(new Float32Array(1600).fill(0.1)); if (part) parts.push(part.length); }
    expect(parts).toEqual([320000, 320000]);
  });
});

describe('assistant noise guard', () => {
  const tone = (frequency: number, amplitude: number, length = 1600) => Float32Array.from({length}, (_, i) => amplitude * Math.sin(2 * Math.PI * frequency * i / 16000));
  const end = (s: UtteranceSegmenter) => { let result: Float32Array | null = null; for (let i = 0; i < 10; i++) result = s.push(new Float32Array(1600)) ?? result; return result; };
  it('rejects quiet room noise, low-frequency hum and separated keyboard-like taps', () => {
    for (const [frequency, amplitude] of [[220, .001], [30, .04]]) {
      const s = new UtteranceSegmenter(true);
      for (let i = 0; i < 30; i++) expect(s.push(tone(frequency, amplitude))).toBeNull();
      expect(end(s)).toBeNull(); expect(s.flush()).toBeNull();
    }
    const taps = new UtteranceSegmenter(true);
    for (let i = 0; i < 7; i++) { taps.push(tone(900, .1, 480)); taps.push(new Float32Array(3200)); }
    expect(end(taps)).toBeNull();
  });
  it('rejects sustained broadband hiss', () => {
    const s = new UtteranceSegmenter(true); let seed = 1;
    for (let i = 0; i < 40; i++) {
      const hiss = Float32Array.from({length:1600}, () => { seed = (seed * 1664525 + 1013904223) >>> 0; return (seed / 0x100000000 - .5) * .08; });
      expect(s.push(hiss)).toBeNull();
    }
    expect(end(s)).toBeNull();
  });
  it('keeps normal speech-like audio after background noise and offers previews without consuming it', () => {
    const s = new UtteranceSegmenter(true);
    for (let i = 0; i < 12; i++) s.push(tone(220, .001));
    for (let i = 0; i < 8; i++) s.push(tone(220, .06));
    expect(s.active).toBe(true); expect(s.preview()?.length).toBeGreaterThanOrEqual(12800);
    expect(end(s)?.length).toBeGreaterThan(12800); expect(s.active).toBe(false);
  });
});
