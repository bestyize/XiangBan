<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Plus,Check,Search,Trash2,ArrowLeft} from 'lucide-vue-next';
import {NAlert,NButton,NInput,NModal,NPopconfirm,NSelect,NSlider,NSwitch} from 'naive-ui';
import type {ControlStore} from '../control-types.ts';
import {createWidgetClient,searchWidgetCities} from '../../widgets/client.ts';
import {baseSize,widgetKinds,validStockSymbol,type WidgetKind,type DesktopWidget,type City,type WidgetContent} from '../../widgets/model.ts';
import {emptyCountdown,emptyNote} from '../../widgets/content.ts';
import {dateKey} from '../../widgets/calendar.ts';
import WidgetSurface from '../../widgets/WidgetSurface.vue';
import ItemManager from '../../widgets/ItemManager.vue';
import StockWatchlistEditor from '../../widgets/StockWatchlistEditor.vue';
import CountdownEditor from '../../widgets/CountdownEditor.vue';
const props=defineProps<{store:ControlStore;settingsRequest?:{id:string}|null}>();
const emit=defineEmits<{settingsOpened:[]}>();
const client=createWidgetClient(),{layout,ready,busy,error}=client;
const selected=ref(''),settingsOpen=ref(false),showAdded=ref(false),stockSymbol=ref(''),cityQuery=ref(''),cities=ref<City[]>([]),cityBusy=ref(false),flash=ref('');
const chosen=computed(()=>layout.value.widgets.find(w=>w.id===selected.value));
const titleOf=(w:DesktopWidget)=>widgetKinds.find(k=>k.kind===w.kind)?.title||'桌面组件';
const monitors=computed(()=>layout.value.monitors??[]),monitorOptions=computed(()=>monitors.value.map(m=>({label:`${m.name}${m.primary?' · 主屏幕':''}`,value:m.id})));
const cityOptions=computed(()=>cities.value.map((c,i)=>({label:[c.name,c.admin1,c.country].filter(Boolean).join(' · '),value:i})));
const board=ref<HTMLElement>(),boardScale=ref(.75);let observer:ResizeObserver|undefined,flashTimer:ReturnType<typeof setTimeout>;
const positions:Record<WidgetKind,{left:number;top:number}>= {clock:{left:0,top:0},weather:{left:615,top:0},notes:{left:961,top:0},reminders:{left:0,top:328},stocks:{left:501,top:328},woodfish:{left:925,top:328},calendar:{left:0,top:681},countdown:{left:522,top:681}};
const demo=(kind:WidgetKind):DesktopWidget=>({id:`demo${kind}`,kind,theme:'ivory',scale:.5,x:0,y:0,enabled:true,city:null,...(kind==='notes'?{note:{...emptyNote(),items:[{id:'task1',text:'完成产品周报',done:true,time:'09:00'},{id:'task2',text:'准备下周的设计评审',done:false,time:'14:00'},{id:'task3',text:'阅读一本书',done:false,time:'20:00'}]}}:kind==='reminders'?{living:{symbol:'1.000001',count:0,reminders:[{id:'demo1',title:'产品设计方案评审',at:dateKey(new Date())+'T10:00',end:'11:30',location:'会议室 A302',done:false},{id:'demo2',title:'与开发同学同步进度',at:dateKey(new Date())+'T14:00',end:'15:00',done:false},{id:'demo3',title:'健身',at:dateKey(new Date())+'T19:00',end:'20:00',done:false}]}}:kind==='woodfish'?{living:{count:108,symbol:'1.000001',reminders:[]}}:kind==='stocks'?{living:{count:0,symbol:'116.00700',reminders:[]}}:kind==='countdown'?{countdown:{...emptyCountdown(),title:'我们的纪念日',date:dateKey(new Date(Date.now()+79*86400000))}}:{})});
async function action(fn:()=>Promise<unknown>){try{await fn();}catch(e){props.store.notify(e,'error');}}
async function add(kind:WidgetKind){await action(async()=>{await client.add(kind);flash.value=kind;clearTimeout(flashTimer);flashTimer=setTimeout(()=>flash.value='',1600);props.store.notify('已添加到桌面，按住卡片顶部可拖动','success');});}
function openSettings(id:string){selected.value=id;cityQuery.value='';cities.value=[];settingsOpen.value=true;}
function update(patch:Partial<DesktopWidget>){if(chosen.value)void action(()=>client.update(chosen.value!.id,patch));}
async function saveContent(content:WidgetContent){if(chosen.value)await client.update(chosen.value.id,content);}
async function remove(){if(chosen.value)await action(async()=>{await client.remove(chosen.value!.id);settingsOpen.value=false;});}
watch(chosen,w=>stockSymbol.value=w?.living?.symbol??'116.00700',{immediate:true});
function saveStock(){const symbol=stockSymbol.value.trim();if(!validStockSymbol(symbol)){props.store.notify('例如：116.00700（腾讯）、1.600519（沪市）、0.000001（深市）','error');return;}update({living:{reminders:[],count:0,symbol}});}
async function search(){if(!cityQuery.value.trim()||cityBusy.value)return;cityBusy.value=true;try{cities.value=await searchWidgetCities(cityQuery.value.trim());if(!cities.value.length)props.store.notify('没有找到城市，请试试完整名称','info');}catch(e){props.store.notify(e,'error');}finally{cityBusy.value=false;}}
watch([()=>props.settingsRequest,ready],([request,loaded])=>{if(!request||!loaded)return;if(layout.value.widgets.some(w=>w.id===request.id)){showAdded.value=true;openSettings(request.id);}else props.store.notify('这个组件已不在桌面上','info');emit('settingsOpened');},{immediate:true});
function fit(){if(board.value)boardScale.value=Math.max(.1,Math.min(1,board.value.clientWidth/1314));}
onMounted(()=>{void client.load();observer=new ResizeObserver(fit);if(board.value)observer.observe(board.value);fit();});
onBeforeUnmount(()=>{observer?.disconnect();clearTimeout(flashTimer);client.dispose();});
</script>
<template>
<section class="ui-page xb-widgets-page" data-testid="widgets-page">
 <header class="xb-page-header"><div><h1>桌面组件</h1><p>让日常的小事，在桌面上好好安放。</p></div><button class="added-link" :class="{on:showAdded}" @click="showAdded=!showAdded"><ArrowLeft v-if="showAdded" :size="16"/>{{ showAdded?'返回组件库':'已添加' }}<span>{{ layout.widgets.length }} / 12</span></button></header>
 <NAlert v-if="error" type="error" :show-icon="false">{{ error }}<NButton v-if="!ready" text @click="client.load">重新读取</NButton></NAlert>
 <NAlert v-if="layout.errors.length" type="warning" :show-icon="false">{{ layout.errors.join('；') }}</NAlert>
 <template v-if="!showAdded">
  <div ref="board" class="xb-widget-board-wrap" :style="{height:`${967*boardScale}px`}"><div class="xb-widget-board" :style="{transform:`scale(${boardScale})`}">
   <div v-for="item in widgetKinds" :key="item.kind" class="xb-widget-slot" :class="item.kind" :style="{...positions[item.kind],left:`${positions[item.kind].left}px`,top:`${positions[item.kind].top}px`,width:`${baseSize(item.kind).width}px`,height:`${baseSize(item.kind).height}px`}"><WidgetSurface :widget="demo(item.kind)" preview demo @settings="add(item.kind)"/><button class="add-to-desktop" :class="{added:flash===item.kind}" :disabled="!ready||busy||layout.widgets.length>=12" :data-testid="`add-${item.kind}`" :aria-label="`添加${item.title}到桌面`" @click="add(item.kind)"><Check v-if="flash===item.kind" :size="18"/><Plus v-else :size="18"/>{{ flash===item.kind?'已添加':'添加到桌面' }}</button></div>
  </div></div><p class="xb-page-hint">悬停卡片即可添加 · 展示内容为示例，添加后使用真实时间、天气和行情。</p>
 </template>
 <template v-else>
  <div v-if="!layout.widgets.length" class="xb-empty"><span>还没有桌面组件</span><p>先挑选一个喜欢的，让桌面多一点生活。</p><button @click="showAdded=false">去组件库看看</button></div>
  <div v-else class="xb-added-grid" data-testid="desktop-widgets"><button v-for="w in layout.widgets" :key="w.id" class="xb-added-card desktop-card" :class="{off:!w.enabled}" :aria-label="`调整${titleOf(w)}`" @click="openSettings(w.id)"><div class="xb-added-preview"><div :style="{transform:`scale(${Math.min(280/baseSize(w.kind).width,180/baseSize(w.kind).height)})`}"><WidgetSurface :widget="w" preview/></div></div><strong>{{ titleOf(w) }}</strong><small>{{ w.enabled?'已显示在桌面':'已隐藏' }} · 点击设置</small></button></div>
  <p class="xb-page-hint">按住卡片顶部拖动位置，点击卡片右上角查看设置。数据保存在本机，重新启动后恢复。</p>
 </template>
 <NModal :show="settingsOpen" preset="card" :title="chosen?titleOf(chosen):'组件设置'" style="max-width:560px;width:92vw;border-radius:24px" content-style="max-height:72vh;overflow:auto" data-testid="widget-settings" @update:show="settingsOpen=$event">
  <div v-if="chosen" class="xb-widget-settings">
   <div class="xb-settings-preview"><div :style="{transform:`scale(${Math.min(470/baseSize(chosen.kind).width,220/baseSize(chosen.kind).height)*chosen.scale/1.5})`}"><WidgetSurface :key="chosen.id" :widget="chosen" preview :save="saveContent"/></div></div>
   <label class="xb-setting-row"><span>显示在桌面</span><NSwitch :value="chosen.enabled" :disabled="busy" @update:value="update({enabled:$event})"/></label>
   <label class="xb-setting-field"><span>大小 <small>× {{ chosen.scale.toFixed(2) }}</small></span><NSlider :value="chosen.scale" :min=".5" :max="1.5" :step=".05" :disabled="busy" @update:value="update({scale:$event})"/></label>
   <label v-if="monitors.length>1" class="xb-setting-field"><span>所在显示器</span><NSelect :value="chosen.monitor??monitors.find(m=>m.primary)?.id" :options="monitorOptions" :disabled="busy" @update:value="update({monitor:$event})"/></label>
   <ItemManager v-if="chosen.kind==='notes'||chosen.kind==='reminders'" :key="chosen.id" :widget="chosen" :save="saveContent"/>
   <CountdownEditor v-else-if="chosen.kind==='countdown'" :key="chosen.id" :data="chosen.countdown" :save="saveContent"/>
   <div v-else-if="chosen.kind==='weather'" class="xb-setting-field"><span>天气城市</span><small v-if="chosen.city">当前：{{ chosen.city.name }}</small><div class="xb-city-search"><NInput v-model:value="cityQuery" placeholder="城市名称" @keydown.enter="search"/><NButton :loading="cityBusy" aria-label="搜索城市" @click="search"><Search :size="16"/></NButton></div><NSelect v-if="cities.length" :options="cityOptions" placeholder="选择城市" @update:value="i=>update({city:cities[i]})"/></div>
   <StockWatchlistEditor v-else-if="chosen.kind==='stocks'" :key="chosen.id" :living="chosen.living" :save="saveContent"/>
   <label v-else-if="chosen.kind==='woodfish'" class="xb-setting-row"><span>木鱼声音</span><NSwitch :value="chosen.living?.sound??true" :disabled="busy" @update:value="update({living:{...chosen.living!,sound:$event}})"/></label>
   <p v-else class="xb-settings-help">显示本机日期和时间。</p>
   <div class="xb-settings-footer"><NPopconfirm positive-text="移除" negative-text="取消" @positive-click="remove"><template #trigger><NButton quaternary :disabled="busy" aria-label="移除组件"><Trash2 :size="16"/>移除组件</NButton></template>移除这个组件及其本地内容？</NPopconfirm><NButton data-testid="widget-settings-done" @click="settingsOpen=false">完成</NButton></div>
  </div>
 </NModal>
