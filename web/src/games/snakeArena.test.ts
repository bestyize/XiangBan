import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

interface Point { x: number; y: number }
interface Snake extends Point {
  id: number; mass: number; score: number; kills: number; alive: boolean
  angle: number; targetAngle: number; energy: number; boosting: boolean; boostLocked: boolean
  invulnerable: number; aiTimer: number; respawnAt: number; trail: Point[]; previous: Point
}
interface Food extends Point { id: number; value: number; remains: boolean; color: number; phase: number }
interface GameEvent { type: string; id?: number; killerId?: number | null }
interface State {
  snakes: Snake[]; food: Food[]; phase: string; time: number; remaining: number | null; roundType: string
  reason: string | null; events: GameEvent[]
}
interface Rules {
  WORLD: { width: number; height: number }; MIN_MASS: number
  createGame(options?: { seed?: number; mode?: string; roundType?: string }): State
  step(state: State, input?: { angle?: number | null; boost?: boolean }, dt?: number): GameEvent[]
  bodyPoints(snake: Snake): Point[]
  segmentDistanceSq(point: Point, a: Point, b: Point): number
  segmentsDistanceSq(a: Point, b: Point, c: Point, d: Point): number
  rankings(state: State): Snake[]
}
const context: { XiabanSnakeArena?: Rules } = {}
runInNewContext(readFileSync(new URL('../../public/game-packs/snake-arena/rules.js', import.meta.url), 'utf8'), context)
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
  it('creates reproducible independent local AI arenas with the selected opponent count', () => {
    const a = rules.createGame({ seed: 7 })
    const b = rules.createGame({ seed: 7 })
    expect(a.snakes).toEqual(b.snakes)
    expect(a.food).toEqual(b.food)
    expect(a.snakes).toHaveLength(9)
    const lively = rules.createGame({ mode: 'lively', seed: 7 })
    expect(lively.snakes).toHaveLength(12)
    expect(a.snakes[0].invulnerable).toBe(5)
    expect(lively.snakes[0].invulnerable).toBe(3)
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
    expect(Math.abs(player.angle - before)).toBeLessThanOrEqual(3.1 / 60 + 1e-9)
    expect(Math.hypot(player.x - player.previous.x, player.y - player.previous.y)).toBeCloseTo(118 / 60)
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
    expect(Math.hypot(player.x - player.previous.x, player.y - player.previous.y)).toBeCloseTo(218 / 30)
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
    expect(other.score).toBe(80)
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
  })

  it('keeps AI respawns available after the timed-round boundary in endless survival', () => {
    const state = rules.createGame({ seed: 12, roundType: 'endless', mode: 'lively' })
    expect(state.snakes).toHaveLength(12)
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
})
