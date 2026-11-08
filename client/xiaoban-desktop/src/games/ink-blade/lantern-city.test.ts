import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'
/* eslint-disable @typescript-eslint/no-explicit-any */
type Any = any
const context: Record<string, Any> = {}
for (const file of ['levels.js','rules.js','progress.js']) runInNewContext(readFileSync(new URL(`../../../public/game-packs/ink-blade/${file}`,import.meta.url),'utf8'),context)
const {XiabanInkLevels:levels,XiabanInkBlade:rules,XiabanInkProgress:progress}=context
const dt=1/60
function arena(level='5-3',x=2700){const s=rules.createGame({level,seed:11});const e=s.plan[0];s.lock=[e.lock[0]+40,e.lock[1]-40];s.encounter=99;s.player.x=x;return s}
function place(s:Any,type:string,x:number,extra:Any={}){const e=rules.spawnEnemy(s,{type,x});Object.assign(e,{state:'idle',t:0,cd:99},extra);return e}
function run(s:Any,n:number,input:(i:number)=>Any=()=>({})){const events:Any[]=[];for(let i=0;i<n;i++)events.push(...rules.step(s,input(i),dt));return events}
function mirror(s:Any,owner:Any,x=2760){s.mirrors=[{id:91,owner:owner.id,x,y:rules.GROUND,facing:-1,age:1}]}

