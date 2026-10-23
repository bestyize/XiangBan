import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { SmartSegmenter } from './segmenter.ts';
import type { SileroVad } from './models.ts';

const FRAME = 640; // 40 ms at 16 kHz
/** A fake VAD that treats loud samples as speech, one probability per 512 samples like Silero. */
function fakeVad() {
  let pending: number[] = [];
  return {
    reset: vi.fn(() => { pending = []; }),
    process: vi.fn(async (samples: Float32Array) => {
      pending.push(...samples);
      const out: number[] = [];
      while (pending.length >= 512) { const w = pending.splice(0, 512); out.push(Math.abs(w[0]) > 0.05 ? 0.9 : 0.05); }
      return out;
    }),
  } as unknown as SileroVad;
}
const speech = () => new Float32Array(FRAME).fill(0.2);
const quiet = () => new Float32Array(FRAME);

async function run(segmenter: SmartSegmenter, frames: Float32Array[]) {
  const out: Float32Array[] = [];
  for (const frame of frames) { const audio = segmenter.push(frame); if (audio) out.push(audio); await flushPromises(); }
  return out;
}

describe('SmartSegmenter', () => {
  it('ends quickly after a short pause when Smart Turn says the sentence is complete', async () => {
    const judge = { predict: vi.fn(async () => 0.9) };
    const segmenter = new SmartSegmenter(fakeVad(), judge);
    const out = await run(segmenter, [...Array(25)].map(speech).concat([...Array(8)].map(quiet)));
    expect(judge.predict).toHaveBeenCalledOnce();
    expect(out).toHaveLength(1);
    // About a second of speech plus the pre-roll and a short pause: ended well before the 1.6 s fallback.
    expect(out[0].length).toBeLessThan(25 * FRAME + 8 * FRAME);
  });
  it('waits out a mid-sentence pause and keeps it in one utterance', async () => {
    const judge = { predict: vi.fn(async () => 0.1) };
    const segmenter = new SmartSegmenter(fakeVad(), judge);
    const first = await run(segmenter, [...Array(15)].map(speech).concat([...Array(22)].map(quiet)));
    expect(first).toEqual([]); expect(judge.predict).toHaveBeenCalledTimes(2);
    const rest = await run(segmenter, [...Array(15)].map(speech).concat([...Array(45)].map(quiet)));
    expect(rest).toHaveLength(1);
    expect(rest[0].length).toBeGreaterThan(45 * FRAME);
  });
  it('falls back to the long pause when Smart Turn fails', async () => {
    const onError = vi.fn();
    const segmenter = new SmartSegmenter(fakeVad(), { predict: async () => { throw new Error('wasm'); } }, onError);
    const out = await run(segmenter, [...Array(15)].map(speech).concat([...Array(45)].map(quiet)));
    expect(out).toHaveLength(1); expect(onError).toHaveBeenCalled();
  });
  it('ignores clicks shorter than a syllable', async () => {
    const judge = { predict: vi.fn(async () => 0.9) };
    const segmenter = new SmartSegmenter(fakeVad(), judge);
    const out = await run(segmenter, [speech(), ...[...Array(50)].map(quiet)]);
    expect(out).toEqual([]); expect(judge.predict).not.toHaveBeenCalled(); expect(segmenter.pending).toBe(false);
  });
  it('offers the utterance so far for live captions and flushes it on release', async () => {
    const segmenter = new SmartSegmenter(fakeVad(), { predict: async () => 0.1 });
    await run(segmenter, [...Array(30)].map(speech));
    expect(segmenter.partial()!.length).toBeGreaterThanOrEqual(29 * FRAME);
    expect(segmenter.active).toBe(true);
    expect(segmenter.flush()!.length).toBeGreaterThanOrEqual(30 * FRAME);
    expect(segmenter.pending).toBe(false);
  });
});
