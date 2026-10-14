<script setup lang="ts">
import {computed,ref,watch,nextTick,onBeforeUnmount} from "vue";
import {NButton,NInput,NSelect,NSlider,NSwitch,NTabs,NTabPane,NEmpty,NAlert,NPagination,NCollapse,NCollapseItem,NColorPicker,NSpin,NPopconfirm} from "naive-ui";
import {Search,Maximize2,X,Monitor,Trash2} from "lucide-vue-next";
import {invoke,wallpaperDefaults,type WallpaperLibrary,type WallpaperOptions} from "../../bridge.ts";
import {siteMedia,siteFullMedia} from "../../wallpaper-media.ts";
import type {ControlStore} from "../control-types.ts";
import {useWallpaperWorkspace,type Selection} from "../wallpaper/useWallpaperWorkspace.ts";
import MediaPreview from "../wallpaper/MediaPreview.vue";
import {createWallpaperContextMenu,type ContextAction} from "../wallpaper/context-menu.ts";
const props=defineProps<{store:ControlStore}>();
const {progress,progressPercent,saved,tab,media,query,category,page,localSource,selection,draft,busy,downloading,cancelling,loading,error,feed,categories,kind,select,cancel,cancelDownload,load,localItems,action}=useWallpaperWorkspace(props.store);
const current=computed(()=>selection.value?.source==='local'?selection.value.item.definition:null);
const title=computed(()=>selection.value?.source==='local'?selection.value.item.definition.title:selection.value?.item.title || '');
const cards=computed<Selection[]>(()=>tab.value==='online'?feed.items.map(item=>({source:'online',item})):localItems.value.map(item=>({source:'local',item})));
const cardId=(card:Selection)=>card.source==='local'?card.item.definition.id:card.item.id;
const cardTitle=(card:Selection)=>card.source==='local'?card.item.definition.title:card.item.title;
const cardKind=(card:Selection)=>card.source==='local'?card.item.definition.kind:card.item.media_type;
const sources=(card:Selection)=>card.source==='online'?siteMedia(card.item,props.store.settings.websiteUrl):{poster:card.item.coverUrl || (card.item.definition.kind==='image'?card.item.previewUrl || '':''),video:card.item.previewUrl || ''};
const preview=computed(()=>selection.value?sources(selection.value):{poster:'',video:''});
const viewer=ref(false);
const fullMedia=computed(()=>selection.value?.source==='online'?siteFullMedia(selection.value.item,props.store.settings.websiteUrl):preview.value);
const canView=computed(()=>Boolean(preview.value.poster || preview.value.video));
const closeViewer=ref<HTMLButtonElement>();let viewerOpener:HTMLElement|null=null;
function openViewer(){if(!canView.value)return;viewerOpener=document.activeElement instanceof HTMLElement?document.activeElement:null;viewer.value=true;}
watch(selection,()=>{viewer.value=false;});
watch(viewer,async open=>{await nextTick();if(open)closeViewer.value?.focus();else{viewerOpener?.focus();viewerOpener=null;}});
const types=[{label:'全部',value:''},{label:'图片',value:'image'},{label:'视频',value:'video'},{label:'动画',value:'canvas'}];
const typeOptions=computed(()=>tab.value==='online'?types.filter(type=>type.value!=='canvas'):types);
const sourceOptions=[{label:'全部来源',value:''},{label:'我的导入',value:'local'},{label:'内置壁纸',value:'builtin'},{label:'订阅壁纸',value:'subscription'}];
const categoryOptions=computed(()=>[{label:'全部',value:''},...categories.value.map(c=>({label:c.name,value:c.id}))]);
watch(tab,value=>{if(value==='online'&&media.value==='canvas')media.value='';});
const fits=[{label:'铺满屏幕',value:'cover'},{label:'完整显示',value:'contain'},{label:'拉伸适配',value:'fill'}];
async function importMedia(){try {const id=await props.store.importMedia();if(id){tab.value='local';localSource.value='local';const item=props.store.library.items.find(e=>e.definition.id===id);if(item)select({source:'local',item});}}catch(e){props.store.notify(e,'error');}}
function setFit(value:string){if(busy.value)return;draft.value.fit=value as WallpaperOptions['fit'];}
async function useNativeWallpaper(){
  if(busy.value)return;
  selection.value=null;busy.value=true;
  try{await props.store.applyWallpaper('companion',wallpaperDefaults,false);props.store.notify('已使用原生壁纸','success');}
  catch(e){props.store.notify(e,'error');}
  finally{busy.value=false;}
}

