<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { scatterPoints, scatterPriceAxis, scatterPriceTicks, scatterScoreAxis, scatterFrontier, median, linearFraction, logFraction, vendorColor, formatValue, type Board, type ScatterPoint } from '@/lib/modelRankings'
const props=withDefaults(defineProps<{board:Board;full?:boolean}>(),{full:false})
defineEmits<{close:[]}>()
const vendor=ref(''),selected=ref(''),query=ref(''),showList=ref(false),showAll=ref(false)
const host=ref<HTMLElement|null>(null),width=ref(360),height=ref(320)
let observer:ResizeObserver|undefined
watch(host,element=>{observer?.disconnect();if(element){observer=new ResizeObserver(([entry])=>{if(entry&&entry.contentRect.width&&entry.contentRect.height){width.value=entry.contentRect.width;height.value=entry.contentRect.height}});observer.observe(element)}})
onBeforeUnmount(()=>observer?.disconnect())
const vendors=computed(()=>new Map(props.board.vendors.map(v=>[v.id,v.name])))
const ordered=computed(()=>scatterPoints(props.board).sort((a,b)=>b.intelligence-a.intelligence||a.id.localeCompare(b.id)))
const vendorIds=computed(()=>[...new Set(ordered.value.map(p=>p.vendor_id))])
const visible=computed(()=>props.full||!vendorIds.value.includes(vendor.value)?ordered.value:ordered.value.filter(p=>p.vendor_id===vendor.value))
const current=computed(()=>visible.value.find(p=>p.id===selected.value)||visible.value[0])
const frontier=computed(()=>scatterFrontier(visible.value))
const featured=computed(()=>{
  if(showAll.value||visible.value.length<=28)return visible.value
  const ids=new Set<string>(),names=new Set<string>()
  return [...frontier.value,...visible.value.filter(p=>{const name=p.name.split(' (')[0]!;if(names.has(name))return false;names.add(name);return true})].filter(p=>{if(ids.has(p.id))return false;ids.add(p.id);return true}).slice(0,28)
})
const highlighted=computed(()=>new Set([...featured.value.map(p=>p.id),...(current.value?[current.value.id]:[])]))
const priceAxis=computed(()=>scatterPriceAxis(visible.value)),scoreAxis=computed(()=>scatterScoreAxis(visible.value))
const plot=computed(()=>({left:34,top:16,width:Math.max(80,width.value-50),height:Math.max(80,height.value-48)}))
function x(price:number){return plot.value.left+plot.value.width*logFraction(price,priceAxis.value.lo,priceAxis.value.hi)}
function y(score:number){return plot.value.top+plot.value.height*(1-linearFraction(score,scoreAxis.value.lo,scoreAxis.value.hi))}
const medianPrice=computed(()=>median(visible.value.map(p=>p.weighted_price))??priceAxis.value.lo)
const medianScore=computed(()=>median(visible.value.map(p=>p.intelligence))??scoreAxis.value.lo)
const priceTicks=computed(()=>scatterPriceTicks(priceAxis.value).reduce<number[]>((kept,price)=>{if(!kept.length||x(price)-x(kept.at(-1)!)>=40)kept.push(price);return kept},[]))
const scoreTicks=computed(()=>[0,.25,.5,.75,1].map(t=>scoreAxis.value.lo+(scoreAxis.value.hi-scoreAxis.value.lo)*t))
const line=computed(()=>frontier.value.map(p=>`${x(p.weighted_price)},${y(p.intelligence)}`).join(' '))
const matches=computed(()=>visible.value.filter(p=>`${p.name} ${vendors.value.get(p.vendor_id)||p.vendor_id}`.toLowerCase().includes(query.value.trim().toLowerCase())))
function color(point:ScatterPoint){return vendorColor(vendors.value.get(point.vendor_id)||point.vendor_id)}
function pick(event:MouseEvent){
  const rect=(event.currentTarget as SVGSVGElement).getBoundingClientRect()
  const px=(event.clientX-rect.left)*width.value/rect.width,py=(event.clientY-rect.top)*height.value/rect.height
  const nearby=visible.value.map(p=>({p,d:(x(p.weighted_price)-px)**2+(y(p.intelligence)-py)**2})).filter(p=>p.d<=28**2).sort((a,b)=>a.d-b.d||a.p.id.localeCompare(b.p.id))
  if(!nearby[0])return
  const overlap=nearby.filter(p=>p.d<=nearby[0]!.d+8**2),index=overlap.findIndex(p=>p.p.id===current.value?.id)
  selected.value=overlap[index>=0?(index+1)%overlap.length:0]!.p.id
}
function shortText(text:string,maxWidth:number){let out='';for(const char of text){if((out+char).split('').reduce((sum,c)=>sum+(c.charCodeAt(0)>255?12:6.5),0)>maxWidth-12)return out+'…';out+=char}return out}
const labels=computed(()=>{
  if(!current.value)return []
  const top=props.full?featured.value:featured.value.filter((p,i,a)=>a.findIndex(q=>q.name.split(' (')[0]===p.name.split(' (')[0])===i).slice(0,3)
  const candidates=[current.value,...top].filter((p,i,a)=>a.findIndex(q=>q.id===p.id)===i)
  const result:{point:ScatterPoint;text:string;left:number;top:number;width:number;height:number}[]=[]
  for(const point of candidates){
    const text=point.name.replace('Default Fallback','fallback'),boxWidth=Math.min(props.full?220:150,plot.value.width,Math.max(44,Array.from(text).reduce((sum,c)=>sum+(c.charCodeAt(0)>255?12:6.5),0)+8)),boxHeight=20
    let placed=false
    for(let step=0;step<=(props.full?8:3)&&!placed;step++)for(const dy of step===0?[0]:[-step*22,step*22])for(const side of [1,-1]){
      if(placed)break
      const left=Math.max(plot.value.left,Math.min(plot.value.left+plot.value.width-boxWidth,x(point.weighted_price)+(side===1?9:-boxWidth-9)))
      const top=Math.max(plot.value.top,Math.min(plot.value.top+plot.value.height-boxHeight,y(point.intelligence)-10+dy))
      const pointInside=x(point.weighted_price)>=left-3&&x(point.weighted_price)<=left+boxWidth+3&&y(point.intelligence)>=top-3&&y(point.intelligence)<=top+boxHeight+3
      if(!pointInside&&!result.some(r=>left<r.left+r.width+3&&left+boxWidth+3>r.left&&top<r.top+r.height+3&&top+boxHeight+3>r.top)){result.push({point,text:shortText(text,boxWidth),left,top,width:boxWidth,height:boxHeight});placed=true}
    }
  }
  return result
})
</script>
<template><div class="scatter-plot" :class="{full}">
  <div v-if="!full" class="vendors" aria-label="散点图厂商筛选"><button :class="{on:!vendor}" @click="vendor=''">全部厂商</button><button v-for="id in vendorIds" :key="id" :class="{on:vendor===id}" :aria-pressed="vendor===id" @click="vendor=vendor===id?'':id"><i :style="{background:vendorColor(vendors.get(id)||id)}"/>{{ vendors.get(id)||id }}</button></div>
  <div class="chart-toolbar"><p>{{ full ? `AA 智能指数 v${board.index_version} · ${board.as_of.slice(0,10)} · 美元/百万 Token · 输入 75% + 输出 25% · 对数轴` : '浅绿：智能高于中位数、价格低于中位数 · 虚线：帕累托前沿' }}</p><template v-if="full"><button :aria-pressed="showAll" @click="showAll=!showAll">{{ showAll?`全部 ${visible.length}`:`精选 ${highlighted.size}/${visible.length}` }}</button><button :aria-expanded="showList" aria-label="搜索和选择模型" @click="showList=!showList">搜索</button><button aria-label="退出全屏" @click="$emit('close')">关闭</button></template></div>
  <p v-if="!visible.length" class="empty">暂无同时具备智能指数和完整价格的模型。</p>
  <div v-else class="chart-body"><div ref="host" class="chart-area"><svg :viewBox="`0 0 ${width} ${height}`" role="img" aria-label="智能与价格散点图，横轴为对数价格，纵轴为智能指数，可从模型列表选择模型" @click="pick">
    <rect :x="plot.left" :y="plot.top" :width="Math.max(0,x(medianPrice)-plot.left)" :height="Math.max(0,y(medianScore)-plot.top)" class="advantage" />
    <g v-for="score in scoreTicks" :key="score"><line :x1="plot.left" :x2="plot.left+plot.width" :y1="y(score)" :y2="y(score)" class="grid"/><text :x="plot.left-5" :y="y(score)+4" text-anchor="end" class="axis">{{ formatValue(score) }}</text></g>
    <line :x1="plot.left" :x2="plot.left" :y1="plot.top" :y2="plot.top+plot.height" class="grid"/>
    <text v-for="price in priceTicks" :key="price" :x="x(price)" :y="plot.top+plot.height+20" text-anchor="middle" class="axis">{{ formatValue(price,'input') }}</text>
    <polyline v-if="frontier.length>1" :points="line" class="frontier" />
    <line v-for="label in labels" :key="`line-${label.point.id}`" :x1="x(label.point.weighted_price)" :y1="y(label.point.intelligence)" :x2="Math.max(label.left,Math.min(label.left+label.width,x(label.point.weighted_price)))" :y2="Math.max(label.top,Math.min(label.top+label.height,y(label.point.intelligence)))" class="leader"/>
    <circle v-for="point in visible" :key="point.id" :cx="x(point.weighted_price)" :cy="y(point.intelligence)" r="3.5" :fill="highlighted.has(point.id)?color(point):'var(--ink-3)'" :opacity="highlighted.has(point.id)?1:.3"><title>{{ point.name }}：智能 {{ point.intelligence }}，加权价格 {{ formatValue(point.weighted_price,'input') }}</title></circle>
    <circle v-if="current" :cx="x(current.weighted_price)" :cy="y(current.intelligence)" r="8" :stroke="color(current)" fill="none" stroke-width="1.5"/>
    <g v-for="label in labels" :key="label.point.id" class="point-name" @click.stop="selected=label.point.id"><title>{{ label.point.name }}</title><rect :x="label.left" :y="label.top" :width="label.width" :height="label.height" rx="3" :class="{active:label.point.id===current?.id}"/><text :x="label.left+4" :y="label.top+14" :font-weight="label.point.id===current?.id?600:400">{{ label.text }}</text></g>
  </svg></div>
  <aside v-if="full&&showList" class="search-list"><input v-model="query" aria-label="搜索图中模型" placeholder="搜索图中模型" type="search"><p>{{ matches.length }} 个模型</p><button v-for="point in matches" :key="point.id" :class="{on:point.id===current?.id}" @click="selected=point.id"><i :style="{background:color(point)}"/><span>{{ point.name }}<small>{{ formatValue(point.intelligence) }} 分 · {{ formatValue(point.weighted_price,'input') }}</small></span></button><p v-if="!matches.length">没有匹配模型。</p></aside></div>
  <div v-if="current" class="selected-detail"><strong>{{ current.name }}</strong><p>{{ vendors.get(current.vendor_id)||current.vendor_id }} · 智能 {{ formatValue(current.intelligence) }} · 加权价格 {{ formatValue(current.weighted_price,'input') }}/百万 Token</p></div>
  <div v-if="!full" class="model-names" aria-label="选择图中模型"><button v-for="point in visible" :key="point.id" :aria-pressed="point.id===current?.id" :class="{on:point.id===current?.id}" @click="selected=point.id"><i :style="{background:color(point)}"/>{{ point.name }}</button></div>
  <p v-if="!full" class="hint">点名称或圆点定位模型 · 重叠圆点可连续点选</p>
