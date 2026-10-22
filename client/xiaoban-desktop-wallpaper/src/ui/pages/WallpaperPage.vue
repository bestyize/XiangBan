<script setup lang="ts">
import ResourceShareActions from "../components/ResourceShareActions.vue";
import {computed,ref,watch,nextTick,onBeforeUnmount,onActivated,onDeactivated} from "vue";
import {NButton,NSelect,NSlider,NSwitch,NEmpty,NAlert,NPagination,NCollapse,NCollapseItem,NColorPicker,NSpin,NPopconfirm} from "naive-ui";
import {Search,Maximize2,X,Monitor,Trash2,Upload,Sparkles,ChevronDown,Check,Image as ImageIcon} from "lucide-vue-next";
import {invoke,wallpaperDefaults,type WallpaperLibrary,type WallpaperOptions} from "../../bridge.ts";
import {siteMedia,siteFullMedia,isAnimatedWallpaper} from "../../wallpaper-media.ts";
import type {ControlStore} from "../control-types.ts";
import {useWallpaperWorkspace,type Selection} from "../wallpaper/useWallpaperWorkspace.ts";
import MediaPreview from "../wallpaper/MediaPreview.vue";
import StudioActions from "../components/StudioActions.vue";
import {createWallpaperContextMenu,type ContextAction} from "../wallpaper/context-menu.ts";
const props=defineProps<{store:ControlStore}>();
const {progress,progressPercent,saved,tab,media,query,category,page,localSource,selection,draft,busy,downloading,cancelling,loading,error,feed,categories,kind,select,cancel,cancelDownload,load,localItems,action}=useWallpaperWorkspace(props.store);
const current=computed(()=>selection.value?.source==='local'?selection.value.item.definition:null);
const title=computed(()=>selection.value?.source==='local'?selection.value.item.definition.title:selection.value?.item.title || '');
/** 官方 = the wallpapers shipped with the app; 我的 = imported and subscribed ones; 在线资源 = the website feed. */
const scope=ref<'official'|'mine'|'online'>('official');
watch(scope,value=>{tab.value=value==='online'?'online':'local';},{flush:'sync'});
const scopes=[{id:'official' as const,label:'官方'},{id:'mine' as const,label:'我的'},{id:'online' as const,label:'在线资源'}];
const scoped=computed(()=>localItems.value.filter(item=>scope.value==='official'?item.source==='builtin':item.source!=='builtin'));
const cards=computed<Selection[]>(()=>tab.value==='online'?feed.items.map(item=>({source:'online',item})):scoped.value.map(item=>({source:'local',item})));
const active=(card:Selection)=>card.source==='local'&&props.store.settings.wallpaperId===card.item.definition.id;
const isSelected=(card:Selection)=>!!selection.value&&selection.value.source===card.source&&cardId(selection.value)===cardId(card);
const kindLabel=(card:Selection)=>{const kind=cardKind(card);return kind==='video'?'视频':isAnimatedWallpaper(kind)?'动画':'图片';};
/** 预览 opens the settings drawer; 设为壁纸 applies with the wallpaper's own defaults at once. */
function inspect(card:Selection){if(!busy.value)select(card);}
async function applyNow(card:Selection){if(busy.value)return;select(card);await action(true);}
const importMenu=ref(false);
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
const types=[{label:'全部',value:''},{label:'动画',value:'canvas'},{label:'视频',value:'video'},{label:'图片',value:'image'}];
const typeOptions=computed(()=>tab.value==='online'?types.filter(type=>type.value!=='canvas'):types);
const categoryOptions=computed(()=>[{label:'全部',value:''},...categories.value.map(c=>({label:c.name,value:c.id}))]);
watch(tab,value=>{if(value==='online'&&media.value==='canvas')media.value='';});
const fits=[{label:'铺满屏幕',value:'cover'},{label:'完整显示',value:'contain'},{label:'拉伸适配',value:'fill'}];
async function importedResource(result:{id:string}){await props.store.refreshLibrary();scope.value='mine';media.value='';const item=props.store.library.items.find(e=>e.definition.id===result.id);if(item)select({source:'local',item});}
async function importMedia(){try {const id=await props.store.importMedia();if(id){scope.value='mine';importMenu.value=false;const item=props.store.library.items.find(e=>e.definition.id===id);if(item)select({source:'local',item});}}catch(e){props.store.notify(e,'error');}}
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
  if(isAnimatedWallpaper(cardKind(card)))items.push({key:'preview',label:'独立窗口预览',disabled:busy.value});
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
onDeactivated(()=>{window.removeEventListener("keydown",onKey);viewer.value=false;context.value=null;});
onActivated(()=>window.addEventListener("keydown",onKey));
</script>
<template>
<section class="wallpaper-workbench" data-testid="wallpapers-page">
<StudioActions>
<div class="import-wrap"><button type="button" class="xb-btn" aria-haspopup="menu" :aria-expanded="importMenu" :disabled="busy" data-testid="wallpapers-import" @click="importMenu=!importMenu"><Upload :size="14"/>导入<ChevronDown :size="13"/></button>
<template v-if="importMenu"><div class="menu-dismiss" @pointerdown="importMenu=false"></div><div class="xb-menu import-menu" role="menu" @click.capture="importMenu=false"><button type="button" role="menuitem" class="xb-menu-item" data-testid="import-media" :disabled="busy" @click="importMedia"><ImageIcon :size="15"/>导入图片 / 视频</button><ResourceShareActions :store="store" kind="wallpaper" variant="menu" import-label="导入资源包" :disabled="busy" @imported="importedResource"/></div></template></div>
<button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="wallpapers-create" @click="store.startCreation('wallpaper')"><Sparkles :size="14"/>让小伴创作</button>
</StudioActions>
<div class="wp-scroll" data-testid="wallpapers-scroll">
<div class="filters">
<div class="xb-seg" role="group" aria-label="壁纸来源"><button v-for="item in scopes" :key="item.id" type="button" :disabled="busy" :aria-pressed="scope===item.id" :data-testid="`wallpaper-scope-${item.id}`" @click="scope=item.id">{{item.label}}</button></div>
<span class="divider"></span>
<div class="chips" role="group" aria-label="壁纸类型"><button v-for="type in typeOptions" :key="type.value" type="button" class="fchip" :class="{on:media===type.value}" :disabled="busy" :aria-pressed="media===type.value" @click="media=type.value">{{type.label}}</button></div>
<div class="filters-end"><label class="xb-search search-box"><Search :size="14"/><input v-model="query" :disabled="busy" placeholder="搜索素材" data-testid="wallpaper-search" @keyup.enter="page=1;load()"/></label><button v-if="tab==='online'" type="button" class="xb-btn is-sm" :disabled="busy" @click="page=1;load()">搜索</button><span class="count">{{tab==='online'?feed.total:cards.length+(scope==='official'?1:0)}} 张</span></div>
</div>
<div v-if="tab==='online'&&categoryOptions.length>1" class="chips cat-row" role="group" aria-label="分类"><button v-for="option in categoryOptions" :key="option.value" type="button" class="fchip" :class="{on:category===option.value}" :disabled="busy" :aria-pressed="category===option.value" @click="category=option.value">{{option.label}}</button></div>
<NAlert v-if="error" type="error" class="feed-alert">{{error}}<NButton size="small" @click="load">重试</NButton></NAlert>
<NAlert v-if="tab==='online'&&!store.desktop" type="info" class="feed-alert">在线资源需要桌面应用连接真实网站。浏览器预览可查看官方与我的壁纸。</NAlert>
<NSpin :show="tab==='online'&&loading"><div class="gallery">
<article v-if="scope==='official'" class="wc" :class="{used:store.settings.wallpaperId==='companion'}"><div class="im"><div class="wallpaper-card native-wallpaper" role="button" tabindex="0" :aria-disabled="busy" data-testid="wallpaper-card-native" @click="useNativeWallpaper" @keydown.enter.prevent="useNativeWallpaper" @keydown.space.prevent="useNativeWallpaper"><div class="native-cover"><Monitor :size="44" :stroke-width="1.3"/></div><span v-if="store.settings.wallpaperId==='companion'" class="lb used-label"><Check :size="11"/>使用中</span></div></div><div class="nm"><b>使用原生壁纸</b><span class="wc-type">恢复系统背景</span></div></article>
<article v-for="card in cards" :key="card.source+cardId(card)" class="wc" :class="{used:active(card),selected:isSelected(card)}">
<div class="im"><div class="wallpaper-card" role="button" tabindex="0" :aria-disabled="busy" :data-testid="'wallpaper-card-'+cardId(card)" @click="select(card)" @keydown.enter.prevent="select(card)" @keydown.space.prevent="select(card)" @contextmenu.prevent="openContext($event,card)"><MediaPreview :kind="cardKind(card)" :source="sources(card).video" :poster="sources(card).poster" :title="cardTitle(card)"/><span v-if="active(card)" class="lb used-label"><Check :size="11"/>使用中</span></div>
<div v-if="!active(card)" class="hover-actions"><button type="button" class="ha-btn" :disabled="busy" :data-testid="'wallpaper-preview-'+cardId(card)" @click.stop="inspect(card)">预览</button><button type="button" class="ha-btn primary" :disabled="busy" :data-testid="'wallpaper-use-'+cardId(card)" @click.stop="applyNow(card)">设为壁纸</button></div></div>
<div class="nm"><b>{{cardTitle(card)}}</b><span class="wc-type">{{kindLabel(card)}}</span></div></article>
</div>
<div v-if="!cards.length&&!loading" class="xb-empty"><span class="xb-empty-icon"><ImageIcon :size="24"/></span><h3>{{tab==='online'?'暂无匹配的在线资源':scope==='mine'?'还没有自己的壁纸':'没有这一类壁纸'}}</h3><p>{{scope==='mine'?'点右上角“导入”添加图片或视频，也可以让小伴创作一个动画壁纸。':'换个类型或关键词试试。'}}</p></div></NSpin>
<div v-if="tab==='online'" class="pagination"><span class="muted">共 {{feed.total}} 张</span><NPagination :disabled="busy" :page="page" :page-count="page+(feed.has_more?1:0)" @update:page="page=$event"/></div>
</div>
<Transition name="drawer"><aside v-if="selection" class="inspector" data-testid="wallpaper-inspector"><div class="inspector-heading"><span class="muted">预览与设置</span><NButton text data-testid="cancel-selection" :disabled="busy" @click="cancel">取消选择</NButton></div><div class="inspector-scroll" data-testid="inspector-scroll"><div class="inspector-preview" :class="{viewable:canView && kind!=='video'}" data-testid="inspector-preview" :title="canView && kind!=='video' ? '点击全屏查看' : undefined" @click="kind!=='video' && openViewer()"><MediaPreview :key="selection.source+cardId(selection)" :kind="kind || ''" :source="preview.video" :poster="preview.poster" :title="title" controls :fit="draft.fit" :rate="draft.playbackRate"/><button v-if="canView" type="button" class="preview-expand" data-testid="open-fullscreen" aria-label="全屏查看" title="全屏查看" @click.stop="openViewer"><Maximize2 :size="16"/></button></div><h2>{{title}}</h2><p class="muted">{{selection.source==='local'?selection.item.definition.description:selection.item.description}}</p><div class="draft-note">修改后点击“应用到桌面”生效。</div>
<div v-if="!isAnimatedWallpaper(kind || '')" class="field"><label>画面适配</label><NSelect :disabled="busy" :value="draft.fit" :options="fits" data-testid="wallpaper-fit" @update:value="setFit"/></div><div v-if="kind==='video'" class="field"><label>播放速度 <span>{{draft.playbackRate}}×</span></label><NSlider :disabled="busy" v-model:value="draft.playbackRate" :min="0.25" :max="2" :step="0.25"/><small class="muted">静音循环播放</small></div>
<NCollapse v-if="current?.controls?.length"><NCollapseItem title="动画参数" name="controls"><div v-for="control in current.controls" :key="control.key" class="field"><label>{{control.label}}</label><NSlider :disabled="busy" v-if="control.type==='range'" :value="Number(draft.values[control.key])" :min="control.min" :max="control.max" :step="control.step" @update:value="draft.values[control.key]=$event"/><NSwitch :disabled="busy" v-else-if="control.type==='boolean'" :value="Boolean(draft.values[control.key])" @update:value="draft.values[control.key]=$event"/><NColorPicker :disabled="busy" v-else-if="control.type==='color'" :value="String(draft.values[control.key])" :show-alpha="false" @update:value="draft.values[control.key]=$event"/><NSelect :disabled="busy" v-else :value="String(draft.values[control.key])" :options="(control.options||[]).map(value=>({label:value,value}))" @update:value="draft.values[control.key]=$event"/></div></NCollapseItem></NCollapse>
</div><footer class="inspector-actions" data-testid="inspector-actions"><div v-if="busy" class="download-feedback" role="status" data-testid="wallpaper-progress"><span>{{progressPercent===null ? "正在准备壁纸文件…" : progressPercent>=100 ? "下载完成，正在校验 / 应用…" : `正在下载 ${progressPercent}%`}}</span><progress v-if="progress" max="100" :value="progressPercent || 0"/><progress v-else max="100"/><NButton v-if="downloading" size="small" secondary data-testid="cancel-download" :loading="cancelling" :disabled="cancelling" @click="cancelDownload">{{cancelling?'正在取消…':'取消下载'}}</NButton></div><div class="actions"><ResourceShareActions v-if="selection.source==='local' && selection.item.source==='local' && kind==='web'" :store="store" kind="wallpaper" :id="cardId(selection)" :disabled="busy"/><NButton v-if="selection.source==='online'" block :loading="busy" :disabled="saved || busy" data-testid="save-wallpaper" @click="action(false)">{{saved?'已保存到我的壁纸':'保存到我的壁纸'}}</NButton><NButton v-if="isAnimatedWallpaper(kind || '')" block :disabled="busy" @click="action(true,true)">独立窗口预览</NButton><NButton type="primary" size="large" block :loading="busy" data-testid="apply-wallpaper" :disabled="busy" @click="action(true)">应用到桌面</NButton><NPopconfirm v-if="selection.source==='local' && selection.item.source==='local'" positive-text="删除" negative-text="取消" @positive-click="deleteLocal(cardId(selection))"><template #trigger><NButton quaternary block type="error" :disabled="busy" data-testid="delete-wallpaper"><template #icon><Trash2 :size="14" /></template>删除这幅壁纸</NButton></template>确定要删除这幅壁纸吗？本地文件将被清除。</NPopconfirm></div></footer></aside></Transition>

