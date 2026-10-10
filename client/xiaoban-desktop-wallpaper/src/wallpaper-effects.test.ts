import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EffectParticles, normalizeWallpaperEffects, WallpaperEffects, wallpaperEffectsDefaults } from "./wallpaper-effects.ts";

const rain = () => normalizeWallpaperEffects({ rain: { enabled: true, intensity: 0.5, speed: 1 } });
describe("image effect configuration and simulation", () => {
  it("loads legacy preferences, clamps invalid settings and preserves independent parameters", () => {
    expect(normalizeWallpaperEffects(undefined)).toEqual(wallpaperEffectsDefaults);
    const options = normalizeWallpaperEffects({
      rain: { enabled: true, intensity: Infinity, speed: -2 },
      snow: { enabled: true, intensity: 3, speed: NaN },
      fireflies: { intensity: -1, speed: 10 },
    });
    expect(options.rain).toEqual({ enabled: true, intensity: 0.5, speed: 0.25 });
    expect(options.snow).toEqual({ enabled: false, intensity: 1, speed: 1 });
    expect(options.fireflies).toEqual({ enabled: false, intensity: 0, speed: 3 });
  });
  it("does not mutate applied preferences when a preview draft changes", () => {
    const applied = rain(), draft = normalizeWallpaperEffects(applied);
    draft.rain.speed = 2;
    draft.rain.enabled = false;
    expect(applied.rain.speed).toBe(1);
    expect(applied.rain.enabled).toBe(true);
    expect(wallpaperEffectsDefaults.rain.enabled).toBe(false);
  });
  it("makes intensity change particle density and speed change motion", () => {
    const slow = new EffectParticles(() => 0.1), fast = new EffectParticles(() => 0.1);
    const low = rain(), high = rain(); high.rain.intensity = 1; high.rain.speed = 2;
    slow.configure(low, 1920, 1080); fast.configure(high, 1920, 1080);
    expect(slow.particles.rain).toHaveLength(200);
    expect(fast.particles.rain).toHaveLength(400);
    slow.advance(0.05, low, 1920, 1080); fast.advance(0.05, high, 1920, 1080);
    expect(fast.particles.rain[0].y - 0.1).toBeCloseTo(2 * (slow.particles.rain[0].y - 0.1));
  });
  it("bounds particles on large monitors, resizes and removes disabled effects", () => {
    const particles = new EffectParticles(() => 0.9), options = rain(); options.rain.intensity = 1;
    particles.configure(options, 7680, 4320);
    expect(particles.particles.rain.length).toBe(600);
    particles.configure(options, 640, 360);
    expect(particles.particles.rain.length).toBeLessThan(100);
    particles.advance(10, options, 640, 360);
    expect(particles.particles.rain.every((p) => p.x >= 0 && p.x < 1 && p.y >= 0 && p.y < 1)).toBe(true);
    particles.configure(normalizeWallpaperEffects(), 640, 360);
    expect(particles.particles.rain).toHaveLength(0);
  });
  it("provides distinct snow and firefly motion", () => {
    for (const name of ["snow", "fireflies"] as const) {
      const particles = new EffectParticles(() => 0.2);
      const options = normalizeWallpaperEffects({ [name]: { enabled: true } });
      particles.configure(options, 960, 600);
      particles.advance(0.1, options, 960, 600);
      expect(particles.particles[name].length).toBeGreaterThan(0);
      expect(particles.particles[name][0].y).not.toBe(0.2);
      expect(particles.particles.rain).toHaveLength(0);
    }
  });
});

