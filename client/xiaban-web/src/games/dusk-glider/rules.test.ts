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
