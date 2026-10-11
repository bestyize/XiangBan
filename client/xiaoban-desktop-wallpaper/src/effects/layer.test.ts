import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { drawEffects } from "./draw.ts";
import { EffectsLayer } from "./layer.ts";
import { parsePack } from "./parse.ts";
import { ParticleSystem } from "./sim.ts";
import type { EffectPack } from "./types.ts";

const make = (shape: Record<string, unknown>, extra: Record<string, unknown> = {}): EffectPack => {
  const parsed = parsePack({ schemaVersion: 1, id: "t", name: "t", layers: [{ count: 6, shape, colors: [["#ffffff", "#aabbcc"]], size: 20, speed: 50, alpha: 1, ...extra }] });
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.pack;
};
const params = { density: 1, speed: 1, size: 1 };

/** A canvas context that records which drawing calls were made. */
function recorder() {
  const calls: string[] = [];
  const gradient = { addColorStop: () => {} };
  const ctx = new Proxy({} as Record<string, unknown>, {
    get: (target, key: string) => key in target ? target[key] : (...args: unknown[]) => { calls.push(key); return args.length >= 0 && /^create/.test(key) ? gradient : undefined; },
    set: (target, key: string, value) => { target[key] = value; return true; },
  }) as unknown as CanvasRenderingContext2D;
  return { ctx, calls };
}
function drawn(pack: EffectPack, seconds = 0.2) {
  const sim = new ParticleSystem(pack, () => 0.5);
  sim.configure(1920, 1080, params);
  sim.advance(seconds);
  const { ctx, calls } = recorder();
  drawEffects(ctx, sim);
  return calls;
}

describe("drawing the four kinds of shape", () => {
  beforeEach(() => { vi.stubGlobal("Path2D", class { constructor(public d: string) {} }); });
  afterEach(() => vi.unstubAllGlobals());
  it("circle: one filled disc each", () => {
    const calls = drawn(make({ kind: "circle" }));
    expect(calls.filter((c) => c === "arc")).toHaveLength(6); expect(calls).not.toContain("createRadialGradient");
  });
  it("glow: a radial gradient disc each, plus a solid centre when it has a core", () => {
    expect(drawn(make({ kind: "glow" })).filter((c) => c === "createRadialGradient")).toHaveLength(6);
    const withCore = drawn(make({ kind: "glow", core: 0.3 }));
    expect(withCore.filter((c) => c === "arc")).toHaveLength(12);
  });
  it("streak: a line with a fading tail, and a halo only when it has a head", () => {
    const plain = drawn(make({ kind: "streak", length: 40, width: 2 }));
    expect(plain.filter((c) => c === "stroke")).toHaveLength(6); expect(plain).not.toContain("createRadialGradient");
    const headed = drawn(make({ kind: "streak", length: 40, width: 2, head: 3 }));
    expect(headed.filter((c) => c === "stroke")).toHaveLength(12); expect(headed.filter((c) => c === "createRadialGradient")).toHaveLength(6);
  });
  it("path: filled with a gradient, stroked on request, with an optional vein", () => {
    const filled = drawn(make({ kind: "path", d: "M0,-1 L1,1 L-1,1 Z" }));
    expect(filled.filter((c) => c === "fill")).toHaveLength(6); expect(filled).not.toContain("stroke");
    const stroked = drawn(make({ kind: "path", d: "M0,0 L1,1", style: "stroke" }));
    expect(stroked).not.toContain("fill"); expect(stroked.filter((c) => c === "stroke")).toHaveLength(6);
    const veined = drawn(make({ kind: "path", d: "M0,-1 L1,1 L-1,1 Z", vein: { d: "M0,-1 L0,1", color: "#000000" } }));
    expect(veined.filter((c) => c === "stroke")).toHaveLength(6);
    const lit = drawn(make({ kind: "path", d: "M0,-1 L1,1 L-1,1 Z", halo: { color: "#ffaa00" } }));
    expect(lit.filter((c) => c === "createRadialGradient")).toHaveLength(6); expect(lit.filter((c) => c === "fillRect")).toHaveLength(6);
  });
  it("ring: a soft wide outline and a sharp one; beam: one tall gradient; glow can be flattened", () => {
    const rings = drawn(make({ kind: "ring", aspect: 0.3 }));
    expect(rings.filter((c) => c === "stroke")).toHaveLength(12); expect(rings).not.toContain("fill");
    const beams = drawn(make({ kind: "beam", length: 200 }));
    expect(beams.filter((c) => c === "createRadialGradient")).toHaveLength(6); expect(beams.filter((c) => c === "fillRect")).toHaveLength(6);
    expect(drawn(make({ kind: "glow", aspect: 0.4 })).filter((c) => c === "scale")).toHaveLength(6);
  });
  it("adds light together only for layers that ask, and leaves the canvas in normal mode", () => {
    const sim = new ParticleSystem(make({ kind: "glow" }, { blend: "add" }), () => 0.5);
    sim.configure(1920, 1080, params); sim.advance(0.1);
    const modes: string[] = [];
    const { ctx } = recorder();
    Object.defineProperty(ctx, "globalCompositeOperation", { set: (mode: string) => modes.push(mode), get: () => modes.at(-1) });
    drawEffects(ctx, sim);
    expect(modes).toEqual(["lighter", "source-over"]);
  });
  it("clears the canvas first and draws nothing for particles that are resting", () => {
    const resting = make({ kind: "circle" }, { burst: { duration: 1, rest: 100, x: 0.5, y: 0.5, travel: 100 }, count: 3 });
    const sim = new ParticleSystem(resting, () => 0.99);
    sim.configure(1920, 1080, params);
    const { ctx, calls } = recorder();
    drawEffects(ctx, sim);
    expect(calls[0]).toBe("clearRect");
    expect(calls).not.toContain("arc");
  });
});

