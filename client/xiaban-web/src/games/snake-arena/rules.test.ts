import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

interface Point { x: number; y: number }
interface Snake extends Point {
  id: number; mass: number; score: number; kills: number; alive: boolean
  angle: number; targetAngle: number; energy: number; boosting: boolean; boostLocked: boolean
  invulnerable: number; aiTimer: number; respawnAt: number; trail: Point[]; previous: Point
}
interface Food extends Point { id: number; value: number; remains: boolean; color: number; phase: number; shape?: string; ownerId?: number | null }
interface GameEvent { type: string; id?: number; killerId?: number | null; key?: string; value?: number; reward?: number; victimMass?: number }
interface State {
  snakes: Snake[]; food: Food[]; phase: string; time: number; remaining: number | null; roundType: string
  reason: string | null; events: GameEvent[]; reviveUsed: boolean
}
interface Rules {
  WORLD: { width: number; height: number }; MIN_MASS: number
  MAX_MASS: number; MAX_FOOD: number; BASE_SPEED: number; BOOST_SPEED: number; TURN_SPEED: number; FOOD_SHAPES: string[]
  createGame(options?: { seed?: number; mode?: string; roundType?: string }): State
  step(state: State, input?: { angle?: number | null; boost?: boolean }, dt?: number): GameEvent[]
  bodyPoints(snake: Snake): Point[]
  segmentDistanceSq(point: Point, a: Point, b: Point): number
  segmentsDistanceSq(a: Point, b: Point, c: Point, d: Point): number
  rankings(state: State): Snake[]
  radius(snake: Snake): number
  bodyLength(snake: Snake): number
  activateShield(state: State, seconds?: number): boolean
  revivePlayer(state: State): boolean
}
const context: { XiabanSnakeArena?: Rules } = {}
runInNewContext(readFileSync(new URL('../../../public/game-packs/snake-arena/rules.js', import.meta.url), 'utf8'), context)
const rules = context.XiabanSnakeArena!

function solo() {
  const state = rules.createGame({ seed: 42 })
  state.snakes = [state.snakes[0]]
  state.food = []
  state.snakes[0].invulnerable = 0
  return state
}
function position(snake: Snake, x: number, y: number, angle: number) {
  Object.assign(snake, { x, y, angle, targetAngle: angle, invulnerable: 0, aiTimer: Infinity })
  snake.trail = [{ x: x - Math.cos(angle) * 6, y: y - Math.sin(angle) * 6 }]
  return snake
}
function food(x: number, y: number, value = 1): Food {
  return { x, y, value, id: 999, remains: false, color: 0, phase: 0 }
}

