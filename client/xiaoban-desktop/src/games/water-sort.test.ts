import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { Window } from 'happy-dom'
import { describe, expect, it, vi } from 'vitest'

const source = (file: string) => readFileSync(new URL(`../../public/game-packs/water-sort/${file}`, import.meta.url), 'utf8')
const core: any = {}
for (const file of ['engine.js', 'levels.js', 'progress.js']) runInNewContext(source(file), core)
const R = core.XiabanWaterSort, P = core.PourJoyProgress, levels = core.PourJoyLevels
const plain = (value: any) => JSON.parse(JSON.stringify(value))

describe('Pour Joy campaign and challenge rules', () => {
  it('ships 36 distinct, solvable levels in four nine-level chapters', () => {
    expect(levels).toHaveLength(36)
    expect(new Set(levels.map((l: any) => l.board.map((t: number[]) => t.join('')).sort().join('|'))).size).toBe(36)
    for (const level of levels) {
      let state = R.initial(level)
      expect(R.validate(state, level), level.id).toBe(true)
      expect(R.isWon(state, level), level.id).toBe(false)
      const before = JSON.stringify(state)
      for (const move of level.solution) {
        const previous = JSON.stringify(state), next = R.pour(state, level, ...move)
        expect(next, level.id).not.toBeNull()
        expect(JSON.stringify(state)).toBe(previous)
        state = next
        expect(R.validate(state, level), level.id).toBe(true)
      }
      expect(R.isWon(state, level), level.id).toBe(true)
      expect(JSON.stringify(R.initial(level))).toBe(before)
      expect(level.solution.length).toBeLessThanOrEqual(level.par)
    }
    for (let i = 0; i < 4; i++) expect(levels.filter((l: any) => l.chapter === i)).toHaveLength(9)
  })
  it('pours the contiguous top run and enforces capacity, color and empty source', () => {
    const level = { orders:{}, seals:{} }, state = { board:[[0,1,1,1],[1,1],[0],[]], opened:[] }
    expect(R.getPour(state,level,0,1).amount).toBe(2)
    expect(R.pour(state,level,0,1).board).toEqual([[0,1],[1,1,1,1],[0],[]])
    expect(R.pour(state,level,0,2)).toBeNull()
    expect(R.pour(state,level,3,0)).toBeNull()
    expect(R.pour(state,level,0,0)).toBeNull()
    expect(R.pour(state,level,-1,0)).toBeNull()
  })
  it('seals latch open after completing the key flavor and undo restores the latch', () => {
    const level = { colors:2, board:[[0,0,0],[0],[1,1,1,1],[]], seals:{3:0}, orders:{} }
    const before = R.initial(level)
    expect(R.pour(before,level,1,3)).toBeNull()
    const unlocked = R.pour(before,level,1,0)
    expect(unlocked.opened).toEqual([3])
    expect(R.pour(unlocked,level,0,3)).not.toBeNull()
    expect(R.locked(before,level,3)).toBe(true)
  })
  it('order cups allow temporary colors but require the correct final position', () => {
    const level = {orders:{0:1},seals:{}}, state = {board:[[0,0,0,0],[1,1,1,1],[]],opened:[]}
    expect(R.isWon(state,level)).toBe(false)
    const solution = R.solve(state,level,5000)
    expect(solution).not.toBeNull()
    let s = state; for (const move of solution) s=R.pour(s,level,...move)
    expect(R.isWon(s,level)).toBe(true)
  })
  it('daily variants are reproducible, distinct, valid and retain a solution', () => {
    const seen = new Set()
    for (let i=1;i<=28;i++) {
      const id='d202611'+String(i).padStart(2,'0'), level=R.daily(id,levels)
      expect(plain(R.daily(id,levels))).toEqual(plain(level))
      expect(plain(R.daily(id,[...levels,levels[0]]))).toEqual(plain(level))
      let state=R.initial(level)
      for(const move of level.solution) state=R.pour(state,level,...move)
      expect(R.isWon(state,level),id).toBe(true)
      seen.add(JSON.stringify(level.board))
    }
    expect(seen.size).toBeGreaterThan(20)
    for (const id of ['d20260230','d20261301','d19990101','d20260000','<script>','c99']) expect(P.getLevel(id)).toBeNull()
  })
  it('bounded hints stop and reference solutions are not falsely optimal claims', () => {
    const search=R.createSearch(R.initial(levels[0]),levels[0],3)
    expect(search.step(100).done).toBe(true)
    expect(R.stars(10,10,0)).toBe(3)
    expect(R.stars(10,10,1)).toBe(2)
    expect(R.stars(16,10,0)).toBe(1)
  })
})

