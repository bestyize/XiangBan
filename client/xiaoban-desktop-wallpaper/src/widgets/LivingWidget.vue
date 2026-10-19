<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { Check, Plus, RefreshCw, Trash2, Volume2, VolumeX } from 'lucide-vue-next';
import { desktop, invoke } from '../bridge.ts';
import { emptyNote } from './content.ts';
import type { DesktopWidget, WidgetContent, LivingData, ReminderItem, NoteData } from './model.ts';

type Weather = { text:string; updatedAt:number; source:string };
type ForecastDay={date:string;high:number;low:number;code:number};
type Quote = {symbol:string;name:string;price:number;change:number;percent:number;updatedAt:string;points:number[];period:string;source:string};
const props=withDefaults(defineProps<{
  widget:DesktopWidget; now:Date; preview?:boolean; editing?:boolean; weather?:Weather|null; weatherBusy?:boolean;
  save?:(content:WidgetContent)=>Promise<void>; refreshWeather?:()=>void; settings?:()=>void;
}>(),{preview:false,editing:false});
const locked=computed(()=>props.preview||props.editing);
const baseLiving=():LivingData=>({reminders:[],count:0,symbol:'1.000001'});
const living=computed(()=>props.widget.living??baseLiving());
const localError=ref('');
const clock=computed(()=>props.now.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit',hour12:false}));
const dateLabel=computed(()=>props.now.toLocaleDateString('zh-CN',{month:'long',day:'numeric',weekday:'long'}));
const todayKey=computed(()=>[props.now.getFullYear(),String(props.now.getMonth()+1).padStart(2,'0'),String(props.now.getDate()).padStart(2,'0')].join('-'));
const forecast=ref<ForecastDay[]>([]),forecastError=ref('');
let forecastBusy=false,forecastQueued=false;
const displayForecast=computed(()=>props.preview?[{date:todayKey.value,high:28,low:22,code:2},{date:'',high:27,low:21,code:0},{date:'',high:25,low:20,code:61}]:forecast.value);
function weatherGlyph(code:number){return code===0?'☀️':code>=51&&code<=99?'🌧️':'⛅';}
async function loadForecast(){
  if(props.preview||!desktop||props.widget.kind!=='weather'||!props.widget.city||document.hidden)return;
  if(forecastBusy){forecastQueued=true;return;}
  forecastBusy=true;forecastError.value='';
  const requested=JSON.stringify(props.widget.city);
  try{const result=await invoke<{days:ForecastDay[]}>('get_widget_forecast');if(requested===JSON.stringify(props.widget.city))forecast.value=result.days;}
  catch(e){if(requested===JSON.stringify(props.widget.city))forecastError.value=String(e);}
  finally{forecastBusy=false;if(forecastQueued){forecastQueued=false;void loadForecast();}}
}
watch(()=>JSON.stringify(props.widget.city),()=>{forecast.value=[];void loadForecast();});
const weatherDesc=computed(()=>props.weather?.text.match(/：([^，]+)，/)?.[1]??'天气待加载');
const temp=computed(()=>props.weather?.text.match(/(-?\d+(?:\.\d+)?)°C/)?.[1]??'—');
const stockChoices=[{symbol:'1.000001',name:'上证指数',code:'SH'},{symbol:'0.399001',name:'深证成指',code:'SZ'},{symbol:'0.399006',name:'创业板指',code:'CYB'}];
const selectedStock=computed(()=>stockChoices.find(x=>x.symbol===living.value.symbol)??{symbol:living.value.symbol,name:'自选股票',code:living.value.symbol.split('.')[1]??'SH/SZ'});
const periodChoices=['1D','5D'] as const;
const period=ref<'1D'|'5D'>('1D');
let reloadQueued=false;
const quote=ref<Quote|null>(null),quoteBusy=ref(false),quoteError=ref('');
const quoteTime=computed(()=>quote.value?.updatedAt?.replace('T',' ').slice(5,16)??'—');
const chartPoints=computed(()=>{
  const series=quote.value?.points??[];
  if(series.length<2)return '';
  const min=Math.min(...series),max=Math.max(...series),spread=Math.max(.001,max-min);
  return series.map((v,i)=>((i/(series.length-1))*316).toFixed(1)+','+(96-(v-min)/spread*78).toFixed(1)).join(' ');
});
const quoteUp=computed(()=>Number(quote.value?.change??0)>=0);
const notes=computed(()=>props.widget.note??emptyNote());
const shownTasks=computed(()=>notes.value.items.slice(0,4));
const doneCount=computed(()=>notes.value.items.filter(x=>x.done).length);
const taskText=ref(''),addingTask=ref(false);
const reminderList=computed(()=>[...living.value.reminders].sort((a,b)=>a.at.localeCompare(b.at)).slice(0,4));
const newReminder=ref(''),reminderAt=ref(''),addingReminder=ref(false),editingReminderId=ref<string|null>(null);
const count=ref(living.value.count),sound=ref(true),tapping=ref(false);
let lastSynced=0;
watch(()=>living.value.count,value=>{if(value>=lastSynced)count.value=Math.max(count.value,value);});
let quotePoll:ReturnType<typeof setInterval>|undefined,hitTimer:ReturnType<typeof setTimeout>|undefined;
let saveQueue:Promise<void>=Promise.resolve();
function persist(content:WidgetContent){
  if(locked.value||!props.save)return Promise.resolve();
  localError.value='';
  const operation=saveQueue.catch(()=>{}).then(()=>props.save!(content));
  saveQueue=operation.catch(e=>{localError.value=String(e);});
  return operation;
}
function notePatch(items:NoteData['items']){
  return persist({note:{...notes.value,mode:'todo',items}});
}
async function addTask(){
  const text=taskText.value.trim();
  if(!text||notes.value.items.length>=12||locked.value)return;
  try{await notePatch([...notes.value.items,{id:crypto.randomUUID().replaceAll('-','').slice(0,16),text,done:false}]);taskText.value='';addingTask.value=false;}catch{}
}
function toggleTask(id:string){
  if(locked.value)return;
  void notePatch(notes.value.items.map(item=>item.id===id?{...item,done:!item.done}:item));
}
function removeTask(id:string){
  if(locked.value)return;
  void notePatch(notes.value.items.filter(item=>item.id!==id));
}
function patchLiving(changes:Partial<LivingData>){return persist({living:{...living.value,...changes}});}
function editReminder(item:ReminderItem){if(locked.value)return;editingReminderId.value=item.id;newReminder.value=item.title;reminderAt.value=item.at;addingReminder.value=true;}
function startReminder(){editingReminderId.value=null;newReminder.value='';reminderAt.value='';addingReminder.value=true;}
function addReminder(){
  if(locked.value||!newReminder.value.trim()||!reminderAt.value||(!editingReminderId.value&&living.value.reminders.length>=12))return;
  const date=new Date(reminderAt.value);
  if(!Number.isFinite(date.getTime())){localError.value='请选择有效的提醒时间';return;}
  const item:ReminderItem={id:crypto.randomUUID().replaceAll('-','').slice(0,16),title:newReminder.value.trim().slice(0,60),at:reminderAt.value.slice(0,16),done:false};
  const list=editingReminderId.value?living.value.reminders.map(x=>x.id===editingReminderId.value?{...item,id:x.id,done:x.done}:x):[...living.value.reminders,item];
  void patchLiving({reminders:list}).then(()=>{newReminder.value='';addingReminder.value=false;editingReminderId.value=null;}).catch(()=>{});
}
function toggleReminder(id:string){if(!locked.value)void patchLiving({reminders:living.value.reminders.map(x=>x.id===id?{...x,done:!x.done}:x)});}
function removeReminder(id:string){if(!locked.value)void patchLiving({reminders:living.value.reminders.filter(x=>x.id!==id)});}
function reminderTime(value:string){const date=new Date(value);return date.toDateString()===props.now.toDateString()?value.slice(11):value.slice(5,10)+' '+value.slice(11);}
async function chooseStock(symbol:string){
  if(locked.value||symbol===living.value.symbol)return;
  quote.value=null;quoteError.value='';period.value='1D';
  try{await patchLiving({symbol});}catch{}
}
async function loadQuote(){
  if(props.widget.kind!=='stocks'||props.preview||!desktop||document.hidden)return;
  if(quoteBusy.value){reloadQueued=true;return;}
  quoteBusy.value=true;quoteError.value='';
  try{
    const result=await invoke<Quote>('get_widget_stocks',{period:period.value});
    if(result.symbol===living.value.symbol&&result.period===period.value)quote.value=result;
  }catch(e){quoteError.value=String(e);}finally{quoteBusy.value=false;if(reloadQueued){reloadQueued=false;void loadQuote();}}
}
watch([()=>living.value.symbol,period],()=>{quote.value=null;void loadQuote();});
function visible(){if(!document.hidden){void loadQuote();void loadForecast();}}
onMounted(()=>{
  if(props.widget.kind==='stocks'){void loadQuote();quotePoll=setInterval(()=>{if(!document.hidden)void loadQuote();},60000);document.addEventListener('visibilitychange',visible);}
  if(props.widget.kind==='weather'){void loadForecast();document.addEventListener('visibilitychange',visible);}
});
onBeforeUnmount(()=>{if(quotePoll)clearInterval(quotePoll);if(hitTimer)clearTimeout(hitTimer);document.removeEventListener('visibilitychange',visible);});
let audio:AudioContext|null=null;
function playClick(){
  if(!sound.value)return;
  try{
    audio??=new AudioContext();
    const oscillator=audio.createOscillator(),gain=audio.createGain(),t=audio.currentTime;
    oscillator.type='sine';oscillator.frequency.setValueAtTime(340,t);oscillator.frequency.exponentialRampToValueAtTime(150,t+.11);
    gain.gain.setValueAtTime(.001,t);gain.gain.exponentialRampToValueAtTime(.17,t+.012);gain.gain.exponentialRampToValueAtTime(.001,t+.15);
    oscillator.connect(gain);gain.connect(audio.destination);oscillator.start(t);oscillator.stop(t+.16);
  }catch{}
}
function tap(){
  if(locked.value)return;
  count.value=Math.min(9999999,count.value+1);
  lastSynced=count.value;
  tapping.value=true;clearTimeout(hitTimer);hitTimer=setTimeout(()=>{tapping.value=false;},170);
  playClick();
  // Serial writes preserve the exact count under rapid clicks; each instance stores its own data.
  void patchLiving({count:count.value}).catch(()=>{});
}
function resetCount(){if(locked.value||!window.confirm('要清空这只木鱼的敲击次数吗？'))return;count.value=0;lastSynced=0;void patchLiving({count:0});}
function weatherRefresh(){if(!props.preview&&!props.editing)props.refreshWeather?.();}
</script>

<template>
  <div class="living-content" :class="[widget.kind,{preview:preview}]">
    <template v-if="widget.kind==='clock'">
      <p class="living-overline">{{ dateLabel }}</p>
      <div class="clock-digits">{{ clock }}</div>
      <p class="clock-caption">好的一天，从此刻开始</p>
      <svg class="clock-landscape" viewBox="0 0 360 145" preserveAspectRatio="xMidYMax slice" aria-hidden="true">
        <defs><linearGradient id="mountain-fog" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#e7edf4"/><stop offset="1" stop-color="#b6c9dc"/></linearGradient></defs>
        <circle cx="283" cy="61" r="29" fill="#f6d8bd"/>
        <path d="M-20 116 Q43 80 74 101 Q125 49 183 94 Q220 55 258 86 Q308 65 390 111 V160 H-20Z" fill="#dce7ef"/>
        <path d="M-20 132 L58 97 86 113 144 72 195 117 248 94 290 115 343 69 390 121 V160 H-20Z" fill="url(#mountain-fog)"/>
        <path d="M-20 143 Q55 125 113 140 T260 137 T390 139 V160 H-20Z" fill="#879fb5" opacity=".45"/>
      </svg>
    </template>

    <template v-else-if="widget.kind==='weather'">
      <div class="weather-degree"><strong>{{ preview?'26':temp }}</strong><span>°</span></div>
      <div class="weather-condition">{{ preview?'多云':weatherDesc }}</div>
      <div class="weather-illustration" aria-hidden="true">
        <svg viewBox="0 0 200 135"><defs><filter id="wc" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="5"/></filter></defs>
          <circle cx="146" cy="46" r="26" fill="#fff0b8" opacity=".8" filter="url(#wc)"/>
          <path d="M38 109C9 99 25 63 58 68 64 32 109 20 128 49 152 38 179 54 179 78 195 96 180 116 155 116H51C46 116 42 113 38 109Z" fill="#fff" opacity=".94"/>
          <path d="M65 110C49 104 58 84 77 87 83 62 116 65 126 84 151 78 162 112 135 118H76Z" fill="#dcecff" opacity=".8"/>
        </svg>
      </div>
      <div v-if="displayForecast.length" class="weather-days">
        <div v-for="(day,index) in displayForecast" :key="index"><span>{{ ['今天','明天','后天'][index] }}</span><b>{{ weatherGlyph(day.code) }}</b><small>{{ day.low }}° / {{ day.high }}°</small></div>
      </div>
      <div v-else class="forecast-unavailable">{{ forecastError?'三日预报暂不可用':'正在载入三日预报…' }}</div>
      <div class="weather-foot">
        <div><span>{{ preview?'演示预览':weather?new Date(weather.updatedAt*1000).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'})+' 更新':'选择城市后显示天气' }}</span><small>{{ preview?'示例城市 · 深圳':widget.city?.name??'尚未设置城市' }}</small></div>
        <button :disabled="locked||weatherBusy" aria-label="刷新天气" title="刷新天气" @click="weatherRefresh"><RefreshCw :size="16" :class="{spinning:weatherBusy}"/></button>
      </div>
    </template>

    <template v-else-if="widget.kind==='notes'">
      <div class="living-topline"><strong>待办</strong><span>{{ doneCount }} / {{ notes.items.length }}</span><button class="plus-action" aria-label="新增待办" :disabled="locked||notes.items.length>=12" @click="addingTask=!addingTask"><Plus :size="18"/></button></div>
      <div class="task-stack">
        <div v-if="!notes.items.length" class="living-empty">今天想完成什么？<small>写下一件小事，就从这里开始。</small></div>
        <div v-for="item in shownTasks" :key="item.id" class="task-row"><button class="task-check" :class="{checked:item.done}" :disabled="locked" :aria-label="'切换待办：'+item.text" @click="toggleTask(item.id)"><Check v-if="item.done" :size="14"/></button><span :class="{done:item.done}">{{ item.text }}</span><button class="remove-mini" :disabled="locked" :aria-label="'删除待办：'+item.text" @click="removeTask(item.id)"><Trash2 :size="13"/></button></div>
        <small v-if="notes.items.length>4" class="extra-line">还有 {{ notes.items.length-4 }} 项 · 在设置中查看全部</small>
      </div>
      <form v-if="addingTask&&!locked" class="inline-form" @pointerdown.stop @submit.prevent="addTask"><input v-model="taskText" maxlength="120" autofocus placeholder="下一件小事…"/><button aria-label="确认新增待办" :disabled="!taskText.trim()">添加</button></form>
      <p v-else class="living-bottom-text">{{ notes.items.length===doneCount?'今天的小事都完成了':'把每件小事，慢慢做好。' }}</p>
    </template>

    <template v-else-if="widget.kind==='reminders'">
      <div class="living-topline"><strong>提醒</strong><span>{{ reminderList.length?'今天与近期':'安排好每一天' }}</span><button class="plus-action" aria-label="新增提醒" :disabled="locked||living.reminders.length>=12" @click="addingReminder?addingReminder=false:startReminder()"><Plus :size="18"/></button></div>
      <div v-if="!reminderList.length" class="living-empty reminders-empty">还没有提醒<small>点击右上角 +，记录重要的时刻。</small></div>
      <div v-else class="reminder-stack"><div v-for="(r,i) in reminderList" :key="r.id" class="reminder-row"><button class="reminder-dot" :class="['color-'+i,{done:r.done}]" :disabled="locked" :aria-label="'完成提醒：'+r.title" @click="toggleReminder(r.id)"/><time>{{ reminderTime(r.at) }}</time><button type="button" class="reminder-name" :class="{done:r.done}" :disabled="locked" :aria-label="'编辑提醒：'+r.title" @click="editReminder(r)">{{ r.title }}</button><button :disabled="locked" class="remove-mini" :aria-label="'删除提醒：'+r.title" @click="removeReminder(r.id)"><Trash2 :size="13"/></button></div></div>
      <form v-if="addingReminder&&!locked" class="reminder-editor" @pointerdown.stop @submit.prevent="addReminder"><input v-model="newReminder" maxlength="60" placeholder="提醒内容" required/><input v-model="reminderAt" type="datetime-local" required/><div><button type="button" @click="addingReminder=false">取消</button><button type="submit">{{ editingReminderId?'保存修改':'添加提醒' }}</button></div></form>
      <p class="living-bottom-text">所有提醒保存在本机 · 本组件为日程摘要</p>
    </template>

    <template v-else-if="widget.kind==='stocks'">
      <div class="stock-header"><div><strong>{{ preview?'AAPL':quote?.name||selectedStock.name }}</strong><small>{{ preview?'Apple Inc.':selectedStock.code+' · 指数行情' }}</small></div><div class="stock-value"><b>{{ preview?'172.28':quote?quote.price.toFixed(2):'—' }}</b><span :class="quoteUp?'rising':'falling'">{{ preview?'+1.54%':quote?(quote.change>=0?'+':'')+quote.percent.toFixed(2)+'%':'行情待连接' }}</span></div></div>
      <div class="stock-graph"><svg viewBox="0 0 320 110" preserveAspectRatio="none" aria-label="价格趋势图"><path v-for="i in 3" :key="i" :d="'M0 '+(i*27)+' H320'" stroke="currentColor" opacity=".1" stroke-dasharray="3 5"/><polyline v-if="chartPoints" :points="chartPoints" fill="none" :stroke="quoteUp?'#ea5d59':'#47b58a'" stroke-width="2.7" stroke-linecap="round" stroke-linejoin="round"/><polyline v-else-if="preview" points="0,85 20,75 44,78 66,55 84,65 102,45 123,56 146,37 166,54 188,44 210,24 235,36 260,17 280,28 320,9" fill="none" stroke="#5acba1" stroke-width="3" stroke-linecap="round"/></svg><p v-if="!chartPoints&&!preview">{{ quoteBusy?'正在获取市场行情…':quoteError?'行情暂时不可用，请重试':'等待行情数据' }}</p></div>
      <div class="stock-periods"><button v-for="p in periodChoices" :key="p" :class="{selected:period===p}" :disabled="locked" @click="period=p">{{ p }}</button><span>1M</span><span>3M</span><span>1Y</span><button class="stock-reload" :disabled="locked||quoteBusy" aria-label="重新获取行情" @click="loadQuote"><RefreshCw :size="14"/></button></div>
      <div class="stock-tickers"><button v-for="choice in stockChoices" :key="choice.symbol" :disabled="locked" :class="{selected:!preview&&living.symbol===choice.symbol}" @click="chooseStock(choice.symbol)">{{ choice.name }}</button></div>
      <p class="stock-source">{{ preview?'示例行情 · 非实时':quote?'东方财富公开数据 · '+quoteTime:quoteError?'数据不可用 · 请检查网络':'等待行情' }}</p>
    </template>

    <template v-else-if="widget.kind==='woodfish'">
      <div class="woodfish-count">{{ preview?'108':count }}</div><p class="woodfish-caption">今日功德 +{{ preview?'108':count }}</p>
      <button class="fish-target" :class="{hit:tapping}" :disabled="locked" aria-label="敲击电子木鱼" @click="tap">
        <svg viewBox="0 0 310 175" aria-hidden="true">
          <defs><radialGradient id="fishwood" cx=".33" cy=".2" r=".85"><stop stop-color="#e6b884"/><stop offset=".6" stop-color="#ad7346"/><stop offset="1" stop-color="#774629"/></radialGradient><linearGradient id="fishhammer" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#e7b98a"/><stop offset="1" stop-color="#95603d"/></linearGradient></defs>
          <ellipse cx="132" cy="152" rx="85" ry="12" fill="#9e7659" opacity=".14"/>
          <path d="M64 123 C58 79 96 52 147 60 C194 66 218 96 206 133 C194 157 80 161 64 123Z" fill="url(#fishwood)"/>
          <path d="M86 119 Q134 133 187 117" stroke="#653a28" stroke-width="10" stroke-linecap="round" fill="none"/>
          <path d="M105 132 Q143 140 168 130" fill="none" stroke="#f6d3a2" stroke-width="3" opacity=".5"/>
          <path d="M210 47 L276 10" stroke="#925c39" stroke-width="11" stroke-linecap="round"/>
          <path d="M202 42 Q194 29 208 18 Q225 10 236 25 Q241 44 220 50Z" fill="url(#fishhammer)"/>
          <ellipse cx="213" cy="29" rx="9" ry="7" fill="#f7deb7" opacity=".55"/>
        </svg>
      </button>
      <div class="fish-actions"><button :disabled="locked" :aria-label="sound?'静音':'开启音效'" @click="sound=!sound"><Volume2 v-if="sound" :size="16"/><VolumeX v-else :size="16"/></button><span>心平气和 · 万事顺遂</span><button :disabled="locked" aria-label="清空计数" title="清空计数" @click="resetCount">归零</button></div>
    </template>
    <p v-if="localError" class="living-error" role="alert">{{ localError }}</p>
  </div>
</template>

<style scoped>
.living-content{height:100%;position:relative;font-family:-apple-system,BlinkMacSystemFont,'PingFang SC','Microsoft YaHei UI',sans-serif;box-sizing:border-box;--ink:#20242b;--muted:#858a94;color:var(--ink)}
.living-content :is(button,input){font:inherit}.living-content button{cursor:pointer}.living-content button:disabled{cursor:default;opacity:.42}.living-content button:focus-visible,.living-content input:focus-visible{outline:2px solid #8bacc9;outline-offset:3px}
.living-overline{position:absolute;left:0;top:0;color:var(--muted);font-size:13px;margin:0}
.clock-digits{position:absolute;top:39px;left:-3px;font-size:79px;letter-spacing:-5px;font-weight:530;line-height:1.05;font-variant-numeric:tabular-nums;white-space:nowrap}
.clock-caption{position:absolute;top:136px;left:0;margin:0;font-size:13px;letter-spacing:5px;color:#707c88}
.clock-landscape{position:absolute;left:-18px;right:-18px;bottom:-18px;width:calc(100% + 36px);height:142px}
.weather{color:white;--ink:white;--muted:#e3f2ff}.weather-degree{position:absolute;top:0;left:0;display:flex;align-items:start}.weather-degree strong{font-size:94px;line-height:1;letter-spacing:-6px;font-weight:450}.weather-degree span{font-size:46px;font-weight:400;padding-top:3px}.weather-condition{position:absolute;top:99px;font-size:17px;opacity:.96}.weather-illustration{position:absolute;right:-18px;top:38px;width:215px;height:155px}.weather-illustration svg{width:100%;height:100%}.weather-days{position:absolute;bottom:61px;left:0;right:0;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:3px;z-index:1}.weather-days>div{background:#ffffff27;border:1px solid #ffffff26;border-radius:12px;text-align:center;padding:7px 0;display:grid;gap:2px}.weather-days span{font-size:11px}.weather-days b{font-size:18px}.weather-days small{font-size:11px;white-space:nowrap}.forecast-unavailable{position:absolute;bottom:84px;font-size:11px;opacity:.8}.weather-foot{position:absolute;bottom:5px;left:0;right:0;display:flex;align-items:center;justify-content:space-between;border-top:1px solid #ffffff50;padding-top:12px}.weather-foot div{display:grid;gap:4px;font-size:12px}.weather-foot small{font-size:12px;opacity:.8}.weather-foot button{border:0;border-radius:50%;width:30px;height:30px;display:grid;place-items:center;background:#ffffff33;color:#fff}
.living-topline{display:flex;align-items:center;gap:10px;height:28px}.living-topline strong{font-size:22px;font-weight:650}.living-topline>span{margin-left:auto;font-size:12px;color:var(--muted)}.plus-action{border:0;background:#f5ece7;border-radius:50%;width:34px;height:34px;display:grid;place-items:center;color:#694d3a}
.task-stack{margin-top:20px;display:grid;gap:0}.task-row{height:42px;border-bottom:1px solid #20242b0c;display:flex;align-items:center;gap:12px;font-size:14px}.task-row>span{flex:1;text-overflow:ellipsis;overflow:hidden;white-space:nowrap}.task-row .done,.reminder-row .done{text-decoration:line-through;color:var(--muted)}.task-check{height:23px;width:23px;border:1.5px solid #c5c4c2;background:none;border-radius:50%;display:grid;place-items:center;color:white}.task-check.checked{background:#60cba5;border-color:#60cba5}.remove-mini{opacity:0;background:none;border:0;color:#a8a2a0;padding:5px}.task-row:hover .remove-mini,.reminder-row:hover .remove-mini,.remove-mini:focus-visible{opacity:1}.extra-line{padding-top:7px;color:var(--muted);font-size:12px}.inline-form{position:absolute;bottom:0;left:0;right:0;display:flex;background:#ffffffee;padding:8px;border-radius:12px;gap:8px;box-shadow:0 5px 20px #0002}.inline-form input{width:100%;min-width:0;border:0;background:transparent;padding:6px}.inline-form button{background:#20242b;color:white;border:0;border-radius:8px;padding:5px 10px;white-space:nowrap}.living-bottom-text{position:absolute;bottom:0;left:0;right:0;font-size:12px;opacity:.57;margin:0}.living-empty{padding-top:44px;font-size:16px;color:#8d8e94;display:grid;text-align:center;gap:8px}.living-empty small{font-size:12px}
.reminder-stack{position:relative;margin-top:24px;display:grid;gap:4px}.reminder-stack:before{content:'';position:absolute;top:14px;bottom:14px;left:10px;width:1px;background:#d4d7df}.reminder-row{position:relative;display:flex;align-items:center;gap:11px;min-height:43px;font-size:13px}.reminder-row time{font-size:12px;color:#747785;min-width:75px}.reminder-name{flex:1;min-width:0;text-align:left;background:none;border:0;color:inherit;overflow:hidden;white-space:nowrap;text-overflow:ellipsis;padding:4px 0}.reminder-dot{width:21px;height:21px;border-radius:50%;border:5px solid #f4f0ee;background:#f27472;z-index:1;flex-shrink:0}.reminder-dot.color-1{background:#6aace5}.reminder-dot.color-2{background:#c49bd8}.reminder-dot.color-3{background:#edb542}.reminder-dot.done{background:#b9b9bd}.reminders-empty{padding-top:55px}.reminder-editor{position:absolute;inset:5px -6px 2px;background:#fff9f5;z-index:2;border-radius:12px;display:grid;align-content:center;gap:10px;padding:13px;box-shadow:0 5px 20px #0002}.reminder-editor input{box-sizing:border-box;width:100%;border:1px solid #ddd5cd;border-radius:8px;padding:8px;font-size:13px}.reminder-editor>div{display:flex;justify-content:flex-end;gap:12px}.reminder-editor button{border:0;background:none;font-size:13px}.reminder-editor button[type=submit]{background:#20242b;color:#fff;padding:7px 13px;border-radius:8px}
.stocks{color:#f4f7f8;--ink:#f4f7f8;--muted:#a7b2b9}.stock-header{display:flex;justify-content:space-between;align-items:flex-start}.stock-header>div:first-child{display:grid;gap:4px}.stock-header strong{font-size:25px;font-weight:700}.stock-header small{font-size:12px;color:#a4b2b9}.stock-value{text-align:right;display:grid;gap:3px}.stock-value b{font-size:27px;font-weight:600;font-variant-numeric:tabular-nums}.stock-value span{font-size:15px}.rising{color:#ff6f68}.falling{color:#4ec59d}.stock-graph{height:103px;margin-top:17px;position:relative;color:#99aeb5}.stock-graph svg{width:100%;height:100%}.stock-graph p{position:absolute;inset:0;display:grid;place-items:center;color:#b2b6bc;font-size:13px;background:#1c222770}.stock-periods{display:flex;justify-content:space-between;align-items:center;margin-top:4px;color:#929da5;font-size:12px}.stock-periods :is(button,span){padding:4px 9px;border-radius:14px;background:none;border:0;color:inherit}.stock-periods .selected{color:#fff;border:1px solid #73817e;background:#2b393b}.stock-periods span{opacity:.42}.stock-periods .stock-reload{padding:4px}.stock-tickers{display:flex;gap:8px;margin-top:11px}.stock-tickers button{min-width:0;flex:1;padding:9px 2px;background:#283235;color:#c4d0d3;border:1px solid #384349;border-radius:9px;font-size:11px}.stock-tickers button.selected{border-color:#77978e;color:white;background:#31433f}.stock-source{position:absolute;bottom:-9px;right:0;font-size:10px;color:#a6b1b6}
.woodfish{text-align:left;color:#4d3423}.woodfish-count{font-size:66px;letter-spacing:-3px;font-weight:480;line-height:1}.woodfish-caption{font-size:12px;margin:5px 0 0;color:#705d4d}.fish-target{position:absolute;top:65px;left:0;width:100%;height:165px;border:0;background:none;display:flex;justify-content:center;align-items:center;padding:0;transition:transform .12s ease}.fish-target svg{max-width:310px;height:100%;filter:drop-shadow(0 9px 14px #a57c5a30)}.fish-target.hit{transform:translateY(8px) scale(.975)}.fish-actions{position:absolute;bottom:0;left:0;right:0;display:flex;align-items:center;justify-content:space-between;font-size:12px;color:#745c4e}.fish-actions button{background:none;border:0;color:#745c4e;display:flex;align-items:center;font-size:12px;padding:5px}
.living-error{position:absolute;bottom:20px;left:0;right:0;border-radius:8px;background:#ffe9e2;color:#9d2821;padding:7px;font-size:11px;max-height:40px;overflow:auto}
@keyframes spin{to{transform:rotate(360deg)}}.spinning{animation:spin 1s linear infinite}@media(prefers-reduced-motion:reduce){.fish-target,.spinning{animation:none;transition:none}}
</style>
