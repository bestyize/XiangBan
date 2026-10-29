import { describe, expect, it } from "vitest";
import { hexToRgb, normalizeChoice, normalizeParams, parsePack, paramsOf } from "./parse.ts";

const layer = (extra: Record<string, unknown> = {}) => ({ count: 10, shape: { kind: "circle" }, colors: ["#fff"], ...extra });
const pack = (extra: Record<string, unknown> = {}) => ({ schemaVersion: 1, id: "t", name: "测试", layers: [layer()], ...extra });
const ok = (raw: unknown) => { const parsed = parsePack(raw); if (!parsed.ok) throw new Error(parsed.error); return parsed.pack; };
const error = (raw: unknown) => { const parsed = parsePack(raw); if (parsed.ok) throw new Error("expected a rejection"); return parsed.error; };

describe("effect pack check", () => {
  it("fills in everything a short pack leaves out", () => {
    const parsed = ok(pack());
    const first = parsed.layers[0];
    expect(parsed.source).toBe("builtin");
    expect(first.size).toEqual([10, 20]);
    expect(first.direction).toEqual([90, 90]);
    expect(first.flip).toBeNull(); expect(first.twinkle).toBeNull(); expect(first.burst).toBeNull();
    expect(first.colors).toEqual([{ base: [255, 255, 255], tip: [255, 255, 255] }]);
    expect(parsed.sky).toHaveLength(2);
  });
  it("accepts a number or a [min, max] pair, puts them in order and keeps them inside the limits", () => {
    const first = ok(pack({ layers: [layer({ size: 30, speed: [900, 100], alpha: [-3, 9], count: 99999, wind: 5000 })] })).layers[0];
    expect(first.size).toEqual([30, 30]);
    expect(first.speed).toEqual([100, 900]);
    expect(first.alpha).toEqual([0, 1]);
    expect(first.count).toBe(1500);
    expect(first.wind).toBe(600);
    expect(ok(pack({ layers: [layer({ size: "big", speed: [1, "x"], alpha: [1] })] })).layers[0].size).toEqual([10, 20]);
  });
  it("reads colours as one value or a base and tip pair, and rejects anything that is not a hex colour", () => {
    expect(hexToRgb("#ff8000")).toEqual([255, 128, 0]);
    expect(hexToRgb("#f80")).toEqual([255, 136, 0]);
    for (const bad of ["red", "#ggg", "#12345", "rgb(1,2,3)", 7, null]) expect(hexToRgb(bad), String(bad)).toBeNull();
    expect(ok(pack({ layers: [layer({ colors: [["#000000", "#ffffff"], "#102030"] })] })).layers[0].colors[1]).toEqual({ base: [16, 32, 48], tip: [16, 32, 48] });
    expect(error(pack({ layers: [layer({ colors: ["url(javascript:1)"] })] }))).toContain("颜色");
    expect(error(pack({ layers: [layer({ colors: [] })] }))).toContain("颜色");
  });
  it("accepts only plain SVG path data for a shape", () => {
    const path = (d: unknown) => pack({ layers: [layer({ shape: { kind: "path", d } })] });
    expect(ok(path("M0,-1 L1,1 L-1,1 Z")).layers[0].shape).toMatchObject({ kind: "path", style: "fill" });
    for (const bad of ["", "M0,0 L1,1; alert(1)", "M0 0 <script>", 12, "M0,0 ".repeat(2000)]) expect(error(path(bad)), String(bad).slice(0, 12)).toContain("path");
    expect(error(pack({ layers: [layer({ shape: { kind: "evil" } })] }))).toContain("shape.kind");
    const vein = pack({ layers: [layer({ shape: { kind: "path", d: "M0,0 L1,1", vein: { d: "M0,0 L1,1 <", color: "#000" } } })] });
    expect(error(vein)).toContain("vein");
  });
  it("describes meteors as bursts and keeps a flip from collapsing to nothing", () => {
    const first = ok(pack({ layers: [layer({ burst: { duration: [1, 2], rest: [3, 4], x: [0.2, 1.5], y: [-0.1, 0.3], travel: 800 }, flip: { rate: 1, min: 0 } })] })).layers[0];
    expect(first.burst).toEqual({ duration: [1, 2], rest: [3, 4], x: [0.2, 1.5], y: [-0.1, 0.3], travel: [800, 800],
      group: 1, spread: 0, stagger: 0, launch: null, gravity: 0, ease: "linear", fade: "in-out", grow: [1, 1], fill: 0, flatten: [1, 1], flash: 0, mix: 0 });
    expect(first.flip).toEqual({ rate: [1, 1], min: 0.05 });
  });
  it("reads firework groups, bands and the new shapes, clamped, and keeps a band off burst layers", () => {
    const fireworks = ok(pack({ layers: [layer({ shape: { kind: "ring", width: 90, aspect: 3 }, blend: "add", orient: "motion",
      burst: { group: 999, spread: 720, stagger: -1, launch: 1, gravity: 50, ease: "out", fade: "out", grow: [0.1, 1], fill: 3, flatten: [0, 2], flash: 9, mix: -1 }, band: { width: 0.2 } })] })).layers[0];
    expect(fireworks.burst).toMatchObject({ group: 200, spread: 360, stagger: 0, launch: [1, 1], gravity: 50, ease: "out", fade: "out", grow: [0.1, 1], fill: 1, flatten: [0.05, 1], flash: 2, mix: 0 });
    expect(fireworks.band).toBeNull();
    expect(fireworks.shape).toEqual({ kind: "ring", width: [40, 40], aspect: 1 });
    expect(fireworks.blend).toBe("add"); expect(fireworks.orient).toBe("motion");
    const sky = ok(pack({ layers: [layer({ shape: { kind: "beam", length: 300, focus: 2 }, band: { angle: 120, offset: -0.3, width: 0.1, falloff: "even", wave: { amplitude: 1, count: 2 } } })] })).layers[0];
    expect(sky.shape).toEqual({ kind: "beam", length: [300, 300], focus: 0.9 });
    expect(sky.band).toEqual({ angle: 90, offset: -0.3, width: 0.1, falloff: "even", wave: { amplitude: 0.5, count: 2, speed: 0 } });
    expect(sky.blend).toBe("normal"); expect(sky.orient).toBe("fixed");
    const lantern = ok(pack({ layers: [layer({ shape: { kind: "path", d: "M0,0 L1,1 Z", halo: { scale: 9, color: "#ffaa00" } } })] })).layers[0];
    expect(lantern.shape).toMatchObject({ halo: { scale: 6, y: 0, color: [255, 170, 0], alpha: 0.4 } });
    expect(error(pack({ layers: [layer({ shape: { kind: "path", d: "M0,0 L1,1 Z", halo: { color: "red" } } })] }))).toContain("halo");
  });
  it("rejects packs that are not packs, with a reason the user can act on", () => {
    expect(error(null)).toContain("对象");
    expect(error(pack({ schemaVersion: 2 }))).toContain("schemaVersion");
    for (const id of ["", "Big", "a b", "../x", 3, "a".repeat(49)]) expect(error(pack({ id })), String(id)).toContain("id");
    expect(error(pack({ name: "  " }))).toContain("name");
    expect(error(pack({ layers: [] }))).toContain("layers");
    expect(error(pack({ layers: Array.from({ length: 7 }, () => layer()) }))).toContain("layers");
    expect(error(pack({ layers: Array.from({ length: 3 }, () => layer({ count: 1500 })) }))).toContain("3000");
    expect(error(pack({ layers: [5] }))).toContain("第 1 层");
  });
  it("keeps names short and the sky to valid colours", () => {
    const parsed = ok(pack({ name: "很长".repeat(20), description: "说".repeat(200), sky: ["#000", "oops", "#112233", "#445566", "#778899", "#aabbcc"] }));
    expect(parsed.name).toHaveLength(24); expect(parsed.description).toHaveLength(80);
    expect(parsed.sky).toEqual(["#000", "#112233", "#445566", "#778899"]);
    expect(ok(pack({ sky: ["#000"] })).sky).toHaveLength(2);
  });
});

