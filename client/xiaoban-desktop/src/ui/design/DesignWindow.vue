<script setup lang="ts">
import {onBeforeUnmount,onMounted,ref,watchEffect} from 'vue';
import {invoke,listen} from '../../bridge.ts';
import {accent,followAccent,followHost,isDark} from '../appearance.ts';
import {applyUiTokens} from '../design-tokens.ts';
import DesignWorkbench from './DesignWorkbench.vue';
watchEffect(()=>{applyUiTokens(isDark.value,accent.value);document.documentElement.dataset.theme=isDark.value?'dark':'light';});
type Appearance={appearance?:string;accent?:string};
let disposed=false,offSettings:(()=>void)|undefined;
function follow(value:Appearance){followHost(value.appearance);followAccent(value.accent);}
onMounted(async()=>{
 let changes=0;
 try{
  const off=await listen<Appearance>('settings-changed',value=>{changes++;follow(value);});
  if(disposed){off();return;}offSettings=off;
  const version=changes,value=await invoke<Appearance>('get_appearance');
  if(!disposed&&version===changes)follow(value);
 }catch(e){if(!disposed)notify(e);}
});
onBeforeUnmount(()=>{disposed=true;offSettings?.();clearTimeout(timer);});
const id=new URLSearchParams(location.search).get('id')??'';
const toast=ref('');let timer:ReturnType<typeof setTimeout>|undefined;
function notify(message:unknown){toast.value=String(message);clearTimeout(timer);timer=setTimeout(()=>toast.value='',4000);}
async function close(){try{await invoke('design_window_close',{id});}catch(e){notify(e);}}
async function dock(){try{await invoke('design_window_return',{id});}catch(e){notify(e);}}
async function request(message:string,session:string){try{await invoke('design_window_request',{id,message,session});notify('修改要求已交给小伴');}catch(e){notify(e);}}
</script>
<template><main class="design-window"><DesignWorkbench :project-id="id" detached :notify="notify" @close="close" @dock="dock" @request="request"/><p v-if="toast" class="window-toast" role="status">{{toast}}</p></main></template>
<style scoped>.design-window{width:100vw;height:100vh;display:flex;overflow:hidden;color:var(--ui-text-primary);background:var(--ui-surface);font-family:var(--ui-font-family)}.window-toast{position:fixed;bottom:30px;left:50%;transform:translateX(-50%);background:var(--ui-surface);border:1px solid var(--ui-line);box-shadow:var(--ui-shadow-menu);border-radius:10px;font-size:12px;padding:10px 16px;z-index:99;max-width:80vw}</style>