</section>
</template>
<style scoped>
.xb-widgets-page{max-width:1430px;margin:0 auto;padding:28px 30px 34px!important;background:var(--ui-app-bg);border-radius:20px;display:flex;flex-direction:column;gap:22px;overflow-y:auto;min-height:0;height:100%;width:100%;flex:1;color:var(--ui-text-primary);box-sizing:border-box}.xb-page-header{display:flex;justify-content:space-between;align-items:center;gap:20px}.xb-page-header h1{font-size:25px;letter-spacing:.5px;margin:0;font-weight:600}.xb-page-header p{font-size:13px;color:#9b8c7b;margin:8px 0 0}.added-link{display:flex;gap:10px;align-items:center;border:1px solid #cfc2b0;background:#f7f3ec88;border-radius:12px;padding:10px 15px;font-size:13px;color:#7c6853;cursor:pointer}.added-link>span{font-variant-numeric:tabular-nums;font-size:11px;color:#a4927e}.xb-widget-board-wrap{position:relative;width:100%;flex:none}.xb-widget-board{position:absolute;top:0;left:0;width:1314px;height:967px;transform-origin:top left}.xb-widget-slot{position:absolute}.add-to-desktop{position:absolute;right:17px;bottom:13px;z-index:22;display:flex;align-items:center;gap:7px;border:1px solid #c4b09b;background:#f8f2e9f5;box-shadow:0 4px 15px #5f4a3425;color:#806347;border-radius:22px;padding:10px 16px;font:15px -apple-system,'PingFang SC',sans-serif;opacity:0;transform:translateY(5px);transition:.18s;cursor:pointer}.xb-widget-slot:hover .add-to-desktop,.xb-widget-slot:focus-within .add-to-desktop,.add-to-desktop.added{opacity:1;transform:none}.add-to-desktop:disabled{cursor:default;opacity:.4}.xb-page-hint{font-size:12px;color:#a4937e;line-height:1.8;text-align:center;margin:0}.xb-added-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(290px,1fr));gap:20px}.xb-added-card{border:0;border-radius:22px;background:#f7f3eb;box-shadow:0 5px 18px #7e6a4620;padding:18px;display:flex;flex-direction:column;align-items:flex-start;cursor:pointer;color:inherit}.xb-added-card.off{opacity:.5}.xb-added-preview{height:195px;width:100%;display:grid;place-items:center;position:relative;pointer-events:none}.xb-added-preview>div{position:absolute;left:50%;top:0;transform-origin:top left;translate:-140px 0}.xb-added-card strong{font-size:16px;font-weight:500;margin-top:10px}.xb-added-card>small{font-size:12px;color:#a18e77;margin-top:7px}.xb-empty{padding:140px 20px;text-align:center}.xb-empty>span{font-size:23px}.xb-empty p{font-size:14px;color:#a18c73;margin:14px}.xb-empty button{border:1px solid #c8b39b;border-radius:12px;background:#f9f4ec;padding:10px 15px;color:#9a7958;cursor:pointer}.xb-widget-settings{color:var(--ui-text-primary);display:grid;gap:20px}.xb-settings-preview{position:relative;width:100%;height:235px;background:var(--ui-app-bg);border-radius:18px;display:flex;justify-content:center;align-items:center;overflow:hidden}.xb-settings-preview>div{transform-origin:center;position:absolute}.xb-setting-row{display:flex;justify-content:space-between;align-items:center;font-size:14px}.xb-setting-field{display:grid;gap:10px;font-size:14px}.xb-setting-field>span{display:flex;justify-content:space-between}.xb-setting-field small{font-size:12px;color:#a29382}.xb-city-search{display:flex;gap:8px}.xb-settings-footer{display:flex;justify-content:space-between;padding-top:8px;border-top:1px solid #ebe4da}.xb-settings-help{font-size:13px;line-height:1.8;color:#9a8975;margin:0}@media(max-width:760px){.xb-widgets-page{padding:18px 14px!important}.xb-page-header h1{font-size:21px}.xb-page-header p{display:none}.add-to-desktop{opacity:1}.xb-added-grid{grid-template-columns:1fr}}
</style>
