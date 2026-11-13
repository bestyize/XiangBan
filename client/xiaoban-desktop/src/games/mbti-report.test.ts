import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { Window, type HTMLElement } from 'happy-dom'
import { describe, expect, it } from 'vitest'

const source=(file:string)=>readFileSync(new URL(`../../public/game-packs/mbti/${file}`,import.meta.url),'utf8')
function load(){const c:any={};for(const file of ['rules.js','profiles.js','report.js'])runInNewContext(source(file),c);return {c,rules:c.XiaobanMbtiRules,engine:c.XiaobanMbtiReport,profiles:c.XiaobanMbtiProfiles}}
const {rules,engine,profiles}=load()
const answersFor=(type:string)=>rules.questions.map((q:any)=>q.options.findIndex((o:any)=>o.pole===type[q.dimension]))
const resultFor=(answers:any[])=>({...rules.score(answers),answers,completedAt:'2026-11-08T13:46:00.000Z'})
const types=['INTJ','INTP','ENTJ','ENTP','INFJ','INFP','ENFJ','ENFP','ISTJ','ISFJ','ESTJ','ESFJ','ISTP','ISFP','ESTP','ESFP']

describe('offline MBTI report personalization',()=>{
  it.each(types)('%s has a complete, distinct type report',type=>{
    const report=engine.build(resultFor(answersFor(type)))
    expect(report.type).toBe(type)
    expect(report.profile.role).toBeTruthy()
    expect(report.profile.description.length).toBeGreaterThan(65)
    for(const key of ['strengths','work','blindSpots','growth'])expect(report.profile[key]).toHaveLength(3)
    expect(report.profile.relationship.length).toBeGreaterThan(30)
    expect(report.dimensions.every((d:any)=>d.band==='strong')).toBe(true)
    expect(report.actions).toHaveLength(3)
  })
  it('retains all 16 unique roles and original introductions',()=>{
    expect(new Set(Object.values(profiles).map((p:any)=>p.role)).size).toBe(16)
    expect(new Set(Object.values(profiles).map((p:any)=>p.description)).size).toBe(16)
  })
  it('varies context advice even when the type and all four scores are identical',()=>{
    const a=answersFor('INFJ'),b=answersFor('INFJ')
    for(const [answers,ids] of [[a,['ei-3','ei-8','ei-11','ei-2']],[b,['ei-1','ei-5','ei-10','ei-2']]] as any){
      rules.questions.forEach((q:any,i:number)=>{if(ids.includes(q.id))answers[i]=q.options.findIndex((o:any)=>o.pole==='E')})
    }
    expect(rules.score(a)).toEqual(rules.score(b))
    const ar=engine.build(resultFor(a)),br=engine.build(resultFor(b))
    expect(ar.type).toBe('INFJ');expect(br.type).toBe('INFJ')
    expect(ar.contexts.find((c:any)=>c.id==='collaboration').pole).toBe('E')
    expect(br.contexts.find((c:any)=>c.id==='collaboration').pole).toBe('I')
    expect(ar.actions).not.toEqual(br.actions)
    ar.contexts.forEach((row:any)=>row.evidence.forEach((e:any)=>{
      const i=rules.questions.findIndex((q:any)=>q.id===e.id)
      expect(e.answer).toBe(rules.questions[i].options[a[i]].text)
    }))
  })
  it('distinguishes near, moderate, strong and tied preferences without changing scoring',()=>{
    const bands=[]
    for(const first of [6,7,8,9]){
      const a=answersFor('ESTJ');let count=0
      rules.questions.forEach((q:any,i:number)=>{if(q.dimension===0){const pole=count++<first?'E':'I';a[i]=q.options.findIndex((o:any)=>o.pole===pole)}})
      bands.push(engine.build(resultFor(a)).dimensions[0].band)
    }
    expect(bands).toEqual(['balanced','near','moderate','strong'])
  })
  it('does not infer scenario preferences or strong traits from one answer',()=>{
    const a=answersFor('INFJ').map((x:any,i:number)=>i<4?x:null)
    const report=engine.build(resultFor(a))
    expect(report.contexts).toEqual([])
    expect(report.dimensions.every((d:any)=>d.band==='developing')).toBe(true)
    expect(engine.build(resultFor(Array(48).fill(null)))).toBeNull()
  })
  it('keeps legacy results, and restores new results from authoritative answer snapshots',()=>{
    const result=resultFor(answersFor('ISFP'))
    const legacy={type:result.type,percentages:result.percentages,totals:result.totals,completedAt:result.completedAt}
    const migrated=rules.normalizeSave({version:1,lastResult:legacy})
    expect(migrated.version).toBe(2)
    expect(engine.build(migrated.lastResult).contexts).toEqual([])
    const saved=rules.normalizeSave({version:2,lastResult:{...result,type:'ESTJ',percentages:[100,100,100,100]}})
    expect(saved.lastResult.type).toBe('ISFP')
    expect(saved.lastResult.answers).toEqual(result.answers)
    expect(engine.build(JSON.parse(JSON.stringify(saved.lastResult)))).toEqual(engine.build(result))
  })
})

