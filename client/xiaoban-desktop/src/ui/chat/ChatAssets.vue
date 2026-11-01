<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ArrowLeft, Download, Ellipsis, ExternalLink, Files, FileText, Folder, FolderOpen, Image, LoaderCircle, MessageSquare, PanelsTopLeft, PanelRightClose, PanelRightOpen, Plus, Search, Upload, X, Copy } from 'lucide-vue-next';
import { invoke, listen } from '../../bridge.ts';
import type { Payload, Summary } from '../design/model.ts';
import type { ReplyBlock } from './replies.ts';
import { assetMeta, conversationAssets, type ConversationAsset, type LocalAsset } from './assets.ts';
import AssetThumbnail from './AssetThumbnail.vue';

const props = defineProps<{
  session: string | null; designs: Summary[]; blocks: ReplyBlock[]; open: boolean; pinned: boolean;
  anchor: HTMLElement | null; ensureSession: () => Promise<string | null>;
  notify: (message: unknown, type?: 'error' | 'info' | 'success') => void;
}>();
const emit = defineEmits<{ close: []; pin: []; count: [count: number]; design: [id: string]; locate: [block: string]; browseDesigns: [] }>();
const host = ref<HTMLElement>();
const files = ref<LocalAsset[]>([]), query = ref(''), tab = ref<'all'|'design'|'image'|'file'>('all');
const loading = ref(false), importing = ref(false), error = ref(''), importMenu = ref(false), menu = ref(''), busy = ref('');
const folder = ref<ConversationAsset | null>(null), children = ref<ConversationAsset[]>([]), truncated = ref(false), folderBusy = ref(false);
const assets = computed(() => conversationAssets(props.designs, files.value, props.blocks));
const tabs = computed(() => ([['all','全部'],['design','设计稿'],['image','图片'],['file','文件']] as const).map(([id,label])=>({id,label,count:assets.value.filter(a=>id==='all'||a.type===id).length})));
const filtered = computed(() => assets.value.filter(a => (tab.value==='all'||a.type===tab.value) && (!query.value.trim() || `${a.name} ${a.local?.path??a.id}`.toLowerCase().includes(query.value.trim().toLowerCase()))));
const shown = computed(() => folder.value ? children.value : filtered.value);
const activeMenu = computed(() => shown.value.find(a => a.key === menu.value));
const menuTop=ref(44);
function toggleMenu(asset:ConversationAsset,event:MouseEvent){
  const button=event.currentTarget as HTMLElement,box=host.value?.getBoundingClientRect();
  if(box)menuTop.value=Math.max(44,Math.min(button.getBoundingClientRect().bottom-box.top,box.height-190));
  menu.value=menu.value===asset.key?'':asset.key;importMenu.value=false;
}
let listEpoch = 0, folderEpoch = 0, disposed = false, off: (()=>void) | undefined;
watch(() => assets.value.length, count => emit('count', count), { immediate: true });

async function refresh() {
  const version = ++listEpoch, session = props.session;
  if (!session) { files.value=[]; loading.value=false; return; }
  loading.value=true;
  try {
    const result=await invoke<LocalAsset[]>('chat_assets_list',{sessionId:session});
    if(version===listEpoch&&!disposed) {files.value=Array.isArray(result)?result:[];error.value='';}
  } catch(e) {if(version===listEpoch&&!disposed)error.value=String(e);}
  finally {if(version===listEpoch&&!disposed)loading.value=false;}
}
watch(() => props.session, () => {
  folderEpoch++; folder.value=null;children.value=[];folderBusy.value=false;busy.value='';files.value=[];query.value='';menu.value='';importMenu.value=false;error.value='';tab.value='all';void refresh();
},{immediate:true});
// Tool completions and completed assistant messages change this key; streaming text does not re-read history.
watch(() => props.blocks.map(b=>[b.id,b.calls.filter(c=>c.status!=='running').map(c=>`${c.id}:${c.status}`).join(','),b.items.filter(i=>i.kind==='assistant'&&!i.streaming).map(i=>i.id+':'+i.text.length).join(',')].join('|')).join(';'),()=>void refresh());
watch(() => props.open, open=>{menu.value='';importMenu.value=false;if(open)void refresh();});
watch([tab,query],()=>{folderEpoch++;folder.value=null;children.value=[];folderBusy.value=false;menu.value='';});