describe('snake arena shipped simulation', () => {
  it('creates reproducible independent arenas with compatible legacy modes and a fixed mixed opponent count', () => {
    const a = rules.createGame({ seed: 7 })
    const b = rules.createGame({ seed: 7 })
    expect(a.snakes).toEqual(b.snakes)
    expect(a.food).toEqual(b.food)
    expect(a.snakes).toHaveLength(11)
    const lively = rules.createGame({ mode: 'lively', seed: 7 })
    expect(lively.snakes).toEqual(a.snakes)
    expect(lively.food).toEqual(a.food)
    expect(a.snakes[0].invulnerable).toBe(2)
    expect(lively.snakes[0].invulnerable).toBe(2)
    expect(a.snakes.filter(snake => snake.id && snake.mass < 150)).toHaveLength(4)
    expect(a.snakes.filter(snake => snake.id && snake.mass >= 150 && snake.mass < 500)).toHaveLength(4)
    expect(a.snakes.filter(snake => snake.id && snake.mass >= 500)).toHaveLength(2)
    for (const bot of a.snakes.slice(1)) expect(Math.hypot(bot.x - a.snakes[0].x, bot.y - a.snakes[0].y)).toBeGreaterThan(500)
    a.snakes[0].trail[0].x = -10
    expect(b.snakes[0].trail[0].x).toBeGreaterThan(0)
    for (const snake of b.snakes) for (const point of rules.bodyPoints(snake)) {
      expect(point.x).toBeGreaterThan(20)
      expect(point.x).toBeLessThan(rules.WORLD.width - 20)
      expect(point.y).toBeGreaterThan(20)
      expect(point.y).toBeLessThan(rules.WORLD.height - 20)
    }
  })

  it('limits steering speed and takes the short turn across the angle seam', () => {
    const state = solo(), player = state.snakes[0]
    player.angle = Math.PI - 0.01
    rules.step(state, { angle: -Math.PI + 0.01 })
    expect(player.angle).toBeCloseTo(Math.PI + 0.01)
    const before = player.angle
    rules.step(state, { angle: 0 })
    expect(Math.abs(player.angle - before)).toBeCloseTo(rules.TURN_SPEED / 60)
    expect(Math.hypot(player.x - player.previous.x, player.y - player.previous.y)).toBeCloseTo(152 / 60)
  })

  it('follows a sampled curve with no detached tail when turning and growing', () => {
    const state = solo(), player = state.snakes[0]
    for (let i = 0; i < 200; i += 1) {
      if (i === 70) player.mass += 40
      rules.step(state, { angle: i * 0.025 })
      const points = rules.bodyPoints(player)
      for (let j = 1; j < points.length; j += 1) {
        expect(Math.hypot(points[j].x - points[j - 1].x, points[j].y - points[j - 1].y)).toBeLessThanOrEqual(6.001)
      }
    }
    expect(player.alive).toBe(true)
  })

  it('awards points and mass once, and lets the closest head win contested food', () => {
    const state = rules.createGame({ seed: 1 })
    const player = position(state.snakes[0], 500, 500, 0)
    const bot = position(state.snakes[1], 520, 500, 0)
    player.invulnerable = bot.invulnerable = 1
    state.snakes = [player, bot]
    state.food = [food(502, 500, 3)]
    const original = player.mass
    const events = rules.step(state)
    expect(player.mass).toBe(original + 3)
    expect(player.score).toBe(30)
    expect(bot.score).toBe(0)
    expect(state.food).toHaveLength(0)
    expect(events.filter((event) => event.type === 'eat')).toHaveLength(1)
    rules.step(state)
    expect(player.score).toBe(30)
  })

  it('boosts faster, sheds mass without reducing points, and stops at the mass floor', () => {
    const state = solo(), player = state.snakes[0]
    player.score = 400
    rules.step(state, { boost: true }, 1 / 30)
    expect(Math.hypot(player.x - player.previous.x, player.y - player.previous.y)).toBeCloseTo(280 / 30)
    expect(player.energy).toBeLessThan(100)
    for (let i = 0; i < 20; i += 1) rules.step(state, { boost: true })
    expect(player.mass).toBeLessThan(60)
    expect(player.score).toBe(400)
    expect(state.food.length).toBeGreaterThan(0)
    player.mass = rules.MIN_MASS
    rules.step(state, { boost: true })
    expect(player.boosting).toBe(false)
    expect(player.mass).toBe(rules.MIN_MASS)
  })

  it('recharges exhausted boost with hysteresis rather than flickering each tick', () => {
    const state = solo(), player = state.snakes[0]
    player.energy = 0
    for (let i = 0; i < 30; i += 1) {
      rules.step(state, { boost: true })
      expect(player.boosting).toBe(false)
    }
    expect(player.energy).toBeCloseTo(11)
    for (let i = 0; i < 42; i += 1) rules.step(state, { boost: true })
    expect(player.boosting).toBe(true)
  })

  it('lets the player cross its own body and ignores spawning ghosts', () => {
    const state = solo(), player = position(state.snakes[0], 500, 500, 0)
    player.trail = [{ x: 504, y: 470 }, { x: 504, y: 530 }]
    rules.step(state)
    expect(player.alive).toBe(true)
    const other = rules.createGame({ seed: 2 }).snakes[1]
    position(other, 530, 510, Math.PI / 2)
    other.trail = [{ x: 504, y: 475 }, { x: 504, y: 530 }]
    other.invulnerable = 1
    state.snakes.push(other)
    rules.step(state)
    expect(player.alive).toBe(true)
  })

  it('eliminates a head hitting another body and drops edible remains', () => {
    const state = rules.createGame({ seed: 4 })
    const player = position(state.snakes[0], 500, 500, 0)
    const other = position(state.snakes[1], 505, 550, Math.PI / 2)
    other.trail = [{ x: 505, y: 545 }, { x: 505, y: 450 }]
    state.snakes = [player, other]; state.food = []
    rules.step(state)
    expect(player.alive).toBe(false)
    expect(other.alive).toBe(true)
    expect(other.kills).toBe(1)
    expect(other.score).toBe(80 + Math.floor(player.mass * 0.25))
    expect(state.reason).toBe('collision')
    expect(state.food.some((entry) => entry.remains)).toBe(true)
    const snapshot = JSON.stringify({ time: state.time, player })
    rules.step(state, { boost: true })
    expect(JSON.stringify({ time: state.time, player })).toBe(snapshot)
  })

  it('resolves head-on collisions simultaneously and emits only one death per snake', () => {
    const state = rules.createGame({ seed: 5 })
    const a = position(state.snakes[0], 500, 500, 0)
    const b = position(state.snakes[1], 525, 500, Math.PI)
    state.snakes = [a, b]; state.food = []
    const events = rules.step(state, {}, 1 / 30)
    expect(a.alive).toBe(false)
    expect(b.alive).toBe(false)
    expect(events.filter((event) => event.type === 'death')).toHaveLength(2)
    expect(a.kills).toBe(1)
    expect(b.kills).toBe(1)
  })

  it('detects intersecting swept paths even if their endpoints are apart', () => {
    expect(rules.segmentsDistanceSq({ x: -10, y: 0 }, { x: 10, y: 0 }, { x: 0, y: -10 }, { x: 0, y: 10 })).toBe(0)
    expect(rules.segmentDistanceSq({ x: 5, y: 3 }, { x: 0, y: 0 }, { x: 10, y: 0 })).toBe(9)
  })

  it('makes the visible border lethal and handles the exact three-minute finish', () => {
    const wall = solo()
    position(wall.snakes[0], 2387, 500, 0)
    rules.step(wall)
    expect(wall.reason).toBe('wall')
    const timed = solo()
    timed.time = 179.99
    const events = rules.step(timed)
    expect(timed.remaining).toBe(0)
    expect(timed.phase).toBe('over')
    expect(timed.reason).toBe('timeout')
    expect(timed.snakes[0].alive).toBe(true)
    expect(events.some((event) => event.type === 'finish')).toBe(true)
  })

  it('does not advance on invalid delta times or after a round ends', () => {
    const state = solo(), before = state.time
    for (const dt of [0, -1, NaN, Infinity]) rules.step(state, {}, dt)
    expect(state.time).toBe(before)
    state.phase = 'over'
    rules.step(state)
    expect(state.time).toBe(before)
  })

  it('keeps endless survival running beyond three minutes and ends only when the player dies', () => {
    const state = rules.createGame({ seed: 42, roundType: 'endless' })
    state.snakes = [state.snakes[0]]
    for (let frame = 0; frame < 360 * 30; frame += 1) {
      rules.step(state, { angle: -0.2 + state.time * 0.6 }, 1 / 30)
    }
    expect(state.time).toBeGreaterThan(359)
    expect(state.phase).toBe('playing')
    expect(state.remaining).toBeNull()
    expect(state.reason).toBeNull()
    expect(state.snakes[0].score).toBeGreaterThan(0)
    expect(state.food.length).toBeGreaterThan(0)
    expect(state.food.length).toBeLessThanOrEqual(1100)

    position(state.snakes[0], 2399, 850, 0)
    rules.step(state)
    expect(state.phase).toBe('over')
    expect(state.reason).toBe('wall')
    expect(state.events.filter(event => event.type === 'death')).toHaveLength(1)
  }, 20_000) // Six-minute simulation needs more than 5s on a small CI runner.

  it('keeps AI respawns available after the timed-round boundary in endless survival', () => {
    const state = rules.createGame({ seed: 12, roundType: 'endless', mode: 'lively' })
    expect(state.snakes).toHaveLength(11)
    state.time = 300
    const dead = state.snakes[1]
    dead.alive = false; dead.respawnAt = 290
    rules.step(state)
    expect(state.phase).toBe('playing')
    expect(state.snakes[1]).not.toBe(dead)
    expect(state.snakes[1].alive).toBe(true)
    expect(state.remaining).toBeNull()
  })

  it('defaults to timed rules for unknown round types', () => {
    const state = rules.createGame({ roundType: 'unknown' })
    expect(state.roundType).toBe('timed')
    expect(state.remaining).toBe(180)
  })

  it('respawns AI away from all living bodies and gives it a grace period', () => {
    const state = rules.createGame({ seed: 12 })
    const dead = state.snakes[1]
    dead.alive = false; dead.respawnAt = 0
    rules.step(state)
    const spawned = state.snakes[1]
    expect(spawned).not.toBe(dead)
    expect(spawned.alive).toBe(true)
    expect(spawned.invulnerable).toBeGreaterThan(2)
    const player = state.snakes[0]
    expect(Math.hypot(spawned.x - player.x, spawned.y - player.y)).toBeGreaterThan(220)
  })

  it('ranks length first with a deterministic tie break and includes the eliminated player', () => {
    const state = rules.createGame({ seed: 13 })
    state.snakes.forEach((snake) => { snake.mass = 100; snake.score = 0 })
    state.snakes[0].alive = false
    state.snakes[2].mass = 150
    state.snakes[3].score = 80
    expect(rules.rankings(state).map((snake) => snake.id).slice(0, 3)).toEqual([2, 3, 0])
  })

  it('picks up snacks slightly beyond the previous collection range', () => {
    const state = solo(), player = position(state.snakes[0], 500, 500, 0)
    const reach = rules.radius(player)
    state.food = [food(500 + rules.BASE_SPEED / 60, 500 + reach + 16, 3)]
    rules.step(state)
    expect(player.mass).toBe(63)
    expect(player.score).toBe(30)
  })

  // 1500 steps at near-MAX_MASS body length is the heaviest case here: ~2.5 s locally, so the 5 s default is too
  // tight for a slower CI runner.
  it('grows beyond 1000 while keeping mass, physical radius and sampled body work bounded', () => {
    const state = solo(), player = state.snakes[0]
    player.mass = 999
    state.food = [food(player.x, player.y, 10)]
    rules.step(state)
    expect(player.mass).toBe(1009)
    const small = { ...player, mass: 60 }
    expect(rules.radius(player)).toBeGreaterThan(rules.radius(small) * 1.8)
    expect(rules.bodyLength(player)).toBeGreaterThan(rules.bodyLength(small) * 5)
    player.mass = rules.MAX_MASS - 2
    state.food = [food(player.x, player.y, 100)]
    rules.step(state)
    expect(player.mass).toBe(rules.MAX_MASS)
    for (let frame = 0; frame < 1500; frame += 1) rules.step(state, { angle: frame * 0.015 })
    expect(player.alive).toBe(true)
    expect(player.trail.length).toBeLessThanOrEqual(519)
    expect(rules.bodyLength(player)).toBeLessThanOrEqual(3105)
    expect(rules.radius(player)).toBeLessThanOrEqual(42)
  }, 30000)

  it('gives larger opponents proportionally valuable remains and bounded food even on a full map', () => {
    function eliminate(mass: number, full = false) {
      const state = solo()
      const bot = position(rules.createGame({ seed: 31 }).snakes[1], 2399, 400, 0)
      bot.mass = mass
      state.snakes.push(bot)
      if (full) state.food = Array.from({ length: rules.MAX_FOOD }, (_, id) => ({ ...food(100, 100), id }))
      rules.step(state)
      return state
    }
    const small = eliminate(60)
    const large = eliminate(2000, true)
    const value = (state: State) => state.food.filter(entry => entry.ownerId === 1).reduce((sum, entry) => sum + entry.value, 0)
    expect(value(large)).toBeGreaterThan(value(small) * 10)
    expect(value(large)).toBe(1300)
    expect(large.food.length).toBeLessThanOrEqual(rules.MAX_FOOD)
    const remains = large.food.filter(entry => entry.remains)
    expect(remains.length).toBeLessThanOrEqual(110)
    expect(remains.every(entry => entry.ownerId === 1 && rules.FOOD_SHAPES.includes(entry.shape!))).toBe(true)
  })

  it('creates every food shape and keeps AI respawns mixed, safe and inside the world', () => {
    const state = rules.createGame({ seed: 33, roundType: 'endless' })
    expect(new Set(state.food.map(entry => entry.shape)).size).toBe(5)
    expect(state.food.every(entry => entry.ownerId === null)).toBe(true)
    const masses: number[] = []
    for (let frame = 0; frame < 30; frame += 1) {
      const dead = state.snakes[1]
      dead.alive = false; dead.respawnAt = state.time
      rules.step(state, { angle: -0.2 })
      const bot = state.snakes[1]
      expect(bot).not.toBe(dead)
      masses.push(bot.mass)
      const r = rules.radius(bot)
      expect(Math.hypot(bot.x - state.snakes[0].x, bot.y - state.snakes[0].y)).toBeGreaterThan(340)
      for (const point of rules.bodyPoints(bot)) {
        expect(point.x).toBeGreaterThan(r)
        expect(point.x).toBeLessThan(rules.WORLD.width - r)
        expect(point.y).toBeGreaterThan(r)
        expect(point.y).toBeLessThan(rules.WORLD.height - r)
      }
    }
    expect(masses.some(mass => mass < 150)).toBe(true)
    expect(masses.some(mass => mass >= 180 && mass < 500)).toBe(true)
    expect(masses.some(mass => mass >= 650)).toBe(true)
  })

  it('emits each growth and ranking milestone once, including after shrinking and regrowing', () => {
    const state = solo(), player = state.snakes[0]
    player.mass = 5000
    const keys = rules.step(state).filter(event => event.type === 'milestone').map(event => event.key)
    for (const mass of [100, 250, 500, 1000, 2500, 5000]) expect(keys).toContain('growth-' + mass)
    expect(keys).toContain('top-3')
    expect(keys).toContain('first-place')
    player.mass = 60
    rules.step(state)
    player.mass = 5000
    expect(rules.step(state).some(event => event.type === 'milestone')).toBe(false)
  })

  it('rewards a small player for eliminating a giant and deduplicates first-kill milestones', () => {
    const state = solo(), player = position(state.snakes[0], 500, 500, 0)
    player.trail = [{ x: 500, y: 550 }, { x: 500, y: 620 }]
    const bot = position(rules.createGame({ seed: 34 }).snakes[1], 500, 580, -Math.PI / 2)
    bot.mass = 800
    state.snakes.push(bot)
    const events = rules.step(state)
    expect(bot.alive).toBe(false)
    expect(player.alive).toBe(true)
    expect(player.kills).toBe(1)
    expect(player.score).toBe(280)
    expect(events).toContainEqual(expect.objectContaining({ type: 'death', id: 1, reward: 280, victimMass: 800 }))
    expect(events).toContainEqual(expect.objectContaining({ type: 'milestone', key: 'first-kill' }))
    expect(events).toContainEqual(expect.objectContaining({ type: 'milestone', key: 'giant-slayer' }))
    bot.alive = true
    position(bot, 500, 580, -Math.PI / 2)
    state.food = []
    expect(rules.step(state).filter(event => event.type === 'milestone' && ['first-kill', 'giant-slayer'].includes(event.key!))).toEqual([])
    expect(player.kills).toBe(2)
  })

  it('shields against walls, bodies and heads without turning a protected snake into an AI weapon', () => {
    const state = solo(), player = position(state.snakes[0], 500, 500, 0)
    const bot = position(rules.createGame({ seed: 35 }).snakes[1], 525, 500, Math.PI)
    state.snakes.push(bot)
    expect(rules.activateShield(state, 3)).toBe(true)
    rules.step(state)
    expect(player.alive).toBe(true)
    expect(bot.alive).toBe(true)
    expect(player.kills).toBe(0)
    expect(bot.kills).toBe(0)
    position(player, 2399, 500, 0)
    rules.activateShield(state, 5)
    rules.step(state)
    expect(player.alive).toBe(true)
    expect(player.x).toBeLessThanOrEqual(rules.WORLD.width - rules.radius(player))
    expect(player.trail.every(point => point.x <= rules.WORLD.width - rules.radius(player))).toBe(true)
    player.invulnerable = 0.001
    rules.step(state)
    expect(player.alive).toBe(false)
    expect(state.reason).toBe('wall')
  })

  it('does not kill an AI that intersects the protected player body', () => {
    const state = solo(), player = position(state.snakes[0], 500, 500, 0)
    player.trail = [{ x: 500, y: 550 }, { x: 500, y: 620 }]
    const bot = position(rules.createGame({ seed: 36 }).snakes[1], 500, 580, -Math.PI / 2)
    state.snakes.push(bot)
    rules.activateShield(state)
    rules.step(state)
    expect(bot.alive).toBe(true)
    expect(player.kills).toBe(0)
    player.invulnerable = 0
    rules.step(state)
    expect(bot.alive).toBe(false)
    expect(player.kills).toBe(1)
  })

  it('revives a collision loss once with 60% growth, unchanged score/time and cleared owned remains', () => {
    const state = solo(), player = position(state.snakes[0], 2380, 500, 0)
    player.mass = 1000; player.score = 1234; player.kills = 2
    state.food = [{ ...food(100, 100, 5), remains: true, ownerId: 7 }]
    rules.step(state)
    expect(state.phase).toBe('over')
    expect(state.food.some(entry => entry.ownerId === 0)).toBe(true)
    const clock = state.time, remaining = state.remaining
    expect(rules.revivePlayer(state)).toBe(true)
    const revived = state.snakes[0]
    expect(revived.mass).toBe(600)
    expect(revived.score).toBe(1234)
    expect(revived.kills).toBe(2)
    expect(revived.invulnerable).toBe(5)
    expect(revived.alive).toBe(true)
    expect(state.time).toBe(clock)
    expect(state.remaining).toBe(remaining)
    expect(state.phase).toBe('playing')
    expect(state.reviveUsed).toBe(true)
    expect(state.food.some(entry => entry.ownerId === 0)).toBe(false)
    expect(state.food.some(entry => entry.ownerId === 7)).toBe(true)
    expect(state.events).toContainEqual(expect.objectContaining({ type: 'revive', mass: 600 }))
    for (const point of rules.bodyPoints(revived)) {
      expect(point.x).toBeGreaterThan(rules.radius(revived))
      expect(point.y).toBeGreaterThan(rules.radius(revived))
    }
    position(revived, 2399, 500, 0)
    rules.step(state)
    expect(state.phase).toBe('over')
    expect(rules.revivePlayer(state)).toBe(false)
  })

  it('keeps a minimum revived mass of 60 and refuses timeout/exit revival or invalid shields', () => {
    const state = solo()
    position(state.snakes[0], 2399, 500, 0)
    rules.step(state)
    expect(rules.revivePlayer(state)).toBe(true)
    expect(state.snakes[0].mass).toBe(60)
    for (const reason of ['timeout', 'exit', 'unknown']) {
      const ended = solo()
      ended.phase = 'over'; ended.reason = reason; ended.snakes[0].alive = false
      expect(rules.revivePlayer(ended)).toBe(false)
      expect(ended.reviveUsed).toBe(false)
      expect(rules.activateShield(ended)).toBe(false)
    }
    const timed = solo()
    timed.phase = 'over'; timed.reason = 'collision'; timed.snakes[0].alive = false; timed.remaining = 0
    expect(rules.revivePlayer(timed)).toBe(false)
    for (const seconds of [0, -1, NaN, Infinity]) expect(rules.activateShield(state, seconds)).toBe(false)
  })

  it('guarantees a shielded in-bounds revive even when living bodies block every ordinary spawn', () => {
    const state = solo()
    state.phase = 'over'; state.reason = 'collision'; state.snakes[0].alive = false
    state.snakes[0].mass = 500; state.snakes[0].score = 321
    const blocker = position(rules.createGame({ seed: 37 }).snakes[1], 1200, 850, 0)
    blocker.mass = 1000
    blocker.trail = []
    for (let row = 0; row < 28; row += 1) {
      for (let col = 0; col < 25; col += 1) {
        blocker.trail.push({ x: (row % 2 ? 24 - col : col) * 100, y: row * 60 })
      }
    }
    state.snakes.push(blocker)
    expect(rules.revivePlayer(state)).toBe(true)
    const player = state.snakes[0]
    expect(player.invulnerable).toBe(5)
    expect(player.score).toBe(321)
    expect(player.mass).toBe(300)
    expect(player.trail.length).toBeLessThanOrEqual(23)
    expect(Math.hypot(player.x - blocker.x, player.y - blocker.y)).toBeGreaterThan(800)
    for (const point of rules.bodyPoints(player)) {
      expect(point.x).toBeGreaterThan(rules.radius(player))
      expect(point.x).toBeLessThan(rules.WORLD.width - rules.radius(player))
      expect(point.y).toBeGreaterThan(rules.radius(player))
      expect(point.y).toBeLessThan(rules.WORLD.height - rules.radius(player))
    }
  })

  it.each([1, 2, 7, 12, 13, 42, 123, 2026])('lets an untouched default opening collect food for five seconds without an AI ambush (seed %i)', (seed) => {
    const state = rules.createGame({ seed })
    expect(state.snakes).toHaveLength(11)
    // Keep every real opponent and all food. Only the ordinary two-second
    // spawn protection applies; AI steering rather than collision exemptions
    // must keep the initial snack route approachable.
    for (let frame = 0; frame < 150; frame += 1) rules.step(state, {}, 1 / 30)
    expect(state.snakes[0].alive).toBe(true)
    expect(state.phase).toBe('playing')
    expect(state.time).toBeCloseTo(5)
    expect(state.snakes[0].invulnerable).toBe(0)
    expect(state.snakes[0].score).toBeGreaterThan(0)
  })

  it('suppresses AI boosting during the warmup and restores ordinary competitive movement after twenty seconds', () => {
    const state = solo()
    position(state.snakes[0], 500, 500, 0)
    const bot = position(rules.createGame({ seed: 43 }).snakes[1], 1500, 500, 0)
    bot.aiTimer = 0
    state.snakes.push(bot); state.food = [food(1550, 500, 3)]
    rules.step(state)
    expect(bot.boosting).toBe(false)
    expect(bot.energy).toBe(100)
    state.time = 20
    bot.aiTimer = 0
    rules.step(state)
    expect(bot.boosting).toBe(true)
    expect(bot.energy).toBeLessThan(100)
  })

  it('caps food and elimination scores at the host/share limit while preserving growth and kill credit', () => {
    const eating = solo(), eater = eating.snakes[0]
    eater.score = 1e9 - 5
    eating.food = [food(eater.x, eater.y, 3)]
    rules.step(eating)
    expect(eater.score).toBe(1e9)
    expect(eater.mass).toBe(63)
    eating.food = [food(eater.x, eater.y, 3)]
    rules.step(eating)
    expect(eater.score).toBe(1e9)
    expect(eater.mass).toBe(66)

    const hunting = solo(), player = position(hunting.snakes[0], 500, 500, 0)
    player.score = 1e9 - 10
    player.trail = [{ x: 500, y: 550 }, { x: 500, y: 620 }]
    const bot = position(rules.createGame({ seed: 44 }).snakes[1], 500, 580, -Math.PI / 2)
    bot.mass = 800
    hunting.snakes.push(bot)
    rules.step(hunting)
    expect(player.score).toBe(1e9)
    expect(Number.isSafeInteger(player.score)).toBe(true)
    expect(player.kills).toBe(1)
    expect(bot.alive).toBe(false)
  })
})
