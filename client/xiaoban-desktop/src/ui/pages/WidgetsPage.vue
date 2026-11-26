<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {Check,Pin,Monitor,MoreHorizontal,Plus,Settings2,Eye,EyeOff,Sparkles,Trash2} from 'lucide-vue-next';
import {NAlert,NModal,NPopconfirm} from 'naive-ui';
import type {ControlStore} from '../control-types.ts';
import StudioActions from '../components/StudioActions.vue';
import ResourceShareActions from '../components/ResourceShareActions.vue';
import {createWidgetClient} from '../../widgets/client.ts';
import {widgetKinds,type WidgetKind,type DesktopWidget,type CustomWidgetPack} from '../../widgets/model.ts';
import {emptyCountdown,emptyNote} from '../../widgets/content.ts';
import {dateKey} from '../../widgets/calendar.ts';
import WidgetPreview from '../components/WidgetPreview.vue';
import WidgetSettingsMenu from '../../widgets/WidgetSettingsMenu.vue';

const props=defineProps<{store:ControlStore;settingsRequest?:{id:string}|null}>();
const emit=defineEmits<{settingsOpened:[]}>();
const client=createWidgetClient(),{layout,ready,busy,error}=client;
const selected=ref(''),settingsOpen=ref(false),flash=ref(''),locatedId=ref('');
const filter=ref<'all'|'desk'|'mine'>('all');
const customWidgets=ref<CustomWidgetPack[]>([]);
const userCustomWidgets=computed(()=>customWidgets.value.filter(p=>p.source!=='builtin'));
const galleryCustomWidgets=computed(()=>filter.value==='mine'?userCustomWidgets.value:customWidgets.value);

/** The key (`clock` or `custom:<id>`) whose lightweight layer xb-menu is open. */
const layerMenuKey=ref('');
const moreFor=ref('');

const chosen=computed(()=>layout.value.widgets.find(w=>w.id===selected.value));
const monitors=computed(()=>layout.value.monitors??[]);
let flashTimer:ReturnType<typeof setTimeout>|undefined;
let locateTimer:ReturnType<typeof setTimeout>|undefined;
const desktopWidgets=computed(()=>layout.value.widgets);
const widgetName=(widget:DesktopWidget)=>widget.kind==='custom'?(widget.custom?.name??customWidgets.value.find(p=>p.id===widget.customId)?.name??'自定义组件'):(widgetKinds.find(k=>k.kind===widget.kind)?.title??'组件');
const visibleCount=computed(()=>desktopWidgets.value.filter(w=>w.enabled).length);
const isWideKind=(kind:WidgetKind)=>kind==='clock'||kind==='countdown';

function monitorLabel(widget:DesktopWidget){
  if(!monitors.value.length)return '';
  const m=widget.monitor?monitors.value.find(x=>x.id===widget.monitor):monitors.value.find(x=>x.primary);
  if(!m)return '';
  return m.primary?'主屏':m.name;
}
function deskBadge(widget:DesktopWidget){
  const mon=monitorLabel(widget);
  const pct=`${Math.round(widget.scale*100)}%`;
  return mon?`${mon} · ${pct}`:pct;
}
function deskStats(kind:WidgetKind,customId?:string){
  const list=desktopWidgets.value.filter(w=>customId?w.customId===customId:w.kind===kind);
  return {total:list.length,visible:list.filter(w=>w.enabled).length,firstId:list[0]?.id||''};
}

async function setLayer(widget:DesktopWidget,layer:'desktop'|'topmost'){if((widget.layer??'desktop')===layer)return;await action(()=>client.update(widget.id,{layer}));}
async function setEnabled(widget:DesktopWidget,enabled:boolean){moreFor.value='';await action(()=>client.update(widget.id,{enabled}));}
async function removeWidget(widget:DesktopWidget){moreFor.value='';await action(async()=>{await client.remove(widget.id);props.store.notify('已从桌面移除','success');});}
async function setAllEnabled(enabled:boolean){await action(async()=>{const targets=desktopWidgets.value.filter(w=>w.enabled!==enabled);await Promise.all(targets.map(widget=>client.update(widget.id,{enabled})));});}