function icon(asset: ConversationAsset) {return asset.design?PanelsTopLeft:asset.local?.kind==='folder'?Folder:asset.type==='image'?Image:FileText;}
function title(asset: ConversationAsset) {return [asset.local?.path??asset.name,assetMeta(asset),asset.local?.imported?'手动导入':'对话成果'].join(' · ');}
function nativeArgs(asset: ConversationAsset) {return {sessionId:props.session,id:asset.id,relative:asset.relative??''};}
async function openFolder(asset: ConversationAsset) {
  const version=++folderEpoch,session=props.session;folder.value=asset;children.value=[];truncated.value=false;folderBusy.value=true;menu.value='';error.value='';
  try {
    const result=await invoke<{entries:LocalAsset[];truncated:boolean}>('chat_asset_children',nativeArgs(asset));
    if(version!==folderEpoch||session!==props.session||disposed)return;
    children.value=result.entries.map(local=>({key:`${asset.id}:${local.path}`,id:asset.id,name:local.name,type:local.kind==='image'?'image':'file',updatedMs:local.updatedMs,local,relative:[asset.relative,local.name].filter(Boolean).join('/'),blockId:asset.blockId}));
    truncated.value=result.truncated;
  } catch(e) {if(version===folderEpoch&&!disposed)error.value=String(e);}
  finally {if(version===folderEpoch&&!disposed)folderBusy.value=false;}
}
async function action(asset: ConversationAsset, how: 'open'|'reveal'|'save'|'copy'|'locate' = 'open') {
  menu.value='';error.value='';
  if(how==='locate') {if(asset.blockId){emit('locate',asset.blockId);if(!props.pinned)emit('close');}return;}
  if(how==='copy') {try{await navigator.clipboard.writeText(asset.local!.path);props.notify('已复制路径','success');}catch(e){props.notify(e,'error');}return;}
  if(asset.local?.kind==='folder'&&how==='open') {await openFolder(asset);return;}
  if(asset.design) {emit('design',asset.id);if(!props.pinned)emit('close');return;}
  if(busy.value===asset.key)return;
  const session=props.session;busy.value=asset.key;
  try {
    const saved=asset.generated
      ? await invoke(how==='save'?'chat_generated_image_save':'chat_generated_image_open',{id:asset.id})
      : await invoke('chat_asset_open',{...nativeArgs(asset),action:how});
    if(how==='save'&&saved===true)props.notify('文件已保存','success');
  } catch(e) {if(session===props.session&&!disposed)error.value=String(e);}
  finally {if(!disposed&&session===props.session&&busy.value===asset.key)busy.value='';}
}
async function back() {
  const current=folder.value;if(!current)return;
  if(!current.relative){folderEpoch++;folder.value=null;children.value=[];return;}
  const relative=current.relative.split('/').slice(0,-1).join('/');
  const root=assets.value.find(a=>a.id===current.id);if(!root){folder.value=null;return;}
  await openFolder(relative?{...root,relative,name:relative.split('/').pop()!,local:{...root.local!,path:root.local!.path+'/'+relative}}:root);
}
async function importAsset(kind:'file'|'folder'|'design'|'new-design') {
  if(importing.value)return;
  importing.value=true;importMenu.value=false;let owner=props.session;error.value='';
  try {
    const session=await props.ensureSession();if(!session)return;owner=session;
    if(kind==='design'||kind==='new-design') {
      const result=await invoke<Payload|null>(kind==='design'?'design_import':'design_create',{session});
      if(result?.project&&props.session===session)emit('design',result.project.id);
    } else {
      const count=await invoke<number>('chat_assets_import',{sessionId:session,directory:kind==='folder'});
      if(count>0)props.notify(`已导入 ${count} 份资产`,'success');
      if(props.session===session)await refresh();
    }
  } catch(e) {if(props.session===owner)error.value=String(e);else props.notify(e,'error');}
  finally {if(!disposed)importing.value=false;}
}
function outside(event: PointerEvent) {
  const target=event.target as Node;
  if(!props.open||host.value?.contains(target)||props.anchor?.contains(target))return;
  menu.value='';importMenu.value=false;if(!props.pinned)emit('close');
}
function escape(event: KeyboardEvent) {
  if(event.key!=='Escape'||!props.open)return;
  if(menu.value)menu.value='';else if(importMenu.value)importMenu.value=false;else {emit('close');void nextTick(()=>props.anchor?.focus());}
  event.stopPropagation();
}
function tabKey(event:KeyboardEvent) {
  if(!['ArrowRight','ArrowLeft','Home','End'].includes(event.key))return;
  event.preventDefault();const index=tabs.value.findIndex(t=>t.id===tab.value),next=event.key==='Home'?0:event.key==='End'?3:(index+(event.key==='ArrowRight'?1:3))%4;
  tab.value=tabs.value[next].id;void nextTick(()=>host.value?.querySelector<HTMLButtonElement>(`[data-tab="${tab.value}"]`)?.focus());
}
onMounted(async()=>{
  document.addEventListener('pointerdown',outside);document.addEventListener('keydown',escape);
  const unlisten=await listen<{session:string}>('chat-assets-changed',payload=>{if(payload.session===props.session)void refresh();});
  if(disposed)unlisten();else off=unlisten;
});
onBeforeUnmount(()=>{disposed=true;listEpoch++;folderEpoch++;off?.();document.removeEventListener('pointerdown',outside);document.removeEventListener('keydown',escape);});
</script>