describe("the user's choice", () => {
  it("clamps the three parameters and falls back to 1 for anything unreadable", () => {
    expect(normalizeParams({ density: 99, speed: 0, size: 0.1 })).toEqual({ density: 2.5, speed: 0.25, size: 0.5 });
    expect(normalizeParams({ density: "x", speed: NaN })).toEqual({ density: 1, speed: 1, size: 1 });
    expect(normalizeParams(null)).toEqual({ density: 1, speed: 1, size: 1 });
  });
  it("reads a saved choice and ignores what cannot be an effect id", () => {
    const choice = normalizeChoice({ selected: "maple", params: { maple: { density: 2 }, "Bad Id": { density: 2 } } });
    expect(choice.selected).toBe("maple");
    expect(Object.keys(choice.params)).toEqual(["maple"]);
    expect(paramsOf(choice, "maple").density).toBe(2);
    expect(paramsOf(choice, "other")).toEqual({ density: 1, speed: 1, size: 1 });
    expect(normalizeChoice({ selected: "No Good" }).selected).toBe("");
    expect(normalizeChoice(undefined)).toEqual({ selected: "", params: {} });
    expect(normalizeChoice({ selected: "rain", monitor: "screen-2" }).monitor).toBe("screen-2");
    expect(normalizeChoice({ selected: "rain", monitor: "*" }).monitor).toBe("*");
  });
});
