import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
interface Input { move?: number; jump?: boolean; jumpPressed?: boolean; slashPressed?: boolean; dashPressed?: boolean; inkPressed?: boolean }
interface Rules {
  GROUND: number; DIFFICULTIES: Any; STAGES: Any[]
  createGame(options?: { difficulty?: string; start?: string; level?: string; seed?: number; seenPrompts?: string[] }): Any
  step(state: Any, input?: Input, dt?: number): Any[]
  result(state: Any): Any
  evaluate(state: Any): { seals: number; grade: number }
  challenge(state: Any): { value: number; goal: number; met: boolean }
  spawnEnemy(state: Any, spec: Any): Any
}
interface Levels { LEVELS: Any[]; level(id: string): Any; volume(id: number): Any; levelByCode(code: number): Any }
const context: { XiabanInkBlade?: Rules; XiabanInkLevels?: Levels } = {}
for (const file of ['levels.js', 'rules.js']) runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`, import.meta.url), 'utf8'), context)
const rules = context.XiabanInkBlade!
const levels = context.XiabanInkLevels!
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
  it('starts the whole scroll by default or a single 幅, with difficulty-specific health', () => {
    const easy = rules.createGame({ difficulty: 'easy' })
    const normal = rules.createGame({ difficulty: 'anything' })
    expect([easy.player.hp, normal.player.hp]).toEqual([7, 5])
    expect(normal.difficulty).toBe('normal')
    expect(normal.level.id).toBe('1-scroll')
    expect(normal.plan.map((encounter: Any) => encounter.id)).toEqual(['gate', 'grove', 'crows', 'bridge', 'rain', 'boss'])
    const boss = rules.createGame({ start: 'boss' })
    expect(boss.level.id).toBe('1-6')
    expect(boss.player.x).toBe(levels.level('1-6').start)
    expect(boss.plan.map((encounter: Any) => encounter.id)).toEqual(['boss'])
    expect(rules.createGame({ level: 'missing' }).level.id).toBe('1-scroll')
  })

  it('keeps every 幅 inside its window of the scroll with its encounters inside that window', () => {
    for (const level of levels.LEVELS) {
      const world = levels.volume(level.volume)
      expect(level.start).toBeGreaterThan(level.range[0])
      expect(level.start).toBeLessThan(level.range[1])
      for (const id of level.encounters) {
        const encounter = world.encounters[id]
        expect(encounter.lock[0] + 40).toBeGreaterThanOrEqual(level.range[0])
        expect(encounter.lock[1] - 40).toBeLessThanOrEqual(level.range[1])
        expect(encounter.trigger).toBeGreaterThanOrEqual(level.start)
      }
      expect(level.grades).toEqual([...level.grades].sort((a: number, b: number) => a - b))
      expect(levels.levelByCode(level.code)).toBe(level)
    }
    const state = rules.createGame({ level: '1-3', seed: 2 })
    run(state, 60 * 3, { move: -1 })
    expect(state.player.x).toBeGreaterThanOrEqual(levels.level('1-3').range[0] + 16 - 0.01)
  })

  it('skips tutorial prompts the player has already seen', () => {
    const fresh = run(rules.createGame({ level: '1-1' }), 2)
    expect(fresh.some((e) => e.type === 'prompt' && e.key === 'move')).toBe(true)
    const seen = run(rules.createGame({ level: '1-1', seenPrompts: ['move'] }), 2)
    expect(seen.some((e) => e.type === 'prompt' && e.key === 'move')).toBe(false)
  })

  it('moves, jumps with a double jump, and locks the gate encounter until every dummy falls', () => {
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
    expect(state.player.kills).toBe(5)
    expect(state.encounter).toBe(1)
    expect(state.phase).toBe('playing')
  })

  it('ends a single 幅 when its last encounter is cleared and awards seals and a grade', () => {
    const state = rules.createGame({ level: '1-1', seed: 1 })
    for (let i = 0; i < 60 * 30 && state.phase === 'playing'; i++) {
      const target = state.enemies.find((e: Any) => e.alive && e.delay <= 0)
      const move = target ? (Math.abs(target.x - state.player.x) > 40 ? Math.sign(target.x - state.player.x) : 0) : 1
      rules.step(state, { move, slashPressed: Boolean(target) && i % 10 === 0 }, DT)
    }
    expect(state.reason).toBe('clear')
    const result = rules.result(state)
    expect(result).toMatchObject({ level: '1-1', code: 101, cleared: true, kills: 5 })
    expect(result.seals & 3).toBe(3)
    expect(result.grade).toBeGreaterThanOrEqual(1)
    expect(state.bonus.flawless).toBe(600)
  })

  it('awards each seal only for its own condition and grades by score', () => {
    const state = rules.createGame({ level: '1-2' })
    expect(rules.evaluate(state)).toEqual({ seals: 0, grade: 0 })
    state.reason = 'clear'
    state.player.damageTaken = 2
    state.player.score = 0
    expect(rules.evaluate(state)).toEqual({ seals: 1, grade: 1 })
    state.player.damageTaken = 1
    state.player.executions = 3
    state.player.score = levels.level('1-2').grades[1]
    expect(rules.evaluate(state)).toEqual({ seals: 7, grade: 3 })
    expect(rules.challenge(state)).toMatchObject({ value: 3, goal: 3, met: true })
    const scroll = rules.createGame({ level: '1-scroll' })
    scroll.reason = 'clear'; scroll.time = 400
    expect(rules.evaluate(scroll).seals & 4).toBe(0)
    scroll.time = 300
    expect(rules.evaluate(scroll).seals & 4).toBe(4)
  })

  it('counts air kills, arrow kills and boss executions for challenges', () => {
    const state = arena()
    const crow = place(state, 'crow', 1440)
    crow.y = state.player.y - 40
    state.player.onGround = false; state.player.y -= 40; state.player.vy = -200
    run(state, 14, (i) => ({ slashPressed: i === 0 }))
    expect(crow.alive).toBe(false)
    expect(state.player.airKills).toBe(1)
    const shooter = arena()
    const archer = place(shooter, 'archer', 1700, { facing: -1 })
    shooter.projectiles.push({ id: 900, type: 'arrow', owner: 'enemy', x: 1450, y: shooter.player.y - 30, vx: -470, vy: 0, life: 2, source: archer.id, dead: false })
    run(shooter, 1, { slashPressed: true })
    run(shooter, 60)
    expect(archer.alive).toBe(false)
    expect(shooter.player.deflectKills).toBe(1)
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

  it('makes the player slide on ice zones in Volume 2', () => {
    const groundState = rules.createGame({ level: '2-1', seed: 1 })
    groundState.player.x = 520
    run(groundState, 20, { move: 1 })
    const groundSpeed = groundState.player.vx
    run(groundState, 15, { move: 0 })
    expect(groundState.player.vx).toBeLessThan(groundSpeed * 0.1)

    const iceState = rules.createGame({ level: '2-3', seed: 1 })
    iceState.player.x = 2500
    run(iceState, 20, { move: 1 })
    run(iceState, 15, { move: 0 })
    expect(iceState.player.vx).toBeGreaterThan(groundState.player.vx * 5)
  })

  it('tests Volume 2 enemies: spearman thrust vs sweep', () => {
    const s = rules.createGame({ level: '2-2', seed: 1 })
    s.lock = [1000, 2000]; s.encounter = 99; s.player.x = 1400
    const spear = place(s, 'spearman', 1480, { facing: -1 })
    spear.state = 'tele'; spear.move = 'thrust'; spear.t = 0.01; spear.teleTotal = 0.5
    rules.step(s, { slashPressed: true }, DT)
    const thrustEvents = run(s, 10)
    expect(thrustEvents.some((e) => e.type === 'hurt')).toBe(true)

    s.player.iframes = 0; s.player.hurt = 0; s.player.hp = 5
    spear.state = 'tele'; spear.move = 'sweep'; spear.t = 0.005; spear.teleTotal = 0.5
    rules.step(s, { slashPressed: true }, DT)
    const sweepEvents = run(s, 10)
    expect(sweepEvents.some((e) => e.type === 'parry')).toBe(true)
    expect(spear.state).toBe('stagger')
  })

  it('clears Volume 2 boss angler with scripted parries and executions', () => {
    const s = rules.createGame({ level: '2-6', difficulty: 'easy', seed: 7 })
    for (let i = 0; i < 60 * 200 && s.phase === 'playing'; i++) {
      const p = s.player
      const b = s.enemies.find((e: Any) => e.type === 'angler' && e.alive)
      const move = b ? (Math.abs(b.x - p.x) > 60 ? Math.sign(b.x - p.x) : 0) : 1
      rules.step(s, { move, slashPressed: Boolean(b) && i % 9 === 0, inkPressed: p.ink >= 100 }, DT)
      p.hp = Math.max(p.hp, 1)
    }
    expect(s.phase).toBe('over')
    expect(s.reason).toBe('clear')
    const r = rules.result(s)
    expect(r.level).toBe('2-6')
    expect(r.cleared).toBe(true)
    expect(r.kills).toBe(1)
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
