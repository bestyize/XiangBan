// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const spies = vi.hoisted(() => ({ created: [] as unknown[][], updates: [] as unknown[][], destroyed: 0, snapshot: { canvasPresent: true } }));
vi.mock('./effects/layer.ts', () => ({
  EffectsLayer: class {
    constructor(...args: unknown[]) { spies.created.push(args); }
    update(...args: unknown[]) { spies.updates.push(args); }
    snapshot() { return spies.snapshot; }
    destroy() { spies.destroyed++; }
  },
}));
const invoke = vi.hoisted(() => vi.fn(async () => undefined));
vi.mock('./bridge.ts', async (original) => ({ ...(await original<typeof import('./bridge.ts')>()), desktop: true, invoke }));

const raw = (id: string, extra: Record<string, unknown> = {}) => ({ schemaVersion: 1, id, name: id, layers: [{ count: 10, shape: { kind: 'circle' }, colors: ['#fff'] }], ...extra });
const packOf = (call: unknown[]) => call[0] as { id: string } | null;

describe('desktop effects layer', () => {
  beforeEach(() => { spies.created.length = 0; spies.updates.length = 0; spies.destroyed = 0; invoke.mockClear(); document.body.innerHTML = ''; });
  // Each test mounts a layer on the shared window; leaving the page releases its listeners.
  afterEach(() => { window.dispatchEvent(new Event('pagehide')); delete window.__XIAOBAN_EFFECTS__; });

  it("draws the host's effect over a transparent page and reports it ready", async () => {
    window.__XIAOBAN_EFFECTS__ = { effect: raw('petals'), params: { density: 1.4, speed: 0.5, size: 2 }, paused: false, fps: 30 };
    const { mountEffects } = await import('./effects-scene.ts');
    await mountEffects();
    expect(document.body.className).toBe('effects-scene');
    expect(spies.created).toHaveLength(1);
    const [, pack, params, config] = spies.created[0] as [unknown, { id: string }, unknown, unknown];
    expect(pack.id).toBe('petals'); expect(params).toEqual({ density: 1.4, speed: 0.5, size: 2 }); expect(config).toEqual({ paused: false, fps: 30 });
    expect(invoke).toHaveBeenCalledWith('scene_ready', { report: { kind: 'effects', effects: spies.snapshot } });
  });
  it('shows nothing, and does not fall over, when the pack it was given is not valid', async () => {
    window.__XIAOBAN_EFFECTS__ = { effect: raw('x', { schemaVersion: 9 }), params: {}, paused: false, fps: 60 };
    const { mountEffects } = await import('./effects-scene.ts');
    await mountEffects();
    expect(spies.created[0][1]).toBeNull();
    window.dispatchEvent(new CustomEvent('effects-config', { detail: { effect: null } }));
    expect(packOf(spies.updates.at(-1)!)).toBeNull();
  });
  it('follows later changes, keeping the same pack object while the data is the same', async () => {
    window.__XIAOBAN_EFFECTS__ = { effect: raw('rain'), params: {}, paused: false, fps: 60 };
    const { mountEffects } = await import('./effects-scene.ts');
    await mountEffects();
    const first = spies.created[0][1];
    window.dispatchEvent(new CustomEvent('effects-config', { detail: { effect: raw('rain'), params: { density: 2 }, paused: true, fps: 30 } }));
    const same = spies.updates.at(-1)!;
    expect(same[0]).toBe(first);
    expect(same[1]).toEqual({ density: 2, speed: 1, size: 1 }); expect(same[2]).toEqual({ paused: true, fps: 30 });
    window.dispatchEvent(new CustomEvent('effects-config', { detail: { effect: raw('rain', { name: '改过的' }) } }));
    expect(spies.updates.at(-1)![0]).not.toBe(first);
    window.dispatchEvent(new CustomEvent('effects-config', { detail: { effect: raw('stars') } }));
    expect(packOf(spies.updates.at(-1)!)?.id).toBe('stars');
  });
  it('stops when the page goes away', async () => {
    window.__XIAOBAN_EFFECTS__ = { effect: raw('rain'), params: {}, paused: false, fps: 60 };
    const { mountEffects } = await import('./effects-scene.ts');
    await mountEffects();
    window.dispatchEvent(new Event('pagehide'));
    expect(spies.destroyed).toBe(1);
    const updates = spies.updates.length;
    window.dispatchEvent(new CustomEvent('effects-config', { detail: { paused: true } }));
    expect(spies.updates).toHaveLength(updates);
  });
  it('in the isolated test profile proves it cannot reach commands that change the desktop', async () => {
    window.__XIAOBAN_EFFECTS__ = { effect: raw('rain'), params: {}, paused: false, fps: 60, test: true };
    invoke.mockImplementationOnce(async () => { throw new Error('blocked'); });
    const { mountEffects } = await import('./effects-scene.ts');
    await mountEffects();
    expect(invoke).toHaveBeenNthCalledWith(1, 'set_autostart', { enabled: false });
    expect(invoke).toHaveBeenNthCalledWith(2, 'scene_ready', { report: { kind: 'effects', effects: spies.snapshot, privilegedIpcBlocked: true } });
  });
});