<template>
  <aside v-if="open" ref="host" class="chat-assets" :class="{'is-pinned':pinned}" data-testid="chat-assets" aria-label="对话资产">
    <header class="assets-head"><b>对话资产</b><span class="asset-count">{{ assets.length }}</span><span class="spacer" /><button type="button" class="xb-icon-btn is-sm" :title="pinned?'切换为浮层':'固定为侧栏'" :aria-label="pinned?'切换为浮层':'固定为侧栏'" @click="emit('pin')"><PanelRightClose v-if="pinned" :size="15" /><PanelRightOpen v-else :size="15" /></button><button type="button" class="xb-icon-btn is-sm" data-testid="chat-assets-import" aria-label="导入资产" :aria-expanded="importMenu" :disabled="importing" title="导入文件、文件夹或设计稿" @click="importMenu=!importMenu;menu=''"><LoaderCircle v-if="importing" :size="15" /><Plus v-else :size="15" /></button><button type="button" class="xb-icon-btn is-sm" aria-label="收起对话资产" @click="emit('close')"><X :size="15" /></button></header>
    <div v-if="importMenu" class="xb-menu import-menu" role="menu" aria-label="导入资产">
      <button class="xb-menu-item" role="menuitem" title="关联本机文件，保留原位置；不自动发送给 AI" aria-description="关联本机文件，保留原位置；不自动发送给 AI" @click="importAsset('file')"><Upload :size="14" />导入文件</button>
      <button class="xb-menu-item" role="menuitem" title="关联本机文件夹，保留原位置" aria-description="关联本机文件夹，保留原位置" @click="importAsset('folder')"><FolderOpen :size="14" />导入文件夹</button>
      <button class="xb-menu-item" role="menuitem" @click="importAsset('design')"><PanelsTopLeft :size="14" />导入设计源文件</button>
      <div class="xb-menu-sep" />
      <button class="xb-menu-item" role="menuitem" @click="importMenu=false;emit('browseDesigns')"><Files :size="14" />关联已有设计稿</button>
      <button class="xb-menu-item" role="menuitem" @click="importAsset('new-design')"><Plus :size="14" />新建设计稿</button>
    </div>
    <label class="assets-search"><Search :size="13" aria-hidden="true" /><input v-model="query" type="search" aria-label="搜索对话资产" placeholder="搜索资产" /></label>
    <div class="assets-tabs" role="tablist" aria-label="资产类型"><button v-for="t in tabs" :id="`asset-tab-${t.id}`" :key="t.id" :data-tab="t.id" role="tab" type="button" aria-controls="chat-assets-list" :aria-selected="tab===t.id" @click="tab=t.id" @keydown="tabKey">{{t.label}}<small>{{t.count}}</small></button></div>
    <div v-if="error" class="assets-error" role="alert">{{error}}<button type="button" class="xb-link" @click="folder?openFolder(folder):refresh()">重试</button></div>
    <div id="chat-assets-list" class="assets-list" role="tabpanel" :aria-labelledby="`asset-tab-${tab}`" :aria-busy="loading||folderBusy" @scroll="menu=''">
      <div v-if="folder" class="folder-crumb"><button type="button" class="xb-icon-btn is-sm" aria-label="返回上一级" @click="back"><ArrowLeft :size="14" /></button><span :title="folder.local?.path">{{folder.name}}</span><button type="button" class="xb-icon-btn is-sm" title="在文件管理器中打开" aria-label="在文件管理器中打开" @click="action(folder,'reveal')"><ExternalLink :size="14" /></button></div>
      <p v-if="folderBusy" class="assets-empty" role="status">正在读取文件夹…</p>
      <template v-else>
        <section v-for="type in ['design','image','file'] as const" v-show="shown.some(a=>a.type===type)" :key="type" class="asset-group">
          <div v-if="!folder" class="group-head"><span>{{{design:'设计稿',image:'图片',file:'交付文件'}[type]}}</span><span>{{shown.filter(a=>a.type===type).length}}</span></div>
          <div :class="{'asset-images':type==='image','is-large':tab==='image'}">
            <div v-for="asset in shown.filter(a=>a.type===type)" :key="asset.key" class="asset-item" :class="{'is-image':type==='image'}" data-testid="conversation-asset">
              <button type="button" class="asset-open" :title="title(asset)" :aria-description="title(asset)" :aria-label="`打开${asset.name}`" :disabled="busy===asset.key" @click="action(asset)">
                <AssetThumbnail v-if="type==='image'&&session" :asset="asset" :session="session" />
                <span v-else class="asset-file-icon" :class="{design:type==='design'}"><component :is="icon(asset)" :size="15" /></span>
                <span class="asset-name">{{asset.name}}</span><small v-if="type!=='image'" class="asset-meta" :class="{'is-missing':asset.local&&!asset.local.exists}">{{assetMeta(asset)}}</small>
              </button>
              <button type="button" class="xb-icon-btn is-sm asset-more" :aria-label="`${asset.name}的更多操作`" :aria-expanded="menu===asset.key" aria-haspopup="menu" @click="toggleMenu(asset,$event)"><Ellipsis :size="14" /></button>
            </div>
          </div>
        </section>
        <div v-if="!shown.length" class="assets-empty"><Files :size="24" aria-hidden="true" /><span>{{loading?'正在读取资产…':query?'没有找到相关资产':folder?'文件夹为空':'这段对话还没有资产'}}</span><button v-if="query" class="xb-link" @click="query=''">清除搜索</button><button v-else-if="!folder" class="xb-link" :disabled="importing" @click="importMenu=true"><Upload :size="13" />导入资产</button></div>
        <p v-if="truncated&&folder" class="assets-warning">仅展示前 500 项，请在文件管理器中查看全部内容。</p>
      </template>
    </div>
    <div v-if="activeMenu" class="xb-menu asset-menu" :style="{top:menuTop+'px'}" role="menu" :aria-label="`${activeMenu.name}的操作`">
      <button class="xb-menu-item" role="menuitem" @click="action(activeMenu)"><ExternalLink :size="14" />{{activeMenu.local?.kind==='folder'?'打开文件夹':'打开'}}</button>
      <button v-if="!activeMenu.design&&activeMenu.local?.kind!=='folder'" class="xb-menu-item" role="menuitem" :disabled="activeMenu.local&&!activeMenu.local.exists" @click="action(activeMenu,'save')"><Download :size="14" />另存为</button>
      <button class="xb-menu-item" role="menuitem" :disabled="!activeMenu.blockId" :title="activeMenu.blockId?'定位生成它的消息':'这份资产没有可定位的交付消息'" @click="action(activeMenu,'locate')"><MessageSquare :size="14" />定位消息</button>
      <template v-if="activeMenu.local"><button class="xb-menu-item" role="menuitem" @click="action(activeMenu,'copy')"><Copy :size="14" />复制路径</button><button class="xb-menu-item" role="menuitem" :disabled="!activeMenu.local.exists" @click="action(activeMenu,'reveal')"><FolderOpen :size="14" />在文件夹中显示</button></template>
    </div>
    <footer><span>当前对话</span><span>{{folder?'文件夹内容':'设计稿 · 图片 · 文件'}}</span></footer>
  </aside>
