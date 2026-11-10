import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* 竹林卷 · 架刀、收刀、回墨 and the finesse cap on grades (PRD 3.18). */
/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
interface Input { move?: number; slashPressed?: boolean; jumpPressed?: boolean; dashPressed?: boolean }
const context: Any = {}
for (const file of ['levels.js', 'rules.js']) runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`, import.meta.url), 'utf8'), context)
const rules = context.XiabanInkBlade
const levels = context.XiabanInkLevels
const DT = 1 / 60

function run(state: Any, frames: number, input: Input | ((i: number) => Input) = {}) {
  const events: Any[] = []
  for (let i = 0; i < frames; i++) events.push(...rules.step(state, typeof input === 'function' ? input(i) : input, DT))
  return events
}

// The player alone in a locked arena, facing right, with the tutorial prompts already seen.
function arena(level = '1-2', difficulty = 'normal') {
  const state = rules.createGame({ level, difficulty, seed: 3 })
  state.lock = [1000, 1800]
  state.encounter = 99
  state.player.x = 1400
  return state
}

function place(state: Any, type: string, x: number, extra: Any = {}) {
  const enemy = rules.spawnEnemy(state, { type, x })
  Object.assign(enemy, { state: 'idle', t: 0, cd: 99, facing: -1 }, extra)
  return enemy
}

const slashOnce = (i: number) => ({ slashPressed: i === 0 })

// One cut that lands, then enough frames for the swing and the flinch to finish.
function poke(state: Any) {
  const events = run(state, 12, slashOnce)
  run(state, 30)
  return events
}

describe('竹林卷 · 架刀', () => {
  it('lets the first cut land and blocks the next frontal cut with a telegraphed answer', () => {
    const state = arena()
    const puppet = place(state, 'puppet', 1450)
    expect(poke(state).some((e) => e.type === 'hit')).toBe(true)
    expect([puppet.hp, puppet.guard]).toEqual([2, true])

    const events = run(state, 8, slashOnce)
    expect(events.some((e) => e.type === 'block' && e.guard)).toBe(true)
    expect(events.some((e) => e.type === 'tell' && e.id === puppet.id)).toBe(true)
    expect(puppet.hp).toBe(2)
    expect(puppet.state).toBe('tele')
    expect(puppet.teleTotal).toBeGreaterThanOrEqual(0.65)
    expect(state.player.guardBlocks).toBe(1)
  })

  it('drops presses during the recoil and refuses to parry with a blade that has not settled', () => {
    const state = arena()
    const puppet = place(state, 'puppet', 1450)
    poke(state)
    run(state, 5, slashOnce)
    expect(state.player.recoil).toBeGreaterThan(0)
    rules.step(state, { slashPressed: true }, DT)
    expect(state.player.slashBuffer).toBeLessThanOrEqual(0)

    // Hammering the button through the answering cut gets the swordsman hit, never a parry.
    const hp = state.player.hp
    const events = run(state, 70, (i) => ({ slashPressed: i % 6 === 0 }))
    expect(events.some((e) => e.type === 'parry')).toBe(false)
    expect(state.player.hp).toBe(hp - 1)
    expect(puppet.alive).toBe(true)
  })

  it('parries the answering cut when the swordsman waits for the glint', () => {
    const state = arena()
    const puppet = place(state, 'puppet', 1450)
    poke(state)
    run(state, 8, slashOnce)
    expect(puppet.state).toBe('tele')
    const events: Any[] = []
    let pressed = false
    for (let i = 0; i < 80; i++) {
      const press: boolean = !pressed && puppet.state === 'tele' && puppet.t <= 0.08
      pressed = pressed || press
      events.push(...rules.step(state, { slashPressed: press }, DT))
    }
    expect(events.some((e) => e.type === 'parry')).toBe(true)
    expect(puppet.state).toBe('stagger')
    expect(puppet.guard).toBe(false)
    // The opening is an execution for a fresh swing.
    const finish = run(state, 30, slashOnce)
    expect(finish.some((e) => e.type === 'execute')).toBe(true)
    expect(puppet.alive).toBe(false)
  })

  it('leaves the guard down while the enemy recovers from its own attack', () => {
    const state = arena()
    const puppet = place(state, 'puppet', 1450, { state: 'recover', t: 2, guard: true })
    run(state, 1)
    expect(puppet.guard).toBe(false)
    run(state, 12, slashOnce)
    expect(puppet.hp).toBe(2)
    expect(puppet.guard).toBe(false)
  })

  it('does not guard from behind, against the ink stroke, or in a 卷 without the rule', () => {
    const behind = arena()
    // Mid-flinch it cannot turn round, so its back is open.
    const turned = place(behind, 'puppet', 1450, { facing: 1, guard: true, state: 'hurt', t: 1 })
    run(behind, 12, slashOnce)
    expect(turned.hp).toBe(2)

    const snow = arena('2-2')
    const ronin = place(snow, 'ronin', 1450)
    run(snow, 12, slashOnce)
    run(snow, 30)
    run(snow, 12, slashOnce)
    expect(ronin.hp).toBe(1)
    expect(snow.player.guardBlocks).toBe(0)
  })

  it('raises the boss guard once his counter is provoked and drops it when he recovers', () => {
    const state = arena('1-6')
    state.lock = [5300, 6060]; state.player.x = 5600
    const boss = place(state, 'boss', 5650, { t: 99 })
    const full = boss.hp
    run(state, 12, slashOnce); run(state, 24)
    boss.state = 'idle'; boss.t = 99; boss.x = 5650; state.player.x = 5600
    run(state, 12, slashOnce)
    expect(boss.hp).toBe(full - 2)
    expect([boss.state, boss.guard]).toEqual(['combo', true])

    boss.t = 99; boss.x = 5650; state.player.x = 5600; state.player.iframes = 9
    run(state, 30)
    const events = run(state, 12, slashOnce)
    expect(events.some((e) => e.type === 'block' && e.guard)).toBe(true)
    expect(boss.hp).toBe(full - 2)

    boss.state = 'recover'; boss.t = 2; boss.combo = null; boss.x = 5650; state.player.x = 5600
    run(state, 30)
    expect(boss.guard).toBe(false)
    run(state, 12, slashOnce)
    expect(boss.hp).toBe(full - 3)
  })
})

describe('竹林卷 · 收刀', () => {
  it('parries with a settled swing but not with one thrown straight after another', () => {
    const settled = arena()
    const first = place(settled, 'puppet', 1460, { state: 'attack', t: 0.12, hitDone: false })
    expect(run(settled, 3, slashOnce).some((e) => e.type === 'parry')).toBe(true)
    expect(first.state).toBe('stagger')

    const hasty = arena()
    rules.step(hasty, { slashPressed: true }, DT)
    run(hasty, 9)
    rules.step(hasty, { slashPressed: true }, DT)
    run(hasty, 9)
    expect(hasty.player.action.rested).toBe(false)
    place(hasty, 'puppet', 1460, { state: 'attack', t: 0.12, hitDone: false })
    const hp = hasty.player.hp
    expect(run(hasty, 3, slashOnce).some((e) => e.type === 'parry')).toBe(false)
    expect(hasty.player.hp).toBe(hp - 1)
  })

  it('keeps a string of parries flowing: the swing after a parry is always settled', () => {
    const state = arena()
    place(state, 'puppet', 1460, { state: 'attack', t: 0.12, hitDone: false })
    run(state, 24, slashOnce)
    expect(state.player.parries).toBe(1)
    // The parrying swing is still running, so this press chains from it; it parries all the same.
    expect(state.player.action?.type).toBe('slash')
    place(state, 'puppet', state.player.x + 55, { state: 'attack', t: 0.12, hitDone: false })
    run(state, 8, slashOnce)
    expect(state.player.parries).toBe(2)
  })

  it.each([
    [0, false], [4, false], [12, true],
  ])('respects the full recovery after a heavy slash (%i extra frames)', (wait, canParry) => {
    const state = arena()
    run(state, 10, slashOnce)
    run(state, 10, slashOnce)
    run(state, 26, slashOnce)
    expect(state.player.action?.stage).toBe(2)
    run(state, wait)

    place(state, 'puppet', state.player.x + 55, { state: 'attack', t: 0.12, hitDone: false })
    const events = run(state, 3, slashOnce)
    expect(events.some((e) => e.type === 'parry')).toBe(canParry)
  })

  it('widens the parry window in the 幅 that teaches it', () => {
    const windowOf = (level: string) => {
      const state = arena(level)
      rules.step(state, { slashPressed: true }, DT)
      run(state, 11)
      place(state, 'puppet', state.player.x + 50, { state: 'attack', t: 0.12, hitDone: false })
      return run(state, 2).some((e) => e.type === 'parry')
    }
    expect(levels.level('1-2').parryEase).toBeGreaterThan(0)
    expect(windowOf('1-2')).toBe(true)
    expect(windowOf('1-5')).toBe(false)
  })
})

describe('竹林卷 · 回墨 and grades', () => {
  it('restores one 墨血 when a wave is cleared, never above the maximum', () => {
    const state = rules.createGame({ level: '1-2', seed: 3 })
    state.player.x = 1600
    run(state, 2)
    expect(state.lock).not.toBeNull()
    state.player.hp = 3
    for (const enemy of state.enemies) enemy.alive = false
    const events = run(state, 2)
    expect(events.some((e) => e.type === 'heal')).toBe(true)
    expect(state.player.hp).toBe(4)

    const full = rules.createGame({ level: '1-2', seed: 3 })
    full.player.x = 1600
    run(full, 2)
    for (const enemy of full.enemies) enemy.alive = false
    run(full, 2)
    expect(full.player.hp).toBe(full.player.maxHp)

    const snow = rules.createGame({ level: '2-1', seed: 3 })
    snow.player.x = 960
    run(snow, 2)
    snow.player.hp = 3
    for (const enemy of snow.enemies) enemy.alive = false
    run(snow, 2)
    expect(snow.player.hp).toBe(3)
  })

  it('restores one 墨血 for the final wave before calculating the clear bonus', () => {
    const state = rules.createGame({ level: '1-2', seed: 3 })
    state.player.x = 1600
    run(state, 2)
    for (let wave = 0; wave < 2; wave++) {
      for (const enemy of state.enemies) enemy.alive = false
      run(state, 46)
    }
    expect(state.wave).toBe(2)
    expect(state.enemies.some((enemy: Any) => enemy.alive)).toBe(true)
    state.player.hp = 3
    for (const enemy of state.enemies) enemy.alive = false

    const events = run(state, 3)
    expect(events.filter((e) => e.type === 'heal')).toHaveLength(1)
    expect(state.reason).toBe('clear')
    expect(state.player.hp).toBe(4)
    expect(state.bonus.hp).toBe(600)
  })

  it('restores one 墨血 for executing the boss', () => {
    const state = arena('1-6')
    state.lock = [5300, 6060]; state.player.x = 5600; state.player.hp = 2
    const boss = place(state, 'boss', 5650, { state: 'broken', t: 2, openedAt: -1 })
    const events = run(state, 12, slashOnce)
    expect(events.some((e) => e.type === 'execute')).toBe(true)
    expect(state.player.hp).toBe(3)
    expect(boss.hp).toBe(boss.maxHp - 10)
  })

  it('holds a high score at 妙品 until the 幅 has seen enough parries and executions', () => {
    const state = rules.createGame({ level: '1-2' })
    const level = levels.level('1-2')
    state.reason = 'clear'
    state.player.score = level.grades[2] + 1
    expect(rules.evaluate(state)).toMatchObject({ grade: 2, capped: true })
    expect(rules.result(state)).toMatchObject({ capped: true, finesse: 0, finesseGoal: level.finesse })
    state.player.parries = 2; state.player.executions = level.finesse - 2
    expect(rules.evaluate(state)).toMatchObject({ grade: 4, capped: false })
    // 山门 has only straw dummies, so it sets no finesse and grades by score alone.
    const gate = rules.createGame({ level: '1-1' })
    gate.reason = 'clear'; gate.player.score = levels.level('1-1').grades[2]
    expect(rules.evaluate(gate)).toMatchObject({ grade: 4, capped: false })
  })

  it.each([
    ['1-2', 4], ['1-2', 6], ['1-2', 10], ['1-5', 6], ['1-5', 10], ['1-6', 6], ['1-6', 10],
  ])('never gives a button masher 神品 on 江湖 (%s, a press every %i frames)', (level, period) => {
    for (const seed of [8, 21, 77]) {
      const state = rules.createGame({ level, seed })
      for (let i = 0; i < 60 * 200 && state.phase === 'playing'; i++) {
        const player = state.player
        const enemy = state.enemies.filter((e: Any) => e.alive && e.delay <= 0).sort((a: Any, b: Any) => Math.abs(a.x - player.x) - Math.abs(b.x - player.x))[0]
        const move = enemy ? (Math.abs(enemy.x - player.x) > 60 ? Math.sign(enemy.x - player.x) : 0) : state.lock ? 0 : 1
        rules.step(state, { move, slashPressed: i % period === 0 }, DT)
      }
      expect(rules.evaluate(state).grade).toBeLessThan(3)
    }
  })
})
