import { describe, expect, it } from 'vitest';
import { runPool } from '@/lib/pool';

const tick = () => new Promise((r) => setTimeout(r, 1));

describe('runPool', () => {
  it('runs every item once and never exceeds the concurrency limit', async () => {
    let running = 0, peak = 0;
    const seen: number[] = [];
    await runPool([1, 2, 3, 4, 5, 6, 7], 3, async (n) => {
      running++; peak = Math.max(peak, running);
      await tick();
      seen.push(n); running--;
    });
    expect(seen.sort()).toEqual([1, 2, 3, 4, 5, 6, 7]);
    expect(peak).toBe(3);
  });

  it('stops taking new items once asked to, letting running ones finish', async () => {
    const seen: number[] = [];
    let stop = false;
    await runPool([1, 2, 3, 4, 5, 6], 2, async (n) => { await tick(); seen.push(n); if (n === 2) stop = true; }, () => stop);
    expect(seen.length).toBeLessThan(6);
    expect(seen).toContain(2);
  });

  it('copes with empty input and a limit larger than the list', async () => {
    await runPool([], 3, async () => { throw new Error('unreachable'); });
    const seen: string[] = [];
    await runPool(['a'], 9, async (x) => { seen.push(x); });
    expect(seen).toEqual(['a']);
  });
});
