import { defaultParams, limits, type Burst, type ColorPair, type EffectPack, type EffectParams, type EffectsChoice, type Layer, type Range, type Rgb, type Shape, type Vein } from "./types.ts";

type Obj = Record<string, unknown>;
const isObj = (value: unknown): value is Obj => typeof value === "object" && value !== null && !Array.isArray(value);
const clamp = (value: number, [min, max]: Range) => Math.max(min, Math.min(max, value));
const num = (value: unknown, fallback: number, bounds: Range) => typeof value === "number" && Number.isFinite(value) ? clamp(value, bounds) : fallback;
/** A number, or `[min, max]`; anything else gives the fallback. The result is always ordered and inside `bounds`. */
function range(value: unknown, fallback: Range, bounds: Range): Range {
  const pair = typeof value === "number" ? [value, value] : Array.isArray(value) && value.length === 2 ? value : null;
  if (!pair || !pair.every((n) => typeof n === "number" && Number.isFinite(n))) return fallback;
  const [a, b] = [clamp(pair[0], bounds), clamp(pair[1], bounds)];
  return a <= b ? [a, b] : [b, a];
}
export function hexToRgb(value: unknown): Rgb | null {
  if (typeof value !== "string") return null;
  const match = /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.exec(value);
  if (!match) return null;
  const hex = match[1].length === 3 ? [...match[1]].map((c) => c + c).join("") : match[1];
  return [0, 2, 4].map((i) => parseInt(hex.slice(i, i + 2), 16)) as Rgb;
}
const pathData = (value: unknown) => typeof value === "string" && value.length > 0 && value.length <= limits.pathLength && /^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s+-]+$/.test(value) ? value : null;

function colors(value: unknown): ColorPair[] | string {
  if (!Array.isArray(value) || value.length === 0 || value.length > 24) return "colors 需要 1 到 24 个颜色";
  const out: ColorPair[] = [];
  for (const item of value) {
    const base = hexToRgb(Array.isArray(item) ? item[0] : item), tip = hexToRgb(Array.isArray(item) ? (item[1] ?? item[0]) : item);
    if (!base || !tip) return "颜色只能写成 #rgb 或 #rrggbb";
    out.push({ base, tip });
  }
  return out;
}

function shape(value: unknown): Shape | string {
  const raw = isObj(value) ? value : {};
  switch (raw.kind) {
    case "circle": return { kind: "circle" };
    case "glow": return { kind: "glow", core: num(raw.core, 0, [0, 0.9]) };
    case "streak": return { kind: "streak", length: range(raw.length, [20, 40], [2, 800]), width: range(raw.width, [1, 2], [0.3, 40]), head: num(raw.head, 0, [0, 8]) };
    case "path": {
      const d = pathData(raw.d);
      if (!d) return "path 的 d 必须是不超过 6000 个字符的 SVG 路径数据";
      let vein: Vein | null = null;
      if (isObj(raw.vein)) {
        const veinD = pathData(raw.vein.d), color = hexToRgb(raw.vein.color ?? "#000000");
        if (!veinD || !color) return "vein 的路径或颜色无效";
        vein = { d: veinD, color, alpha: num(raw.vein.alpha, 0.5, [0, 1]), width: num(raw.vein.width, 0.06, [0.005, 0.5]) };
      }
      const style = raw.style === "stroke" || raw.style === "both" ? raw.style : "fill";
      return { kind: "path", d, style, lineWidth: num(raw.lineWidth, 0.08, [0.01, 0.5]), vein };
    }
    default: return "shape.kind 只能是 circle、glow、path 或 streak";
  }
}

