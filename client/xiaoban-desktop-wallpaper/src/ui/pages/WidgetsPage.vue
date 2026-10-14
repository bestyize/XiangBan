<script setup lang="ts">
import { computed, watch, onMounted, onBeforeUnmount, ref } from 'vue';
import { Check, Eye, LayoutGrid, Move, Plus, Search, Trash2 } from 'lucide-vue-next';
import { NAlert, NButton, NInput, NPopconfirm, NSelect, NSlider, NSwitch } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { createWidgetClient, searchWidgetCities } from '../../widgets/client.ts';
import { baseSize, widgetKinds, widgetThemes, type WidgetKind, type DesktopWidget, type City } from '../../widgets/model.ts';
import WidgetSurface from '../../widgets/WidgetSurface.vue';
import NotesWidget from '../../widgets/NotesWidget.vue';
import CountdownEditor from '../../widgets/CountdownEditor.vue';
import type { WidgetContent } from '../../widgets/model.ts';
import { emptyNote, emptyCountdown } from '../../widgets/content.ts';
import InteractivePage from './InteractivePage.vue';
const props=defineProps<{store:ControlStore}>();
const client=createWidgetClient(),{layout,ready,busy,error}=client;
const tab=ref<'library'|'desktop'|'scenes'>('library'),selected=ref(''),cityQuery=ref(''),cities=ref<City[]>([]),cityBusy=ref(false);
const chosen=computed(()=>layout.value.widgets.find(widget=>widget.id===selected.value)??null);
const enabled=computed(()=>layout.value.widgets.filter(widget=>widget.enabled));
const customTemplates=ref<any[]>([]);
async function loadCustomTemplates(){
  if(!props.store.desktop)return;
  try{customTemplates.value=await invoke<any[]>('list_custom_widgets');}catch(e){}
}
async function addCustom(template:any){
  await action(async()=>{
    await client.add('custom',template.id);
    selected.value=layout.value.widgets.at(-1)?.id??'';
    tab.value='desktop';
    props.store.notify(`已添加「${template.name}」到桌面`,'success');
  });
}
async function deleteCustomTemplate(id:string,name:string){
  await action(async()=>{
    await invoke('remove_custom_widget',{id});
    props.store.notify(`已删除小组件模板「${name}」`,'success');
    await loadCustomTemplates();
    await client.load();
  });
}
const monitorOptions=computed(()=>displays.value.map(m=>({label:`${m.name}${m.primary?' · 主屏幕':''}`,value:m.id})));
const previewMonitor=ref('');
const displays=computed(()=>layout.value.monitors?.length?layout.value.monitors:[{id:'primary',name:'主屏幕',width:layout.value.desktop?.width??Math.max(1280,screen.width),height:layout.value.desktop?.height??Math.max(800,screen.height),primary:true}]);
const primaryDisplay=computed(()=>displays.value.find(m=>m.primary)??displays.value[0]);
const activeDisplay=computed(()=>displays.value.find(m=>m.id===previewMonitor.value)??primaryDisplay.value);
function monitorOf(widget:DesktopWidget){return displays.value.find(m=>m.id===widget.monitor)??primaryDisplay.value;}
const visibleWidgets=computed(()=>enabled.value.filter(w=>monitorOf(w).id===activeDisplay.value.id));
const previewWallpaper=computed(()=>props.store.library.items.find(item=>item.definition.id===props.store.settings.wallpaperId)?.coverUrl??'');
const TOP=32,BOTTOM=88,PAD=8;
const demo=(kind:WidgetKind):DesktopWidget=>({id:`demo${kind}`,kind,theme:kind==='fortune'?'paper':'glass',scale:1,x:0,y:0,enabled:true,city:null,...(kind==='notes'?{note:{...emptyNote(),text:'慢慢来，也会抵达。\n\n把今天的小事，放在这里。'}}:kind==='countdown'?{countdown:{...emptyCountdown(),title:'下一次旅行',date:'2026-12-31'}}:{})});
async function action(task:()=>Promise<unknown>){try{await task();}catch(e){props.store.notify(e,'error');}}
async function add(kind:WidgetKind){await action(async()=>{await client.add(kind);selected.value=layout.value.widgets.at(-1)?.id??'';tab.value='desktop';if(layout.value.errors.length)props.store.notify('组件已保存，桌面显示遇到问题，请查看页面提示','error');else props.store.notify('已添加到桌面，可以直接拖动摆放','success');});}
function update(patch:Partial<DesktopWidget>){if(chosen.value)void action(()=>client.update(chosen.value!.id,patch));}
async function saveChosenContent(content:WidgetContent){if(chosen.value)await client.update(chosen.value.id,content);}
function select(id:string){const widget=layout.value.widgets.find(w=>w.id===id);if(widget)previewMonitor.value=monitorOf(widget).id;if(selected.value===id)return;selected.value=id;cities.value=[];cityQuery.value='';}
const scaleDraft=ref<{id:string;value:number}|null>(null);let scaleTimer:ReturnType<typeof setTimeout>|undefined;
const scaleOf=(widget:DesktopWidget)=>scaleDraft.value?.id===widget.id?scaleDraft.value.value:widget.scale;
function commitScale(){clearTimeout(scaleTimer);const draft=scaleDraft.value;if(!draft)return Promise.resolve();return action(()=>client.update(draft.id,{scale:draft.value})).finally(()=>{if(scaleDraft.value===draft)scaleDraft.value=null;});}
function setScale(value:number){if(!chosen.value)return;scaleDraft.value={id:chosen.value.id,value};clearTimeout(scaleTimer);scaleTimer=setTimeout(()=>void commitScale(),250);}
async function remove(id:string){await action(async()=>{await client.remove(id);if(selected.value===id)selected.value=layout.value.widgets[0]?.id??'';});}
async function search(){if(!cityQuery.value.trim()||cityBusy.value)return;cityBusy.value=true;try{cities.value=await searchWidgetCities(cityQuery.value.trim());if(!cities.value.length)props.store.notify('没有找到城市，可以试试完整名称','info');}catch(e){props.store.notify(e,'error');}finally{cityBusy.value=false;}}
function chooseCity(index:number){const city=cities.value[index];if(city)update({city});}
const cityOptions=computed(()=>cities.value.map((city,index)=>({label:[city.name,city.admin1,city.country].filter(Boolean).join(' · '),value:index})));
async function tuckAway(){try{await invoke('hide_control');}catch(e){props.store.notify(e,'error');}}
const canvas=ref<HTMLElement>();
const previewScale=ref(.4);
const desktopSize=computed(()=>({width:activeDisplay.value.width,height:activeDisplay.value.height}));
function targetMonitor(id:string){previewMonitor.value=id;update({monitor:id});}
let resizeObserver:ResizeObserver|undefined;
function fitPreview(){const rect=canvas.value?.getBoundingClientRect();if(rect&&rect.width)previewScale.value=rect.width/desktopSize.value.width;}
const windowSize=(widget:DesktopWidget)=>{const size=baseSize(widget.kind),scale=scaleOf(widget);return {width:size.width*scale+16,height:size.height*scale+16};};
let drag:{id:string;pointerX:number;pointerY:number;x:number;y:number;width:number;height:number;moved:boolean}|null=null;
const draft=ref<Record<string,{x:number;y:number}>>({});
function position(widget:DesktopWidget){const point=draft.value[widget.id]??widget,box=windowSize(widget),k=previewScale.value;return {left:`${(PAD+point.x*Math.max(0,desktopSize.value.width-box.width))*k}px`,top:`${(TOP+PAD+point.y*Math.max(0,desktopSize.value.height-box.height-BOTTOM))*k}px`};}
function begin(event:PointerEvent,widget:DesktopWidget){select(widget.id);if(event.button!==0||busy.value||!canvas.value)return;const box=windowSize(widget),k=previewScale.value;drag={id:widget.id,pointerX:event.clientX,pointerY:event.clientY,x:widget.x,y:widget.y,width:Math.max(1,(desktopSize.value.width-box.width)*k),height:Math.max(1,(desktopSize.value.height-box.height-BOTTOM)*k),moved:false};(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);}
function move(event:PointerEvent){if(!drag)return;const dx=event.clientX-drag.pointerX,dy=event.clientY-drag.pointerY;if(!drag.moved&&Math.hypot(dx,dy)<=5)return;drag.moved=true;draft.value[drag.id]={x:Math.max(0,Math.min(1,drag.x+dx/drag.width)),y:Math.max(0,Math.min(1,drag.y+dy/drag.height))};}
async function finish(event:PointerEvent){if(!drag)return;move(event);const id=drag.id,point=draft.value[id];drag=null;if(point)await action(()=>client.update(id,point));delete draft.value[id];}
function nudge(event:KeyboardEvent,widget:DesktopWidget){const step=event.shiftKey?.05:.01,delta={ArrowLeft:[-step,0],ArrowRight:[step,0],ArrowUp:[0,-step],ArrowDown:[0,step]}[event.key as 'ArrowLeft'];if(!delta||busy.value)return;event.preventDefault();void action(()=>client.update(widget.id,{x:Math.max(0,Math.min(1,widget.x+delta[0])),y:Math.max(0,Math.min(1,widget.y+delta[1]))}));}
watch([tab,()=>layout.value.widgets],()=>{if(tab.value=='desktop'&&!chosen.value&&layout.value.widgets.length)select(layout.value.widgets[0].id);},{immediate:true});
watch(tab,(next,previous)=>{if(previous==='desktop'&&next!=='desktop'&&layout.value.editing)void action(()=>client.edit(false));});
watch(canvas,element=>{resizeObserver?.disconnect();if(element){resizeObserver=new ResizeObserver(fitPreview);resizeObserver.observe(element);fitPreview();}});
watch(desktopSize,fitPreview);
onMounted(()=>{void client.load();void loadCustomTemplates();});onBeforeUnmount(()=>{void commitScale();if(layout.value.editing)void client.edit(false).catch(e=>props.store.notify(e,'error'));resizeObserver?.disconnect();client.dispose();});
</script>
<template>
  <section class="ui-page widgets-page" data-testid="widgets-page">
    <header class="page-header"><div><h1>桌面组件</h1><p class="page-subtitle">壁纸是风景，小组件是生活。把喜欢的小物件，留在你的桌面上。</p></div><span class="widget-count">{{ layout.widgets.length }} / 12 个组件</span></header>
    <nav class="widget-tabs" role="tablist" aria-label="桌面组件分区"><button role="tab" :aria-selected="tab==='library'" :class="{on:tab==='library'}" @click="tab='library'">组件库</button><button role="tab" :aria-selected="tab==='desktop'" :class="{on:tab==='desktop'}" @click="tab='desktop'">我的桌面<span v-if="layout.widgets.length">{{ layout.widgets.length }}</span></button><button role="tab" :aria-selected="tab==='scenes'" :class="{on:tab==='scenes'}" @click="tab='scenes'">互动场景</button></nav>
    <NAlert v-if="error" type="error" :show-icon="false">{{ error }}<NButton v-if="!ready" size="tiny" text @click="client.load">重新读取</NButton></NAlert>
    <NAlert v-if="layout.errors.length" type="warning" :show-icon="false">{{ layout.errors.join('；') }}</NAlert>
    <template v-if="tab==='library'">
      <div class="widget-library-intro"><span>小小一隅，也有好日子</span><p>自由添加、拖动和搭配。换一张壁纸，小物件依然留在原处。</p></div>
      <div class="widget-library">
        <article v-for="(item,index) in widgetKinds" :key="item.kind" class="widget-library-card">
          <div class="widget-art" :class="item.kind"><span class="widget-index">0{{ index+1 }} / DESKTOP OBJECTS</span><div class="demo-widget" :class="item.kind"><WidgetSurface :widget="demo(item.kind)" preview/></div></div>
          <div class="widget-card-copy"><span class="widget-category">{{ item.label }} · 可添加多个</span><h2>{{ item.title }}</h2><p>{{ item.description }}</p><button :disabled="!ready||busy||layout.widgets.length>=12" :data-testid="`add-${item.kind}`" class="widget-add" @click="add(item.kind)"><Plus :size="14"/>{{ layout.widgets.length>=12?'桌面已放满 12 个':'添加到桌面' }}</button></div>
        </article>
      </div>
      <template v-if="customTemplates.length">
        <div class="widget-library-intro custom-intro"><span>巧思妙想 · 自创组件</span><p>由小伴现场创作或导入的专属桌面小部件，可自由摆放与清理。</p></div>
        <div class="widget-library">
          <article v-for="item in customTemplates" :key="item.id" class="widget-library-card">
            <div class="widget-art custom">
              <span class="widget-index">CUSTOM OBJECT</span>
              <div class="demo-widget custom-demo">
                <WidgetSurface :widget="{ id: 'demo' + item.id, kind: 'custom', customId: item.id, custom: item, theme: item.theme || 'glass', scale: 1, x: 0, y: 0, enabled: true, city: null }" preview/>
              </div>
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
              <button :disabled="!ready||busy||layout.widgets.length>=12" class="widget-add" @click="addCustom(item)"><Plus :size="14"/>{{ layout.widgets.length>=12?'桌面已放满 12 个':'添加到桌面' }}</button>
            </div>
          </article>
        </div>
      </template>
      <p class="widget-hint">轻点组件使用功能，按住拖动即可摆放，位置会自动保存。求签、日历、便签和纪念日可离线使用，天气需要选择城市并联网。</p>
    </template>
    <template v-else-if="tab==='desktop'"><div class="desktop-toolbar"><div><h2>你的桌面布局</h2><p>{{ layout.editing?'真实桌面上的组件已锁定功能，按住即可拖动，松手自动保存。摆好后点“完成摆放”。':'拖动下方预览即可摆放，位置与真实桌面一致。选中后也可用方向键微调。' }}</p></div><div class="desktop-tools"><NSelect v-if="displays.length>1" :value="activeDisplay.id" :options="monitorOptions" aria-label="预览显示器" style="width:180px" @update:value="previewMonitor=$event"/><NButton v-if="store.desktop" round @click="tuckAway"><template #icon><Eye :size="14"/></template>收起面板</NButton><NButton round :type="layout.editing?'primary':'default'" :disabled="!ready||busy" data-testid="edit-widgets" @click="action(()=>client.edit(!layout.editing))"><template #icon><Check v-if="layout.editing" :size="14"/><Move v-else :size="14"/></template>{{ layout.editing?'完成摆放':'在桌面上摆放' }}</NButton></div></div><div class="desktop-editor"><div ref="canvas" class="desktop-canvas" :class="{editing:layout.editing}" :style="{aspectRatio:`${desktopSize.width} / ${desktopSize.height}`,'--inset-top':`${TOP*previewScale}px`,'--inset-bottom':`${BOTTOM*previewScale}px`,...(previewWallpaper&&activeDisplay.primary?{backgroundImage:`linear-gradient(0deg,#071f2933,#071f2922),url(${JSON.stringify(previewWallpaper)})`}:{})}"><span class="canvas-label">DESKTOP · {{ activeDisplay.name }}</span><div v-for="widget in visibleWidgets" :key="widget.id" class="canvas-widget" :class="{selected:selected===widget.id}" :style="{...position(widget),width:`${baseSize(widget.kind).width*scaleOf(widget)*previewScale}px`,height:`${baseSize(widget.kind).height*scaleOf(widget)*previewScale}px`}" :data-testid="`layout-${widget.id}`" tabindex="0" role="button" :aria-label="`${widgetKinds.find(item=>item.kind===widget.kind)?.label}组件，方向键微调位置`" @keydown="nudge($event,widget)" @focus="select(widget.id)" @pointerdown="begin($event,widget)" @pointermove="move" @pointerup="finish" @pointercancel="finish"><div class="canvas-widget-content" :style="{transform:`scale(${scaleOf(widget)*previewScale})`}"><WidgetSurface :widget="widget" :editing="layout.editing" preview/></div><span v-if="layout.editing" class="canvas-drag-corner"><Move :size="12"/></span></div><div v-if="!visibleWidgets.length" class="empty-layout"><LayoutGrid :size="30" :stroke-width="1.2"/><h3>桌面还在等第一件小物</h3><p>从组件库挑一个喜欢的，放在这里。</p><button @click="tab='library'">去添加组件<Plus :size="12"/></button></div></div><aside class="widget-inspector"><template v-if="chosen"><div class="inspector-heading"><div><span>组件设置</span><h3>{{ widgetKinds.find(item=>item.kind===chosen!.kind)?.title }}</h3></div><NPopconfirm positive-text="移除" negative-text="取消" @positive-click="remove(chosen.id)"><template #trigger><NButton quaternary circle :disabled="busy" aria-label="移除这个组件" data-testid="remove-widget"><Trash2 :size="15"/></NButton></template>移除后，这个组件的内容、样式和设置会一并清除。</NPopconfirm></div><div v-if="displays.length>1" class="widget-field"><span>所在显示器</span><NSelect :value="monitorOf(chosen).id" :options="monitorOptions" :disabled="busy" aria-label="组件所在显示器" @update:value="targetMonitor"/></div><label class="widget-field inline"><span>显示在桌面</span><NSwitch :value="chosen.enabled" :disabled="busy" @update:value="update({enabled:$event})"/></label><div class="widget-field"><span>外观样式</span><NSelect :value="chosen.theme" :options="widgetThemes" :disabled="busy" @update:value="update({theme:$event})"/></div><div class="widget-field"><span>大小 <small>×{{ scaleOf(chosen).toFixed(2) }}</small></span><NSlider :value="scaleOf(chosen)" :min=".75" :max="1.5" :step=".05" :disabled="busy" @update:value="setScale"/></div><NotesWidget v-if="chosen.kind==='notes'" :key="chosen.id" :note="chosen.note" :save="saveChosenContent"/><CountdownEditor v-else-if="chosen.kind==='countdown'" :key="chosen.id" :data="chosen.countdown" :save="saveChosenContent"/><template v-else-if="chosen.kind==='weather'"><div class="widget-field"><span>天气城市</span><small v-if="chosen.city" class="current-city">当前：{{ chosen.city.name }} {{ chosen.city.admin1 }}</small><div class="city-search"><NInput v-model:value="cityQuery" placeholder="输入城市名称" @keydown.enter="search"/><NButton :loading="cityBusy" :disabled="!cityQuery.trim()" aria-label="搜索城市" @click="search"><Search :size="14"/></NButton></div><NSelect v-if="cities.length" :options="cityOptions" placeholder="选择搜索结果" :disabled="busy" @update:value="chooseCity"/></div><p class="inspector-note">天气沿用“关怀提醒”的天气来源。每半小时更新一次；可随时在组件上手动刷新。</p></template><p v-else class="inspector-note">{{ chosen.kind==='fortune'?'点击签筒求签，揭晓后可以展开好签册收藏。':'显示本机时间与农历。点击日期查看详情；休／班标识使用已收录的官方年度安排。点击月份回到今天。' }}</p></template><div v-else class="inspector-empty"><Move :size="22"/><p>点击预览中的组件，<br>调整它的大小和样式。</p></div></aside></div><div v-if="layout.widgets.length" class="widget-instance-list"><button v-for="(widget,index) in layout.widgets" :key="widget.id" :class="{on:selected===widget.id,hidden:!widget.enabled}" @click="select(widget.id)">{{ widgetKinds.find(item=>item.kind===widget.kind)?.label }} {{ index+1 }}<small>{{ widget.enabled?monitorOf(widget).name:'已隐藏' }}</small></button></div><p class="widget-hint">{{ store.info.platform==='windows'?'按 Win+D 可以查看桌面上的组件。':'' }} 组件可拖到其他屏幕，也可在设置中选择显示器；位置和内容保存在本机，重启后恢复。关闭小伴桌面时，组件也会关闭。</p></template>
    <InteractivePage v-else :store="store"/>
  </section>
