<script setup lang="ts">
import {computed,ref,watch} from 'vue';
import {ArrowLeft,ChevronRight,Check,X,Trash2,ListTodo,CalendarDays,MapPin,ChartNoAxesCombined,Heart,Volume2,Monitor,Eye,SlidersHorizontal,Layers,Palette} from 'lucide-vue-next';
import type {City,DesktopWidget,WidgetContent,WidgetMonitor,WidgetTheme} from './model.ts';
import {widgetKinds} from './model.ts';
import {searchWidgetCities} from './client.ts';
import ItemManager from './ItemManager.vue';
import CountdownEditor from './CountdownEditor.vue';
import StockWatchlistEditor from './StockWatchlistEditor.vue';

const props=defineProps<{widget:DesktopWidget;monitors?:WidgetMonitor[];update:(patch:Partial<DesktopWidget>)=>Promise<unknown>;remove:()=>Promise<unknown>;searchCities?:(query:string)=>Promise<City[]>}>();
const emit=defineEmits<{close:[]}>();
const panel=ref(false),confirmRemove=ref(false),busy=ref(false),error=ref(''),query=ref(''),cities=ref<City[]>([]),searching=ref(false);
const draftScale=ref(props.widget.scale);
watch(()=>props.widget.scale,v=>{draftScale.value=v;});

const title=computed(()=>props.widget.kind==='custom'?(props.widget.custom?.name||'自定义组件'):(widgetKinds.find(k=>k.kind===props.widget.kind)?.title||'组件设置'));
const living=computed(()=>props.widget.living??{symbol:'1.000001',count:0,reminders:[]});
const editor=computed(()=>({notes:{text:'管理待办',icon:ListTodo,detail:`${props.widget.note?.items.length??0} 项`},reminders:{text:'全部日程',icon:CalendarDays,detail:`${living.value.reminders.length} 项`},weather:{text:'天气城市',icon:MapPin,detail:props.widget.city?.name||'选择城市'},stocks:{text:'管理自选',icon:ChartNoAxesCombined,detail:`${living.value.symbols?.length||1} 只`},countdown:{text:'日期设置',icon:Heart,detail:props.widget.countdown?.title||'纪念日'}}[props.widget.kind as 'notes'|'reminders'|'weather'|'stocks'|'countdown']));

const scalePresets=[{label:'小 65%',value:0.65},{label:'推荐 85%',value:0.85},{label:'大 115%',value:1.15}];
const materials:Array<{value:WidgetTheme;label:string}>=[{value:'ivory',label:'暖白纸质'},{value:'dark',label:'深色炭灰'},{value:'glass',label:'柔雾玻璃'}];

