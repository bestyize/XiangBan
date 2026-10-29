import { afterEach, describe, expect, it, vi } from 'vitest';
import { ballDisc, glassFrame, HoverIntent, orbFrame } from './morph.ts';

describe('ball and capsule stand-ins', () => {
  it('draws the ball disc in the middle of its square', () => {
    expect(ballDisc({ x: 408, y: 520, size: 72 })).toEqual({ left: 422, top: 534, width: 44, height: 44 });
  });
  it('lays the travelling mark exactly over a disc of any size', () => {
    // The ball: 44px, so no scaling.
    expect(orbFrame({ left: 422, top: 534, width: 44, height: 44 })).toEqual({ transform: 'translate(422px, 534px) scale(1)' });
    // The capsule's 32px button: centred on it and scaled down.
    expect(orbFrame({ left: 26, top: 540, width: 32, height: 32 })).toEqual({ transform: 'translate(20px, 534px) scale(0.7273)' });
  });
  it('describes the glass as a box with a radius', () => {
    expect(glassFrame({ left: 16, top: 528, width: 448, height: 52 }, 26, 1))
      .toEqual({ left: '16px', top: '528px', width: '448px', height: '52px', borderRadius: '26px', opacity: 1 });
  });
});

describe('hover intent', () => {
  afterEach(() => vi.useRealTimers());
  it('fires once the pointer has rested, not while it is crossing', () => {
    vi.useFakeTimers();
    const fire = vi.fn(), intent = new HoverIntent(fire, 280, 6);
    intent.enter(0, 0);
    for (let i = 1; i <= 5; i++) { vi.advanceTimersByTime(100); intent.move(i * 10, 0); }
    expect(fire).not.toHaveBeenCalled();
    intent.move(53, 2); vi.advanceTimersByTime(279); expect(fire).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1); expect(fire).toHaveBeenCalledOnce();
  });
  it('does nothing after the pointer leaves', () => {
    vi.useFakeTimers();
    const fire = vi.fn(), intent = new HoverIntent(fire);
    intent.enter(5, 5); vi.advanceTimersByTime(200); intent.cancel(); vi.advanceTimersByTime(1000);
    expect(fire).not.toHaveBeenCalled(); expect(intent.waiting).toBe(false);
  });
});
