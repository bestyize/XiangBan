<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Check,Layers,Pin,Monitor,Sparkles,Trash2} from 'lucide-vue-next';
import {NAlert,NButton,NModal,NPopconfirm,NTabs,NTabPane} from 'naive-ui';
import type {ControlStore} from '../control-types.ts';
import ResourceShareActions from '../components/ResourceShareActions.vue';
import {createWidgetClient} from '../../widgets/client.ts';
import {baseSize,widgetKinds,type WidgetKind,type DesktopWidget,type CustomWidgetPack} from '../../widgets/model.ts';
import {emptyCountdown,emptyNote} from '../../widgets/content.ts';
import {dateKey} from '../../widgets/calendar.ts';
import WidgetSurface from '../../widgets/WidgetSurface.vue';
import WidgetSettingsMenu from '../../widgets/WidgetSettingsMenu.vue';

const props=defineProps<{store:ControlStore;settingsRequest?:{id:string}|null}>();
const emit=defineEmits<{settingsOpened:[]}>();
const client=createWidgetClient(),{layout,ready,busy,error}=client;
const selected=ref(''),settingsOpen=ref(false),flash=ref('');
const activeCategory=ref<'builtin'|'custom'>('builtin');
const customWidgets=ref<CustomWidgetPack[]>([]);

/** The widget kind or 'custom' whose add-button menu is open. */
const layerFor=ref<WidgetKind|null>(null);
const targetCustomId=ref<string|null>(null);
const targetCustomName=ref<string>('');

const chosen=computed(()=>layout.value.widgets.find(w=>w.id===selected.value));
const monitors=computed(()=>layout.value.monitors??[]);
const board=ref<HTMLElement>(),boardScale=ref(.75);
let observer:ResizeObserver|undefined,flashTimer:ReturnType<typeof setTimeout>;

const positions:Record<Exclude<WidgetKind,'custom'>,{left:number;top:number}>= {clock:{left:0,top:0},weather:{left:615,top:0},notes:{left:961,top:0},reminders:{left:0,top:328},stocks:{left:501,top:328},woodfish:{left:925,top:328},calendar:{left:0,top:681},countdown:{left:522,top:681}};
const demo=(kind:WidgetKind):DesktopWidget=>({id:`demo${kind}`,kind,theme:'ivory',scale:.5,x:0,y:0,enabled:true,city:null,...(kind==='notes'?{note:{...emptyNote(),items:[{id:'task1',text:'完成产品周报',done:true,time:'09:00'},{id:'task2',text:'准备下周的设计评审',done:false,time:'14:00'},{id:'task3',text:'阅读一本书',done:false,time:'20:00'}]}}:kind==='reminders'?{living:{symbol:'1.000001',count:0,reminders:[{id:'demo1',title:'产品设计方案评审',at:dateKey(new Date())+'T10:00',end:'11:30',location:'会议室 A302',done:false},{id:'demo2',title:'与开发同学同步进度',at:dateKey(new Date())+'T14:00',end:'15:00',done:false},{id:'demo3',title:'健身',at:dateKey(new Date())+'T19:00',end:'20:00',done:false}]}}:kind==='woodfish'?{living:{count:108,symbol:'1.000001',reminders:[]}}:kind==='stocks'?{living:{count:0,symbol:'116.00700',reminders:[]}}:kind==='countdown'?{countdown:{...emptyCountdown(),title:'我们的纪念日',date:dateKey(new Date(Date.now()+79*86400000))}}:{})});
const customDemo=(pack:CustomWidgetPack):DesktopWidget=>({id:`demo-${pack.id}`,kind:'custom',theme:pack.theme||'transparent',scale:1,x:0,y:0,enabled:true,city:null,customId:pack.id,custom:pack});

async function action(fn:()=>Promise<unknown>){try{await fn();}catch(e){props.store.notify(e,'error');}}

async function add(kind:WidgetKind,layer:'desktop'|'topmost'){
  const cid=targetCustomId.value;
  layerFor.value=null;
  targetCustomId.value=null;
  await action(async()=>{
    await client.add(kind,layer,cid||undefined);
    flash.value=kind==='custom'?(cid||'custom'):kind;
    clearTimeout(flashTimer);
    flashTimer=setTimeout(()=>flash.value='',1600);
    props.store.notify(layer==='topmost'?'已添加到桌面，会始终显示在其他窗口上方':'已添加到桌面，位于桌面图标下方','success');
  });
}

const layers=[{value:'desktop' as const,label:'贴在桌面',hint:'位于桌面图标下方，不遮挡其他窗口',icon:Monitor},{value:'topmost' as const,label:'始终置顶',hint:'浮在所有窗口上方，随时看得见',icon:Pin}];
const layerTitle=computed(()=>{
  if(layerFor.value==='custom') return targetCustomName.value||'自定义组件';
  return widgetKinds.find(k=>k.kind===layerFor.value)?.label??'组件';
});

