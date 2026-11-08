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
 it('keeps the umbrella front guarded even during the opening',()=>{
  const s=arena(),e=place(s,'umbrella',1095,{facing:-1,state:'umbrellaOpen',t:1.4,openedAt:-1})
  run(s,14,i=>({slashPressed:i===0}));expect(e.hp).toBe(4);expect(e.alive).toBe(true)
 })
 it('does not auto-move after parry and falls back when no safe destination exists',()=>{
  const s=arena(),e=place(s,'umbrella',s.lock[1]-20,{facing:-1,state:'umbrellaOpen',t:1.4})
  s.player.x=e.x-65;s.player.borrowTarget=e.id;s.player.borrowUntil=2
  const x=s.player.x;run(s,5);expect(Math.abs(s.player.x-x)).toBeLessThan(1)
  R.step(s,{move:1,dashPressed:true},dt);expect(s.player.action.borrow).toBeNull();expect(s.player.borrowSteps).toBe(0)
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
 it.each(['6-1','6-2','6-3','6-4','6-5','6-6','6-scroll'])('can complete %s with actual counters and no health override',level=>{
  const s=R.createGame({level,difficulty:'easy',seed:22});let last=-99
  for(let i=0;i<60*(level==='6-scroll'?600:300)&&s.phase==='playing';i++){
   const p=s.player,foes=s.enemies.filter((e:Any)=>e.alive&&e.delay<=0&&e.state!=='spawn'&&e.state!=='dying')
   const opened=foes.find((e:Any)=>['stagger','umbrellaOpen','broken'].includes(e.state))
   const target=opened||foes.sort((a:Any,b:Any)=>Math.abs(a.x-p.x)-Math.abs(b.x-p.x))[0]
   const dist=target?Math.abs(target.x-p.x):999,fresh=s.clock-last>.34
   const input:Any={move:!target?1:dist>65?Math.sign(target.x-p.x):0}
   const slash=()=>{input.slashPressed=true;last=s.clock}
   const danger=foes.find((e:Any)=>e.state==='mountTell'&&e.sweep&&e.t<.2&&Math.abs(e.x-p.x)<170)
   const incoming=foes.find((e:Any)=>e.state==='mountTell'&&!e.sweep&&e.t<.055&&Math.abs(e.x-p.x)<R.ENEMY[e.type].reach+8&&Math.abs(e.y-p.y)<40)
   const dive=foes.find((e:Any)=>e.state==='ropeDive'&&e.y>p.y-65&&Math.abs(e.x-p.x)<80)
   const arrow=s.projectiles.find((a:Any)=>a.owner==='enemy'&&!a.dead&&Math.abs(a.x-p.x)<80&&Math.abs(a.y-(p.y-32))<43)
   const borrow=foes.find((e:Any)=>e.id===p.borrowTarget&&Math.abs(e.x-p.x)<115&&Math.abs(e.y-p.y)<5)
   if(p.action?.type==='dash')input.move=p.action.borrow?.dir||p.facing
   else if(danger&&p.onGround){input.jumpPressed=true;input.jump=true}
   else if(borrow&&p.borrowUntil>s.clock&&p.onGround&&p.dashCd<=0){input.move=Math.sign(borrow.x-p.x);input.dashPressed=true}
   else if((incoming||dive)&&fresh){const e=incoming||dive;p.facing=Math.sign(e.x-p.x)||p.facing;slash()}
   else if(arrow&&fresh){p.facing=Math.sign(arrow.x-p.x)||p.facing;input.move=0;slash()}
   else if(target&&['stagger','umbrellaOpen','broken','recover'].includes(target.state)&&dist<82&&Math.abs(target.y-p.y)<72&&fresh){p.facing=Math.sign(target.x-p.x)||p.facing;slash()}
   else if(target?.type==='crossbow'&&dist<110){if(p.onGround&&target.y<p.y-20){input.jumpPressed=true;input.jump=true}if(dist<80&&Math.abs(target.y-p.y)<65&&fresh){p.facing=Math.sign(target.x-p.x)||p.facing;slash()}}
   if(!p.onGround&&p.vy<0)input.jump=true
   if(p.ink>=100&&target?.state!=='broken')input.inkPressed=true
   R.step(s,input,dt)
  }
  expect(s.reason,JSON.stringify({level,time:s.time,hp:s.player.hp,kills:s.player.kills,parries:s.player.parries,borrow:s.player.borrowSteps,x:s.player.x,foes:s.enemies.map((e:Any)=>({type:e.type,hp:e.hp,x:e.x,y:e.y,state:e.state,cd:e.cd}))})).toBe('clear')
  if(process.env.INK_PLAYTEST_REPORT)console.info(JSON.stringify({level,seconds:Math.round(s.time),hp:s.player.hp,score:s.player.score,parries:s.player.parries,borrow:s.player.borrowSteps}))
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
  const events=run(s,350)
  expect(events.some(e=>e.type==='gust')).toBe(true)
  expect(s.wind.phase).toBe('tele')
 })

})