<Teleport to="body"><Transition name="viewer"><div v-if="viewer && selection" class="fullscreen-viewer" role="dialog" aria-modal="true" :aria-label="`全屏查看：${title}`" data-testid="fullscreen-viewer" @click.self="viewer=false"><div class="fullscreen-media" @click.self="viewer=false"><MediaPreview :key="'full'+selection.source+cardId(selection)" :kind="kind || ''" :source="fullMedia.video" :poster="fullMedia.poster" :title="title" controls fit="contain" :rate="draft.playbackRate"/></div><div class="fullscreen-bar"><span class="fullscreen-title">{{title}}</span><button ref="closeViewer" type="button" class="fullscreen-close" data-testid="close-fullscreen" aria-label="退出全屏" title="退出全屏（Esc）" @click="viewer=false"><X :size="20"/></button></div></div></Transition></Teleport>
<Teleport to="body"><template v-if="context"><div class="context-dismiss" @pointerdown="context=null" @contextmenu.prevent="context=null"></div><div class="material-context" role="menu" data-testid="material-context-menu" :style="{left:context.x+'px',top:context.y+'px'}" @keydown.esc="context=null"><button v-for="item in context.items" :key="item.key" role="menuitem" :disabled="item.disabled" @click="chooseContext(item.key)">{{item.label}}</button></div></template></Teleport>
</section>
</template>
<style scoped>
.wallpaper-workbench{position:relative;height:100%;width:100%;min-height:0;display:flex;flex-direction:column;overflow:hidden}
.wp-scroll{flex:1;min-height:0;overflow:auto;padding:18px 28px 28px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.import-wrap{position:relative}
.menu-dismiss{position:fixed;inset:0;z-index:30}
.import-menu{position:absolute;right:0;top:38px;z-index:31;min-width:180px}
.filters{display:flex;align-items:center;gap:8px;min-width:0}
.divider{width:1px;height:16px;background:var(--ui-border-strong);margin:0 6px;flex:none}
.chips{display:flex;align-items:center;gap:4px;min-width:0}
.cat-row{margin-top:10px;flex-wrap:wrap}
.fchip{height:28px;padding:0 11px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);font-size:12.5px;cursor:pointer;white-space:nowrap}
.fchip:hover:not(.on):not(:disabled){color:var(--ui-text-primary);background:var(--ui-surface-hover)}
.fchip.on{background:var(--ui-soft);color:var(--ui-ink);font-weight:500}
.filters-end{margin-left:auto;display:flex;align-items:center;gap:8px;flex:none}
.search-box{width:170px}
.count{font-size:12px;color:var(--ui-text-secondary);white-space:nowrap}
.feed-alert{margin-top:12px;border-radius:10px}
.gallery{margin-top:14px;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:20px 16px}
.wc{display:flex;flex-direction:column;gap:9px;min-width:0}
.im{position:relative;aspect-ratio:16/10;border-radius:12px;box-shadow:0 0 0 1px var(--ui-line),0 2px 6px var(--ui-shadow-hover);background:var(--ui-surface-alt)}
.wc.used .im{box-shadow:0 0 0 2px var(--ui-surface),0 0 0 4px var(--ui-btn)}
.wc.selected:not(.used) .im{box-shadow:0 0 0 2px var(--ui-surface),0 0 0 4px var(--ui-border-strong)}
.wallpaper-card{position:absolute;inset:0;border-radius:12px;overflow:hidden;cursor:pointer;isolation:isolate;outline:none}
.wallpaper-card:focus-visible{box-shadow:0 0 0 3px var(--ui-btn)}
.wallpaper-card[aria-disabled=true]{cursor:default}
.native-cover{width:100%;height:100%;display:grid;place-items:center;background:var(--ui-soft);color:var(--ui-ink)}
.lb{position:absolute;top:8px;left:8px;z-index:2;height:20px;padding:0 7px;border-radius:6px;background:rgba(0,0,0,.42);backdrop-filter:blur(8px);color:#fff;font-size:11px;font-weight:500;display:inline-flex;align-items:center;gap:4px}
.lb.used-label{background:var(--ui-btn)}
.hover-actions{position:absolute;left:0;right:0;bottom:0;z-index:3;display:flex;justify-content:flex-end;align-items:flex-end;gap:6px;padding:10px;border-radius:0 0 12px 12px;background:linear-gradient(to top,rgba(0,0,0,.45),rgba(0,0,0,0) 90%);opacity:0;transition:opacity .15s;pointer-events:none}
.wc:hover .hover-actions,.wc:focus-within .hover-actions{opacity:1;pointer-events:auto}
.ha-btn{height:28px;padding:0 11px;border-radius:8px;border:0;background:rgba(255,255,255,.9);backdrop-filter:blur(8px);color:#1D1D1F;font-size:12.5px;cursor:pointer}
.ha-btn.primary{background:linear-gradient(var(--ui-btn-hi),var(--ui-btn));color:#fff;font-weight:500}
.nm{display:flex;align-items:baseline;gap:8px;padding:0 2px;min-width:0}
.nm b{font-size:14px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.wc-type{font-size:12px;color:var(--ui-text-secondary);white-space:nowrap}
.pagination{margin-top:18px;display:flex;align-items:center;justify-content:space-between;font-size:13px}

.inspector{position:absolute;z-index:20;top:8px;right:16px;bottom:12px;width:380px;max-width:calc(100% - 32px);display:flex;flex-direction:column;min-height:0;overflow:hidden;background:var(--ui-surface-raised);border-radius:16px;box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog)}
.drawer-enter-active,.drawer-leave-active{transition:transform 260ms cubic-bezier(.2,.8,.2,1),opacity 200ms}
.drawer-enter-from,.drawer-leave-to{transform:translateX(48px);opacity:0}
.inspector-heading{height:48px;flex:none;display:flex;justify-content:space-between;align-items:center;padding:0 18px;font-size:13px}
.inspector-scroll{flex:1;min-height:0;overflow:auto;padding:0 18px 16px;overscroll-behavior:none;scroll-behavior:auto;scrollbar-width:thin}
.inspector-preview{position:relative;aspect-ratio:16/10;overflow:hidden;background:var(--ui-surface-alt);border-radius:12px}
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
.inspector h2{font-size:17px;font-weight:600;margin:14px 0 6px}
.inspector-scroll>p{font-size:13px;color:var(--ui-text-secondary);line-height:1.6;margin:0;max-height:42px;overflow:hidden}
.draft-note{font-size:12px;color:var(--ui-text-tertiary);margin:14px 0;background:var(--ui-surface-sunken);padding:8px 12px;border-radius:10px}
.field{margin:14px 0}
.field>label{display:flex;justify-content:space-between;font-size:13px;font-weight:500;margin-bottom:8px}
.field small{font-size:12px}
.inspector-actions{flex:none;padding:12px 18px 18px}
.actions{display:grid;gap:8px}
.download-feedback{display:grid;gap:6px;margin-bottom:12px;color:var(--ui-ink);font-size:12px}
.download-feedback progress{width:100%;height:5px;accent-color:var(--ui-btn)}
.download-feedback .n-button{justify-self:end}

.context-dismiss{position:fixed;inset:0;z-index:3000}
.material-context{position:fixed;z-index:3001;width:216px;padding:5px;background:var(--ui-surface-raised);-webkit-backdrop-filter:blur(20px);backdrop-filter:blur(20px);border:0;border-radius:var(--ui-radius-menu);box-shadow:var(--ui-shadow-menu);font-size:13px}
.material-context button{display:block;width:100%;height:34px;border:0;border-radius:8px;background:none;text-align:left;padding:0 10px;color:var(--ui-text-primary);cursor:pointer;font:inherit}
.material-context button:hover{background:var(--ui-surface-hover)}
.material-context button:disabled{color:var(--ui-text-tertiary);cursor:default}
@media(max-width:900px){.gallery{grid-template-columns:repeat(2,minmax(0,1fr))}.filters{flex-wrap:wrap}}
</style>

