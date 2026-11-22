import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
interface Input { move?: number; jump?: boolean; jumpPressed?: boolean; slashPressed?: boolean; dashPressed?: boolean; inkPressed?: boolean }
const context: Record<string, Any> = {}
for (const file of ['levels.js', 'rules.js', 'progress.js']) {
  runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`, import.meta.url), 'utf8'), context)
}
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

function arena(level: string, x: number, difficulty = 'normal') {
  const state = rules.createGame({ level, seed: 7, difficulty })
  const enc = state.plan[0]
  state.lock = [enc.lock[0] + 40, enc.lock[1] - 40]
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

/** Scripted swordsman that reads cinnabar tells, respects 军中不乱刀, parries ripostes, dodges hazards and disperses stains. */
function runParryBot(level: string, difficulty = 'normal', seed = 7, maxSeconds = 260) {
  const state = rules.createGame({ level, difficulty, seed })
  let lastSlash = -99
  const stages = new Set<number>()
  for (let i = 0; i < 60 * maxSeconds && state.phase === 'playing'; i++) {
    const p = state.player
    const boss = state.enemies.find((e: Any) => e.type === 'nameless' && e.alive && e.delay <= 0)
    if (boss) stages.add(boss.namelessStage)
    const activeEnemies = state.enemies.filter((e: Any) => e.alive && e.delay <= 0 && e.state !== 'spawn' && e.state !== 'dying')
    const target = activeEnemies.sort((a: Any, b: Any) => Math.abs(a.x - p.x) - Math.abs(b.x - p.x))[0]
    const fresh = !p.action && p.recoil <= 0 && i - lastSlash > 24
    const input: Input = {}

    // 1. Dodge volleys
    const volley = state.volleys.find((v: Any) => !v.landed && Math.abs(p.x - v.x) < v.w / 2 + 16 && !rules.sheltered(state, p.x))
    // 2. Dodge blank stroke (or clash if ink full!)
    const blank = state.blankStroke
    // 3. Deflect incoming enemy projectiles
    const shot = state.projectiles.find((s: Any) => !s.dead && s.owner === 'enemy' && Math.abs(s.x - p.x) < 58 && Math.abs(s.y - (p.y - 26)) < 48)
    // 4. Enemy about to hit (parriable or danger)
    const danger = activeEnemies.find((e: Any) => {
      if (e.state === 'rush' || e.state === 'leap') return Math.abs(e.x - p.x) < 130
      if (e.state === 'rushTell' || e.state === 'crouch') return e.t < 0.14
      return false
    })
    const parriable = activeEnemies.find((e: Any) => {
      const dist = Math.abs(e.x - p.x)
      if (e.type === 'shadeRider' && e.state === 'attack' && !e.hitDone) return dist < 82 && Math.sign(p.x - e.x) === Math.sign(e.vx)
      if (e.type === 'crow' && e.state === 'dive' && !e.hitDone) return Math.hypot(e.x - p.x, e.y - (p.y - 28)) < 68
      if (e.state === 'airRiposteTele' && e.t < 0.08) return true
      if (e.state === 'airRiposteAttack' && !e.hitDone) return Math.abs(e.x - p.x) < 62
      if (['tele', 'riposteTele', 'duelTell', 'duelThrustTell'].includes(e.state) && e.type !== 'shadeArcher' && e.type !== 'archer' && e.type !== 'crow') return e.t < 0.06 && dist < 135
      if (e.state === 'combo' && e.combo?.phase === 'tele') return e.t < 0.06 && dist < 130
      return false
    })
    const staggered = activeEnemies.find((e: Any) => (e.state === 'stagger' || e.state === 'broken') && p.slashStartClock <= (e.openedAt ?? -9))
    const gathering = state.stains.find((s: Any) => !s.dead && s.phase === 'gather')

    if (blank && p.ink >= 100) {
      input.inkPressed = true
    } else if (blank && ((blank.phase === 'tell' && blank.tele - blank.t < 0.16) || blank.phase === 'active')) {
      input.jumpPressed = p.onGround
      input.jump = true
    } else if (danger) {
      input.jumpPressed = p.onGround
      input.jump = true
      if (!p.onGround && p.dashCd <= 0) input.dashPressed = true
    } else if (volley) {
      const wall = (state.world.ramparts || []).find((w: Any) => Math.abs(w.x - p.x) < 120)
      if (wall) input.move = Math.sign(wall.x - 20 - p.x)
      else input.move = p.x < volley.x ? -1 : 1
      if (p.dashCd <= 0 && volley.tele - volley.t < 0.25) input.dashPressed = true
    } else if (shot && fresh) {
      p.facing = Math.sign(shot.x - p.x) || p.facing
      input.slashPressed = true
      lastSlash = i
    } else if (parriable && fresh) {
      p.facing = Math.sign(parriable.x - p.x) || p.facing
      input.slashPressed = true
      lastSlash = i
    } else if (staggered) {
      const dist = Math.abs(staggered.x - p.x)
      if (dist > 62) input.move = Math.sign(staggered.x - p.x)
      else if (fresh) { p.facing = Math.sign(staggered.x - p.x) || p.facing; input.slashPressed = true; lastSlash = i }
    } else if (gathering) {
      const dist = Math.abs(gathering.x - p.x)
      if (dist > 55) input.move = Math.sign(gathering.x - p.x)
      else if (fresh) { p.facing = Math.sign(gathering.x - p.x) || p.facing; input.slashPressed = true; lastSlash = i }
    } else if (target) {
      const dist = Math.abs(target.x - p.x)
      if (target.onPlatform && target.y < p.y - 25 && dist < 130 && p.onGround && i % 40 === 0) {
        input.jumpPressed = true; input.jump = true
      }
      // For shade/nameless, bait their attack or strike during recover/counterCd instead of blindly hitting into 拆
      if ((target.type === 'shade' || target.type === 'nameless') && target.state === 'recover' && dist < 72 && fresh) {
        p.facing = Math.sign(target.x - p.x) || p.facing
        input.slashPressed = true; lastSlash = i
      } else if (target.type === 'shade' && dist <= 74 && fresh && target.counterCd <= 0 && target.state === 'idle' && i % 55 === 0) {
        // Deliberately provoke 拆 to farm riposte parry -> clean execution!
        p.facing = Math.sign(target.x - p.x) || p.facing
        input.slashPressed = true; lastSlash = i
      } else if ((target.type === 'shield' || target.type === 'pavise') && dist > 68) {
        input.move = Math.sign(target.x - p.x)
      } else if (target.type === 'shadeArcher' || target.type === 'archer') {
        if (dist > 180) input.move = Math.sign(target.x - p.x)
        else if (dist < 62 && target.y >= p.y - 35 && fresh) { p.facing = Math.sign(target.x - p.x) || p.facing; input.slashPressed = true; lastSlash = i }
      } else if (target.type === 'shadeRider') {
        // Stand ready to parry its charge
        if (dist > 260) input.move = Math.sign(target.x - p.x)
      } else if (dist > 66) {
        input.move = Math.sign(target.x - p.x)
      } else if (target.type === 'soldier' && fresh && target.state === 'recover') {
        p.facing = Math.sign(target.x - p.x) || p.facing
        input.slashPressed = true; lastSlash = i
      }
      if (p.ink >= 100 && (!boss || boss.namelessStage < 3)) input.inkPressed = true
    } else if (!state.lock) {
      input.move = 1
    }

    rules.step(state, input, DT)
  }
  return { state, result: rules.result(state), stages }
}

/** Pure button-mashing script that walks toward enemies and mashes slash without reading tells. */
function runMashBot(level: string, seed = 7, maxSeconds = 120) {
  const state = rules.createGame({ level, difficulty: 'normal', seed })
  for (let i = 0; i < 60 * maxSeconds && state.phase === 'playing'; i++) {
    const p = state.player
    const target = state.enemies.find((e: Any) => e.alive && e.delay <= 0 && e.state !== 'spawn')
    const move = target ? (Math.abs(target.x - p.x) > 48 ? Math.sign(target.x - p.x) : 0) : 1
    rules.step(state, { move, slashPressed: i % 8 === 0 }, DT)
  }
  return { state, result: rules.result(state) }
}

describe('归墨卷 · structure, saves and isolation', () => {
  it('defines six stages and a scroll with codes 701–706 and 700, bounded arenas, valid enemies and telegraphs >= 0.45s', () => {
    const vol = levels.volume(7)
    const list = levels.volumeLevels(7)
    expect(vol.name).toBe('归墨卷')
    expect(list.map((l: Any) => l.code)).toEqual([701, 702, 703, 704, 705, 706, 700])
    const state = rules.createGame({ level: '7-scroll' })
    expect(state.plan.map((e: Any) => e.id)).toEqual(['ashes', 'ruined_pass', 'posthouse', 'bamboo_sea', 'blank', 'nameless'])
    for (const l of list) {
      if (l.challenge.stat !== 'time') expect(state.player).toHaveProperty(l.challenge.stat)
      for (const encId of l.encounters) {
        const enc = vol.encounters[encId]
        expect(enc.lock[1] - enc.lock[0] - 80).toBeLessThanOrEqual(680)
        expect(enc.lock[0] + 40).toBeGreaterThanOrEqual(l.range[0])
        expect(enc.lock[1] - 40).toBeLessThanOrEqual(l.range[1])
        for (const wave of enc.waves) for (const spec of wave) expect(rules.ENEMY[spec.type]).toBeDefined()
      }
    }
    expect(rules.ENEMY.shade.tele).toBeGreaterThanOrEqual(0.45)
    expect(rules.ENEMY.shadeRider.tele).toBeGreaterThanOrEqual(0.45)
  })

  it('preserves old saves, unlocks stages sequentially, and unlocks 千里江山 only after all 7 bosses fall', () => {
    const old = { ...progress.empty(), volume: 6, levels: { '1-6': { clears: 1, best: 5000, grade: 2, seals: 3, time: 80 } } }
    const restored = progress.normalize(old)
    expect(restored.levels['1-6']).toEqual(old.levels['1-6'])
    expect(progress.unlocked(restored, '7-1')).toBe(true)
    expect(progress.unlocked(restored, '7-2')).toBe(false)
    expect(progress.grandScrollUnlocked(restored)).toBe(false)
    let save = restored
    for (const id of ['2-6', '3-6', '4-6', '5-6', '6-6']) {
      save = progress.record(save, { level: id, cleared: true, score: 5000, seals: 1, grade: 1, seconds: 90 }).save
    }
    expect(progress.grandScrollUnlocked(save)).toBe(false)
    save = progress.record(save, { level: '7-6', cleared: true, score: 8000, seals: 1, grade: 2, seconds: 110 }).save
    expect(progress.grandScrollUnlocked(save)).toBe(true)
    expect(progress.unlocked(save, '7-scroll')).toBe(true)
    expect(progress.normalize({ ...save, volume: 7 }).volume).toBe(7)
    expect(progress.maxSeals(7)).toBe(21)
  })

  it('does not leak counterGuard or reform mechanics into earlier volumes', () => {
    for (const id of [1, 2, 3, 4, 5, 6]) {
      const vol = levels.volume(id)
      expect(vol.counterGuard).toBeUndefined()
      expect(vol.reform).toBeUndefined()
      expect(vol.whiteout).toBeUndefined()
    }
    const bamboo = rules.createGame({ level: '1-2', seed: 3 })
    bamboo.lock = [1370, 2050]; bamboo.encounter = 99; bamboo.player.x = 1600
    const puppet = place(bamboo, 'puppet', 1650, { facing: -1 })
    const ev = run(bamboo, 16, (i) => ({ slashPressed: i === 0 }))
    expect(ev.some((e) => e.type === 'counter')).toBe(false)
    expect(puppet.hp).toBe(2)
  })
})

describe('归墨卷 · 拆与回锋 (counter & riposte)', () => {
  it('counters frontal slashes with 0.18s recoil (dashable), ripostes in 0.50s, and yields a parryable execution', () => {
    const state = arena('7-1', 1000)
    const shade = place(state, 'shade', 1052, { facing: -1 })
    const ev = run(state, 10, (i) => ({ slashPressed: i === 0 }))
    expect(ev.some((e) => e.type === 'counter')).toBe(true)
    expect(shade.hp).toBe(3)
    expect(state.player.hp).toBe(state.player.maxHp)
    expect(state.player.recoil).toBeGreaterThan(0)
    expect(shade.state).toBe('riposteTele')
    expect(shade.teleTotal).toBeCloseTo(0.50)

    // Wait for recoil to clear and parry the incoming riposte with a single fresh swing
    let parried = false, pressed = false
    for (let i = 0; i < 60 && !parried; i++) {
      const press = !pressed && shade.state === 'riposteTele' && shade.t < 0.05
      if (press) pressed = true
      parried = rules.step(state, { slashPressed: press }, DT).some((e: Any) => e.type === 'parry')
    }
    expect(parried).toBe(true)
    expect(shade.state).toBe('stagger')
    expect(state.player.counterParries).toBe(1)

    // Fresh swing executes the staggered shade
    state.hitstop = 0; state.slow = 0
    while (state.player.action) run(state, 1)
    state.player.facing = Math.sign(shade.x - state.player.x) || 1
    const exec = run(state, 20, (i) => ({ move: Math.abs(shade.x - state.player.x) > 55 ? Math.sign(shade.x - state.player.x) : 0, slashPressed: i === 2 }))
    expect(exec.some((e) => e.type === 'execute')).toBe(true)
    expect(shade.alive).toBe(false)
  })

  it('allows dashing during recoil, does not counter back hits, and respects the 2.4s counter cooldown', () => {
    const dashTest = arena('7-1', 1000)
    place(dashTest, 'shade', 1052, { facing: -1 })
    run(dashTest, 6, (i) => ({ slashPressed: i === 0 }))
    dashTest.hitstop = 0
    expect(dashTest.player.recoil).toBeGreaterThan(0)
    const dashed = run(dashTest, 3, (i) => ({ dashPressed: i === 0, move: -1 }))
    expect(dashed.some((e) => e.type === 'dash')).toBe(true)

    const backTest = arena('7-1', 1100)
    backTest.player.facing = -1
    // Early tele facing away: counterable from front, must NOT counter from behind
    const shade = place(backTest, 'shade', 1050, { facing: -1, state: 'tele', t: 0.45, teleTotal: 0.5 })
    const ev = run(backTest, 12, (i) => ({ slashPressed: i === 0 }))
    expect(ev.some((e) => e.type === 'counter')).toBe(false)
    expect(shade.hp).toBe(2)

    const cdTest = arena('7-1', 1000)
    const s2 = place(cdTest, 'shade', 1052, { facing: -1, counterCd: 2.0 })
    const ev2 = run(cdTest, 12, (i) => ({ slashPressed: i === 0 }))
    expect(ev2.some((e) => e.type === 'counter')).toBe(false)
    expect(s2.hp).toBe(2)
  })

  it('rejects chained swings from parrying a riposte under 军中不乱刀', () => {
    const state = arena('7-1', 1000)
    rules.step(state, { slashPressed: true }, DT)
    run(state, 9)
    rules.step(state, { slashPressed: true }, DT)
    expect(state.player.action.chained).toBe(true)
    place(state, 'shade', state.player.x + 48, { facing: -1, state: 'riposteAttack', t: 0.12, vx: -260, hitDone: false })
    const ev = run(state, 5)
    expect(ev.some((e) => e.type === 'parry')).toBe(false)
    expect(ev.some((e) => e.type === 'hurt')).toBe(true)
  })
})

describe('归墨卷 · 二返之箭, 落马回锋, 不斩不散 & 纸雾', () => {
  it('makes shadeArcher bounce the first deflected arrow back once, and die cleanly to 二返之箭', () => {
    const state = arena('7-4', 3500) // even in reform zone, 二返之箭 must not leave a stain!
    const archer = place(state, 'shadeArcher', 3760, { facing: -1, cd: 0 })
    // Let archer shoot
    for (let i = 0; i < 90 && !state.projectiles.length; i++) rules.step(state, {}, DT)
    expect(state.projectiles.length).toBe(1)
    archer.cd = 99
    // First player deflect
    let firstDeflected = false
    for (let i = 0; i < 90 && !firstDeflected; i++) {
      const s = state.projectiles[0]
      const near = s && Math.abs(s.x - state.player.x) < 42
      firstDeflected = rules.step(state, { slashPressed: near }, DT).some((e: Any) => e.type === 'deflect')
    }
    expect(firstDeflected).toBe(true)
    // Arrow hits idle archer -> archer counters it back!
    const counterEvents = run(state, 60)
    expect(counterEvents.some((e) => e.type === 'counter' && e.arrow)).toBe(true)
    expect(archer.alive).toBe(true)
    // Wait for archer to release the returned arrow, then player deflects a second time (二返之箭)
    let secondDeflected = false
    for (let i = 0; i < 120 && !secondDeflected; i++) {
      const s = state.projectiles.find((p: Any) => p.owner === 'enemy')
      const near = s && Math.abs(s.x - state.player.x) < 46 && !state.player.action
      secondDeflected = rules.step(state, { slashPressed: Boolean(near) }, DT).some((e: Any) => e.type === 'deflect' && e.doubleReturn)
    }
    expect(secondDeflected).toBe(true)
    run(state, 60)
    expect(archer.alive).toBe(false)
    expect(state.player.doubleReturns).toBe(1)
    expect(state.stains).toHaveLength(0)
  })

  it('turns a parried shadeRider into an airborne 落马回锋 whose parry opens a 2s stagger', () => {
    const state = arena('7-3', 2700)
    const rider = place(state, 'shadeRider', 2860, { facing: -1, state: 'attack', t: 1.5, vx: -460, hitDone: false })
    let unhorsed = false
    for (let i = 0; i < 50 && !unhorsed; i++) {
      const close = rider.x - state.player.x < 75
      unhorsed = rules.step(state, { slashPressed: close }, DT).some((e: Any) => e.type === 'unhorse')
    }
    expect(unhorsed).toBe(true)
    expect(rider.type).toBe('shade')
    expect(rider.state).toBe('airRiposteTele')
    expect(rider.y).toBeLessThan(G - 40)

    state.hitstop = 0; state.slow = 0
    let counterParried = false
    for (let i = 0; i < 80 && !counterParried; i++) {
      const press = rider.state === 'airRiposteTele' && rider.t < 0.06 && !state.player.action
      counterParried = rules.step(state, { slashPressed: press }, DT).some((e: Any) => e.type === 'parry')
    }
    expect(counterParried).toBe(true)
    expect(rider.state).toBe('stagger')
    expect(rider.y).toBe(G)
    expect(state.player.riderCounters).toBe(1)
  })

  it('reforms unexecuted shades once in 竹海, blocks wave clear until gone, and allows slashing the gathering stain', () => {
    const state = arena('7-4', 3600)
    const shade = place(state, 'shade', 3650, { facing: -1, hp: 1, counterCd: 5 })
    run(state, 12, (i) => ({ slashPressed: i === 0 }))
    expect(shade.alive).toBe(false)
    expect(state.stains).toHaveLength(1)
    // Advance until gather phase (2.6s)
    const gatherEv = run(state, 60 * 2.8)
    expect(gatherEv.some((e) => e.type === 'stainGather')).toBe(true)
    expect(state.stains[0].phase).toBe('gather')
    // Cut the gathering stain!
    const breakEv = run(state, 12, (i) => ({ slashPressed: i === 0 }))
    expect(breakEv.some((e) => e.type === 'stainBreak')).toBe(true)
    expect(state.stains).toHaveLength(0)

    // Now let one actually reform -> 2 HP, half score, never reforms twice
    const s2 = arena('7-4', 3600)
    place(s2, 'shade', 3650, { facing: -1, hp: 1, counterCd: 5 })
    run(s2, 12, (i) => ({ slashPressed: i === 0 }))
    expect(s2.stains).toHaveLength(1)
    const refEv = run(s2, 60 * 3.6)
    expect(refEv.some((e) => e.type === 'stainReform')).toBe(true)
    expect(s2.stains).toHaveLength(0)
    expect(s2.player.reforms).toBe(1)
    const reformed = s2.enemies.find((e: Any) => e.alive && e.reformed)
    expect(reformed).toBeDefined()
    expect(reformed.hp).toBe(2)
    reformed.counterCd = 5; reformed.hp = 1
    run(s2, 12, (i) => ({ slashPressed: i === 0 }))
    expect(reformed.alive).toBe(false)
    expect(s2.stains).toHaveLength(0) // never reforms twice
  })

  it('shortens archer sight in 留白 whiteout and shelters behind 断关 ramparts', () => {
    const fog = arena('7-5', 4400)
    const archer = place(fog, 'shadeArcher', 4800, { facing: -1, cd: 0 })
    run(fog, 5)
    expect(archer.state).toBe('idle')
    fog.player.x = 4520
    run(fog, 5)
    expect(archer.state).toBe('tele')

    const wall = levels.volume(7).ramparts[0]
    const cover = arena('7-2', wall.x - 20)
    expect(rules.sheltered(cover, cover.player.x)).toBe(true)
    rules.callVolley(cover, cover.player.x, 280, 1.2)
    run(cover, 60 * 1.3)
    expect(cover.player.hp).toBe(cover.player.maxHp)
  })
})

describe('归墨卷 · 无名剑客 (Boss)', () => {
  it('spawns Nameless directly into a short hittable intro like 岳停云', () => {
    const state = arena('7-6', 5840)
    const boss = rules.spawnEnemy(state, { type: 'nameless', x: 5890 })
    expect(boss.state).toBe('intro')
    expect(boss.t).toBeCloseTo(0.8)
    const before = boss.hp
    const ev = run(state, 10, (i) => ({ slashPressed: i === 0 }))
    expect(ev.some((e) => e.type === 'hit')).toBe(true)
    expect(boss.hp).toBeLessThan(before)
  })

  it('supports 笔会 brush clash during 留白一笔 and jumping over it when not clashing', () => {
    const clash = arena('7-6', 5600)
    const boss = place(clash, 'nameless', 5800, { facing: -1, namelessStage: 3, state: 'blankTell', t: 0.8, teleTotal: 1.1 })
    clash.blankStroke = { phase: 'tell', t: 0.3, tele: 1.1, dur: 0.35, dir: -1, x0: 5340, x1: 6020, hit: false }
    clash.player.ink = 100
    const ev = run(clash, 4, (i) => ({ inkPressed: i === 0 }))
    expect(ev.some((e) => e.type === 'brushClash')).toBe(true)
    expect(clash.blankStroke).toBeNull()
    expect(boss.posture).toBeGreaterThanOrEqual(30)
    expect(boss.state).toBe('recover')

    // Jumping clears active blankStroke
    const jump = arena('7-6', 5600)
    place(jump, 'nameless', 5800, { facing: -1, namelessStage: 3, state: 'blankTell', t: 0.20, teleTotal: 1.1 })
    jump.blankStroke = { phase: 'tell', t: 0.90, tele: 1.1, dur: 0.22, dir: -1, x0: 5340, x1: 6020, hit: false }
    run(jump, 45, (i) => ({ jumpPressed: i === 1, jump: i >= 1 && i < 24 }))
    expect(jump.player.hp).toBe(jump.player.maxHp)
  })

  it('triggers 剑鸣大破 after 3 consecutive stage-3 parries (2 on easy) and executes for 12 damage', () => {
    for (const [difficulty, needed] of [['normal', 3], ['easy', 2]] as const) {
      const state = arena('7-6', 5620, difficulty)
      const boss = place(state, 'nameless', 5700, { facing: -1, namelessStage: 3, hp: 18, state: 'duelThrustTell', t: 0.05, teleTotal: 0.55 })
      let broken = false
      let clashes = 0
      for (let i = 0; i < 220 && !broken; i++) {
        const press = !state.player.action && (
          (boss.state === 'duelThrustTell' && boss.t < 0.06) ||
          (boss.state === 'duelTell' && boss.t < 0.06)
        )
        const ev = rules.step(state, { slashPressed: press }, DT)
        clashes += ev.filter((e: Any) => e.type === 'duelClash').length
        if (ev.some((e: Any) => e.type === 'resonanceBreak')) broken = true
      }
      expect(clashes).toBe(needed)
      expect(broken).toBe(true)
      expect(boss.state).toBe('broken')
      expect(state.player.resonanceBreaks).toBe(1)

      state.hitstop = 0; state.slow = 0
      while (state.player.action) run(state, 1)
      state.player.facing = Math.sign(boss.x - state.player.x) || 1
      const before = boss.hp
      const exec = run(state, 20, (i) => ({ slashPressed: i === 0 }))
      expect(exec.some((e) => e.type === 'execute')).toBe(true)
      expect(before - boss.hp).toBe(12)
    }
  })

  it('lets a timed-parry swordsman clear all six stages and defeat Nameless across all 3 stages', () => {
    for (const id of ['7-1', '7-2', '7-3', '7-4', '7-5']) {
      const { result } = runParryBot(id, 'easy', 11, 220)
      expect(result.cleared, `stage ${id} failed: ${JSON.stringify(result)}`).toBe(true)
    }
    const bossRun = runParryBot('7-6', 'easy', 11, 260)
    expect(bossRun.result.cleared, JSON.stringify(bossRun.result)).toBe(true)
    expect([...bossRun.stages].sort()).toEqual([1, 2, 3])
  })

  it('prevents a pure button-mashing script from getting 神品 or clearing 荒驿 / 无名 on normal', () => {
    for (const id of ['7-3', '7-6']) {
      const { result } = runMashBot(id, 7, 100)
      expect(result.cleared).toBe(false)
    }
    for (const id of ['7-1', '7-2', '7-4', '7-5']) {
      const { result } = runMashBot(id, 7, 100)
      expect(result.grade).toBeLessThanOrEqual(2)
    }
  })

  it('stays deterministic for the same seed and inputs', () => {
    const play = () => {
      const state = rules.createGame({ level: '7-scroll', seed: 9 })
      run(state, 60 * 25, (i) => ({ move: i % 240 < 200 ? 1 : -1, slashPressed: i % 13 === 0, jumpPressed: i % 89 === 0, jump: i % 89 < 12, dashPressed: i % 157 === 0 }))
      return JSON.stringify(rules.result(state)) + state.player.x.toFixed(3)
    }
    expect(play()).toBe(play())
  })
})
