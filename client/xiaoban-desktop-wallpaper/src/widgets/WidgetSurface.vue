<script setup lang="ts">
import {computed,onActivated,onDeactivated,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Check,ChevronLeft,ChevronRight,Heart,MapPin,Settings2,Plus,Sunrise,Leaf,ArrowUp,ArrowDown,Volume2,VolumeX,Trash2,Pencil,List} from 'lucide-vue-next';
import {desktop,invoke} from '../bridge.ts';
import {baseSize,calendarDays,type DesktopWidget,type WidgetPayload,type WidgetContent,type LivingData,type NoteData,type ReminderItem,type TodoItem} from './model.ts';
import {dateKey,lunarDate} from './calendar.ts';
import {emptyNote,emptyCountdown,countdownValue,occursOn,itemDone,toggleItem,scheduleLabel} from './content.ts';
import ItemForm from './ItemForm.vue';
import {cornerResize} from './resize.ts';
import {customWidgetDocument,isCustomDragMessage} from './custom-drag.ts';
import './surface.css';
const props=withDefaults(defineProps<{widget:DesktopWidget;preview?:boolean;demo?:boolean;editing?:boolean;payload?:WidgetPayload;renderScale?:number;save?:(c:WidgetContent)=>Promise<void>}>(),{preview:false,demo:false,editing:false,renderScale:1});
const emit=defineEmits<{settings:[]}>();
const now=ref(new Date()),error=ref(''),saving=ref(false),addOpen=ref(false),editingItem=ref<TodoItem|ReminderItem>();
const size=computed(()=>baseSize(props.widget.kind,props.widget.custom)),clock=computed(()=>now.value.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit',hour12:false}));
const customFrame=ref<HTMLIFrameElement>();
const surfaceScale=computed(()=>props.widget.kind==='custom'?props.renderScale:1);
function customLayout(){customFrame.value?.contentWindow?.postMessage({type:'xiaoban-widget-layout',scale:props.renderScale,width:size.value.width,height:size.value.height},'*');}
watch(()=>props.renderScale,customLayout,{flush:'post'});
const customDocument=computed(()=>{
  const html=props.widget.custom?.html||'<div style="color:#999;padding:12px;font-size:12px">未找到小组件内容</div>';
  return customWidgetDocument(html,!props.preview);
});
const lunarLabel=computed(()=>{const year=new Intl.DateTimeFormat('zh-CN-u-ca-chinese',{year:'numeric'}).formatToParts(now.value).find(p=>(p.type as string)==='yearName')?.value;return `${year?year+'年 ':''}${lunarDate(now.value).full}`;});
const greeting=computed(()=>now.value.getHours()<6?'夜深了':now.value.getHours()<12?'上午好':now.value.getHours()<18?'下午好':'晚上好');
const locked=computed(()=>props.editing),living=computed<LivingData>(()=>props.widget.living??{count:0,symbol:'116.00700',reminders:[]});
const note=computed(()=>props.widget.note??emptyNote()),monthOffset=ref(0),selectedDay=ref(dateKey(now.value));
const month=computed(()=>new Date(now.value.getFullYear(),now.value.getMonth()+monthOffset.value,1));
const days=computed(()=>calendarDays(month.value.getFullYear(),month.value.getMonth()));
const countdown=computed(()=>props.widget.countdown??emptyCountdown()),remaining=computed(()=>countdownValue(countdown.value,now.value));
const events=computed(()=>living.value.reminders.filter(e=>occursOn(e.at,e.repeat,now.value)).sort((a,b)=>a.at.slice(11).localeCompare(b.at.slice(11))));
const timeNow=computed(()=>now.value.toTimeString().slice(0,5));
const isCurrent=(event:ReminderItem)=>props.demo?event.id==='demo1':!itemDone(event,now.value)&&event.at.slice(11)<=timeNow.value&&timeNow.value<(event.end||'23:59');
const weather=ref<{text:string;source:string;updatedAt:number}|null>(null),forecast=ref<{date:string;high:number;low:number;code:number}[]>([]),weatherBusy=ref(false);
const temperature=computed(()=>props.demo?'16':weather.value?.text.match(/(-?\d+(?:\.\d+)?)°C/)?.[1]??'—');
const weatherText=computed(()=>props.demo?'多云':weather.value?.text.match(/：([^，]+)，/)?.[1]??'等待天气');
const highLow=computed(()=>props.demo?{high:21,low:8}:forecast.value[0]);
type Air={aqi:number;pm25:number|null;pm10:number|null;modelAt:number;updatedAt:number;standard:string;source:string};
const air=ref<Air|null>(null),airError=ref('');
const airLabel=computed(()=>{if(props.demo)return '空气优';if(!air.value)return airError.value?'空气数据暂不可用':'空气质量待更新';const a=air.value.aqi;return a<=50?'空气优':a<=100?'空气良':a<=150?'敏感人群注意':a<=200?'空气不健康':a<=300?'空气很不健康':'空气危险';});
const airTitle=computed(()=>air.value?`${air.value.standard} ${Math.round(air.value.aqi)} · PM2.5 ${air.value.pm25??'—'} μg/m³ · ${new Date(air.value.modelAt*1000).toLocaleString('zh-CN')} · ${air.value.source}`:airError.value);
let alive=true,weatherQueued=false,quoteQueued=false,listQueued=false;
async function refreshWeather(){
 if(props.preview||!desktop||!props.widget.city||document.hidden)return;
 if(weatherBusy.value){weatherQueued=true;return;}weatherBusy.value=true;error.value='';airError.value='';const key=JSON.stringify(props.widget.city);
 const requests=[invoke<typeof weather.value>('get_widget_weather'),invoke<{days:typeof forecast.value}>('get_widget_forecast'),invoke<Air>('get_widget_air_quality')] as const;
 const results=await Promise.allSettled(requests);
 if(alive&&key===JSON.stringify(props.widget.city)){
  const [w,f,a]=results;
  if(w.status==='fulfilled')weather.value=w.value;else error.value=String(w.reason);
  if(f.status==='fulfilled')forecast.value=f.value.days;
  if(a.status==='fulfilled')air.value=a.value;else airError.value=String(a.reason);
 }
 weatherBusy.value=false;if(weatherQueued){weatherQueued=false;void refreshWeather();}
}
watch(()=>JSON.stringify(props.widget.city),()=>{weather.value=null;forecast.value=[];air.value=null;void refreshWeather();});
type Quote={symbol:string;name:string;price:number;change:number;percent:number;points:number[];updatedAt:string;period:string;source:string};
const quote=ref<Quote|null>(null),quoteBusy=ref(false),period=ref('1D');
const periods=[{value:'1D',label:'1天'},{value:'5D',label:'1周'},{value:'1M',label:'1月'},{value:'1Y',label:'1年'}];
const demoPrices=[302,303,306,305,307,305,304,303,304,303,305,308,306,305,307,306,308,310,309,310,312,313,312,314,313,315,317,316,314,315,314,316,315,317,316,318,317,320,319,319.6];
const displayQuote=computed(()=>props.demo?{name:'腾讯控股',symbol:'116.00700',price:309.6,change:4.8,percent:1.58,points:demoPrices,updatedAt:'',period:period.value,source:'展示数据'}:quote.value);
const up=computed(()=>(displayQuote.value?.change??0)>=0);
const chart=computed(()=>{const s=displayQuote.value?.points??[];if(s.length<2)return '';const min=Math.min(...s),max=Math.max(...s),range=Math.max(.001,max-min);return s.map((v,i)=>`${(i/(s.length-1)*340).toFixed(2)},${(120-(v-min)/range*110).toFixed(2)}`).join(' ');});
async function loadQuote(){
 if(props.preview||!desktop||props.widget.kind!=='stocks'||document.hidden)return;
 if(quoteBusy.value){quoteQueued=true;return;}quoteBusy.value=true;error.value='';const requested=living.value.symbol,range=period.value;
 try{const result=await invoke<Quote>('get_widget_stocks',{period:range,symbol:requested});if(alive&&result.symbol===living.value.symbol&&result.period===period.value)quote.value=result;}
 catch(e){if(alive&&requested===living.value.symbol&&range===period.value)error.value=String(e);}finally{quoteBusy.value=false;if(quoteQueued){quoteQueued=false;void loadQuote();}}
}
watch([()=>living.value.symbol,period],()=>{quote.value=null;void loadQuote();});
async function save(content:WidgetContent){
 if(props.editing||saving.value)return false;error.value='';saving.value=true;
 try{if(props.save)await props.save(content);else if(desktop&&!props.preview)await invoke('save_desktop_widget_content',{content});else throw Error('请添加到桌面后使用');return true;}
 catch(e){error.value=String(e);return false;}finally{saving.value=false;}
}
function settings(){if(props.preview){if(!props.demo)emit('settings');return;}void invoke('show_widget_settings').catch(e=>error.value=String(e));}
function openItem(item?:TodoItem|ReminderItem){editingItem.value=item;addOpen.value=true;}
function taskWhen(item:TodoItem){return scheduleLabel(item.date||'',item.time||'',item.repeat,now.value);}
async function toggleTask(id:string){await save({note:{...note.value,items:note.value.items.map(i=>i.id===id?toggleItem(i,now.value):i)}});}
async function removeTask(id:string){await save({note:{...note.value,items:note.value.items.filter(i=>i.id!==id)}});}
async function commitItem(item:TodoItem|ReminderItem){
 const todo=props.widget.kind==='notes',items=todo?note.value.items:living.value.reminders;
 if(!items.some(i=>i.id===item.id)&&items.length>=12){error.value='最多保存 12 项，请先删除不需要的事项';return;}
 const updated=items.some(i=>i.id===item.id)?items.map(i=>i.id===item.id?item:i):[...items,item];
 const content:WidgetContent=todo?{note:{...note.value,mode:'todo',items:updated as TodoItem[]}}:{living:{...living.value,reminders:updated as ReminderItem[]}};
 if(await save(content)){addOpen.value=false;editingItem.value=undefined;}
}
async function removeEvent(id:string){await save({living:{...living.value,reminders:living.value.reminders.filter(i=>i.id!==id)}});}
async function toggleEvent(event:ReminderItem){await save({living:{...living.value,reminders:living.value.reminders.map(i=>i.id===event.id?toggleItem(i,now.value):i)}});}
const watchlist=computed(()=>living.value.symbols?.length?living.value.symbols:[living.value.symbol]);
type ListQuote={symbol:string;name:string;price:number;change:number;percent:number;updatedAt:number|null};
const listOpen=ref(false),listBusy=ref(false),listError=ref(''),quotes=ref<ListQuote[]>([]);
async function loadWatchlist(){
 if(props.preview||!desktop||props.widget.kind!=='stocks'||document.hidden)return;
 if(listBusy.value){listQueued=true;return;}listBusy.value=true;listError.value='';const key=watchlist.value.join(',');
 try{const r=await invoke<{quotes:ListQuote[]}>('get_widget_stock_watchlist');if(alive&&key===watchlist.value.join(','))quotes.value=r.quotes;}catch(e){if(alive&&key===watchlist.value.join(','))listError.value=String(e);}finally{listBusy.value=false;if(listQueued){listQueued=false;void loadWatchlist();}}
}
watch(()=>watchlist.value.join(','),()=>{quotes.value=[];void loadWatchlist();});
function quoteFor(symbol:string){return quotes.value.find(q=>q.symbol===symbol);}
async function chooseStock(symbol:string){if(symbol!==living.value.symbol){if(!await save({living:{...living.value,symbol,symbols:watchlist.value}}))return;}listOpen.value=false;}
const count=ref(props.widget.living?.count??0),sound=ref(props.widget.living?.sound??true),striking=ref(false),pulse=ref(0);let audio:AudioContext|null=null,hitTimer:ReturnType<typeof setTimeout>|undefined;
watch(()=>props.widget.living?.sound,v=>sound.value=v??true);
async function toggleSound(){const next=!sound.value;soundBusy=true;try{if(await save({living:{...living.value,sound:next,count:count.value}}))sound.value=next;}finally{soundBusy=false;void persistHits();}}
let hitBusy=false,soundBusy=false,acknowledged=count.value;
watch(()=>props.widget.living?.count,v=>{acknowledged=v??0;if(!hitBusy&&!soundBusy)count.value=acknowledged;});
async function persistHits(){
 if(hitBusy||soundBusy)return;hitBusy=true;
 try{while(count.value!==acknowledged){const target=count.value;if(!await save({living:{...living.value,sound:sound.value,count:target}})){count.value=acknowledged;break;}acknowledged=target;}}
 finally{hitBusy=false;}
}
function tap(){
 if(locked.value)return;
 count.value=Math.min(9999999,count.value+1);striking.value=false;pulse.value++;
 requestAnimationFrame(()=>{striking.value=true;});clearTimeout(hitTimer);hitTimer=setTimeout(()=>striking.value=false,240);
 if(sound.value){try{audio??=new AudioContext();void audio.resume();const o=audio.createOscillator(),g=audio.createGain(),t=audio.currentTime;o.frequency.setValueAtTime(420,t);o.frequency.exponentialRampToValueAtTime(160,t+.16);g.gain.setValueAtTime(.18,t);g.gain.exponentialRampToValueAtTime(.001,t+.2);o.connect(g);g.connect(audio.destination);o.start(t);o.stop(t+.21);}catch{}}
 void persistHits();
}
const resizer=cornerResize(phase=>invoke('resize_desktop_widget',{phase}),e=>error.value=String(e));
const resizing=ref(false);
function startResize(e:PointerEvent){
 if(e.button!==0||props.preview||!props.payload)return;
 const handle=e.currentTarget as HTMLElement;handle.setPointerCapture?.(e.pointerId);resizing.value=true;void resizer.begin();
}
function finishResize(cancel=false){if(!resizing.value)return;resizing.value=false;resizer.finish(cancel);}
function resizeKey(e:KeyboardEvent){if(e.key==='Escape'&&resizing.value){e.preventDefault();finishResize(true);}}
let drag:{x:number;y:number;grabX:number;grabY:number;moved:boolean}|null=null,suppressClick=false,moving=false,pending:{grabX:number;grabY:number;commit:boolean}|null=null;
let clickTimer:ReturnType<typeof setTimeout>|undefined;
async function movePending(){if(moving)return;moving=true;try{while(pending){const p=pending;pending=null;await invoke('move_desktop_widget',p);}}catch(e){error.value=String(e);}finally{moving=false;}}
function startDrag(e:PointerEvent){if(e.button!==0||props.preview||!props.payload)return;if((e.target as Element).closest('button,input,textarea,select,form,[data-no-drag]'))return;suppressClick=false;drag={x:e.screenX,y:e.screenY,grabX:e.clientX/Math.max(1,innerWidth),grabY:e.clientY/Math.max(1,innerHeight),moved:false};(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);}
function move(e:Pick<PointerEvent,'screenX'|'screenY'>,commit=false){if(!drag)return;if(Math.hypot(e.screenX-drag.x,e.screenY-drag.y)>5)drag.moved=true;if(drag.moved){suppressClick=true;pending={grabX:drag.grabX,grabY:drag.grabY,commit};void movePending();}if(commit){drag=null;clickTimer=setTimeout(()=>suppressClick=false,0);}}
function customDrag(event:MessageEvent){
 if(props.preview||!props.payload||!customFrame.value?.contentWindow||event.source!==customFrame.value.contentWindow)return;
 if(event.data?.type==='xiaoban-widget-settings'){settings();return;}
 if(props.editing||!isCustomDragMessage(event.data))return;
 const data=event.data;
 if(data.phase==='start'){
  const rect=customFrame.value.getBoundingClientRect();
  suppressClick=false;drag={x:data.screenX,y:data.screenY,grabX:(rect.left+data.x*rect.width)/Math.max(1,innerWidth),grabY:(rect.top+data.y*rect.height)/Math.max(1,innerHeight),moved:false};
 }else move(data,data.phase==='end');
}
onMounted(()=>window.addEventListener('message',customDrag));
onBeforeUnmount(()=>window.removeEventListener('message',customDrag));
function capture(e:MouseEvent){if((e.target as Element).closest('.widget-settings-button'))return;if(suppressClick||props.editing){e.stopImmediatePropagation();e.preventDefault();}}
let clockTimer:ReturnType<typeof setTimeout>|undefined,networkTimer:ReturnType<typeof setInterval>|undefined,active=true;
function scheduleClock(){if(!active||clockTimer)return;clockTimer=setTimeout(()=>{clockTimer=undefined;now.value=new Date();scheduleClock();},60000-Date.now()%60000+20);}
function startTimers(){scheduleClock();if(!props.preview&&(props.widget.kind==='weather'||props.widget.kind==='stocks')&&!networkTimer)networkTimer=setInterval(visible,props.widget.kind==='stocks'?60000:1800000);}
function stopTimers(){clearTimeout(clockTimer);clearInterval(networkTimer);clockTimer=undefined;networkTimer=undefined;}
function visible(){if(!active)return;now.value=new Date();if(!document.hidden){if(props.widget.kind==='weather')void refreshWeather();else if(props.widget.kind==='stocks'){void loadQuote();void loadWatchlist();}}}
onMounted(()=>{startTimers();if(props.widget.kind==='weather')void refreshWeather();if(props.widget.kind==='stocks'){void loadQuote();void loadWatchlist();}document.addEventListener('visibilitychange',visible);document.addEventListener('keydown',resizeKey);});
onActivated(()=>{if(!active){active=true;startTimers();visible();}});
onDeactivated(()=>{active=false;stopTimers();});
onBeforeUnmount(()=>{resizer.finish(true);document.removeEventListener('keydown',resizeKey);alive=false;stopTimers();clearTimeout(hitTimer);clearTimeout(clickTimer);document.removeEventListener('visibilitychange',visible);void audio?.close();});
</script>
<template>
<article class="xb-widget" :class="[widget.kind,{editing,resizing,falling:!up}]" :style="{width:`${size.width*surfaceScale}px`,height:`${size.height*surfaceScale}px`,'--widget-scale':widget.kind==='custom'?1:widget.scale}" @pointerdown="startDrag" @pointermove="move($event)" @pointerup="move($event,true)" @pointercancel="move($event,true)" @click.capture="capture" @contextmenu.prevent.stop="settings">
 <template v-if="widget.kind==='clock'">
  <div class="clock-art" aria-hidden="true"/><div class="clock-greeting"><Sunrise :size="31"/><div><span>{{ greeting }}</span><small>新的一天，也要好好生活</small></div></div>
  <div class="clock-time">{{ clock }}</div><div class="clock-date">{{ now.getMonth()+1 }}月{{ now.getDate() }}日 <span>{{ now.toLocaleDateString('zh-CN',{weekday:'long'}) }}</span></div><p class="clock-lunar">{{ lunarLabel }}</p>
 </template>
 <template v-else-if="widget.kind==='weather'">
  <header><span class="city-label"><MapPin :size="21" fill="currentColor"/>{{ demo?'杭州市':widget.city?.name||'选择城市' }}</span></header>
  <div class="weather-art" :class="{'is-rain':weatherText.includes('雨'),'is-sun':weatherText==='晴'}" aria-hidden="true"><i class="weather-sun"/><i class="weather-cloud"/></div>
  <div class="weather-temperature">{{ temperature }}<sup>°C</sup></div><div class="weather-description">{{ weatherText }} <span class="air-quality" :title="airTitle">{{ airLabel }}<small v-if="!demo&&air">US AQI {{ Math.round(air.aqi) }}</small><Leaf v-if="demo||air&&air.aqi<=50" :size="17" fill="#9bc378"/></span></div>
  <div v-if="highLow" class="weather-range"><ArrowUp :size="18"/>{{ highLow.high }}° <ArrowDown :size="18"/>{{ highLow.low }}°</div>
  <button v-if="!preview&&!widget.city" class="inline-link weather-configure" @click="settings">设置天气城市</button><small v-if="!preview&&weather" class="data-source">{{ weather.source }} · {{ new Date(weather.updatedAt*1000).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'}) }}</small><small v-if="weatherBusy" class="data-source">正在更新…</small>
 </template>
 <template v-else-if="widget.kind==='notes'">
  <header><h2>待办事项</h2><div class="widget-header-actions"><button class="add-button" :disabled="locked||saving" aria-label="添加待办" @click="openItem()"><Plus/></button></div></header>
  <div class="todo-list"><div v-for="item in note.items" :key="item.id" class="todo-row" :class="{done:itemDone(item,now)}"><button class="check-circle" :aria-pressed="itemDone(item,now)" :aria-label="`${itemDone(item,now)?'恢复':'完成'}${item.text}`" :disabled="locked||saving" @click="toggleTask(item.id)"><Check v-if="itemDone(item,now)" :size="19"/></button><button class="item-text" :disabled="locked||saving" aria-label="编辑待办" @click="openItem(item)">{{ item.text }}</button><small v-if="item.time||item.date">{{ taskWhen(item) }}</small><button class="row-remove" :disabled="locked||saving" aria-label="删除待办" @click="removeTask(item.id)"><Trash2 :size="13"/></button></div><p v-if="!note.items.length" class="empty-state">把今天想做的事，记在这里。<button @click="openItem()">添加第一件小事</button></p></div>
 </template>
 <template v-else-if="widget.kind==='reminders'">
  <header><h2>今日日程</h2><div class="widget-header-actions"><button class="agenda-date" aria-label="添加日程" :disabled="locked" @click="openItem()">{{ now.getMonth()+1 }}月{{ now.getDate() }}日 周{{ ['日','一','二','三','四','五','六'][now.getDay()] }} <Plus :size="14"/></button></div></header>
  <div class="agenda-list"><div v-for="event in events" :key="event.id" class="agenda-row" :class="{current:isCurrent(event),done:itemDone(event,now)}"><button class="agenda-check" :class="{on:itemDone(event,now)}" :aria-label="`${itemDone(event,now)?'恢复':'完成'}${event.title}`" :disabled="locked||saving" @click="toggleEvent(event)"><Check v-if="itemDone(event,now)" :size="12"/></button><div><button class="event-title" :disabled="locked||saving" aria-label="编辑日程" @click="openItem(event)">{{ event.title }}</button><p>{{ event.at.slice(11) }}<template v-if="event.end"> — {{ event.end }}</template></p><p v-if="event.location" class="location"><MapPin :size="17" fill="currentColor"/>{{ event.location }}</p></div><span v-if="isCurrent(event)" class="event-status">进行中</span><button class="row-remove" :disabled="locked||saving" aria-label="删除日程" @click="removeEvent(event.id)"><Trash2 :size="14"/></button></div><p v-if="!events.length" class="empty-state">今天的时间，留给喜欢的事。<button @click="openItem()">安排一个日程</button></p></div>
 </template>
 <template v-else-if="widget.kind==='stocks'">
  <header><span><strong>{{ displayQuote?.name||'股票行情' }}</strong><small class="stock-code">{{ living.symbol.startsWith('116.')?living.symbol.split('.')[1]+'.HK':living.symbol.split('.')[1] }}</small></span><div class="widget-header-actions"><button class="stock-list-open" aria-label="查看自选列表" @click="listOpen=!listOpen;loadWatchlist()">自选 {{ watchlist.length }}</button></div></header>
  <div class="stock-price">{{ displayQuote?displayQuote.price.toFixed(3):'—' }}</div><div class="stock-change">{{ displayQuote?(up?'+':'')+displayQuote.change.toFixed(3):'等待行情' }}<span v-if="displayQuote">{{ up?'+':'' }}{{ displayQuote.percent.toFixed(2) }}%</span></div>
  <svg v-if="chart" class="stock-chart" viewBox="0 0 340 140" preserveAspectRatio="none" role="img" aria-label="价格走势"><defs><linearGradient :id="`chart-${widget.id}`" x2="0" y2="1"><stop stop-color="currentColor" stop-opacity=".32"/><stop offset="1" stop-color="currentColor" stop-opacity="0"/></linearGradient></defs><polygon :points="`0,140 ${chart} 340,140`" :fill="`url(#chart-${widget.id})`"/><polyline :points="chart" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/><circle :cx="chart.split(' ').at(-1)?.split(',')[0]" :cy="chart.split(' ').at(-1)?.split(',')[1]" r="4" fill="currentColor" stroke="#ffe8e2" stroke-width="1.5"/></svg><div v-else class="stock-chart no-chart">{{ quoteBusy?'正在读取行情…':displayQuote?'暂未形成完整走势':'联网后显示真实走势' }}</div>
  <nav class="stock-periods" aria-label="行情周期"><button v-for="p in periods" :key="p.value" :class="{on:period===p.value}" :aria-pressed="period===p.value" :disabled="locked||quoteBusy" @click="period=p.value">{{ p.label }}</button></nav><small v-if="!preview&&quote" class="stock-source">{{ quote.updatedAt }} · {{ quote.source }}</small>
 </template>
 <template v-else-if="widget.kind==='woodfish'">
  <header><h2 class="fish-title"><i/>电子木鱼</h2></header>
  <button class="fish-hit" :class="{striking}" :disabled="locked" aria-label="敲木鱼" @click="tap"><div class="fish-photo-body"/><div class="fish-photo-mallet"/></button><div class="merit-bubble" :class="{show:striking}" :key="pulse">功德 <b>+1</b></div>
  <div class="fish-footer"><div><strong>{{ count }}<small>次</small></strong><p>心平气和 · 万事顺遂</p></div><button class="fish-knock" :disabled="locked" @click="tap">敲一敲</button></div><button class="sound-toggle" :aria-label="sound?'关闭木鱼声音':'打开木鱼声音'" :disabled="saving||locked" @click="toggleSound"><Volume2 v-if="sound" :size="16"/><VolumeX v-else :size="16"/></button>
 </template>
 <template v-else-if="widget.kind==='calendar'">
  <header><button class="month-title" aria-label="回到本月" @click="monthOffset=0;selectedDay=dateKey(now)">{{ month.getFullYear() }}年{{ month.getMonth()+1 }}月</button><div class="month-controls"><button aria-label="上个月" @click="monthOffset--"><ChevronLeft :size="20"/></button><button aria-label="下个月" @click="monthOffset++"><ChevronRight :size="20"/></button></div></header>
  <div class="calendar-week"><span v-for="d in ['日','一','二','三','四','五','六']" :key="d">{{ d }}</span></div><div class="calendar-grid"><button v-for="d in days" :key="dateKey(d.date)" :class="{muted:!d.current,today:dateKey(d.date)===dateKey(now),selected:selectedDay===dateKey(d.date)}" :aria-label="dateKey(d.date)" :aria-pressed="selectedDay===dateKey(d.date)" @click="selectedDay=dateKey(d.date)">{{ d.day }}</button></div>
 </template>
 <template v-else-if="widget.kind==='countdown'">
  <div class="lake-art" aria-hidden="true"/><header><h2 class="anniversary-title"><Heart :size="23" fill="#ef796b" stroke="#ec6c5c"/>{{ countdown.title }}</h2></header><p class="countdown-date">{{ remaining.date.replace('-','年').replace('-','月') }}日 {{ new Date(remaining.date+'T12:00').toLocaleDateString('zh-CN',{weekday:'long'}) }}</p><div class="countdown-value"><strong>{{ remaining.days }}</strong><span>{{ remaining.label==='还有'?'天后':remaining.label==='已经'?'天了':remaining.label==='就是今天'?'就是今天':'天前' }}</span></div><p class="anniversary-quote"><span>“</span> 和你在一起的每一天，<br>　 都是值得期待的日子。 <span>”</span></p>
 </template>
 <template v-else-if="widget.kind==='custom'">
  <header class="custom-widget-header">
   <span class="custom-widget-title">{{ widget.custom?.name||'自创组件' }}</span>
   
  </header>
  <div class="custom-widget-box" :data-testid="`custom-${widget.customId||'widget'}`">
   <iframe
     ref="customFrame"
     @load="customLayout"
     :srcdoc="customDocument"
     sandbox="allow-scripts"
     class="custom-widget-frame"
     scrolling="no"
     tabindex="-1"
   />
  </div>
 </template>
 <button v-if="!demo" type="button" class="widget-settings-button" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
 <button v-if="!preview&&payload" class="resize-corner" aria-label="按比例调整组件大小" title="拖动调整大小 · Esc 取消" data-no-drag @pointerdown.stop.prevent="startResize" @pointermove.stop="resizer.move()" @pointerup.stop="finishResize()" @pointercancel.stop="finishResize(true)" @lostpointercapture="finishResize()"><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M5 15 15 5M10 15l5-5M15 15h.01" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg></button>
 <div v-if="addOpen" class="widget-popover" data-no-drag><ItemForm :key="editingItem?.id||'new'" :mode="widget.kind==='notes'?'todo':'agenda'" :item="editingItem" :busy="saving" @save="commitItem" @cancel="addOpen=false"/></div>
 <section v-if="listOpen&&widget.kind==='stocks'" class="stock-list-panel" data-no-drag><header><strong>我的自选</strong><button aria-label="关闭自选列表" @click="listOpen=false">×</button></header><button v-for="symbol in watchlist" :key="symbol" :class="{selected:symbol===living.symbol}" :disabled="saving" @click="chooseStock(symbol)"><span>{{ quoteFor(symbol)?.name||symbol }}<small>{{ symbol }}</small></span><strong>{{ quoteFor(symbol)?.price.toFixed(3)||'—' }}</strong><small :class="{down:(quoteFor(symbol)?.change??0)<0}">{{ quoteFor(symbol)?((quoteFor(symbol)!.percent>=0?'+':'')+quoteFor(symbol)!.percent.toFixed(2)+'%'):'暂无数据' }}</small></button><p v-if="listBusy">正在更新…</p><p v-if="listError" role="alert">{{ listError }}<button @click="loadWatchlist">重试</button></p><p>点击证券查看走势；在组件设置中管理自选。</p></section>
 <p v-if="error" class="widget-error" role="alert">{{ error }}<button aria-label="关闭错误提示" @click="error=''">×</button></p><div v-if="editing" class="drag-cover">按住卡片拖动</div>
</article>
</template>

