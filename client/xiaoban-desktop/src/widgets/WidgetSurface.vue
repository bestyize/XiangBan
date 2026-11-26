<script setup lang="ts">
import {computed,onActivated,onDeactivated,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Check,ChevronLeft,ChevronRight,Heart,Flag,Sparkles,CalendarDays,MapPin,Settings2,Plus,Sunrise,Sun,Sunset,Moon,ArrowUp,ArrowDown,Volume2,VolumeX,Trash2} from 'lucide-vue-next';
import {desktop,invoke} from '../bridge.ts';
import {baseSize,calendarDays,type DesktopWidget,type WidgetPayload,type WidgetContent,type LivingData,type ReminderItem,type TodoItem} from './model.ts';
import {dateKey,lunarDate,calendarLabel,holidayInfo} from './calendar.ts';
import {emptyNote,emptyCountdown,countdownValue,occursOn,itemDone,toggleItem,scheduleLabel} from './content.ts';
import ItemForm from './ItemForm.vue';
import {cornerResize} from './resize.ts';
import {customWidgetDocument,customWidgetHtmlUrl,customWidgetShell,isCustomDragMessage} from './custom-drag.ts';
import './surface.css';

const props=withDefaults(defineProps<{widget:DesktopWidget;preview?:boolean;demo?:boolean;editing?:boolean;payload?:WidgetPayload;renderScale?:number;save?:(c:WidgetContent)=>Promise<void>}>(),{preview:false,demo:false,editing:false,renderScale:1});
const emit=defineEmits<{settings:[]}>();

const now=ref(new Date()),error=ref(''),saving=ref(false),addOpen=ref(false),editingItem=ref<TodoItem|ReminderItem>();
const size=computed(()=>baseSize(props.widget.kind,props.widget.custom));
const material=computed(()=>props.widget.kind==='custom'?'ivory':(['ivory','dark','glass'].includes(props.widget.theme)?props.widget.theme:'ivory'));

