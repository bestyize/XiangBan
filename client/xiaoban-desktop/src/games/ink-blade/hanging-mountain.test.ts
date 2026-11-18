import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'
/* eslint-disable @typescript-eslint/no-explicit-any */
type Any=any
const ctx:Record<string,Any>={}
for(const f of ['levels.js','rules.js','progress.js'])runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${f}`,import.meta.url),'utf8'),ctx)
const {XiabanInkLevels:L,XiabanInkBlade:R,XiabanInkProgress:P}=ctx,dt=1/60
function arena(level='6-1',x=1040){const s=R.createGame({level,seed:12});const e=s.plan[0];s.lock=[e.lock[0]+40,e.lock[1]-40];s.encounter=99;s.player.x=x;return s}
function place(s:Any,type:string,x:number,extra:Any={}){const e=R.spawnEnemy(s,{type,x});Object.assign(e,{state:'idle',t:0,cd:99},extra);return e}
function run(s:Any,n:number,input:(i:number)=>Any=()=>({})){const events:Any[]=[];for(let i=0;i<n;i++)events.push(...R.step(s,input(i),dt));return events}
describe('悬山卷',()=>{
 it('has six stages, a scroll and eleven roles including airborne and terrain threats',()=>{
  expect(L.volumeLevels(6).map((l:Any)=>l.code)).toEqual([601,602,603,604,605,606,600])
  const types=new Set(Object.values(L.volume(6).encounters).flatMap((e:Any)=>e.waves.flat().map((n:Any)=>n.type)))
  expect(types.size).toBe(11)
  for(const t of types)expect(R.ENEMY[t as string]).toBeDefined()
  expect(P.unlocked(P.normalize({v:1,volume:5,levels:{}}),'6-1')).toBe(true)
 })
 it('opens an umbrella on parry, crosses continuously, then executes from behind',()=>{
  const s=arena(),e=place(s,'umbrella',1105,{facing:-1,state:'mountTell',t:.01,teleTotal:.7,move:'thrust'})
  run(s,28,i=>({slashPressed:i===0}))
  expect(e.state).toBe('umbrellaOpen');expect(s.player.borrowTarget).toBe(e.id)
  const before=s.player.x
  R.step(s,{move:1,dashPressed:true},dt)
  expect(s.player.action.borrow).not.toBeNull()
  expect(s.player.x-before).toBeGreaterThan(0);expect(s.player.x-before).toBeLessThan(25)
  run(s,22)
  expect(s.player.x).toBeGreaterThan(e.x);expect(s.player.facing).toBe(-1)
  const events=run(s,28,i=>({slashPressed:i===0}))
  expect(events.some(e=>e.type==='execute')).toBe(true)
  expect(s.player.umbrellaExecutions).toBe(1);expect(e.alive).toBe(false)
 })
 it('uses a neutral dash after a parry, even during slash recovery or dash cooldown',()=>{
  const s=arena(),e=place(s,'umbrella',1125,{facing:-1,state:'mountTell',t:.01,teleTotal:.7,move:'thrust'})
  s.player.dashCd=.45
  run(s,9,i=>({slashPressed:i===0}))
  expect(R.borrowLanding(s)?.id).toBe(e.id)
  const events=run(s,48,i=>({dashPressed:i===0,slashPressed:i===2}))
  expect(events.some(e=>e.type==='borrowLand')).toBe(true)
  expect(events.some(e=>e.type==='execute')).toBe(true)
  expect(s.player.umbrellaExecutions).toBe(1)
 })
 it('keeps the umbrella front guarded even during the opening',()=>{
  const s=arena(),e=place(s,'umbrella',1095,{facing:-1,state:'umbrellaOpen',t:1.4,openedAt:-1})
  run(s,14,i=>({slashPressed:i===0}));expect(e.hp).toBe(4);expect(e.alive).toBe(true)
 })
 it('does not auto-move after parry and falls back when no safe destination exists',()=>{
  const s=arena(),e=place(s,'umbrella',s.lock[1]-20,{facing:-1,state:'umbrellaOpen',t:1.4})
  s.player.x=e.x-65;s.player.borrowTarget=e.id;s.player.borrowUntil=2
  const x=s.player.x;run(s,5);expect(Math.abs(s.player.x-x)).toBeLessThan(1)
  expect(R.borrowLanding(s)).toBeNull()
  R.step(s,{move:1,dashPressed:true},dt);expect(s.player.action.borrow).toBeNull();expect(s.player.borrowSteps).toBe(0)
 })
 it('keeps a retreat a normal dash even with a clear borrow-step landing',()=>{
  const s=arena(),e=place(s,'umbrella',1100,{facing:-1,state:'umbrellaOpen',t:1.4})
  s.player.borrowTarget=e.id;s.player.borrowUntil=2
  expect(R.borrowLanding(s)).not.toBeNull()
  R.step(s,{move:-1,dashPressed:true},dt)
  expect(s.player.action.borrow).toBeNull()
  expect(s.player.x).toBeLessThan(1040)
 })
 it('can climb from the bridge deck to all three upper landings', () => {
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
 })

 it('rewards a descending cut from a ledge only against a lower unguarded target', () => {
  function cut(fromLedge: boolean) {
   const s = arena('6-2', 1883)
   Object.assign(s.player, { onGround: false, y: 253, vy: 145, perchY: fromLedge ? 248 : null })
   const target = place(s, 'disciple', 1920, { state: 'recover', t: 2 })
   const events = run(s, 16, i => ({ slashPressed: i === 0 }))
   return { damage: target.maxHp - target.hp, plunge: events.some(e => e.type === 'plungeHit') }
  }
  const normal = cut(false), plunging = cut(true)
  expect(normal.plunge).toBe(false)
  expect(plunging).toEqual({ damage: normal.damage + 1, plunge: true })
 })

 it('leaves the heavy sword stuck facing the old line only after borrowing past a second-phase heavy blow', () => {
  for (const [phase2, move, shouldPlant] of [[true, 'cleave', true], [true, 'rush', true], [true, 'lift', false], [false, 'cleave', false]] as const) {
   const s = arena('6-6', 5680)
   const boss = place(s, 'yue', 5745, { phase2, state: 'mountTell', t: .01, teleTotal: .7, move, facing: -1 })
   run(s, 9, i => ({ slashPressed: i === 0 }))
   expect(R.borrowLanding(s)?.id).toBe(boss.id)
   const crossing = run(s, 50, i => ({ dashPressed: i === 0 }))
   expect(crossing.filter(e => e.type === 'swordStone')).toHaveLength(shouldPlant ? 1 : 0)
   if (shouldPlant) {
    expect(boss.facing).toBe(-1)
    expect(s.player.x).toBeGreaterThan(boss.x)
    expect(s.player.yueBackHits).toBe(0)
    run(s, 30, i => ({ slashPressed: i === 0 }))
    expect(s.player.yueBackHits).toBe(1)
   }
  }
 })

 it('shakes down only nearby hostile bolts without crediting a sword deflection', () => {
  const s = arena('6-5', 4755)
  const bolt = (id: number, dx: number, owner: string) => ({ id, type: 'arrow', originType: 'crossbow', owner,
   x: s.bell.x + dx, y: s.bell.y, vx: 0, vy: 0, life: 2, dead: false })
  s.projectiles.push(bolt(501, 124, 'enemy'), bolt(502, 220, 'enemy'), bolt(503, 110, 'player'))
  const events = run(s, 17, i => ({ jumpPressed: i === 0, jump: true, slashPressed: i === 3 }))
  expect(events.filter(e => e.type === 'boltShaken')).toHaveLength(1)
  expect(s.projectiles.map((shot: Any) => shot.id)).toEqual([502, 503])
  expect(s.player.boltDeflects).toBe(0)
 })

 it('locks a falling rock to its warning and lets a dodged rock hit the pursuer', () => {
  const s = arena('6-1', 1050)
  place(s, 'quarrier', 1280, { cd: 0 })
  run(s, 2)
  expect(s.mountainFalls).toHaveLength(1)
  const x = s.mountainFalls[0].x
  const pursuer = place(s, 'disciple', x, { state: 'recover', t: 3 })
  s.player.x = x - 94
  const events = run(s, 100)
  expect(events.filter(e => e.type === 'rockImpact')).toHaveLength(1)
  expect(pursuer.hp).toBe(pursuer.maxHp - 2)
  expect(s.player.damageTaken).toBe(0)
 })

 it('lets the gate stone pin an umbrella and grant an actual crossing', () => {
  const s = arena('6-1', 1040), guard = place(s, 'umbrella', 1104, { facing: -1 })
  s.mountainFalls.push({ owner: 501, x: guard.x, age: 1, warning: 1, landed: false })
  run(s, 1)
  expect(guard.state).toBe('umbrellaOpen')
  expect(guard.hp).toBe(guard.maxHp)
  expect(R.borrowLanding(s)?.id).toBe(guard.id)
  run(s, 24, i => ({ dashPressed: i === 0 }))
  expect(s.player.x).toBeGreaterThan(guard.x)
 })

 it('lands the bridge parry on the same upper plank shown by its footprint', () => {
  const s = arena('6-2', 1722)
  const guard = place(s, 'umbrella', 1788, { facing: -1, state: 'mountTell', t: .01, teleTotal: .7, move: 'thrust' })
  run(s, 12, i => ({ slashPressed: i === 0 }))
  const landing = R.borrowLanding(s)
  expect(landing?.id).toBe(guard.id)
  expect(landing.y).toBe(278)
  run(s, 34, i => ({ dashPressed: i === 0 }))
  expect(s.player.onGround).toBe(true)
  expect(s.player.y).toBe(landing.y)
  expect(s.player.x).toBeCloseTo(landing.x)
 })

 it('counts a parry only on the windward side during a gust', () => {
  for (const [x, count] of [[2720, 0], [2670, 1]]) {
   const s = arena('6-3', x)
   s.clock = 6; s.windSince = 0
   place(s, 'disciple', x + 60, { facing: -1, state: 'mountTell', t: .01, teleTotal: .7, move: 'thrust' })
   run(s, 12, i => ({ slashPressed: i === 0 }))
   expect(s.player.parries).toBe(1)
   expect(s.player.windParries).toBe(count)
  }
 })

 it('uses the bell to cancel a windup and denies interrupt credit while the hammer is idle', () => {
  for (const ready of [true, false]) {
   const s = arena('6-5', 4755)
   const hammer = place(s, 'shizhen', 4843, { state: ready ? 'mountTell' : 'idle', t: 1.4, move: 'slam', sweep: true })
   run(s, 17, i => ({ jumpPressed: i === 0, jump: true, slashPressed: i === 3 }))
   expect(s.player.bellInterrupts).toBe(ready ? 1 : 0)
   expect(s.player.bellRings).toBe(1)
   if (ready) expect(hammer.state).toBe('recover')
   expect(s.bellWave).toBeNull()
  }
 })

 it('can break Yue with his own stone while leaving the execution to the player', () => {
  const s = arena('6-6', 5640)
  const boss = place(s, 'yue', 5740, { phase2: true, state: 'recover', t: 1.1, posture: 82 })
  s.mountainFalls.push({ owner: boss.id, x: boss.x, age: 1, warning: 1, landed: false })
  run(s, 1)
  expect(boss.state).toBe('broken')
  expect(boss.hp).toBe(boss.maxHp)
  expect(s.player.bossExecutions).toBe(0)
 })

 it('moves the sheltered side of the posts when the gust reverses', () => {
  const s = arena('6-3', 2700)
  for (const dir of [-1, 1]) {
   s.wind = { phase: 'blow', dir, power: 1 }
   expect(R.windSpeed(s, 2705 + dir * 26)).toBe(0)
   expect(R.windSpeed(s, 2705 - dir * 26)).toBe(dir * s.world.wind.force)
  }
 })

 it('delays the hostile bell wave and lets a high platform clear the crest', () => {
  for (const elevated of [false, true]) {
   const s = arena('6-5', 4882)
   if (elevated) s.player.y = 218
   const boss = place(s, 'shizhen', 4680, { move: 'slam', state: 'mountTell', sweep: true, t: .001, facing: -1 })
   run(s, 26)
   expect(s.bellWave).not.toBeNull()
   expect(s.player.damageTaken).toBe(0)
   run(s, 52)
   expect(s.player.damageTaken).toBe(elevated ? 0 : 1)
   expect(boss.alive).toBe(true)
  }
 })

 it('uses independent elite posture without treating the elite as the final boss',()=>{
  const s=arena('6-4',3700),e=place(s,'yanhui',3765,{facing:-1,state:'mountTell',t:.01,teleTotal:.65})
  run(s,20,i=>({slashPressed:i===0}))
  expect(R.isBoss(e)).toBe(false);expect(R.isElite(e)).toBe(true);expect(e.posture).toBeGreaterThan(0);expect(e.state).not.toBe('stagger')
  Object.assign(e,{state:'broken',t:2.7,openedAt:s.clock-1,hp:8,x:s.player.x+45});s.player.action=null;s.hitstop=0;s.slow=0
  const events=run(s,20,i=>({slashPressed:i===0}))
  expect(e.alive).toBe(false);expect(events.some(e=>e.type==='bossDown')).toBe(false);expect(s.player.eliteExecutions).toBe(1)
 })
 it('fires two separately telegraphed bolts before a long reload',()=>{
  const s=arena('6-2',1650),e=place(s,'crossbow',1930,{facing:-1,cd:0})
  const events=run(s,100)
  expect(events.filter(e=>e.type==='shoot')).toHaveLength(2)
  expect(events.filter(e=>e.type==='tell')).toHaveLength(2)
  expect(e.state).toBe('recover')
 })
 it('gives the second bolt a new aim and leaves the elevated archer alive after one return',()=>{
  const s=arena('6-2',1770),e=place(s,'crossbow',1980,{y:248,homeY:248,onPlatform:true,facing:-1,cd:0})
  run(s,1)
  expect(e.boltAim.x).toBe(1770)
  s.player.x=1810
  run(s,54)
  expect(e.boltAim.x).toBe(1810)
  run(s,90,()=>({slashPressed:s.projectiles.some((a:Any)=>a.owner==='enemy'&&a.x-s.player.x>28&&a.x-s.player.x<72)}))
  expect(s.player.boltDeflects).toBeGreaterThan(0)
  expect(s.player.boltReturns).toBe(1)
  expect(e.alive).toBe(true)
  expect(e.hp).toBeLessThan(e.maxHp)
 })
 it('lets a player jump a monk sweep but cannot parry it',()=>{
  for(const jump of [false,true]){
   const s=arena('6-3',2700);place(s,'monk',2770,{state:'mountTell',t:.15,teleTotal:.9,move:'sweep',sweep:true,facing:-1})
   const events=run(s,18,i=>({jumpPressed:jump&&i===0,jump,slashPressed:!jump&&i===7}))
   expect(events.some(e=>e.type==='parry')).toBe(false);expect(s.player.hp).toBe(s.player.maxHp-(jump?0:1))
  }
 })
 it('evaluates both requirements of combined seals',()=>{
  const s=arena('6-2',1650);s.player.ropeParries=1;s.player.boltDeflects=1
  expect(R.challenge(s).met).toBe(false);s.player.boltDeflects=2;expect(R.challenge(s).met).toBe(true)
 })
 it('counts a single counter after jumping over a monk sweep',()=>{
  const s=arena('6-3',2725),e=place(s,'monk',2790,{state:'mountTell',t:.16,teleTotal:.98,move:'sweep',sweep:true,facing:-1})
  const events=run(s,62,i=>({jumpPressed:i===0,jump:i<20,slashPressed:i===32||i===54}))
  expect(events.filter(e=>e.type==='sweepCounter')).toHaveLength(1)
  expect(s.player.monkCounters).toBe(1)
  expect(s.player.damageTaken).toBe(0)
  expect(e.hp).toBeLessThan(e.maxHp)
 })
 it('rings the bell once per cooldown and breaks Shi Zhen’s posture without dealing direct damage',()=>{
  const s=arena('6-5',4755),e=place(s,'shizhen',4910,{posture:51,facing:-1})
  const events=run(s,40,i=>({jumpPressed:i===0,jump:i<18,slashPressed:i===3||i===30}))
  expect(events.filter(e=>e.type==='mountainBell'&&e.effective)).toHaveLength(1)
  expect(s.player.bellRings).toBe(1)
  expect(e.state).toBe('broken')
  expect(e.hp).toBe(e.maxHp)
 })
 it('requires a posture execution to finish Yue, even at one health',()=>{
  const s=arena('6-6',5720),e=place(s,'yue',5770,{facing:-1,state:'recover',t:4,hp:1,phase2:true})
  run(s,22,i=>({slashPressed:i===0}))
  expect(e.hp).toBe(1)
  expect(e.state).not.toBe('dying')
  Object.assign(e,{state:'broken',t:2.7,openedAt:s.clock-1})
  run(s,22,i=>({slashPressed:i===0}))
  expect(e.state).toBe('dying')
  expect(s.player.bossExecutions).toBe(1)
 })
 it('lets the player land a frontal hit during Yue’s entrance', () => {
  const s = R.createGame({ level: '6-6', seed: 12 })
  s.player.x = s.plan[0].trigger
  const entry = R.step(s, { move: 1 }, dt)
  const boss = s.enemies[0]
  expect(entry.some((e: Any) => e.type === 'bossIntro')).toBe(true)
  expect(boss.state).toBe('intro')
  run(s, 21, () => ({ move: 1 }))
  expect(s.player.x).toBeLessThan(boss.x)
  const firstCut = run(s, 13, i => ({ slashPressed: i === 0 }))
  expect(firstCut.some(e => e.type === 'hit' && e.id === boss.id)).toBe(true)
  expect(boss.hp).toBeLessThan(boss.maxHp)
 })
 const stages = ['6-1', '6-2', '6-3', '6-4', '6-5', '6-6', '6-scroll']
 const playthroughs = stages.flatMap(level => ['easy', 'normal'].map(difficulty => ({ level, difficulty })))

 it.each(playthroughs)('can earn the technique seal in $level on $difficulty', ({ level, difficulty }) => {
  const s = R.createGame({ level, difficulty, seed: 22 })
  let lastSlash = -99
  for (let i = 0; i < 60 * 500 && s.phase === 'playing'; i++) {
   const p = s.player
   const foes = s.enemies.filter((e: Any) => e.alive && e.delay <= 0 && e.state !== 'spawn' && e.state !== 'dying')
   const opened = foes.find((e: Any) => ['stagger', 'umbrellaOpen', 'broken'].includes(e.state))
   const target = opened || foes.sort((a: Any, b: Any) => Math.abs(a.x - p.x) - Math.abs(b.x - p.x))[0]
   const distance = target ? Math.abs(target.x - p.x) : Infinity
   const toward = Math.sign((target?.x ?? p.x) - p.x)
   const fresh = s.clock - lastSlash > .32
   const input: Any = { move: !target ? 1 : distance > 72 ? toward : 0 }
   const slash = (direction: number) => {
    p.facing = direction || p.facing
    input.move = 0
    input.slashPressed = true
    lastSlash = s.clock
   }
   const incoming = foes.find((e: Any) => ['mountTell', 'mountAttack'].includes(e.state) && !e.sweep && !e.hitDone &&
    (e.state === 'mountAttack' || e.t < .055) && Math.abs(e.y - p.y) < 40 &&
    R.overlaps(R.summitReach(e), { x0: p.x - 20, x1: p.x + 20, y0: p.y - 54, y1: p.y }))
   const sweep = foes.find((e: Any) => e.state === 'mountTell' && e.sweep && e.t < .2 && Math.abs(e.x - p.x) < 175)
   const dive = foes.find((e: Any) => ['ropeDive', 'dive'].includes(e.state) && !e.hitDone && e.y > p.y - 68 && Math.abs(e.x - p.x) < 72)
   const rock = s.mountainFalls.find((f: Any) => !f.landed && f.warning - f.age < .43 && Math.abs(p.x - f.x) < 56)
   const echo = s.bellWave && Math.abs(Math.abs(p.x - s.bellWave.x) - s.bellWave.radius) < 54 && s.bellWave.t > .43
   const bolt = s.projectiles.find((a: Any) => a.owner === 'enemy' && !a.dead && Math.abs(a.x - p.x) < 80 && Math.abs(a.y - (p.y - 32)) < 42)
   const wantsBell = s.bell && target?.type === 'shizhen' && p.bellRings === 0 &&
    Math.abs(target.x - s.bell.x) < s.bell.radius && target.state !== 'broken'

   if (p.action?.type === 'dash') input.move = 0
   else if (rock && p.iframes <= 0) {
    const roomRight = s.lock[1] - p.x > 78, roomLeft = p.x - s.lock[0] > 78
    input.move = roomRight && (!roomLeft || p.x >= rock.x) ? 1 : -1
    input.dashPressed = p.dashCd <= 0
   }
   else if ((sweep || echo) && p.onGround) { input.move = 0; input.jumpPressed = true; input.jump = true }
   else if (R.borrowLanding(s) && opened?.state !== 'broken') { input.move = 0; input.dashPressed = true }
   else if ((incoming || dive) && fresh) slash(Math.sign((incoming || dive).x - p.x))
   else if (bolt && fresh) slash(Math.sign(bolt.x - p.x))
   else if (wantsBell) {
    const x = s.bell.x - 30
    input.move = Math.abs(x - p.x) > 13 ? Math.sign(x - p.x) : 0
    if (!input.move && p.onGround) { input.jumpPressed = true; input.jump = true }
    if (fresh && !p.onGround && p.y < 280 && p.y > 230 && Math.abs(p.x - s.bell.x) < 62) slash(Math.sign(s.bell.x - p.x))
   } else if (target?.type === 'monk' && level === '6-3' && p.windParries === 0 && p.monkCounters > 0 && target.state !== 'stagger') {
    input.move = distance > 82 ? toward : 0
   } else if (target?.type === 'yanhui' && p.yanBackHits < 2 && !p.borrowMark && target.state !== 'broken') {
    input.move = distance > 81 ? toward : 0
   } else if (target && ['stagger', 'umbrellaOpen', 'broken', 'recover'].includes(target.state) && distance < 82 && Math.abs(target.y - p.y) < 72 && fresh) {
    if (target.type !== 'monk' || !target.sweep || !p.onGround || target.sweepEvaded || p.windParries > 0) slash(toward)
   } else if (target?.type === 'crossbow') {
    if (p.boltDeflects < 2) {
     const x = target.x - toward * 138
     input.move = Math.abs(x - p.x) > 10 ? Math.sign(x - p.x) : 0
    } else if (distance < 110) {
     if (p.onGround && target.y < p.y - 20) { input.jumpPressed = true; input.jump = true }
     if (distance < 80 && Math.abs(target.y - p.y) < 65 && fresh) slash(toward)
    }
   }
   // Get off an archer's perch after the archer goes down. Walk past its nearest edge.
   if (p.onGround && target && p.y < target.y - 40 && target.type !== 'crossbow' && !wantsBell) {
    const perch = s.world.platforms.find((v: Any) => v.y === p.y && p.x >= v.x - p.w && p.x <= v.x + v.w + p.w)
    if (perch) { input.move = p.x < perch.x + perch.w / 2 ? -1 : 1; input.jump = false; input.jumpPressed = false }
   }
   if (!p.onGround && p.vy < 0) input.jump = true
   if (p.ink >= 100 && target?.state !== 'broken' && !wantsBell &&
       (level === '6-scroll' || (foes.length >= 2 && R.challenge(s).met))) input.inkPressed = true
   R.step(s, input, dt)
  }
  const result = R.result(s)
  expect(result.cleared, JSON.stringify({ level, difficulty, time: s.time, hp: s.player.hp, kills: s.player.kills, challenge: R.challenge(s) })).toBe(true)
  expect(result.seals & 4).toBe(4)
  if (process.env.INK_PLAYTEST_REPORT) console.info({ level, difficulty, seconds: result.seconds, score: result.score, grade: result.grade, seals: result.seals })
 })
 it('counts a parried rope dive and brings the fighter safely to the deck',()=>{
  const s=arena('6-2',1750),e=place(s,'rope',1800,{state:'ropeDive',t:.2,ropeMounted:true,y:R.GROUND-25,vx:-120,vy:150,facing:-1})
  run(s,16,i=>({slashPressed:i===0}))
  expect(s.player.ropeParries).toBe(1);expect(e.y).toBe(R.GROUND);expect(e.state).toBe('stagger')
 })
 it('does not grant a borrowed rear-hit posture bonus twice',()=>{
  const s=arena('6-6',5720),e=place(s,'yue',5770,{facing:1,state:'recover',t:4})
  s.player.borrowMark={id:e.id,until:10,used:false}
  run(s,20,i=>({slashPressed:i===0}));const count=s.player.yueBackHits
  run(s,40,i=>({slashPressed:i===10}))
  expect(count).toBe(1);expect(s.player.yueBackHits).toBe(1)
 })

 it.each(['6-1','6-4','6-6'])('does not give a normal-mode mash bot a high grade in %s',level=>{
  const s=R.createGame({level,seed:8})
  for(let i=0;i<60*180&&s.phase==='playing';i++){
   const e=s.enemies.find((e:Any)=>e.alive&&e.delay<=0)
   R.step(s,{move:e?Math.sign(e.x-s.player.x):1,slashPressed:i%6===0},dt)
  }
  expect(R.evaluate(s).grade).toBeLessThan(3)
 })

 it('starts a fresh wind warning cycle when entering the corridor late in a scroll',()=>{
  const s=arena('6-scroll',2300);s.lock=[2400,3120];s.clock=107
  run(s,1);s.player.x=2400;run(s,1)
  expect(s.wind.phase).toBe('calm');expect(R.windSpeed(s,2400)).toBe(0)
  const events=run(s,Math.ceil((s.world.wind.cycle[0]+.2)/dt))
  expect(events.some(e=>e.type==='gust')).toBe(true)
  expect(s.wind.phase).toBe('tele')
 })

})
