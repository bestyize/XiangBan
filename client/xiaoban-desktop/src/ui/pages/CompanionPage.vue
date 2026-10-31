<script setup lang="ts">
import {computed,onDeactivated,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {NAlert,NModal,NPopconfirm,NSelect,NSwitch} from 'naive-ui';
import {MoreHorizontal,Download,HeartHandshake,Monitor,Pin,Plus,Settings2,Sparkles,Trash2,X} from 'lucide-vue-next';
import type {ControlStore} from '../control-types.ts';
import {EXPRESSIONS,stillFrame} from '../../companion/expressions.ts';
import StudioActions from '../components/StudioActions.vue';
import PetSettingsDialog,{type PetSettingsPatch} from '../components/PetSettingsDialog.vue';
import ResourceShareActions from '../components/ResourceShareActions.vue';
import PetActor from '../../pets/PetActor.vue';
import {createPetClient} from '../../pets/client.ts';
import {definitionOf,type PetDefinition,type PetTheme,type PetInstance} from '../../pets/model.ts';
import {petMarkup} from '../../pets/render.ts';
const images=new WeakMap<PetDefinition,string>();
function petImage(pet:PetDefinition){let image=images.get(pet);if(!image){image=`data:image/svg+xml;charset=utf-8,${encodeURIComponent(petMarkup(pet,stillFrame('happy'),'gallery').replace(/viewBox="[^"]+"/,'viewBox="0 -72 320 352"'))}`;images.set(pet,image);}return image;}
import {invoke} from '../../bridge.ts';
const props=defineProps<{store:ControlStore;settingsRequest?:{id:string}|null}>();
const emit=defineEmits<{settingsOpened:[]}>();
const client=createPetClient(),{catalog,ready,busy,error}=client;
/** 全部 shows every theme, 在桌面上 one card per pet on the desktop, 我的 only the user's own themes. */
const filter=ref<'all'|'desk'|'mine'>('all'),hover=ref(''),editing=ref<string|null>(null);
const adding=ref<{themeId:string;petId:string;name:string}|null>(null),proactivityOpen=ref(false);
const themes=computed(()=>filter.value==='mine'?catalog.value.themes.filter(t=>t.source==='custom'):catalog.value.themes);
const themeMenu=ref(''),manageMenu=ref('');
function managePet(themeId:string,petId:string){const items=onDeskOf(themeId,petId);if(items.length===1)openManage(items[0].id);else manageMenu.value=manageMenu.value===`${themeId}/${petId}`?'':`${themeId}/${petId}`;}
function closeMenus(){themeMenu.value='';manageMenu.value='';}
function onKey(e:KeyboardEvent){if(e.key==='Escape')closeMenus();}
const instances=computed(()=>catalog.value.layout.instances);
const current=computed(()=>instances.value.find(p=>p.id===editing.value));
const currentTheme=computed(()=>catalog.value.themes.find(t=>t.id===current.value?.themeId));
const onDeskOf=(themeId:string,petId:string)=>instances.value.filter(p=>p.themeId===themeId&&p.petId===petId);
const deskGroups=computed(()=>{
  const groups=catalog.value.themes.map(theme=>({theme,items:instances.value.filter(p=>p.themeId===theme.id)})).filter(group=>group.items.length);
  const orphans=instances.value.filter(p=>!catalog.value.themes.some(t=>t.id===p.themeId));
  return orphans.length?[...groups,{theme:undefined,items:orphans}]:groups;
});
const themeNameOf=(id:string)=>catalog.value.themes.find(t=>t.id===id)?.name??'作品暂不可用';
interface GreetingSettings {enabled:boolean;frequency:string;contextEnabled?:boolean;calendarRegion?:'CN'|null}
const greetingSettings=ref<GreetingSettings>({enabled:false,frequency:'natural',contextEnabled:false,calendarRegion:null}),greetingReady=ref(false),greetingBusy=ref(false);
interface EnvironmentContext {
 temporal?:{localDay:number;hour:number;minute:number};
 calendar?:{dayKind:string};
 weather?:{condition:string};
 activity?:{mode:string};
 task?:{state:string};
}
const environment=ref<EnvironmentContext|null>(null);
const environmentLabels=computed(()=>{
 const value=environment.value;
 if(!value)return '尚未读取';
 const dates=value.temporal ? `${value.temporal.localDay} ${String(value.temporal.hour).padStart(2,'0')}:${String(value.temporal.minute).padStart(2,'0')}` : '时间暂无数据';
 const days:Record<string,string>={workday:'工作日',rest_day:'休息日',holiday:'节假日',makeup_workday:'调休工作日',unknown:'地区日历未知'};
 const day=value.calendar ? (days[value.calendar.dayKind]||'地区日历未知') : '地区日历暂无数据';
 const conditions:Record<string,string>={clear:'晴',cloudy:'多云',fog:'雾',rain:'雨',snow:'雪',thunderstorm:'雷雨'};
 const weather=value.weather ? (conditions[value.weather.condition]||'天气未知') : '天气无有效缓存';
 const mode=value.activity?.mode || 'unknown';
 const task=value.task?.state ? ` · 任务状态：${value.task.state}` : '';
 return `${dates} · ${day} · ${weather} · 活动状态：${mode}${task}`;
});
async function inspectEnvironment(){
 try{environment.value=await invoke<EnvironmentContext>('get_proactivity_context');}
 catch(e){props.store.notify(e,'error');}
}
const calendarRegions=[{label:'未指定 · 不推断法定工作日',value:'none'},{label:'中国大陆 · 2026 年官方调休',value:'CN'}];
const greetingModes=[{label:'安静陪伴 · 最多每天 1 次',value:'quiet'},{label:'自然陪伴 · 最多每天 2 次',value:'natural'},{label:'活泼伙伴 · 最多每天 4 次',value:'lively'}];
async function loadGreeting(){if(!props.store.desktop)return;try{const result=await invoke<GreetingSettings>('get_proactivity_settings');if(result){greetingSettings.value=result;greetingReady.value=true;}}catch(e){props.store.notify(e,'error');}}
async function changeGreeting(patch:Partial<GreetingSettings>){if(!greetingReady.value||greetingBusy.value)return;greetingBusy.value=true;try{greetingSettings.value=await invoke<GreetingSettings>('configure_proactivity',{settings:{...greetingSettings.value,...patch}});environment.value=null;props.store.notify('感知与陪伴设置已保存');}catch(e){props.store.notify(e,'error');}finally{greetingBusy.value=false;}}
function openManage(id:string){if(!instances.value.some(p=>p.id===id)){props.store.notify('这个伙伴已不在桌面上','info');return;}editing.value=id;}
async function act(action:()=>Promise<unknown>,success?:string){try{await action();if(success)props.store.notify(success);}catch(e){props.store.notify(e,'error');}}
const placements=[{value:'topmost' as const,label:'始终置顶',hint:'显示在其他窗口上方，随时陪着你',icon:Pin},{value:'desktop' as const,label:'贴在桌面',hint:'放在桌面上，不遮挡其他窗口',icon:Monitor}];
function requestAdd(themeId:string,petId:string,name:string){if(ready.value&&!busy.value)adding.value={themeId,petId,name};}
async function add(layer:PetInstance['layer']){const target=adding.value;if(!target)return;adding.value=null;await act(async()=>{await client.add(target.themeId,target.petId,layer);if(!props.store.settings.companionEnabled)await props.store.setCompanion(true);},'已添加到桌面，可拖动位置，右键管理');}
function create(base?:PetTheme){void props.store.startCreation('pet').then(()=>{if(base)props.store.creationDraft=`请以宠物主题「${base.name}」（${base.id}）为参考，创作我的自定义宠物主题。先读取 pet_read_format 和现有主题，按我的要求调整：`;});}
function update(patch:Partial<PetInstance>){if(current.value)void act(()=>client.update(current.value!.id,patch));}
function saveSettings(patch:PetSettingsPatch){update(patch);}
function removeCurrent(){const id=current.value?.id;editing.value=null;if(id)void act(()=>client.remove(id));}
function centerCurrent(){update({x:.5,y:.5});}
function createFromCurrent(){const theme=currentTheme.value;editing.value=null;create(theme);}
watch(()=>props.settingsRequest,async request=>{if(!request)return;await client.load();if(props.settingsRequest!==request)return;openManage(request.id);emit('settingsOpened');},{immediate:true});
function manage(e:Event){const id=(e as CustomEvent<{id:string}>).detail?.id;void client.load().then(()=>{if(id)openManage(id);});}
onMounted(()=>{void client.load();void loadGreeting();window.addEventListener('pets-manage',manage);window.addEventListener('keydown',onKey);});
onDeactivated(()=>{closeMenus();hover.value='';editing.value=null;adding.value=null;proactivityOpen.value=false;});
onBeforeUnmount(()=>{client.dispose();window.removeEventListener('pets-manage',manage);window.removeEventListener('keydown',onKey);});
</script>
<template>
<section class="pets-page" data-testid="companion-page" @keydown.esc.capture="closeMenus()">
 <div v-if="themeMenu||manageMenu" class="menu-dismiss" @click="closeMenus()"/>
 <StudioActions>
  <ResourceShareActions :store="store" kind="pet" variant="xb" import-label="导入" :disabled="!ready||busy" @imported="async()=>{await client.load();filter='mine';}"/>
  <button type="button" class="xb-btn-primary" data-testid="pet-create" @click="create()"><Sparkles :size="14"/>让小伴创作</button>
 </StudioActions>
 <div class="pt-toolbar">
  <div class="xb-seg" role="group" aria-label="筛选">
   <button type="button" :aria-pressed="filter==='all'" data-testid="pet-filter-all" @click="closeMenus();filter='all'">全部</button>
   <button type="button" :aria-pressed="filter==='desk'" data-testid="pet-filter-desk" @click="closeMenus();filter='desk'">在桌面上 <b class="cnt">{{ instances.length }}</b></button>
   <button type="button" :aria-pressed="filter==='mine'" data-testid="pet-filter-mine" @click="closeMenus();filter='mine'">我的</button>
  </div>
  <div class="pt-tools">
   <button v-if="store.desktop" type="button" class="xb-btn-ghost" aria-haspopup="dialog" data-testid="pet-proactivity-open" @click="proactivityOpen=true"><HeartHandshake :size="14"/>主动陪伴</button>
   <span class="pt-label">桌面伙伴</span>
   <NSwitch :value="store.settings.companionEnabled" aria-label="显示全部桌面伙伴" data-testid="pet-master-switch" @update:value="value=>act(()=>store.setCompanion(value))"/>
  </div>
 </div>
 <div class="pt-scroll">
  <NAlert v-if="error" type="error" :show-icon="false" class="pt-alert">{{ error }}<button type="button" class="xb-link" @click="client.load()">重新载入</button></NAlert>

  <div v-if="ready&&filter==='mine'&&!themes.length" class="xb-empty" data-testid="pet-custom-empty"><span class="xb-empty-icon"><Sparkles :size="24"/></span><h3>你想象中的小伙伴，在这里诞生。</h3><p>让 AI 创作你喜欢的小伙伴，也可以导入别人分享的 ZIP 宠物资源包。</p><button type="button" class="xb-btn-primary" @click="create()">开始创作</button></div>

  <template v-if="filter!=='desk'">
   <section v-for="theme in themes" :key="theme.id" class="pet-theme-detail" :aria-label="theme.name" :data-testid="`pet-theme-${theme.id}`">
    <header class="th-head" :title="theme.description" :aria-description="theme.description"><h2>{{ theme.name }}</h2><span class="sub">{{ theme.pets.length }} 款</span>
     <div class="theme-actions"><button type="button" class="xb-icon-btn is-sm" :aria-label="`${theme.name}更多操作`" :aria-expanded="themeMenu===theme.id" @click="themeMenu=themeMenu===theme.id?'':theme.id"><MoreHorizontal :size="17"/></button><div v-if="themeMenu===theme.id" class="theme-menu xb-menu"><button type="button" class="xb-menu-item" @click="closeMenus();create(theme)">基于此主题创作</button>
      <button v-if="theme.source==='builtin'" type="button" class="xb-menu-item" :disabled="busy" title="导出参考 JSON" data-testid="pet-export-reference" @click="act(()=>client.exportTheme(theme.id))"><Download :size="13"/>导出参考 JSON</button>
      <ResourceShareActions v-if="theme.source==='custom'" :store="store" kind="pet" :id="theme.id" :disabled="busy"/>
      <NPopconfirm v-if="theme.source==='custom'" @positive-click="act(()=>client.deleteTheme(theme.id))"><template #trigger><button type="button" class="xb-menu-item is-danger" :disabled="busy" title="删除主题" aria-label="删除主题"><Trash2 :size="14"/>删除主题</button></template>删除这个自定义主题？已添加的宠物需先移除。</NPopconfirm></div></div></header>
    <p v-if="theme.source==='custom'&&themeMenu===theme.id" class="pet-theme-export-note" data-testid="pet-theme-export-note">导出或分享整套主题（含 {{ theme.pets.length }} 款形象），不包含桌面位置、昵称或心情。</p>
    <div class="pet-cards">
     <article v-for="p in theme.pets" :key="p.id" class="pc pet-card" :data-testid="`pet-card-${p.id}`" @pointerenter="hover=`${theme.id}/${p.id}`" @pointerleave="hover=''">
      <div class="st" :style="{'--theme-color':theme.color}"><PetActor v-if="hover===`${theme.id}/${p.id}`" :definition="p" :size="180" gallery :paused="store.settings.paused"/><img v-else :src="petImage(p)" :alt="p.name" width="150" height="150"/>
      </div>
      <div class="nm"><b :title="p.description" :aria-description="p.description">{{ p.name }}</b><span v-if="onDeskOf(theme.id,p.id).length" class="pet-status" :title="`桌面上有 ${onDeskOf(theme.id,p.id).length} 只${p.name}`"><i/>{{ onDeskOf(theme.id,p.id).length }} 只</span></div>
      <div v-if="onDeskOf(theme.id,p.id).length" class="btns"><button type="button" class="xb-btn pc-set" aria-haspopup="dialog" :data-testid="`pet-manage-${onDeskOf(theme.id,p.id)[0].id}`" @click="managePet(theme.id,p.id)"><Settings2 :size="14"/>管理</button><button type="button" class="xb-btn pc-plus" :disabled="!ready||busy" :aria-label="`再添加一只${p.name}`" aria-haspopup="dialog" :aria-expanded="adding?.themeId===theme.id&&adding?.petId===p.id" :data-testid="`pet-add-${p.id}`" @click="requestAdd(theme.id,p.id,p.name)"><Plus :size="14"/></button></div>
      <button v-else type="button" class="xb-btn add-desk" :disabled="!ready||busy" aria-haspopup="dialog" :aria-expanded="adding?.themeId===theme.id&&adding?.petId===p.id" :data-testid="`pet-add-${p.id}`" @click="requestAdd(theme.id,p.id,p.name)"><Plus :size="14"/>添加到桌面</button>
      <div v-if="manageMenu===`${theme.id}/${p.id}`" class="instance-menu xb-menu" :aria-label="`管理${p.name}`"><button v-for="instance in onDeskOf(theme.id,p.id)" :key="instance.id" type="button" class="xb-menu-item" @click="closeMenus();openManage(instance.id)"><Settings2 :size="14"/>{{ instance.name }}<span class="xb-menu-hint">{{ instance.enabled?'':'已隐藏' }}</span></button></div>
     </article>

    </div>
   </section>
  </template>

  <section v-else class="desktop-pets" data-testid="desktop-pets">
   <div v-if="!instances.length" class="desktop-pets-empty">还没有小伙伴，挑一只喜欢的添加到桌面吧。<button type="button" class="xb-link" @click="closeMenus();filter='all'">去挑选伙伴</button></div>
   <section v-for="group in deskGroups" :key="group.theme?.id??'orphans'" class="pet-theme-detail">
    <header class="th-head"><h2>{{ group.theme?.name??'作品暂不可用' }}</h2><span class="sub">{{ group.items.length }} 只在桌面上</span></header>
    <div class="pet-cards">
     <article v-for="p in group.items" :key="p.id" class="pc pet-card instance" :class="{off:!p.enabled}" :data-testid="`pet-instance-${p.id}`" @contextmenu.prevent.stop="openManage(p.id)" @pointerenter="hover=p.id" @pointerleave="hover=''">
      <div class="st" :style="{'--theme-color':group.theme?.color}">
       <template v-if="definitionOf(catalog.themes,p)"><PetActor v-if="hover===p.id" :definition="definitionOf(catalog.themes,p)!" :size="180" gallery :expression="p.expression" :links="p.links" :paused="store.settings.paused"/><img v-else :src="petImage(definitionOf(catalog.themes,p)!)" :alt="p.name" width="150" height="150"/></template>
       <span v-else class="gone">作品暂不可用</span>
       </div>
      <div class="nm"><b :title="definitionOf(catalog.themes,p)?.name">{{ p.name }}</b><span class="sub">{{ EXPRESSIONS[p.expression].label }} · {{ p.size }}px · {{ p.layer==='desktop'?'贴在桌面':'始终置顶' }}</span><NSwitch :value="p.enabled" size="small" :disabled="busy" :aria-label="`显示${p.name}`" :data-testid="`pet-visible-${p.id}`" @update:value="enabled=>act(()=>client.update(p.id,{enabled}))"/></div>
      <div class="btns"><button type="button" class="xb-btn pc-set" aria-haspopup="dialog" :data-testid="`pet-manage-${p.id}`" @click="editing=p.id"><Settings2 :size="14"/>管理</button><button v-if="definitionOf(catalog.themes,p)" type="button" class="xb-btn pc-plus" :disabled="!ready||busy" :aria-label="`再添加一只${p.name}`" @click="requestAdd(p.themeId,p.petId,definitionOf(catalog.themes,p)!.name)"><Plus :size="14"/></button></div>
     </article>
    </div>
   </section>
  </section>
 </div>

 <NModal :show="!!adding" :auto-focus="false" @update:show="show=>{if(!show)adding=null}">
  <section v-if="adding" class="pet-placement-dialog" role="dialog" aria-modal="true" aria-labelledby="pet-placement-title" data-testid="pet-placement-dialog">
   <header><h2 id="pet-placement-title">把{{ adding.name }}放在哪里？</h2><p>之后也可以在伙伴设置中修改。</p></header>
   <button v-for="option in placements" :key="option.value" type="button" class="pet-placement-option" :data-testid="`pet-placement-${option.value}`" :disabled="busy||!ready" @click="add(option.value)"><component :is="option.icon" :size="21"/><span><b>{{ option.label }}</b><small>{{ option.hint }}</small></span></button>
   <button type="button" class="pet-placement-cancel" data-testid="pet-placement-cancel" @click="adding=null">取消</button>
  </section>
 </NModal>

 <PetSettingsDialog :show="!!current" :instance="current" :themes="catalog.themes" :busy="busy" :paused="store.settings.paused" @update:show="value=>{if(!value)editing=null}" @save="saveSettings" @remove="removeCurrent" @center="centerCurrent" @create="createFromCurrent">
  <template #extra><div v-if="currentTheme?.source==='custom'" class="pet-share-panel" data-testid="pet-instance-share"><b>分享这款宠物形象</b><p>打包当前所属的整个主题，不包含这只宠物的个人设置。</p><ResourceShareActions :store="store" kind="pet" :id="current!.themeId" :disabled="busy"/></div></template>
 </PetSettingsDialog>

 <NModal :show="proactivityOpen" :auto-focus="false" @update:show="proactivityOpen=$event">
  <section v-if="store.desktop" class="proactivity-dialog" role="dialog" aria-modal="true" aria-label="主动陪伴" data-testid="pet-proactivity-settings">
   <header><h2>主动陪伴</h2><button type="button" class="xb-icon-btn" aria-label="关闭" @click="proactivityOpen=false"><X :size="16"/></button></header>
   <p>偶尔停留在宠物身上，它会轻声搭话。不自动朗读、不读取屏幕，也不会主动联系 AI。问候和环境上下文分别授权，均默认关闭；环境上下文仅复用已有时间、日历、天气缓存及任务状态。</p>
   <div class="xb-group">
    <div class="xb-row"><div class="xb-row-text"><b>悬停问候</b><span>鼠标停在宠物身上时，偶尔打个招呼</span></div><NSwitch :value="greetingSettings.enabled" :disabled="!greetingReady||greetingBusy" data-testid="pet-proactivity-switch" @update:value="enabled=>changeGreeting({enabled})"/></div>
    <div class="xb-row"><div class="xb-row-text"><b>环境上下文</b><span>让问候参考时间、日历和天气</span></div><NSwitch :value="greetingSettings.contextEnabled??false" :disabled="!greetingReady||greetingBusy" data-testid="pet-context-switch" @update:value="enabled=>changeGreeting({contextEnabled:enabled})"/></div>
    <div class="xb-row"><div class="xb-row-text"><b>节假日地区</b></div><NSelect :value="greetingSettings.calendarRegion??'none'" :options="calendarRegions" :disabled="!greetingReady||greetingBusy||!greetingSettings.contextEnabled" size="small" style="width:235px" data-testid="pet-proactivity-calendar" @update:value="region=>changeGreeting({calendarRegion:region==='CN'?'CN':null})"/></div>
    <div class="xb-row"><div class="xb-row-text"><b>陪伴频率</b></div><NSelect :value="greetingSettings.frequency" :options="greetingModes" :disabled="!greetingReady||greetingBusy" size="small" style="width:235px" data-testid="pet-proactivity-frequency" @update:value="frequency=>changeGreeting({frequency})"/></div>
   </div>
   <div v-if="greetingSettings.contextEnabled" class="pet-context-preview"><button type="button" class="xb-btn is-sm" data-testid="pet-inspect-context" @click="inspectEnvironment">查看最近感知状态</button><span data-testid="pet-context-summary">{{ environmentLabels }}</span></div>
  </section>
 </NModal>
</section>
</template>
<style scoped>
.pets-page{container-type:inline-size;height:100%;min-height:0;min-width:0;display:flex;flex-direction:column;overflow:hidden}
.pt-toolbar{flex:none;display:flex;align-items:center;gap:10px;padding:16px 28px 0}
.pt-toolbar .cnt{font-weight:600;color:var(--ui-ink)}
.pt-tools{margin-left:auto;display:flex;align-items:center;gap:10px}
.pt-label{font-size:12px;color:var(--ui-text-secondary)}
.pt-scroll{flex:1;min-height:0;overflow-y:auto;padding:4px 28px 28px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.pt-alert{margin-top:14px;border-radius:10px}
.pet-theme-detail{margin-top:22px}
.th-head{display:flex;align-items:baseline;gap:10px;min-width:0}
.th-head h2{margin:0;font-size:15px;font-weight:600;white-space:nowrap}
.th-head .sub{font-size:12px;color:var(--ui-text-secondary);min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.theme-actions{margin-left:auto;display:flex;align-items:center;gap:12px;flex:none}
.theme-actions .xb-link{display:inline-flex;align-items:center;gap:4px;font-size:12.5px;font-weight:500}
.theme-actions .xb-link.muted{color:var(--ui-text-secondary);font-weight:400}
.pet-theme-export-note{margin:4px 0 0;font-size:11.5px;color:var(--ui-text-tertiary)}
.pet-cards{margin-top:12px;display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:25px 20px}
.pc{position:relative;min-width:0;display:flex;flex-direction:column;gap:9px;background:transparent}
.pc.off .st{opacity:.45;filter:grayscale(.5)}
.st{position:relative;aspect-ratio:1.13;border-radius:13px;display:flex;align-items:center;justify-content:center;background:radial-gradient(ellipse at 50% 80%,var(--ui-soft),var(--ui-configuration-canvas) 70%);overflow:hidden}
.st img,.st :deep(.pet-actor){position:absolute!important;inset:10px;width:calc(100% - 20px)!important;height:calc(100% - 20px)!important;object-fit:contain}
.st .gone{font-size:12px;color:var(--ui-text-tertiary)}
.badge{position:absolute;top:8px;left:8px;max-width:calc(100% - 16px);height:20px;padding:0 8px;border-radius:6px;background:var(--ui-btn);color:#fff;font-size:11px;font-weight:500;display:inline-flex;align-items:center;gap:5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.badge i{width:6px;height:6px;border-radius:50%;background:#fff;opacity:.9;flex:none}
.nm{display:flex;align-items:baseline;gap:8px;padding:0;min-width:0}
.nm b{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;font-size:13px;font-weight:500;white-space:nowrap}
.nm .sub{flex:1;min-width:0;font-size:12px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.nm .n-switch{align-self:center;flex:none}
.btns{display:flex;gap:6px}
.pc-set{flex:1;height:30px;border:0;box-shadow:none;background:var(--ui-control);color:var(--ui-text-secondary);font-size:12px}.pc-set:hover{background:var(--ui-control-hover)}
.pc-plus{width:30px;height:30px;padding:0;border:0;box-shadow:none;background:transparent;color:var(--ui-text-secondary)}.pc-plus:hover{background:var(--ui-soft);color:var(--ui-ink)}
.add-desk{height:32px;background:var(--ui-soft);color:var(--ui-ink);box-shadow:none;font-weight:500}
.add-desk:hover:not(:disabled){background:var(--ui-soft);filter:brightness(.97)}
.make-own{border:0;cursor:pointer;align-items:center;justify-content:center;gap:8px;min-height:242px;color:var(--ui-text-secondary);background:var(--ui-surface-sunken);box-shadow:inset 0 0 0 1.5px var(--ui-border-strong);font-family:inherit}
.make-own:hover{box-shadow:inset 0 0 0 1.5px var(--ui-btn)}
.make-own .mk{width:52px;height:52px;border-radius:16px;background:var(--ui-soft);color:var(--ui-ink);display:grid;place-items:center}
.make-own b{font-size:14px;font-weight:600;color:var(--ui-text-primary)}
.make-own .sub{font-size:12px}
.desktop-pets-empty{margin-top:22px;padding:30px;border-radius:14px;background:var(--ui-surface-sunken);text-align:center;font-size:13px;color:var(--ui-text-secondary)}
.desktop-pets-empty .xb-link{margin-left:10px}
.pet-share-panel{display:grid;gap:8px;padding:12px;border-radius:12px;background:var(--ui-surface-sunken)}
.pet-share-panel b{font-size:12px;font-weight:600;color:var(--ui-text-primary)}
.pet-share-panel p{margin:0;font-size:11.5px;line-height:1.6;font-weight:400;color:var(--ui-text-secondary)}
.pet-placement-dialog{width:372px;max-width:calc(100vw - 32px);padding:24px;border-radius:var(--ui-radius-dialog);background:var(--ui-surface);box-shadow:var(--ui-shadow-dialog);display:grid;gap:10px;color:var(--ui-text-primary)}
.pet-placement-dialog h2{font-size:17px;font-weight:600;margin:0}
.pet-placement-dialog header p{font-size:12px;color:var(--ui-text-secondary);margin:6px 0 4px}
.pet-placement-option{display:flex;align-items:center;gap:14px;width:100%;padding:14px 16px;border:0;border-radius:12px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border);color:var(--ui-text-primary);text-align:left;cursor:pointer;font:inherit}
.pet-placement-option:hover{box-shadow:0 0 0 1.5px var(--ui-btn);background:var(--ui-soft)}
.pet-placement-option>svg{flex:none;color:var(--ui-ink)}
.pet-placement-option span{display:grid;gap:4px}
.pet-placement-option b{font-size:14px;font-weight:600}
.pet-placement-option small{font-size:12px;color:var(--ui-text-secondary)}
.pet-placement-cancel{justify-self:center;border:0;background:none;color:var(--ui-text-secondary);font-size:12.5px;padding:6px 12px;cursor:pointer}
.proactivity-dialog{width:540px;max-width:calc(100vw - 32px);padding:18px 22px 22px;border-radius:var(--ui-radius-dialog);background:var(--ui-surface-sunken);box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog);color:var(--ui-text-primary)}
.proactivity-dialog header{display:flex;align-items:center;justify-content:space-between;margin-bottom:6px}
.proactivity-dialog h2{margin:0;font-size:16px;font-weight:600}
.proactivity-dialog>p{margin:0 0 14px;font-size:12px;line-height:1.7;color:var(--ui-text-secondary)}
.pet-context-preview{margin-top:12px;display:flex;align-items:center;flex-wrap:wrap;gap:10px;font-size:11.5px;color:var(--ui-text-secondary)}
.pet-context-preview span{flex:1;min-width:200px}
@container(max-width:760px){.pet-cards{grid-template-columns:repeat(3,minmax(0,1fr))}}
.pet-status{font-size:11px;color:var(--ui-text-secondary);display:flex;align-items:center;gap:4px;white-space:nowrap}.pet-status i{width:5px;height:5px;border-radius:50%;background:var(--ui-success)}
.theme-actions{position:relative}.theme-menu,.instance-menu{position:absolute;z-index:31;right:0;top:32px;width:210px;max-height:260px;overflow:auto}.instance-menu{top:100%;left:0;right:auto;width:100%;min-width:160px}.menu-dismiss{position:fixed;inset:0;z-index:30}.theme-menu .xb-menu-item{font-size:12px}.pet-card button:focus-visible{outline:2px solid var(--ui-btn);outline-offset:3px}
.theme-menu :deep(.resource-share-actions){display:flex;flex-direction:column;align-items:stretch;gap:2px}.theme-menu :deep(.n-button){width:100%;justify-content:flex-start;height:33px;background:transparent;border-radius:6px}.theme-menu :deep(.n-button:hover){background:var(--ui-surface-hover)}
</style>