async function apply(patch:Partial<DesktopWidget>){busy.value=true;error.value='';try{await props.update(patch);}catch(e){error.value=String(e);throw e;}finally{busy.value=false;}}
function update(patch:Partial<DesktopWidget>){void apply(patch).catch(()=>{});}
async function save(content:WidgetContent){await apply(content);}
async function remove(){busy.value=true;error.value='';try{await props.remove();emit('close');}catch(e){error.value=String(e);}finally{busy.value=false;}}
async function search(){if(searching.value||!query.value.trim())return;searching.value=true;error.value='';cities.value=[];try{cities.value=await (props.searchCities??searchWidgetCities)(query.value.trim());if(!cities.value.length)error.value='没有找到城市，试试完整名称';}catch(e){error.value=String(e);}finally{searching.value=false;}}
</script>
<template>
<section class="widget-settings-menu" role="dialog" :aria-label="`${title}设置`" @keydown.esc.stop.prevent="emit('close')">
 <header class="menu-title"><button v-if="panel" class="icon-button" aria-label="返回组件选项" @click="panel=false"><ArrowLeft :size="16"/></button><strong>{{ panel?editor?.text:title }}</strong><button class="icon-button menu-close" aria-label="关闭组件设置" @click="emit('close')"><X :size="16"/></button></header>
 <div class="menu-content">
 <template v-if="!panel">
  <div class="menu-row menu-scale"><SlidersHorizontal :size="15"/><label for="widget-scale">大小</label><output>{{ Math.round(draftScale*100) }}%</output></div>
  <div class="menu-size-control"><span>50%</span><input id="widget-scale" type="range" aria-label="组件大小" :value="draftScale" min="0.5" max="1.5" step="0.01" :disabled="busy" @input="draftScale=Number(($event.target as HTMLInputElement).value)" @change="update({scale:Number(($event.target as HTMLInputElement).value)})"/><span>150%</span></div>
  <div class="menu-presets" role="group" aria-label="快捷大小">
   <button v-for="p in scalePresets" :key="p.value" type="button" :class="{'is-on':Math.abs(widget.scale-p.value)<0.02}" :disabled="busy" @click="draftScale=p.value;update({scale:p.value})">{{ p.label }}</button>
  </div>
  <template v-if="widget.kind!=='custom'">
   <div class="menu-row"><Palette :size="15"/><span>卡片材质</span></div>
   <div class="menu-presets" role="group" aria-label="卡片材质">
    <button v-for="m in materials" :key="m.value" type="button" :aria-pressed="(widget.theme||'ivory')===m.value" :class="{'is-on':(widget.theme||'ivory')===m.value}" :disabled="busy" :data-testid="`widget-material-${m.value}`" @click="update({theme:m.value})">{{ m.label }}</button>
   </div>
  </template>
  <div class="menu-separator"/>
  <button v-if="editor" class="menu-row menu-action" :disabled="busy" @click="panel=true"><component :is="editor.icon" :size="15"/><span>{{ editor.text }}</span><small>{{ editor.detail }}</small><ChevronRight :size="14"/></button>
  <button v-if="widget.kind==='woodfish'" class="menu-row menu-action" :disabled="busy" :aria-pressed="living.sound??true" @click="update({living:{...living,sound:!(living.sound??true)}})"><Volume2 :size="15"/><span>敲击声音</span><small>{{ living.sound??true?'开启':'关闭' }}</small><Check v-if="living.sound??true" :size="15" class="accent-check"/></button>
  <label v-if="(monitors?.length??0)>1" class="menu-row"><Monitor :size="15"/><span>显示器</span><select :value="widget.monitor??monitors?.find(m=>m.primary)?.id" aria-label="组件显示器" :disabled="busy" @change="update({monitor:($event.target as HTMLSelectElement).value})"><option v-for="m in monitors" :key="m.id" :value="m.id">{{ m.name }}</option></select></label>
  <label class="menu-row menu-layer"><Layers :size="15"/><span>显示层级</span><select :value="widget.layer??'desktop'" aria-label="组件显示层级" :disabled="busy" @change="update({layer:($event.target as HTMLSelectElement).value as 'desktop'|'topmost'})"><option value="desktop">贴在桌面</option><option value="topmost">始终置顶</option></select></label>
  <button class="menu-row menu-action" :disabled="busy" :aria-pressed="widget.enabled" @click="update({enabled:!widget.enabled})"><Eye :size="15"/><span>显示在桌面</span><Check v-if="widget.enabled" :size="15" class="accent-check"/></button>
  <div class="menu-separator"/>
  <div v-if="confirmRemove" class="menu-remove-confirm"><span>移除组件及其内容？</span><button :disabled="busy" @click="confirmRemove=false">取消</button><button class="danger" :disabled="busy" @click="remove">移除</button></div>
  <button v-else class="menu-row menu-action menu-remove" :disabled="busy" @click="confirmRemove=true"><Trash2 :size="15"/><span>移除组件</span></button>
 </template>
 <template v-else>
  <ItemManager v-if="widget.kind==='notes'||widget.kind==='reminders'" :widget="widget" :save="save"/>
  <CountdownEditor v-else-if="widget.kind==='countdown'" :data="widget.countdown" :save="save"/>
  <StockWatchlistEditor v-else-if="widget.kind==='stocks'" :living="widget.living" :save="save"/>
  <section v-else-if="widget.kind==='weather'" class="menu-city"><form @submit.prevent="search"><input v-model="query" aria-label="城市名称" placeholder="搜索城市"/><button :disabled="searching||busy">{{ searching?'查询中…':'搜索' }}</button></form><p v-if="widget.city">当前：{{ widget.city.name }}</p><button v-for="(city,index) in cities" :key="index" class="menu-city-result" :disabled="busy" @click="update({city})"><MapPin :size="14"/><span>{{ city.name }}<small>{{ [city.admin1,city.country].filter(Boolean).join(' · ') }}</small></span><Check v-if="widget.city?.latitude===city.latitude&&widget.city?.longitude===city.longitude" :size="14" class="accent-check"/></button></section>
 </template>
 <p v-if="error" class="menu-error" role="alert">{{ error }}</p>
 </div>
