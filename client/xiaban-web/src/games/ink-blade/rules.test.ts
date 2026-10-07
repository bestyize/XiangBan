import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
interface Input { move?: number; jump?: boolean; jumpPressed?: boolean; slashPressed?: boolean; dashPressed?: boolean; inkPressed?: boolean }
interface Rules {
  GROUND: number; START_X: number; BOSS_START: number; ENCOUNTERS: Any[]; DIFFICULTIES: Any; STAGES: Any[]
  createGame(options?: { difficulty?: string; start?: string; seed?: number }): Any
  step(state: Any, input?: Input, dt?: number): Any[]
  result(state: Any): Any
  spawnEnemy(state: Any, spec: Any): Any
}
const context: { XiabanInkBlade?: Rules } = {}
runInNewContext(readFileSync(new URL('../../../public/game-packs/ink-blade/rules.js', import.meta.url), 'utf8'), context)
const rules = context.XiabanInkBlade!
const DT = 1 / 60

function run(state: Any, frames: number, input: Input | ((i: number) => Input) = {}) {
  const events: Any[] = []
  for (let i = 0; i < frames; i++) events.push(...rules.step(state, typeof input === 'function' ? input(i) : input, DT))
  return events
}

// An arena with the player alone, ready for a hand-placed enemy.
function arena(difficulty = 'normal') {
  const state = rules.createGame({ difficulty, seed: 3 })
  state.lock = [1000, 1800]
  state.encounter = 99
  state.player.x = 1400
  return state
}

function place(state: Any, type: string, x: number, extra: Any = {}) {
  const enemy = rules.spawnEnemy(state, { type, x })
  Object.assign(enemy, { state: 'idle', t: 0, cd: 99 }, extra)
  return enemy
}

