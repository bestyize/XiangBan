import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
interface Rules {
  SEA: number; DAY_START: number; FEVER_CHAIN: number; CLOUD: number; START_X: number
  createGame(options?: { seed?: number }): Any
  step(state: Any, input?: { hold?: boolean }, dt?: number): Any[]
  result(state: Any): Any
  sample(state: Any, x: number): { h: number; d: number; dd: number; water: boolean }
  terrain(state: Any, x: number): { h: number; d: number; dd: number }
  islandAt(state: Any, x: number): Any
  ensure(state: Any, x: number): void
  PERFECT: number; PERFECT_EASY: number
  perfectWindow(islandIndex: number): number
  predictLanding(state: Any, hold: boolean, seconds?: number): Any
  advice(state: Any): Any
  nextIsland(state: Any): Any
}
const context: { XiabanDuskGlider?: Rules } = {}
// Exercise the exact classic script shipped in the sandboxed game pack.
runInNewContext(readFileSync(new URL('../../../public/game-packs/dusk-glider/rules.js', import.meta.url), 'utf8'), context)
const rules = context.XiabanDuskGlider!
const DT = 1 / 60

function run(state: Any, frames: number, hold: boolean | ((s: Any) => boolean) = false) {
  const events: Any[] = []
  for (let i = 0; i < frames && state.phase === 'playing'; i++) events.push(...rules.step(state, { hold: typeof hold === 'function' ? hold(state) : hold }, DT))
  return events
}

// Finds an x on the first island where the slope matches the predicate.
function findSlope(state: Any, predicate: (d: number, h: number) => boolean, from = 60) {
  for (let x = from; x < 600; x += 0.5) {
    const s = rules.sample(state, x)
    if (!s.water && predicate(s.d, s.h)) return x
  }
  throw new Error('no matching slope')
}

// Puts the bird in the air just above the ground at x, flying along `angle` at `speed`.
function airborne(state: Any, x: number, angle: number, speed: number, air = 1) {
  const b = state.bird
  Object.assign(b, { x, y: rules.sample(state, x).h + 0.02, vx: Math.cos(angle) * speed, vy: Math.sin(angle) * speed, ground: false, air, apex: 3, cloud: true })
}

// Steps until the airborne bird touches down and returns that step's events.
function touchdown(state: Any) {
  for (let i = 0; i < 30; i++) {
    const events = rules.step(state, { hold: false }, DT)
    if (state.bird.ground || events.some((e) => e.type === 'skip')) return events
  }
  throw new Error('bird never landed')
}

describe('dusk glider terrain', () => {
  it('builds the same islands for the same seed with smooth, consistent slopes', () => {
    const a = rules.createGame({ seed: 7 }), b = rules.createGame({ seed: 7 }), c = rules.createGame({ seed: 8 })
    rules.ensure(a, 3000); rules.ensure(b, 3000); rules.ensure(c, 3000)
    expect(a.world.points).toEqual(b.world.points)
    expect(a.world.points).not.toEqual(c.world.points)
    for (let x = -50; x < 3000; x += 37.3) {
      const s = rules.terrain(a, x)
      const numeric = (rules.terrain(a, x + 0.01).h - rules.terrain(a, x - 0.01).h) / 0.02
      expect(s.d).toBeCloseTo(numeric, 2)
    }
    // Sampling extends the world, so check a fixed copy of the islands generated so far.
    const islands = [...a.world.islands]
    expect(islands.length).toBeGreaterThan(3)
    for (const isle of islands) {
      expect(rules.sample(a, (isle.x1 + isle.end) / 2).water).toBe(true)
      expect(rules.sample(a, isle.firstPeak).water).toBe(false)
      expect(rules.islandAt(a, isle.firstPeak).index).toBe(isle.index)
    }
  })
})

