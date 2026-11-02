<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref,watch} from 'vue';
import {ArrowUpRight,Code2,Download,ExternalLink,Eye,LoaderCircle,PanelRight,RefreshCw,X} from 'lucide-vue-next';
import {invoke} from '../../bridge.ts';
import type {PreviewInfo,PreviewTarget} from './model.ts';
const props=defineProps<{target:PreviewTarget;detached?:boolean;notify:(message:unknown,type?:'success'|'error'|'info')=>void}>();
const emit=defineEmits<{close:[];detach:[];dock:[]}>();
const info=ref<PreviewInfo|null>(null),mode=ref<'preview'|'source'>('preview'),loading=ref(false),error=ref(''),auto=ref(true),frame=ref<HTMLIFrameElement|null>(null);
const canPreview=computed(()=>info.value?.kind!=='text');
const runtimeError=ref('');
let generation=0,disposed=false,timer:ReturnType<typeof setTimeout>|undefined,polling=false;
let owner:PreviewTarget|undefined;
async function release(value:PreviewInfo|null,target:PreviewTarget|undefined){if(value&&target)await invoke('chat_preview_release',{target,token:value.token}).catch(()=>{});}
async function load(){
 const epoch=++generation,target={...props.target};loading.value=true;error.value='';runtimeError.value='';
 try{
  const next=await invoke<PreviewInfo>('chat_preview_read',{target});
  if(disposed||epoch!==generation){await release(next,target);return;}
  const before=info.value,beforeOwner=owner;info.value=next;owner=target;await release(before,beforeOwner);
 }catch(e){if(!disposed&&epoch===generation){error.value=String(e);const old=info.value;info.value=null;await release(old,owner);owner=undefined;}}
 finally{if(!disposed&&epoch===generation)loading.value=false;}
}
async function poll(){
 if(disposed)return;
 if(auto.value&&info.value&&!loading.value&&!polling&&document.visibilityState!=='hidden'){
  polling=true;const value=info.value,target={...props.target},epoch=generation;
  try{const version=await invoke<string>('chat_preview_revision',{target,token:value.token});if(!disposed&&epoch===generation&&version!==value.revision)await load();}
  catch(e){if(!disposed&&epoch===generation){error.value=String(e);const old=info.value;info.value=null;await release(old,owner);}}
  finally{polling=false;}
 }
 if(!disposed)timer=setTimeout(poll,2000);
}
async function action(action:'open'|'save'){
 try{const saved=await invoke<boolean>('chat_preview_action',{target:props.target,action});if(action==='save'&&saved)props.notify('文件已保存','success');}catch(e){props.notify(e,'error');}
}
watch(()=>JSON.stringify(props.target),()=>{const old=info.value,oldOwner=owner;info.value=null;owner=undefined;mode.value='preview';void release(old,oldOwner);void load();},{immediate:true});
function message(event:MessageEvent){if(event.source===frame.value?.contentWindow&&event.data?.channel==='xiaoban-preview'&&event.data.token===info.value?.token&&typeof event.data.message==='string')runtimeError.value=event.data.message.slice(0,500);}
onMounted(()=>{timer=setTimeout(poll,2000);window.addEventListener('message',message);});
onBeforeUnmount(()=>{disposed=true;generation++;clearTimeout(timer);window.removeEventListener('message',message);void release(info.value,owner);});
</script>
<template>
 <section class="preview-panel" data-testid="preview-panel" aria-label="作品预览">
  <header><b :title="info?.name">{{info?.name||'作品预览'}}</b><span class="spacer"/>
   <button v-if="canPreview" class="xb-icon-btn" :class="{'is-active':mode==='preview'}" title="预览" aria-label="预览" @click="mode='preview'"><Eye :size="15"/></button>
   <button v-if="info?.source!=null" class="xb-icon-btn" :class="{'is-active':mode==='source'}" title="源码" aria-label="源码" @click="mode='source'"><Code2 :size="15"/></button>
   <button class="xb-icon-btn" title="刷新预览" aria-label="刷新预览" :disabled="loading" @click="load"><LoaderCircle v-if="loading" :size="15" class="spin"/><RefreshCw v-else :size="15"/></button>
   <button class="xb-icon-btn" title="另存为" aria-label="另存为" :disabled="!info" @click="action('save')"><Download :size="15"/></button>
   <button class="xb-icon-btn" title="在外部应用打开" aria-label="在外部应用打开" @click="action('open')"><ExternalLink :size="15"/></button>
   <button class="xb-icon-btn" :title="detached?'回到聊天右侧':'独立窗口'" :aria-label="detached?'回到聊天右侧':'独立窗口'" @click="detached?emit('dock'):emit('detach')"><PanelRight v-if="detached" :size="15"/><ArrowUpRight v-else :size="15"/></button>
   <button v-if="!detached" class="xb-icon-btn" title="关闭当前作品" aria-label="关闭当前作品" @click="emit('close')"><X :size="15"/></button>
  </header>
  <div class="preview-options"><label><input v-model="auto" type="checkbox">自动刷新</label><span v-if="info?.kind==='html'" title="运行当前文件及同目录静态资源；外部网络资源不会加载" aria-description="运行当前文件及同目录静态资源；外部网络资源不会加载">本地网页</span></div>
  <p v-if="error" role="alert" class="preview-error">{{error}}<button class="xb-text-btn" @click="load">重试</button></p>
  <p v-else-if="!info" role="status" class="preview-loading">正在准备预览…</p>
  <p v-if="runtimeError" role="alert" class="preview-error">{{runtimeError}}</p>
  <pre v-if="info&&(mode==='source'||!canPreview)" class="preview-source" tabindex="0" aria-label="作品源码">{{info.source}}</pre>
  <iframe v-if="info&&canPreview" v-show="mode==='preview'" ref="frame" :key="info.token" :src="info.url" :title="info.name" sandbox="allow-scripts" referrerpolicy="no-referrer" />
 </section>
</template>
<style scoped>
.preview-panel{display:flex;flex-direction:column;min-width:0;min-height:0;width:100%;height:100%;color:var(--ui-text-primary);background:var(--ui-surface)}header{display:flex;align-items:center;gap:2px;min-height:44px;padding:0 10px;border-bottom:1px solid var(--ui-line);flex-shrink:0}header b{font-size:12px;font-weight:500;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:35%}.spacer{flex:1}.preview-options{display:flex;align-items:center;justify-content:space-between;min-height:30px;padding:0 12px;font-size:11px;color:var(--ui-text-secondary);border-bottom:1px solid var(--ui-line)}label{display:flex;align-items:center;gap:5px}input{accent-color:var(--ui-btn)}iframe{width:100%;flex:1;min-height:0;border:0;background:var(--ui-surface)}.preview-source{flex:1;min-height:0;overflow:auto;margin:0;padding:18px;font:12px/1.7 var(--ui-font-mono,monospace);white-space:pre;tab-size:2;color:var(--ui-text-primary);background:var(--ui-surface)}.preview-error,.preview-loading{margin:16px;font-size:12px;line-height:1.7;overflow-wrap:anywhere}.preview-error{color:var(--ui-danger)}.spin{animation:spin 1s linear infinite}@keyframes spin{to{transform:rotate(360deg)}}
</style>