function page(saved:any){
  const window=new Window();window.document.body.innerHTML=source('index.html').split('<body>')[1].split('</body>')[0]
  const writes:any[]=[];const c:any={window,document:window.document,location:{search:''},URLSearchParams,setTimeout,clearTimeout,
    XiaobanBuiltinRuntime:{connect:(fn:any)=>fn({save:saved,safeArea:{top:88,right:0,bottom:16,left:0}}),send:(type:string,payload:any)=>{if(type==='game.save')writes.push(JSON.parse(JSON.stringify(payload.data)))}}}
  for(const file of ['rules.js','profiles.js','report.js','report-view.js','game.js'])runInNewContext(source(file),c)
  return {window,writes,click:(id:string)=>(window.document.getElementById(id)! as HTMLElement).click()}
}
it('finishes a real quiz flow, saves its answers, and reopens the same personalized report',()=>{
  const a=answersFor('INFJ'),partial=[...a];partial[47]=null
  const first=page({version:1,progress:{answers:partial,current:47,mode:'all'},lastResult:null})
  first.click('resumeButton');first.click('option'+a[47])
  expect(first.writes.at(-1).progress.answers).toEqual(a)
  first.click('nextButton')
  const saved=first.writes.at(-1)
  expect(saved.version).toBe(2);expect(saved.progress).toBeNull();expect(saved.lastResult.answers).toEqual(a)
  expect(first.window.document.querySelectorAll('.report-section')).toHaveLength(6)
  const text=first.window.document.getElementById('reportBody')!.textContent
  const second=page(saved);second.click('lastResultButton')
  expect(second.window.document.getElementById('reportBody')!.textContent).toBe(text)
  expect(second.window.document.getElementById('resultType')!.textContent).toBe('INFJ')
  first.window.close();second.window.close()
})

describe('native-style MBTI interaction',()=>{
  it('keeps selections editable and restores the current question before advancing',()=>{
    const first=page(null)
    first.click('startButton');first.click('option0');first.click('option1')
    expect(first.writes.at(-1).progress.current).toBe(0)
    expect(first.writes.at(-1).progress.answers[0]).toBe(1)
    const restored=page(first.writes.at(-1));restored.click('resumeButton')
    expect(restored.window.document.getElementById('option1')!.getAttribute('aria-pressed')).toBe('true')
    restored.click('nextButton')
    expect(restored.writes.at(-1).progress.current).toBe(1)
    expect((restored.window.document.getElementById('nextButton') as any).disabled).toBe(true)
    restored.click('previousButton')
    expect(restored.window.document.getElementById('option1')!.getAttribute('aria-pressed')).toBe('true')
    first.window.close();restored.window.close()
  })

  it('reviews missing answers, prevents empty results, and fills the original gaps',()=>{
    const current=page(null);current.click('startButton')
    for(let i=0;i<48;i++)current.click('skipButton')
    expect(current.window.document.querySelector<HTMLElement>('#review')!.hidden).toBe(false)
    expect((current.window.document.getElementById('finishButton') as any).disabled).toBe(true)
    current.click('fillButton')
    for(let i=0;i<48;i++){current.click('option0');current.click('nextButton')}
    expect(current.writes.at(-1).progress).toBeNull()
    expect(current.writes.at(-1).lastResult.answers).toEqual(Array(48).fill(0))
    expect(current.window.document.querySelector<HTMLElement>('#result')!.hidden).toBe(false)
    current.window.close()
  })

  it('switches chapters without losing answer evidence or session action marks',()=>{
    const current=page({version:2,lastResult:resultFor(answersFor('INFJ'))})
    current.click('lastResultButton')
    const doc=current.window.document
    expect(doc.querySelectorAll('.report-section:not([hidden])')).toHaveLength(1)
    expect(doc.querySelectorAll('.report-dimension')).toHaveLength(4)
    current.click('report-tab-work')
    expect(doc.querySelector<HTMLElement>('#report-work')!.hidden).toBe(false)
    expect(doc.querySelectorAll('#report-work .report-evidence')).not.toHaveLength(0)
    expect(current.window.history.state).toMatchObject({page:'result',chapter:'work'})
    current.click('report-tab-growth')
    const check=doc.querySelector('.report-step input') as any
    check.click();expect(check.checked).toBe(true)
    current.click('homeButton');current.click('lastResultButton')
    expect((doc.querySelector('.report-step input') as any).checked).toBe(true)
    expect(doc.querySelector<HTMLElement>('#report-growth')!.hidden).toBe(false)
    current.window.close()
  })

  it('restores report chapters and falls back safely from obsolete quiz history',()=>{
    const current=page({version:2,lastResult:resultFor(answersFor('INTJ'))})
    current.click('lastResultButton');current.click('report-tab-growth')
    current.window.dispatchEvent(new current.window.PopStateEvent('popstate',{state:{mbti:true,page:'result',chapter:'traits'}}))
    expect(current.window.document.querySelector<HTMLElement>('#report-traits')!.hidden).toBe(false)
    current.window.dispatchEvent(new current.window.PopStateEvent('popstate',{state:{mbti:true,page:'quiz'}}))
    expect(current.window.document.querySelector<HTMLElement>('#home')!.hidden).toBe(false)
    expect(current.window.history.state).toMatchObject({page:'home'})
    current.window.close()
  })
})

