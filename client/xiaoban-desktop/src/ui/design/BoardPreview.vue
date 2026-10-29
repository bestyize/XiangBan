<script setup lang="ts">
import {onMounted,onBeforeUnmount,ref,watch} from 'vue';
const props=defineProps<{src:string;title:string;editing:boolean;commenting?:boolean;theme?:'light'|'dark';pins?:{num:number;node?:string;text:string}[];selection?:string[];editorCommand?:{seq:number;command:string};writable?:boolean;zoom?:number;guides?:boolean;token:string}>();
const frame=ref<HTMLIFrameElement|null>(null),host=ref<HTMLElement|null>(null),visible=ref(false);let observer:IntersectionObserver|undefined;
const emit=defineEmits<{message:[data:Record<string,any>];error:[message:string]}>();
function sync(){
  const win=frame.value?.contentWindow;if(!win)return;
  win.postMessage({channel:'open-claude-design',type:'mode',editing:props.editing,commenting:Boolean(props.commenting),writable:props.writable!==false,zoom:props.zoom??1,guides:props.guides!==false},'*');
  if(props.theme)win.postMessage({channel:'open-claude-design',type:'theme',theme:props.theme},'*');
  if(props.pins)win.postMessage({channel:'open-claude-design',type:'pins',pins:props.pins},'*');
}
function select(){frame.value?.contentWindow?.postMessage({channel:'open-claude-design',type:'editor-selection',ids:props.selection??[]},'*');}
function message(e:MessageEvent){if(e.source!==frame.value?.contentWindow||e.data?.channel!=='open-claude-design'||e.data.token!==props.token)return;if(e.data.type==='ready'||e.data.type==='editor-ready'){sync();select();}if(e.data.type==='error')emit('error',String(e.data.message));emit('message',e.data);}
onMounted(()=>{window.addEventListener('message',message);if(typeof IntersectionObserver==='undefined')visible.value=true;else{observer=new IntersectionObserver(entries=>{if(entries.some(e=>e.isIntersecting)){visible.value=true;observer?.disconnect();}},{rootMargin:'300px'});if(host.value)observer.observe(host.value);}});
onBeforeUnmount(()=>{observer?.disconnect();window.removeEventListener('message',message);});
watch(()=>[props.editing,props.commenting,props.theme,props.pins,props.writable,props.zoom,props.guides],sync,{deep:true});
watch(()=>props.selection?.join('\0'),select);
watch(()=>props.editorCommand?.seq,()=>{if(props.editorCommand)frame.value?.contentWindow?.postMessage({channel:'open-claude-design',type:'editor-command',command:props.editorCommand.command},'*');});
</script>
<template><div ref="host" class="board-preview"><iframe v-if="visible" ref="frame" :src="src" :title="title" sandbox="allow-scripts" referrerpolicy="no-referrer" @load="sync" /><span v-else class="pending">预览待加载</span></div></template>
<style scoped>.board-preview,.board-preview iframe{width:100%;height:100%;border:0;background:white}.pending{display:block;padding:24px;color:#999;font:12px system-ui}</style>