const demo=(kind:WidgetKind):DesktopWidget=>({id:`demo${kind}`,kind,theme:'ivory',scale:.5,x:0,y:0,enabled:true,city:null,...(kind==='notes'?{note:{...emptyNote(),items:[{id:'task1',text:'完成产品周报',done:true,time:'09:00'},{id:'task2',text:'准备下周的设计评审',done:false,time:'14:00'},{id:'task3',text:'阅读一本书',done:false,time:'20:00'}]}}:kind==='reminders'?{living:{symbol:'1.000001',count:0,reminders:[{id:'demo1',title:'产品设计方案评审',at:dateKey(new Date())+'T10:00',end:'11:30',location:'会议室 A302',done:false},{id:'demo2',title:'与开发同学同步进度',at:dateKey(new Date())+'T14:00',end:'15:00',done:false},{id:'demo3',title:'健身',at:dateKey(new Date())+'T19:00',end:'20:00',done:false}]}}:kind==='woodfish'?{living:{count:108,symbol:'1.000001',reminders:[]}}:kind==='stocks'?{living:{count:0,symbol:'116.00700',reminders:[]}}:kind==='countdown'?{countdown:{...emptyCountdown(),title:'我们的纪念日',date:dateKey(new Date(Date.now()+79*86400000))}}:{})});
const customDemo=(pack:CustomWidgetPack):DesktopWidget=>({id:`demo-${pack.id}`,kind:'custom',theme:pack.theme||'transparent',scale:1,x:0,y:0,enabled:true,city:null,customId:pack.id,custom:pack});

async function action(fn:()=>Promise<unknown>){try{await fn();}catch(e){props.store.notify(e,'error');}}

async function add(kind:WidgetKind,layer:'desktop'|'topmost',customId?:string){
  layerMenuKey.value='';
  await action(async()=>{
    await client.add(kind,layer,customId);
    flash.value=kind==='custom'?(customId||'custom'):kind;
    clearTimeout(flashTimer);
    flashTimer=setTimeout(()=>flash.value='',1600);
    props.store.notify(layer==='topmost'?'已添加到桌面，会始终显示在其他窗口上方':'已添加到桌面，位于桌面图标下方','success');
  });
}

const layers=[{value:'desktop' as const,label:'贴在桌面',hint:'位于桌面图标下方，不遮挡其他窗口',icon:Monitor},{value:'topmost' as const,label:'始终置顶',hint:'浮在所有窗口上方，随时看得见',icon:Pin}];