describe("trusted overlay lifecycle", () => {
  let host: HTMLElement, children: unknown[], doc: EventTarget & { hidden: boolean }, win: EventTarget;
  let callbacks: Map<number, FrameRequestCallback>, frames: number, draw: ReturnType<typeof vi.fn>;
  let disconnect: ReturnType<typeof vi.fn>, canvas: { width: number; height: number };
  beforeEach(() => {
    children = []; callbacks = new Map(); frames = 0; draw = vi.fn(); disconnect = vi.fn();
    host = { classList: { add: vi.fn(), remove: vi.fn() }, append: (node: unknown) => children.push(node),
      getBoundingClientRect: () => ({ width: 960, height: 600 }) } as unknown as HTMLElement;
    const context = { setTransform: vi.fn(), clearRect: draw, beginPath: vi.fn(), moveTo: vi.fn(),
      lineTo: vi.fn(), stroke: vi.fn(), arc: vi.fn(), fill: vi.fn(), save: vi.fn(), restore: vi.fn(), translate: vi.fn(), rotate: vi.fn(), ellipse: vi.fn(), createLinearGradient: () => ({addColorStop: vi.fn()}), createRadialGradient: () => ({addColorStop: vi.fn()}) };
    canvas = { width: 0, height: 0, setAttribute: vi.fn(), getContext: () => context,
      remove: () => { children.length = 0; } } as typeof canvas;
    doc = Object.assign(new EventTarget(), { hidden: false, createElement: () => canvas });
    win = Object.assign(new EventTarget(), { devicePixelRatio: 3 });
    vi.stubGlobal("document", doc); vi.stubGlobal("window", win);
    vi.stubGlobal("ResizeObserver", class { observe() {} disconnect = disconnect; constructor(_callback: unknown) {} });
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => { callbacks.set(++frames, callback); return frames; });
    vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
  });
  afterEach(() => vi.unstubAllGlobals());
  const frame = (time: number) => {
    const [id, callback] = callbacks.entries().next().value!;
    callbacks.delete(id); callback(time);
  };
  it("runs image previews at the requested FPS, freezes pause, and uses bounded DPR", () => {
    const effects = new WallpaperEffects(host, "image", rain(), { paused: false, fps: 30 });
    expect(children).toHaveLength(1); expect(canvas.width).toBe(1920);
    frame(0); const initial = draw.mock.calls.length;
    frame(16); expect(draw.mock.calls.length).toBe(initial);
    frame(34); expect(draw.mock.calls.length).toBe(initial + 1);
    effects.update("image", rain(), { paused: true, fps: 30 });
    expect(callbacks.size).toBe(0); expect(children).toHaveLength(1);
    effects.update("image", rain(), { paused: false, fps: 60 });
    expect(callbacks.size).toBe(1); effects.destroy(); expect(callbacks.size).toBe(0);
  });
  it("keeps the overlay on video and releases it on interactive canvas without losing parameters", () => {
    const options = rain(), effects = new WallpaperEffects(host, "image", options);
    effects.update("video", options); expect(children).toHaveLength(1); expect(callbacks.size).toBe(1);
    expect(effects.snapshot()).toMatchObject({ supported: true, canvasPresent: true, options });
    effects.update("canvas", options); expect(children).toHaveLength(0); expect(callbacks.size).toBe(0);
    expect(disconnect).toHaveBeenCalledOnce();
    effects.update("image", options); expect(children).toHaveLength(1); expect(callbacks.size).toBe(1);
    expect(effects.snapshot()).toMatchObject({ supported: true, canvasPresent: true, running: true, options });
    effects.update("image", normalizeWallpaperEffects()); expect(children).toHaveLength(0);
    expect(options.rain.enabled).toBe(true); effects.destroy();
  });
  it("suspends hidden views and pagehide removes canvas, observer and animation", () => {
    const effects = new WallpaperEffects(host, "image", rain());
    doc.hidden = true; doc.dispatchEvent(new Event("visibilitychange")); expect(callbacks.size).toBe(0);
    doc.hidden = false; doc.dispatchEvent(new Event("visibilitychange")); expect(callbacks.size).toBe(1);
    win.dispatchEvent(new Event("pagehide")); expect(children).toHaveLength(0); expect(callbacks.size).toBe(0);
    const draws = draw.mock.calls.length;
    win.dispatchEvent(new Event("resize")); expect(draw.mock.calls.length).toBe(draws);
    effects.update("image", rain()); expect(children).toHaveLength(0);
  });
});