describe('Pour Joy durable progress', () => {
  it('replays active moves and rejects corrupt/illegal or post-win moves', () => {
    const data=P.fresh(),level=levels[0]
    data.session={id:level.id,mode:'campaign',moves:level.solution.slice(0,2),hints:1}
    expect(P.sanitize(data).session.moves).toHaveLength(2)
    data.session.moves=[[99,0]];expect(P.sanitize(data).session).toBeNull()
    data.session.moves=[...level.solution,[0,1]];expect(P.sanitize(data).session).toBeNull()
    data.session={id:'c36',mode:'campaign',moves:[],hints:0};expect(P.sanitize(data).session).toBeNull()
    data.session.mode='challenge';expect(P.sanitize(data).session.id).toBe('c36')
    expect(P.sanitize({version:2,records:{c36:{moves:3,stars:3,assisted:false}}}).records).toEqual({})
    expect(P.sanitize({version:900}).session).toBeNull()
  })
  it('unlocks sequentially, preserves best stars, and isolates friend/daily results', () => {
    const data=P.fresh(),level=levels[0],session={id:level.id,mode:'campaign',moves:level.solution,hints:0}
    expect(P.finish(data,session).stars).toBe(3)
    expect(P.unlocked(data)).toBe(1)
    P.finish(data,{...session,hints:1});expect(data.records.c01.stars).toBe(3)
    P.finish(data,{id:'c36',mode:'challenge',moves:levels[35].solution,hints:0})
    expect(data.records.c36).toBeUndefined()
    const daily=P.getLevel('d20261112');P.finish(data,{id:daily.id,mode:'daily',moves:daily.solution,hints:0})
    expect(data.daily.d20261112.stars).toBe(3);expect(P.unlocked(data)).toBe(1)
    expect(plain(P.sanitize(data).records)).toEqual(plain(data.records))
  })
})

async function fixture(saved: any = null, failRead = false) {
  const window = new Window({url:'http://localhost/'})
  window.document.body.innerHTML=source('index.html').match(/<body[^>]*>([\s\S]*)<\/body>/)![1]
  const values: any[] = [], events: string[] = []
  const sdk={info:{query:{}},ready:vi.fn(),can:()=>false,on:vi.fn(),storage:{get:vi.fn(async()=>{if(failRead)throw Error('disk unavailable');return saved}),set:vi.fn(async(_key:string,value:any)=>{values.push(plain(value))})},game:{start:()=>events.push('start'),progress:()=>events.push('progress'),result:()=>events.push('result')}}
  const c:any={window,document:window.document,navigator:window.navigator,location:window.location,history:window.history,URLSearchParams,CustomEvent:window.CustomEvent,xb:sdk,performance,innerWidth:390,innerHeight:844,matchMedia:()=>({matches:true}),getComputedStyle:window.getComputedStyle.bind(window),addEventListener:window.addEventListener.bind(window),dispatchEvent:window.dispatchEvent.bind(window),setTimeout:window.setTimeout.bind(window),clearTimeout:window.clearTimeout.bind(window)}
  for(const file of ['engine.js','levels.js','progress.js','host.js','art.js','game.js'])runInNewContext(source(file),c)
  const tick=()=>new Promise(resolve=>setTimeout(resolve,10))
  await tick()
  const click=async(selector:string)=>{const node=window.document.querySelector(selector) as any;expect(node,selector).toBeTruthy();node.click();await tick()}
  return {window,sdk,values,events,c,click,tick}
}

describe('Pour Joy game and host integration',()=>{
  it('starts once, saves a committed pour, undoes, completes and persists rewards',async()=>{
    const f=await fixture();try{
      expect(f.sdk.ready).toHaveBeenCalledTimes(1)
      await f.click('#continue')
      expect(f.events).toEqual(['start','progress'])
      const [a,b]=levels[0].solution[0]
      await f.click(`[data-tube="${a}"]`);await f.click(`[data-tube="${b}"]`)
      expect(f.values.at(-1).session.moves).toEqual([[a,b]])
      await f.click('#undo');expect(f.values.at(-1).session.moves).toEqual([])
      for(const [from,to] of levels[0].solution){await f.click(`[data-tube="${from}"]`);await f.click(`[data-tube="${to}"]`)}
      expect(f.events.filter(e=>e==='result')).toHaveLength(1)
      expect(f.values.at(-1).records.c01.stars).toBe(3)
      expect(f.window.document.body.textContent).toContain('完美出杯')
      await f.click('[data-view="receipt"]')
      expect(f.window.document.body.textContent).toContain('PJ-C01')
    }finally{await f.window.happyDOM.close()}
  })
  it('restores an unfinished round without losing hints or undo history',async()=>{
    const saved=P.fresh();saved.session={id:'c01',mode:'campaign',moves:levels[0].solution.slice(0,2),hints:1}
    const f=await fixture(saved);try{await f.click('#continue');expect(f.window.document.querySelector('#moveCount')!.textContent).toBe('02');await f.click('#undo');expect(f.values.at(-1).session.moves).toHaveLength(1);expect(f.values.at(-1).session.hints).toBe(1)}finally{await f.window.happyDOM.close()}
  })
  it('does not overwrite unreadable storage and offers a read-only session',async()=>{
    const f=await fixture(null,true);try{expect(f.window.document.body.textContent).toContain('存档暂时读不到');expect(f.sdk.storage.set).not.toHaveBeenCalled();await f.click('#temporaryPlay');await f.click('#continue');const [a,b]=levels[0].solution[0];await f.click(`[data-tube="${a}"]`);await f.click(`[data-tube="${b}"]`);expect(f.sdk.storage.set).not.toHaveBeenCalled()}finally{await f.window.happyDOM.close()}
  })
  it('serializes saves so a slower earlier write cannot overwrite newer progress',async()=>{
    const f=await fixture();try{
      let release:()=>void=()=>{}
      f.sdk.storage.set.mockImplementationOnce(async()=>{await new Promise<void>(resolve=>{release=resolve})})
      const first=f.c.PourJoyHost.save({version:2,marker:1})
      await f.tick()
      const second=f.c.PourJoyHost.save({version:2,marker:2})
      await f.tick();expect(f.sdk.storage.set).toHaveBeenCalledTimes(1)
      release();await first;await second
      expect(f.values.at(-1).marker).toBe(2)
    }finally{await f.window.happyDOM.close()}
  })
})