function toggleLayerMenu(key:string){
  if(!ready.value||busy.value)return;
  moreFor.value='';
  layerMenuKey.value=layerMenuKey.value===key?'':key;
}
function locateOnDesktop(id:string){
  layerMenuKey.value='';
  moreFor.value='';
  filter.value='desk';
  locatedId.value=id;
  clearTimeout(locateTimer);
  locateTimer=setTimeout(()=>locatedId.value='',1600);
}
function deriveCustom(pack:CustomWidgetPack){
  void Promise.resolve(props.store.startCreation('widget')).then(()=>{
    props.store.creationDraft=`请以自定义桌面组件「${pack.name}」（${pack.id}）为参考，创作我的新桌面组件：`;
  }).catch(e=>props.store.notify(e,'error'));
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

function openSettings(id:string){moreFor.value='';selected.value=id;settingsOpen.value=true;}
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

onMounted(()=>{
  void client.load();
  void loadCustom();
  window.addEventListener('keydown',onKey);
});
function onKey(event:KeyboardEvent){
  if(event.key==='Escape'){
    if(moreFor.value)moreFor.value='';
    if(layerMenuKey.value)layerMenuKey.value='';
  }
}
onBeforeUnmount(()=>{window.removeEventListener('keydown',onKey);clearTimeout(flashTimer);clearTimeout(locateTimer);client.dispose();});
</script>

<template>
<section class="widgets-page" data-testid="widgets-page">
 <StudioActions>
  <ResourceShareActions :store="store" kind="widget" variant="xb" import-label="导入" html-kind="widget" :disabled="busy" @imported="onImported"/>
  <button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="widgets-create" @click="store.startCreation('widget')"><Sparkles :size="14"/>让小伴创作</button>
 </StudioActions>
 <div class="wd-toolbar">
  <div class="xb-seg" role="group" aria-label="组件筛选">
   <button type="button" :aria-pressed="filter==='all'" data-testid="widgets-filter-all" @click="filter='all';moreFor='';layerMenuKey=''">全部</button>
   <button type="button" :aria-pressed="filter==='desk'" data-testid="widgets-filter-desk" @click="filter='desk';moreFor='';layerMenuKey=''">在桌面上 <b class="cnt">{{ desktopWidgets.length }}</b></button>
   <button type="button" :aria-pressed="filter==='mine'" data-testid="widgets-filter-mine" @click="filter='mine';moreFor='';layerMenuKey=''">我的</button>
  </div>
  <span v-if="filter==='all'" class="wd-toolbar-hint">点 + 选择贴桌面或置顶 · 宽组件跨列预览</span>
 </div>
 <div class="wd-body">
  <NAlert v-if="error" type="error" :show-icon="false" class="wd-alert">{{ error }}<button v-if="!ready" type="button" class="xb-link" @click="client.load">重新读取</button></NAlert>
  <NAlert v-if="layout.errors.length" type="warning" :show-icon="false" class="wd-alert">{{ layout.errors.join('；') }}</NAlert>

  <section v-if="filter==='desk'" aria-label="桌面上的组件" data-testid="widgets-on-desktop">
   <div class="sec">桌面上<span class="sub">{{ desktopWidgets.length }} 个 · 点击预览直接调整设置，拖动可调位置</span><button v-if="desktopWidgets.length" type="button" class="sec-link" data-testid="widgets-toggle-all" @click="setAllEnabled(visibleCount===0)">{{ visibleCount===0?'全部显示':'全部隐藏' }}</button></div>
   <div v-if="!desktopWidgets.length" class="desk-empty" data-testid="widgets-desktop-empty">桌面上还没有组件。<button type="button" class="xb-link" @click="filter='all'">去挑选组件</button></div>
   <div v-else class="desk-grid">
    <article v-for="(widget, index) in desktopWidgets" :key="widget.id" class="on-desk" :class="{hidden:!widget.enabled,located:locatedId===widget.id}" :data-testid="`desk-widget-${widget.id}`">
     <WidgetPreview class="stage is-clickable" :widget="widget" @click="openSettings(widget.id)">
      <div class="desk-badges">
       <span class="desk-meta-tag">{{ deskBadge(widget) }}</span>
       <span v-if="!widget.enabled" class="hidden-tag">已隐藏</span>
      </div>
     </WidgetPreview>
     <div class="row"><b>{{ widgetName(widget) }}</b>
      <div class="xb-seg" role="radiogroup" :aria-label="`${widgetName(widget)}层级`"><button v-for="option in layers" :key="option.value" type="button" role="radio" :aria-checked="(widget.layer??'desktop')===option.value" :disabled="busy" :data-testid="`desk-layer-${option.value}-${widget.id}`" @click="setLayer(widget,option.value)">{{ option.label }}</button></div>
      <div class="more-wrap"><button type="button" class="xb-icon-btn is-sm" :aria-label="`${widgetName(widget)}更多设置`" aria-haspopup="menu" :aria-expanded="moreFor===widget.id" :data-testid="`desk-more-${widget.id}`" @click="moreFor=moreFor===widget.id?'':widget.id"><MoreHorizontal :size="15"/></button>
       <template v-if="moreFor===widget.id"><div class="menu-dismiss" @pointerdown="moreFor=''"></div><div class="xb-menu more-menu" :class="{'is-up':desktopWidgets.length>3&&index>=desktopWidgets.length-3}" role="menu">
        <button type="button" role="menuitem" class="xb-menu-item" :data-testid="`desk-settings-${widget.id}`" @click="openSettings(widget.id)"><Settings2 :size="15"/>组件设置</button>
        <button type="button" role="menuitem" class="xb-menu-item" @click="setEnabled(widget,!widget.enabled)"><component :is="widget.enabled?EyeOff:Eye" :size="15"/>{{ widget.enabled?'隐藏':'显示在桌面' }}</button>
        <div class="xb-menu-sep"></div>
        <button type="button" role="menuitem" class="xb-menu-item is-danger" :data-testid="`desk-remove-${widget.id}`" @click="removeWidget(widget)"><Trash2 :size="15"/>从桌面移除</button>
       </div></template></div>
     </div>
    </article>
   </div>
  </section>

  <section v-if="filter!=='desk'" aria-label="添加组件" class="add-section">
   <div v-if="filter==='mine'&&!userCustomWidgets.length" class="xb-empty" data-testid="custom-empty">
    <span class="xb-empty-icon"><Sparkles :size="24"/></span>
    <h3>还没有自定义组件</h3>
    <p>可以导入 ZIP 资源包或单文件 HTML，也可以让小伴创作。</p>
    <button v-if="store.desktop" type="button" class="xb-btn" @click="store.startCreation('widget')">让小伴写一个组件</button>
   </div>
   <template v-else>
    <div v-if="filter==='all'" class="sec-sub">官方内置<span>8 款 · 暖白 / 深色 / 柔雾玻璃</span></div>
    <div class="gallery" :data-testid="filter==='mine'?'custom-widgets':'widget-gallery'">
     <template v-if="filter==='all'">
      <div v-for="(item, idx) in widgetKinds" :key="item.kind" class="gt xb-widget-slot" :class="[item.kind, {'is-wide': isWideKind(item.kind)}]" :title="item.description" :aria-description="item.description">
       <WidgetPreview class="pv" :widget="demo(item.kind)" demo @click="toggleLayerMenu(item.kind)"/>
       <div class="ft">
        <b>{{ item.title }}</b>
        <button v-if="deskStats(item.kind).total" type="button" class="desk-pill" :class="{'is-hidden':deskStats(item.kind).visible===0}" :aria-label="`管理${item.title}`" title="定位到桌面上的组件" @click="locateOnDesktop(deskStats(item.kind).firstId)">{{ deskStats(item.kind).visible===0?'已隐藏':`桌面 ${deskStats(item.kind).total}` }}</button>
        <div class="add-wrap">
         <button type="button" class="add" :class="{added:flash===item.kind}" :disabled="!ready||busy" :data-testid="`add-${item.kind}`" :aria-label="`添加${item.title}到桌面`" aria-haspopup="menu" :aria-expanded="layerMenuKey===item.kind" @click.stop="toggleLayerMenu(item.kind)"><Check v-if="flash===item.kind" :size="14"/><Plus v-else :size="14"/></button>
         <template v-if="layerMenuKey===item.kind">
          <div class="menu-dismiss" @pointerdown="layerMenuKey=''"></div>
          <div class="xb-menu layer-menu" :class="{'is-up':idx>=widgetKinds.length-2}" role="menu" data-testid="widget-layer-menu">
           <button v-for="option in layers" :key="option.value" type="button" role="menuitem" class="xb-menu-item" :title="option.hint" :aria-description="option.hint" :data-testid="`layer-${option.value}-${item.kind}`" @click="add(item.kind,option.value)"><component :is="option.icon" :size="14"/>{{ option.label }}<span v-if="option.value==='desktop'" class="xb-menu-hint">推荐</span></button>
          </div>
         </template>
        </div>
       </div>
      </div>
     </template>
    </div>

    <template v-if="galleryCustomWidgets.length">
     <div v-if="filter==='all'" class="sec-sub is-spaced">创意与自定义<span>{{ galleryCustomWidgets.length }} 款 · 支持导入或小伴创作</span></div>
     <div class="gallery" :class="{'custom-grid':filter==='all'}">
      <div v-for="pack in galleryCustomWidgets" :key="pack.id" class="gt" :data-testid="`custom-card-${pack.id}`" :title="pack.description">
       <WidgetPreview class="pv" :widget="customDemo(pack)" demo @click="toggleLayerMenu(`custom:${pack.id}`)">
        <div class="card-tools">
         <button v-if="store.desktop" type="button" class="widget-derive-btn" title="基于它让小伴二次创作" :aria-label="`基于${pack.name}创作`" @click.stop="deriveCustom(pack)"><Sparkles :size="12"/></button>
         <ResourceShareActions :store="store" kind="widget" :id="pack.id" icons/>
         <NPopconfirm v-if="pack.source !== 'builtin'" positive-text="删除" negative-text="取消" @positive-click="removeCustomPack(pack.id, pack.name)"><template #trigger><button class="widget-delete-btn" title="删除这个自定义组件" :aria-label="`删除组件 ${pack.name}`" :data-testid="`delete-custom-${pack.id}`" @click.stop><Trash2 :size="13"/></button></template>确定要删除自创组件「{{ pack.name }}」吗？桌面上若有该组件实例也会一并清理。</NPopconfirm>
        </div>
       </WidgetPreview>
       <div class="ft">
        <b>{{ pack.name }}</b>
        <button v-if="deskStats('custom',pack.id).total" type="button" class="desk-pill" :class="{'is-hidden':deskStats('custom',pack.id).visible===0}" :aria-label="`管理${pack.name}`" title="定位到桌面上的组件" @click="locateOnDesktop(deskStats('custom',pack.id).firstId)">{{ deskStats('custom',pack.id).visible===0?'已隐藏':`桌面 ${deskStats('custom',pack.id).total}` }}</button>
        <div class="add-wrap">
         <button type="button" class="add" :class="{added:flash===pack.id}" :disabled="!ready||busy" :data-testid="`add-custom-${pack.id}`" :aria-label="`添加${pack.name}到桌面`" aria-haspopup="menu" :aria-expanded="layerMenuKey===`custom:${pack.id}`" @click.stop="toggleLayerMenu(`custom:${pack.id}`)"><Check v-if="flash===pack.id" :size="14"/><Plus v-else :size="14"/></button>
         <template v-if="layerMenuKey===`custom:${pack.id}`">
          <div class="menu-dismiss" @pointerdown="layerMenuKey=''"></div>
          <div class="xb-menu layer-menu is-up" role="menu" data-testid="widget-layer-menu">
           <button v-for="option in layers" :key="option.value" type="button" role="menuitem" class="xb-menu-item" :title="option.hint" :aria-description="option.hint" :data-testid="`layer-${option.value}-${pack.id}`" @click="add('custom',option.value,pack.id)"><component :is="option.icon" :size="14"/>{{ option.label }}<span v-if="option.value==='desktop'" class="xb-menu-hint">推荐</span></button>
          </div>
         </template>
        </div>
       </div>
      </div>
     </div>
    </template>
   </template>
   <p v-if="filter==='all'||galleryCustomWidgets.length" class="wd-hint">缩略图为示例，添加后使用真实时间、天气和行情；在「在桌面上」可随时切换材质、大小和层级。</p>
  </section>
 </div>

 <NModal :show="settingsOpen" :auto-focus="false" style="width:336px;max-width:92vw" data-testid="widget-settings" @update:show="settingsOpen=$event">
  <WidgetSettingsMenu v-if="chosen" :key="chosen.id" :widget="chosen" :monitors="monitors" :update="update" :remove="remove" @close="settingsOpen=false"/>
 </NModal>
</section>
</template>

<style scoped>
.widgets-page{container-type:inline-size;min-width:0;height:100%;min-height:0;display:flex;flex-direction:column}
.wd-toolbar{flex:none;display:flex;align-items:center;gap:12px;flex-wrap:wrap;padding:16px 28px 0}.wd-toolbar .cnt{font-weight:600;color:var(--ui-ink)}
.wd-toolbar-hint{margin-left:auto;font-size:12px;color:var(--ui-text-tertiary)}
.wd-body{flex:1;min-height:0;overflow-y:auto;padding:18px 28px 28px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.wd-alert{margin-bottom:14px;border-radius:10px}
.sec{display:flex;align-items:center;gap:10px;font-size:15px;font-weight:600}
.sec .sub{font-size:12px;font-weight:400;color:var(--ui-text-secondary)}
.sec-link{margin-left:auto;border:0;background:none;padding:0;font-size:12.5px;font-weight:500;color:var(--ui-ink);cursor:pointer}
.sec-sub{display:flex;align-items:baseline;gap:8px;margin:0 0 12px;font-size:13.5px;font-weight:600;color:var(--ui-text-primary)}
.sec-sub span{font-size:12px;font-weight:400;color:var(--ui-text-tertiary)}
.sec-sub.is-spaced{margin-top:26px}
.desk-empty{margin-top:12px;padding:26px;border-radius:14px;background:var(--ui-surface-sunken);font-size:13px;color:var(--ui-text-secondary);text-align:center}
.desk-grid{margin-top:12px;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}
.on-desk{min-width:0;border-radius:15px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-line),var(--ui-shadow-card);padding:8px 8px 8px;transition:box-shadow .18s}
.on-desk.located{box-shadow:0 0 0 2px var(--ui-btn),var(--ui-shadow-card)}
.stage{height:170px;aspect-ratio:auto;display:grid;place-items:center}
.stage.is-clickable{cursor:pointer}
.stage.is-clickable:hover{background:color-mix(in srgb,var(--ui-configuration-canvas) 84%,var(--ui-soft))}
.on-desk.hidden :deep(.preview-inner){opacity:.45;filter:grayscale(.6)}
.desk-badges{position:absolute;top:8px;left:8px;right:8px;display:flex;align-items:center;justify-content:space-between;gap:6px;pointer-events:none}
.desk-meta-tag{height:20px;padding:0 7px;border-radius:6px;background:rgba(20,20,24,.54);backdrop-filter:blur(8px);color:#fff;font-size:10.5px;font-weight:500;display:inline-flex;align-items:center}
.hidden-tag{height:20px;padding:0 7px;border-radius:6px;background:rgba(20,20,24,.72);color:#fff;font-size:10.5px;display:inline-flex;align-items:center}
.on-desk .row{display:flex;align-items:center;gap:8px;padding:9px 2px 2px}
.on-desk .row b{flex:1;min-width:0;font-size:13.5px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.on-desk .xb-seg>button{height:22px;padding:0 8px;font-size:12px}
.more-wrap,.add-wrap{position:relative;flex:none}
.menu-dismiss{position:fixed;inset:0;z-index:30}
.more-menu,.layer-menu{position:absolute;right:0;top:calc(100% + 6px);z-index:31;min-width:176px}
.more-menu.is-up,.layer-menu.is-up{top:auto;bottom:calc(100% + 6px)}
.add-section{margin-top:0}
.gallery{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:20px 18px}
.gt{min-width:0;position:relative}
.gt.is-wide{grid-column:span 2}
.gt :deep(.widget-preview.pv){height:172px;aspect-ratio:auto;cursor:pointer;box-shadow:inset 0 0 0 1px var(--ui-line);transition:background .16s,box-shadow .16s}
.gt:hover :deep(.widget-preview.pv){background:color-mix(in srgb,var(--ui-configuration-canvas) 82%,var(--ui-soft));box-shadow:inset 0 0 0 1px var(--ui-border-strong)}
.ft{display:flex;align-items:center;gap:7px;padding:9px 2px 0}
.ft b{flex:1;min-width:0;font-size:13px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.desk-pill{flex:none;height:22px;padding:0 8px;border:0;border-radius:999px;background:var(--ui-soft);color:var(--ui-ink);font-size:11px;font-weight:600;cursor:pointer;white-space:nowrap;transition:filter .14s}
.desk-pill:hover{filter:brightness(.95)}
.desk-pill.is-hidden{background:var(--ui-control);color:var(--ui-text-secondary)}
.add{flex:none;width:28px;height:28px;display:grid;place-items:center;border:0;border-radius:8px;background:var(--ui-soft);color:var(--ui-ink);cursor:pointer;transition:background .14s,color .14s}
.add:hover:not(:disabled){background:var(--ui-btn);color:#fff}
.add.added{background:var(--ui-btn);color:#fff}
.card-tools{position:absolute;top:6px;right:6px;z-index:3;display:flex;gap:4px;opacity:0;transition:opacity .15s}
.gt:hover .card-tools,.card-tools:focus-within{opacity:1}
.widget-derive-btn,.widget-delete-btn{width:26px;height:26px;border-radius:50%;border:0;background:rgba(20,20,24,.54);backdrop-filter:blur(8px);color:#fff;display:grid;place-items:center;cursor:pointer}
.widget-derive-btn:hover{background:var(--ui-btn)}
.widget-delete-btn:hover{background:var(--ui-danger)}
.wd-hint{margin:20px 0 0;font-size:12px;color:var(--ui-text-tertiary);text-align:center}
@container(max-width:860px){.gallery{grid-template-columns:repeat(2,minmax(0,1fr))}.gt.is-wide{grid-column:span 2}.desk-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
@container(max-width:520px){.gallery{grid-template-columns:1fr}.gt.is-wide{grid-column:span 1}.desk-grid{grid-template-columns:1fr}}
.gt button:focus-visible{outline:2px solid var(--ui-btn);outline-offset:3px}
</style>
