/* 暮色滑翔 · deterministic simulation: island terrain, glide physics, landings, daylight and scoring. No DOM access. */
(() => {
  'use strict'

  const G = 22 // ground and passive-air gravity scale, m/s²
  const AIR_G = 16.5 // gravity while gliding with the finger up
  const DIVE = 52 // extra weight while the finger is down
  const FRICTION = 0.22
  const DRAG = 0.00085
  const MIN_SPEED = 6.5 // the bird flaps to keep at least this pace in daylight
  const MAX_SPEED = 64
  const SEA = 0
  const SEABED = -7
  const DAY_START = 56
  const PERFECT = 0.25 // radians between flight path and slope for a perfect slide
  const PERFECT_EASY = 0.4 // the first island forgives more, easing back to PERFECT by the fourth
  const GOOD = 0.6
  const JUDGE_AIR = 0.32 // shorter hops re-attach silently
  const FLIGHT_AIR = 0.6
  const CLOUD = 24 // metres above the ground that count as touching the clouds
  const FEVER_CHAIN = 3
  const START_X = 8
  const SKIP_SPEED = 24
  const HOLD_SLOPE = -0.03 // steeper downhill than this is where a good player keeps the finger down

  function rng(seed) {
    let s = (seed >>> 0) || 1
    return () => {
      s = (s + 0x6d2b79f5) >>> 0
      let t = s
      t = Math.imul(t ^ (t >>> 15), t | 1)
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296
    }
  }

  // ------------------------------------------------------------ terrain

  // Key points alternate between valleys and peaks; cosine interpolation gives zero slope at every key point
  // and smooth hills in between. Islands end with a tall ramp and a strait of sea before the next beach.
  function addIsland(world) {
    const n = world.islands.length
    const r = world.random
    const d = Math.min(1, n / 6)
    const pts = world.points
    let x = pts[pts.length - 1][0]
    const x0 = x
    const hills = Math.min(12, 7 + n)
    const add = (dx, y) => { x += dx; pts.push([x, y]) }
    // A low dune right after the beach: a strong ramp jump clears it and lands on its far side.
    if (n === 0) add(70, 18 + r() * 4)
    else add(30 + r() * 6, 8 + r() * 3)
    const firstPeak = x
    for (let i = 0; i < hills; i++) {
      const span = (24 + r() * 18) * (1 - 0.22 * d)
      const big = r() < 0.18
      add(span, 2.4 + r() * 3.6)
      add(span * (big ? 1.3 : 1) * (0.9 + r() * 0.25), (big ? 19 : 9.5 + r() * 7) + 5 * d)
    }
    add(46 + r() * 10, 3 + r() * 2)
    add(40 + r() * 8, 23 + 4 * d + r() * 3)
    const ramp = x
    add(30, SEABED)
    const x1 = x
    add(16 + 10 * d + r() * 8, SEABED)
    world.islands.push({ index: n, x0, firstPeak, ramp, x1, end: x })
  }

  function createWorld(seed) {
    const world = { random: rng(seed), points: [[-260, SEABED], [-70, SEABED]], islands: [] }
    addIsland(world)
    return world
  }

  function ensure(state, x) {
    const world = state.world
    while (world.points[world.points.length - 1][0] < x + 60) addIsland(world)
  }

  function segment(points, x) {
    let lo = 0, hi = points.length - 1
    if (x <= points[0][0]) return 0
    if (x >= points[hi][0]) return hi - 1
    while (hi - lo > 1) {
      const mid = (lo + hi) >> 1
      if (points[mid][0] <= x) lo = mid; else hi = mid
    }
    return lo
  }

  /** Raw terrain height with first and second derivatives. */
  function terrain(state, x) {
    ensure(state, x)
    const pts = state.world.points
    const i = segment(pts, x)
    const [xa, ya] = pts[i], [xb, yb] = pts[i + 1]
    const w = xb - xa
    const t = Math.max(0, Math.min(1, (x - xa) / w))
    const dy = yb - ya
    return {
      h: ya + dy * (1 - Math.cos(Math.PI * t)) / 2,
      d: dy * Math.PI / (2 * w) * Math.sin(Math.PI * t),
      dd: dy * Math.PI * Math.PI / (2 * w * w) * Math.cos(Math.PI * t),
    }
  }

  /** The surface the bird rests on: the hills, or the sea where the hills dip under it. */
  function sample(state, x) {
    const s = terrain(state, x)
    if (s.h < SEA) return { h: SEA, d: 0, dd: 0, water: true, ground: s.h }
    return { h: s.h, d: s.d, dd: s.dd, water: false, ground: s.h }
  }

  function islandAt(state, x) {
    ensure(state, x)
    const list = state.world.islands
    for (let i = list.length - 1; i >= 0; i--) if (x >= list[i].x0) return list[i]
    return list[0]
  }

  function islands(state, x0, x1) {
    ensure(state, x1)
    return state.world.islands.filter((isle) => isle.end >= x0 && isle.x0 <= x1)
  }

  // ------------------------------------------------------------ game

  function createGame(options = {}) {
    const seed = Number.isFinite(options.seed) ? options.seed : Math.floor(Math.random() * 1e9)
    const state = {
      seed, world: createWorld(seed), phase: 'playing', time: 0,
      daylight: DAY_START, night: false, nightTime: 0, island: 0,
      bird: { x: START_X, y: 0, vx: 11, vy: 0, speed: 11, ground: false, air: 0, apex: 0, cloud: false, flap: 0, hold: false },
      scoreF: 0, score: 0, chain: 0, maxChain: 0, fever: false, perfects: 0, flights: 0, clouds: 0, splashes: 0, skips: 0,
      distance: 0, maxX: START_X, endReason: null, best: { speed: 0, height: 0 },
    }
    state.bird.y = sample(state, START_X).h + 2.5
    return state
  }

  function mult(state) { return state.fever ? 2 : 1 }

  function award(state, points) {
    state.scoreF += points
    state.score = Math.floor(state.scoreF)
  }

  /** The window depends on the island being landed on, so a flight across the strait is judged where it ends. */
  function perfectWindow(islandIndex) {
    return PERFECT + (PERFECT_EASY - PERFECT) * Math.max(0, 1 - islandIndex / 3)
  }

  /**
   * What a touchdown at `x` with this velocity counts as. Shared by the real landing and the landing preview,
   * so the marker the player sees can never disagree with what happens.
   */
  function judge(state, x, vx, vy, air) {
    const s = sample(state, x)
    const theta = Math.atan(s.d)
    const v = Math.hypot(vx, vy)
    const diff = Math.abs(Math.atan2(vy, vx) - theta)
    let kind
    if (s.water && v > SKIP_SPEED && -vy < v * 0.42) kind = 'skip'
    else if (air < JUDGE_AIR) kind = 'hop'
    else if (s.water) kind = 'water'
    else if (theta < -0.1 && diff < perfectWindow(islandAt(state, x).index)) kind = 'perfect'
    else if (diff < GOOD) kind = 'good'
    else kind = 'thud'
    return { kind, diff, theta, v, h: s.h, water: s.water }
  }

  function land(state, events) {
    const b = state.bird
    const s = sample(state, b.x)
    const air = b.air
    const { kind, theta, v, diff } = judge(state, b.x, b.vx, b.vy, air)
    const along = v * Math.max(0, Math.cos(diff))
    b.ground = true
    b.y = s.h
    if (air >= FLIGHT_AIR) {
      state.flights += 1
      const points = Math.floor(air * 25) * mult(state)
      award(state, points)
      events.push({ type: 'flight', air, points, apex: b.apex })
    }
    if (kind === 'skip') {
      // A fast, shallow touch skips off the sea like a stone and keeps the chain.
      b.ground = false
      b.y = SEA + 0.01
      b.vy = Math.max(3, -b.vy * 0.5)
      b.vx *= 0.86
      b.speed = Math.hypot(b.vx, b.vy)
      b.air = 0; b.apex = 0
      state.skips += 1
      events.push({ type: 'skip', x: b.x, y: SEA, speed: v })
      return
    }
    if (kind === 'hop') {
      b.speed = Math.max(along, s.water ? 0 : b.speed * 0.9)
    } else if (kind === 'water') {
      b.speed = along * 0.6
      state.chain = 0; state.splashes += 1
      events.push({ type: 'splash', x: b.x, y: SEA, speed: v })
    } else if (kind === 'perfect') {
      state.chain += 1; state.perfects += 1
      state.maxChain = Math.max(state.maxChain, state.chain)
      b.speed = along * 1.08 + 1.5
      const points = (60 + 15 * Math.min(state.chain, 8)) * mult(state)
      award(state, points)
      events.push({ type: 'perfect', x: b.x, y: b.y, chain: state.chain, points })
    } else if (kind === 'good') {
      b.speed = along
      events.push({ type: 'land', x: b.x, y: b.y, speed: v })
    } else {
      b.speed = Math.max(0, v * Math.max(0.2, Math.cos(diff)) * 0.7)
      state.chain = 0
      events.push({ type: 'thud', x: b.x, y: b.y, speed: v })
    }
    const fever = state.chain >= FEVER_CHAIN
    if (fever !== state.fever) { state.fever = fever; events.push({ type: fever ? 'fever' : 'feverEnd', x: b.x, y: b.y }) }
    b.speed = Math.min(MAX_SPEED, b.speed)
    b.air = 0
  }

  /**
   * Advances one fixed step. `input.hold` is the single control: weight while it is down, glide while it is up.
   * Returns presentation events.
   */
  function step(state, input = {}, dt = 1 / 60) {
    const events = []
    if (state.phase !== 'playing') return events
    const b = state.bird
    const hold = Boolean(input.hold)
    b.hold = hold
    state.time += dt

    if (!state.night) {
      state.daylight = Math.max(0, state.daylight - dt)
      if (state.daylight <= 0) { state.night = true; events.push({ type: 'night' }) }
    } else state.nightTime += dt

    const x0 = b.x
    if (b.ground) {
      let s = sample(state, b.x)
      const theta = Math.atan(s.d)
      const g = G + (hold ? DIVE : 0)
      let a
      if (s.water) a = -(b.speed - 4) * 1.7
      else a = -g * Math.sin(theta) - FRICTION - DRAG * b.speed * b.speed
      if (state.night) a -= 7
      b.speed += a * dt
      if (!state.night && b.speed < MIN_SPEED) {
        b.speed += (MIN_SPEED - b.speed) * Math.min(1, 3 * dt)
        b.flap -= dt
        if (b.flap <= 0) { b.flap = 0.38; events.push({ type: 'flap', x: b.x, y: b.y }) }
      }
      b.speed = Math.max(0, Math.min(MAX_SPEED, b.speed))
      b.x += b.speed * Math.cos(theta) * dt
      s = sample(state, b.x)
      const t2 = Math.atan(s.d)
      b.y = s.h
      b.vx = b.speed * Math.cos(t2); b.vy = b.speed * Math.sin(t2)
      // Leave the ground when the hill curves away faster than gravity can hold the bird to it.
      const curve = -s.dd / Math.pow(1 + s.d * s.d, 1.5)
      if (!s.water && curve > 0 && b.speed * b.speed * curve > g * Math.cos(t2)) {
        b.ground = false; b.air = 0; b.apex = 0; b.cloud = false
        events.push({ type: 'launch', x: b.x, y: b.y, speed: b.speed })
      }
    } else {
      const g = hold ? G + DIVE : AIR_G
      b.vy -= g * dt
      const v = Math.hypot(b.vx, b.vy)
      const keep = 1 - DRAG * v * dt
      b.vx *= keep; b.vy *= keep
      b.x += b.vx * dt; b.y += b.vy * dt
      b.air += dt
      b.speed = Math.hypot(b.vx, b.vy)
      const s = sample(state, b.x)
      const above = b.y - s.h
      b.apex = Math.max(b.apex, above)
      state.best.height = Math.max(state.best.height, above)
      if (!b.cloud && above >= CLOUD) {
        b.cloud = true; state.clouds += 1
        const points = 150 * mult(state)
        award(state, points)
        events.push({ type: 'cloud', x: b.x, y: b.y, points })
      }
      if (b.y <= s.h) land(state, events)
    }
    state.best.speed = Math.max(state.best.speed, b.speed)

    if (b.x > state.maxX) {
      award(state, (b.x - state.maxX) * mult(state))
      state.maxX = b.x
      state.distance = Math.floor(state.maxX - START_X)
    }
    if (b.x > x0) {
      const isle = islandAt(state, b.x)
      if (isle.index > state.island) {
        state.island = isle.index
        const bonus = Math.max(9, 22 - 1.6 * isle.index)
        const wasNight = state.night
        state.daylight = (state.night ? 0 : state.daylight) + bonus
        state.night = false; state.nightTime = 0
        const points = 200 + 100 * isle.index
        award(state, points)
        events.push({ type: 'island', index: isle.index, bonus, points, dawn: wasNight })
      }
    }

    if (state.night && ((b.ground && b.speed < 1.2) || state.nightTime > 14)) {
      state.phase = 'over'; state.endReason = 'night'
      events.push({ type: 'over', reason: 'night' })
    }
    return events
  }

  // ------------------------------------------------------------ guidance

  const RANK = { perfect: 5, good: 3, skip: 3, hop: 2, far: 2, thud: 1, water: 0 }

  /**
   * Where and how the airborne bird would touch down if `hold` stayed as given. It steps the exact flight
   * integration of `step` without touching the state. Returns null on the ground.
   */
  function predictLanding(state, hold, seconds = 6) {
    const b = state.bird
    if (b.ground) return null
    const dt = 1 / 60
    const g = hold ? G + DIVE : AIR_G
    let { x, y, vx, vy, air } = b
    for (let t = 0; t < seconds; t += dt) {
      vy -= g * dt
      const keep = 1 - DRAG * Math.hypot(vx, vy) * dt
      vx *= keep; vy *= keep
      x += vx * dt; y += vy * dt
      air += dt
      const s = sample(state, x)
      if (y <= s.h) {
        const j = judge(state, x, vx, vy, air)
        return { kind: j.kind, diff: j.diff, x, y: s.h, t: air - b.air, hold }
      }
    }
    return { kind: 'far', diff: Math.PI, x, y, t: seconds, hold }
  }

  /**
   * What a good player would do right now: hold while sliding downhill, release at the bottom and on the way up,
   * and in the air pick whichever input makes the better landing. `glide` and `dive` are the two previews.
   */
  function advice(state) {
    const b = state.bird
    if (b.ground) {
      const s = sample(state, b.x)
      if (s.water) return { phase: 'water', hold: false, reason: 'water', kind: null, glide: null, dive: null }
      const down = s.d < HOLD_SLOPE
      return { phase: 'ground', hold: down, reason: down ? 'downhill' : 'rise', kind: null, glide: null, dive: null }
    }
    const glide = predictLanding(state, false), dive = predictLanding(state, true)
    const value = (p) => RANK[p.kind] * 10 - p.diff
    const best = value(dive) > value(glide) + 0.02 ? dive : glide
    return { phase: 'air', hold: best.hold, reason: best.hold ? 'dive' : 'glide', kind: best.kind, glide, dive }
  }

  /** The next island the bird has not reached yet, with the distance to its beach. */
  function nextIsland(state) {
    const index = state.island + 1
    ensure(state, state.bird.x + 1800)
    const isle = state.world.islands[index]
    return isle ? { index, x: isle.x0, distance: Math.max(0, Math.round(isle.x0 - state.bird.x)) } : null
  }

  function result(state) {
    return {
      score: state.score, perfects: state.perfects, maxChain: state.maxChain, islands: state.island + 1,
      flights: state.flights, clouds: state.clouds, distance: state.distance, seconds: Math.floor(state.time),
      topSpeed: Math.round(state.best.speed * 3.6), topHeight: Math.round(state.best.height),
    }
  }

  globalThis.XiabanDuskGlider = {
    G, AIR_G, DIVE, SEA, SEABED, DAY_START, PERFECT, PERFECT_EASY, HOLD_SLOPE, GOOD, CLOUD, FEVER_CHAIN, START_X, MAX_SPEED,
    createGame, step, result, sample, terrain, ensure, islandAt, islands, perfectWindow, judge, predictLanding, advice, nextIsland,
  }
})()