describe('ink blade rules', () => {
  it('starts on the scroll or directly at the boss with difficulty-specific health', () => {
    const easy = rules.createGame({ difficulty: 'easy' })
    const normal = rules.createGame({ difficulty: 'anything' })
    expect([easy.player.hp, normal.player.hp]).toEqual([7, 5])
    expect(normal.difficulty).toBe('normal')
    expect(normal.player.x).toBe(rules.START_X)
    const boss = rules.createGame({ start: 'boss' })
    expect(boss.player.x).toBe(rules.BOSS_START)
    expect(boss.encounter).toBe(rules.ENCOUNTERS.length - 1)
  })

  it('moves, jumps with a double jump, and locks the gate encounter until both dummies fall', () => {
    const state = rules.createGame({ seed: 1 })
    const jumps = run(state, 2, (i) => ({ move: 1, jump: true, jumpPressed: i === 0 }))
    expect(jumps.some((e) => e.type === 'jump' && !e.double)).toBe(true)
    run(state, 20, { move: 1, jump: true })
    const second = run(state, 1, { move: 1, jump: true, jumpPressed: true })
    expect(second.some((e) => e.type === 'jump' && e.double)).toBe(true)
    run(state, 60 * 4, { move: 1 })
    expect(state.lock).not.toBeNull()
    expect(state.enemies.filter((e: Any) => e.type === 'dummy')).toHaveLength(2)
    expect(state.player.x).toBeLessThanOrEqual(state.lock[1])
    for (let i = 0; i < 60 * 12 && state.lock; i++) {
      const target = state.enemies.find((e: Any) => e.alive && e.delay <= 0)
      const move = target ? (Math.abs(target.x - state.player.x) > 40 ? Math.sign(target.x - state.player.x) : 0) : 1
      rules.step(state, { move, slashPressed: Boolean(target) && i % 10 === 0 }, DT)
    }
    expect(state.lock).toBeNull()
    expect(state.player.kills).toBe(2)
    expect(state.encounter).toBe(1)
  })

  it('chains three slash stages and resets after a pause', () => {
    const state = arena()
    const stages: number[] = []
    for (let i = 0; i < 90; i++) for (const e of rules.step(state, { slashPressed: i % 8 === 0 }, DT)) if (e.type === 'slash') stages.push(e.stage)
    expect(stages.slice(0, 3)).toEqual([0, 1, 2])
    run(state, 70)
    const next = run(state, 2, { slashPressed: true })
    expect(next.find((e) => e.type === 'slash')?.stage).toBe(0)
  })

  it('parries a telegraphed strike, staggers the puppet and executes it', () => {
    const state = arena()
    const puppet = place(state, 'puppet', 1460, { facing: -1 })
    puppet.state = 'tele'; puppet.t = 0.05; puppet.teleTotal = 0.5
    const events = run(state, 1, { slashPressed: true })
    events.push(...run(state, 20))
    expect(events.some((e) => e.type === 'parry')).toBe(true)
    expect(state.player.hp).toBe(5)
    expect(puppet.state).toBe('stagger')
    state.hitstop = 0; state.slow = 0
    run(state, 30)
    const exec = run(state, 20, (i) => ({ slashPressed: i === 0 }))
    expect(exec.some((e) => e.type === 'execute')).toBe(true)
    expect(puppet.alive).toBe(false)
    expect(state.player.executions).toBe(1)
  })

  it('takes damage without a parry and grants invulnerability afterward', () => {
    const state = arena()
    const puppet = place(state, 'puppet', 1460, { facing: -1 })
    puppet.state = 'tele'; puppet.t = 0.01; puppet.teleTotal = 0.5
    const events = run(state, 12)
    expect(events.some((e) => e.type === 'hurt')).toBe(true)
    expect(state.player.hp).toBe(4)
    expect(state.player.iframes).toBeGreaterThan(0.5)
    expect(state.player.combo).toBe(0)
  })

  it('uses the wider parry window on the easy difficulty', () => {
    expect(rules.DIFFICULTIES.easy.parry).toBeGreaterThan(rules.DIFFICULTIES.normal.parry)
    for (const [difficulty, expected] of [['easy', 'parry'], ['normal', 'hurt']] as const) {
      const state = arena(difficulty)
      rules.step(state, { slashPressed: true }, DT)
      while (state.clock - state.player.slashStartClock < 0.2) rules.step(state, {}, DT)
      // The blade arrives 0.2s after the swing began: inside the easy window only.
      place(state, 'puppet', state.player.x + 60, { facing: -1, state: 'attack', t: 0.12, hitDone: false })
      const events = run(state, 2)
      expect(events.some((e) => e.type === expected)).toBe(true)
    }
  })

  it('deflects arrows back into the archer', () => {
    const state = arena()
    const archer = place(state, 'archer', 1700, { facing: -1 })
    state.projectiles.push({ id: 900, type: 'arrow', owner: 'enemy', x: 1450, y: state.player.y - 30, vx: -470, vy: 0, life: 2, source: archer.id, dead: false })
    const events = run(state, 1, { slashPressed: true })
    events.push(...run(state, 60))
    expect(events.some((e) => e.type === 'deflect')).toBe(true)
    expect(archer.alive).toBe(false)
    expect(state.player.parries).toBe(1)
  })

  it('blocks frontal hits with the shield but not hits from behind', () => {
    const state = arena()
    const shield = place(state, 'shield', 1450, { facing: -1 })
    const front = run(state, 14, (i) => ({ slashPressed: i === 0 }))
    expect(front.some((e) => e.type === 'block')).toBe(true)
    expect(shield.hp).toBe(5)
    state.player.x = 1500; state.player.facing = -1; shield.facing = -1; shield.state = 'idle'; shield.turn = 0
    const back = run(state, 14, (i) => ({ slashPressed: i === 0 }))
    expect(back.some((e) => e.type === 'hit')).toBe(true)
    expect(shield.hp).toBeLessThan(5)
  })

  it('breaks the boss posture with parries and opens a heavy execution', () => {
    const state = arena()
    const boss = place(state, 'boss', 1470, { facing: -1, state: 'idle', t: 99 })
    for (let k = 0; k < 5 && boss.state !== 'broken'; k++) {
      boss.state = 'combo'; boss.combo = { i: 0, phase: 'active' }; boss.t = 0.13; boss.hitDone = false
      boss.x = 1470; state.player.x = 1400; state.player.iframes = 0; state.player.action = null; state.player.slashEndClock = -9
      rules.step(state, { slashPressed: true }, DT)
      run(state, 2)
      state.hitstop = 0
    }
    expect(boss.state).toBe('broken')
    const hp = boss.hp
    state.hitstop = 0; state.slow = 0
    run(state, 30)
    const events = run(state, 20, (i) => ({ slashPressed: i === 0 }))
    expect(events.some((e) => e.type === 'execute' && e.boss)).toBe(true)
    expect(hp - boss.hp).toBeGreaterThanOrEqual(10)
  })

  it('cannot parry the phantom dash, but a dash slips through it', () => {
    const state = arena()
    const boss = place(state, 'boss', 1700, { facing: -1 })
    boss.state = 'phantom'; boss.t = 1.2; boss.vx = -1150; boss.hitDone = false
    rules.step(state, { slashPressed: true }, DT)
    run(state, 30)
    expect(state.player.hp).toBe(4)
    const dodge = arena()
    const second = place(dodge, 'boss', 1700, { facing: -1 })
    second.state = 'phantom'; second.t = 1.2; second.vx = -1150; second.hitDone = false
    run(dodge, 12)
    rules.step(dodge, { dashPressed: true, move: -1 }, DT)
    run(dodge, 30)
    expect(dodge.player.hp).toBe(5)
  })

  it('clears the boss scroll with a scripted swordsman and reports finished metrics', () => {
    const state = rules.createGame({ start: 'boss', difficulty: 'easy', seed: 11 })
    for (let i = 0; i < 60 * 240 && state.phase === 'playing'; i++) {
      const p = state.player
      const boss = state.enemies.find((e: Any) => e.type === 'boss' && e.alive)
      const move = boss ? (Math.abs(boss.x - p.x) > 60 ? Math.sign(boss.x - p.x) : 0) : 1
      rules.step(state, { move, slashPressed: Boolean(boss) && i % 9 === 0, inkPressed: p.ink >= 100 }, DT)
      p.hp = Math.max(p.hp, 1)
    }
    expect(state.phase).toBe('over')
    expect(state.reason).toBe('clear')
    const result = rules.result(state)
    expect(result.cleared).toBe(true)
    expect(result.kills).toBe(1)
    expect(result.score).toBeGreaterThan(2000)
    expect(state.bonus.hp).toBe(state.player.hp * 150)
  })

  it('ends the round when the swordsman falls', () => {
    const state = arena()
    state.player.hp = 1
    const puppet = place(state, 'puppet', 1460, { facing: -1 })
    puppet.state = 'tele'; puppet.t = 0.01; puppet.teleTotal = 0.5
    const events = run(state, 60 * 4)
    expect(events.some((e) => e.type === 'death')).toBe(true)
    expect(state.phase).toBe('over')
    expect(state.reason).toBe('death')
    expect(rules.result(state).cleared).toBe(false)
  })

  it('is deterministic for the same seed and inputs', () => {
    const play = () => {
      const state = rules.createGame({ seed: 5 })
      run(state, 60 * 30, (i) => ({ move: i % 300 < 260 ? 1 : -1, slashPressed: i % 11 === 0, jumpPressed: i % 97 === 0, jump: i % 97 < 12, dashPressed: i % 173 === 0 }))
      return JSON.stringify(rules.result(state)) + state.player.x.toFixed(3)
    }
    expect(play()).toBe(play())
  })
})