async function deleteLocal(id: string) {
  if (busy.value) return;
  busy.value = true;
  try {
    await props.store.deleteWallpaper(id);
    props.store.notify('已删除壁纸', 'success');
    cancel();
  } catch (e) {
    props.store.notify(e, 'error');
  } finally {
    busy.value = false;
  }
}

const nativeContext=createWallpaperContextMenu();
const context=ref<{x:number;y:number;card:Selection;items:ContextAction[]} | null>(null);
function contextActions(card:Selection):ContextAction[]{
  const items:ContextAction[]=[{key:'inspect',label:'预览与设置'},{key:'apply',label:'应用到桌面',disabled:busy.value}];
  if(card.source==='online')items.push({key:'save',label:saved.value?'已保存到我的壁纸':'保存到我的壁纸',disabled:busy.value||saved.value});
  if(cardKind(card)==='canvas')items.push({key:'preview',label:'独立窗口预览',disabled:busy.value});
  if(card.source==='local' && card.item.source==='local')items.push({key:'delete',label:'删除这幅壁纸',disabled:busy.value});
  return items;
}
function chooseContext(key:string){
  const target = context.value?.card;
  context.value=null;
  if(key==='apply')void action(true);
  else if(key==='save')void action(false);
  else if(key==='preview')void action(true,true);
  else if(key==='delete' && target)void deleteLocal(cardId(target));
}
async function openContext(event:MouseEvent,card:Selection){
  if(busy.value)return;
  const selected=selection.value;
  if(!selected||selected.source!==card.source||cardId(selected)!==cardId(card))select(card);
  const items=contextActions(card);
  try{if(props.store.desktop&&await nativeContext.show(items,chooseContext))return;}
  catch(e){props.store.notify(e,'error');}
  context.value={x:Math.max(4,Math.min(event.clientX,window.innerWidth-214)),y:Math.max(4,Math.min(event.clientY,window.innerHeight-items.length*34-16)),card,items};
}
function onKey(event:KeyboardEvent){if(event.key==="Escape"){if(viewer.value)viewer.value=false;else if(context.value)context.value=null;else cancel();}}
window.addEventListener("keydown",onKey);
onBeforeUnmount(()=>{nativeContext.dispose();window.removeEventListener("keydown",onKey);});
</script>
<template>
<section class="ui-page wallpaper-workbench" data-testid="wallpapers-page">
<div class="workbench-tools">
<div class="source-row"><NTabs v-model:value="tab" type="line" size="large"><NTabPane name="local" :disabled="busy"><template #tab>我的壁纸</template></NTabPane><NTabPane name="online" :disabled="busy"><template #tab>在线资源</template></NTabPane></NTabs><div class="toolbar"><NInput :disabled="busy" v-model:value="query" clearable round placeholder="搜索素材" data-testid="wallpaper-search" @keyup.enter="page=1;load()"><template #prefix><Search :size="16"/></template></NInput><NButton :disabled="busy" v-if="tab==='online'" type="primary" round @click="page=1;load()">搜索</NButton></div><div class="source-actions"><NButton size="small" round data-testid="import-media" :disabled="busy" @click="importMedia">导入图片 / 视频</NButton></div></div>
<div class="chip-row"><div class="chip-group" role="group" aria-label="壁纸类型"><button v-for="type in typeOptions" :key="type.value" class="chip" :class="{on:media===type.value}" :disabled="busy" :aria-pressed="media===type.value" @click="media=type.value">{{type.label}}</button></div><span class="chip-divider"></span><div class="chip-scroll" role="group" :aria-label="tab==='online'?'分类':'本地来源'"><template v-if="tab==='online'"><button v-for="option in categoryOptions" :key="option.value" class="chip" :class="{on:category===option.value}" :disabled="busy" :aria-pressed="category===option.value" @click="category=option.value">{{option.label}}</button></template><template v-else><button v-for="option in sourceOptions" :key="option.value" class="chip" :class="{on:localSource===option.value}" :disabled="busy" :aria-pressed="localSource===option.value" @click="localSource=option.value">{{option.label}}</button></template></div></div>
</div>
<NAlert v-if="error" type="error" class="feed-alert">{{error}}<NButton size="small" @click="load">重试</NButton></NAlert>
<NAlert v-if="tab==='online'&&!store.desktop" type="info">在线资源需要桌面应用连接真实网站。浏览器预览可查看我的壁纸。</NAlert>
<div class="workspace" :class="{'has-selection':selection}"><div class="gallery-area"><div class="gallery-scroll"><NSpin :show="loading"><div class="gallery"><button v-if="tab==='local'" class="wallpaper-card native-wallpaper" :disabled="busy" :class="{selected:store.settings.wallpaperId==='companion'}" data-testid="wallpaper-card-native" @click="useNativeWallpaper"><div class="card-cover native-cover"><Monitor :size="64" :stroke-width="1.3"/><span class="type-badge">{{store.settings.wallpaperId==='companion'?'正在使用':'系统壁纸'}}</span></div><div class="card-info"><strong>使用原生壁纸</strong><span>恢复系统设置的桌面背景</span></div></button><button v-for="card in cards" :key="card.source+cardId(card)" class="wallpaper-card" :disabled="busy" :class="{selected:selection&&cardId(selection)===cardId(card)&&selection.source===card.source}" :data-testid="'wallpaper-card-'+cardId(card)" @click="select(card)" @contextmenu.prevent="openContext($event,card)"><div class="card-cover"><MediaPreview :kind="cardKind(card)" :source="sources(card).video" :poster="sources(card).poster" :title="cardTitle(card)"/><span class="type-badge">{{cardKind(card)==='video'?'视频':cardKind(card)==='canvas'?'动画':'图片'}}</span></div><div class="card-info"><strong>{{cardTitle(card)}}</strong><span class="muted">{{card.source==='online'?card.item.author.name:card.item.installed?'本地可用':'订阅资源'}}</span></div></button></div><NEmpty v-if="!cards.length&&!loading" :description="tab==='online'?'暂无匹配的在线资源':'还没有这一类壁纸'" class="empty-state"/></NSpin></div><div class="gallery-footer"><div v-if="tab==='online'" class="pagination"><span class="muted">共 {{feed.total}} 张</span><NPagination :disabled="busy" :page="page" :page-count="page+(feed.has_more?1:0)" @update:page="page=$event"/></div><p v-else class="gallery-note muted">{{cards.length}} 个素材</p></div></div>
<Transition name="drawer"><aside v-if="selection" class="inspector" data-testid="wallpaper-inspector"><div class="inspector-heading"><span class="muted">预览与设置</span><NButton text data-testid="cancel-selection" :disabled="busy" @click="cancel">取消选择</NButton></div><div class="inspector-scroll" data-testid="inspector-scroll"><div class="inspector-preview" :class="{viewable:canView && kind!=='video'}" data-testid="inspector-preview" :title="canView && kind!=='video' ? '点击全屏查看' : undefined" @click="kind!=='video' && openViewer()"><MediaPreview :key="selection.source+cardId(selection)" :kind="kind || ''" :source="preview.video" :poster="preview.poster" :title="title" controls :fit="draft.fit" :rate="draft.playbackRate"/><button v-if="canView" type="button" class="preview-expand" data-testid="open-fullscreen" aria-label="全屏查看" title="全屏查看" @click.stop="openViewer"><Maximize2 :size="16"/></button></div><h2>{{title}}</h2><p class="muted">{{selection.source==='local'?selection.item.definition.description:selection.item.description}}</p><div class="draft-note">修改后点击“应用到桌面”生效。</div>
<div v-if="kind!=='canvas'" class="field"><label>画面适配</label><NSelect :disabled="busy" :value="draft.fit" :options="fits" data-testid="wallpaper-fit" @update:value="setFit"/></div><div v-if="kind==='video'" class="field"><label>播放速度 <span>{{draft.playbackRate}}×</span></label><NSlider :disabled="busy" v-model:value="draft.playbackRate" :min="0.25" :max="2" :step="0.25"/><small class="muted">静音循环播放</small></div>
<NCollapse v-if="current?.controls?.length"><NCollapseItem title="动画参数" name="controls"><div v-for="control in current.controls" :key="control.key" class="field"><label>{{control.label}}</label><NSlider :disabled="busy" v-if="control.type==='range'" :value="Number(draft.values[control.key])" :min="control.min" :max="control.max" :step="control.step" @update:value="draft.values[control.key]=$event"/><NSwitch :disabled="busy" v-else-if="control.type==='boolean'" :value="Boolean(draft.values[control.key])" @update:value="draft.values[control.key]=$event"/><NColorPicker :disabled="busy" v-else-if="control.type==='color'" :value="String(draft.values[control.key])" :show-alpha="false" @update:value="draft.values[control.key]=$event"/><NSelect :disabled="busy" v-else :value="String(draft.values[control.key])" :options="(control.options||[]).map(value=>({label:value,value}))" @update:value="draft.values[control.key]=$event"/></div></NCollapseItem></NCollapse>
</div><footer class="inspector-actions" data-testid="inspector-actions"><div v-if="busy" class="download-feedback" role="status" data-testid="wallpaper-progress"><span>{{progressPercent===null ? "正在准备壁纸文件…" : progressPercent>=100 ? "下载完成，正在校验 / 应用…" : `正在下载 ${progressPercent}%`}}</span><progress v-if="progress" max="100" :value="progressPercent || 0"/><progress v-else max="100"/><NButton v-if="downloading" size="small" secondary data-testid="cancel-download" :loading="cancelling" :disabled="cancelling" @click="cancelDownload">{{cancelling?'正在取消…':'取消下载'}}</NButton></div><div class="actions"><NButton v-if="selection.source==='online'" block :loading="busy" :disabled="saved || busy" data-testid="save-wallpaper" @click="action(false)">{{saved?'已保存到我的壁纸':'保存到我的壁纸'}}</NButton><NButton v-if="kind==='canvas'" block :disabled="busy" @click="action(true,true)">独立窗口预览</NButton><NButton type="primary" size="large" block :loading="busy" data-testid="apply-wallpaper" :disabled="busy" @click="action(true)">应用到桌面</NButton><NPopconfirm v-if="selection.source==='local' && selection.item.source==='local'" positive-text="删除" negative-text="取消" @positive-click="deleteLocal(cardId(selection))"><template #trigger><NButton quaternary block type="error" :disabled="busy" data-testid="delete-wallpaper"><template #icon><Trash2 :size="14" /></template>删除这幅壁纸</NButton></template>确定要删除这幅壁纸吗？本地文件将被清除。</NPopconfirm></div></footer></aside></Transition>
</div>