function layer(value: unknown, index: number): Layer | string {
  if (!isObj(value)) return `第 ${index + 1} 层不是对象`;
  const form = shape(value.shape), palette = colors(value.colors);
  if (typeof form === "string") return `第 ${index + 1} 层：${form}`;
  if (typeof palette === "string") return `第 ${index + 1} 层：${palette}`;
  const pair = (key: string, fallback: Range, bounds: Range): { amplitude: Range; frequency: Range } => {
    const raw = isObj(value[key]) ? (value[key] as Obj) : {};
    return { amplitude: range(raw.amplitude, [0, 0], bounds), frequency: range(raw.frequency, fallback, [0, 5]) };
  };
  let burst: Burst | null = null;
  if (isObj(value.burst)) {
    const raw = value.burst;
    burst = { duration: range(raw.duration, [1, 1.5], [0.2, 20]), rest: range(raw.rest, [2, 6], [0, 60]),
      x: range(raw.x, [0, 1], [-1, 2]), y: range(raw.y, [0, 1], [-1, 2]), travel: range(raw.travel, [300, 600], [10, 6000]) };
  }
  const flip = isObj(value.flip) ? { rate: range(value.flip.rate, [1, 2], [0, 20]), min: num(value.flip.min, 0.2, [0.05, 1]) } : null;
  const twinkle = isObj(value.twinkle) ? { rate: range(value.twinkle.rate, [0.3, 0.8], [0, 10]), depth: num(value.twinkle.depth, 0.6, [0, 1]) } : null;
  return {
    count: Math.round(num(value.count, 0, [0, limits.countPerLayer])), shape: form,
    size: range(value.size, [10, 20], [0.5, 400]), speed: range(value.speed, [0, 0], [0, 4000]),
    direction: range(value.direction, [90, 90], [-360, 720]), wind: num(value.wind, 0, [-600, 600]),
    sway: pair("sway", [0.15, 0.35], [0, 600]), wander: pair("wander", [0.05, 0.15], [0, 600]),
    spin: range(value.spin, [0, 0], [-20, 20]), pendulum: num(value.pendulum, 0, [0, 3]), scatter: num(value.scatter, 1, [0, 1]),
    flip, alpha: range(value.alpha, [0.5, 1], [0, 1]), colors: palette, twinkle, burst,
  };
}

export type ParsedPack = { ok: true; pack: EffectPack } | { ok: false; error: string };
/** Checks a pack read from a file and returns a safe copy: every number is clamped, every path and colour verified. */
export function parsePack(raw: unknown): ParsedPack {
  if (!isObj(raw)) return { ok: false, error: "特效包不是一个对象" };
  if (raw.schemaVersion !== 1) return { ok: false, error: "不支持这个 schemaVersion（需要 1）" };
  if (typeof raw.id !== "string" || !limits.idPattern.test(raw.id)) return { ok: false, error: "id 只能用小写字母、数字、- 和 _，且不超过 48 个字符" };
  if (typeof raw.name !== "string" || !raw.name.trim()) return { ok: false, error: "缺少 name" };
  if (!Array.isArray(raw.layers) || raw.layers.length === 0 || raw.layers.length > limits.layers) return { ok: false, error: `layers 需要 1 到 ${limits.layers} 层` };
  const layers: Layer[] = [];
  for (const [index, item] of raw.layers.entries()) {
    const parsed = layer(item, index);
    if (typeof parsed === "string") return { ok: false, error: parsed };
    layers.push(parsed);
  }
  if (layers.reduce((sum, item) => sum + item.count, 0) > limits.totalCount) return { ok: false, error: `所有层的 count 加起来不能超过 ${limits.totalCount}` };
  const sky = Array.isArray(raw.sky) ? raw.sky.filter((c): c is string => hexToRgb(c) !== null).slice(0, 4) : [];
  return { ok: true, pack: { id: raw.id, name: raw.name.trim().slice(0, 24), description: typeof raw.description === "string" ? raw.description.slice(0, 80) : "",
    sky: sky.length >= 2 ? sky : ["#10182f", "#2a3358"], layers, source: raw.source === "user" ? "user" : "builtin" } };
}

export function normalizeParams(value: unknown): EffectParams {
  const raw = isObj(value) ? value : {};
  return { density: num(raw.density, defaultParams.density, limits.density), speed: num(raw.speed, defaultParams.speed, limits.speed), size: num(raw.size, defaultParams.size, limits.size) };
}
export function normalizeChoice(value: unknown): EffectsChoice {
  const raw = isObj(value) ? value : {};
  const params: EffectsChoice["params"] = {};
  if (isObj(raw.params)) for (const [id, item] of Object.entries(raw.params).slice(0, 64)) if (limits.idPattern.test(id)) params[id] = normalizeParams(item);
  return { selected: typeof raw.selected === "string" && limits.idPattern.test(raw.selected) ? raw.selected : "", params };
}
export const paramsOf = (choice: EffectsChoice, id: string): EffectParams => choice.params[id] ?? { ...defaultParams };
