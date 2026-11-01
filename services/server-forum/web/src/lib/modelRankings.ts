/** Public v2 DTOs and algorithms mirror mobile community-core/ModelRankings.kt. */
export interface Model { id:string; slug?:string; name:string; vendor_id:string; release_date?:string|null; metrics:Record<string,number|null|undefined>; pricing:{ input:number|null; output:number|null }; performance:{ output_tokens_per_second:number|null; time_to_first_token_seconds?:number|null } }
export interface Board { source:string; source_url?:string; as_of:string; index_version:string; schema_version?:string; tier?:string; price_unit?:string; vendors:{id:string;name:string;country?:string|null}[]; models:Model[]; updated_at?:string|null; sample?:boolean }
export interface UsageRow { id:string; name:string; tokens:string; rank:number; share_pct:number; previous_rank?:number|null; rank_change?:number|null; new_entry?:boolean }
export interface Usage { source:string; source_url?:string; date:string; as_of:string; total_tokens:string; other_tokens:string; models:UsageRow[]; previous_date_available:boolean; updated_at?:string|null; version?:string; period?:string; timezone?:string; token_unit?:string; sample?:boolean }
export interface Metric { id:string; label:string; unit:string; higherIsBetter:boolean }
export const METRICS:readonly Metric[] = [
  {id:'intelligence',label:'智能指数',unit:'AA 智能指数',higherIsBetter:true},
  ...[['finance_and_accounting','财务会计'],['strategy_and_ops','战略运营'],['legal','法律'],['healthcare_and_medical','医疗健康'],['engineering','工程'],['economics','经济']].map(([id,label])=>({id:id!,label:label!,unit:'能力指数',higherIsBetter:true})),
  {id:'input',label:'输入价格',unit:'美元 / 百万 Token',higherIsBetter:false},
  {id:'output',label:'输出价格',unit:'美元 / 百万 Token',higherIsBetter:false},
  {id:'speed',label:'输出速度',unit:'Token / 秒',higherIsBetter:true},
]
const compareIds = (a:string,b:string) => a<b ? -1 : a>b ? 1 : 0
const valid = (value:unknown):value is number => typeof value === 'number' && Number.isFinite(value) && value >= 0
export function metricValue(model:Model,metricId:string):number|null {
  const value = metricId === 'speed' ? model.performance.output_tokens_per_second : metricId === 'input' ? model.pricing.input : metricId === 'output' ? model.pricing.output : model.metrics[metricId]
  return valid(value) ? value : null
}
export interface ModelRank { model:Model; value:number|null; rank:number|null; fraction:number|null }
export function rankModels(models:Model[],metricId:string):ModelRank[] {
  const higher = METRICS.find(m=>m.id===metricId)?.higherIsBetter ?? true
  const sorted = models.map(model=>({model,value:metricValue(model,metricId)})).sort((a,b)=>a.value == null ? b.value == null ? compareIds(a.model.id,b.model.id) : 1 : b.value == null ? -1 : (higher ? b.value-a.value : a.value-b.value) || compareIds(a.model.id,b.model.id))
  const values = sorted.flatMap(row=>row.value == null ? [] : [row.value])
  const max = Math.max(0,...values), min = values.length ? Math.min(...values) : 0
  let rank = 0
  return sorted.map(({model,value})=>({model,value,rank:value == null ? null : ++rank,fraction:value == null ? null : higher ? max === 0 ? 0 : value/max : value===0 ? 1 : min===0 ? .04 : min/value}))
}
export function availableMetrics(board:Board):Metric[] { const found = METRICS.filter(m=>board.models.some(model=>metricValue(model,m.id)!=null)); return found.length ? found : [METRICS[0]!] }
export function formatValue(value:number|null|undefined,metricId='intelligence'):string { return !valid(value) ? '暂无数据' : metricId==='input'||metricId==='output' ? `$${value}` : new Intl.NumberFormat('zh-CN',{maximumFractionDigits:1}).format(value) }
export function formatTokens(value:string):string { return typeof value==='string' && /^(0|[1-9]\d{0,79})$/.test(value) ? value.replace(/\B(?=(\d{3})+(?!\d))/g,',') : '暂无数据' }
export function formatShare(value:number,digits=6):string { if (!valid(value)) return '暂无数据'; const precision=Math.min(10,Math.max(0,Math.floor(digits))); return value>0 && value<10**-precision ? `<${(10**-precision).toFixed(precision)}%` : `${precision===0 ? Math.round(value) : value.toFixed(precision).replace(/(\.\d*?)0+$/,'$1').replace(/\.$/,'')}%` }
export function movement(row:UsageRow,previousDateAvailable:boolean):string { return !previousDateAvailable ? '暂无对比' : row.new_entry ? '新入榜' : row.rank_change == null || !Number.isFinite(row.rank_change) ? '暂无对比' : row.rank_change>0 ? `↑ ${row.rank_change}` : row.rank_change<0 ? `↓ ${-row.rank_change}` : '—' }
export function vendorColor(name:string):string {
  const n=name.toLowerCase()
  const colors:[string,string][]=[['openai','#10A37F'],['anthropic','#D97757'],['google','#4285F4'],['deepseek','#4D6BFE'],['z-ai','#5366D3'],['zhipu','#5366D3'],['meta','#00A3D9'],['tencent','#2F6BFF'],['spacex','#8B5CF6'],['xai','#8B5CF6'],['alibaba','#FF7A00'],['qwen','#615CED'],['xiaomi','#FF6900'],['inclusion','#49B3F1'],['stepfun','#00B8A9'],['reflection','#155C2C'],['nvidia','#76B900'],['thinking','#6B7280'],['mistral','#EF9538']]
  return colors.find(([key])=>n.includes(key))?.[1] || '#4E83D7'
}
export interface ScatterPoint { id:string; name:string; vendor_id:string; intelligence:number; weighted_price:number }
export interface AxisRange {lo:number;hi:number}
export function scatterPoints(board:Board):ScatterPoint[] { return board.models.flatMap(model=>{
  const score=metricValue(model,'intelligence'),input=metricValue(model,'input'),output=metricValue(model,'output')
  if (score==null || input==null || output==null) return []
  // Normalize only the derived blend; original input/output quotes remain untouched.
  const price=Number((input*.75+output*.25).toPrecision(15))
  return Number.isFinite(price)&&price>0 ? [{id:model.id,name:model.name,vendor_id:model.vendor_id,intelligence:score,weighted_price:price}] : []
}) }
export function scatterPriceAxis(points:ScatterPoint[]):AxisRange {
  const prices=points.map(p=>p.weighted_price).filter(v=>Number.isFinite(v)&&v>0)
  if (!prices.length) return {lo:.1,hi:1}
  const lo=Math.min(...prices),hi=Math.max(...prices),padding=Math.max((Math.log10(hi)-Math.log10(lo))*.04,.02)
  const lower=10**(Math.log10(lo)-padding),upper=10**(Math.log10(hi)+padding)
  return {lo:lower>0&&Number.isFinite(lower)?lower:lo,hi:Number.isFinite(upper)?upper:hi}
}
export function scatterPriceTicks(range:AxisRange):number[] {
  if (!(range.lo>0&&range.hi>range.lo&&Number.isFinite(range.hi))) return []
  const ticks:number[]=[]
  for(let exponent=Math.floor(Math.log10(range.lo));exponent<=Math.ceil(Math.log10(range.hi));exponent++) for(const scale of [1,2,3,5]) { const value=Number(`${scale}e${exponent}`);if(value>=range.lo&&value<=range.hi&&Number.isFinite(value))ticks.push(value) }
  return ticks.length ? [...new Set(ticks)] : [range.lo,range.hi]
}
export function scatterFrontier(points:ScatterPoint[]):ScatterPoint[] { let best=-Infinity;return [...points].sort((a,b)=>a.weighted_price-b.weighted_price||b.intelligence-a.intelligence||compareIds(a.id,b.id)).filter(p=>{if(p.intelligence>best){best=p.intelligence;return true}return false}) }
export function scatterScoreAxis(points:ScatterPoint[]):AxisRange { const values=points.map(p=>p.intelligence).filter(valid);if(!values.length)return {lo:0,hi:5};const lo=Math.floor(Math.min(...values)/5)*5;return {lo,hi:Math.max(lo+5,Math.ceil(Math.max(...values)/5)*5)} }
export function median(values:number[]):number|null { const sorted=values.filter(Number.isFinite).sort((a,b)=>a-b);const n=sorted.length;return n ? sorted[Math.floor((n-1)/2)]!/2+sorted[Math.floor(n/2)]!/2 : null }
export function linearFraction(value:number,lo:number,hi:number):number {return hi>lo ? Math.max(0,Math.min(1,(value-lo)/(hi-lo))) : .5}
export function logFraction(value:number,lo:number,hi:number):number {return value>0&&lo>0&&hi>lo ? linearFraction(Math.log10(value),Math.log10(lo),Math.log10(hi)) : .5}
