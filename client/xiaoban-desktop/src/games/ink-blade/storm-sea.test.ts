import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
const context: Record<string, Any> = {}
for (const file of ['levels.js', 'rules.js', 'progress.js']) runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`, import.meta.url), 'utf8'), context)
const { XiabanInkLevels: levels, XiabanInkBlade: rules, XiabanInkProgress: progress } = context
const DT = 1 / 60

function arena(level: string, x: number) {
  const state = rules.createGame({ level, seed: 7 })
  const encounter = state.plan[0]
  state.lock = [encounter.lock[0] + 40, encounter.lock[1] - 40]
  state.encounter = 99
  state.player.x = x
  return state
}

function place(state: Any, type: string, x: number, extra: Any = {}) {
  const enemy = rules.spawnEnemy(state, { type, x })
  Object.assign(enemy, { state: 'idle', t: 0, cd: 99 }, extra)
  return enemy
}

function run(state: Any, count: number, input: (i: number) => Record<string, unknown> = () => ({})) {
  const events: Any[] = []
  for (let i = 0; i < count; i++) events.push(...rules.step(state, input(i), DT))
  return events
}

describe('怒涛卷', () => {
  it('has six unlockable 幅, one full scroll and a durable save', () => {
    const stages = levels.volumeLevels(4)
    expect(stages.map((stage: Any) => stage.code)).toEqual([401, 402, 403, 404, 405, 406, 400])
    expect(stages.every((stage: Any) => stage.encounters.every((id: string) => levels.volume(4).encounters[id]))).toBe(true)
    let save = progress.empty()
    expect(progress.unlocked(save, '4-1')).toBe(true)
    expect(progress.unlocked(save, '4-2')).toBe(false)
    save = progress.record(save, { level: '4-1', cleared: true, score: 2100, grade: 2, seals: 3, seconds: 55 }).save
    expect(progress.unlocked(progress.normalize(save), '4-2')).toBe(true)
    expect(progress.unlocked(save, '4-scroll')).toBe(false)
  })

  it('pulls a parried hook pirate into execution range', () => {
    const state = arena('4-1', 1050)
    state.player.facing = 1
    const pirate = place(state, 'hooker', 1124, { facing: -1, state: 'tele', t: 0.01, teleTotal: 0.62 })
    rules.step(state, { slashPressed: true }, DT)
    const events = run(state, 10)
    expect(events.some((event: Any) => event.type === 'hookPull')).toBe(true)
    expect(pirate.state).toBe('stagger')
    expect(pirate.x - state.player.x).toBeLessThan(65)
    expect(state.player.hookPulls).toBe(1)
    run(state, 28)
    expect(run(state, 20, (i) => ({ slashPressed: i === 0 })).some((event: Any) => event.type === 'execute')).toBe(true)
  })

  it('warns before a tide surge, which can be jumped and takes ground enemies', () => {
    const standing = arena('4-3', 2700)
    const pirate = place(standing, 'pirate', 2800)
    standing.tide.zone = standing.world.tide.zones[0]
    standing.tide.phase = 'warn'; standing.tide.t = 0.18
    expect(run(standing, 15).some((event: Any) => event.type === 'tideSurge')).toBe(true)
    expect(standing.player.hp).toBe(standing.player.maxHp) // The visible wave has not reached mid-arena yet.
    run(standing, 70)
    expect(standing.player.hp).toBe(standing.player.maxHp - 1)
    expect(pirate.alive).toBe(false)
    expect(standing.player.tideKills).toBe(1)

    const jumping = arena('4-3', 2700)
    jumping.tide.zone = jumping.world.tide.zones[0]
    jumping.tide.phase = 'warn'; jumping.tide.t = 0.18
    run(jumping, 47, (i) => ({ jumpPressed: i === 20, jump: i >= 20 }))
    expect(jumping.player.hp).toBe(jumping.player.maxHp)
  })

  it('makes the dragon telegraph a head strike and an unblockable sea wave', () => {
    const state = arena('4-6', 5700)
    const dragon = place(state, 'dragon', 5820, { facing: -1, state: 'idle', t: 0, moves: 0 })
    expect(run(state, 1).some((event: Any) => event.type === 'tell')).toBe(true)
    expect(dragon.state).toBe('dragonTell')
    dragon.state = 'dragonWaveTell'; dragon.t = 0.01
    expect(run(state, 2).some((event: Any) => event.type === 'seaWave')).toBe(true)
    expect(state.projectiles.some((shot: Any) => shot.type === 'seaWave')).toBe(true)
  })

  it('lets a fresh slash parry the dragon head, while a sea wave passes through a slash', () => {
    const head = arena('4-6', 5700)
    head.player.facing = 1
    const dragon = place(head, 'dragon', 5790, { facing: -1, state: 'dragonTell', t: 0.01, teleTotal: 0.66 })
    rules.step(head, { slashPressed: true }, DT)
    expect(run(head, 8).some((event: Any) => event.type === 'parry')).toBe(true)
    expect(dragon.posture).toBeGreaterThan(0)

    const wave = arena('4-6', 5700)
    wave.player.facing = 1
    wave.projectiles.push({ id: 999, type: 'seaWave', owner: 'enemy', x: 5730, y: rules.GROUND, vx: -360, vy: 0, life: 1, dead: false })
    const events = run(wave, 8, (i) => ({ slashPressed: i === 0 }))
    expect(events.some((event: Any) => event.type === 'deflect')).toBe(false)
    expect(wave.player.hp).toBe(wave.player.maxHp - 1)
  })

  it('drops a cut spar onto enemies once, without damaging the player', () => {
    const state = arena('4-2', 1530)
    const enemy = place(state, 'hooker', 1700)
    const hp = state.player.hp
    const events = run(state, 80, (i) => ({ slashPressed: i === 0 }))
    expect(events.some((e: Any) => e.type === 'rigCut')).toBe(true)
    expect(events.filter((e: Any) => e.type === 'rigSlam')).toHaveLength(1)
    expect(enemy.alive).toBe(false)
    expect(state.player.riggingKills).toBe(1)
    expect(state.player.hp).toBe(hp)
    expect(run(state, 90, (i) => ({ slashPressed: i % 30 === 0 })).some((e: Any) => e.type === 'rigSlam')).toBe(false)
  })

  it('gives the dual-blade pirate a readable second cut, canceled by parrying the first', () => {
    const state = arena('4-1', 1050)
    const enemy = place(state, 'pirate', 1120, { facing: -1, state: 'attack', t: 0.01, hitDone: true, chain: 0 })
    run(state, 1)
    expect(enemy.state).toBe('tele')
    expect(enemy.teleTotal).toBeGreaterThanOrEqual(0.45)
    expect(enemy.chain).toBe(1)
    enemy.t = 0.01
    run(state, 20, (i) => ({ slashPressed: i === 0 }))
    expect(enemy.state).toBe('stagger')
    expect(state.player.parries).toBe(1)
  })

  it('blocks frontal mash on the hook guard but allows a slash behind it', () => {
    const front = arena('4-1', 1050)
    const guard = place(front, 'hooker', 1100, { facing: -1, state: 'tele', t: 4, teleTotal: 4 })
    run(front, 16, (i) => ({ slashPressed: i === 0 }))
    expect(guard.hp).toBe(guard.maxHp)
    const rear = arena('4-1', 1050)
    const exposed = place(rear, 'hooker', 1100, { facing: 1, state: 'tele', t: 4, teleTotal: 4 })
    run(rear, 16, (i) => ({ slashPressed: i === 0 }))
    expect(exposed.hp).toBeLessThan(exposed.maxHp)
  })

  it('locks the dragon emergence target before impact and leaves a punish window', () => {
    const state = arena('4-6', 5660)
    const dragon = place(state, 'dragon', 5780, { state: 'dragonDive', t: 0.01 })
    run(state, 2)
    expect(dragon.state).toBe('dragonDiveTell')
    const target = dragon.diveX
    const hp = state.player.hp
    run(state, 85, () => ({ move: -1 }))
    expect(dragon.diveX).toBe(target)
    expect(dragon.state).toBe('recover')
    expect(dragon.t).toBeGreaterThan(0.8)
    expect(state.player.hp).toBe(hp)
  })

  it('makes the emergence unblockable but lets a high jump clear it', () => {
    for (const airborne of [false, true]) {
      const state = arena('4-6', 5700)
      place(state, 'dragon', 5700, { state: 'dragonEmerge', t: 0.2, hitDone: false })
      if (airborne) { state.player.y = rules.GROUND - 120; state.player.onGround = false }
      const events = run(state, 2, (i) => ({ slashPressed: i === 0 }))
      expect(events.some((e: Any) => e.type === 'parry')).toBe(false)
      expect(state.player.hp).toBe(state.player.maxHp - (airborne ? 0 : 1))
    }
  })

  it('can finish the dragon fight with timed parries and executions', () => {
    const state = rules.createGame({ level: '4-6', difficulty: 'easy', seed: 9 })
    for (let i = 0, last = -99; i < 60 * 220 && state.phase === 'playing'; i++) {
      const player = state.player
      const dragon = state.enemies.find((enemy: Any) => enemy.type === 'dragon' && enemy.alive)
      const distance = dragon ? Math.abs(dragon.x - player.x) : 999
      const fresh = i - last > 25
      const input: Record<string, unknown> = { move: !dragon ? 1 : distance > 65 ? Math.sign(dragon.x - player.x) : 0 }
      if (dragon?.state === 'dragonDiveTell') input.move = player.x <= dragon.diveX ? -1 : 1
      if (dragon?.state === 'dragonWaveTell' && dragon.t < 0.14 && player.onGround) { input.jumpPressed = true; input.jump = true }
      else if (dragon?.state === 'dragonTell' && dragon.t < 0.045 && distance < 126 && fresh) { input.slashPressed = true; last = i }
      else if (dragon && ['broken', 'recover'].includes(dragon.state) && distance < 85 && fresh) { input.slashPressed = true; last = i }
      if (player.ink >= 100) input.inkPressed = true
      rules.step(state, input, DT)
    }
    expect(state.reason, JSON.stringify({ boss: state.enemies.find((enemy: Any) => enemy.type === 'dragon'), player: { x: state.player.x, score: state.player.score, parries: state.player.parries, executions: state.player.executions }, time: state.time })).toBe('clear')
    expect(rules.result(state)).toMatchObject({ code: 406, cleared: true, kills: 1 })
  })
})