</template>
<style scoped>
.widgets-page{display:grid;gap:24px;max-width:1260px;margin:0 auto}.widget-count{font-size:12px;color:var(--ui-text-tertiary);white-space:nowrap}.widget-tabs{display:flex;gap:24px;border-bottom:1px solid var(--ui-border)}.widget-tabs button{border:0;background:none;color:var(--ui-text-secondary);font:13px 'Microsoft YaHei',sans-serif;padding:0 1px 13px;cursor:pointer;position:relative}.widget-tabs button.on{color:var(--ui-text-primary);font-weight:600}.widget-tabs button.on:after{position:absolute;content:'';bottom:-1px;left:0;right:0;height:2px;background:#a89065}.widget-tabs span{margin-left:7px;background:var(--ui-surface-alt);padding:1px 5px;border-radius:5px;font-size:10px;color:var(--ui-text-tertiary)}.widget-library-intro{padding-left:16px;border-left:2px solid #bba477}.widget-library-intro>span{font:21px 'KaiTi','STKaiti',serif;letter-spacing:2px}.widget-library-intro p{font-size:12px;color:var(--ui-text-secondary);margin:8px 0 0}.widget-library{display:grid;grid-template-columns:repeat(3,1fr);gap:20px}.widget-library-card{border:1px solid var(--ui-border);background:var(--ui-surface);border-radius:16px;overflow:hidden;box-shadow:0 6px 20px #10252c0a}.widget-art{height:256px;position:relative;background:radial-gradient(ellipse at 45% 40%,#e1d5ba,#c7b390);overflow:hidden}.widget-art.notes{background:radial-gradient(ellipse at 40% 30%,#e4d8b6,#a99a78)}.widget-art.countdown{background:radial-gradient(ellipse at 70% 30%,#b293a0,#644f65)}.widget-art.calendar{background:radial-gradient(ellipse at 70% 30%,#698c86,#243f46)}.widget-art.weather{background:radial-gradient(ellipse at 65% 10%,#7e9fa8,#344e62)}.widget-index{position:absolute;top:18px;left:20px;font-size:8px;letter-spacing:2px;color:#5a5546;opacity:.5}.calendar .widget-index,.weather .widget-index{color:#f1ecda}.custom-intro{margin-top:28px}.widget-art.custom{background:radial-gradient(ellipse at 50% 30%,#b5c9c3,#3d6168)}.demo-widget.custom-demo{top:30px;transform:translateX(-50%) scale(.6)}.custom-card-header{display:flex;justify-content:space-between;align-items:center}.custom-delete-btn{border:0;background:none;color:var(--ui-text-tertiary);cursor:pointer;padding:4px;border-radius:4px;display:flex;align-items:center;transition:color .2s}.custom-delete-btn:hover{color:#dc2626}.demo-widget{position:absolute;left:50%;top:30px;transform:translateX(-50%) scale(.64);transform-origin:top center}.demo-widget.calendar{top:27px;transform:translateX(-50%) scale(.53)}.demo-widget.notes{top:28px;transform:translateX(-50%) scale(.58)}.demo-widget.countdown{top:45px;transform:translateX(-50%) scale(.8)}.demo-widget.weather{top:57px;transform:translateX(-50%) scale(.75)}.widget-card-copy{padding:20px 22px 23px}.widget-category{font-size:10px;letter-spacing:1px;color:var(--ui-text-tertiary)}.widget-card-copy h2{font:23px 'KaiTi','STKaiti',serif;letter-spacing:2px;margin:12px 0 8px}.widget-card-copy p{font-size:12px;color:var(--ui-text-secondary);margin:0;line-height:1.7}.widget-add{margin-top:22px;display:flex;align-items:center;justify-content:center;gap:7px;padding:9px 14px;border:1px solid var(--ui-border);border-radius:7px;background:var(--ui-surface-alt);color:var(--ui-text-primary);font-size:12px;cursor:pointer;transition:background .2s}.widget-add:hover{background:var(--ui-primary-soft)}.widget-hint{font-size:12px;line-height:1.8;color:var(--ui-text-tertiary);margin:0}.desktop-toolbar{display:flex;justify-content:space-between;align-items:center;gap:14px}.desktop-toolbar h2{font-size:16px;margin:0 0 6px}.desktop-toolbar p{font-size:12px;color:var(--ui-text-secondary);margin:0}.desktop-tools{display:flex;gap:9px;flex:none}.desktop-editor{display:grid;grid-template-columns:minmax(0,1fr) 265px;gap:18px}.desktop-canvas{position:relative;width:100%;border-radius:16px;background:radial-gradient(ellipse at 20% 10%,#708b82,#203943 60%,#102832);background-position:center;background-size:cover;box-shadow:inset 0 0 80px #071a2222;overflow:hidden;border:1px solid var(--ui-border)}.desktop-canvas:before,.desktop-canvas:after{content:'';position:absolute;left:0;right:0;pointer-events:none;background:#0713190f}.desktop-canvas:before{top:0;height:var(--inset-top)}.desktop-canvas:after{bottom:0;height:var(--inset-bottom)}.desktop-canvas.editing{outline:1px dashed #af9b75;outline-offset:3px}.canvas-label{position:absolute;top:18px;left:20px;font-size:10px;letter-spacing:2px;color:#f6edce;opacity:.5;pointer-events:none}.canvas-widget{position:absolute;touch-action:none;cursor:grab;transition:outline .2s;border-radius:12px}.desktop-canvas.editing .canvas-widget{cursor:grab}.canvas-widget:focus-visible{outline:2px solid #e6d29b;outline-offset:3px}.canvas-widget.selected{outline:1px solid #d0b982b3;outline-offset:3px}.canvas-widget-content{position:absolute;inset:0;transform-origin:top left;pointer-events:none}.canvas-drag-corner{position:absolute;top:-12px;right:-12px;border:1px solid #c5b17d;background:#203c3e;color:#e7d7ad;border-radius:50%;width:23px;height:23px;display:grid;place-items:center;pointer-events:none}.widget-inspector{padding:22px;border:1px solid var(--ui-border);border-radius:14px;background:var(--ui-surface)}.inspector-heading{display:flex;justify-content:space-between;align-items:start}.inspector-heading span{font-size:10px;color:var(--ui-text-tertiary);letter-spacing:1px}.inspector-heading h3{font:22px 'KaiTi','STKaiti',serif;letter-spacing:2px;margin:8px 0 6px}.widget-field{display:grid;gap:10px;margin-top:22px}.widget-field>span{font-size:12px;display:flex;justify-content:space-between}.widget-field small{font-size:11px;color:var(--ui-text-tertiary)}.widget-field.inline{display:flex;justify-content:space-between;align-items:center}.city-search{display:flex;gap:6px}.inspector-note{font-size:11px;line-height:1.9;color:var(--ui-text-tertiary);margin-top:24px}.inspector-empty{display:grid;place-items:center;height:100%;color:var(--ui-text-tertiary);text-align:center;font-size:12px;line-height:2;align-content:center;gap:12px}.widget-instance-list{display:flex;flex-wrap:wrap;gap:8px}.widget-instance-list button{display:flex;align-items:center;gap:12px;padding:8px 12px;border:1px solid var(--ui-border);border-radius:8px;color:var(--ui-text-secondary);background:var(--ui-surface);font-size:12px;cursor:pointer}.widget-instance-list .on{border-color:#af9468;color:var(--ui-text-primary)}.widget-instance-list .hidden{opacity:.5}.widget-instance-list small{font-size:10px;color:var(--ui-text-tertiary)}.empty-layout{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;color:#e4dfcb;gap:7px}.empty-layout h3{font:22px 'KaiTi','STKaiti',serif;letter-spacing:2px;margin:10px 0 3px}.empty-layout p{font-size:12px;opacity:.6;margin:0}.empty-layout button{display:flex;align-items:center;gap:8px;color:#e4d0a4;background:#12343670;border:1px solid #bca27260;padding:9px 16px;border-radius:7px;margin-top:13px;font-size:11px;cursor:pointer}button:disabled{opacity:.4;cursor:default}button:focus-visible{outline:2px solid #af9468;outline-offset:3px}@media(max-width:1000px){.widget-library{grid-template-columns:repeat(2,1fr)}.desktop-editor{grid-template-columns:minmax(0,1fr) 225px}.widget-inspector{padding:17px}}@media(max-width:720px){.widget-library{grid-template-columns:1fr}.desktop-editor{grid-template-columns:1fr}.desktop-toolbar{flex-wrap:wrap}.widget-art{height:245px}}
</style>

