import { describe, expect, it } from "vitest";
import { loadBuiltinPacks, readList } from "./catalog.ts";
import { parsePack } from "./parse.ts";
import type { EffectPack } from "./types.ts";

// The files themselves, read by the bundler: the shipped effects and the example for people who write their own.
const files = import.meta.glob<unknown>(["../../public/effects/*.json", "../../docs/effect-pack-example.json"], { eager: true, import: "default" });
const read = (name: string): any => files[`../../public/effects/${name}`];
const catalog: string[] = read("catalog.json").effects;
const packs = new Map<string, EffectPack>();
for (const id of catalog) { const parsed = parsePack(read(`${id}.json`)); if (parsed.ok) packs.set(id, parsed.pack); }
const total = (pack: EffectPack) => pack.layers.reduce((sum, layer) => sum + layer.count, 0);
const biggest = (pack: EffectPack) => Math.max(...pack.layers.map((layer) => layer.size[1]));

describe("the effects shipped with the app", () => {
  it("are exactly the files in the folder, each one valid, and their ids are their file names", () => {
    for (const id of catalog) expect([...packs.get(id)!.name], id).toHaveLength(4);
    const onDisk = Object.keys(files).filter((path) => path.includes("/public/effects/") && !path.endsWith("/catalog.json")).map((path) => path.replace(/^.*\//, "").slice(0, -5)).sort();
    expect([...catalog].sort()).toEqual(onDisk);
    expect(catalog).toEqual(["lanterns", "maple", "galaxy", "butterfly-dream", "fireworks", "snow", "rain", "fireflies", "petals", "meteors", "stars", "aurora"]);
    for (const id of catalog) {
      const parsed = parsePack(read(`${id}.json`));
      expect(parsed.ok ? "" : parsed.error, id).toBe("");
      expect(packs.get(id)?.id).toBe(id);
    }
  });
  it("show leaves as maple leaves and big enough to be seen", () => {
    expect(packs.has("leaves" as string)).toBe(false);
    const maple = packs.get("maple")!;
    expect(maple.name).toBe("层林尽染");
    expect(maple.layers.every((layer) => layer.shape.kind === "path")).toBe(true);
    expect(biggest(maple)).toBeGreaterThanOrEqual(60);
    expect(Math.min(...maple.layers.map((layer) => layer.size[0]))).toBeGreaterThanOrEqual(18);
    expect(total(maple)).toBeGreaterThanOrEqual(100);
    expect(maple.layers[0].colors.length).toBeGreaterThanOrEqual(5);
  });
  it("have enough particles and big enough shapes to read on a 1080p screen", () => {
    for (const id of ["rain", "snow", "fireflies", "petals", "maple", "stars"]) expect(total(packs.get(id)!), id).toBeGreaterThanOrEqual(100);
    expect(total(packs.get("rain")!)).toBeGreaterThanOrEqual(500);
    expect(biggest(packs.get("petals")!)).toBeGreaterThanOrEqual(30);
    expect(Math.max(...packs.get("snow")!.layers.map((layer) => layer.size[1]))).toBeGreaterThanOrEqual(25);
    expect(biggest(packs.get("fireflies")!)).toBeGreaterThanOrEqual(28);
  });
  it("include a starry sky with stars of several kinds, twinkling", () => {
    const stars = packs.get("stars")!;
    expect(stars.name).toBe("星罗棋布");
    expect(stars.layers.length).toBeGreaterThanOrEqual(3);
    expect(new Set(stars.layers.map((layer) => layer.shape.kind)).size).toBeGreaterThanOrEqual(3);
    expect(stars.layers.every((layer) => layer.twinkle !== null)).toBe(true);
    expect(stars.layers.some((layer) => layer.shape.kind === "path")).toBe(true);
  });
  it("make meteors that cross the screen in turns, not a constant fall", () => {
    const meteors = packs.get("meteors")!;
    expect(meteors.layers.every((layer) => layer.burst !== null && layer.shape.kind === "streak")).toBe(true);
    expect(Math.min(...meteors.layers.map((layer) => layer.burst!.rest[0]))).toBeGreaterThan(1);
  });
  it("make lanterns rise upright with a warm glow, and butterflies flap their wings", () => {
    const lanterns = packs.get("lanterns")!.layers.filter((layer) => layer.shape.kind === "path");
    expect(lanterns.length).toBeGreaterThanOrEqual(2);
    for (const layer of lanterns) {
      expect(layer.scatter).toBe(0); expect(layer.direction[0]).toBeGreaterThan(240); expect(layer.direction[1]).toBeLessThan(300);
      expect(layer.shape.kind === "path" && layer.shape.halo).toBeTruthy();
    }
    expect(packs.get("butterfly-dream")!.layers.filter((layer) => layer.flip !== null && layer.flip.rate[0] >= 1.5).length).toBeGreaterThanOrEqual(2);
  });
  it("launch fireworks that burst into whole groups of sparks", () => {
    const fireworks = packs.get("fireworks")!;
    for (const layer of fireworks.layers) {
      expect(layer.burst!.group).toBeGreaterThanOrEqual(20); expect(layer.burst!.spread).toBe(360); expect(layer.burst!.launch).not.toBeNull();
      expect(layer.count % layer.burst!.group).toBe(0); expect(layer.blend).toBe("add"); expect(layer.burst!.flash).toBeGreaterThan(0);
    }
    expect(fireworks.layers.filter((layer) => layer.burst!.fill >= 0.8).length).toBeGreaterThanOrEqual(3);
    expect(fireworks.layers.some((layer) => layer.burst!.gravity >= 90 && layer.burst!.duration[0] >= 3)).toBe(true);
    expect(fireworks.layers.some((layer) => layer.burst!.flatten[1] < 1)).toBe(true);
  });
  it("keep the aurora and the Milky Way in bands across the sky", () => {
    const curtains = packs.get("aurora")!.layers.filter((layer) => layer.shape.kind === "beam");
    expect(curtains.length).toBeGreaterThanOrEqual(2);
    expect(curtains.every((layer) => layer.band !== null && layer.band.offset < 0 && layer.band.wave.amplitude > 0 && layer.blend === "add")).toBe(true);
    const galaxy = packs.get("galaxy")!;
    expect(galaxy.layers.filter((layer) => layer.band !== null && layer.band.angle !== 0).reduce((sum, layer) => sum + layer.count, 0)).toBeGreaterThanOrEqual(800);
  });
  it("come with an example for people who write their own, and it is valid", () => {
    const example = files["../../docs/effect-pack-example.json"] as { id: string };
    const parsed = parsePack({ ...example, source: "user" });
    expect(parsed.ok && parsed.pack.source).toBe("user");
    expect(catalog).not.toContain(example.id);
  });
});

describe("loading packs for the control window", () => {
  it("keeps the packs that pass the check and says which others did not and why", () => {
    const list = readList({ packs: [read("rain.json"), { schemaVersion: 1, id: "bad", name: "坏的", layers: [] }, 5], errors: ["a.json：JSON 格式错误"], directory: "/x/effects" });
    expect(list.packs.map((pack) => pack.id)).toEqual(["rain"]);
    expect(list.errors).toEqual(["a.json：JSON 格式错误", "坏的：layers 需要 1 到 6 层", "未命名特效：特效包不是一个对象"]);
    expect(list.directory).toBe("/x/effects");
    expect(readList(undefined)).toEqual({ packs: [], errors: [], directory: "" });
  });
  it("reads the built-in packs over the web when there is no desktop host", async () => {
    const served = new Map<string, unknown>([["/effects/catalog.json", read("catalog.json")], ...catalog.map((id) => [`/effects/${id}.json`, read(`${id}.json`)] as [string, unknown])]);
    const fetcher = (async (url: string) => served.has(url) ? { ok: true, json: async () => served.get(url) } : { ok: false, json: async () => null }) as unknown as typeof fetch;
    const loaded = await loadBuiltinPacks(fetcher);
    expect(loaded.packs.map((pack) => pack.id)).toEqual(catalog);
    expect(loaded.errors).toEqual([]);
    expect(loaded.packs.every((pack) => pack.source === "builtin")).toBe(true);
  });
});