describe('public result snapshots',()=>{
  const {c}=load()
  c.URLSearchParams=URLSearchParams
  c.location={search:''}
  runInNewContext(source('share.js'),c)
  const share=c.XiaobanMbtiShare
  it('round trips only type, four scores, answer counts and completion date',()=>{
    const input=resultFor(answersFor('INFJ'))
    const url=new URL(share.link(input))
    expect(url.origin).toBe('https://chat.xiangban.xyz')
    expect(url.pathname).toBe('/builtin-games/mbti/share.html')
    expect([...url.searchParams.keys()]).toEqual(['t','p','n','d'])
    expect(share.parse(url.search)).toEqual({type:input.type,percentages:input.percentages,totals:input.totals,completedAt:input.completedAt})
    expect(url.href).not.toContain('answers')
  })
  it('rejects malformed, inconsistent, out-of-range and incomplete links',()=>{
    for(const query of [
      '?t=INFJ&p=0,0,0,0&n=12,12,12,12&d=2026-11-08T13%3A46%3A00.000Z',
      '?t=INFJ&p=-1,0,0,100&n=12,12,12,12&d=2026-11-08T13%3A46%3A00.000Z',
      '?t=INFJ&p=0,0,0,100&n=12,0,12,12&d=2026-11-08T13%3A46%3A00.000Z',
      '?t=INFJ&p=0,0,0,100&n=12,12,12,12&d=invalid',
      '?t=INFJ&p=0,0,0,100&n=12,12,12,12',
      '?t=INFJ&p=0,0,0,100&n=12,12,12,12&d=2026-11-08T13%3A46%3A00.000Z&t=ESTJ',
      '?t=INFJ&p=0,0,0,100&n=12,12,12,12&d=2026-11-08T13%3A46%3A00.000Z&answers=private',
      '?t=INFJ&p=0,0,0,100&n=12,12,12,12&d=2026-02-31T13%3A46%3A00.000Z',
    ]) expect(share.parse(query)).toBeNull()
  })
  it('renders a shared report with chapters while keeping individual answers out of the link',()=>{
    const result=resultFor(answersFor('INFJ')),url=share.link(result),window=new Window({url})
    window.document.body.innerHTML='<main id="publicResult"></main>'
    const c:any={window,document:window.document,location:window.location,URLSearchParams,setTimeout,clearTimeout}
    for(const file of ['rules.js','profiles.js','report.js','share.js','share-page.js'])runInNewContext(source(file),c)
    const text=window.document.getElementById('publicResult')!.textContent!
    for(const label of ['INFJ','四维性格倾向','你的力量与另一面','你的工作方式','你的人际关系','把洞察带进生活'])expect(text).toContain(label)
    expect(url).not.toContain('answers')
    expect(window.document.querySelectorAll('.public-section')).toHaveLength(5)
    window.close()
  })
})