// Clock: dynamic greeting, subtitle, scene, optional seconds
const showSeconds=ref(false);
const clock=computed(()=>now.value.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit',hour12:false}));
const secondsText=computed(()=>`:${String(now.value.getSeconds()).padStart(2,'0')}`);
const hour=computed(()=>now.value.getHours());
const isNightClock=computed(()=>hour.value<6||hour.value>=21);
const greeting=computed(()=>hour.value<6?'夜深了':hour.value<12?'上午好':hour.value<18?'下午好':hour.value<21?'晚上好':'夜深了');
const clockSubtitle=computed(()=>hour.value<6?'早点休息，明天见':hour.value<12?'微风正好，按自己的节奏来':hour.value<18?'午后时光，保持专注':hour.value<21?'辛苦一天，放松一下':'早点休息，明天见');
const clockIcon=computed(()=>hour.value<6||hour.value>=21?Moon:hour.value<12?Sunrise:hour.value<18?Sun:Sunset);

const customFrame=ref<HTMLIFrameElement>();
const customShell=customWidgetShell(desktop);
const surfaceScale=computed(()=>props.widget.kind==='custom'?props.renderScale:1);
function customLayout(){customFrame.value?.contentWindow?.postMessage({type:'xiaoban-widget-layout',scale:props.renderScale,width:size.value.width,height:size.value.height},'*');}
watch(()=>props.renderScale,customLayout,{flush:'post'});
const customDocument=computed(()=>{
  const html=props.widget.custom?.html||'<div style="color:#999;padding:12px;font-size:12px">未找到小组件内容</div>';
  return customWidgetDocument(html,!props.preview);
});
const standaloneId = computed(() => desktop && props.widget.custom?.standaloneHtml ? props.widget.custom.id : undefined);
const customFrameKey = computed(() => standaloneId.value ? `local:${standaloneId.value}` : customDocument.value);
let initializedFrame: HTMLIFrameElement | undefined;
let htmlLoad: AbortController | undefined;
watch(customFrame, () => {
  if (initializedFrame !== customFrame.value) htmlLoad?.abort();
});

async function initializeCustomFrame(frame: HTMLIFrameElement) {
  if (initializedFrame === frame || !frame.contentWindow) return;
  htmlLoad?.abort();
  initializedFrame = frame;
  const id = standaloneId.value;
  if (!id) {
    frame.contentWindow.postMessage({ type: 'xiaoban-widget-init', html: customDocument.value }, '*');
    return;
  }
  const load = new AbortController();
  htmlLoad = load;
  try {
    const response = await fetch(customWidgetHtmlUrl(id), { signal: load.signal });
    if (!response.ok) throw new Error('找不到组件的 HTML 文件');
    const html = await response.text();
    if (!load.signal.aborted && customFrame.value === frame) {
      frame.contentWindow?.postMessage({ type: 'xiaoban-widget-init', html: customWidgetDocument(html, !props.preview) }, '*');
    }
  } catch (cause) {
    if (!load.signal.aborted) error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    if (htmlLoad === load) htmlLoad = undefined;
  }
}

const lunarLabel=computed(()=>{const year=new Intl.DateTimeFormat('zh-CN-u-ca-chinese',{year:'numeric'}).formatToParts(now.value).find(p=>(p.type as string)==='yearName')?.value;return `${year?year+'年 ':''}${lunarDate(now.value).full}`;});
const locked=computed(()=>props.editing),living=computed<LivingData>(()=>props.widget.living??{count:0,symbol:'116.00700',reminders:[]});
const note=computed(()=>props.widget.note??emptyNote()),quickTodo=ref('');
const todoDoneCount=computed(()=>note.value.items.filter(i=>itemDone(i,now.value)).length);

// Calendar: lunar labels, 2026 holidays, selected day detail, return to today
const monthOffset=ref(0),selectedDay=ref(dateKey(now.value));
const month=computed(()=>new Date(now.value.getFullYear(),now.value.getMonth()+monthOffset.value,1));
const days=computed(()=>calendarDays(month.value.getFullYear(),month.value.getMonth()).map(item=>{
  const key=dateKey(item.date);
  const holiday=holidayInfo(item.date);
  const lunar=calendarLabel(item.date);
  const festival=/节|春节|元宵|端午|七夕|中秋|重阳|腊八|龙抬头/.test(lunar);
  return {...item,key,lunar,holiday,festival};
}));
const selectedDayInfo=computed(()=>{
  const [y,m,d]=selectedDay.value.split('-').map(Number);
  const dt=new Date(y||now.value.getFullYear(),(m||1)-1,d||1,12);
  const lunar=lunarDate(dt);
  const holiday=holidayInfo(dt);
  const extra=holiday.holiday?`${holiday.holiday}放假`:holiday.makeup?`${holiday.makeup}调休补班`:lunar.festival||'';
  return {solar:`${dt.getMonth()+1}月${dt.getDate()}日`,lunar:lunar.full,extra,isToday:selectedDay.value===dateKey(now.value)};
});

// Countdown: custom icon, quote, "就是今天" hero
const countdown=computed(()=>props.widget.countdown??emptyCountdown()),remaining=computed(()=>countdownValue(countdown.value,now.value));
const countdownIconComponent=computed(()=>({heart:Heart,flag:Flag,sparkle:Sparkles,calendar:CalendarDays}[countdown.value.icon||'heart']??Heart));
const countdownQuote=computed(()=>countdown.value.quote?.trim()||(props.demo?'每一次倒数，都是奔赴期待的开始。':''));

// Agenda: Today / Tomorrow switch + flex status badge
const agendaDay=ref<'today'|'tomorrow'>('today');
const agendaTargetDate=computed(()=>agendaDay.value==='today'?now.value:new Date(now.value.getFullYear(),now.value.getMonth(),now.value.getDate()+1,12));
const events=computed(()=>living.value.reminders.filter(e=>occursOn(e.at,e.repeat,agendaTargetDate.value)).sort((a,b)=>a.at.slice(11).localeCompare(b.at.slice(11))));
const timeNow=computed(()=>now.value.toTimeString().slice(0,5));
const isCurrent=(event:ReminderItem)=>agendaDay.value==='today'&&(props.demo?event.id==='demo1':!itemDone(event,now.value)&&event.at.slice(11)<=timeNow.value&&timeNow.value<(event.end||'23:59'));

// Weather: 6-state vector icons, AQI pill, 3-day mini forecast
const weather=ref<{text:string;source:string;updatedAt:number}|null>(null),forecast=ref<{date:string;high:number;low:number;code:number}[]>([]),weatherBusy=ref(false);
const temperature=computed(()=>props.demo?'16':weather.value?.text.match(/(-?\d+(?:\.\d+)?)°C/)?.[1]??'—');
const weatherText=computed(()=>props.demo?'多云':weather.value?.text.match(/：([^，]+)，/)?.[1]??'等待天气');
const weatherKind=computed<'sun'|'night'|'cloudy'|'overcast'|'rain'|'thunder'|'snow'>(()=>{
  const t=weatherText.value;
  if(t.includes('雷'))return 'thunder';
  if(t.includes('雪')||t.includes('冰雹'))return 'snow';
  if(t.includes('雨'))return 'rain';
  if(t.includes('阴')||t.includes('雾')||t.includes('霾'))return 'overcast';
  if(t==='晴'||t.includes('晴'))return (hour.value<6||hour.value>=19)&&!props.demo?'night':'sun';
  return 'cloudy';
});
const highLow=computed(()=>props.demo?{high:21,low:8}:forecast.value[0]);
const miniForecast=computed(()=>{
  if(props.demo)return [{label:'明天',high:19,low:9},{label:'周六',high:15,low:7},{label:'周日',high:20,low:10}];
  const week=['周日','周一','周二','周三','周四','周五','周六'];
  return forecast.value.slice(1,4).map((d,idx)=>{
    const dt=new Date(d.date+'T12:00');
    return {label:idx===0?'明天':Number.isNaN(dt.getTime())?d.date.slice(5):week[dt.getDay()],high:Math.round(d.high),low:Math.round(d.low)};
  });
});
type Air={aqi:number;pm25:number|null;pm10:number|null;modelAt:number;updatedAt:number;standard:string;source:string};
const air=ref<Air|null>(null),airError=ref('');
const airLevel=computed<'good'|'mid'|'bad'|'muted'>(()=>{
  if(props.demo)return 'good';
  if(!air.value)return 'muted';
  return air.value.aqi<=50?'good':air.value.aqi<=100?'mid':'bad';
});
const airLabel=computed(()=>{if(props.demo)return '空气优';if(!air.value)return airError.value?'空气数据暂不可用':'空气待更新';const a=air.value.aqi;return a<=50?'空气优':a<=100?'空气良':a<=150?'敏感注意':a<=200?'不健康':a<=300?'很不健康':'空气危险';});
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

// Stocks: formatted 2-decimal price, prev-close baseline, interactive hover crosshair
type Quote={symbol:string;name:string;price:number;change:number;percent:number;points:number[];updatedAt:string;period:string;source:string};
const quote=ref<Quote|null>(null),quoteBusy=ref(false),period=ref('1D');
const periods=[{value:'1D',label:'1天'},{value:'5D',label:'1周'},{value:'1M',label:'1月'},{value:'1Y',label:'1年'}];
const demoPrices=[302,303,306,305,307,305,304,303,304,303,305,308,306,305,307,306,308,310,309,310,312,313,312,314,313,315,317,316,314,315,314,316,315,317,316,318,317,320,319,309.6];
const displayQuote=computed(()=>props.demo?{name:'腾讯控股',symbol:'116.00700',price:309.6,change:4.8,percent:1.58,points:demoPrices,updatedAt:'',period:period.value,source:'展示数据'}:quote.value);
const up=computed(()=>(displayQuote.value?.change??0)>=0);
const formatPrice=(v:number)=>v.toLocaleString('en-US',{minimumFractionDigits:2,maximumFractionDigits:2});
const prevClose=computed(()=>displayQuote.value?displayQuote.value.price-displayQuote.value.change:null);
const chartBounds=computed(()=>{
  const s=displayQuote.value?.points??[];
  if(s.length<2)return null;
  const all=prevClose.value!=null?[...s,prevClose.value]:s;
  const min=Math.min(...all),max=Math.max(...all),range=Math.max(.001,max-min);
  const coords=s.map((v,i)=>({x:+(i/(s.length-1)*340).toFixed(2),y:+(116-(v-min)/range*100).toFixed(2),v}));
  const prevY=prevClose.value!=null?+(116-(prevClose.value-min)/range*100).toFixed(2):null;
  return {coords,points:coords.map(c=>`${c.x},${c.y}`).join(' '),prevY};
});
const chart=computed(()=>chartBounds.value?.points??'');
const hoverIdx=ref<number|null>(null);
const hoverPoint=computed(()=>hoverIdx.value!=null&&chartBounds.value?chartBounds.value.coords[hoverIdx.value]??null:null);
function onChartMove(e:PointerEvent){
  if(!chartBounds.value)return;
  const rect=(e.currentTarget as HTMLElement).getBoundingClientRect();
  if(rect.width<=0)return;
  const ratio=Math.max(0,Math.min(1,(e.clientX-rect.left)/rect.width));
  hoverIdx.value=Math.round(ratio*(chartBounds.value.coords.length-1));
}

async function loadQuote(){
 if(props.preview||!desktop||props.widget.kind!=='stocks'||document.hidden)return;
 if(quoteBusy.value){quoteQueued=true;return;}quoteBusy.value=true;error.value='';const requested=living.value.symbol,range=period.value;
 try{const result=await invoke<Quote>('get_widget_stocks',{period:range,symbol:requested});if(alive&&result.symbol===living.value.symbol&&result.period===period.value)quote.value=result;}
 catch(e){if(alive&&requested===living.value.symbol&&range===period.value)error.value=String(e);}finally{quoteBusy.value=false;if(quoteQueued){quoteQueued=false;void loadQuote();}}
}
watch([()=>living.value.symbol,period],()=>{quote.value=null;hoverIdx.value=null;void loadQuote();});

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
async function addQuickTask(){
  const text=quickTodo.value.trim();
  if(!text||locked.value||saving.value)return;
  if(note.value.items.length>=12){error.value='最多保存 12 项，请先删除不需要的事项';return;}
  const item:TodoItem={id:crypto.randomUUID().replaceAll('-','').slice(0,16),text,done:false,date:dateKey(now.value),time:'',notify:false,repeat:'none',completedOn:''};
  if(await save({note:{...note.value,mode:'todo',items:[...note.value.items,item]}}))quickTodo.value='';
}
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

// Woodfish: multi-bubble rapid taps + customizable merit label + local today count
const meritWords=['功德 +1','烦恼 -1','Bug -1','灵感 +1'];
const meritWordIdx=ref(0);
const meritBubbles=ref<Array<{id:number;text:string;x:number;y:number}>>([]);
const count=ref(props.widget.living?.count??0),sound=ref(props.widget.living?.sound??true),striking=ref(false);
const todayHits=ref(props.demo?18:0);
let audio:AudioContext|null=null,hitTimer:ReturnType<typeof setTimeout>|undefined,bubbleSeq=0;
function loadTodayHits(){
  if(props.demo)return;
  try{
    const saved=JSON.parse(localStorage.getItem(`xiaoban.wf.${props.widget.id}`)||'null');
    if(saved&&saved.day===dateKey(now.value)&&typeof saved.n==='number')todayHits.value=saved.n;
  }catch{}
}
function saveTodayHits(){
  if(props.demo)return;
  try{localStorage.setItem(`xiaoban.wf.${props.widget.id}`,JSON.stringify({day:dateKey(now.value),n:todayHits.value}));}catch{}
}
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
 count.value=Math.min(9999999,count.value+1);
 todayHits.value++;saveTodayHits();
 striking.value=false;
 const id=++bubbleSeq;
 meritBubbles.value=[...meritBubbles.value.slice(-5),{id,text:meritWords[meritWordIdx.value],x:16+(id*17)%42,y:16+(id*11)%18}];
 setTimeout(()=>{meritBubbles.value=meritBubbles.value.filter(b=>b.id!==id);},780);
 requestAnimationFrame(()=>{striking.value=true;});clearTimeout(hitTimer);hitTimer=setTimeout(()=>striking.value=false,220);
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
 if(!customFrame.value?.contentWindow||event.source!==customFrame.value.contentWindow)return;
 if(event.data?.type==='xiaoban-widget-shell-ready'){
  void initializeCustomFrame(customFrame.value);
  return;
 }
 if(props.preview||!props.payload)return;
 if(event.data?.type==='xiaoban-widget-settings'){settings();return;}
 if(props.editing||!isCustomDragMessage(event.data))return;
 const data=event.data;
 if(data.phase==='start'){
  const rect=customFrame.value.getBoundingClientRect();
  suppressClick=false;drag={x:data.screenX,y:data.screenY,grabX:(rect.left+data.x*rect.width)/Math.max(1,innerWidth),grabY:(rect.top+data.y*rect.height)/Math.max(1,innerHeight),moved:false};
 }else move(data,data.phase==='end');
}
onMounted(()=>window.addEventListener('message',customDrag));
onBeforeUnmount(()=>{htmlLoad?.abort();window.removeEventListener('message',customDrag);});
function capture(e:MouseEvent){if((e.target as Element).closest('.widget-settings-button'))return;if(suppressClick||props.editing){e.stopImmediatePropagation();e.preventDefault();}}