describe('dusk glider physics', () => {
  it('drops the bird onto the first slope and lets it slide', () => {
    const state = rules.createGame({ seed: 1 })
    expect(state.bird.ground).toBe(false)
    expect(state.daylight).toBe(rules.DAY_START)
    const events = run(state, 240)
    expect(events.some((e) => ['perfect', 'land', 'thud'].includes(e.type))).toBe(true)
    expect(state.bird.x).toBeGreaterThan(rules.START_X + 5)
  })

  it('gains more speed downhill with the finger down, and launches off a crest when released', () => {
    const base = rules.createGame({ seed: 2 })
    const x = findSlope(base, (d) => d < -0.45)
    const heavy = rules.createGame({ seed: 2 }), light = rules.createGame({ seed: 2 })
    for (const s of [heavy, light]) Object.assign(s.bird, { x, y: rules.sample(s, x).h, ground: true, speed: 12 })
    run(heavy, 30, true); run(light, 30, false)
    expect(heavy.bird.speed).toBeGreaterThan(light.bird.speed + 3)

    // Released at speed on the way up, the bird leaves the ground over the next crest.
    const up = findSlope(base, (d) => d > 0.4)
    const flier = rules.createGame({ seed: 2 })
    Object.assign(flier.bird, { x: up, y: rules.sample(flier, up).h, ground: true, speed: 34 })
    const events = run(flier, 120, false)
    expect(events.some((e) => e.type === 'launch')).toBe(true)
    // Holding instead keeps it glued to the hill.
    const glued = rules.createGame({ seed: 2 })
    Object.assign(glued.bird, { x: up, y: rules.sample(glued, up).h, ground: true, speed: 18 })
    expect(run(glued, 50, true).some((e) => e.type === 'launch')).toBe(false)
  })

  it('rewards landings along a downslope, chains them into fever and breaks the chain on an upslope', () => {
    const state = rules.createGame({ seed: 3 })
    const down = findSlope(state, (d) => d < -0.5)
    const up = findSlope(state, (d) => d > 0.5)
    const slope = Math.atan(rules.sample(state, down).d)
    for (let i = 1; i <= rules.FEVER_CHAIN; i++) {
      airborne(state, down, slope - 0.1, 26)
      const events = touchdown(state)
      expect(events.find((e) => e.type === 'perfect')?.chain).toBe(i)
    }
    expect(state.fever).toBe(true)
    expect(state.perfects).toBe(rules.FEVER_CHAIN)
    const before = state.score
    airborne(state, down, slope - 0.1, 26)
    const fever = touchdown(state).find((e) => e.type === 'perfect')
    expect(fever.points).toBe((60 + 15 * (rules.FEVER_CHAIN + 1)) * 2)
    expect(state.score).toBeGreaterThan(before)

    airborne(state, up, -0.6, 26)
    const thud = touchdown(state)
    expect(thud.some((e) => e.type === 'thud')).toBe(true)
    expect(thud.some((e) => e.type === 'feverEnd')).toBe(true)
    expect([state.chain, state.fever, state.maxChain]).toEqual([0, false, rules.FEVER_CHAIN + 1])
    expect(state.bird.speed).toBeLessThan(26 * 0.7)
  })

  it('does not judge short hops', () => {
    const state = rules.createGame({ seed: 3 })
    const down = findSlope(state, (d) => d < -0.5)
    airborne(state, down, Math.atan(rules.sample(state, down).d) - 0.1, 20, 0.1)
    const events = touchdown(state)
    expect(events.some((e) => ['perfect', 'land', 'thud'].includes(e.type))).toBe(false)
    expect(state.bird.ground).toBe(true)
  })

  it('splashes into the sea when steep or slow, and skips off it when fast and shallow', () => {
    const state = rules.createGame({ seed: 4 })
    const isle = state.world.islands[0]
    const strait = (isle.x1 + isle.end) / 2
    Object.assign(state.bird, { x: strait, y: 0.05, vx: 30, vy: -6, ground: false, air: 1 })
    const skip = run(state, 1)
    expect(skip.some((e) => e.type === 'skip')).toBe(true)
    expect(state.bird.ground).toBe(false)
    expect(state.bird.vy).toBeGreaterThan(0)

    state.chain = 2
    Object.assign(state.bird, { x: strait, y: 0.05, vx: 12, vy: -14, ground: false, air: 1 })
    const splash = run(state, 1)
    expect(splash.some((e) => e.type === 'splash')).toBe(true)
    expect(state.chain).toBe(0)
    run(state, 120)
    expect(state.bird.speed).toBeLessThan(8)
  })

  it('counts a cloud touch once per flight', () => {
    const state = rules.createGame({ seed: 5 })
    const x = findSlope(state, (d) => Math.abs(d) < 0.05)
    Object.assign(state.bird, { x, y: rules.sample(state, x).h + rules.CLOUD + 2, vx: 20, vy: 2, ground: false, air: 0, apex: 0, cloud: false })
    const events = run(state, 20)
    expect(events.filter((e) => e.type === 'cloud')).toHaveLength(1)
    expect(state.clouds).toBe(1)
  })
})

