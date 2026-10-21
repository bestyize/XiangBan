<script setup lang="ts">
import {onBeforeUnmount,onMounted,ref} from 'vue';
import {invoke} from '../bridge.ts';
interface Greeting {id:number;petName:string;text:string;appearance:'system'|'light'|'dark'}
const greeting=ref<Greeting|null>(null),error=ref('');
let timeout:ReturnType<typeof setTimeout>|undefined;
async function finish(outcome:'chat'|'poke'|'dismissed'|'ignored'|'snoozed'){
 const active=greeting.value;if(!active)return;greeting.value=null;
 if(timeout)clearTimeout(timeout);
 try { await invoke('respond_companion_greeting',{id:active.id,outcome}); }
 catch(e) { error.value=String(e); }
}
onMounted(async()=>{
 document.documentElement.style.background='transparent';document.body.className='pet-greeting-scene';
 try{
  greeting.value=await invoke<Greeting>('get_companion_greeting');
  timeout=setTimeout(()=>void finish('ignored'),8000);
 }catch(e){error.value=String(e);}
});
onBeforeUnmount(()=>{if(timeout)clearTimeout(timeout);});
</script>
<template>
 <div class="greeting-shell" :data-appearance="greeting?.appearance||'system'">
  <section v-if="greeting" class="greeting-card" role="status" aria-live="polite">
   <header><span class="greeting-name">{{greeting.petName}}</span><button class="greeting-close" title="关闭问候" aria-label="关闭问候" @click="finish('dismissed')">×</button></header>
   <p>{{greeting.text}}</p>
   <footer><button class="greeting-action" @click="finish('chat')">聊两句</button><button class="greeting-action" @click="finish('poke')">摸摸头</button><button class="greeting-later" @click="finish('snoozed')">暂不打扰</button></footer>
  </section>
  <span v-else-if="error" class="greeting-error">问候已结束</span>
 </div>
</template>
<style>
html:has(body.pet-greeting-scene),body.pet-greeting-scene,body.pet-greeting-scene #app{width:100%;height:100%;margin:0;background:transparent!important;overflow:hidden}
.greeting-shell{box-sizing:border-box;width:100%;height:100%;padding:9px 11px 14px;display:flex;align-items:center;justify-content:center;font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Microsoft YaHei",sans-serif}
.greeting-card{box-sizing:border-box;width:100%;padding:13px 14px 11px;border-radius:17px;background:#fff;box-shadow:0 7px 24px rgba(30,24,20,.18),0 1px 3px rgba(30,24,20,.08);color:#282725}
.greeting-card header{display:flex;align-items:center;justify-content:space-between;gap:10px}.greeting-name{font-size:11px;font-weight:650;color:#847d78;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}
.greeting-close{border:0;background:transparent;color:#8d8783;font-size:18px;line-height:14px;cursor:pointer;padding:3px;flex:none}
.greeting-card p{font-size:12px;line-height:1.5;margin:7px 0 10px;min-height:19px;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.greeting-card footer{display:flex;gap:8px;align-items:center}.greeting-card footer button{border:0;background:#f5f4f2;border-radius:8px;color:#4c4744;font:inherit;font-size:10px;cursor:pointer;padding:6px 8px;white-space:nowrap}
.greeting-card footer button:hover{background:#ece9e6}.greeting-card footer .greeting-later{background:transparent;color:#938d89;margin-left:auto;padding-right:0}
.greeting-error{font-size:11px;color:#8d8783}
.greeting-shell[data-appearance="dark"] .greeting-card{background:#292827;color:#f3f2f0;box-shadow:0 8px 24px #0007}.greeting-shell[data-appearance="dark"] .greeting-card footer button{background:#42403d;color:#efedeb}.greeting-shell[data-appearance="dark"] .greeting-card footer .greeting-later{background:transparent;color:#aaa39b}
@media(prefers-color-scheme:dark){.greeting-shell[data-appearance="system"] .greeting-card{background:#292827;color:#f3f2f0;box-shadow:0 8px 24px #0007}.greeting-shell[data-appearance="system"] .greeting-card footer button{background:#42403d;color:#efedeb}.greeting-shell[data-appearance="system"] .greeting-card footer .greeting-later{background:transparent;color:#aaa39b}}
</style>
