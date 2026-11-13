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
 it('has six stages, a scroll and all nine distinct roles',()=>{
  expect(L.volumeLevels(6).map((l:Any)=>l.code)).toEqual([601,602,603,604,605,606,600])
  const types=new Set(Object.values(L.volume(6).encounters).flatMap((e:Any)=>e.waves.flat().map((n:Any)=>n.type)))
  expect(types.size).toBe(9)
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
   const dive = foes.find((e: Any) => e.state === 'ropeDive' && e.y > p.y - 65 && Math.abs(e.x - p.x) < 80)
   const bolt = s.projectiles.find((a: Any) => a.owner === 'enemy' && !a.dead && Math.abs(a.x - p.x) < 80 && Math.abs(a.y - (p.y - 32)) < 42)
   const wantsBell = s.bell && target?.type === 'shizhen' && p.bellRings === 0 &&
    Math.abs(target.x - s.bell.x) < s.bell.radius && target.state !== 'broken'

   if (p.action?.type === 'dash') input.move = 0
   else if (sweep && p.onGround) { input.move = 0; input.jumpPressed = true; input.jump = true }
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
   if (!p.onGround && p.vy < 0) input.jump = true
   if (p.ink >= 100 && level === '6-scroll' && target?.state !== 'broken' && !wantsBell) input.inkPressed = true
   R.step(s, input, dt)
  }
  const result = R.result(s)
  expect(result.cleared, JSON.stringify({ level, difficulty, time: s.time, hp: s.player.hp, kills: s.player.kills })).toBe(true)
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