describe('灯市卷',()=>{
 it('adds six stages and a scroll without changing old save records or leaking terrain tutorials',()=>{
  expect(levels.volumeLevels(5).map((x:Any)=>x.code)).toEqual([501,502,503,504,505,506,500])
  const old={...progress.empty(),volume:4,levels:{'4-1':{clears:2,best:4000,grade:3,seals:7,time:60}}}
  const restored=progress.normalize(old)
  expect(restored.levels['4-1']).toEqual(old.levels['4-1'])
  expect(progress.unlocked(restored,'5-1')).toBe(true)
  expect(progress.unlocked(restored,'5-2')).toBe(false)
  for(const id of [1,2,3])expect(levels.volume(id).prompts.some((p:Any)=>p.key==='rigging')).toBe(false)
 })
 it('gives each encounter a bounded arena, valid enemy types, and a reachable start',()=>{
  for(const level of levels.volumeLevels(5))for(const id of level.encounters){const e=levels.volume(5).encounters[id];expect(e.lock[1]-e.lock[0]-80).toBeLessThanOrEqual(680);expect(e.lock[0]).toBeGreaterThanOrEqual(level.range[0]);expect(e.lock[1]).toBeLessThanOrEqual(level.range[1]);for(const wave of e.waves)for(const spec of wave)expect(rules.ENEMY[spec.type]).toBeDefined()}
 })
 it('warns before a handheld-mirror counter without spawning a double',()=>{
  const s=arena(),owner=place(s,'mirror',2760)
  const events=run(s,8,i=>({slashPressed:i===0}))
  expect(events.some(e=>e.type==='mirrorTell')).toBe(true)
  expect(s.reflection.state).toBe('tell')
  expect(s.player.hp).toBe(s.player.maxHp)
  expect(owner.hp).toBe(owner.maxHp)
  expect(s.reflection.t).toBeGreaterThan(.45)
 })
 it('returns a fresh parry to its source and opens one execution window',()=>{
  const s=arena(),owner=place(s,'mirror',2760)
  let last=-99
  const events=run(s,120,i=>{
   const r=s.reflection
   const attack=i===0||(r?.state==='fly'&&Math.abs(r.x-s.player.x)<45&&i-last>24)
   if(attack)last=i
   return {slashPressed:attack}
  })
  expect(events.some(e=>e.type==='mirrorBreak')).toBe(true)
  expect(s.player.mirrorReturns).toBe(1)
  expect(s.player.parries).toBe(1)
  expect(s.player.hp).toBe(s.player.maxHp)
  expect(s.reflection).toBeNull()
  expect(s.mirrors).toHaveLength(0)
  expect(owner.mirrorOpened).toBe(true)
 })
 it('allows dodging a return and removes it when its owner dies',()=>{
  const s=arena(),owner=place(s,'mirror',2760)
  run(s,9,i=>({slashPressed:i===0}))
  run(s,60,i=>({jumpPressed:i===25,jump:i>=25}))
  expect(s.player.hp).toBe(s.player.maxHp)
  owner.alive=false;run(s,1)
  expect(s.reflection).toBeNull();expect(s.mirrors).toHaveLength(0)
 })
 it('never creates actor doubles for mirror performers or the judge',()=>{
  const s=arena();place(s,'mirror',2800);place(s,'mirror',3000)
  run(s,240)
  expect(s.mirrors).toHaveLength(0)
  const fight=arena('5-6',5700);place(fight,'judge',5820,{judgeStage:2,t:1})
  run(fight,240)
  expect(fight.mirrors).toHaveLength(0)
 })
 it('makes the watchman block frontal slashes and lose his guard on a parry',()=>{
  const s=arena(),paper=place(s,'watchman',2770,{facing:-1,state:'tele',t:.01,teleTotal:.75})
  const events=run(s,25,i=>({slashPressed:i===0}))
  expect(s.player.guardBreaks).toBe(1)
  expect(paper.state).toBe('stagger')
  expect(events.some(e=>e.type==='guardBreak')).toBe(true)
 })
 it('warns before a horizontal brush stroke, which a jump can avoid',()=>{
  for(const jump of [false,true]){
   const s=arena('5-6',5700);place(s,'judge',5820,{state:'idle',t:0,moves:2})
   run(s,1);expect(s.writing[0].kind).toBe('horizontal');expect(s.writing[0].phase).toBe('tell')
   run(s,105,i=>({jumpPressed:jump&&i===44,jump:jump&&i>=44}))
   expect(s.player.hp).toBe(s.player.maxHp-(jump?0:1))
   expect(s.player.writingDodges).toBe(jump?1:0)
  }
 })
 it('clears mirror and stroke hazards on either judge phase transition',()=>{
  const s=arena('5-6',5700),boss=place(s,'judge',5750,{state:'recover',t:4,hp:38})
  mirror(s,boss,5960)
  run(s,12,i=>({slashPressed:i===0}))
  expect(boss.judgeStage).toBe(2);expect(boss.state).toBe('roar');expect(s.mirrors).toHaveLength(0)
  run(s,110);Object.assign(boss,{state:'recover',t:4,hp:17,x:s.player.x+40});s.player.action=null;s.player.slashBuffer=0
  run(s,12,i=>({slashPressed:i===0}))
  expect(boss.judgeStage).toBe(3);expect(boss.state).toBe('roar');expect(s.writing).toHaveLength(0)
 })
 it('can finish all three judge phases with readable parries, movement and recovery attacks',()=>{
  const s=rules.createGame({level:'5-6',difficulty:'easy',seed:9}),stages=new Set<number>()
  let last=-99
  for(let i=0;i<60*220&&s.phase==='playing';i++){
   const p=s.player,b=s.enemies.find((e:Any)=>e.type==='judge'&&e.alive),r=s.reflection,w=s.writing[0]
   const distance=b?Math.abs(p.x-b.x):999,fresh=i-last>25
   const input:Any={move:!b?1:distance>68?Math.sign(b.x-p.x):0}
   if(b)stages.add(b.judgeStage)
   if(w?.phase==='tell'){
    if(w.kind==='horizontal'){if(w.t<.2&&p.onGround){input.jumpPressed=true;input.jump=true}}
    else if(w.kind==='vertical'){if(Math.abs(p.x-w.x)<65)input.move=p.x<=w.x?-1:1}
    else input.move=p.x>w.x-130?-1:0
   }else if(r){input.move=0;if(r.state==='fly'&&Math.abs(r.x-p.x)<45&&fresh){p.facing=Math.sign(r.x-p.x)||p.facing;input.slashPressed=true;last=i}}
   else if(b?.state==='judgeTell'&&b.t<.045&&distance<125&&fresh){p.facing=Math.sign(b.x-p.x)||p.facing;input.slashPressed=true;last=i}
   else if(b&&['recover','broken'].includes(b.state)&&distance<85&&fresh){input.slashPressed=true;last=i}
   if(p.ink>=100)input.inkPressed=true
   rules.step(s,input,dt)
  }
  expect([...stages]).toEqual([1,2,3])
  expect(s.reason,JSON.stringify({hp:s.player.hp,time:s.time,boss:s.enemies.find((e:Any)=>e.type==='judge')})).toBe('clear')
 })
 it.each(['5-1','5-2','5-3','5-4','5-5'])('can clear %s through timed counters and terrain movement',level=>{
  const s=rules.createGame({level,difficulty:'easy',seed:23})
  let last=-99
  for(let i=0;i<60*240&&s.phase==='playing';i++){
   const p=s.player,r=s.reflection,w=s.writing[0],fresh=s.clock-last>.34
   const foes=s.enemies.filter((e:Any)=>e.alive&&e.delay<=0&&e.state!=='spawn')
   const open=foes.find((e:Any)=>e.state==='stagger')
   const threat=foes.find((e:Any)=>e.state==='tele'&&Math.abs(e.x-p.x)<rules.ENEMY[e.type].reach+30)
   const nearest=foes.sort((a:Any,b:Any)=>Math.abs(a.x-p.x)-Math.abs(b.x-p.x))[0]
   const target=open||threat||nearest
   const input:Any={move:!target?1:Math.abs(target.x-p.x)>65?Math.sign(target.x-p.x):0}
   const slash=()=>{input.slashPressed=true;last=s.clock}
   if(w?.phase==='tell'){
    if(w.kind==='horizontal'){if(w.t<.2&&p.onGround){input.jumpPressed=true;input.jump=true}}
    else if(Math.abs(p.x-w.x)<65)input.move=p.x<=w.x?-1:1
   }else if(r){input.move=0;p.facing=Math.sign(r.x-p.x)||p.facing;if(r.state==='fly'&&Math.abs(r.x-p.x)<48&&fresh)slash()}
   else if(open){if(Math.abs(open.x-p.x)<76&&fresh){p.facing=Math.sign(open.x-p.x)||p.facing;slash()}}
   else if(threat&&threat.t<.055&&fresh){p.facing=Math.sign(threat.x-p.x)||p.facing;slash()}
   else if(s.mirrors.length&&fresh){
    const m=s.mirrors.filter((m:Any)=>m.age>.65).sort((a:Any,b:Any)=>Math.abs(a.x-p.x)-Math.abs(b.x-p.x))[0]
    if(m){input.move=Math.abs(m.x-p.x)>55?Math.sign(m.x-p.x):0;if(Math.abs(m.x-p.x)<72){p.facing=Math.sign(m.x-p.x)||p.facing;slash()}}
   }else if(target?.state==='recover'&&Math.abs(target.x-p.x)<80&&fresh){p.facing=Math.sign(target.x-p.x)||p.facing;slash()}
   if(p.ink>=100)input.inkPressed=true
   rules.step(s,input,dt)
  }
  expect(s.reason,JSON.stringify({level,hp:s.player.hp,time:s.time,kills:s.player.kills,foes:s.enemies.map((e:Any)=>({type:e.type,hp:e.hp,state:e.state,x:e.x})),x:s.player.x})).toBe('clear')
 })
 it('does not reward a normal-mode mash bot with a high grade',()=>{
  const s=rules.createGame({level:'5-3',seed:31})
  for(let i=0;i<60*150&&s.phase==='playing';i++){
   const target=s.enemies.find((e:Any)=>e.alive&&e.delay<=0)
   rules.step(s,{move:target?Math.sign(target.x-s.player.x):1,slashPressed:i%6===0},dt)
  }
  expect(rules.evaluate(s).grade).toBeLessThan(3)
 })

 it.each(['vertical','box'])('leaves a reachable escape route for %s strokes',kind=>{
  const s=arena('5-6',5700),boss=place(s,'judge',5820,{state:'idle',t:0,moves:kind==='box'?5:3,judgeStage:kind==='box'?2:1})
  // Both scripted selections create the expected geometries without mutating their hitboxes.
  if(kind==='vertical'){boss.x=6000;boss.moves=1}
  run(s,1)
  expect(s.writing[0].kind).toBe(kind)
  const hp=s.player.hp
  run(s,130,()=>({move:-1}))
  expect(s.player.hp).toBe(hp)
  expect(s.player.writingDodges).toBe(1)
 })
 it('protects the judge during its phase-change animation',()=>{
  const s=arena('5-6',5700),boss=place(s,'judge',5750,{state:'roar',t:1.2,hp:30,judgeStage:2})
  run(s,15,i=>({slashPressed:i===0}))
  expect(boss.hp).toBe(30)
 })

})
