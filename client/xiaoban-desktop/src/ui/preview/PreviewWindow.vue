<script setup lang="ts">
import {onMounted,onBeforeUnmount,ref,watchEffect} from 'vue';
import {invoke,listen} from '../../bridge.ts';
import {accent,followAccent,followHost,isDark} from '../appearance.ts';
import {applyUiTokens} from '../design-tokens.ts';
import WindowChrome from '../configuration/WindowChrome.vue';
import WindowResizeEdges from '../configuration/WindowResizeEdges.vue';
import PreviewPanel from './PreviewPanel.vue';
import type {PreviewTarget} from './model.ts';
import '../configuration/surfaces.css';
document.documentElement.classList.add('configuration-frameless');
let target:PreviewTarget|undefined;try{target=JSON.parse(new URLSearchParams(location.search).get('provider')||'null')??undefined;}catch{}
const toast=ref('');let timer:ReturnType<typeof setTimeout>|undefined,off:(()=>void)|undefined,disposed=false;
function notify(value:unknown){toast.value=String(value);clearTimeout(timer);timer=setTimeout(()=>toast.value='',4500);}
watchEffect(()=>{applyUiTokens(isDark.value,accent.value);document.documentElement.dataset.theme=isDark.value?'dark':'light';});
onMounted(async()=>{
 let changes=0;const follow=(value:{appearance?:string;accent?:string})=>{followHost(value.appearance);followAccent(value.accent);};
 const stop=await listen<{appearance?:string;accent?:string}>('settings-changed',value=>{changes++;follow(value);});if(disposed){stop();return;}off=stop;
 const version=changes;try{const value=await invoke<{appearance?:string;accent?:string}>('get_appearance');if(!disposed&&version===changes)follow(value);}catch(e){notify(e);}
});
onBeforeUnmount(()=>{disposed=true;off?.();clearTimeout(timer);});
async function dock(){try{await invoke('chat_preview_return',{target});}catch(e){notify(e);}}
</script>
<template><main class="preview-window"><WindowChrome title="作品预览"/><PreviewPanel v-if="target" :target="target" detached :notify="notify" @dock="dock"/><p v-else role="alert">预览参数无效</p><p v-if="toast" class="preview-toast" role="status">{{toast}}</p><WindowResizeEdges/></main></template>
<style scoped>.preview-window{height:100vh;width:100vw;display:flex;flex-direction:column;overflow:hidden;background:var(--ui-surface);color:var(--ui-text-primary);border-radius:12px}.preview-window :deep(.preview-panel){flex:1;height:auto}.preview-toast{position:absolute;bottom:24px;left:16px;right:16px;margin:0;padding:12px;border-radius:8px;background:var(--ui-surface-raised,var(--ui-surface));box-shadow:var(--ui-shadow-menu);font-size:12px}</style>
