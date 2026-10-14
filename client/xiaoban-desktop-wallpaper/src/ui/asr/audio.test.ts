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
