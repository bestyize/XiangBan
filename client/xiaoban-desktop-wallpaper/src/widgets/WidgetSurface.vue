<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { ChevronLeft, ChevronRight, Cloud, CloudRain, RefreshCw, Settings2, Snowflake, Sun } from 'lucide-vue-next';
import { desktop, invoke } from '../bridge.ts';
import { baseSize, calendarDays, type DesktopWidget, type WidgetPayload } from './model.ts';
import { customWidgetDocument } from './custom-document.ts';
import NotesWidget from './NotesWidget.vue';
import CountdownWidget from './CountdownWidget.vue';
import { dateKey, lunarDate, holidayInfo, calendarLabel } from './calendar.ts';
import type { WidgetContent } from './model.ts';
import FortuneVessel from '../rituals/FortuneVessel.vue';
import { fortunes, openRitual } from '../rituals/journal.ts';
const props=withDefaults(defineProps<{ widget: DesktopWidget; editing?: boolean; preview?: boolean; payload?: WidgetPayload }>(),{editing:false,preview:false});
const size=computed(()=>baseSize(props.widget.kind, props.widget.custom));
const customDocument=computed(()=>customWidgetDocument(props.widget.custom?.html || '<div style="color:#999;padding:12px;font-size:12px">未找到小组件内容</div>',props.widget.theme));
const now=ref(new Date()),monthOffset=ref(0),phase=ref<'idle'|'shaking'|'result'>('idle'),fortune=ref(0),error=ref('');
const clock=computed(()=>now.value.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit',hour12:false}));
const month=computed(()=>new Date(now.value.getFullYear(),now.value.getMonth()+monthOffset.value,1));
const selected=ref(dateKey(new Date()));
const selectedDate=computed(()=>{const [y,m,d]=selected.value.split('-').map(Number);return new Date(y,m-1,d,12);});
const selectedInfo=computed(()=>holidayInfo(selectedDate.value));
const days=computed(()=>calendarDays(month.value.getFullYear(),month.value.getMonth()));
const sameDay=(date:Date)=>date.toDateString()===now.value.toDateString();
const weather=ref<{text:string;updatedAt:number;modelAt:number;source:string}|null>(null),weatherBusy=ref(false);
const temperature=computed(()=>weather.value?.text.match(/(-?\d+(?:\.\d+)?)°C/)?.[1]??'—');
const weatherDescription=computed(()=>weather.value?.text.match(/：([^，]+)，/)?.[1]??'等待天气');
const weatherIcon=computed(()=>weatherDescription.value.includes('雪')?Snowflake:weatherDescription.value.includes('雨')?CloudRain:weatherDescription.value==='晴'?Sun:Cloud);
let clockTimer:ReturnType<typeof setInterval>|undefined,weatherTimer:ReturnType<typeof setInterval>|undefined,fortuneTimer:ReturnType<typeof setTimeout>|undefined;
let active=true;
async function refreshWeather() {
  if(props.preview||!desktop||!props.widget.city||weatherBusy.value)return;
  const requestedCity=JSON.stringify(props.widget.city);
  weatherBusy.value=true;error.value='';
  try { const result=await invoke<typeof weather.value>('get_widget_weather');if(active&&requestedCity===JSON.stringify(props.widget.city))weather.value=result; }
  catch(e){if(active&&requestedCity===JSON.stringify(props.widget.city))error.value=String(e);}finally{if(active){weatherBusy.value=false;if(requestedCity!==JSON.stringify(props.widget.city))void refreshWeather();}}
}
watch(()=>JSON.stringify(props.widget.city),()=>{weather.value=null;void refreshWeather();});
function draw() {
  if(phase.value==='shaking'||props.editing)return;
  phase.value='shaking';fortune.value=crypto.getRandomValues(new Uint32Array(1))[0]%fortunes.length;
  fortuneTimer=setTimeout(()=>{phase.value='result';},window.matchMedia('(prefers-reduced-motion: reduce)').matches?120:1850);
}
async function detail() { try { await openRitual('fortune',fortune.value); } catch(e){error.value=String(e);} }
async function settings() { if(props.preview)return;try{await invoke('show_widget_settings');}catch(e){error.value=String(e);} }
let drag:{startX:number;startY:number;grabX:number;grabY:number;moved:boolean}|null=null;
let suppressClick=false,clickTimer:ReturnType<typeof setTimeout>|undefined;
let moving=false,pending:{grabX:number;grabY:number;commit:boolean}|null=null;
async function movePending(){
  if(moving)return;moving=true;
  try{while(pending){const point=pending;pending=null;try{await invoke('move_desktop_widget',point);}catch(e){error.value=String(e);pending=null;}}}finally{moving=false;}
}
async function saveContent(content:WidgetContent){await invoke('save_desktop_widget_content',{content});}
function resetMonth(){monthOffset.value=0;selected.value=dateKey(now.value);}
function startDrag(event:PointerEvent){
  if(event.button!==0||props.preview||!props.payload)return;
  if(!props.editing&&(event.target as Element).closest('input,textarea,select,[data-no-drag]'))return;
  if(clickTimer)clearTimeout(clickTimer);suppressClick=false;
  drag={startX:event.screenX,startY:event.screenY,grabX:Math.max(0,Math.min(1,event.clientX/Math.max(1,window.innerWidth))),grabY:Math.max(0,Math.min(1,event.clientY/Math.max(1,window.innerHeight))),moved:false};
  // Capture on the original target so a stationary button press keeps its normal click.
  (event.target as Element).setPointerCapture?.(event.pointerId);
}
function dragTo(event:PointerEvent,commit=false){
  if(!drag)return;const dx=event.screenX-drag.startX,dy=event.screenY-drag.startY;
  if(!drag.moved&&Math.hypot(dx,dy)>5){drag.moved=true;suppressClick=true;if(event.type==='pointermove')(event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);}
  if(drag.moved){event.preventDefault();pending={grabX:drag.grabX,grabY:drag.grabY,commit};void movePending();}
  if(commit){drag=null;clickTimer=setTimeout(()=>{suppressClick=false;},0);}
}
function captureClick(event:MouseEvent){if(suppressClick){event.preventDefault();event.stopImmediatePropagation();suppressClick=false;}}
function visible(){now.value=new Date();if(!document.hidden&&props.widget.kind==='weather')void refreshWeather();}
onMounted(()=>{clockTimer=setInterval(()=>now.value=new Date(),15000);if(props.widget.kind==='weather'){void refreshWeather();weatherTimer=setInterval(()=>{if(!document.hidden)void refreshWeather();},1800000);}document.addEventListener('visibilitychange',visible);});
onBeforeUnmount(()=>{active=false;if(clockTimer)clearInterval(clockTimer);if(weatherTimer)clearInterval(weatherTimer);if(fortuneTimer)clearTimeout(fortuneTimer);if(clickTimer)clearTimeout(clickTimer);document.removeEventListener('visibilitychange',visible);});
</script>
<template>
  <article class="widget-surface" :class="[widget.theme,widget.kind,{editing}]" :style="{width:`${size.width}px`,height:`${size.height}px`}" @pointerdown="startDrag" @pointermove="dragTo($event)" @pointerup="dragTo($event,true)" @pointercancel="dragTo($event,true)" @click.capture="captureClick">
    <div v-if="editing" class="widget-drag-cover" aria-hidden="true" title="按住组件任意位置拖动"><span>编辑中 · 按住拖动</span></div>
    <header class="widget-heading"><span>{{ widget.kind==='fortune'?'一签好意':widget.kind==='calendar'?'日有所序':widget.kind==='notes'?'心事有处':widget.kind==='countdown'?'好日将至':widget.kind==='custom'?(widget.custom?.name||'桌面小组件'):'晴雨之间' }}</span><button v-if="!editing&&!preview" class="widget-settings" aria-label="设置这个组件" title="设置这个组件" @click="settings"><Settings2 :size="14"/></button></header>
    <template v-if="widget.kind==='fortune'"><div v-if="phase!=='result'" class="mini-vessel"><FortuneVessel :shaking="phase==='shaking'"/></div><div v-else class="mini-fortune" aria-live="polite"><span class="mini-sign">第 {{ String(fortune+1).padStart(2,'0') }} 签</span><h2>{{ fortunes[fortune].title }}</h2><p class="mini-verse">{{ fortunes[fortune].verse }}</p><p>{{ fortunes[fortune].blessing }}</p><span class="mini-action">{{ fortunes[fortune].action }}</span></div><div class="fortune-controls"><button v-if="phase!=='result'" :disabled="phase==='shaking'||editing" class="fortune-draw" @click="draw">{{ phase==='shaking'?'好签正在途中…':'求一支好签' }}</button><template v-else><button class="fortune-draw" :disabled="editing" @click="detail">展开好签册</button><button class="tiny-link" @click="phase='idle'">再求一签</button></template><small>心有所愿，日有所暖</small></div></template>
    <template v-else-if="widget.kind==='calendar'"><div class="clock-row"><strong>{{ clock }}</strong><span>{{ now.toLocaleDateString('zh-CN',{month:'long',day:'numeric',weekday:'long'}) }}</span></div><div class="month-navigation"><button aria-label="上个月" @click="monthOffset--"><ChevronLeft :size="14"/></button><button class="month-name" @click="resetMonth">{{ month.getFullYear() }} 年 {{ month.getMonth()+1 }} 月</button><button aria-label="下个月" @click="monthOffset++"><ChevronRight :size="14"/></button></div><div class="calendar-week"><span v-for="day in ['一','二','三','四','五','六','日']" :key="day">{{ day }}</span></div><div class="calendar-grid"><button v-for="day in days" :key="dateKey(day.date)" :class="{muted:!day.current,today:sameDay(day.date),selected:selected===dateKey(day.date),weekend:day.date.getDay()===0||day.date.getDay()===6,makeup:!!holidayInfo(day.date).makeup}" :aria-label="`${dateKey(day.date)} ${lunarDate(day.date).full} ${holidayInfo(day.date).holiday} ${holidayInfo(day.date).makeup?'调休上班':''}`" :aria-pressed="selected===dateKey(day.date)" @click="selected=dateKey(day.date)"><span>{{ day.day }}</span><small>{{ calendarLabel(day.date) }}</small><i v-if="holidayInfo(day.date).badge" :class="{work:!!holidayInfo(day.date).makeup}">{{ holidayInfo(day.date).badge }}</i></button></div><div class="calendar-detail" aria-live="polite"><span>{{ selectedDate.toLocaleDateString('zh-CN',{month:'long',day:'numeric'}) }} · {{ lunarDate(selectedDate).full }}</span><span>{{ selectedInfo.holiday?`${selectedInfo.holiday}假期`:selectedInfo.makeup?`${selectedInfo.makeup}调休上班`:'' }}</span></div><p class="calendar-note">{{ holidayInfo(month).covered?'休 / 班 · 国务院公布的年度安排':'本年调休数据未收录，仅展示日期与农历' }}</p></template>
    <NotesWidget v-else-if="widget.kind==='notes'" :note="widget.note" :preview="preview" :editing="editing" :save="saveContent"/>
    <CountdownWidget v-else-if="widget.kind==='countdown'" :data="widget.countdown" :now="now"/>
    <div v-else-if="widget.kind==='custom'" class="custom-widget-box" :data-testid="`custom-${widget.customId||'widget'}`">
      <iframe
        :srcdoc="customDocument"
        sandbox="allow-scripts"
        class="custom-widget-frame"
        tabindex="-1"
      />
    </div>
    <template v-else><div class="weather-city"><span>{{ widget.city?.name??'选一座你关心的城市' }}</span><button v-if="widget.city&&!preview" :disabled="weatherBusy" aria-label="刷新天气" @click="refreshWeather"><RefreshCw :size="12" :class="{refreshing:weatherBusy}"/></button></div><div class="weather-main"><div><strong>{{ temperature }}<i v-if="temperature!=='—'">°</i></strong><span>{{ widget.city?weatherBusy&&!weather?'正在读取天气':weatherDescription:'设置城市后显示天气' }}</span></div><component :is="weatherIcon" :size="48" :stroke-width="1.2"/></div><p v-if="weather" class="weather-message">{{ weather.text.split('。').slice(1).join('。').trim() || '愿每一次出门，都遇见好天气。' }}</p><button v-else-if="!widget.city" class="weather-configure" :disabled="preview" @click="settings">选择城市<ChevronRight :size="12"/></button><p class="weather-source">{{ weather ? `${new Date(weather.updatedAt*1000).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'})} 更新 · ${weather.source}` : preview&&widget.city ? '实时天气显示在桌面上' : '天气数据尚未载入' }}</p></template>
    <p v-if="error" class="widget-error" role="alert" title="点击关闭" @click="error=''">{{ error }}</p>
  </article>
</template>
<style scoped>
.widget-surface.custom.transparent{padding:0;border:0;border-radius:0;text-shadow:none}
.custom.transparent .custom-widget-box{height:100%;margin:0;border-radius:0}
.custom.transparent .widget-heading{position:absolute;top:8px;right:8px;z-index:2;opacity:0;pointer-events:none}
.custom.transparent .widget-heading>span{display:none}
.custom.transparent:hover .widget-heading,.custom.transparent:focus-within .widget-heading{opacity:1;pointer-events:auto}
.widget-surface{position:relative;box-sizing:border-box;overflow:hidden;padding:17px 18px;border-radius:18px;font-family:'Microsoft YaHei','PingFang SC',sans-serif;user-select:none;touch-action:none;cursor:grab;color:#edf1e9;border:1px solid #e3ead13d;background:linear-gradient(135deg,#ffffff24,#ffffff0c),#1230398c;box-shadow:0 8px 24px #00131b24;backdrop-filter:blur(18px)}.custom-widget-box{position:relative;width:100%;height:calc(100% - 24px);margin-top:5px;border-radius:10px;overflow:hidden}.custom-widget-frame{width:100%;height:100%;border:0;background:transparent;display:block}.widget-surface.paper{color:#70573f;background:radial-gradient(ellipse at 35% 20%,#fff8e9,#eee2cb);border-color:#ceb894a3;box-shadow:0 8px 24px #49302126}.widget-surface.transparent{background:none;border-color:transparent;box-shadow:none;backdrop-filter:none;text-shadow:0 1px 5px #0009}.widget-drag-cover{position:absolute;inset:0;z-index:5;cursor:grab;touch-action:none}.widget-drag-cover:active{cursor:grabbing}.widget-drag-cover span{position:absolute;left:50%;bottom:10px;transform:translateX(-50%);font-size:11px;letter-spacing:1px;white-space:nowrap;color:#f3e2b8;background:#234440d9;padding:4px 12px;border-radius:12px}.paper .widget-drag-cover span{color:#705038;background:#ead8bacc}.widget-surface.editing{outline:1px dashed #e9d29c;outline-offset:-3px}.widget-heading{display:flex;align-items:center;justify-content:space-between;height:19px;font-size:11px;letter-spacing:2px;opacity:.7}.widget-heading button{display:flex;border:0;background:none;color:inherit;padding:3px;cursor:pointer}.widget-heading .widget-settings{opacity:.38;padding:7px;margin:-7px;transition:opacity .2s}.widget-surface:hover .widget-settings,.widget-settings:focus-visible{opacity:1}.mini-vessel{height:204px;margin:1px -15px -4px}.fortune-controls{text-align:center;display:grid;justify-items:center;gap:10px}.fortune-draw{border:1px solid #d6b67e5c;background:linear-gradient(110deg,#a55346,#813a32);border-radius:6px;color:#fce8c7;font:11px 'Microsoft YaHei',sans-serif;letter-spacing:1px;padding:9px 19px;cursor:pointer}.fortune-controls small{font-size:10px;letter-spacing:2px;opacity:.6}.mini-fortune{text-align:center;padding-top:8px;height:178px;box-sizing:border-box}.mini-sign{font-size:10px;letter-spacing:2px;opacity:.65}.mini-fortune h2{font:23px 'KaiTi','STKaiti',serif;letter-spacing:4px;margin:10px 0 6px}.paper .mini-fortune h2{color:#934b3d}.mini-fortune p{font-size:11px;line-height:1.7;margin:8px 0}.mini-verse{font-family:'KaiTi','STKaiti',serif!important;opacity:.8}.mini-action{font-size:10px;opacity:.7}.tiny-link{border:0;background:none;color:inherit;font-size:11px;cursor:pointer;opacity:.7;padding:2px 8px}.clock-row{display:grid;gap:4px;padding-top:2px}.clock-row strong{font-size:36px;font-weight:300;letter-spacing:1px;font-variant-numeric:tabular-nums;line-height:1.1}.clock-row>span{font-size:11px;opacity:.7;letter-spacing:1px}.month-navigation{display:flex;align-items:center;justify-content:space-between;margin:10px 0 6px}.month-navigation button{background:none;border:0;color:inherit;display:flex;cursor:pointer;padding:3px}.month-name{font-size:11px;letter-spacing:1px}.calendar-week,.calendar-grid{display:grid;grid-template-columns:repeat(7,1fr);text-align:center}.calendar-week{font-size:10px;opacity:.6;margin-bottom:4px}.calendar-grid{font-size:11px;row-gap:2px}.calendar-grid button{position:relative;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;height:30px;width:34px;justify-self:center;line-height:1;border:0;border-radius:6px;background:none;color:inherit;font:inherit;cursor:pointer}.calendar-grid small{font-size:8px;opacity:.7;max-width:32px;white-space:nowrap;overflow:hidden}.calendar-grid i{position:absolute;right:0;top:0;font-size:7px;font-style:normal;color:#8bbdab}.calendar-grid i.work{color:#df9986}.paper .calendar-grid i{color:#48846d}.paper .calendar-grid i.work{color:#ae5144}.calendar-grid .selected:not(.today){outline:1px solid #bba26d80}.calendar-detail{display:flex;justify-content:space-between;gap:5px;margin-top:10px;font-size:9px;opacity:.75}.calendar-grid .makeup:not(.today){color:inherit}.calendar-grid .muted{opacity:.23}.calendar-grid .weekend:not(.muted):not(.makeup){color:#d5b584}.calendar-grid .today{border-radius:50%;background:#c6a66d;color:#163437;font-weight:650}.paper .calendar-grid .today{background:#a75444;color:#fff3db}.calendar-note{font:10px 'KaiTi','STKaiti',serif;letter-spacing:1px;opacity:.6;text-align:center;margin:8px 0 0}.weather-city{display:flex;justify-content:space-between;align-items:center;margin-top:14px;font-size:12px;letter-spacing:1px}.weather-city button{border:0;background:none;color:inherit;opacity:.7;cursor:pointer;display:flex;padding:6px;margin:-6px}.weather-main{display:flex;align-items:center;justify-content:space-between;margin-top:13px;color:#e5d2aa}.paper .weather-main{color:#8a6a38}.paper .calendar-grid .weekend:not(.muted):not(.makeup){color:#a75444}.weather-main>div{display:grid;gap:3px}.weather-main strong{font-size:47px;font-weight:300;line-height:1.1;font-variant-numeric:tabular-nums}.weather-main strong i{font-size:26px;font-style:normal;vertical-align:top}.weather-main span{font-size:11px;opacity:.8}.weather-message{font-size:11px;opacity:.8;line-height:1.6;margin:13px 0 0;max-height:36px;overflow:hidden}.weather-source{position:absolute;bottom:16px;left:18px;right:18px;font-size:10px;line-height:1.5;opacity:.55;margin:0;overflow-wrap:anywhere}.weather-configure{display:flex;align-items:center;gap:4px;margin-top:13px;border:0;background:none;color:inherit;padding:4px 0;font-size:12px;cursor:pointer;opacity:.85}.widget-error{position:absolute;bottom:5px;left:12px;right:12px;z-index:2;cursor:pointer;font-size:10px;background:#471e1deb;color:#ffe4cd;padding:6px;border-radius:4px;margin:0;line-height:1.5;max-height:48px;overflow:auto}.widget-surface button:disabled{opacity:.4;cursor:default}.widget-surface button:focus-visible{outline:1px solid #bba26d;outline-offset:3px}@keyframes refresh{to{transform:rotate(360deg)}}.refreshing{animation:refresh 1.2s linear infinite}@media(prefers-reduced-motion:reduce){*{animation:none!important;transition:none!important}}
</style>