let clockTimer:ReturnType<typeof setTimeout>|undefined,networkTimer:ReturnType<typeof setInterval>|undefined,active=true;
function scheduleClock(){
  if(!active||clockTimer)return;
  const delay=showSeconds.value?1000-Date.now()%1000+15:60000-Date.now()%60000+20;
  clockTimer=setTimeout(()=>{clockTimer=undefined;now.value=new Date();scheduleClock();},delay);
}
watch(showSeconds,()=>{clearTimeout(clockTimer);clockTimer=undefined;now.value=new Date();scheduleClock();});
function startTimers(){scheduleClock();if(!props.preview&&(props.widget.kind==='weather'||props.widget.kind==='stocks')&&!networkTimer)networkTimer=setInterval(visible,props.widget.kind==='stocks'?60000:1800000);}
function stopTimers(){clearTimeout(clockTimer);clearInterval(networkTimer);clockTimer=undefined;networkTimer=undefined;}
function visible(){if(!active)return;now.value=new Date();if(!document.hidden){if(props.widget.kind==='weather')void refreshWeather();else if(props.widget.kind==='stocks'){void loadQuote();void loadWatchlist();}}}
onMounted(()=>{loadTodayHits();startTimers();if(props.widget.kind==='weather')void refreshWeather();if(props.widget.kind==='stocks'){void loadQuote();void loadWatchlist();}document.addEventListener('visibilitychange',visible);document.addEventListener('keydown',resizeKey);});
onActivated(()=>{if(!active){active=true;startTimers();visible();}});
onDeactivated(()=>{active=false;stopTimers();});
onBeforeUnmount(()=>{resizer.finish(true);document.removeEventListener('keydown',resizeKey);alive=false;stopTimers();clearTimeout(hitTimer);clearTimeout(clickTimer);document.removeEventListener('visibilitychange',visible);void audio?.close();});
</script>
<template>
<article class="xb-widget" :class="[widget.kind,{editing,resizing,falling:!up}]" :data-material="material" :style="{width:`${size.width*surfaceScale}px`,height:`${size.height*surfaceScale}px`,'--widget-scale':widget.kind==='custom'?1:widget.scale}" @pointerdown="startDrag" @pointermove="move($event)" @pointerup="move($event,true)" @pointercancel="move($event,true)" @click.capture="capture" @contextmenu.prevent.stop="settings">
 <!-- 1. 桌面时钟 -->
 <template v-if="widget.kind==='clock'">
  <svg class="clock-scene-svg" viewBox="0 0 340 260" preserveAspectRatio="xMaxYMax slice" aria-hidden="true">
   <defs>
    <linearGradient :id="`cm-${widget.id}`" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="white" stop-opacity="0"/><stop offset="40%" stop-color="white" stop-opacity="1"/></linearGradient>
    <mask :id="`cmask-${widget.id}`"><rect width="340" height="260" :fill="`url(#cm-${widget.id})`"/></mask>
   </defs>
   <g v-if="isNightClock" :mask="`url(#cmask-${widget.id})`">
    <circle cx="246" cy="86" r="28" fill="#B9C7E8" opacity=".46"/>
    <circle cx="188" cy="60" r="2.2" fill="#E6D9B8" opacity=".82"/><circle cx="294" cy="54" r="1.8" fill="#E6D9B8" opacity=".72"/><circle cx="222" cy="44" r="1.5" fill="#E6D9B8" opacity=".62"/>
    <path d="M40 260 Q145 148 230 188 T340 156 L340 260 Z" fill="#6F7896" opacity=".32"/>
    <path d="M120 260 Q216 184 286 212 T340 194 L340 260 Z" fill="#4D546D" opacity=".38"/>
   </g>
   <g v-else :mask="`url(#cmask-${widget.id})`">
    <circle cx="248" cy="126" r="64" fill="#F6C892" opacity=".28"/><circle cx="248" cy="126" r="32" fill="#F1A96A" opacity=".72"/>
    <path d="M46 260 Q152 144 236 188 T340 162 L340 260 Z" fill="#BFA992" opacity=".44"/>
    <path d="M132 260 Q228 186 294 214 T340 198 L340 260 Z" fill="#A58E76" opacity=".40"/>
   </g>
  </svg>
  <header>
   <div class="clock-greeting"><span class="clock-ic"><component :is="clockIcon" :size="20"/></span><div><span>{{ greeting }}</span><small>{{ clockSubtitle }}</small></div></div>
   <div class="widget-header-actions">
    <button v-if="!demo" type="button" class="hdr-pill" :class="{on:showSeconds}" :aria-pressed="showSeconds" title="切换显示秒数" data-no-drag @click.stop="showSeconds=!showSeconds">秒</button>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <div class="clock-digits"><div class="clock-time">{{ clock }}</div><span v-if="showSeconds" class="clock-sec">{{ secondsText }}</span></div>
  <div class="clock-meta"><span class="clock-date">{{ now.getMonth()+1 }}月{{ now.getDate() }}日 <span>{{ now.toLocaleDateString('zh-CN',{weekday:'long'}) }}</span></span><span class="clock-lunar">{{ lunarLabel }}</span></div>
 </template>

 <!-- 2. 城市天气 -->
 <template v-else-if="widget.kind==='weather'">
  <header>
   <span class="city-label"><MapPin :size="17" fill="currentColor"/>{{ demo?'杭州市':widget.city?.name||'选择城市' }}</span>
   <div class="widget-header-actions">
    <span class="air-quality" :class="`is-${airLevel}`" :title="airTitle"><i aria-hidden="true"/>{{ airLabel }}<small v-if="!demo&&air">US AQI {{ Math.round(air.aqi) }}</small></span>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <div class="weather-main">
   <div class="weather-svg" aria-hidden="true">
    <svg v-if="weatherKind==='sun'" viewBox="0 0 64 56"><circle cx="32" cy="28" r="14" fill="#F8B73B"/><g stroke="#F8B73B" stroke-width="3" stroke-linecap="round"><path d="M32 6v5M32 45v5M10 28h5M49 28h5M16.5 12.5l3.5 3.5M44 40l3.5 3.5M47.5 12.5L44 16M20 40l-3.5 3.5"/></g></svg>
    <svg v-else-if="weatherKind==='night'" viewBox="0 0 64 56"><path d="M38 11a16 16 0 1 0 14 23 13 13 0 1 1-14-23Z" fill="#7D96D6"/><circle cx="46" cy="15" r="2.2" fill="#F2C86B"/><circle cx="53" cy="24" r="1.5" fill="#F2C86B"/></svg>
    <svg v-else-if="weatherKind==='rain'" viewBox="0 0 64 56"><path d="M16 35h32a9.5 9.5 0 0 0 1-19 13.5 13.5 0 0 0-26 2.8A8.5 8.5 0 0 0 16 35Z" fill="#D9E2EA" stroke="rgba(90,110,130,.22)" stroke-width="1.2"/><g stroke="#5D9AD4" stroke-width="2.6" stroke-linecap="round"><path d="M23 40l-2.5 7M33 40l-2.5 7M43 40l-2.5 7"/></g></svg>
    <svg v-else-if="weatherKind==='thunder'" viewBox="0 0 64 56"><path d="M16 34h32a9.5 9.5 0 0 0 1-19 13.5 13.5 0 0 0-26 2.8A8.5 8.5 0 0 0 16 34Z" fill="#9AA7B6"/><path d="M33 34l-6 10h6l-3 9 11-13h-6l4-6Z" fill="#F5B433"/></svg>
    <svg v-else-if="weatherKind==='snow'" viewBox="0 0 64 56"><path d="M16 35h32a9.5 9.5 0 0 0 1-19 13.5 13.5 0 0 0-26 2.8A8.5 8.5 0 0 0 16 35Z" fill="#E8EEF4" stroke="rgba(120,140,160,.22)" stroke-width="1.2"/><g fill="#7BB4E8"><circle cx="23" cy="43" r="2.5"/><circle cx="33" cy="47" r="2.5"/><circle cx="43" cy="43" r="2.5"/></g></svg>
    <svg v-else-if="weatherKind==='overcast'" viewBox="0 0 64 56"><path d="M24 31h26a8 8 0 0 0 1-16 11 11 0 0 0-21 2A7 7 0 0 0 24 31Z" fill="#CDD5DE"/><path d="M14 44h31a9.5 9.5 0 0 0 1-19 13 13 0 0 0-25 2.6A8.5 8.5 0 0 0 14 44Z" fill="#EEF2F6" stroke="rgba(110,125,140,.18)" stroke-width="1.2"/></svg>
    <svg v-else viewBox="0 0 64 56"><circle cx="42" cy="20" r="13" fill="#F8BE4F"/><path d="M16 48h32a10 10 0 0 0 1-20 14 14 0 0 0-27 3 9 9 0 0 0-6 17Z" fill="#FFFFFF" stroke="rgba(140,120,96,.18)" stroke-width="1.2"/></svg>
   </div>
   <div class="weather-temp-col">
    <div class="weather-temperature">{{ temperature }}<sup>°C</sup></div>
    <div class="weather-description">
     <span>{{ weatherText }}</span>
     <span v-if="highLow" class="weather-range"><ArrowUp :size="13"/>{{ Math.round(highLow.high) }}° <ArrowDown :size="13"/>{{ Math.round(highLow.low) }}°</span>
    </div>
   </div>
  </div>
  <div v-if="miniForecast.length" class="weather-forecast" aria-label="未来天气预报">
   <div v-for="day in miniForecast" :key="day.label" class="wf-day"><span>{{ day.label }}</span><b>{{ day.high }}° / {{ day.low }}°</b></div>
  </div>
  <button v-if="!preview&&!widget.city" class="inline-link weather-configure" @click="settings">设置天气城市</button>
  <small v-if="!preview&&weather" class="data-source">{{ weather.source }} · {{ new Date(weather.updatedAt*1000).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'}) }}</small>
  <small v-if="weatherBusy" class="data-source">正在更新…</small>
 </template>

 <!-- 3. 待办事项 -->
 <template v-else-if="widget.kind==='notes'">
  <header>
   <h2>待办事项 <span v-if="note.items.length" class="title-count">{{ todoDoneCount }}/{{ note.items.length }}</span></h2>
   <div class="widget-header-actions">
    <button class="add-button" :disabled="locked||saving" aria-label="添加待办" title="添加待办（含提醒与重复）" @click="openItem()"><Plus :size="17"/></button>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <div class="todo-list">
   <div v-for="item in note.items" :key="item.id" class="todo-row" :class="{done:itemDone(item,now)}">
    <button class="check-circle" :aria-pressed="itemDone(item,now)" :aria-label="`${itemDone(item,now)?'恢复':'完成'}${item.text}`" :disabled="locked||saving" @click="toggleTask(item.id)"><Check v-if="itemDone(item,now)" :size="14"/></button>
    <button class="item-text" :disabled="locked||saving" aria-label="编辑待办" @click="openItem(item)">{{ item.text }}</button>
    <div class="todo-meta">
     <small v-if="item.time||item.date" class="todo-when">{{ taskWhen(item) }}</small>
     <button class="row-remove" :disabled="locked||saving" aria-label="删除待办" @click="removeTask(item.id)"><Trash2 :size="13"/></button>
    </div>
   </div>
   <p v-if="!note.items.length" class="empty-state">把今天想做的事，记在这里。<button @click="openItem()">添加第一件小事</button></p>
  </div>
  <form v-if="!demo&&!preview" class="todo-quick" data-no-drag @submit.prevent="addQuickTask">
   <input v-model="quickTodo" type="text" maxlength="120" :disabled="locked||saving||note.items.length>=12" placeholder="记一件小事，回车即加…" aria-label="快速添加待办"/>
   <button v-if="quickTodo.trim()" type="submit" :disabled="locked||saving">添加</button>
  </form>
 </template>

 <!-- 4. 今日日程 -->
 <template v-else-if="widget.kind==='reminders'">
  <header>
   <div class="agenda-title-wrap">
    <h2>今日日程</h2>
    <div v-if="!demo" class="mini-seg" role="group" aria-label="切换日程日期" data-no-drag>
     <button type="button" :aria-pressed="agendaDay==='today'" @click="agendaDay='today'">今天</button>
     <button type="button" :aria-pressed="agendaDay==='tomorrow'" @click="agendaDay='tomorrow'">明天</button>
    </div>
   </div>
   <div class="widget-header-actions">
    <button class="agenda-date" aria-label="添加日程" title="添加日程" :disabled="locked" @click="openItem()"><span>{{ agendaTargetDate.getMonth()+1 }}月{{ agendaTargetDate.getDate() }}日</span><Plus :size="15"/></button>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <div class="agenda-list">
   <div v-for="event in events" :key="event.id" class="agenda-row" :class="{current:isCurrent(event),done:itemDone(event,agendaTargetDate)}">
    <button class="agenda-check" :class="{on:itemDone(event,agendaTargetDate)}" :aria-label="`${itemDone(event,agendaTargetDate)?'恢复':'完成'}${event.title}`" :disabled="locked||saving" @click="toggleEvent(event)"><Check v-if="itemDone(event,agendaTargetDate)" :size="11"/></button>
    <div class="agenda-body">
     <div class="agenda-top">
      <button class="event-title" :disabled="locked||saving" aria-label="编辑日程" @click="openItem(event)">{{ event.title }}</button>
      <span v-if="isCurrent(event)" class="event-status">进行中</span>
      <button class="row-remove" :disabled="locked||saving" aria-label="删除日程" @click="removeEvent(event.id)"><Trash2 :size="13"/></button>
     </div>
     <div class="agenda-sub">
      <span>{{ event.at.slice(11) }}<template v-if="event.end"> — {{ event.end }}</template></span>
      <span v-if="event.location" class="location"><MapPin :size="13" fill="currentColor"/>{{ event.location }}</span>
     </div>
    </div>
   </div>
   <p v-if="!events.length" class="empty-state">{{ agendaDay==='today'?'今天的时间，留给喜欢的事。':'明天还没有安排。' }}<button @click="openItem()">安排一个日程</button></p>
  </div>
 </template>

 <!-- 5. 股票行情 -->
 <template v-else-if="widget.kind==='stocks'">
  <header>
   <span class="stock-title"><strong>{{ displayQuote?.name||'股票行情' }}</strong><small class="stock-code">{{ living.symbol.startsWith('116.')?living.symbol.split('.')[1]+'.HK':living.symbol.split('.')[1] }}</small></span>
   <div class="widget-header-actions">
    <button class="stock-list-open" aria-label="查看自选列表" @click="listOpen=!listOpen;loadWatchlist()">自选 {{ watchlist.length }}</button>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <div class="stock-price-row">
   <div class="stock-price">{{ displayQuote?formatPrice(displayQuote.price):'—' }}</div>
   <div class="stock-change">{{ displayQuote?(up?'+':'')+displayQuote.change.toFixed(2):'等待行情' }}<span v-if="displayQuote">({{ up?'+':'' }}{{ displayQuote.percent.toFixed(2) }}%)</span></div>
  </div>
  <div v-if="chart&&chartBounds" class="stock-chart-wrap" data-no-drag @pointermove="onChartMove" @pointerleave="hoverIdx=null">
   <svg class="stock-chart" viewBox="0 0 340 124" preserveAspectRatio="none" role="img" aria-label="价格走势">
    <defs><linearGradient :id="`chart-${widget.id}`" x2="0" y2="1"><stop stop-color="currentColor" stop-opacity=".32"/><stop offset="1" stop-color="currentColor" stop-opacity="0"/></linearGradient></defs>
    <line v-if="chartBounds.prevY!=null" x1="0" :y1="chartBounds.prevY" x2="340" :y2="chartBounds.prevY" stroke="rgba(255,255,255,.22)" stroke-width="1" stroke-dasharray="4 4"/>
    <polygon :points="`0,124 ${chart} 340,124`" :fill="`url(#chart-${widget.id})`"/>
    <polyline :points="chart" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>
    <line v-if="hoverPoint" :x1="hoverPoint.x" y1="6" :x2="hoverPoint.x" y2="120" stroke="rgba(255,255,255,.32)" stroke-width="1"/>
    <circle :cx="(hoverPoint||chartBounds.coords.at(-1))?.x" :cy="(hoverPoint||chartBounds.coords.at(-1))?.y" r="4" fill="currentColor" stroke="#fff" stroke-width="1.5"/>
   </svg>
   <span v-if="hoverPoint" class="stock-hover-tip">{{ formatPrice(hoverPoint.v) }}</span>
   <span v-else-if="prevClose!=null" class="stock-prev-label">昨收 {{ formatPrice(prevClose) }}</span>
  </div>
  <div v-else class="stock-chart no-chart">{{ quoteBusy?'正在读取行情…':displayQuote?'暂未形成完整走势':'联网后显示真实走势' }}</div>
  <nav class="stock-periods" aria-label="行情周期"><button v-for="p in periods" :key="p.value" :class="{on:period===p.value}" :aria-pressed="period===p.value" :disabled="locked||quoteBusy" @click="period=p.value">{{ p.label }}</button></nav>
  <small v-if="!preview&&quote" class="stock-source">{{ quote.updatedAt }} · {{ quote.source }}</small>
 </template>

 <!-- 6. 电子木鱼 -->
 <template v-else-if="widget.kind==='woodfish'">
  <header>
   <h2 class="fish-title"><i/>电子木鱼</h2>
   <div class="widget-header-actions">
    <button v-if="!demo" type="button" class="hdr-pill" title="切换漂浮文案" data-no-drag @click.stop="meritWordIdx=(meritWordIdx+1)%meritWords.length">{{ meritWords[meritWordIdx] }}</button>
    <button class="sound-toggle" :aria-label="sound?'关闭木鱼声音':'打开木鱼声音'" :disabled="saving||locked" @click="toggleSound"><Volume2 v-if="sound" :size="15"/><VolumeX v-else :size="15"/></button>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <button class="fish-hit" :class="{striking}" :disabled="locked" aria-label="敲木鱼" @click="tap">
   <svg class="fish-svg" viewBox="0 0 240 130" aria-hidden="true">
    <defs>
     <radialGradient :id="`wood-${widget.id}`" cx="38%" cy="30%" r="70%"><stop offset="0%" stop-color="#E5BE92"/><stop offset="55%" stop-color="#C69564"/><stop offset="100%" stop-color="#9A6B3E"/></radialGradient>
     <linearGradient :id="`mallet-${widget.id}`" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="#D5A678"/><stop offset="100%" stop-color="#9E6E42"/></linearGradient>
    </defs>
    <ellipse cx="88" cy="112" rx="64" ry="8" fill="rgba(72,48,26,.18)"/>
    <g class="fish-body-g">
     <path d="M22 86 C18 52 46 24 88 24 C128 24 152 48 152 80 C152 102 128 111 88 111 C50 111 26 105 22 86 Z" :fill="`url(#wood-${widget.id})`"/>
     <path d="M40 50 C62 36 108 36 130 50" fill="none" stroke="rgba(255,244,228,.34)" stroke-width="2.5" stroke-linecap="round"/>
     <path d="M48 76 Q96 63 148 74" fill="none" stroke="#4D321B" stroke-width="6.5" stroke-linecap="round"/>
     <circle cx="52" cy="76" r="5" fill="#4D321B"/>
     <path d="M56 92 C78 97 110 96 134 90" fill="none" stroke="rgba(90,56,28,.22)" stroke-width="2" stroke-linecap="round"/>
    </g>
    <g class="fish-mallet-g">
     <path d="M168 54 L224 94" :stroke="`url(#mallet-${widget.id})`" stroke-width="8.5" stroke-linecap="round"/>
     <circle cx="164" cy="51" r="14" :fill="`url(#wood-${widget.id})`"/>
     <circle cx="160" cy="46" r="4" fill="rgba(255,248,236,.35)"/>
    </g>
   </svg>
  </button>
  <div class="merit-layer" aria-hidden="true">
   <span v-for="b in meritBubbles" :key="b.id" class="merit-bubble" :style="{right:`${b.x}px`,top:`${b.y}px`}">{{ b.text }}</span>
   <span v-if="demo&&!meritBubbles.length" class="merit-static">功德 <b>+1</b></span>
  </div>
  <div class="fish-footer">
   <div><strong>{{ count }}<small>次</small></strong><p>{{ todayHits>0?`今日 ${todayHits} 次 · `:'' }}心平气和 · 万事顺遂</p></div>
   <button class="fish-knock" :disabled="locked" @click="tap">敲一敲</button>
  </div>
 </template>

 <!-- 7. 月历 -->
 <template v-else-if="widget.kind==='calendar'">
  <header>
   <div class="cal-head-left">
    <button class="month-title" aria-label="回到本月" @click="monthOffset=0;selectedDay=dateKey(now)">{{ month.getFullYear() }}年{{ month.getMonth()+1 }}月</button>
    <button v-if="monthOffset!==0" type="button" class="hdr-pill on" @click="monthOffset=0;selectedDay=dateKey(now)">回今天</button>
   </div>
   <div class="widget-header-actions">
    <div class="month-controls"><button aria-label="上个月" @click="monthOffset--"><ChevronLeft :size="17"/></button><button aria-label="下个月" @click="monthOffset++"><ChevronRight :size="17"/></button></div>
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <div class="calendar-week"><span v-for="d in ['日','一','二','三','四','五','六']" :key="d">{{ d }}</span></div>
  <div class="calendar-grid">
   <button v-for="d in days" :key="d.key" :class="{muted:!d.current,today:d.key===dateKey(now),selected:selectedDay===d.key}" :aria-label="d.key" :aria-pressed="selectedDay===d.key" @click="selectedDay=d.key">
    <span v-if="d.holiday.badge" class="cal-badge" :class="d.holiday.badge==='休'?'xiu':'ban'">{{ d.holiday.badge }}</span>
    <span class="cal-num">{{ d.day }}</span>
    <span class="cal-lunar" :class="{fest:d.festival}">{{ d.lunar }}</span>
   </button>
  </div>
  <div class="calendar-detail">
   <span><b>{{ selectedDayInfo.solar }}</b> · 农历{{ selectedDayInfo.lunar }}</span>
   <span v-if="selectedDayInfo.extra" class="cal-extra">{{ selectedDayInfo.extra }}</span>
  </div>
 </template>

 <!-- 8. 纪念日 -->
 <template v-else-if="widget.kind==='countdown'">
  <svg class="lake-scene-svg" viewBox="0 0 460 286" preserveAspectRatio="xMaxYMax slice" aria-hidden="true">
   <defs>
    <linearGradient :id="`lf-${widget.id}`" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="white" stop-opacity="0"/><stop offset="34%" stop-color="white" stop-opacity="1"/></linearGradient>
    <mask :id="`lmask-${widget.id}`"><rect width="460" height="286" :fill="`url(#lf-${widget.id})`"/></mask>
    <linearGradient :id="`lsky-${widget.id}`" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="#F2D8BD" stop-opacity=".48"/><stop offset="65%" stop-color="#E5CBB5" stop-opacity=".34"/><stop offset="100%" stop-color="#C8C1B6" stop-opacity=".48"/></linearGradient>
   </defs>
   <g :mask="`url(#lmask-${widget.id})`">
    <rect width="460" height="286" :fill="`url(#lsky-${widget.id})`"/>
    <circle cx="308" cy="106" r="36" fill="#F4C592" opacity=".62"/>
    <path d="M76 184 Q182 104 274 156 T460 128 L460 186 L76 186 Z" fill="#B9AE9F" opacity=".52"/>
    <path d="M148 186 Q260 124 358 164 T460 148 L460 186 Z" fill="#9A8E80" opacity=".48"/>
    <rect x="0" y="186" width="460" height="100" fill="#B5B1AA" opacity=".42"/>
    <path d="M242 206 H364 M272 224 H340 M220 242 H376" stroke="rgba(255,250,242,.42)" stroke-width="2.2" stroke-linecap="round"/>
    <path d="M386 186 L400 146 L414 186 Z M410 186 L428 132 L446 186 Z" fill="#6D6A62" opacity=".66"/>
   </g>
  </svg>
  <header>
   <h2 class="anniversary-title">
    <span class="anniversary-ic" :class="countdown.icon||'heart'"><component :is="countdownIconComponent" :size="16" :fill="(countdown.icon||'heart')==='heart'?'currentColor':'none'"/></span>
    {{ countdown.title }}
   </h2>
   <div class="widget-header-actions">
    <button v-if="!demo" type="button" class="widget-settings-button is-inline" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
   </div>
  </header>
  <p class="countdown-date">{{ remaining.date.replace('-','年').replace('-','月') }}日 {{ new Date(remaining.date+'T12:00').toLocaleDateString('zh-CN',{weekday:'long'}) }}</p>
  <div v-if="remaining.label==='就是今天'" class="countdown-today">就是今天 ✦</div>
  <div v-else class="countdown-value"><strong>{{ remaining.days }}</strong><span>{{ remaining.label==='还有'?'天后':remaining.label==='已经'?'天了':'天前' }}</span></div>
  <p v-if="countdownQuote" class="anniversary-quote">“ {{ countdownQuote }} ”</p>
 </template>

 <!-- 9. 自定义 HTML 组件 -->
 <template v-else-if="widget.kind==='custom'">
  <div class="custom-widget-box" :data-testid="`custom-${widget.customId||'widget'}`">
   <iframe
     :key="customFrameKey"
     ref="customFrame"
     @load="customLayout"
     :src="customShell"
     :title="widget.custom?.name||'自创组件'"
     sandbox="allow-scripts"
     class="custom-widget-frame"
     scrolling="no"
     tabindex="-1"
   />
  </div>
  <button v-if="!demo" type="button" class="widget-settings-button" aria-label="小组件设置" title="设置" data-testid="widget-settings" data-no-drag @pointerdown.stop @click.stop="settings"><Settings2 :size="16"/></button>
 </template>

 <button v-if="!preview&&payload" class="resize-corner" aria-label="按比例调整组件大小" title="拖动调整大小 · Esc 取消" data-no-drag @pointerdown.stop.prevent="startResize" @pointermove.stop="resizer.move()" @pointerup.stop="finishResize()" @pointercancel.stop="finishResize(true)" @lostpointercapture="finishResize()"><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M5 15 15 5M10 15l5-5M15 15h.01" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg></button>
 <div v-if="addOpen" class="widget-popover" data-no-drag><ItemForm :key="editingItem?.id||'new'" :mode="widget.kind==='notes'?'todo':'agenda'" :item="editingItem" :busy="saving" @save="commitItem" @cancel="addOpen=false"/></div>
 <section v-if="listOpen&&widget.kind==='stocks'" class="stock-list-panel" data-no-drag><header><strong>我的自选</strong><button aria-label="关闭自选列表" @click="listOpen=false">×</button></header><button v-for="symbol in watchlist" :key="symbol" :class="{selected:symbol===living.symbol}" :disabled="saving" @click="chooseStock(symbol)"><span>{{ quoteFor(symbol)?.name||symbol }}<small>{{ symbol }}</small></span><strong>{{ quoteFor(symbol)?formatPrice(quoteFor(symbol)!.price):'—' }}</strong><small :class="{down:(quoteFor(symbol)?.change??0)<0}">{{ quoteFor(symbol)?((quoteFor(symbol)!.percent>=0?'+':'')+quoteFor(symbol)!.percent.toFixed(2)+'%'):'暂无数据' }}</small></button><p v-if="listBusy">正在更新…</p><p v-if="listError" role="alert">{{ listError }}<button @click="loadWatchlist">重试</button></p><p>点击证券查看走势；在组件设置中管理自选。</p></section>
 <p v-if="error" class="widget-error" role="alert">{{ error }}<button aria-label="关闭错误提示" @click="error=''">×</button></p>
 <div v-if="editing" class="drag-cover">按住卡片拖动</div>
</article>
</template>
