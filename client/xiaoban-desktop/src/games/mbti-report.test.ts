import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { Window } from 'happy-dom'
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
  return {window,writes,click:(id:string)=>window.document.getElementById(id)!.click()}
}
it('finishes a real quiz flow, saves its answers, and reopens the same personalized report',()=>{
  const a=answersFor('INFJ'),partial=[...a];partial[47]=null
  const first=page({version:1,progress:{answers:partial,current:47,mode:'all'},lastResult:null})
  first.click('resumeButton');first.click('option'+a[47])
  const saved=first.writes.at(-1)
  expect(saved.version).toBe(2);expect(saved.progress).toBeNull();expect(saved.lastResult.answers).toEqual(a)
  expect(first.window.document.querySelectorAll('.report-section')).toHaveLength(6)
  const text=first.window.document.getElementById('reportBody')!.textContent
  const second=page(saved);second.click('lastResultButton')
  expect(second.window.document.getElementById('reportBody')!.textContent).toBe(text)
  expect(second.window.document.getElementById('resultType')!.textContent).toBe('INFJ')
  first.window.close();second.window.close()
})
