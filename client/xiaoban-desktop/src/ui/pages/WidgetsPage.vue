<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Check,Layers,Pin,Monitor,MoreHorizontal,Plus,Settings2,Eye,EyeOff,Sparkles,Trash2} from 'lucide-vue-next';
import {NAlert,NModal,NPopconfirm} from 'naive-ui';
import type {ControlStore} from '../control-types.ts';
import StudioActions from '../components/StudioActions.vue';
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
const filter=ref<'all'|'desk'|'mine'>('all');
const customWidgets=ref<CustomWidgetPack[]>([]);
const userCustomWidgets=computed(()=>customWidgets.value.filter(p=>p.source!=='builtin'));
const galleryCustomWidgets=computed(()=>filter.value==='mine'?userCustomWidgets.value:customWidgets.value);

/** The widget kind or 'custom' whose add-button menu is open. */
const layerFor=ref<WidgetKind|null>(null);
const targetCustomId=ref<string|null>(null);
const targetCustomName=ref<string>('');

const chosen=computed(()=>layout.value.widgets.find(w=>w.id===selected.value));
const monitors=computed(()=>layout.value.monitors??[]);
const body=ref<HTMLElement>(),bodyWidth=ref(0);
let observer:ResizeObserver|undefined,flashTimer:ReturnType<typeof setTimeout>;
/** A widget is drawn at its desktop size and shrunk into a preview stage; these are the stages' sizes. */
const deskStage=computed(()=>({w:bodyWidth.value?(bodyWidth.value-32)/3:280,h:150}));
const galleryStage=computed(()=>({w:bodyWidth.value?(bodyWidth.value-48)/5-2:170,h:116}));
function fitStyle(kind:WidgetKind,stage:{w:number;h:number},pack?:CustomWidgetPack){
  const size=baseSize(kind,pack),scale=Math.min((stage.w-24)/size.width,(stage.h-20)/size.height);
  return {width:`${size.width}px`,height:`${size.height}px`,transform:`scale(${Math.max(.05,scale)})`};
}
const desktopWidgets=computed(()=>layout.value.widgets);
const widgetName=(widget:DesktopWidget)=>widget.kind==='custom'?(widget.custom?.name??customWidgets.value.find(p=>p.id===widget.customId)?.name??'自定义组件'):(widgetKinds.find(k=>k.kind===widget.kind)?.title??'组件');
const visibleCount=computed(()=>desktopWidgets.value.filter(w=>w.enabled).length);
const moreFor=ref('');
async function setLayer(widget:DesktopWidget,layer:'desktop'|'topmost'){if((widget.layer??'desktop')===layer)return;await action(()=>client.update(widget.id,{layer}));}
async function setEnabled(widget:DesktopWidget,enabled:boolean){moreFor.value='';await action(()=>client.update(widget.id,{enabled}));}
async function removeWidget(widget:DesktopWidget){moreFor.value='';await action(async()=>{await client.remove(widget.id);props.store.notify('已从桌面移除','success');});}
async function setAllEnabled(enabled:boolean){await action(async()=>{for(const widget of desktopWidgets.value)if(widget.enabled!==enabled)await client.update(widget.id,{enabled});});}
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
  if(!ready.value||busy.value)return;
  targetCustomId.value=null;
  targetCustomName.value='';
  layerFor.value=layerFor.value===kind?null:kind;
}

function askCustomLayer(pack:CustomWidgetPack){
  if(!ready.value||busy.value)return;
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
  filter.value='mine';
}

function openSettings(id:string){selected.value=id;settingsOpen.value=true;}
async function update(patch:Partial<DesktopWidget>){if(!chosen.value)throw Error('组件已移除');await client.update(chosen.value.id,patch);}
async function remove(){if(chosen.value){await client.remove(chosen.value.id);settingsOpen.value=false;}}

watch([()=>props.settingsRequest,ready],([request,loaded])=>{
  if(!request||!loaded)return;
  if(layout.value.widgets.some(w=>w.id===request.id)){
    filter.value='desk';
    openSettings(request.id);
  } else props.store.notify('这个组件已不在桌面上','info');
  emit('settingsOpened');
},{immediate:true});