describe('dusk glider rounds', () => {
  it('extends daylight on reaching the next island and wakes the bird if night has fallen', () => {
    const state = rules.createGame({ seed: 6 })
    const next = state.world.islands[0]
    rules.ensure(state, next.end + 200)
    const second = state.world.islands[1]
    state.night = true; state.daylight = 0
    Object.assign(state.bird, { x: second.x0 - 0.5, y: rules.sample(state, second.x0 - 0.5).h, ground: true, speed: 20 })
    const events = run(state, 5)
    const island = events.find((e) => e.type === 'island')
    expect(island).toMatchObject({ index: 1, dawn: true })
    expect(state.night).toBe(false)
    expect(state.daylight).toBeGreaterThan(15)
    expect(state.island).toBe(1)
  })

  it('ends at nightfall once the bird settles, and reports the packed result metrics', () => {
    const state = rules.createGame({ seed: 9 })
    let last = 0
    const events: Any[] = []
    for (let i = 0; i < 60 * 400 && state.phase === 'playing'; i++) {
      events.push(...rules.step(state, { hold: state.bird.ground && rules.sample(state, state.bird.x).d < 0 }, DT))
      expect(state.score).toBeGreaterThanOrEqual(last)
      last = state.score
    }
    expect(state.phase).toBe('over')
    expect(events.some((e) => e.type === 'night')).toBe(true)
    expect(events.at(-1)).toEqual({ type: 'over', reason: 'night' })
    const result = rules.result(state)
    expect(result).toMatchObject({ score: state.score, perfects: state.perfects, maxChain: state.maxChain, islands: state.island + 1, flights: state.flights })
    expect(result.islands).toBeGreaterThanOrEqual(1)
    expect(result.distance).toBeGreaterThan(300)
    for (const value of Object.values(result)) expect(Number.isInteger(value) && (value as number) >= 0).toBe(true)
    expect(rules.step(state, { hold: true }, DT)).toEqual([])
  })
})