function askLayer(kind:WidgetKind){
  if(!ready.value||busy.value||layout.value.widgets.length>=12)return;
  targetCustomId.value=null;
  targetCustomName.value='';
  layerFor.value=layerFor.value===kind?null:kind;
}

function askCustomLayer(pack:CustomWidgetPack){
  if(!ready.value||busy.value||layout.value.widgets.length>=12)return;
  targetCustomId.value=pack.id;
  targetCustomName.value=pack.name;
  layerFor.value='custom';
}

async function loadCustom(){
  try{
    customWidgets.value=await client.listCustom();
  }catch{
    customWidgets.value=[];
  }
}

async function removeCustomPack(id:string,name:string){
  try{
    await client.removeCustom(id);
    await loadCustom();
    props.store.notify(`已删除自定义组件「${name}」`,'success');
  }catch(e){
    props.store.notify(e,'error');
  }
}

async function onImported(){
  await loadCustom();
  activeCategory.value='custom';
}

function openSettings(id:string){selected.value=id;settingsOpen.value=true;}
async function update(patch:Partial<DesktopWidget>){if(!chosen.value)throw Error('组件已移除');await client.update(chosen.value.id,patch);}
async function remove(){if(chosen.value){await client.remove(chosen.value.id);settingsOpen.value=false;}}

watch([()=>props.settingsRequest,ready],([request,loaded])=>{
  if(!request||!loaded)return;
  if(layout.value.widgets.some(w=>w.id===request.id)){
    openSettings(request.id);
  } else props.store.notify('这个组件已不在桌面上','info');
  emit('settingsOpened');
},{immediate:true});

function fit(){if(board.value)boardScale.value=Math.max(.1,Math.min(1,board.value.clientWidth/1314));}
onMounted(()=>{
  void client.load();
  void loadCustom();
  observer=new ResizeObserver(fit);
  if(board.value)observer.observe(board.value);
  fit();
});
onBeforeUnmount(()=>{observer?.disconnect();clearTimeout(flashTimer);client.dispose();});
</script>

