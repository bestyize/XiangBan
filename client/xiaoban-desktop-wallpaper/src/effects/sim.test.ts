import { describe, expect, it } from "vitest";
import { parsePack } from "./parse.ts";
import { ParticleSystem } from "./sim.ts";
import type { EffectParams } from "./types.ts";

function seeded(seed = 7) {
  return () => { seed = (seed + 0x6d2b79f5) | 0; let t = Math.imul(seed ^ (seed >>> 15), 1 | seed); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
}
const base = { count: 100, shape: { kind: "circle" }, colors: ["#fff"], size: [10, 30], speed: [60, 120], alpha: [0.5, 1] };
function system(layer: Record<string, unknown> = {}, params: Partial<EffectParams> = {}, w = 1920, h = 1080) {
  const parsed = parsePack({ schemaVersion: 1, id: "t", name: "t", layers: [{ ...base, ...layer }] });
  if (!parsed.ok) throw new Error(parsed.error);
  const sim = new ParticleSystem(parsed.pack, seeded());
  sim.configure(w, h, { density: 1, speed: 1, size: 1, ...params });
  return sim;
}
const first = (sim: ParticleSystem) => sim.layers[0];

describe("how many particles", () => {
  it("scales with density and the size of the screen, and never drops a layer to nothing", () => {
    expect(first(system()).active).toBe(100);
    expect(first(system({}, { density: 2 })).active).toBe(200);
    expect(first(system({}, { density: 0.2 })).active).toBe(20);
    expect(first(system({}, {}, 960, 540)).active).toBe(25);
    expect(first(system({}, {}, 7680, 4320)).active).toBe(160);
    expect(first(system({ count: 1 }, { density: 0.2 }, 320, 180)).active).toBe(1);
    expect(first(system({ count: 0 })).active).toBe(0);
  });
  it("changes the number in place: the particles that stay do not move", () => {
    const sim = system();
    const before = first(sim).pool.slice(0, 60).map((p) => [p.u, p.v, p.d]);
    sim.configure(1920, 1080, { density: 0.6, speed: 1, size: 1 });
    expect(first(sim).active).toBe(60);
    expect(first(sim).pool.slice(0, 60).map((p) => [p.u, p.v, p.d])).toEqual(before);
    sim.configure(1920, 1080, { density: 1.5, speed: 1, size: 1 });
    expect(first(sim).active).toBe(150);
    expect(first(sim).pool.slice(0, 60).map((p) => [p.u, p.v, p.d])).toEqual(before);
  });
  it("draws far particles first so near ones cover them", () => {
    const depths = first(system()).order.map((p) => p.d);
    expect(depths).toEqual([...depths].sort((a, b) => a - b));
  });
});

describe("how they move", () => {
  it("moves along the direction at the layer's speed, nearer ones faster, and wraps around the screen", () => {
    const sim = system({ direction: 90, speed: [100, 300], count: 40 });
    const state = first(sim);
    const start = state.pool.map((p) => ({ y: p.y }));
    sim.advance(0.1);
    const moved = state.pool.map((p, i) => p.y - start[i].y);
    state.pool.forEach((p, i) => { if (moved[i] > 0) expect(moved[i]).toBeCloseTo((100 + 200 * p.d) * 0.1, 1); });
    expect(moved.filter((m) => m > 0).length).toBeGreaterThan(30);
    for (let i = 0; i < 300; i++) sim.advance(0.1);
    expect(state.pool.every((p) => p.u >= 0 && p.u < 1 && p.v >= 0 && p.v < 1 && p.y >= -state.pad && p.y <= 1080 + state.pad)).toBe(true);
  });
  it("runs faster or slower with the speed setting", () => {
    const slow = system({ speed: [100, 100] }, { speed: 1 }), fast = system({ speed: [100, 100] }, { speed: 3 });
    slow.advance(0.05); fast.advance(0.05);
    const travelled = (sim: ParticleSystem) => { const p = first(sim).pool[0]; return p.v; };
    const start = first(system({ speed: [100, 100] })).pool[0].v;
    const unwrap = (v: number) => (v - start + 1) % 1;
    expect(unwrap(travelled(fast)) / unwrap(travelled(slow))).toBeCloseTo(3, 1);
  });
  it("is pushed by the shared gusts only as much as the layer asks, and never beyond them", () => {
    const sim = system({ wind: 0, speed: 0, count: 5 });
    const calm = first(sim).pool.map((p) => p.x);
    for (let i = 0; i < 100; i++) sim.advance(0.05);
    expect(first(sim).pool.map((p) => p.x)).toEqual(calm);
    let strongest = 0;
    for (let i = 0; i < 2000; i++) { sim.advance(0.1); strongest = Math.max(strongest, Math.abs(sim.wind())); }
    expect(strongest).toBeLessThanOrEqual(1);
    expect(strongest).toBeGreaterThan(0.3);
    const gusty = system({ wind: 200, speed: 0, count: 5 }), still = first(gusty).pool.map((p) => p.x);
    for (let i = 0; i < 100; i++) gusty.advance(0.05);
    expect(first(gusty).pool.map((p) => p.x)).not.toEqual(still);
  });
  it("sways from side to side and tilts with the sway like a swinging leaf", () => {
    const sim = system({ speed: 0, sway: { amplitude: 60, frequency: 0.5 }, pendulum: 1, scatter: 0, count: 3 });
    const xs: number[] = [], angles: number[] = [];
    for (let i = 0; i < 80; i++) { sim.advance(0.05); xs.push(first(sim).pool[0].x); angles.push(first(sim).pool[0].angle); }
    expect(Math.max(...xs) - Math.min(...xs)).toBeGreaterThan(5);
    expect(Math.max(...angles)).toBeGreaterThan(0.2); expect(Math.min(...angles)).toBeLessThan(-0.2);
  });
  it("turns over and twinkles within the limits the pack gives", () => {
    const sim = system({ flip: { rate: 1.5, min: 0.3 }, twinkle: { rate: 1, depth: 0.8 }, alpha: [0.4, 0.4], count: 30 });
    const widths: number[] = [], alphas: number[] = [];
    for (let i = 0; i < 100; i++) { sim.advance(0.05); for (const p of first(sim).pool) { widths.push(p.scaleX); alphas.push(p.alpha); } }
    expect(Math.min(...widths)).toBeGreaterThanOrEqual(0.3 - 1e-9); expect(Math.max(...widths)).toBeLessThanOrEqual(1 + 1e-9);
    expect(Math.min(...widths)).toBeLessThan(0.5);
    expect(Math.max(...alphas)).toBeLessThanOrEqual(0.4 + 1e-9); expect(Math.min(...alphas)).toBeLessThan(0.12);
  });
  it("makes particles bigger with the size setting and with depth", () => {
    const small = system({}, { size: 1 }), big = system({}, { size: 2 });
    small.advance(0); big.advance(0);
    const sized = (sim: ParticleSystem) => first(sim).pool.map((p) => p.size);
    sized(small).forEach((s, i) => expect(sized(big)[i]).toBeCloseTo(s * 2, 6));
    expect(Math.min(...sized(small))).toBeGreaterThanOrEqual(10 - 1e-9); expect(Math.max(...sized(small))).toBeLessThanOrEqual(30 + 1e-9);
    expect(system({}, {}, 1920, 2160).unit).toBe(1.8);
  });
});

describe("meteors", () => {
  const meteors = () => system({ count: 6, burst: { duration: [1, 1], rest: [3, 3], x: [0.5, 1], y: [0, 0.3], travel: [400, 400] }, direction: 150, shape: { kind: "streak", length: 100, width: 2 } });
  it("cross the screen for a moment and then rest, fading in and out", () => {
    const sim = meteors();
    const state = first(sim);
    const visible: number[] = [];
    for (let i = 0; i < 400; i++) { sim.advance(0.05); visible.push(state.pool.filter((p) => p.visible).length); }
    expect(Math.max(...visible)).toBeGreaterThan(0);
    expect(Math.min(...visible)).toBe(0);
    const seen = state.pool.find((p) => { for (let i = 0; i < 400 && !p.visible; i++) sim.advance(0.05); return p.visible; })!;
    expect(seen.dx).toBeCloseTo(Math.cos(150 * Math.PI / 180), 6); expect(seen.dy).toBeGreaterThan(0); expect(seen.alpha).toBeLessThanOrEqual(1);
  });
  it("start somewhere new every turn and not all at the same time", () => {
    const sim = meteors(), state = first(sim);
    const starts = new Set(state.pool.map((p) => p.sx.toFixed(3)));
    expect(starts.size).toBeGreaterThan(3);
    expect(new Set(state.pool.map((p) => p.age.toFixed(2))).size).toBeGreaterThan(3);
    const before = state.pool.map((p) => p.sx);
    for (let i = 0; i < 400; i++) sim.advance(0.05);
    expect(state.pool.map((p) => p.sx)).not.toEqual(before);
  });
});

describe("a stopped or empty screen", () => {
  it("does nothing until it has a size, and a long pause cannot throw particles across the screen", () => {
    const parsed = parsePack({ schemaVersion: 1, id: "t", name: "t", layers: [base] });
    if (!parsed.ok) throw new Error(parsed.error);
    const idle = new ParticleSystem(parsed.pack, seeded());
    idle.advance(1);
    expect(idle.counts()).toEqual([0]);
    const sim = system({ speed: [100, 100] }), before = first(sim).pool[0].v;
    sim.advance(60);
    expect((first(sim).pool[0].v - before + 1) % 1).toBeLessThan(0.12);
  });
});
