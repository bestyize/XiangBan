// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { drawEffects } from '../../effects/draw.ts';
import { parsePack } from '../../effects/parse.ts';
import { subscribeEffectStill } from './effect-stills.ts';

vi.mock('../../effects/draw.ts', () => ({ drawEffects: vi.fn() }));
let serial = 0;
function pack() {
  const parsed = parsePack({ schemaVersion: 1, id: `sample-${++serial}`, name: '雪', layers: [{ count: 8, shape: { kind: 'circle' }, colors: ['#fff'] }] });
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.pack;
}

describe('effect still cache and render queue', () => {
  const disposals: (() => void)[] = [];
  beforeEach(() => {
    vi.useFakeTimers(); vi.mocked(drawEffects).mockClear();
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({} as CanvasRenderingContext2D);
    vi.spyOn(HTMLCanvasElement.prototype, 'toBlob').mockImplementation(callback => callback(new Blob()));
    vi.spyOn(URL, 'createObjectURL').mockImplementation(() => `blob:still-${++serial}`);
  });
  afterEach(async () => {
    disposals.splice(0).forEach(dispose => dispose());
    await vi.runAllTimersAsync(); vi.useRealTimers(); vi.restoreAllMocks();
  });

  it('draws at most one pack per tick and shares pending/completed work by content', async () => {
    const first = pack(), ready = vi.fn(), duplicate = vi.fn(), second = vi.fn();
    disposals.push(subscribeEffectStill(first, ready), subscribeEffectStill(JSON.parse(JSON.stringify(first)), duplicate), subscribeEffectStill(pack(), second));
    expect(drawEffects).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(16);
    expect(drawEffects).toHaveBeenCalledTimes(1);
    expect(ready).toHaveBeenCalledOnce();
    expect(duplicate).toHaveBeenCalledWith(ready.mock.calls[0][0]);
    expect(second).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(16);
    expect(drawEffects).toHaveBeenCalledTimes(2);
    const cached = vi.fn();
    disposals.push(subscribeEffectStill(first, cached));
    expect(cached).toHaveBeenCalledWith(ready.mock.calls[0][0]);
    expect(drawEffects).toHaveBeenCalledTimes(2);
    expect(document.querySelector('canvas')).toBeNull();
  });

  it('regenerates when a pack with the same id has different content', async () => {
    const first = pack(), before = vi.fn(), after = vi.fn();
    disposals.push(subscribeEffectStill(first, before));
    await vi.runAllTimersAsync();
    const changed = JSON.parse(JSON.stringify(first));
    changed.layers[0].count += 1;
    disposals.push(subscribeEffectStill(changed, after));
    expect(after).not.toHaveBeenCalled();
    await vi.runAllTimersAsync();
    expect(after.mock.calls[0][0]).not.toBe(before.mock.calls[0][0]);
    expect(drawEffects).toHaveBeenCalledTimes(2);
  });

  it('skips cancelled work and allows it to be requested on return', async () => {
    const current = pack(), cancelled = vi.fn(), resumed = vi.fn();
    const dispose = subscribeEffectStill(current, cancelled);
    dispose(); await vi.runAllTimersAsync();
    expect(drawEffects).not.toHaveBeenCalled();
    disposals.push(subscribeEffectStill(current, resumed));
    await vi.runAllTimersAsync();
    expect(cancelled).not.toHaveBeenCalled();
    expect(resumed).toHaveBeenCalledOnce();
  });

  it('continues the queue after encoding fails and retries on a later subscription', async () => {
    const failed = pack(), ready = vi.fn(), next = vi.fn();
    vi.mocked(HTMLCanvasElement.prototype.toBlob).mockImplementationOnce(callback => callback(null));
    disposals.push(subscribeEffectStill(failed, ready), subscribeEffectStill(pack(), next));
    await vi.runAllTimersAsync();
    expect(ready).not.toHaveBeenCalled(); expect(next).toHaveBeenCalledOnce();
    disposals.push(subscribeEffectStill(failed, vi.fn()));
    await vi.runAllTimersAsync();
    expect(ready).toHaveBeenCalledOnce();
  });
});
