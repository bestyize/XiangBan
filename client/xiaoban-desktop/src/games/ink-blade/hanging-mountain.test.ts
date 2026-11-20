import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'
/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
const ctx: Record<string, Any> = {}
for (const f of ['levels.js', 'rules.js', 'progress.js']) {
  runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${f}`, import.meta.url), 'utf8'), ctx)
}
const { XiabanInkLevels: L, XiabanInkBlade: R, XiabanInkProgress: P } = ctx
const dt = 1 / 60

function arena(level = '6-1', x = 1040) {
  const s = R.createGame({ level, seed: 12 })
  const e = s.plan[0]
  s.lock = [e.lock[0] + 40, e.lock[1] - 40]
  s.encounter = 99
  s.player.x = x
  return s
}
function place(s: Any, type: string, x: number, extra: Any = {}) {
  const e = R.spawnEnemy(s, { type, x })
  Object.assign(e, { state: 'idle', t: 0, cd: 99 }, extra)
  return e
}
function run(s: Any, n: number, input: (i: number) => Any = () => ({})) {
  const events: Any[] = []
  for (let i = 0; i < n; i++) events.push(...R.step(s, input(i), dt))
  return events
}

describe('悬山卷', () => {
  it('has six stages, a scroll, six distinct roles and single-condition 技 challenges', () => {
    const levels = L.volumeLevels(6)
    expect(levels.map((l: Any) => l.code)).toEqual([601, 602, 603, 604, 605, 606, 600])
    const types = new Set(Object.values(L.volume(6).encounters).flatMap((e: Any) => e.waves.flat().map((n: Any) => n.type)))
    expect([...types].sort()).toEqual(['crossbow', 'disciple', 'monk', 'umbrella', 'yanhui', 'yue'])
    for (const t of types) expect(R.ENEMY[t as string]).toBeDefined()
    for (const lv of levels) expect(lv.challenge.all).toBeUndefined()
    expect(P.unlocked(P.normalize({ v: 1, volume: 5, levels: {} }), '6-1')).toBe(true)
  })

  it('opens an umbrella on parry, crosses with simplified borrow-step in any direction, then executes from behind', () => {
    const s = arena(), e = place(s, 'umbrella', 1105, { facing: -1, state: 'mountTell', t: 0.01, teleTotal: 0.7, move: 'thrust' })
    run(s, 28, i => ({ slashPressed: i === 0 }))
    expect(e.state).toBe('umbrellaOpen')
    expect(s.player.borrowTarget).toBe(e.id)
    // Pressing dash in any direction (even away) borrows behind the parried foe when landing is valid.
    R.step(s, { move: -1, dashPressed: true }, dt)
    expect(s.player.action.borrow).not.toBeNull()
    run(s, 22)
    expect(s.player.x).toBeGreaterThan(e.x)
    expect(s.player.facing).toBe(-1)
    const events = run(s, 28, i => ({ slashPressed: i === 0 }))
    expect(events.some(ev => ev.type === 'execute')).toBe(true)
    expect(s.player.umbrellaExecutions).toBe(1)
    expect(e.alive).toBe(false)
  })

  it('borrows even during slash recovery or dash cooldown, and falls back to normal dash near a wall', () => {
    const s = arena(), e = place(s, 'umbrella', 1125, { facing: -1, state: 'mountTell', t: 0.01, teleTotal: 0.7, move: 'thrust' })
    s.player.dashCd = 0.45
    run(s, 9, i => ({ slashPressed: i === 0 }))
    expect(R.borrowLanding(s)?.id).toBe(e.id)
    const events = run(s, 48, i => ({ dashPressed: i === 0, slashPressed: i === 2 }))
    expect(events.some(ev => ev.type === 'borrowLand')).toBe(true)
    expect(events.some(ev => ev.type === 'execute')).toBe(true)

    const s2 = arena(), e2 = place(s2, 'umbrella', s2.lock[1] - 20, { facing: -1, state: 'umbrellaOpen', t: 1.4 })
    s2.player.x = e2.x - 65; s2.player.borrowTarget = e2.id; s2.player.borrowUntil = 2
    expect(R.borrowLanding(s2)).toBeNull()
    R.step(s2, { move: 1, dashPressed: true }, dt)
    expect(s2.player.action.borrow).toBeNull()
    expect(s2.player.borrowSteps).toBe(0)
  })

  it('keeps the umbrella front guarded even during the opening', () => {
    const s = arena(), e = place(s, 'umbrella', 1095, { facing: -1, state: 'umbrellaOpen', t: 1.4, openedAt: -1 })
    run(s, 14, i => ({ slashPressed: i === 0 }))
    expect(e.hp).toBe(4)
    expect(e.alive).toBe(true)
  })

  it('can climb from the bridge deck to all three upper landings and land a borrow-step on the first cantilever', () => {
    const s = arena('6-2', 1700)
    s.lock = s.plan[0].lock
    for (const platform of s.world.platforms.filter((p: Any) => p.x >= 1700 && p.x < 2260)) {
      const takeoff = platform.x - 77
      run(s, 65, () => ({ move: s.player.x < takeoff ? 1 : 0 }))
      const target = platform.x + 28
      run(s, 90, i => ({ jumpPressed: i === 0, jump: i < 15, move: Math.sign(target - s.player.x) * (Math.abs(target - s.player.x) > 4 ? 1 : 0) }))
      expect(s.player.onGround).toBe(true)
      expect(s.player.y).toBe(platform.y)
    }

    const s2 = arena('6-2', 1722)
    const guard = place(s2, 'umbrella', 1788, { facing: -1, state: 'mountTell', t: 0.01, teleTotal: 0.7, move: 'thrust' })
    run(s2, 12, i => ({ slashPressed: i === 0 }))
    const landing = R.borrowLanding(s2)
    expect(landing?.id).toBe(guard.id)
    expect(landing.y).toBe(278)
    run(s2, 34, i => ({ dashPressed: i === 0 }))
    expect(s2.player.onGround).toBe(true)
    expect(s2.player.y).toBe(278)
  })

  it('rings the bronze bell to shake down nearby hostile bolts and stagger nearby ground foes', () => {
    const s = arena('6-5', 4755)
    const bolt = (id: number, dx: number, owner: string) => ({
      id, type: 'arrow', originType: 'crossbow', owner,
      x: s.bell.x + dx, y: s.bell.y, vx: 0, vy: 0, life: 2, dead: false,
    })
    s.projectiles.push(bolt(501, 124, 'enemy'), bolt(502, 220, 'enemy'), bolt(503, 110, 'player'))
    const foe = place(s, 'disciple', s.bell.x + 80, { facing: -1 })
    const events = run(s, 17, i => ({ jumpPressed: i === 0, jump: true, slashPressed: i === 3 }))
    expect(events.filter(ev => ev.type === 'boltShaken')).toHaveLength(1)
    expect(s.projectiles.map((shot: Any) => shot.id)).toEqual([502, 503])
    expect(foe.state).toBe('stagger')
    expect(s.player.bellRings).toBe(1)
    expect(s.player.boltDeflects).toBe(0)
  })

  it('moves the sheltered side of the wind corridor posts when the gust reverses', () => {
    const s = arena('6-3', 2700)
    for (const dir of [-1, 1]) {
      s.wind = { phase: 'blow', dir, power: 1 }
      expect(R.windSpeed(s, 2705 + dir * 26)).toBe(0)
      expect(R.windSpeed(s, 2705 - dir * 26)).toBe(dir * s.world.wind.force)
    }
  })

  it('uses independent elite posture for 燕回 without treating him as the final boss', () => {
    const s = arena('6-4', 3700), e = place(s, 'yanhui', 3765, { facing: -1, state: 'mountTell', t: 0.01, teleTotal: 0.65 })
    run(s, 20, i => ({ slashPressed: i === 0 }))
    expect(R.isBoss(e)).toBe(false)
    expect(R.isElite(e)).toBe(true)
    expect(e.posture).toBeGreaterThan(0)
    Object.assign(e, { state: 'broken', t: 2.7, openedAt: s.clock - 1, hp: 8, x: s.player.x + 45 })
    s.player.action = null; s.hitstop = 0; s.slow = 0
    const events = run(s, 20, i => ({ slashPressed: i === 0 }))
    expect(e.alive).toBe(false)
    expect(events.some(ev => ev.type === 'bossDown')).toBe(false)
    expect(s.player.eliteExecutions).toBe(1)
  })

  it('fires two separately telegraphed bolts before reloading and supports deflection returns', () => {
    const s = arena('6-2', 1770), e = place(s, 'crossbow', 1980, { y: 248, homeY: 248, onPlatform: true, facing: -1, cd: 0 })
    run(s, 1)
    expect(e.boltAim.x).toBe(1770)
    s.player.x = 1810
    run(s, 54)
    expect(e.boltAim.x).toBe(1810)
    run(s, 90, () => ({ slashPressed: s.projectiles.some((a: Any) => a.owner === 'enemy' && a.x - s.player.x > 28 && a.x - s.player.x < 72) }))
    expect(s.player.boltDeflects).toBeGreaterThan(0)
    expect(s.player.boltReturns).toBe(1)
    expect(e.hp).toBeLessThan(e.maxHp)
  })

  it('lets a player jump a monk sweep and count a single counter afterwards', () => {
    for (const jump of [false, true]) {
      const s = arena('6-3', 2700)
      place(s, 'monk', 2770, { state: 'mountTell', t: 0.15, teleTotal: 0.9, move: 'sweep', sweep: true, facing: -1 })
      const events = run(s, 18, i => ({ jumpPressed: jump && i === 0, jump, slashPressed: !jump && i === 7 }))
      expect(events.some(ev => ev.type === 'parry')).toBe(false)
      expect(s.player.hp).toBe(s.player.maxHp - (jump ? 0 : 1))
    }
    const s = arena('6-3', 2725), e = place(s, 'monk', 2790, { state: 'mountTell', t: 0.16, teleTotal: 0.98, move: 'sweep', sweep: true, facing: -1 })
    const events = run(s, 62, i => ({ jumpPressed: i === 0, jump: i < 20, slashPressed: i === 32 || i === 54 }))
    expect(events.filter(ev => ev.type === 'sweepCounter')).toHaveLength(1)
    expect(s.player.monkCounters).toBe(1)
    expect(s.player.damageTaken).toBe(0)
    expect(e.hp).toBeLessThan(e.maxHp)
  })

  it('gives 岳停云 two clean phases: parryable strikes in phase 1, adding rush and 危 low sweep in phase 2', () => {
    const s = R.createGame({ level: '6-6', seed: 12 })
    s.player.x = s.plan[0].trigger
    const entry = R.step(s, { move: 1 }, dt)
    const boss = s.enemies[0]
    expect(entry.some((ev: Any) => ev.type === 'bossIntro')).toBe(true)
    expect(boss.state).toBe('intro')
    run(s, 21, () => ({ move: 1 }))
    const firstCut = run(s, 13, i => ({ slashPressed: i === 0 }))
    expect(firstCut.some(ev => ev.type === 'hit' && ev.id === boss.id)).toBe(true)
    expect(boss.hp).toBeLessThan(boss.maxHp)

    // Reaching <= 50% HP triggers phase 2 roar.
    const s2 = arena('6-6', 5720), b2 = place(s2, 'yue', 5770, { facing: -1, state: 'recover', t: 4, hp: 24 })
    const p2Events = run(s2, 20, i => ({ slashPressed: i === 0 }))
    expect(b2.phase2).toBe(true)
    expect(p2Events.some(ev => ev.type === 'bossPhase')).toBe(true)
  })

  it('does not grant a borrowed rear-hit posture bonus twice', () => {
    const s = arena('6-6', 5720), e = place(s, 'yue', 5770, { facing: 1, state: 'recover', t: 4 })
    s.player.borrowMark = { id: e.id, until: 10, used: false }
    run(s, 20, i => ({ slashPressed: i === 0 }))
    const count = s.player.yueBackHits
    run(s, 40, i => ({ slashPressed: i === 10 }))
    expect(count).toBe(1)
    expect(s.player.yueBackHits).toBe(1)
  })

  it('starts a fresh wind warning cycle when entering the corridor late in a scroll', () => {
    const s = arena('6-scroll', 2300); s.lock = [2400, 3120]; s.clock = 107
    run(s, 1); s.player.x = 2400; run(s, 1)
    expect(s.wind.phase).toBe('calm'); expect(R.windSpeed(s, 2400)).toBe(0)
    const events = run(s, Math.ceil((s.world.wind.cycle[0] + 0.2) / dt))
    expect(events.some(ev => ev.type === 'gust')).toBe(true)
    expect(s.wind.phase).toBe('tele')
  })

  const stages = ['6-1', '6-2', '6-3', '6-4', '6-5', '6-6', '6-scroll']
  const playthroughs = stages.flatMap(level => ['easy', 'normal'].map(difficulty => ({ level, difficulty })))

  it.each(playthroughs)('can clear and earn the 技 seal in $level on $difficulty', ({ level, difficulty }) => {
    const s = R.createGame({ level, difficulty, seed: 22 })
    let lastSlash = -99
    for (let i = 0; i < 60 * 460 && s.phase === 'playing'; i++) {
      const p = s.player
      const foes = s.enemies.filter((e: Any) => e.alive && e.delay <= 0 && e.state !== 'spawn' && e.state !== 'dying')
      const opened = foes.find((e: Any) => ['stagger', 'umbrellaOpen', 'broken'].includes(e.state))
      const target = opened || foes.sort((a: Any, b: Any) => Math.abs(a.x - p.x) - Math.abs(b.x - p.x))[0]
      const distance = target ? Math.abs(target.x - p.x) : Infinity
      const toward = Math.sign((target?.x ?? p.x) - p.x)
      const fresh = s.clock - lastSlash > 0.32
      const input: Any = { move: !target ? 1 : distance > 72 ? toward : 0 }
      const slash = (direction: number) => {
        p.facing = direction || p.facing
        input.move = 0
        input.slashPressed = true
        lastSlash = s.clock
      }
      const incoming = foes.find((e: Any) => ['mountTell', 'mountAttack'].includes(e.state) && !e.sweep && !e.hitDone &&
        (e.state === 'mountAttack' || e.t < 0.055) && Math.abs(e.y - p.y) < 40 &&
        R.overlaps(R.summitReach(e), { x0: p.x - 20, x1: p.x + 20, y0: p.y - 54, y1: p.y }))
      const sweep = foes.find((e: Any) => e.state === 'mountTell' && e.sweep && e.t < 0.2 && Math.abs(e.x - p.x) < 175)
      const bolt = s.projectiles.find((a: Any) => a.owner === 'enemy' && !a.dead && Math.abs(a.x - p.x) < 80 && Math.abs(a.y - (p.y - 32)) < 42)
      const needParry = (level === '6-5' && p.parries < 8) || (level === '6-6' && p.bossExecutions < 2)

      if (p.action?.type === 'dash') input.move = 0
      else if (sweep && p.onGround) { input.move = 0; input.jumpPressed = true; input.jump = true }
      else if (R.borrowLanding(s) && opened?.state !== 'broken' && (!needParry || target?.type === 'umbrella')) { input.move = 0; input.dashPressed = true }
      else if (incoming && fresh) slash(Math.sign(incoming.x - p.x))
      else if (bolt && fresh) slash(Math.sign(bolt.x - p.x))
      else if (target?.type === 'monk' && level === '6-3' && p.monkCounters < 2 && target.state !== 'recover' && target.state !== 'stagger') {
        input.move = distance > 82 ? toward : 0
      } else if (target?.type === 'yanhui' && p.yanBackHits < 2 && !p.borrowMark && target.state !== 'broken') {
        input.move = distance > 81 ? toward : 0
      } else if (target?.type === 'yue' && p.bossExecutions < 2 && target.state !== 'broken') {
        // Hold back on raw damage so Yue gets posture-broken and executed twice.
        input.move = distance > 86 ? toward : 0
      } else if (target && ['stagger', 'umbrellaOpen', 'broken', 'recover'].includes(target.state) && distance < 82 && Math.abs(target.y - p.y) < 72 && fresh) {
        if (target.type !== 'monk' || !target.sweep || target.sweepEvaded || level !== '6-3' || p.monkCounters >= 2) slash(toward)
      } else if (target?.type === 'crossbow') {
        if ((level === '6-2' && p.boltDeflects < 3) || (level === '6-5' && p.parries < 8)) {
          const x = target.x - toward * 145
          input.move = Math.abs(x - p.x) > 10 ? Math.sign(x - p.x) : 0
        } else if (distance < 110) {
          if (p.onGround && target.y < p.y - 20) { input.jumpPressed = true; input.jump = true }
          if (distance < 80 && Math.abs(target.y - p.y) < 65 && fresh) slash(toward)
        }
      }
      if (p.onGround && target && p.y < target.y - 40 && target.type !== 'crossbow') {
        const perch = s.world.platforms.find((v: Any) => v.y === p.y && p.x >= v.x - p.w && p.x <= v.x + v.w + p.w)
        if (perch) { input.move = p.x < perch.x + perch.w / 2 ? -1 : 1; input.jump = false; input.jumpPressed = false }
      }
      if (!p.onGround && p.vy < 0) input.jump = true
      if (p.ink >= 100 && target?.state !== 'broken' && !needParry &&
          (level === '6-scroll' || (foes.length >= 2 && R.challenge(s).met))) input.inkPressed = true
      R.step(s, input, dt)
    }
    const result = R.result(s)
    if (process.env.INK_PLAYTEST_REPORT) console.info('REPORT', JSON.stringify({ level, difficulty, sec: result.seconds, score: result.score, grade: result.grade, seals: result.seals, parries: result.parries, exec: result.executions, dmg: result.damageTaken }))
    expect(result.cleared, JSON.stringify({ level, difficulty, time: s.time, hp: s.player.hp, kills: s.player.kills, challenge: R.challenge(s) })).toBe(true)
    expect(result.seals & 4).toBe(4)
  })

  it.each(['6-1', '6-2', '6-3', '6-4', '6-5', '6-6'])('does not give a normal-mode mash bot 神品 or higher in %s', level => {
    const s = R.createGame({ level, seed: 8 })
    for (let i = 0; i < 60 * 180 && s.phase === 'playing'; i++) {
      const e = s.enemies.find((en: Any) => en.alive && en.delay <= 0)
      R.step(s, { move: e ? Math.sign(e.x - s.player.x) : 1, slashPressed: i % 6 === 0 }, dt)
    }
    expect(R.evaluate(s).grade).toBeLessThan(3)
  })
})