<Teleport to="body"><Transition name="viewer"><div v-if="viewer && selection" class="fullscreen-viewer" role="dialog" aria-modal="true" :aria-label="`全屏查看：${title}`" data-testid="fullscreen-viewer" @click.self="viewer=false"><div class="fullscreen-media" @click.self="viewer=false"><MediaPreview :key="'full'+selection.source+cardId(selection)" :kind="kind || ''" :source="fullMedia.video" :poster="fullMedia.poster" :title="title" controls fit="contain" :rate="draft.playbackRate"/></div><div class="fullscreen-bar"><span class="fullscreen-title">{{title}}</span><button ref="closeViewer" type="button" class="fullscreen-close" data-testid="close-fullscreen" aria-label="退出全屏" title="退出全屏（Esc）" @click="viewer=false"><X :size="20"/></button></div></div></Transition></Teleport>
<Teleport to="body"><template v-if="context"><div class="context-dismiss" @pointerdown="context=null" @contextmenu.prevent="context=null"></div><div class="material-context" role="menu" data-testid="material-context-menu" :style="{left:context.x+'px',top:context.y+'px'}" @keydown.esc="context=null"><button v-for="item in context.items" :key="item.key" role="menuitem" :disabled="item.disabled" @click="chooseContext(item.key)">{{item.label}}</button></div></template></Teleport>
</section>
</template>
<style scoped>
.wallpaper-workbench{height:100%;width:100%;max-width:none;min-height:0;margin:0;display:flex;flex-direction:column;overflow:hidden}
.workbench-tools{flex:none;padding:22px 28px 6px;display:grid;gap:16px}
.source-row{display:flex;align-items:center;gap:28px}
.source-row>.n-tabs{width:auto;flex:none}
.source-row :deep(.n-tab-pane){display:none}
.source-row :deep(.n-tabs-tab){font-size:22px;font-weight:600;color:var(--ui-text-tertiary);padding:4px 0}
.source-row :deep(.n-tabs-tab.n-tabs-tab--active){color:var(--ui-text-primary);font-weight:700}
.source-row :deep(.n-tabs-tab-pad){width:28px}
.source-row :deep(.n-tabs-bar){background:var(--ui-primary);height:4px;border-radius:2px}
.source-row :deep(.n-tabs-nav.n-tabs-nav--line-type .n-tabs-nav-scroll-content){border-bottom-color:transparent}
.toolbar{display:flex;align-items:center;gap:8px;flex:1;max-width:460px}
.toolbar>.n-input{flex:1;--n-color:var(--ui-surface)!important;--n-color-focus:var(--ui-surface)!important;--n-border:1px solid transparent!important}
.source-actions{margin-left:auto;display:flex;gap:10px}
.chip-row{display:flex;align-items:center;gap:14px;min-width:0}
.chip{border:0;border-radius:999px;height:34px;padding:0 18px;background:var(--ui-surface);color:var(--ui-text-secondary);font-size:13px;cursor:pointer;white-space:nowrap;flex:none}
.chip:hover:not(.on):not(:disabled){color:var(--ui-text-primary)}
.chip.on{background:var(--ui-primary);color:#fff;font-weight:650}
.chip-group{display:flex;gap:2px;padding:3px;background:var(--ui-surface);border-radius:999px;flex:none}
.chip-group .chip{height:28px;padding:0 14px;background:transparent}
.chip-group .chip.on{background:var(--ui-text-primary);color:var(--ui-surface)}
.chip-divider{width:1px;height:20px;background:var(--ui-border);flex:none}
.chip-scroll{display:flex;gap:8px;overflow-x:auto;min-width:0;scrollbar-width:none;padding:2px 0}
.chip-scroll::-webkit-scrollbar{display:none}

.workspace{position:relative;display:grid;grid-template-columns:minmax(0,1fr);flex:1;min-height:0;overflow:hidden}
.gallery-area{display:flex;flex-direction:column;min-height:0;min-width:0}
.gallery-scroll{flex:1;min-height:0;overflow:auto;padding:14px 28px 20px;overscroll-behavior:none;scroll-behavior:auto;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent}
.gallery{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));align-content:start;gap:var(--ui-card-gap)}
.wallpaper-card{position:relative;display:block;width:100%;text-align:left;padding:0;border:0;background:var(--ui-surface);border-radius:var(--ui-radius-card);overflow:hidden;cursor:pointer;color:#fff;isolation:isolate;transition:transform 200ms cubic-bezier(.2,.8,.2,1),box-shadow 200ms}
.wallpaper-card:hover{transform:translateY(-3px);box-shadow:0 14px 30px var(--ui-shadow-hover)}
.wallpaper-card.selected{outline:3px solid var(--ui-primary);outline-offset:2px}
.card-cover{aspect-ratio:16/10;position:relative}
.native-cover{display:grid;place-items:center;background:var(--ui-primary-soft);color:var(--ui-primary);padding-bottom:36px}
.native-wallpaper .card-info{background:none;color:var(--ui-text-primary);padding-top:14px}
.native-wallpaper .card-info span{color:var(--ui-text-secondary)}
.type-badge{position:absolute;left:12px;top:12px;background:rgba(0,0,0,.42);backdrop-filter:blur(8px);color:#fff;border-radius:999px;padding:3px 11px;font-size:12px;font-weight:600}
.card-info{position:absolute;left:0;right:0;bottom:0;padding:40px 16px 14px;display:grid;gap:3px;background:linear-gradient(transparent,rgba(0,0,0,.62))}
.card-info strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:15px;font-weight:650}
.card-info span{font-size:12px;color:rgba(255,255,255,.82)}
.gallery-footer{flex:none;min-height:44px;padding:6px 28px 14px}
.pagination{display:flex;align-items:center;justify-content:space-between;font-size:13px}
.gallery-note{font-size:13px;margin:2px 0}

.inspector{position:absolute;z-index:20;top:8px;right:20px;bottom:16px;width:400px;max-width:calc(100% - 40px);display:flex;flex-direction:column;min-height:0;overflow:hidden;background:var(--ui-surface);border-radius:24px;box-shadow:0 24px 64px var(--ui-shadow-pop),0 2px 10px var(--ui-shadow-hover)}
.drawer-enter-active,.drawer-leave-active{transition:transform 260ms cubic-bezier(.2,.8,.2,1),opacity 200ms}
.drawer-enter-from,.drawer-leave-to{transform:translateX(48px);opacity:0}
.inspector-heading{height:52px;flex:none;display:flex;justify-content:space-between;align-items:center;padding:0 20px;font-size:13px}
.inspector-scroll{flex:1;min-height:0;overflow:auto;padding:0 20px 16px;overscroll-behavior:none;scroll-behavior:auto;scrollbar-width:thin}
.inspector-preview{position:relative;aspect-ratio:16/10;overflow:hidden;background:var(--ui-surface-alt);border-radius:16px}
.inspector-preview.viewable{cursor:zoom-in}
.preview-expand{position:absolute;top:10px;right:10px;display:grid;place-items:center;width:32px;height:32px;padding:0;border:0;border-radius:10px;color:#fff;background:rgba(15,23,42,.55);cursor:pointer;opacity:.85;transition:opacity .15s,background .15s}
.preview-expand:hover,.preview-expand:focus-visible{opacity:1;background:rgba(15,23,42,.75)}
.preview-expand:focus-visible,.fullscreen-close:focus-visible{outline:2px solid #fff;outline-offset:2px}
.fullscreen-viewer{position:fixed;inset:0;z-index:2500;display:flex;flex-direction:column;background:rgba(5,8,15,.94)}
.fullscreen-media{flex:1;min-height:0;padding:56px 24px 24px}
.fullscreen-media :deep(.media-preview){background:transparent}
.fullscreen-bar{position:absolute;top:0;left:0;right:0;display:flex;align-items:center;gap:12px;padding:12px 16px;color:#f8fafc;background:linear-gradient(rgba(0,0,0,.5),transparent)}
.fullscreen-title{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:14px}
.fullscreen-close{display:grid;place-items:center;width:36px;height:36px;padding:0;border:0;border-radius:50%;color:#fff;background:rgba(255,255,255,.14);cursor:pointer}
.fullscreen-close:hover{background:rgba(255,255,255,.26)}
.viewer-enter-active,.viewer-leave-active{transition:opacity .18s ease}
.viewer-enter-from,.viewer-leave-to{opacity:0}
@media (prefers-reduced-motion:reduce){.viewer-enter-active,.viewer-leave-active{transition:none}}
.inspector h2{font-size:20px;font-weight:700;margin:16px 0 6px}
.inspector-scroll>p{font-size:13px;color:var(--ui-muted);line-height:1.6;margin:0;max-height:42px;overflow:hidden}
.draft-note{font-size:12px;color:var(--ui-text-tertiary);margin:14px 0;background:var(--ui-surface-alt);padding:8px 12px;border-radius:10px}
.field{margin:14px 0}
.field>label{display:flex;justify-content:space-between;font-size:13px;font-weight:600;margin-bottom:8px}
.field small{font-size:12px}
.inspector-actions{flex:none;padding:12px 20px 20px}
.actions{display:grid;gap:10px}
.download-feedback{display:grid;gap:6px;margin-bottom:12px;color:var(--ui-primary-text);font-size:12px}
.download-feedback progress{width:100%;height:5px;accent-color:var(--ui-primary)}
.download-feedback .n-button{justify-self:end}
.feed-alert{flex:none;max-height:84px;overflow:auto;border-radius:0}

.context-dismiss{position:fixed;inset:0;z-index:3000}
.material-context{position:fixed;z-index:3001;width:216px;padding:6px;background:var(--ui-surface);border:0;border-radius:14px;box-shadow:0 12px 32px var(--ui-shadow-pop);font-size:13px}
.material-context button{display:block;width:100%;height:36px;border:0;border-radius:9px;background:none;text-align:left;padding:0 12px;color:var(--ui-text-primary);cursor:pointer;font:inherit}
.material-context button:hover{background:var(--ui-primary-soft)}
.material-context button:disabled{color:var(--ui-text-tertiary);cursor:default}
</style>