function fit(){if(body.value)bodyWidth.value=Math.max(0,body.value.clientWidth-56);}
onMounted(()=>{
  void client.load();
  void loadCustom();
  observer=new ResizeObserver(fit);
  if(body.value)observer.observe(body.value);
  fit();
  window.addEventListener('keydown',onKey);
});
function onKey(event:KeyboardEvent){if(event.key==='Escape'&&moreFor.value)moreFor.value='';}
onBeforeUnmount(()=>{window.removeEventListener('keydown',onKey);observer?.disconnect();clearTimeout(flashTimer);client.dispose();});
</script>

<template>
<section class="widgets-page" data-testid="widgets-page">
 <StudioActions>
  <ResourceShareActions :store="store" kind="widget" variant="xb" import-label="导入" :disabled="busy" @imported="onImported"/>
  <button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="widgets-create" @click="store.startCreation('widget')"><Sparkles :size="14"/>让小伴创作</button>
 </StudioActions>
 <div class="wd-toolbar">
  <div class="xb-seg" role="group" aria-label="组件筛选">
   <button type="button" :aria-pressed="filter==='all'" data-testid="widgets-filter-all" @click="filter='all';moreFor=''">全部</button>
   <button type="button" :aria-pressed="filter==='desk'" data-testid="widgets-filter-desk" @click="filter='desk';moreFor=''">在桌面上 <b class="cnt">{{ desktopWidgets.length }}</b></button>
   <button type="button" :aria-pressed="filter==='mine'" data-testid="widgets-filter-mine" @click="filter='mine';moreFor=''">我的</button>
  </div>
 </div>
 <div ref="body" class="wd-body">
 <NAlert v-if="error" type="error" :show-icon="false" class="wd-alert">{{ error }}<button v-if="!ready" type="button" class="xb-link" @click="client.load">重新读取</button></NAlert>
 <NAlert v-if="layout.errors.length" type="warning" :show-icon="false" class="wd-alert">{{ layout.errors.join('；') }}</NAlert>

 <section v-if="filter==='desk'" aria-label="桌面上的组件" data-testid="widgets-on-desktop">
  <div class="sec">桌面上<span class="sub">{{ desktopWidgets.length }} 个 · 拖动可在桌面上调整位置</span><button v-if="desktopWidgets.length" type="button" class="sec-link" data-testid="widgets-toggle-all" @click="setAllEnabled(visibleCount===0)">{{ visibleCount===0?'全部显示':'全部隐藏' }}</button></div>
  <div v-if="!desktopWidgets.length" class="desk-empty" data-testid="widgets-desktop-empty">桌面上还没有组件。<button type="button" class="xb-link" @click="filter='all'">去挑选组件</button></div>
  <div v-else class="desk-grid">
   <article v-for="widget in desktopWidgets" :key="widget.id" class="on-desk" :class="{hidden:!widget.enabled}" :data-testid="`desk-widget-${widget.id}`">
    <div class="stage" :style="{height:`${deskStage.h}px`}"><div class="stage-inner" :style="fitStyle(widget.kind,deskStage,widget.custom)"><WidgetSurface :widget="widget" preview/></div><span v-if="!widget.enabled" class="hidden-tag">已隐藏</span></div>
    <div class="row"><b>{{ widgetName(widget) }}</b>
     <div class="xb-seg" role="radiogroup" :aria-label="`${widgetName(widget)}层级`"><button v-for="option in layers" :key="option.value" type="button" role="radio" :aria-checked="(widget.layer??'desktop')===option.value" :disabled="busy" :data-testid="`desk-layer-${option.value}-${widget.id}`" @click="setLayer(widget,option.value)">{{ option.label }}</button></div>
     <div class="more-wrap"><button type="button" class="xb-icon-btn is-sm" :aria-label="`${widgetName(widget)}更多设置`" aria-haspopup="menu" :aria-expanded="moreFor===widget.id" :data-testid="`desk-more-${widget.id}`" @click="moreFor=moreFor===widget.id?'':widget.id"><MoreHorizontal :size="15"/></button>
      <template v-if="moreFor===widget.id"><div class="menu-dismiss" @pointerdown="moreFor=''"></div><div class="xb-menu more-menu" role="menu">
       <button type="button" role="menuitem" class="xb-menu-item" :data-testid="`desk-settings-${widget.id}`" @click="moreFor='';openSettings(widget.id)"><Settings2 :size="15"/>组件设置</button>
       <button type="button" role="menuitem" class="xb-menu-item" @click="setEnabled(widget,!widget.enabled)"><component :is="widget.enabled?EyeOff:Eye" :size="15"/>{{ widget.enabled?'隐藏':'显示在桌面' }}</button>
       <div class="xb-menu-sep"></div>
       <button type="button" role="menuitem" class="xb-menu-item is-danger" :data-testid="`desk-remove-${widget.id}`" @click="removeWidget(widget)"><Trash2 :size="15"/>从桌面移除</button>
      </div></template></div>
    </div>
   </article>
  </div>
 </section>

 <section v-if="filter!=='desk'" aria-label="添加组件" class="add-section">
  <div class="sec">{{ filter==='mine'?'我的组件':'全部组件' }}</div>
   <div v-if="filter==='mine'&&!userCustomWidgets.length" class="xb-empty" data-testid="custom-empty">
    <span class="xb-empty-icon"><Sparkles :size="24"/></span>
    <h3>还没有自定义组件</h3>
    <p>点击右上角的“导入”导入资源包，或点击“让小伴创作”让小伴为你编写。</p>
    <button v-if="store.desktop" type="button" class="xb-btn" @click="store.startCreation('widget')">让小伴写一个组件</button>
   </div>
   <div v-else class="gallery" :data-testid="filter==='mine'?'custom-widgets':'widget-gallery'">
    <template v-if="filter==='all'">
     <div v-for="item in widgetKinds" :key="item.kind" class="gt xb-widget-slot" :class="item.kind" :title="item.description">
      <div class="pv"><div class="stage-inner" :style="fitStyle(item.kind,galleryStage)"><WidgetSurface :widget="demo(item.kind)" preview demo/></div></div>
      <div class="ft"><b>{{ item.title }}</b><button class="add" :class="{added:flash===item.kind}" :disabled="!ready||busy" :data-testid="`add-${item.kind}`" :aria-label="`添加${item.title}到桌面`" aria-haspopup="dialog" :aria-expanded="layerFor===item.kind" @click="askLayer(item.kind)"><Check v-if="flash===item.kind" :size="14"/><Plus v-else :size="14"/></button></div>
     </div>
    </template>
    <div v-for="pack in galleryCustomWidgets" :key="pack.id" class="gt" :data-testid="`custom-card-${pack.id}`">
     <div class="pv"><div class="stage-inner" :style="fitStyle('custom',galleryStage,pack)"><WidgetSurface :widget="customDemo(pack)" preview demo/></div>
      <div class="card-tools"><ResourceShareActions :store="store" kind="widget" :id="pack.id" icons/>
       <NPopconfirm v-if="pack.source !== 'builtin'" positive-text="删除" negative-text="取消" @positive-click="removeCustomPack(pack.id, pack.name)"><template #trigger><button class="widget-delete-btn" title="删除这个自定义组件" :aria-label="`删除组件 ${pack.name}`" :data-testid="`delete-custom-${pack.id}`" @click.stop><Trash2 :size="13"/></button></template>确定要删除自创组件「{{ pack.name }}」吗？桌面上若有该组件实例也会一并清理。</NPopconfirm></div></div>
     <div class="ft"><b>{{ pack.name }}</b><button class="add" :class="{added:flash===pack.id}" :disabled="!ready||busy" :data-testid="`add-custom-${pack.id}`" :aria-label="`添加${pack.name}到桌面`" @click="askCustomLayer(pack)"><Check v-if="flash===pack.id" :size="14"/><Plus v-else :size="14"/></button></div>
    </div>
   </div>
  <p v-if="filter==='all'||galleryCustomWidgets.length" class="wd-hint">添加时先选择放在哪一层 · 缩略图为示例，添加后使用真实时间、天气和行情。</p>
 </section>
 </div>

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
.widgets-page{height:100%;min-height:0;display:flex;flex-direction:column}
.wd-toolbar{flex:none;display:flex;align-items:center;gap:10px;padding:16px 28px 0}.wd-toolbar .cnt{font-weight:600;color:var(--ui-ink)}
.wd-body{flex:1;min-height:0;overflow-y:auto;padding:22px 28px 28px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.wd-alert{margin-bottom:14px;border-radius:10px}
.sec{display:flex;align-items:center;gap:10px;font-size:15px;font-weight:600}
.sec .sub{font-size:12px;font-weight:400;color:var(--ui-text-secondary)}
.sec-link{margin-left:auto;border:0;background:none;padding:0;font-size:12.5px;font-weight:500;color:var(--ui-ink);cursor:pointer}
.desk-empty{margin-top:12px;padding:26px;border-radius:14px;background:var(--ui-surface-sunken);font-size:13px;color:var(--ui-text-secondary);text-align:center}
.desk-grid{margin-top:12px;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}
.on-desk{min-width:0;border-radius:14px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-line),var(--ui-shadow-card);padding:8px 8px 6px}
.stage,.pv{position:relative;overflow:hidden;border-radius:10px;background:var(--ui-surface-sunken)}
.stage{display:grid;place-items:center}
.stage-inner{position:absolute;left:50%;top:50%;translate:-50% -50%;transform-origin:center;pointer-events:none}
.on-desk.hidden .stage-inner{opacity:.45;filter:grayscale(.6)}
.hidden-tag{position:absolute;top:8px;left:8px;height:20px;padding:0 7px;border-radius:6px;background:rgba(0,0,0,.42);color:#fff;font-size:11px;display:inline-flex;align-items:center}
.on-desk .row{display:flex;align-items:center;gap:8px;padding:8px 2px 2px}
.on-desk .row b{flex:1;min-width:0;font-size:13.5px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.on-desk .xb-seg>button{height:22px;padding:0 8px;font-size:12px}
.more-wrap{position:relative;flex:none}
.menu-dismiss{position:fixed;inset:0;z-index:30}
.more-menu{position:absolute;right:0;top:28px;z-index:31;min-width:160px}
.add-section{margin-top:0}
.gallery{margin-top:12px;display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:12px}
.gt{min-width:0;border-radius:12px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-line);padding:6px 6px 4px;transition:box-shadow .15s}
.gt:hover{box-shadow:0 0 0 1px var(--ui-border-strong),0 4px 12px var(--ui-shadow-hover)}
.pv{height:116px}
.ft{display:flex;align-items:center;gap:8px;padding:7px 4px 4px}
.ft b{flex:1;min-width:0;font-size:13px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.add{flex:none;width:26px;height:26px;display:grid;place-items:center;border:0;border-radius:8px;background:var(--ui-soft);color:var(--ui-ink);cursor:pointer}
.add:hover:not(:disabled){filter:brightness(.96)}
.add.added{background:var(--ui-btn);color:#fff}
.card-tools{position:absolute;top:5px;right:5px;z-index:3;display:flex;gap:3px;opacity:0;transition:opacity .15s}
.gt:hover .card-tools,.card-tools:focus-within{opacity:1}
.widget-delete-btn{width:26px;height:26px;border-radius:50%;border:0;background:rgba(0,0,0,.5);color:#fff;display:grid;place-items:center;cursor:pointer}
.widget-delete-btn:hover{background:var(--ui-danger)}
.wd-hint{margin:16px 0 0;font-size:12px;color:var(--ui-text-tertiary);text-align:center}
.layer-dialog{display:grid;gap:10px;padding:22px;border-radius:var(--ui-radius-dialog);background:var(--ui-surface);box-shadow:var(--ui-shadow-dialog);color:var(--ui-text-primary);text-align:left}
.layer-dialog header{display:flex;align-items:flex-start;gap:12px;margin-bottom:4px}
.layer-icon{flex:none;display:grid;place-items:center;width:38px;height:38px;border-radius:12px;background:var(--ui-soft);color:var(--ui-ink)}
.layer-dialog h2{margin:0;font-size:16px;font-weight:600}
.layer-dialog header p{margin:5px 0 0;font-size:12px;line-height:1.6;color:var(--ui-text-secondary)}
.layer-option{display:flex;align-items:center;gap:13px;width:100%;border:0;border-radius:12px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border);padding:13px 15px;text-align:left;color:var(--ui-text-primary);cursor:pointer;font:inherit}
.layer-option:hover,.layer-option:focus-visible{box-shadow:0 0 0 1.5px var(--ui-btn);background:var(--ui-soft)}
.layer-option>svg{flex:none;color:var(--ui-ink)}
.layer-option>span{display:grid;gap:3px;min-width:0}
.layer-option b{font-size:14px;font-weight:600}
.layer-option small{font-size:12px;line-height:1.5;color:var(--ui-text-secondary)}
.layer-cancel{justify-self:center;border:0;background:none;color:var(--ui-text-secondary);font-size:12.5px;padding:6px 12px;cursor:pointer}
.layer-cancel:hover{color:var(--ui-text-primary)}
@media(max-width:1000px){.gallery{grid-template-columns:repeat(3,minmax(0,1fr))}.desk-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>
