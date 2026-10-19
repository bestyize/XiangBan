<script setup lang="ts">
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, watch, onMounted, onBeforeUnmount, ref } from 'vue';
import { Check, Plus, Search, Sparkles, Trash2 } from 'lucide-vue-next';
import { NAlert, NButton, NInput, NModal, NPopconfirm, NSelect, NSlider, NSwitch } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { createWidgetClient, searchWidgetCities } from '../../widgets/client.ts';
import { baseSize, widgetKinds, widgetThemes, type WidgetKind, type DesktopWidget, type City, type CustomWidgetData } from '../../widgets/model.ts';
import WidgetSurface from '../../widgets/WidgetSurface.vue';
import NotesWidget from '../../widgets/NotesWidget.vue';
import CountdownEditor from '../../widgets/CountdownEditor.vue';
import type { WidgetContent } from '../../widgets/model.ts';
import { emptyNote, emptyCountdown, emptyFlower } from '../../widgets/content.ts';
const props=defineProps<{store:ControlStore;settingsRequest?:{id:string}|null}>();
const emit=defineEmits<{settingsOpened:[]}>();
const client=createWidgetClient(),{layout,ready,busy,error}=client;
const MAX_WIDGETS=12;
const selected=ref(''),settingsOpen=ref(false),cityQuery=ref(''),cities=ref<City[]>([]),cityBusy=ref(false);
const chosen=computed(()=>layout.value.widgets.find(widget=>widget.id===selected.value)??null);
const full=computed(()=>layout.value.widgets.length>=MAX_WIDGETS);
const customTemplates=ref<any[]>([]);
async function loadCustomTemplates(){
  if(!props.store.desktop)return;
  try{customTemplates.value=await invoke<any[]>('list_custom_widgets');}catch(e){}
}
async function deleteCustomTemplate(id:string,name:string){
  await action(async()=>{
    await invoke('remove_custom_widget',{id});
    props.store.notify(`已删除小组件模板「${name}」`,'success');
    await loadCustomTemplates();
    await client.load();
  });
}
const displays=computed(()=>layout.value.monitors?.length?layout.value.monitors:[{id:'primary',name:'主屏幕',width:layout.value.desktop?.width??Math.max(1280,screen.width),height:layout.value.desktop?.height??Math.max(800,screen.height),primary:true}]);
const primaryDisplay=computed(()=>displays.value.find(m=>m.primary)??displays.value[0]);
const monitorOptions=computed(()=>displays.value.map(m=>({label:`${m.name}${m.primary?' · 主屏幕':''}`,value:m.id})));
function monitorOf(widget:DesktopWidget){return displays.value.find(m=>m.id===widget.monitor)??primaryDisplay.value;}
const titleOf=(widget:DesktopWidget)=>widget.kind==='custom'?widget.custom?.name||'自创组件':widgetKinds.find(item=>item.kind===widget.kind)?.title??'';
// Library previews sit in a 16/9 stage the width of the card, so every demo widget shrinks to fit it.
const libraryGrid=ref<HTMLElement>();
const stageWidth=ref(0);
let libraryObserver:ResizeObserver|undefined;
function measureStage(){const column=libraryGrid.value?getComputedStyle(libraryGrid.value).gridTemplateColumns.split(' ')[0]:'';const width=parseFloat(column??'');if(Number.isFinite(width)&&width>0)stageWidth.value=width;}
const demoScale=(kind:WidgetKind,custom?:CustomWidgetData)=>{const{width,height}=baseSize(kind,custom);const stage=stageWidth.value||210;return Math.min(1,stage/width,stage*9/16/height);};
const demo=(kind:WidgetKind):DesktopWidget=>({id:`demo${kind}`,kind,theme:'transparent',scale:1,x:0,y:0,enabled:true,city:null,...(kind==='notes'?{note:{...emptyNote(),mode:'todo',items:[{id:'task1',text:'完成产品方案',done:true},{id:'task2',text:'健身 30 分钟',done:false},{id:'task3',text:'读一本喜欢的书',done:false}]}}:kind==='reminders'?{living:{count:0,symbol:'1.000001',reminders:[{id:'a1',title:'团队周会',at:'2026-10-19T09:30',done:false},{id:'a2',title:'设计评审',at:'2026-10-19T15:00',done:false}]}}:kind==='woodfish'?{living:{count:108,symbol:'1.000001',reminders:[]}}:kind==='countdown'?{countdown:{...emptyCountdown(),title:'下一次旅行',date:'2026-12-31'}}:kind==='flower'?{flower:{...emptyFlower(),water:12}}:{})});
async function action(task:()=>Promise<unknown>){try{await task();}catch(e){props.store.notify(e,'error');}}
// A brief check mark on the "+" badge confirms the add without moving the user away from the library.
const flashKey=ref('');let flashTimer:ReturnType<typeof setTimeout>|undefined;
function flash(key:string){flashKey.value=key;clearTimeout(flashTimer);flashTimer=setTimeout(()=>{flashKey.value='';},1600);}
async function addWidget(key:string,create:()=>Promise<unknown>,needsSetup:boolean){
  await action(async()=>{
    await create();
    flash(key);
    const added=layout.value.widgets.at(-1);
    if(layout.value.errors.length)props.store.notify('组件已保存，桌面显示遇到问题，请查看页面提示','error');
    else if(needsSetup&&added){props.store.notify('已添加到桌面，先选一座城市吧','success');openSettings(added.id);}
    else props.store.notify('已添加到桌面，按住组件顶部即可拖动','success');
  });
}
const add=(kind:WidgetKind)=>addWidget(`kind:${kind}`,()=>client.add(kind),kind==='weather');
const addCustom=(template:any)=>addWidget(`custom:${template.id}`,()=>client.add('custom',template.id),false);
function update(patch:Partial<DesktopWidget>){if(chosen.value)void action(()=>client.update(chosen.value!.id,patch));}
async function saveChosenContent(content:WidgetContent){if(chosen.value)await client.update(chosen.value.id,content);}
function openSettings(id:string){selected.value=id;cities.value=[];cityQuery.value='';settingsOpen.value=true;}
function closeSettings(){settingsOpen.value=false;void commitScale();}
const scaleDraft=ref<{id:string;value:number}|null>(null);let scaleTimer:ReturnType<typeof setTimeout>|undefined;
const scaleOf=(widget:DesktopWidget)=>scaleDraft.value?.id===widget.id?scaleDraft.value.value:widget.scale;
function commitScale(){clearTimeout(scaleTimer);const draft=scaleDraft.value;if(!draft)return Promise.resolve();return action(()=>client.update(draft.id,{scale:draft.value})).finally(()=>{if(scaleDraft.value===draft)scaleDraft.value=null;});}
function setScale(value:number){if(!chosen.value)return;scaleDraft.value={id:chosen.value.id,value};clearTimeout(scaleTimer);scaleTimer=setTimeout(()=>void commitScale(),250);}
// The settings preview is fitted to the largest allowed scale, so dragging the size slider visibly grows the widget.
const PREVIEW_W=400,PREVIEW_H=220,MAX_SCALE=1.5;
function previewScale(widget:DesktopWidget){const{width,height}=baseSize(widget.kind,widget.custom);return Math.min(1,PREVIEW_W/(width*MAX_SCALE),PREVIEW_H/(height*MAX_SCALE))*scaleOf(widget);}
async function removeChosen(){
  const id=selected.value;if(!id)return;
  await action(async()=>{await client.remove(id);settingsOpen.value=false;setTimeout(()=>{if(!settingsOpen.value)selected.value='';},300);});
}
async function search(){if(!cityQuery.value.trim()||cityBusy.value)return;cityBusy.value=true;try{cities.value=await searchWidgetCities(cityQuery.value.trim());if(!cities.value.length)props.store.notify('没有找到城市，可以试试完整名称','info');}catch(e){props.store.notify(e,'error');}finally{cityBusy.value=false;}}
function chooseCity(index:number){const city=cities.value[index];if(city)update({city});}
const cityOptions=computed(()=>cities.value.map((city,index)=>({label:[city.name,city.admin1,city.country].filter(Boolean).join(' · '),value:index})));
watch([()=>props.settingsRequest,ready],([request,loaded])=>{
  if(!request||!loaded)return;
  if(layout.value.widgets.some(widget=>widget.id===request.id))openSettings(request.id);
  else props.store.notify('这个组件已不在桌面上','info');
  emit('settingsOpened');
},{immediate:true});
watch(ready,loaded=>{if(loaded&&layout.value.editing)void action(()=>client.edit(false));});
watch(libraryGrid,element=>{libraryObserver?.disconnect();if(element){libraryObserver=new ResizeObserver(measureStage);libraryObserver.observe(element);measureStage();}});
onMounted(()=>{void client.load();void loadCustomTemplates();});onBeforeUnmount(()=>{clearTimeout(flashTimer);void commitScale();if(layout.value.editing)void client.edit(false).catch(e=>props.store.notify(e,'error'));libraryObserver?.disconnect();client.dispose();});
</script>
<template>
  <section class="ui-page widgets-page" data-testid="widgets-page">
    <div class="widgets-top">
      <header class="page-header">
        <div class="widget-title"><h1>桌面组件</h1><span class="widget-count">{{ layout.widgets.length }} / {{ MAX_WIDGETS }} 个组件</span></div>
        <div class="widget-header-actions">
          <ResourceShareActions :store="store" kind="widget" import-label="导入小组件" @imported="loadCustomTemplates"/>
          <NButton type="primary" round :disabled="!store.desktop" data-testid="widgets-create" @click="store.startCreation('widget')"><template #icon><Sparkles :size="15"/></template>创作小组件</NButton>
        </div>
      </header>
    </div>
    <div class="widgets-body">
      <NAlert v-if="error" type="error" :show-icon="false">{{ error }}<NButton v-if="!ready" size="tiny" text @click="client.load">重新读取</NButton></NAlert>
      <NAlert v-if="layout.errors.length" type="warning" :show-icon="false">{{ layout.errors.join('；') }}</NAlert>

      <section v-if="layout.widgets.length" class="widget-section" data-testid="desktop-widgets">
        <div class="section-head"><h2>已在桌面</h2><p>点一张卡片，调整它的样式、大小和内容。</p></div>
        <div class="widget-library">
          <button v-for="widget in layout.widgets" :key="widget.id" type="button" class="widget-library-card desktop-card" :class="{off:!widget.enabled}" :data-testid="`widget-card-${widget.id}`" :aria-label="`调整${titleOf(widget)}`" @click="openSettings(widget.id)">
            <div class="widget-art" :class="widget.kind"><div class="demo-widget" :style="{'--demo-scale':demoScale(widget.kind,widget.custom)}"><WidgetSurface :widget="widget" preview/></div><span v-if="!widget.enabled" class="off-badge">已隐藏</span></div>
            <div class="widget-card-copy"><span class="widget-category">{{ monitorOf(widget).name }}</span><h2>{{ titleOf(widget) }}</h2></div>
          </button>
        </div>
      </section>

      <section class="widget-section">
        <div class="section-head"><h2>组件库</h2><p>点右下角的 + 放到桌面，同一种组件可以添加多个。</p></div>
        <div class="widget-library" ref="libraryGrid">
          <article v-for="item in widgetKinds" :key="item.kind" class="widget-library-card">
            <div class="widget-art" :class="item.kind"><div class="demo-widget" :class="item.kind" :style="{'--demo-scale':demoScale(item.kind)}"><WidgetSurface :widget="demo(item.kind)" preview/></div><button class="widget-add-badge" :disabled="!ready||busy||full" :data-testid="`add-${item.kind}`" :title="full?`桌面已放满 ${MAX_WIDGETS} 个`:'添加到桌面'" :aria-label="full?`桌面已放满 ${MAX_WIDGETS} 个`:`添加${item.title}到桌面`" @click="add(item.kind)"><Check v-if="flashKey===`kind:${item.kind}`" :size="15"/><Plus v-else :size="15"/></button></div>
            <div class="widget-card-copy"><span class="widget-category">{{ item.label }}</span><h2>{{ item.title }}</h2><p>{{ item.description }}</p></div>
          </article>
        </div>
      </section>

      <section v-if="customTemplates.length" class="widget-section">
        <div class="section-head"><h2>自创组件</h2><p>由小伴现场创作或导入的专属桌面小部件。</p></div>
        <div class="widget-library">
          <article v-for="item in customTemplates" :key="item.id" class="widget-library-card">
            <div class="widget-art custom">
              <div class="demo-widget custom-demo" :style="{'--demo-scale':demoScale('custom',item)}">
                <WidgetSurface :widget="{ id: 'demo' + item.id, kind: 'custom', customId: item.id, custom: item, theme: item.theme || 'transparent', scale: 1, x: 0, y: 0, enabled: true, city: null }" preview/>
              </div>
              <button class="widget-add-badge" :disabled="!ready||busy||full" :title="full?`桌面已放满 ${MAX_WIDGETS} 个`:'添加到桌面'" :aria-label="full?`桌面已放满 ${MAX_WIDGETS} 个`:`添加${item.name}到桌面`" @click="addCustom(item)"><Check v-if="flashKey===`custom:${item.id}`" :size="15"/><Plus v-else :size="15"/></button>
            </div>
            <div class="widget-card-copy">
              <div class="custom-card-header">
                <span class="widget-category">自创组件 · {{ item.width || 280 }}×{{ item.height || 240 }}</span>
                <NPopconfirm positive-text="删除" negative-text="取消" @positive-click="deleteCustomTemplate(item.id, item.name)">
                  <template #trigger>
                    <button class="custom-delete-btn" title="删除这个小组件模板" :aria-label="`删除小组件 ${item.name}`"><Trash2 :size="13"/></button>
                  </template>
                  确定要删除小组件模板「{{ item.name }}」吗？若桌面上已有摆放也将同步移除。
                </NPopconfirm>
              </div>
              <h2>{{ item.name }}</h2>
              <p>{{ item.description || '自包含单文件 HTML 桌面小组件' }}</p>
              <ResourceShareActions :store="store" kind="widget" :id="item.id"/>
            </div>
          </article>
        </div>
      </section>

      <p class="widget-hint">{{ store.info.platform==='windows'?'按 Win+D 可以查看桌面上的组件。':'' }} 时钟、待办、提醒、木鱼及原有离线组件不需要联网；天气与股票行情需要网络连接。按住组件顶部拖动，点右上角齿轮调整设置。位置和内容保存在本机，重启后恢复；关闭小伴桌面时，组件也会关闭。</p>
    </div>

    <NModal :show="settingsOpen" preset="card" :title="chosen?titleOf(chosen):'组件设置'" :mask-closable="true" style="max-width:480px;width:92vw" content-style="max-height:60vh;overflow-y:auto" data-testid="widget-settings" @update:show="value=>{if(!value)closeSettings();}">
      <div v-if="chosen" class="settings-body">
        <div class="settings-stage widget-art" :class="chosen.kind"><div class="demo-widget" :style="{'--demo-scale':previewScale(chosen)}"><WidgetSurface :widget="chosen" preview/></div></div>
        <label class="widget-field inline"><span>显示在桌面</span><NSwitch :value="chosen.enabled" :disabled="busy" @update:value="update({enabled:$event})"/></label>
        <div class="widget-field"><span>外观样式</span><NSelect :value="chosen.theme" :options="widgetThemes" :disabled="busy" @update:value="update({theme:$event})"/></div>
        <div class="widget-field"><span>大小（等比） <small>×{{ scaleOf(chosen).toFixed(2) }}</small></span><NSlider :value="scaleOf(chosen)" :min=".75" :max="MAX_SCALE" :step=".05" :disabled="busy" @update:value="setScale"/></div>
        <div v-if="displays.length>1" class="widget-field"><span>所在显示器</span><NSelect :value="monitorOf(chosen).id" :options="monitorOptions" :disabled="busy" aria-label="组件所在显示器" @update:value="update({monitor:$event})"/></div>
        <NotesWidget v-if="chosen.kind==='notes'" :key="chosen.id" :note="chosen.note" :save="saveChosenContent"/>
        <CountdownEditor v-else-if="chosen.kind==='countdown'" :key="chosen.id" :data="chosen.countdown" :save="saveChosenContent"/>
        <template v-else-if="chosen.kind==='weather'">
          <div class="widget-field"><span>天气城市</span><small v-if="chosen.city" class="current-city">当前：{{ chosen.city.name }} {{ chosen.city.admin1 }}</small><div class="city-search"><NInput v-model:value="cityQuery" placeholder="输入城市名称" @keydown.enter="search"/><NButton :loading="cityBusy" :disabled="!cityQuery.trim()" aria-label="搜索城市" @click="search"><Search :size="14"/></NButton></div><NSelect v-if="cities.length" :options="cityOptions" placeholder="选择搜索结果" :disabled="busy" @update:value="chooseCity"/></div>
          <p class="inspector-note">天气沿用“关怀提醒”的天气来源。每半小时更新一次；可随时在组件上手动刷新。</p>
        </template>
        <p v-else-if="chosen.kind==='fortune'" class="inspector-note">点击签筒求签，揭晓后可以展开好签册收藏。</p>
        <p v-else-if="chosen.kind==='calendar'" class="inspector-note">显示本机时间与农历。点击日期查看详情；休／班标识使用已收录的官方年度安排。点击月份回到今天。</p>
        <p v-else-if="chosen.kind==='flower'" class="inspector-note">在组件上点“浇水”，小白花会经历嫩芽、舒展、含苞、初绽、盛放到满庭六个阶段。进度只保存在这台设备，想重新开始可以移除后再添加一个。</p>
      </div>
      <template #footer>
        <div class="settings-footer">
          <NPopconfirm positive-text="移除" negative-text="取消" @positive-click="removeChosen"><template #trigger><NButton quaternary :disabled="busy||!chosen" aria-label="从桌面移除这个组件" data-testid="remove-widget"><template #icon><Trash2 :size="15"/></template>从桌面移除</NButton></template>移除后，这个组件的内容、样式和设置会一并清除。</NPopconfirm>
          <NButton type="primary" round data-testid="widget-settings-done" @click="closeSettings">完成</NButton>
        </div>
      </template>
    </NModal>
  </section>
