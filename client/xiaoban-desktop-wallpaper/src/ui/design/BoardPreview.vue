<script setup lang="ts">
import {onMounted,onBeforeUnmount,ref,watch} from 'vue';
const props=defineProps<{src:string;title:string;editing:boolean;commenting?:boolean;theme?:'light'|'dark';pins?:{num:number;node?:string;text:string}[];token:string}>();
const frame=ref<HTMLIFrameElement|null>(null),host=ref<HTMLElement|null>(null),visible=ref(false);let observer:IntersectionObserver|undefined;
const emit=defineEmits<{message:[data:Record<string,any>];error:[message:string]}>();
function sync(){
  const win=frame.value?.contentWindow;if(!win)return;
  win.postMessage({channel:'open-claude-design',type:'mode',editing:props.editing,commenting:Boolean(props.commenting)},'*');
  if(props.theme)win.postMessage({channel:'open-claude-design',type:'theme',theme:props.theme},'*');
  if(props.pins)win.postMessage({channel:'open-claude-design',type:'pins',pins:props.pins},'*');
}
function message(e:MessageEvent){if(e.source!==frame.value?.contentWindow||e.data?.channel!=='open-claude-design'||e.data.token!==props.token)return;if(e.data.type==='ready')sync();if(e.data.type==='error')emit('error',String(e.data.message));emit('message',e.data);}
onMounted(()=>{window.addEventListener('message',message);if(typeof IntersectionObserver==='undefined')visible.value=true;else{observer=new IntersectionObserver(entries=>{if(entries.some(e=>e.isIntersecting)){visible.value=true;observer?.disconnect();}},{rootMargin:'300px'});if(host.value)observer.observe(host.value);}});
onBeforeUnmount(()=>{observer?.disconnect();window.removeEventListener('message',message);});
watch(()=>[props.editing,props.commenting,props.theme,props.pins],sync,{deep:true});
</script>
<template><div ref="host" class="board-preview"><iframe v-if="visible" ref="frame" :src="src" :title="title" sandbox="allow-scripts" referrerpolicy="no-referrer" @load="sync" /><span v-else class="pending">预览待加载</span></div></template>
<style scoped>.board-preview,.board-preview iframe{width:100%;height:100%;border:0;background:white}.pending{display:block;padding:24px;color:#999;font:12px system-ui}</style>
