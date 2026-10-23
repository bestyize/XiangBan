import { describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { createStreamPlayer, type Clip } from './stream-player.ts';

function harness(ahead = 2) {
  const synthesized: string[] = [];
  const played: string[] = [];
  const clips: { text: string; end(): void; stop: ReturnType<typeof vi.fn> }[] = [];
  const pendingAudio = new Map<string, (bytes: Uint8Array) => void>();
  const deps = {
    ahead,
    synthesize: vi.fn((text: string) => { synthesized.push(text); return new Promise<Uint8Array>(resolve => pendingAudio.set(text, resolve)); }),
    play: vi.fn(async (audio: ArrayBuffer | Uint8Array): Promise<Clip> => {
      const text = new TextDecoder().decode(audio as Uint8Array);
      played.push(text);
      let end!: () => void;
      const done = new Promise<void>(resolve => { end = resolve; });
      const stop = vi.fn(() => end());
      clips.push({ text, end, stop });
      return { done, stop };
    }),
  };
  const ready = (text: string) => pendingAudio.get(text)!(new TextEncoder().encode(text));
  return { deps, synthesized, played, clips, ready };
}

describe('createStreamPlayer', () => {
  it('starts reading the first sentence before the answer is complete', async () => {
    const h = harness(); const onStart = vi.fn();
    const player = createStreamPlayer(h.deps, { onStart });
    player.push('第一句。第二');
    expect(h.synthesized).toEqual(['第一句。']);
    h.ready('第一句。'); await flushPromises();
    expect(h.played).toEqual(['第一句。']); expect(onStart).toHaveBeenCalledOnce(); expect(player.playing).toBe(true);
    player.push('第一句。第二句。');
    expect(h.synthesized).toEqual(['第一句。', '第二句。']);
  });
  it('synthesises ahead but plays strictly in order, then resolves after the last sentence', async () => {
    const h = harness(1);
    const player = createStreamPlayer(h.deps);
    player.push('一。二。三。');
    expect(h.synthesized).toEqual(['一。', '二。']);
    h.ready('二。'); await flushPromises(); expect(h.played).toEqual([]);
    h.ready('一。'); await flushPromises(); expect(h.played).toEqual(['一。']);
    const done = vi.fn(); void player.end().then(done);
    h.clips[0].end(); await flushPromises();
    expect(h.played).toEqual(['一。', '二。']); expect(h.synthesized).toEqual(['一。', '二。', '三。']);
    h.clips[1].end(); h.ready('三。'); await flushPromises(); h.clips[2].end(); await flushPromises();
    expect(done).toHaveBeenCalledOnce(); expect(player.spoken).toBe('一。二。三。'); expect(player.active).toBe(false);
  });
  it('stops at once: the playing clip ends, nothing else is played and end resolves', async () => {
    const h = harness();
    const player = createStreamPlayer(h.deps);
    player.push('一。二。'); h.ready('一。'); h.ready('二。'); await flushPromises();
    const done = vi.fn(); void player.end().then(done);
    player.stop(); await flushPromises();
    expect(h.clips[0].stop).toHaveBeenCalledOnce(); expect(h.played).toEqual(['一。']); expect(done).toHaveBeenCalledOnce();
    player.push('一。二。三。'); expect(h.synthesized).toHaveLength(2);
  });
  it('skips a sentence that fails and reads the rest', async () => {
    const h = harness(); const onError = vi.fn();
    h.deps.synthesize.mockImplementationOnce(() => Promise.reject(new Error('网络错误')));
    const player = createStreamPlayer(h.deps, { onError });
    player.push('坏的。好的。'); h.ready('好的。'); await flushPromises();
    expect(onError).toHaveBeenCalledOnce(); expect(h.played).toEqual(['好的。']);
  });
  it('reads the unfinished tail when the answer ends', async () => {
    const h = harness();
    const player = createStreamPlayer(h.deps);
    player.push('没有句号的回答'); expect(h.synthesized).toEqual([]);
    void player.end('没有句号的回答'); expect(h.synthesized).toEqual(['没有句号的回答']);
  });
});