<template>
<section class="ui-page xb-widgets-page" data-testid="widgets-page">
 <header class="page-header xb-page-header">
  <div><h1>桌面组件</h1><p>让日常的小事，在桌面上好好安放。</p></div>
  <div class="header-actions">
   <ResourceShareActions :store="store" kind="widget" import-label="导入组件" :disabled="busy" @imported="onImported"/>
   <NButton type="primary" round :disabled="!store.desktop" data-testid="widgets-create" @click="store.startCreation('widget')">
    <template #icon><Sparkles :size="15" /></template>创作组件
   </NButton>
  </div>
 </header>
 <NAlert v-if="error" type="error" :show-icon="false">{{ error }}<NButton v-if="!ready" text @click="client.load">重新读取</NButton></NAlert>
 <NAlert v-if="layout.errors.length" type="warning" :show-icon="false">{{ layout.errors.join('；') }}</NAlert>

 <div class="category-tabs-wrap">
  <NTabs v-model:value="activeCategory" type="line" size="large" class="category-tabs">
   <NTabPane name="builtin"><template #tab>内置组件</template></NTabPane>
   <NTabPane name="custom"><template #tab>{{ customWidgets.length ? `自定义组件 (${customWidgets.length})` : '自定义组件' }}</template></NTabPane>
  </NTabs>
 </div>

 <template v-if="activeCategory === 'builtin'">
  <div ref="board" class="xb-widget-board-wrap" :style="{height:`${967*boardScale}px`}"><div class="xb-widget-board" :style="{transform:`scale(${boardScale})`}">
   <div v-for="item in widgetKinds" :key="item.kind" class="xb-widget-slot" :class="item.kind" :style="{...positions[item.kind],left:`${positions[item.kind].left}px`,top:`${positions[item.kind].top}px`,width:`${baseSize(item.kind).width}px`,height:`${baseSize(item.kind).height}px`}"><WidgetSurface :widget="demo(item.kind)" preview demo @settings="askLayer(item.kind)"/><button class="add-to-desktop" :class="{added:flash===item.kind}" :disabled="!ready||busy||layout.widgets.length>=12" :data-testid="`add-${item.kind}`" :aria-label="`添加${item.title}到桌面`" aria-haspopup="dialog" :aria-expanded="layerFor===item.kind" @click="askLayer(item.kind)"><Check v-if="flash===item.kind" :size="18"/><Layers v-else :size="18"/>{{ flash===item.kind?'已添加':'添加到桌面' }}</button></div>
  </div></div><p class="xb-page-hint">点击“添加到桌面”后先选择放在哪一层 · 展示内容为示例，添加后使用真实时间、天气和行情。</p>
 </template>

 <template v-else>
  <div v-if="!customWidgets.length" class="xb-empty" data-testid="custom-empty">
   <span>还没有自定义组件</span>
   <p>点击右上角的「导入组件」导入资源包，或点击「创作组件」让小伴为你编写。</p>
   <button v-if="store.desktop" @click="store.startCreation('widget')">让小伴写一个组件</button>
  </div>
  <div v-else class="custom-widget-grid" data-testid="custom-widgets">
   <div v-for="pack in customWidgets" :key="pack.id" class="custom-card-wrapper" :data-testid="`custom-card-${pack.id}`">
    <div class="custom-widget-card surface">
     <div class="custom-preview-stage">
      <div class="custom-preview-inner" :style="{ width: `${pack.width || 280}px`, height: `${pack.height || 240}px`, transform: `scale(${Math.min(260 / (pack.width || 280), 160 / (pack.height || 240))})` }">
       <WidgetSurface :widget="customDemo(pack)" preview demo />
      </div>
     </div>
     <div class="custom-card-info">
      <strong>{{ pack.name }}</strong>
      <small v-if="pack.description">{{ pack.description }}</small>
      <div class="custom-card-footer">
       <button class="add-to-desktop custom-add" :class="{added:flash===pack.id}" :disabled="!ready||busy||layout.widgets.length>=12" :data-testid="`add-custom-${pack.id}`" :aria-label="`添加${pack.name}到桌面`" @click="askCustomLayer(pack)">
        <Check v-if="flash===pack.id" :size="16"/>
        <Layers v-else :size="16"/>
        {{ flash===pack.id?'已添加':'添加到桌面' }}
       </button>
       <ResourceShareActions :store="store" kind="widget" :id="pack.id" icons />
      </div>
     </div>
    </div>
    <NPopconfirm positive-text="删除" negative-text="取消" @positive-click="removeCustomPack(pack.id, pack.name)">
     <template #trigger>
      <button class="widget-delete-btn" title="删除这个自定义组件" :aria-label="`删除组件 ${pack.name}`" :data-testid="`delete-custom-${pack.id}`" @click.stop>
       <Trash2 :size="13" />
      </button>
     </template>
     确定要删除自创组件「{{ pack.name }}」吗？桌面上若有该组件实例也会一并清理。
    </NPopconfirm>
   </div>
  </div>
 </template>

 <NModal :show="!!layerFor" :auto-focus="false" style="width:372px;max-width:92vw" data-testid="widget-layer-dialog" @update:show="value=>{if(!value){layerFor=null;targetCustomId=null;}}">
  <section v-if="layerFor" class="layer-dialog" role="dialog" aria-labelledby="layer-dialog-title" @keydown.esc="layerFor=null">
   <header><span class="layer-icon"><Layers :size="18"/></span><div><h2 id="layer-dialog-title">把{{ layerTitle }}放在哪一层？</h2><p>添加后也能在组件设置里随时切换。</p></div></header>
   <button v-for="option in layers" :key="option.value" type="button" class="layer-option" :data-testid="`layer-${option.value}-${layerFor === 'custom' ? targetCustomId : layerFor}`" @click="add(layerFor,option.value)"><component :is="option.icon" :size="20"/><span><b>{{ option.label }}</b><small>{{ option.hint }}</small></span></button>
   <button type="button" class="layer-cancel" @click="layerFor=null;targetCustomId=null;">取消</button>
  </section>
 </NModal>

 <NModal :show="settingsOpen" :auto-focus="false" style="width:324px;max-width:92vw" data-testid="widget-settings" @update:show="settingsOpen=$event">
  <WidgetSettingsMenu v-if="chosen" :key="chosen.id" :widget="chosen" :monitors="monitors" :update="update" :remove="remove" @close="settingsOpen=false"/>
 </NModal>
</section>
</template>

