import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
interface Input { move?: number; jump?: boolean; jumpPressed?: boolean; slashPressed?: boolean; dashPressed?: boolean; inkPressed?: boolean }
const context: Record<string, Any> = {}
for (const file of ['levels.js', 'rules.js', 'progress.js']) runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`, import.meta.url), 'utf8'), context)
const rules = context.XiabanInkBlade
const levels = context.XiabanInkLevels
const progress = context.XiabanInkProgress
const DT = 1 / 60
const G = rules.GROUND

function run(state: Any, frames: number, input: Input | ((i: number) => Input) = {}) {
  const events: Any[] = []
  for (let i = 0; i < frames; i++) events.push(...rules.step(state, typeof input === 'function' ? input(i) : input, DT))
  return events
}

// A 幅 of 烽火卷 with its arena locked by hand and no encounter running.
function arena(level: string, lock: [number, number], x: number, difficulty = 'normal') {
  const state = rules.createGame({ level, seed: 3, difficulty })
  state.lock = lock
  state.encounter = 99
  state.player.x = x
  return state
}

function place(state: Any, type: string, x: number, extra: Any = {}) {
  const enemy = rules.spawnEnemy(state, { type, x, ...(extra.spec || {}) })
  delete extra.spec
  Object.assign(enemy, { state: 'idle', t: 0, cd: 99 }, extra)
  return enemy
}

// Wait until the next swing would start a fresh combo, so it can parry under 烽火卷's strict rule.
function settle(state: Any) {
  run(state, 30)
  state.hitstop = 0; state.slow = 0
}

describe('烽火卷 · arrow rain', () => {
  it('warns with a shadow, then hits anyone in the open but not behind a wall', () => {
    const open = arena('3-1', [740, 1400], 1100)
    const tell = rules.callVolley(open, 1100, 280, 1.2)
    expect(tell.tele).toBeCloseTo(1.2)
    const events = run(open, 60 * 1.3)
    expect(events.some((e) => e.type === 'volleyHit')).toBe(true)
    expect(open.player.hp).toBe(open.player.maxHp - 1)

    const wall = levels.volume(3).ramparts[0]
    const cover = arena('3-1', [740, 1400], wall.x - 20)
    expect(rules.sheltered(cover, cover.player.x)).toBe(true)
    rules.callVolley(cover, cover.player.x, 280, 1.2)
    run(cover, 60 * 1.3)
    expect(cover.player.hp).toBe(cover.player.maxHp)
  })

  it('counts soldiers felled by the volley, and starts the beacon volleys only once the fight is locked', () => {
    const state = arena('3-1', [740, 1400], 1100)
    const soldier = place(state, 'soldier', 1160)
    rules.callVolley(state, 1100, 280, 1.2)
    run(state, 60 * 1.3, { move: -1 })
    expect(soldier.alive).toBe(false)
    expect(state.player.volleyKills).toBe(1)

    const walk = rules.createGame({ level: '3-1', seed: 1 })
    expect(run(walk, 60 * 1.2, { move: 1 }).some((e) => e.type === 'volleyTell')).toBe(false)
    const fight = arena('3-1', [740, 1400], 1100)
    expect(run(fight, 60 * 2.2).some((e) => e.type === 'volleyTell')).toBe(true)
  })
})

describe('烽火卷 · wildfire', () => {
  it('spreads downwind faster than upwind, burns out to ash and never relights', () => {
    const state = arena('3-2', [1380, 2100], 1420)
    const cells = state.grass
    const start = cells.findIndex((cell: Any) => cell.x >= 1960)
    rules.igniteAt(state, cells[start].x + 5)
    run(state, 60 * 1.5)
    const lit = (i: number) => cells[i].s !== 'dry'
    expect(lit(start + 3)).toBe(true)
    expect(lit(start - 1)).toBe(true)
    expect(lit(start - 3)).toBe(false)
    run(state, 60 * 3)
    expect(cells[start].s).toBe('ash')
    rules.igniteAt(state, cells[start].x + 5)
    expect(cells[start].s).toBe('ash')
  })

  it('hurts a player standing in flames and credits enemies the fire kills', () => {
    const state = arena('3-2', [1380, 2100], 1420)
    const cell = state.grass.find((c: Any) => c.x >= 1900)
    const soldier = place(state, 'soldier', cell.x + 12, { hp: 2, state: 'stagger', t: 99 })
    rules.igniteAt(state, cell.x + 12)
    run(state, 30)
    expect(soldier.alive).toBe(false)
    expect(state.player.fireKills).toBe(1)

    const burn = arena('3-2', [1380, 2100], 1490)
    rules.igniteAt(burn, 1490)
    run(burn, 20)
    expect(burn.player.hp).toBe(burn.player.maxHp - 1)
  })

  it('lets a fire pot be batted back at its thrower', () => {
    const state = arena('3-2', [1380, 2100], 1420)
    const bomber = place(state, 'bomber', 1700, { facing: -1, cd: 0 })
    let thrown = false
    for (let i = 0; i < 60 * 3 && !thrown; i++) { rules.step(state, {}, DT); thrown = state.projectiles.some((shot: Any) => shot.type === 'pot') }
    expect(thrown).toBe(true)
    bomber.cd = 99
    const pot = state.projectiles.find((shot: Any) => shot.type === 'pot')
    let deflected = false
    for (let i = 0; i < 90 && !deflected; i++) {
      const near = Math.abs(pot.x - state.player.x) < 26 && Math.abs(pot.y - (state.player.y - 30)) < 40
      const events = rules.step(state, { slashPressed: near }, DT)
      deflected = events.some((e: Any) => e.type === 'deflect' && e.pot)
    }
    expect(deflected).toBe(true)
    run(state, 60 * 1.5)
    expect(bomber.alive).toBe(false)
    expect(state.player.fireKills).toBe(1)
  })
})

describe('烽火卷 · cavalry and the strict parry', () => {
  it('throws a parried rider from the saddle; the fresh swing after it executes him', () => {
    const state = arena('3-3', [2440, 3100], 2700)
    state.player.facing = 1
    const rider = place(state, 'cavalry', 2900, { facing: -1, state: 'attack', t: 1.5, vx: -460, hitDone: false })
    let parried = false
    for (let i = 0; i < 60 && !parried; i++) {
      const close = rider.x - state.player.x < 75
      parried = rules.step(state, { slashPressed: close }, DT).some((e: Any) => e.type === 'parry')
    }
    expect(parried).toBe(true)
    expect(rider.type).toBe('footman')
    expect(rider.state).toBe('stagger')
    expect(state.player.unhorses).toBe(1)
    expect(state.player.hp).toBe(state.player.maxHp)
    state.hitstop = 0; state.slow = 0
    run(state, 24)
    const exec = run(state, 20, (i) => ({ slashPressed: i === 0 }))
    expect(exec.some((e) => e.type === 'execute')).toBe(true)
    expect(rider.alive).toBe(false)
  })

  it('cannot be parried by a swing chained onto a combo, but a jump clears the charge', () => {
    const chained = arena('3-3', [2440, 3100], 2700)
    chained.player.facing = 1
    rules.step(chained, { slashPressed: true }, DT)
    run(chained, 9)
    rules.step(chained, { slashPressed: true }, DT)
    expect(chained.player.action.chained).toBe(true)
    place(chained, 'cavalry', chained.player.x + 50, { facing: -1, state: 'attack', t: 1.5, vx: -460, hitDone: false })
    const events = run(chained, 6)
    expect(events.some((e) => e.type === 'parry')).toBe(false)
    expect(events.some((e) => e.type === 'hurt')).toBe(true)

    const jump = arena('3-3', [2440, 3100], 2700)
    const rider = place(jump, 'cavalry', 2960, { facing: -1, state: 'attack', t: 1.5, vx: -460, hitDone: false })
    for (let i = 0; i < 60; i++) rules.step(jump, { jumpPressed: rider.x - jump.player.x < 150 && jump.player.onGround, jump: true }, DT)
    expect(jump.player.hp).toBe(jump.player.maxHp)
  })

  it('still parries chained swings in a 卷 without a strict rule', () => {
    // 雪江卷 has neither 军中不乱刀 nor 竹林卷's 收刀, so a chained swing still parries there.
    const state = rules.createGame({ level: '2-1', seed: 3 })
    state.lock = [1000, 1800]; state.encounter = 99; state.player.x = 1400
    rules.step(state, { slashPressed: true }, DT)
    run(state, 9)
    rules.step(state, { slashPressed: true }, DT)
    const puppet = place(state, 'puppet', 1460, { facing: -1, state: 'attack', t: 0.12, hitDone: false })
    expect(run(state, 2).some((e) => e.type === 'parry')).toBe(true)
    expect(puppet.state).toBe('stagger')
  })
})

describe('烽火卷 · shield wall', () => {
  it('blocks blades from the front; parrying the leader breaks the whole line', () => {
    const state = arena('3-4', [3400, 4080], 3700)
    state.player.facing = 1
    const leader = place(state, 'pavise', 3780, { facing: -1, spec: { group: 't' } })
    const second = place(state, 'pavise', 3822, { facing: -1, spec: { group: 't' } })
    const third = place(state, 'pavise', 3864, { facing: -1, spec: { group: 't' } })
    expect(run(state, 14, (i) => ({ slashPressed: i === 0 })).some((e) => e.type === 'block')).toBe(true)
    expect(leader.hp).toBe(4)
    settle(state)
    state.player.x = 3700
    leader.state = 'tele'; leader.t = 0.01; leader.teleTotal = 0.6
    rules.step(state, { slashPressed: true }, DT)
    const events = run(state, 12)
    expect(events.some((e) => e.type === 'formationBreak')).toBe(true)
    expect([leader, second, third].every((member) => member.state === 'stagger')).toBe(true)
    expect(state.player.formationBreaks).toBe(1)
  })

  it('lets arrow rain and fire through the shields', () => {
    const state = arena('3-4', [3400, 4080], 3600)
    const pavise = place(state, 'pavise', 3700, { facing: -1 })
    rules.callVolley(state, 3700, 280, 0.5)
    run(state, 40, { move: -1 })
    expect(pavise.hp).toBe(1)
  })

  it('only the soldier nearest the player strikes from the line', () => {
    const state = arena('3-4', [3400, 4080], 3700)
    const near = place(state, 'pavise', 3770, { facing: -1, cd: 0, spec: { group: 'n' } })
    const far = place(state, 'pavise', 3812, { facing: -1, cd: 0, spec: { group: 'n' } })
    run(state, 3)
    expect(near.state).toBe('tele')
    expect(far.state).not.toBe('tele')
  })
})

describe('烽火卷 · 萧破虏 the iron-cavalry general', () => {
  it('shrugs off slashes in the saddle; parrying his charge reins him in and builds posture', () => {
    const state = arena('3-6', [5340, 6020], 5600)
    const general = place(state, 'general', 5660, { facing: -1, state: 'idle', t: 99 })
    expect(general.mounted).toBe(true)
    const hp = general.hp
    run(state, 14, (i) => ({ slashPressed: i === 0 }))
    expect(general.hp).toBe(hp)

    settle(state)
    state.player.x = 5600; state.player.facing = 1
    general.x = 5760; general.state = 'charge'; general.facing = -1; general.vx = -540; general.t = 1.4; general.hitDone = false
    let parried = false
    for (let i = 0; i < 40 && !parried; i++) parried = rules.step(state, { slashPressed: general.x - state.player.x < 80 }, DT).some((e: Any) => e.type === 'parry')
    expect(parried).toBe(true)
    expect(general.posture).toBeGreaterThanOrEqual(34)
    expect(general.state).toBe('recover')
  })

  it('is thrown from the horse when his posture breaks and fights on foot in the second phase', () => {
    const state = arena('3-6', [5340, 6020], 5600)
    const general = place(state, 'general', 5700, { facing: -1, state: 'idle', t: 99 })
    general.posture = 90
    general.state = 'charge'; general.x = 5660; general.vx = -540; general.t = 1.4; general.hitDone = false
    state.player.facing = 1
    let events: Any[] = []
    for (let i = 0; i < 40 && general.mounted; i++) events = events.concat(rules.step(state, { slashPressed: general.x - state.player.x < 80 }, DT))
    expect(general.mounted).toBe(false)
    expect(general.phase2).toBe(true)
    expect(general.state).toBe('broken')
    expect(events.some((e) => e.type === 'unhorse' && e.boss)).toBe(true)
    expect(events.some((e) => e.type === 'bossPhase')).toBe(true)
  })

  it('can be beaten by a scripted swordsman who parries his charges', () => {
    const state = rules.createGame({ level: '3-6', difficulty: 'easy', seed: 11 })
    for (let i = 0, last = -99; i < 60 * 300 && state.phase === 'playing'; i++) {
      const p = state.player
      const boss = state.enemies.find((e: Any) => e.type === 'general' && e.alive)
      const distance = boss ? Math.abs(boss.x - p.x) : 999
      const volley = state.volleys.find((v: Any) => !v.landed && Math.abs(p.x - v.x) < v.w / 2 + 12)
      const fresh = i - last > 26
      let input: Input = { move: boss ? (distance > 60 ? Math.sign(boss.x - p.x) : 0) : 1 }
      if (volley) input = { move: p.x < volley.x ? -1 : 1 }
      else if (boss?.state === 'charge' && !boss.hitDone && distance < 80 && fresh) { input = { slashPressed: true }; last = i }
      else if (boss?.state === 'combo' && boss.combo.phase === 'tele' && boss.t < 0.04 && distance < 130 && fresh) { input = { slashPressed: true }; last = i }
      else if (boss?.state === 'rushTell' && boss.t < 0.12) input = { jumpPressed: true, jump: true }
      else if (boss && (boss.state === 'broken' || boss.state === 'recover') && distance < 90 && i % 9 === 0) { input.slashPressed = true; last = i }
      if (p.ink >= 100) input.inkPressed = true
      rules.step(state, input, DT)
      p.hp = Math.max(p.hp, 1)
    }
    expect(state.reason).toBe('clear')
    expect(rules.result(state)).toMatchObject({ level: '3-6', code: 306, cleared: true, kills: 1 })
  })
})

describe('烽火卷 · levels and saves', () => {
  it('lays out six 幅 and a 长卷 with codes 301–306 and 300, each challenge stat tracked', () => {
    const volume = levels.volumeLevels(3)
    expect(volume.map((level: Any) => level.code)).toEqual([301, 302, 303, 304, 305, 306, 300])
    const state = rules.createGame({ level: '3-scroll' })
    expect(state.plan.map((encounter: Any) => encounter.id)).toEqual(['beacon', 'poplar', 'gobi', 'pass', 'raid', 'dais'])
    for (const level of volume) if (level.challenge.stat !== 'time') expect(state.player).toHaveProperty(level.challenge.stat)
    expect(levels.volume(3).copy.boss).toBe('萧破虏')
  })

  it('shortens the archers\' sight in the night raid', () => {
    const state = arena('3-5', [4340, 5020], 4400)
    const archer = place(state, 'archer', 4800, { facing: -1, cd: 0 })
    run(state, 5)
    expect(archer.state).toBe('idle')
    state.player.x = 4520
    run(state, 5)
    expect(archer.state).toBe('tele')
  })

  it('opens 烽火卷 幅 by 幅 and remembers the volume', () => {
    let save = progress.empty()
    expect(progress.unlocked(save, '3-1')).toBe(true)
    expect(progress.unlocked(save, '3-2')).toBe(false)
    save = progress.record(save, { level: '3-1', cleared: true, score: 2000, seals: 1, grade: 1, seconds: 50 }).save
    expect(progress.unlocked(save, '3-2')).toBe(true)
    expect(progress.normalize({ ...save, volume: 3 }).volume).toBe(3)
    expect(progress.normalize({ volume: 9 }).volume).toBe(1)
    expect(progress.maxSeals(3)).toBe(21)
  })

  it('stays deterministic for the same seed and inputs', () => {
    const play = () => {
      const state = rules.createGame({ level: '3-scroll', seed: 5 })
      run(state, 60 * 30, (i) => ({ move: i % 300 < 260 ? 1 : -1, slashPressed: i % 11 === 0, jumpPressed: i % 97 === 0, jump: i % 97 < 12, dashPressed: i % 173 === 0 }))
      return JSON.stringify(rules.result(state)) + state.player.x.toFixed(3)
    }
    expect(play()).toBe(play())
  })
})
