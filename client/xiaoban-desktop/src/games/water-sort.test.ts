import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { Window } from 'happy-dom'
import { describe, expect, it, vi } from 'vitest'

const source = (file: string) => readFileSync(new URL(`../../public/game-packs/water-sort/${file}`, import.meta.url), 'utf8')
const core: any = {}
for (const file of ['engine.js', 'levels.js', 'progress.js']) runInNewContext(source(file), core)
const R = core.XiabanWaterSort, P = core.PourJoyProgress, levels = core.PourJoyLevels
const training = core.PourJoyTraining
const plain = (value: any) => JSON.parse(JSON.stringify(value))

describe('Pour Joy campaign and challenge rules', () => {
  it('ships 45 distinct, solvable campaign levels and three practice rounds', () => {
    expect(levels).toHaveLength(45)
    expect(training.map((l: any) => l.solution.length)).toEqual([3, 4, 6])
    expect(training[0].board[training[0].solution[0][1]]).toEqual([])
    expect(new Set(levels.map((l: any) => l.board.map((t: number[]) => t.join('')).sort().join('|'))).size).toBe(45)
    for (const level of [...levels, ...training]) {
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
    for (let i = 0; i < 5; i++) expect(levels.filter((l: any) => l.chapter === i)).toHaveLength(9)
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

  it('replays an added cup at the original move and treats the result as assisted', () => {
    const level = levels[0], data = P.fresh()
    const session = {id:level.id,mode:'campaign',moves:level.solution,hints:0,extraAt:1}
    const replayed = P.replay(level, session.moves, session.extraAt)
    expect(replayed.history[0].board).toHaveLength(4)
    expect(replayed.history[1].board).toHaveLength(5)
    expect(P.finish(data, session)).toMatchObject({ stars: 2, assisted: true })
    expect(P.replay(level, session.moves, session.moves.length)).toBeNull()
  })
})

it('moves both liquid surfaces together and cleans up when the pour is interrupted', async () => {
  const window = new Window()
  try {
    const {document} = window
    document.body.innerHTML = '<div><div id="source"><span class="liquid-stack"><i></i><i></i><i></i></span></div></div><div id="target"><span class="liquid-stack"><i></i></span></div>'
    const sourceCup = document.querySelector('#source') as HTMLElement
    const targetCup = document.querySelector('#target') as HTMLElement
    const area = (left: number, top: number) => ({left, top, width:60, height:100, bottom:top+100})
    Object.defineProperties(sourceCup, {
      offsetWidth:{value:60}, offsetHeight:{value:100}, offsetTop:{value:10},
      offsetParent:{value:{getBoundingClientRect:()=>area(20, 20)}},
      getBoundingClientRect:{value:()=>area(20, 19)},
    })
    Object.defineProperty(targetCup,'getBoundingClientRect',{value:()=>area(100, 30)})
    Object.defineProperty(targetCup.querySelector('.liquid-stack'),'getBoundingClientRect',{value:()=>area(100, 30)})
    let draw: (now: number) => void = () => {}
    const c: any = {document, performance:{now:()=>0}, requestAnimationFrame:(fn:typeof draw)=>{draw=fn;return 1}, cancelAnimationFrame:()=>{}}
    runInNewContext(source('motion.js'), c)
    const onFlow = vi.fn()
    const animation = c.PourJoyMotion.pour(sourceCup,targetCup,{amount:2,color:'#f54268',onFlow})
    draw(240)
    expect(document.querySelector<HTMLElement>('.pour-ghost .flowing-layer')?.style.height).toBe('25%')
    expect(targetCup.querySelector<HTMLElement>('.flowing-layer')?.style.height).toBe('25%')
    expect(onFlow).toHaveBeenCalledOnce()
    animation.finish()
    await animation.finished
    expect(document.querySelector('.pour-ghost, .pour-stream, .flowing-layer')).toBeNull()
    expect(sourceCup.style.visibility).toBe('')
  } finally { await window.happyDOM.close() }
})

async function fixture(saved: any = null, failRead = false, animate = false) {
  const window = new Window({url:'http://localhost/'})
  window.document.body.innerHTML=source('index.html').match(/<body[^>]*>([\s\S]*)<\/body>/)![1]
  const values: any[] = [], events: string[] = []
  const sdk={info:{query:{}},ready:vi.fn(),can:()=>false,on:vi.fn(),storage:{get:vi.fn(async()=>{if(failRead)throw Error('disk unavailable');return saved}),set:vi.fn(async(_key:string,value:any)=>{values.push(plain(value))})},game:{start:()=>events.push('start'),progress:()=>events.push('progress'),result:()=>events.push('result')}}
  const c:any={window,document:window.document,navigator:window.navigator,location:window.location,history:window.history,URLSearchParams,CustomEvent:window.CustomEvent,xb:sdk,performance,innerWidth:390,innerHeight:844,matchMedia:()=>({matches:!animate}),getComputedStyle:window.getComputedStyle.bind(window),addEventListener:window.addEventListener.bind(window),dispatchEvent:window.dispatchEvent.bind(window),setTimeout:window.setTimeout.bind(window),clearTimeout:window.clearTimeout.bind(window)}
  for(const file of ['engine.js','levels.js','progress.js','host.js','art.js','audio.js','motion.js'])runInNewContext(source(file),c)
  let finishAnimation = () => {}
  if (animate) c.PourJoyMotion = {
    pour: () => ({finished: new Promise<void>(resolve => { finishAnimation = resolve }), finish: () => finishAnimation()}),
  }
  runInNewContext(source('game.js'),c)
  const tick=()=>new Promise(resolve=>setTimeout(resolve,10))
  await tick()
  const click=async(selector:string)=>{const node=window.document.querySelector(selector) as any;expect(node,selector).toBeTruthy();node.click();await tick()}
  return {window,sdk,values,events,c,click,tick,finishAnimation:()=>finishAnimation()}
}

describe('Pour Joy game and host integration',()=>{
  it('guides the last pour when level ten has three sorted colors but an empty order cup', async () => {
    const saved = P.fresh()
    for (const level of levels.slice(0, 9)) {
      P.finish(saved, {id:level.id, mode:'campaign', moves:level.solution, hints:0})
    }
    saved.session = {id:'c10', mode:'campaign', moves:[[0,4],[0,2],[1,4],[3,2],[4,3],[1,4]], hints:0}
    const f = await fixture(saved)
    try {
      await f.click('#continue')
      const doc = f.window.document
      expect(doc.querySelector('.completion-count')?.textContent).toBe('2 / 3')
      expect(doc.querySelector('[data-tube="2"] .complete-seal')).toBeNull()
      expect(doc.querySelector('.game-tip')?.textContent).toBe('把 3 号杯的草莓倒进 1 号订单杯')
      expect(doc.querySelector('#nextLevel')).toBeNull()
      await f.click('[data-tube="2"]')
      await f.click('[data-tube="0"]')
      expect(doc.querySelector('#nextLevel')?.textContent).toContain('下一关 · 第 11 关')
      expect(f.events.filter(event => event === 'result')).toHaveLength(1)
    } finally { await f.window.happyDOM.close() }
  })
  it('saves and undoes pours, then opens level two from the primary action on the result page',async()=>{
    const f=await fixture({...P.fresh(),trainingDone:true});try{
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
      const next = () => f.window.document.querySelector('#nextLevel')
      expect(next()?.textContent).toContain('下一关 · 第 2 关')
      expect(next()?.classList.contains('primary')).toBe(true)
      expect(f.window.document.activeElement).toBe(next())
      expect(f.window.document.querySelector('#undo')).toBeNull()
      await f.click('[data-view="receipt"]')
      expect(f.window.document.body.textContent).toContain('PJ-C01')
      expect(f.window.document.querySelector('.share-page [data-view="play"]')?.textContent).toBe('返回结算')
      await f.click('.share-page [data-view="play"]')
      await f.click('#nextLevel')
      expect(f.window.document.querySelector('.level-heading')?.textContent).toContain('第 2 关')
      expect(f.values.at(-1).session).toMatchObject({id:'c02',mode:'campaign',moves:[]})
      expect(f.events.filter(e=>e==='start')).toHaveLength(2)
      expect(f.events.filter(e=>e==='result')).toHaveLength(1)
    }finally{await f.window.happyDOM.close()}
  })
  it('restores an unfinished round without losing hints or undo history',async()=>{
    const saved=P.fresh();saved.session={id:'c01',mode:'campaign',moves:levels[0].solution.slice(0,2),hints:1}
    const f=await fixture(saved);try{await f.click('#continue');expect(f.window.document.querySelector('#moveCount')!.textContent).toBe('02');await f.click('#undo');expect(f.values.at(-1).session.moves).toHaveLength(1);expect(f.values.at(-1).session.hints).toBe(1)}finally{await f.window.happyDOM.close()}
  })
  it('does not overwrite unreadable storage and offers a read-only session',async()=>{
    const f=await fixture(null,true);try{expect(f.window.document.body.textContent).toContain('存档暂时读不到');expect(f.sdk.storage.set).not.toHaveBeenCalled();await f.click('#temporaryPlay');await f.click('#continue');const [a,b]=training[0].solution[0];await f.click(`[data-tube="${a}"]`);await f.click(`[data-tube="${b}"]`);expect(f.sdk.storage.set).not.toHaveBeenCalled()}finally{await f.window.happyDOM.close()}
  })
  it('guides new players through practice and starts the existing campaign', async () => {
    const f = await fixture()
    try {
      await f.click('#continue')
      expect(f.window.document.querySelector('.training-cue')?.getAttribute('data-tube')).toBe('0')
      for (const level of training) {
        for (const [a, b] of level.solution) {
          await f.click(`[data-tube="${a}"]`)
          await f.click(`[data-tube="${b}"]`)
        }
        expect(f.window.document.querySelector('#roundTitle')).not.toBeNull()
        await f.click('#nextLevel')
      }
      expect(f.values.at(-1)).toMatchObject({trainingDone:true, records:{}, session:{id:'c01', mode:'campaign'}})
    } finally { await f.window.happyDOM.close() }
  })
  it('applies two queued taps against the board after the current pour', async () => {
    const f = await fixture({...P.fresh(),trainingDone:true}, false, true)
    try {
      await f.click('#continue')
      const [first, second] = levels[0].solution
      for (const tube of [...first, ...second]) await f.click(`[data-tube="${tube}"]`)
      expect(f.values.at(-1).session.moves).toEqual([first])
      f.finishAnimation()
      await f.tick()
      expect(f.values.at(-1).session.moves).toEqual([first, second])
      f.finishAnimation()
      await f.tick()
      expect(f.window.document.querySelector('#moveCount')?.textContent).toBe('02')
    } finally { await f.window.happyDOM.close() }
  })
  it('restores and undoes the extra cup from a saved round', async () => {
    const data = P.fresh()
    data.session = {id:'c01',mode:'campaign',moves:[levels[0].solution[0]],hints:0,extraAt:1}
    const f = await fixture(data)
    try {
      await f.click('#continue')
      expect(f.window.document.querySelectorAll('[data-tube]')).toHaveLength(5)
      await f.click('#undo')
      expect(f.window.document.querySelectorAll('[data-tube]')).toHaveLength(4)
      expect(f.values.at(-1).session.moves).toHaveLength(1)
      expect(f.values.at(-1).session.extraAt).toBeNull()
    } finally { await f.window.happyDOM.close() }
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