<style scoped>
.xb-widgets-page{max-width:1430px;margin:0 auto;padding:28px 30px 34px!important;background:var(--ui-app-bg);border-radius:20px;display:flex;flex-direction:column;gap:20px;overflow-y:auto;min-height:0;height:100%;width:100%;flex:1;color:var(--ui-text-primary);box-sizing:border-box}
.xb-page-header{display:flex;justify-content:space-between;align-items:center;gap:20px}
.xb-page-header h1{font-size:25px;letter-spacing:.5px;margin:0;font-weight:600}
.xb-page-header p{font-size:13px;color:#9b8c7b;margin:8px 0 0}
.header-actions{display:flex;gap:10px;align-items:center;flex-wrap:wrap}
.category-tabs-wrap{margin-top:-6px}
.category-tabs :deep(.n-tabs-nav){margin-bottom:0}

.xb-widget-board-wrap{position:relative;width:100%;flex:none}
.xb-widget-board{position:absolute;top:0;left:0;width:1314px;height:967px;transform-origin:top left}
.xb-widget-slot{position:absolute}
.add-to-desktop{position:absolute;right:17px;bottom:13px;z-index:22;display:flex;align-items:center;gap:7px;border:1px solid #c4b09b;background:#f8f2e9f5;box-shadow:0 4px 15px #5f4a3425;color:#806347;border-radius:22px;padding:10px 16px;font:15px -apple-system,'PingFang SC',sans-serif;opacity:0;transform:translateY(5px);transition:.18s;cursor:pointer}
.xb-widget-slot:hover .add-to-desktop,.xb-widget-slot:focus-within .add-to-desktop,.add-to-desktop.added{opacity:1;transform:none}
.add-to-desktop:disabled{cursor:default;opacity:.4}

.custom-widget-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:20px;padding-bottom:24px}
.custom-card-wrapper{position:relative}
.custom-widget-card{border-radius:20px;background:#f7f3eb;box-shadow:0 4px 18px #7e6a4618;padding:16px;display:flex;flex-direction:column;gap:12px;overflow:hidden;position:relative}
.custom-preview-stage{height:170px;width:100%;display:grid;place-items:center;position:relative;background:#fbf8f3;border-radius:14px;overflow:hidden;border:1px solid #ebd9c488}
.custom-preview-inner{position:absolute;left:50%;top:50%;transform-origin:center;translate:-50% -50%;pointer-events:none}
.custom-card-info{display:flex;flex-direction:column;gap:8px}
.custom-card-info strong{font-size:15px;font-weight:600;color:#55432f}
.custom-card-info small{font-size:12px;color:#9b8c7b;line-height:1.5;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.custom-card-footer{display:flex;align-items:center;gap:10px;margin-top:4px}
.custom-add{position:static!important;opacity:1!important;transform:none!important;padding:8px 14px!important;font-size:13px!important;justify-content:center;flex:1;border-radius:12px!important}
.widget-delete-btn{position:absolute;top:8px;right:8px;z-index:3;width:26px;height:26px;border-radius:50%;border:0;background:rgba(0,0,0,0.45);color:#fff;display:grid;place-items:center;cursor:pointer;opacity:0.7;transition:all .2s}
.widget-delete-btn:hover{opacity:1;background:#dc2626}

.layer-dialog{display:grid;gap:10px;padding:22px;border-radius:18px;background:#fbf8f2;border:1px solid #ddcebb;box-shadow:0 18px 44px #4c3a2529;color:#55432f;text-align:left}
.layer-dialog header{display:flex;align-items:flex-start;gap:12px;margin-bottom:4px}
.layer-icon{flex:none;display:grid;place-items:center;width:38px;height:38px;border-radius:12px;background:#f1e7d9;color:#a98a63}
.layer-dialog h2{margin:0;font-size:16px;font-weight:600;color:#4c3a28}
.layer-dialog header p{margin:5px 0 0;font-size:12px;line-height:1.6;color:#a18e77}
.layer-option{display:flex;align-items:center;gap:13px;width:100%;border:1px solid #e2d4c2;border-radius:14px;background:#fffdf9;padding:13px 15px;text-align:left;color:#6f5a42;cursor:pointer;font:inherit;transition:border-color .15s,background .15s}
.layer-option:hover,.layer-option:focus-visible{border-color:#b99b74;background:#f7efe4}
.layer-option>svg{flex:none;color:#a98a63}
.layer-option>span{display:grid;gap:3px;min-width:0}
.layer-option b{font-size:14px;font-weight:600;color:#55432f}
.layer-option small{font-size:12px;line-height:1.5;color:#a18e77}
.layer-cancel{justify-self:center;border:0;background:none;color:#a4907a;font:12.5px inherit;padding:6px 12px;cursor:pointer}
.layer-cancel:hover{color:#7c6853}

.xb-page-hint{font-size:12px;color:#a4937e;line-height:1.8;text-align:center;margin:0}
.xb-empty{padding:100px 20px;text-align:center}
.xb-empty>span{font-size:21px;font-weight:600;color:#55432f}
.xb-empty p{font-size:13px;color:#a18c73;margin:12px auto;max-width:360px;line-height:1.6}
.xb-empty button{border:1px solid #c8b39b;border-radius:12px;background:#f9f4ec;padding:10px 18px;color:#9a7958;cursor:pointer;font-weight:500}
.xb-empty button:hover{background:#f3eae0}

@media(max-width:760px){
  .xb-widgets-page{padding:18px 14px!important}
  .xb-page-header h1{font-size:21px}
  .xb-page-header p{display:none}
  .add-to-desktop{opacity:1}
  .custom-widget-grid{grid-template-columns:1fr}
}
</style>
