<script setup lang="ts">
import {nextTick,onMounted,onBeforeUnmount,ref,watch} from 'vue';
const props=defineProps<{src:string;title:string;editing:boolean;commenting?:boolean;theme?:'light'|'dark';pins?:{num:number;node?:string;text:string}[];selection?:string[];editorCommand?:{seq:number;command:string};writable?:boolean;zoom?:number;guides?:boolean;token:string}>();
const frame=ref<HTMLIFrameElement|null>(null),host=ref<HTMLElement|null>(null),visible=ref(false);let observer:IntersectionObserver|undefined;
const emit=defineEmits<{message:[data:Record<string,any>];error:[message:string]}>();
let ready=false,disposed=false,waitingReady:(()=>void)|undefined;
let captureRequest:{id:string;resolve:(png:string)=>void;reject:(error:Error)=>void;timer:ReturnType<typeof setTimeout>}|undefined;
function cancelCapture(){waitingReady=undefined;if(captureRequest){clearTimeout(captureRequest.timer);captureRequest.reject(Error('导出已取消'));captureRequest=undefined;}}
async function capture(options:{width:number;height:number;scale:1|2}){
 if(captureRequest)throw Error('画板正在导出');
 if(disposed)throw Error('导出已取消');
 return new Promise<string>((resolve,reject)=>{
  const id=crypto.randomUUID();
  captureRequest={id,resolve,reject,timer:setTimeout(()=>{waitingReady=undefined;captureRequest=undefined;reject(Error('画板渲染超时，请重试'));},45000)};
  const request=()=>frame.value?.contentWindow?.postMessage({channel:'open-claude-design',type:'capture',token:props.token,requestId:id,...options},'*');
  visible.value=true;
  void nextTick().then(()=>{if(captureRequest?.id!==id)return;if(ready)request();else waitingReady=request;});
 });
}
defineExpose({capture,cancelCapture});
function sync(){
  const win=frame.value?.contentWindow;if(!win)return;
  win.postMessage({channel:'open-claude-design',type:'mode',editing:props.editing,commenting:Boolean(props.commenting),writable:props.writable!==false,zoom:props.zoom??1,guides:props.guides!==false},'*');
  if(props.theme)win.postMessage({channel:'open-claude-design',type:'theme',theme:props.theme},'*');
  if(props.pins)win.postMessage({channel:'open-claude-design',type:'pins',pins:props.pins},'*');
}
function select(){frame.value?.contentWindow?.postMessage({channel:'open-claude-design',type:'editor-selection',ids:props.selection??[]},'*');}
function message(e:MessageEvent){if(e.source!==frame.value?.contentWindow||e.data?.channel!=='open-claude-design'||e.data.token!==props.token)return;if(e.data.type==='ready'||e.data.type==='editor-ready'){sync();select();}if(e.data.type==='ready'){ready=true;waitingReady?.();waitingReady=undefined;}if(e.data.type==='capture-result'){const pending=captureRequest;if(!pending||pending.id!==e.data.requestId)return;captureRequest=undefined;clearTimeout(pending.timer);if(e.data.error)pending.reject(Error(String(e.data.error)));else if(typeof e.data.png==='string'&&e.data.png.length<=64*1024*1024)pending.resolve(e.data.png);else pending.reject(Error('画板图片无效'));return;}if(e.data.type==='error')emit('error',String(e.data.message));emit('message',e.data);}
onMounted(()=>{window.addEventListener('message',message);if(typeof IntersectionObserver==='undefined')visible.value=true;else{observer=new IntersectionObserver(entries=>{if(entries.some(e=>e.isIntersecting)){visible.value=true;observer?.disconnect();}},{rootMargin:'300px'});if(host.value)observer.observe(host.value);}});
onBeforeUnmount(()=>{disposed=true;cancelCapture();observer?.disconnect();window.removeEventListener('message',message);});
watch(()=>props.src,()=>{ready=false;cancelCapture();});
watch(()=>[props.editing,props.commenting,props.theme,props.pins,props.writable,props.zoom,props.guides],sync,{deep:true});
watch(()=>props.selection?.join('\0'),select);
watch(()=>props.editorCommand?.seq,()=>{if(props.editorCommand)frame.value?.contentWindow?.postMessage({channel:'open-claude-design',type:'editor-command',command:props.editorCommand.command},'*');});
</script>
<template><div ref="host" class="board-preview"><iframe v-if="visible" ref="frame" :src="src" :title="title" sandbox="allow-scripts" referrerpolicy="no-referrer" @load="sync" /><span v-else class="pending">预览待加载</span></div></template>
<style scoped>.board-preview,.board-preview iframe{width:100%;height:100%;border:0;background:white}.pending{display:block;padding:24px;color:#999;font:12px system-ui}</style>