describe("the layer over the desktop", () => {
  let host: HTMLElement, children: unknown[], doc: EventTarget & { hidden: boolean }, win: EventTarget;
  let callbacks: Map<number, FrameRequestCallback>, frames: number, cleared: ReturnType<typeof vi.fn>, disconnect: ReturnType<typeof vi.fn>, canvas: { width: number; height: number };
  beforeEach(() => {
    children = []; callbacks = new Map(); frames = 0; cleared = vi.fn(); disconnect = vi.fn();
    host = { classList: { add: vi.fn(), remove: vi.fn() }, append: (node: unknown) => children.push(node), getBoundingClientRect: () => ({ width: 960, height: 600 }) } as unknown as HTMLElement;
    const gradient = { addColorStop: vi.fn() };
    const context = new Proxy({ clearRect: cleared, setTransform: vi.fn() } as Record<string, unknown>, {
      get: (target, key: string) => key in target ? target[key] : /^create/.test(key) ? () => gradient : vi.fn(), set: () => true });
    canvas = { width: 0, height: 0, setAttribute: vi.fn(), getContext: () => context, remove: () => { children.length = 0; } } as typeof canvas;
    doc = Object.assign(new EventTarget(), { hidden: false, createElement: () => canvas });
    win = Object.assign(new EventTarget(), { devicePixelRatio: 3 });
    vi.stubGlobal("document", doc); vi.stubGlobal("window", win);
    vi.stubGlobal("ResizeObserver", class { observe() {} disconnect = disconnect; constructor(_callback: unknown) {} });
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => { callbacks.set(++frames, callback); return frames; });
    vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
    vi.stubGlobal("Path2D", class {});
  });
  afterEach(() => vi.unstubAllGlobals());
  const frame = (time: number) => { const [id, callback] = callbacks.entries().next().value!; callbacks.delete(id); callback(time); };
  // The host is 960x600, 5/18 of a 1920x1080 screen, and the number of particles follows the area.
  const rain = make({ kind: "streak", length: 30, width: 2 }, { count: 36 });

  it("runs at the requested frame rate, stands still when paused, and caps the pixel ratio", () => {
    const layer = new EffectsLayer(host, rain, params, { paused: false, fps: 30 });
    expect(children).toHaveLength(1); expect(canvas.width).toBe(1920);
    frame(0); const first = cleared.mock.calls.length;
    frame(16); expect(cleared.mock.calls.length).toBe(first);
    frame(34); expect(cleared.mock.calls.length).toBe(first + 1);
    layer.update(rain, params, { paused: true, fps: 30 });
    expect(callbacks.size).toBe(0); expect(children).toHaveLength(1);
    layer.update(rain, params, { paused: false, fps: 60 });
    expect(callbacks.size).toBe(1); layer.destroy(); expect(callbacks.size).toBe(0);
  });
  it("adjusts a running effect in place and starts over only for a different one", () => {
    const layer = new EffectsLayer(host, rain, params, { paused: false, fps: 60 }, () => 0.4);
    expect(layer.snapshot()).toMatchObject({ effect: "t", particles: [10], canvasPresent: true });
    layer.update(rain, { density: 2, speed: 1, size: 1 });
    expect(layer.snapshot().particles).toEqual([20]);
    layer.update(rain, { density: 0.2, speed: 1, size: 1 });
    expect(layer.snapshot().particles).toEqual([2]);
    expect(children).toHaveLength(1); expect(disconnect).not.toHaveBeenCalled();
    const snow = { ...make({ kind: "circle" }, { count: 36 }), id: "snow" };
    layer.update(snow, params);
    expect(layer.snapshot()).toMatchObject({ effect: "snow", particles: [10] });
  });
  it("removes its canvas and listeners when nothing is chosen, and comes back when something is", () => {
    const layer = new EffectsLayer(host, rain, params);
    layer.update(null, params);
    expect(children).toHaveLength(0); expect(callbacks.size).toBe(0); expect(disconnect).toHaveBeenCalledOnce();
    expect(layer.snapshot()).toMatchObject({ canvasPresent: false, effect: "", particles: [] });
    layer.update(rain, params);
    expect(children).toHaveLength(1); expect(callbacks.size).toBe(1);
    layer.destroy();
  });
  it("suspends while hidden, and the page leaving removes canvas, observer and animation", () => {
    const layer = new EffectsLayer(host, rain, params);
    doc.hidden = true; doc.dispatchEvent(new Event("visibilitychange")); expect(callbacks.size).toBe(0);
    doc.hidden = false; doc.dispatchEvent(new Event("visibilitychange")); expect(callbacks.size).toBe(1);
    win.dispatchEvent(new Event("pagehide")); expect(children).toHaveLength(0); expect(callbacks.size).toBe(0);
    const draws = cleared.mock.calls.length;
    win.dispatchEvent(new Event("resize")); expect(cleared.mock.calls.length).toBe(draws);
    layer.update(rain, params); expect(children).toHaveLength(0);
  });
});