</template>
<style scoped>
.widgets-page{display:flex;flex-direction:column;flex:1;min-height:0;width:100%;gap:0}
.widgets-top{flex:none;display:grid;gap:14px;background:var(--ui-app-bg);padding:var(--ui-page-top) var(--ui-page-inset) 14px;border-bottom:1px solid var(--ui-border)}
.widgets-top .page-header{margin-bottom:0}
.widgets-body{flex:1;min-height:0;overflow-y:auto;padding:var(--ui-section-gap) var(--ui-page-inset) 32px;display:grid;gap:calc(var(--ui-section-gap) + 8px);align-content:start}
.widget-title{display:flex;align-items:center;gap:12px}
.widget-count{font-size:12px;color:var(--ui-text-tertiary);white-space:nowrap}
.widget-header-actions{display:flex;align-items:center;justify-content:flex-end;gap:10px;flex-wrap:wrap}
.widget-section{display:grid;gap:14px}
.section-head{display:flex;align-items:baseline;gap:12px;padding-left:12px;border-left:2px solid #bba477}
.section-head h2{font-size:16px;font-weight:650;margin:0;color:var(--ui-text-primary)}
.section-head p{font-size:12px;color:var(--ui-text-secondary);margin:0}
.widget-library{display:grid;grid-template-columns:repeat(auto-fill,minmax(210px,1fr));gap:var(--ui-card-gap)}
.widget-library-card{border:1px solid var(--ui-border);background:var(--ui-surface);border-radius:var(--ui-radius-card);overflow:hidden;box-shadow:0 6px 20px #10252c0a}
.desktop-card{display:block;width:100%;padding:0;text-align:left;font:inherit;color:inherit;cursor:pointer;transition:transform .2s,box-shadow .2s,border-color .2s}
.desktop-card:hover{transform:translateY(-2px);border-color:#af9468;box-shadow:0 10px 26px #10252c14}
.desktop-card.off .widget-art{opacity:.55}
.desktop-card .demo-widget{pointer-events:none}
.off-badge{position:absolute;left:10px;top:10px;z-index:3;padding:2px 8px;border-radius:10px;background:#0d2a2fb8;color:#f0e6c8;font-size:10px}
.widget-art{aspect-ratio:16/9;position:relative;background:radial-gradient(ellipse at 45% 40%,#e1d5ba,#c7b390);overflow:hidden}
.widget-art.notes{background:radial-gradient(ellipse at 40% 30%,#e4d8b6,#a99a78)}
.widget-art.countdown{background:radial-gradient(ellipse at 70% 30%,#b293a0,#644f65)}
.widget-art.calendar{background:radial-gradient(ellipse at 70% 30%,#698c86,#243f46)}
.widget-art.weather{background:radial-gradient(ellipse at 65% 10%,#7e9fa8,#344e62)}
.widget-art.custom{background:radial-gradient(ellipse at 50% 30%,#b5c9c3,#3d6168)}
.widget-art.flower{background:radial-gradient(ellipse at 50% 22%,#53866f,#2f5d52 55%,#1c3a37)}
.custom-card-header{display:flex;justify-content:space-between;align-items:center}
.custom-delete-btn{border:0;background:none;color:var(--ui-text-tertiary);cursor:pointer;padding:4px;border-radius:4px;display:flex;align-items:center;transition:color .2s}
.custom-delete-btn:hover{color:#dc2626}
.demo-widget{position:absolute;left:50%;top:50%;transform:translate(-50%,-50%) scale(var(--demo-scale,1))}
.demo-widget :deep(.widget-heading){display:none}
.widget-card-copy{display:grid;gap:2px;padding:12px 16px 14px}
.widget-category{font-size:11px;letter-spacing:0;color:var(--ui-text-tertiary)}
.widget-card-copy h2{font:15px/1.3 'Microsoft YaHei','Microsoft YaHei UI',sans-serif;letter-spacing:0;font-weight:650;margin:2px 0 0}
.widget-card-copy p{font-size:12px;color:var(--ui-text-secondary);margin:0;line-height:1.5;display:-webkit-box;-webkit-line-clamp:1;line-clamp:1;-webkit-box-orient:vertical;overflow:hidden}
.widget-add-badge{position:absolute;right:10px;bottom:10px;z-index:3;width:30px;height:30px;display:grid;place-items:center;padding:0;border:0;border-radius:50%;background:var(--ui-primary);color:#fff;cursor:pointer;box-shadow:0 4px 14px var(--ui-shadow-hover);transition:transform .2s,background .2s}
.widget-add-badge:hover:not(:disabled){transform:scale(1.08)}
.widget-card-copy .resource-share-actions{margin-top:8px}
.widget-hint{font-size:12px;line-height:1.8;color:var(--ui-text-tertiary);margin:0}
.settings-body{display:grid;gap:18px}
.settings-stage{aspect-ratio:auto;height:220px;border-radius:12px}
.settings-body .widget-field{margin-top:0}
.widget-field{display:grid;gap:10px}
.widget-field>span{font-size:12px;display:flex;justify-content:space-between}
.widget-field small{font-size:11px;color:var(--ui-text-tertiary)}
.widget-field.inline{display:flex;justify-content:space-between;align-items:center}
.city-search{display:flex;gap:6px}
.inspector-note{font-size:11px;line-height:1.9;color:var(--ui-text-tertiary);margin:0}
.settings-footer{display:flex;justify-content:space-between;align-items:center;gap:12px}
button:disabled{opacity:.4;cursor:default}
button:focus-visible{outline:2px solid #af9468;outline-offset:3px}
@media(prefers-reduced-motion:reduce){.desktop-card{transition:none}.desktop-card:hover{transform:none}}
@media(max-width:720px){.section-head{flex-direction:column;gap:4px}}
</style>
