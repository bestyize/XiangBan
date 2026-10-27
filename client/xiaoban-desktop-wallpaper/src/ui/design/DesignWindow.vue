<script setup lang="ts">
import {ref} from 'vue';
import {invoke} from '../../bridge.ts';
import DesignWorkbench from './DesignWorkbench.vue';
const id=new URLSearchParams(location.search).get('id')??'';
const toast=ref('');let timer:ReturnType<typeof setTimeout>|undefined;
function notify(message:unknown){toast.value=String(message);clearTimeout(timer);timer=setTimeout(()=>toast.value='',4000);}
async function close(){try{await invoke('design_window_close',{id});}catch(e){notify(e);}}
async function dock(){try{await invoke('design_window_return',{id});}catch(e){notify(e);}}
async function request(message:string,session:string){try{await invoke('design_window_request',{id,message,session});notify('修改要求已交给小伴');}catch(e){notify(e);}}
</script>
<template><main class="design-window"><DesignWorkbench :project-id="id" detached :notify="notify" @close="close" @dock="dock" @request="request"/><p v-if="toast" class="window-toast" role="status">{{toast}}</p></main></template>
<style scoped>.design-window{width:100vw;height:100vh;display:flex;overflow:hidden;color:var(--ui-text-primary);background:var(--ui-surface);font-family:var(--ui-font-family)}.window-toast{position:fixed;bottom:30px;left:50%;transform:translateX(-50%);background:var(--ui-surface);border:1px solid var(--ui-line);box-shadow:var(--ui-shadow-menu);border-radius:10px;font-size:12px;padding:10px 16px;z-index:99;max-width:80vw}</style>