</div></template>
<style scoped>
.scatter-plot { display:flex; flex-direction:column; min-width:0; color:var(--ink); } .vendors,.model-names { display:flex; gap:6px; overflow:auto; padding:4px 0; } button { flex:none; display:inline-flex; align-items:center; gap:5px; min-height:44px; padding:6px 10px; background:var(--surface-2); color:var(--ink-3); border:0; border-radius:6px; font-size:12px; cursor:pointer; } button.on { color:var(--ink); background:color-mix(in srgb,var(--brand) 10%,var(--surface)); } i { flex:none; display:inline-block; width:6px; height:6px; border-radius:50%; } .chart-toolbar { display:flex; align-items:center; gap:4px; min-height:36px; } .chart-toolbar p { flex:1; min-width:0; font-size:12px; color:var(--ink-3); line-height:1.5; margin:4px 0; } .chart-body { display:flex; min-width:0; gap:8px; } .chart-area { height:330px; min-width:0; flex:1; } svg { display:block; width:100%; height:100%; overflow:visible; } .axis { fill:var(--ink-3);font-size:12px; } .grid { stroke:var(--line);stroke-width:.5; } .advantage { fill:#6edb83;opacity:.16; } .frontier { fill:none;stroke:var(--ink-2);stroke-width:1;stroke-dasharray:3 4; } .leader { stroke:var(--ink-3);stroke-opacity:.3;stroke-width:.5; } .point-name { cursor:pointer; } .point-name rect { fill:transparent; } .point-name rect.active { fill:var(--surface);fill-opacity:.94; } .point-name text { fill:var(--ink);font-size:12px; } .selected-detail { background:var(--surface-2); border-radius:8px; padding:8px; margin-top:4px; font-size:12px; overflow-wrap:anywhere; } .selected-detail p,.hint { color:var(--ink-3);font-size:12px;line-height:1.6;margin:4px 0 0; } .empty { color:var(--ink-3);font-size:14px;padding:24px 0; } .full { height:100%; } .full .chart-body { flex:1;min-height:0; } .full .chart-area { height:100%;min-height:120px; } .search-list { flex:0 0 216px; background:var(--surface-2); padding:8px; border-radius:8px; overflow:auto; } .search-list input { width:100%;min-width:0;background:var(--surface);color:var(--ink);border:0;border-radius:6px;font-size:16px;padding:8px; } .search-list p { font-size:12px;color:var(--ink-3); } .search-list button { width:100%;text-align:left;min-width:0;align-items:flex-start; } .search-list button i { margin-top:5px; } .search-list button span { overflow-wrap:anywhere; } .search-list small { display:block;font-size:12px;color:var(--ink-3);margin-top:4px; }
@media(max-width:600px) { .full .search-list { flex-basis:42%;padding:4px; } .full .chart-toolbar p { font-size:12px; } }
@media(orientation:landscape) and (max-height:600px) { .full .selected-detail { display:none; } .full .chart-toolbar p { overflow:hidden;text-overflow:ellipsis;white-space:nowrap; } }
</style>
