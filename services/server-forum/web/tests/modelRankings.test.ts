import test from 'node:test'
import assert from 'node:assert/strict'
import { METRICS,rankModels,availableMetrics,formatValue,formatTokens,formatShare,movement,scatterPoints,scatterPriceAxis,scatterPriceTicks,scatterFrontier,scatterScoreAxis,median,logFraction,type Model,type Board,type ScatterPoint } from '../src/lib/modelRankings.ts'
const model=(id:string,value:number|null,input=1,output=3):Model=>({id,name:id,vendor_id:'vendor',metrics:{intelligence:value},pricing:{input,output},performance:{output_tokens_per_second:null}})
const board=(models:Model[]):Board=>({source:'test',as_of:'2026-11-01',index_version:'1',models,vendors:[]})

test('rankings retain missing measurements at the end and use mobile deterministic tie ordering',()=>{
  const input=[model('z',20),model('b',null),model('a1',20),model('bad',-1),model('nan',NaN),model('infinite',Infinity),model('a-1',20),model('zero',0)]
  const rows=rankModels(input,'intelligence')
  assert.deepEqual(rows.slice(0,4).map(r=>[r.model.id,r.rank,r.fraction]),[['a-1',1,1],['a1',2,1],['z',3,1],['zero',4,0]])
  assert.ok(rows.slice(4).every(r=>r.rank===null&&r.value===null&&r.fraction===null))
  assert.equal(input[0]?.id,'z')
})

test('lower prices rank first and a zero minimum does not pretend paid models are free',()=>{
  const rows=rankModels([model('paid',1,2),model('free',1,0),model('cheap',1,.001)],'input')
  assert.deepEqual(rows.map(r=>[r.model.id,r.rank,r.fraction]),[['free',1,1],['cheap',2,.04],['paid',3,.04]])
  assert.equal(formatValue(.001,'input'),'$0.001')
  assert.equal(formatValue(.00000001,'output'),'$1e-8')
  assert.equal(formatValue(Infinity,'input'),'暂无数据')
})

test('only measured mobile metrics are offered, with an intelligence fallback for an empty snapshot',()=>{
  const entry=model('a',null);entry.metrics.legal=33
  assert.deepEqual(availableMetrics(board([entry])).map(m=>m.id),['legal','input','output'])
  assert.deepEqual(availableMetrics(board([])).map(m=>m.id),['intelligence'])
  assert.equal(METRICS.length,10)
})

test('token counts remain exact beyond safe integer and 64-bit limits',()=>{
  assert.equal(formatTokens('123456789012345678901234567890'),'123,456,789,012,345,678,901,234,567,890')
  assert.equal(formatTokens('0'),'0')
  for(const value of ['01','-1','1e9','9'.repeat(81),''])assert.equal(formatTokens(value),'暂无数据')
})

test('usage shares keep tiny nonzero values and movements require a published previous date',()=>{
  assert.equal(formatShare(.0000001),'<0.000001%')
  assert.equal(formatShare(.0000001,8),'0.0000001%')
  assert.equal(formatShare(12.5000),'12.5%')
  assert.equal(formatShare(100,0),'100%')
  const row={id:'a',name:'a',tokens:'1',rank:1,share_pct:1,new_entry:true,rank_change:3}
  assert.equal(movement(row,false),'暂无对比')
  assert.equal(movement(row,true),'新入榜')
  assert.equal(movement({...row,new_entry:false},true),'↑ 3')
  assert.equal(movement({...row,new_entry:false,rank_change:0},true),'—')
})

test('scatter uses complete measurements and actual 3:1 input/output pricing',()=>{
  const incomplete=model('missing',10);incomplete.pricing.output=null
  const points=scatterPoints(board([model('ok',60,1,5),model('free',20,0,0),model('invalid',-2),incomplete]))
  assert.deepEqual(points.map(p=>[p.id,p.weighted_price]),[['ok',2]])
  const axis=scatterPriceAxis(points)
  assert.ok(axis.lo<2&&axis.hi>2)
  assert.equal(logFraction(2,axis.lo,axis.hi).toFixed(5),'0.50000')
  assert.deepEqual(scatterScoreAxis(points),{lo:60,hi:65})
})

test('log axis pads the measured range and uses 1/2/3/5 ticks anchored to powers of ten',()=>{
  const points=[{weighted_price:.12},{weighted_price:3.8}] as ScatterPoint[]
  const axis=scatterPriceAxis(points)
  assert.ok(axis.lo>.1&&axis.lo<.12)
  assert.ok(axis.hi>3.8&&axis.hi<5)
  assert.deepEqual(scatterPriceTicks({lo:.12,hi:3.8}),[.2,.3,.5,1,2,3])
  assert.deepEqual(scatterPriceAxis([]),{lo:.1,hi:1})
})

test('frontier removes dominated/equal coordinates and medians follow the mobile advantage region',()=>{
  const point=(id:string,price:number,score:number):ScatterPoint=>({id,name:id,vendor_id:'v',weighted_price:price,intelligence:score})
  const points=[point('b',1,30),point('a',1,30),point('low',1,20),point('dominated',2,29),point('best',3,60)]
  assert.deepEqual(scatterFrontier(points).map(p=>p.id),['a','best'])
  assert.equal(median([1,2,3,8]),2.5)
  assert.equal(median([1,NaN,3]),2)
  assert.equal(median([]),null)
})


test('derived weighted pricing removes floating noise while source quotes remain exact',()=>{
  const entry=model('deepseek',42,.15,1.65)
  assert.equal(scatterPoints(board([entry]))[0]?.weighted_price,.525)
  assert.equal(entry.pricing.input,.15)
  assert.equal(entry.pricing.output,1.65)
  assert.equal(formatValue(entry.pricing.input,'input'),'$0.15')
  assert.equal(formatValue(.00000000123456789,'output'),'$1.23456789e-9')
})