</template>

<style scoped>
.chat-assets{position:absolute;right:12px;top:51px;z-index:26;width:min(300px,calc(100% - 24px));max-height:calc(100% - 63px);display:flex;flex-direction:column;border:1px solid var(--ui-line);border-radius:12px;background:var(--ui-surface);color:var(--ui-text-primary);box-shadow:var(--ui-shadow-menu);font-size:12px;min-height:0}
.chat-assets.is-pinned{position:relative;top:auto;right:auto;width:304px;max-width:45%;max-height:none;flex:none;border:0;border-left:1px solid var(--ui-line);border-radius:0;box-shadow:none}
.assets-head{display:flex;align-items:center;gap:6px;padding:12px 10px 8px;flex:none}.assets-head b{font-size:13px;font-weight:500}.spacer{flex:1}.asset-count{padding:0 5px;border-radius:4px;background:var(--ui-control);font-size:11px;color:var(--ui-text-tertiary)}
.assets-search{display:flex;align-items:center;gap:7px;background:var(--ui-control);border-radius:7px;padding:7px 9px;margin:0 12px 10px;color:var(--ui-text-tertiary)}input{border:0;background:transparent;color:var(--ui-text-primary);font:inherit;width:100%;min-width:0;outline-offset:4px}input::placeholder{color:var(--ui-text-tertiary)}
.assets-tabs{display:flex;gap:2px;padding:0 10px;border-bottom:1px solid var(--ui-line);flex:none}.assets-tabs button{display:flex;align-items:center;gap:5px;border:0;border-bottom:2px solid transparent;background:transparent;color:var(--ui-text-secondary);font:inherit;padding:7px 7px 9px;cursor:pointer;white-space:nowrap}.assets-tabs button[aria-selected=true]{border-bottom-color:var(--ui-btn);color:var(--ui-ink)}.assets-tabs small{font-size:11px;color:var(--ui-text-tertiary)}
.assets-list{padding:10px;overflow:auto;min-height:0;overscroll-behavior:contain;flex:1}.asset-group+.asset-group{margin-top:14px}.group-head{display:flex;justify-content:space-between;color:var(--ui-text-tertiary);font-size:11px;padding:2px 4px 6px}.asset-item{display:flex;position:relative;align-items:center;gap:3px;min-width:0}.asset-open{display:flex;align-items:center;gap:8px;padding:6px 4px;min-width:0;flex:1;border:0;border-radius:7px;background:transparent;color:inherit;cursor:pointer;font:inherit;text-align:left}.asset-open:hover,.asset-open:focus-visible{background:var(--ui-surface-hover)}.asset-file-icon{display:grid;place-items:center;flex:none;width:27px;height:27px;background:var(--ui-control);color:var(--ui-text-secondary);border-radius:6px}.asset-file-icon.design{background:var(--ui-soft);color:var(--ui-ink)}.asset-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0}.asset-meta{margin-left:auto;white-space:nowrap;font-size:11px;color:var(--ui-text-tertiary)}.is-missing{color:var(--ui-danger)}.asset-more{flex:none;color:var(--ui-text-tertiary)}
.asset-images{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px}.asset-images.is-large{grid-template-columns:repeat(2,minmax(0,1fr))}.asset-item.is-image{display:block}.is-image .asset-open{display:flex;flex-direction:column;align-items:stretch;gap:6px;width:100%;padding:0;background:transparent}.is-image .asset-name{font-size:11px;padding:0 2px;max-width:100%}.is-image .asset-more{position:absolute;right:2px;top:2px;background:var(--ui-surface);border-radius:6px;width:24px;height:24px;opacity:0}.is-image:hover .asset-more,.is-image:focus-within .asset-more,.is-image .asset-more[aria-expanded=true]{opacity:1}
.asset-menu,.import-menu{position:absolute;width:190px;z-index:30}.import-menu{right:10px;top:43px}.asset-menu{right:10px}.asset-menu .xb-menu-item,.import-menu .xb-menu-item{font-size:12px}.asset-menu button:disabled{opacity:.4;cursor:default}.assets-empty{min-height:170px;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:12px;color:var(--ui-text-secondary);text-align:center;padding:28px 8px}.assets-empty .xb-link{display:flex;align-items:center;gap:5px}.assets-error{padding:8px 12px;color:var(--ui-danger);font-size:12px;overflow-wrap:anywhere}.assets-error button{margin-left:8px}.assets-warning{font-size:11px;color:var(--ui-text-secondary);padding:8px 3px}.folder-crumb{display:flex;align-items:center;gap:4px;margin-bottom:10px}.folder-crumb span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1}footer{display:flex;justify-content:space-between;gap:8px;flex:none;border-top:1px solid var(--ui-line);color:var(--ui-text-tertiary);font-size:11px;padding:10px 13px}
button:focus-visible{outline:2px solid var(--ui-btn);outline-offset:1px}button:disabled{opacity:.5;cursor:default}
@media(max-width:700px){.chat-assets.is-pinned{position:absolute;right:12px;top:51px;width:min(300px,calc(100% - 24px));max-width:none;max-height:calc(100% - 63px);border:1px solid var(--ui-line);border-radius:12px;box-shadow:var(--ui-shadow-menu)}}
@media(pointer:coarse){.is-image .asset-more{opacity:1}.assets-tabs button,.asset-open,.assets-head button{min-height:40px}.asset-open{font-size:13px}}
</style>