</section>
</template>
<style scoped>
.widget-settings-menu{
  --menu-fg:var(--ui-text-primary,#1d1d1f);
  --menu-muted:var(--ui-text-tertiary,#8e8e93);
  --menu-line:var(--ui-line,rgba(0,0,0,.07));
  --menu-ctrl:var(--ui-control,#f2f2f4);
  --menu-accent:var(--ui-btn,#a8640c);
  --menu-soft:var(--ui-soft,#faeedb);
  --menu-ink:var(--ui-ink,#87500a);
  --menu-danger:var(--ui-danger,#d93a44);
  color:var(--menu-fg);
  background:var(--ui-surface-raised,rgba(255,255,255,.97));
  backdrop-filter:blur(20px);
  border:1px solid var(--menu-line);
  border-radius:16px;
  box-shadow:var(--ui-shadow-menu,0 10px 30px rgba(0,0,0,.16));
  font:13px/1.5 -apple-system,BlinkMacSystemFont,'SF Pro Text','PingFang SC','Segoe UI',sans-serif;
  width:100%;box-sizing:border-box;overflow:hidden;
}
.menu-title{height:44px;display:flex;align-items:center;gap:6px;padding:0 10px 0 16px;border-bottom:1px solid var(--menu-line)}
.menu-title strong{font-weight:600;font-size:13.5px}
.icon-button{display:grid;place-items:center;width:27px;height:27px;border:0;background:none;color:var(--menu-muted);border-radius:7px;cursor:pointer}
.menu-close{margin-left:auto}
.menu-content{padding:6px 8px 8px;max-height:calc(var(--widget-menu-max-height,560px) - 68px);overflow:auto;scrollbar-width:thin}
.menu-row{min-height:34px;padding:6px 10px;display:flex;gap:10px;align-items:center;border:0;background:transparent;color:inherit;width:100%;font:inherit;text-align:left;box-sizing:border-box;border-radius:8px}
.menu-row>svg{color:var(--menu-muted);flex:none}
.menu-row>span,.menu-row>label{flex:1}
.menu-row>small,.menu-scale output{margin-left:auto;font-size:11.5px;color:var(--menu-muted);max-width:125px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-variant-numeric:tabular-nums}
.menu-scale output{color:var(--menu-fg);font-weight:600}
.menu-row>select{max-width:170px;min-width:0;border:0;background:transparent;color:inherit;font:inherit;cursor:pointer}
.menu-action{cursor:pointer}
.menu-action:hover,.icon-button:hover{background:var(--ui-surface-hover,rgba(0,0,0,.045))}
.menu-action:disabled{opacity:.45;cursor:default}
.accent-check{color:var(--menu-accent)!important}
.menu-size-control{display:flex;align-items:center;gap:9px;padding:0 10px 6px 34px;color:var(--menu-muted);font-size:10.5px}
.menu-size-control input{flex:1;min-width:0;accent-color:var(--menu-accent);height:14px;cursor:ew-resize}
.menu-presets{display:grid;grid-template-columns:repeat(3,1fr);gap:6px;padding:2px 10px 8px}
.menu-presets button{height:27px;border:0;border-radius:8px;background:var(--menu-ctrl);color:var(--ui-text-secondary,#6e6e73);font:inherit;font-size:11.5px;font-weight:500;cursor:pointer;transition:background .14s,color .14s}
.menu-presets button:hover:not(:disabled){background:var(--ui-control-hover,#eaeaed);color:var(--menu-fg)}
.menu-presets button.is-on{background:var(--menu-soft);color:var(--menu-ink);font-weight:600}
.menu-separator{height:1px;background:var(--menu-line);margin:5px 8px}
.menu-remove{color:var(--menu-danger)}
.menu-remove>svg{color:var(--menu-danger)}
.menu-remove-confirm{display:flex;align-items:center;gap:9px;font-size:12px;padding:8px 10px}
.menu-remove-confirm>span{flex:1}
.menu-remove-confirm button{border:0;background:none;font:inherit;color:inherit;cursor:pointer;padding:4px 8px;border-radius:6px}
.menu-remove-confirm button:hover{background:var(--ui-surface-hover,rgba(0,0,0,.045))}
.menu-remove-confirm .danger{color:var(--menu-danger);font-weight:600}
.menu-error{font-size:11.5px;line-height:1.5;color:var(--menu-danger);padding:6px 10px;overflow-wrap:anywhere;margin:0}
.menu-city{padding:8px}
.menu-city form{display:flex;gap:7px}
.menu-city input{flex:1;min-width:0;border:0;border-radius:8px;background:var(--menu-ctrl);color:inherit;padding:7px 10px;font:inherit}
.menu-city form button{border:0;background:var(--menu-accent);border-radius:8px;color:#fff;padding:6px 12px;font:inherit;font-weight:500;cursor:pointer}
.menu-city p{font-size:11.5px;color:var(--menu-muted);margin:8px 0}
.menu-city-result{display:flex;gap:9px;align-items:center;width:100%;border:0;background:none;color:inherit;font:inherit;text-align:left;padding:8px 6px;border-radius:8px;cursor:pointer}
.menu-city-result:hover{background:var(--ui-surface-hover,rgba(0,0,0,.045))}
.menu-city-result>span{flex:1;display:grid}
.menu-city-result small{color:var(--menu-muted);font-size:10.5px}
.menu-content :deep(.item-manager),.menu-content :deep(.watchlist-editor),.menu-content :deep(.countdown-editor){padding:8px;margin:0}
.menu-content :deep(.manager-list){max-height:300px;overflow:auto}
.menu-content :deep(.countdown-editor fieldset){gap:10px}
.menu-content :deep(.countdown-editor label){gap:4px}
.menu-content :deep(.xb-item-form fieldset){gap:8px}
.menu-content :deep(.manager-help){font-size:11px}
.menu-content :deep(button:focus-visible),.icon-button:focus-visible{outline:2px solid var(--menu-accent);outline-offset:-2px}
</style>