describe('dusk glider guidance', () => {
  const KIND_OF_EVENT: Record<string, string> = { perfect: 'perfect', land: 'good', thud: 'thud', splash: 'water', skip: 'skip' }
  const RANK: Record<string, number> = { perfect: 5, good: 3, skip: 3, hop: 2, far: 2, thud: 1, water: 0 }

  // A small deterministic generator, so the sampled flights are the same on every run.
  function lcg(seed: number) {
    let s = seed
    return () => { s = (Math.imul(s, 1664525) + 1013904223) >>> 0; return s / 4294967296 }
  }

  it('previews exactly the landing that the simulation then produces', () => {
    const random = lcg(42)
    let checked = 0
    const kinds = new Set<string>()
    for (let i = 0; i < 160; i++) {
      const seed = 1 + Math.floor(random() * 50), hold = random() < 0.5
      const x = 20 + random() * 900, angle = (random() - 0.5) * 1.6, speed = 8 + random() * 40, air = random() * 1.2, lift = random() * 6
      // The world is generated from its seed alone, so two games with the same seed share the same terrain.
      const make = () => { const s = rules.createGame({ seed }); airborne(s, x, angle, speed, air); s.bird.y += lift; return s }
      const base = make(), live = make()
      const preview = rules.predictLanding(base, hold)

      let landed: Any = null
      for (let f = 0; f < 60 * 7 && !landed && live.phase === 'playing'; f++) {
        const events = rules.step(live, { hold }, DT)
        const hit = events.find((e) => e.type in KIND_OF_EVENT)
        if (hit) landed = { kind: KIND_OF_EVENT[hit.type], x: hit.x }
        else if (live.bird.ground) landed = { kind: 'hop', x: live.bird.x }
      }
      if (!landed) { expect(preview.kind).toBe('far'); continue }
      expect(preview.kind).toBe(landed.kind)
      expect(preview.x).toBeCloseTo(landed.x, 6)
      kinds.add(preview.kind)
      checked += 1
    }
    expect(checked).toBeGreaterThan(100)
    // The sample has to cover the interesting outcomes, otherwise the comparison proves little.
    for (const kind of ['perfect', 'good', 'thud']) expect([...kinds]).toContain(kind)
  })

  it('returns no preview on the ground, and marks flights that never land as far', () => {
    const state = rules.createGame({ seed: 2 })
    run(state, 240)
    expect(state.bird.ground).toBe(true)
    expect(rules.predictLanding(state, true)).toBeNull()
    const x = findSlope(state, (d) => Math.abs(d) < 0.05)
    airborne(state, x, 1.2, 40)
    expect(rules.predictLanding(state, false, 0.5).kind).toBe('far')
  })

  it('tells the player to hold on a downslope and release when the hill rises', () => {
    const state = rules.createGame({ seed: 3 })
    const down = findSlope(state, (d) => d < -0.3)
    const up = findSlope(state, (d) => d > 0.3)
    Object.assign(state.bird, { x: down, y: rules.sample(state, down).h, ground: true })
    expect(rules.advice(state)).toMatchObject({ phase: 'ground', hold: true, reason: 'downhill' })
    Object.assign(state.bird, { x: up, y: rules.sample(state, up).h, ground: true })
    expect(rules.advice(state)).toMatchObject({ phase: 'ground', hold: false, reason: 'rise' })
    const strait = (state.world.islands[0].x1 + state.world.islands[0].end) / 2
    Object.assign(state.bird, { x: strait, y: 0, ground: true })
    expect(rules.advice(state)).toMatchObject({ phase: 'water', hold: false })
  })

  it('advises the input with the better preview while airborne', () => {
    const random = lcg(7)
    let holds = 0
    for (let i = 0; i < 80; i++) {
      const state = rules.createGame({ seed: 1 + Math.floor(random() * 30) })
      airborne(state, 30 + random() * 600, (random() - 0.4) * 1.4, 10 + random() * 36, random())
      state.bird.y += random() * 8
      const tip = rules.advice(state)
      expect(tip.phase).toBe('air')
      const chosen = tip.hold ? tip.dive : tip.glide
      const other = tip.hold ? tip.glide : tip.dive
      expect(tip.kind).toBe(chosen.kind)
      expect(RANK[chosen.kind]).toBeGreaterThanOrEqual(RANK[other.kind])
      if (tip.hold) holds += 1
    }
    // Neither answer may dominate, otherwise the preview would not be teaching anything.
    expect(holds).toBeGreaterThan(5)
    expect(holds).toBeLessThan(75)
  })

  it('reaches several islands with perfect landings when the player simply follows the advice', () => {
    const played = (follow: boolean) => {
      const state = rules.createGame({ seed: 9 })
      for (let i = 0; i < 60 * 300 && state.phase === 'playing'; i++) rules.step(state, { hold: follow ? rules.advice(state).hold : false }, DT)
      return rules.result(state)
    }
    const coached = played(true), idle = played(false)
    expect(coached.islands).toBeGreaterThanOrEqual(3)
    expect(coached.perfects).toBeGreaterThanOrEqual(5)
    expect(coached.score).toBeGreaterThan(idle.score * 3)
  })

  it('forgives wider landings on the first island and tightens to the normal window by the fourth', () => {
    const windows = [0, 1, 2, 3, 9].map((island) => rules.perfectWindow(island))
    expect(windows[0]).toBeCloseTo(rules.PERFECT_EASY)
    expect(windows[3]).toBeCloseTo(rules.PERFECT)
    expect(windows[4]).toBeCloseTo(rules.PERFECT)
    for (let i = 1; i < windows.length; i++) expect(windows[i]).toBeLessThanOrEqual(windows[i - 1])
  })

  it('points at the next island and counts the distance down as the bird advances', () => {
    const state = rules.createGame({ seed: 4 })
    const first = rules.nextIsland(state)
    expect(first.index).toBe(1)
    expect(first.x).toBe(state.world.islands[1].x0)
    state.bird.x += 100
    expect(rules.nextIsland(state).distance).toBe(first.distance - 100)
  })
})
