<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Check,ChevronLeft,ChevronRight,Heart,MapPin,MoreHorizontal,Plus,Sunrise,Leaf,ArrowUp,ArrowDown,Volume2,VolumeX,Trash2} from 'lucide-vue-next';
import {desktop,invoke} from '../bridge.ts';
import {baseSize,calendarDays,type DesktopWidget,type WidgetPayload,type WidgetContent,type LivingData,type NoteData,type ReminderItem} from './model.ts';
import {dateKey,lunarDate} from './calendar.ts';
import {emptyNote,emptyCountdown,countdownValue} from './content.ts';
import './surface.css';
const props=withDefaults(defineProps<{widget:DesktopWidget;preview?:boolean;demo?:boolean;editing?:boolean;payload?:WidgetPayload;save?:(c:WidgetContent)=>Promise<void>}>(),{preview:false,demo:false,editing:false});
const emit=defineEmits<{settings:[]}>();
const now=ref(new Date()),error=ref(''),saving=ref(false),addOpen=ref(false),taskText=ref(''),taskTime=ref(''),eventDate=ref(dateKey(new Date())),eventStart=ref('09:00'),eventEnd=ref('10:00'),eventLocation=ref('');
const size=computed(()=>baseSize(props.widget.kind)),clock=computed(()=>now.value.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit',hour12:false}));
const lunarLabel=computed(()=>{const year=new Intl.DateTimeFormat('zh-CN-u-ca-chinese',{year:'numeric'}).formatToParts(now.value).find(p=>(p.type as string)==='yearName')?.value;return `${year?year+'年 ':''}${lunarDate(now.value).full}`;});
const greeting=computed(()=>now.value.getHours()<6?'夜深了':now.value.getHours()<12?'上午好':now.value.getHours()<18?'下午好':'晚上好');
const locked=computed(()=>props.editing),living=computed<LivingData>(()=>props.widget.living??{count:0,symbol:'116.00700',reminders:[]});
const note=computed(()=>props.widget.note??emptyNote()),monthOffset=ref(0),selectedDay=ref(dateKey(now.value));
const month=computed(()=>new Date(now.value.getFullYear(),now.value.getMonth()+monthOffset.value,1));
const days=computed(()=>calendarDays(month.value.getFullYear(),month.value.getMonth()));
const countdown=computed(()=>props.widget.countdown??emptyCountdown()),remaining=computed(()=>countdownValue(countdown.value,now.value));
const events=computed(()=>living.value.reminders.filter(e=>e.at.slice(0,10)===dateKey(now.value)).sort((a,b)=>a.at.localeCompare(b.at)));
const timeNow=computed(()=>now.value.toTimeString().slice(0,5));
const isCurrent=(event:ReminderItem)=>props.demo?event.id==='demo1':!event.done&&event.at.slice(11)<=timeNow.value&&timeNow.value<(event.end||'23:59');
const weather=ref<{text:string;source:string;updatedAt:number}|null>(null),forecast=ref<{date:string;high:number;low:number;code:number}[]>([]),weatherBusy=ref(false);
const temperature=computed(()=>props.demo?'16':weather.value?.text.match(/(-?\d+(?:\.\d+)?)°C/)?.[1]??'—');
const weatherText=computed(()=>props.demo?'多云':weather.value?.text.match(/：([^，]+)，/)?.[1]??'等待天气');
const highLow=computed(()=>props.demo?{high:21,low:8}:forecast.value[0]);
let alive=true,weatherQueued=false,quoteQueued=false;
async function refreshWeather(){
 if(props.preview||!desktop||!props.widget.city||document.hidden)return;
 if(weatherBusy.value){weatherQueued=true;return;}weatherBusy.value=true;error.value='';const key=JSON.stringify(props.widget.city);
 try{const [w,f]=await Promise.all([invoke<typeof weather.value>('get_widget_weather'),invoke<{days:typeof forecast.value}>('get_widget_forecast')]);if(alive&&key===JSON.stringify(props.widget.city)){weather.value=w;forecast.value=f.days;}}
 catch(e){if(alive&&key===JSON.stringify(props.widget.city))error.value=String(e);}finally{weatherBusy.value=false;if(weatherQueued){weatherQueued=false;void refreshWeather();}}
}
watch(()=>JSON.stringify(props.widget.city),()=>{weather.value=null;forecast.value=[];void refreshWeather();});
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
 try{const result=await invoke<Quote>('get_widget_stocks',{period:range});if(alive&&result.symbol===living.value.symbol&&result.period===period.value)quote.value=result;}
 catch(e){if(alive&&requested===living.value.symbol&&range===period.value)error.value=String(e);}finally{quoteBusy.value=false;if(quoteQueued){quoteQueued=false;void loadQuote();}}
}
watch([()=>living.value.symbol,period],()=>{quote.value=null;void loadQuote();});
async function save(content:WidgetContent){
 if(props.editing||saving.value)return false;error.value='';saving.value=true;
 try{if(props.save)await props.save(content);else if(desktop&&!props.preview)await invoke('save_desktop_widget_content',{content});else throw Error('请添加到桌面后使用');return true;}
 catch(e){error.value=String(e);return false;}finally{saving.value=false;}
}
function settings(){if(props.preview){if(!props.demo)emit('settings');return;}void invoke('show_widget_settings').catch(e=>error.value=String(e));}
const identity=()=>crypto.randomUUID().replaceAll('-','').slice(0,16);
async function toggleTask(id:string){await save({note:{...note.value,items:note.value.items.map(i=>i.id===id?{...i,done:!i.done}:i)}});}
async function removeTask(id:string){await save({note:{...note.value,items:note.value.items.filter(i=>i.id!==id)}});}
async function addItem(){
 if(!taskText.value.trim())return;
 let content:WidgetContent;
 if(props.widget.kind==='notes'){if(note.value.items.length>=12){error.value='最多 12 项待办';return;}content={note:{...note.value,mode:'todo',text:'',items:[...note.value.items,{id:identity(),text:taskText.value.trim(),done:false,time:taskTime.value}]}};}
 else{if(living.value.reminders.length>=12){error.value='最多 12 项日程';return;}if(eventEnd.value<=eventStart.value){error.value='结束时间应晚于开始时间';return;}content={living:{...living.value,reminders:[...living.value.reminders,{id:identity(),title:taskText.value.trim(),at:`${eventDate.value}T${eventStart.value}`,done:false,end:eventEnd.value,location:eventLocation.value}]}};}
 if(await save(content)){addOpen.value=false;taskText.value='';}
}
async function removeEvent(id:string){await save({living:{...living.value,reminders:living.value.reminders.filter(i=>i.id!==id)}});}
const count=ref(props.widget.living?.count??0),sound=ref(true),striking=ref(false),pulse=ref(0);let audio:AudioContext|null=null,hitTimer:ReturnType<typeof setTimeout>|undefined;
let hitBusy=false,acknowledged=count.value;
watch(()=>props.widget.living?.count,v=>{acknowledged=v??0;if(!hitBusy)count.value=acknowledged;});
async function persistHits(){
 if(hitBusy)return;hitBusy=true;
 try{while(count.value!==acknowledged){const target=count.value;if(!await save({living:{...living.value,count:target}})){count.value=acknowledged;break;}acknowledged=target;}}
 finally{hitBusy=false;}
}
function tap(){
 if(locked.value)return;
 count.value=Math.min(9999999,count.value+1);striking.value=false;pulse.value++;
 requestAnimationFrame(()=>{striking.value=true;});clearTimeout(hitTimer);hitTimer=setTimeout(()=>striking.value=false,240);
 if(sound.value){try{audio??=new AudioContext();void audio.resume();const o=audio.createOscillator(),g=audio.createGain(),t=audio.currentTime;o.frequency.setValueAtTime(420,t);o.frequency.exponentialRampToValueAtTime(160,t+.16);g.gain.setValueAtTime(.18,t);g.gain.exponentialRampToValueAtTime(.001,t+.2);o.connect(g);g.connect(audio.destination);o.start(t);o.stop(t+.21);}catch{}}
 void persistHits();
}
let drag:{x:number;y:number;grabX:number;grabY:number;moved:boolean}|null=null,suppressClick=false,moving=false,pending:{grabX:number;grabY:number;commit:boolean}|null=null;
let clickTimer:ReturnType<typeof setTimeout>|undefined;
async function movePending(){if(moving)return;moving=true;try{while(pending){const p=pending;pending=null;await invoke('move_desktop_widget',p);}}catch(e){error.value=String(e);}finally{moving=false;}}
function startDrag(e:PointerEvent){if(e.button!==0||props.preview||!props.payload)return;if((e.target as Element).closest('button,input,textarea,select,form,[data-no-drag]'))return;suppressClick=false;drag={x:e.screenX,y:e.screenY,grabX:e.clientX/Math.max(1,innerWidth),grabY:e.clientY/Math.max(1,innerHeight),moved:false};(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);}
function move(e:PointerEvent,commit=false){if(!drag)return;if(Math.hypot(e.screenX-drag.x,e.screenY-drag.y)>5)drag.moved=true;if(drag.moved){suppressClick=true;pending={grabX:drag.grabX,grabY:drag.grabY,commit};void movePending();}if(commit){drag=null;clickTimer=setTimeout(()=>suppressClick=false,0);}}
function capture(e:MouseEvent){if(suppressClick||props.editing){e.stopImmediatePropagation();e.preventDefault();}}
let clockTimer:ReturnType<typeof setInterval>,networkTimer:ReturnType<typeof setInterval>;
function visible(){now.value=new Date();if(!document.hidden){if(props.widget.kind==='weather')void refreshWeather();else if(props.widget.kind==='stocks')void loadQuote();}}
onMounted(()=>{clockTimer=setInterval(()=>now.value=new Date(),1000);if(props.widget.kind==='weather')void refreshWeather();if(props.widget.kind==='stocks')void loadQuote();networkTimer=setInterval(visible,props.widget.kind==='stocks'?60000:1800000);document.addEventListener('visibilitychange',visible);});
onBeforeUnmount(()=>{alive=false;clearInterval(clockTimer);clearInterval(networkTimer);clearTimeout(hitTimer);clearTimeout(clickTimer);document.removeEventListener('visibilitychange',visible);void audio?.close();});
</script>
<template>
<article class="xb-widget" :class="[widget.kind,{editing,falling:!up}]" :style="{width:`${size.width}px`,height:`${size.height}px`}" @pointerdown="startDrag" @pointermove="move($event)" @pointerup="move($event,true)" @pointercancel="move($event,true)" @click.capture="capture">
 <template v-if="widget.kind==='clock'">
  <div class="clock-art" aria-hidden="true"/><div class="clock-greeting"><Sunrise :size="31"/><div><span>{{ greeting }}</span><small>新的一天，也要好好生活</small></div></div>
  <div class="clock-time">{{ clock }}</div><div class="clock-date">{{ now.getMonth()+1 }}月{{ now.getDate() }}日 <span>{{ now.toLocaleDateString('zh-CN',{weekday:'long'}) }}</span></div><p class="clock-lunar">{{ lunarLabel }}</p><button class="more hover-more" aria-label="时钟设置" @click="settings"><MoreHorizontal/></button>
 </template>
 <template v-else-if="widget.kind==='weather'">
  <header><span class="city-label"><MapPin :size="21" fill="currentColor"/>{{ demo?'杭州市':widget.city?.name||'选择城市' }}</span><button class="more" aria-label="天气设置" @click="settings"><MoreHorizontal/></button></header>
  <div class="weather-art" :class="{'is-rain':weatherText.includes('雨'),'is-sun':weatherText==='晴'}" aria-hidden="true"><i class="weather-sun"/><i class="weather-cloud"/></div>
  <div class="weather-temperature">{{ temperature }}<sup>°C</sup></div><div class="weather-description">{{ weatherText }} <span v-if="demo">空气优 <Leaf :size="19" fill="#9bc378"/></span></div>
  <div v-if="highLow" class="weather-range"><ArrowUp :size="18"/>{{ highLow.high }}° <ArrowDown :size="18"/>{{ highLow.low }}°</div>
  <button v-if="!preview&&!widget.city" class="inline-link weather-configure" @click="settings">设置天气城市</button><small v-if="!preview&&weather" class="data-source">{{ weather.source }} · {{ new Date(weather.updatedAt*1000).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'}) }}</small><small v-if="weatherBusy" class="data-source">正在更新…</small>
 </template>
 <template v-else-if="widget.kind==='notes'">
  <header><h2>待办事项</h2><button class="add-button" :disabled="locked||saving" aria-label="添加待办" @click="addOpen=!addOpen"><Plus/></button></header>
  <div class="todo-list"><div v-for="item in note.items" :key="item.id" class="todo-row" :class="{done:item.done}"><button class="check-circle" :aria-pressed="item.done" :aria-label="`${item.done?'恢复':'完成'}${item.text}`" :disabled="locked||saving" @click="toggleTask(item.id)"><Check v-if="item.done" :size="19"/></button><span>{{ item.text }}</span><small v-if="item.time">今天 {{ item.time }}</small><button class="row-remove" :disabled="locked||saving" aria-label="删除待办" @click="removeTask(item.id)"><Trash2 :size="13"/></button></div><p v-if="!note.items.length" class="empty-state">把今天想做的事，记在这里。<button @click="addOpen=true">添加第一件小事</button></p></div>
 </template>
 <template v-else-if="widget.kind==='reminders'">
  <header><h2>今日日程</h2><button class="agenda-date" aria-label="添加日程" :disabled="locked" @click="addOpen=!addOpen">{{ now.getMonth()+1 }}月{{ now.getDate() }}日 周{{ ['日','一','二','三','四','五','六'][now.getDay()] }} <Plus :size="14"/></button></header>
  <div class="agenda-list"><div v-for="event in events" :key="event.id" class="agenda-row" :class="{current:isCurrent(event),done:event.done}"><i/><div><strong>{{ event.title }}</strong><p>{{ event.at.slice(11) }}<template v-if="event.end"> — {{ event.end }}</template></p><p v-if="event.location" class="location"><MapPin :size="17" fill="currentColor"/>{{ event.location }}</p></div><span v-if="isCurrent(event)" class="event-status">进行中</span><button class="row-remove" :disabled="locked||saving" aria-label="删除日程" @click="removeEvent(event.id)"><Trash2 :size="14"/></button></div><p v-if="!events.length" class="empty-state">今天的时间，留给喜欢的事。<button @click="addOpen=true">安排一个日程</button></p></div>
 </template>
 <template v-else-if="widget.kind==='stocks'">
  <header><span><strong>{{ displayQuote?.name||'股票行情' }}</strong><small class="stock-code">{{ living.symbol.startsWith('116.')?living.symbol.split('.')[1]+'.HK':living.symbol.split('.')[1] }}</small></span><button class="more" aria-label="股票设置" @click="settings"><MoreHorizontal/></button></header>
  <div class="stock-price">{{ displayQuote?displayQuote.price.toFixed(3):'—' }}</div><div class="stock-change">{{ displayQuote?(up?'+':'')+displayQuote.change.toFixed(3):'等待行情' }}<span v-if="displayQuote">{{ up?'+':'' }}{{ displayQuote.percent.toFixed(2) }}%</span></div>
  <svg v-if="chart" class="stock-chart" viewBox="0 0 340 140" preserveAspectRatio="none" role="img" aria-label="价格走势"><defs><linearGradient :id="`chart-${widget.id}`" x2="0" y2="1"><stop stop-color="currentColor" stop-opacity=".32"/><stop offset="1" stop-color="currentColor" stop-opacity="0"/></linearGradient></defs><polygon :points="`0,140 ${chart} 340,140`" :fill="`url(#chart-${widget.id})`"/><polyline :points="chart" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/><circle :cx="chart.split(' ').at(-1)?.split(',')[0]" :cy="chart.split(' ').at(-1)?.split(',')[1]" r="4" fill="currentColor" stroke="#ffe8e2" stroke-width="1.5"/></svg><div v-else class="stock-chart no-chart">{{ quoteBusy?'正在读取行情…':'联网后显示真实走势' }}</div>
  <nav class="stock-periods" aria-label="行情周期"><button v-for="p in periods" :key="p.value" :class="{on:period===p.value}" :aria-pressed="period===p.value" :disabled="locked||quoteBusy" @click="period=p.value">{{ p.label }}</button></nav><small v-if="!preview&&quote" class="stock-source">{{ quote.updatedAt }} · {{ quote.source }}</small>
 </template>
 <template v-else-if="widget.kind==='woodfish'">
  <header><h2 class="fish-title"><i/>电子木鱼</h2><button class="more" aria-label="木鱼设置" @click="settings"><MoreHorizontal/></button></header>
  <button class="fish-hit" :class="{striking}" :disabled="locked" aria-label="敲木鱼" @click="tap"><div class="fish-photo-body"/><div class="fish-photo-mallet"/></button><div class="merit-bubble" :class="{show:striking}" :key="pulse">功德 <b>+1</b></div>
  <div class="fish-footer"><div><strong>{{ count }}<small>次</small></strong><p>心平气和 · 万事顺遂</p></div><button class="fish-knock" :disabled="locked" @click="tap">敲一敲</button></div><button class="sound-toggle" :aria-label="sound?'关闭木鱼声音':'打开木鱼声音'" @click="sound=!sound"><Volume2 v-if="sound" :size="16"/><VolumeX v-else :size="16"/></button>
 </template>
 <template v-else-if="widget.kind==='calendar'">
  <header><button class="month-title" aria-label="回到本月" @click="monthOffset=0;selectedDay=dateKey(now)">{{ month.getFullYear() }}年{{ month.getMonth()+1 }}月</button><div class="month-controls"><button aria-label="上个月" @click="monthOffset--"><ChevronLeft :size="20"/></button><button aria-label="下个月" @click="monthOffset++"><ChevronRight :size="20"/></button></div></header>
  <div class="calendar-week"><span v-for="d in ['日','一','二','三','四','五','六']" :key="d">{{ d }}</span></div><div class="calendar-grid"><button v-for="d in days" :key="dateKey(d.date)" :class="{muted:!d.current,today:dateKey(d.date)===dateKey(now),selected:selectedDay===dateKey(d.date)}" :aria-label="dateKey(d.date)" :aria-pressed="selectedDay===dateKey(d.date)" @click="selectedDay=dateKey(d.date)">{{ d.day }}</button></div>
 </template>
 <template v-else-if="widget.kind==='countdown'">
  <div class="lake-art" aria-hidden="true"/><header><h2 class="anniversary-title"><Heart :size="23" fill="#ef796b" stroke="#ec6c5c"/>{{ countdown.title }}</h2><button class="more hover-more" aria-label="纪念日设置" @click="settings"><MoreHorizontal/></button></header><p class="countdown-date">{{ remaining.date.replace('-','年').replace('-','月') }}日 {{ new Date(remaining.date+'T12:00').toLocaleDateString('zh-CN',{weekday:'long'}) }}</p><div class="countdown-value"><strong>{{ remaining.days }}</strong><span>{{ remaining.label==='还有'?'天后':remaining.label==='已经'?'天了':remaining.label==='就是今天'?'就是今天':'天前' }}</span></div><p class="anniversary-quote"><span>“</span> 和你在一起的每一天，<br>　 都是值得期待的日子。 <span>”</span></p>
 </template>
 <form v-if="addOpen" class="widget-popover" data-no-drag @submit.prevent="addItem"><header><strong>{{ widget.kind==='notes'?'添加待办':'添加日程' }}</strong><button type="button" class="close-form" aria-label="关闭编辑" @click="addOpen=false">×</button></header><input v-model="taskText" :maxlength="widget.kind==='notes'?120:60" placeholder="写下要做的事" aria-label="事项名称" required/><input v-if="widget.kind==='notes'" v-model="taskTime" type="time" aria-label="待办时间"/><template v-else><input v-model="eventDate" type="date" aria-label="日程日期" required/><div class="time-inputs"><input v-model="eventStart" type="time" aria-label="开始时间" required/><span>—</span><input v-model="eventEnd" type="time" aria-label="结束时间" required/></div><input v-model="eventLocation" maxlength="80" placeholder="地点（可选）" aria-label="日程地点"/></template><button class="form-save" :disabled="saving||locked">{{ saving?'保存中…':'保存' }}</button></form>
 <p v-if="error" class="widget-error" role="alert">{{ error }}<button aria-label="关闭错误提示" @click="error=''">×</button></p><div v-if="editing" class="drag-cover">按住卡片拖动</div>
</article>
</template>
