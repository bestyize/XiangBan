<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {invoke} from '../bridge.ts';
import PetActor from './PetActor.vue';
import type {PetInstance,PetDefinition} from './model.ts';
interface Payload{instance:PetInstance;definition:PetDefinition;paused:boolean;fps:30|60;links:boolean;effect:string;displayScale?:number}
declare global{interface Window{__XIAOBAN_PET__?:Payload}}
const payload=ref<Payload>(),actor=ref<InstanceType<typeof PetActor>>(),ratio=ref(1),error=ref('');
let hit:boolean|undefined,drag:{x:number;y:number;grabX:number;grabY:number;moved:boolean}|undefined,suppress=false,moving=false;
let pending:{grabX:number;grabY:number;commit:boolean}|undefined;
let lastPointer:{x:number;y:number}|undefined;
let hoverTimer:ReturnType<typeof setTimeout>|undefined,hoverStart=0,hoverToken=0;
let dialogueTimer:ReturnType<typeof setTimeout>|undefined;
function cancelHover(){if(hoverTimer)clearTimeout(hoverTimer);hoverTimer=undefined;hoverToken++;}
function scheduleHover(){
 cancelHover();if(!payload.value||payload.value.paused||!payload.value.instance.links||!payload.value.links)return;
 hoverStart=Date.now();const token=hoverToken;
 hoverTimer=setTimeout(()=>{
  hoverTimer=undefined;
  if(token!==hoverToken||!hit||drag||!payload.value||payload.value.paused)return;
  void invoke('pet_proactive_hover',{durationMs:Math.min(10000,Date.now()-hoverStart)}).catch(()=>{});
 },900);
}
const size=computed(()=>payload.value!.instance.size*ratio.value);
function resize(){ratio.value=payload.value?.displayScale?payload.value.displayScale/(window.devicePixelRatio||1):1;}
function config(event:Event){payload.value=(event as CustomEvent<Payload>).detail;resize();}
function pointer(event:Event){
 const point=(event as CustomEvent<{x:number;y:number}>).detail;if(!actor.value)return;
 if(!lastPointer||point.x!==lastPointer.x||point.y!==lastPointer.y){actor.value.look(point.x,point.y);lastPointer=point;}
 if(drag)return;
 const next=actor.value.hit(point.x,point.y);
 if(next!==hit){hit=next;if(next)scheduleHover();else cancelHover();void invoke('set_pet_hit_test',{hit:next}).catch(()=>{hit=undefined;cancelHover();});}
}
async function movePending(){if(moving)return;moving=true;try{while(pending){const p=pending;pending=undefined;await invoke('move_pet_instance',p);}}catch(e){error.value=String(e);}finally{moving=false;}}
function down(e:PointerEvent){if(e.button!==0||!actor.value?.hit(e.clientX,e.clientY))return;clearTimeout(dialogueTimer);cancelHover();drag={x:e.screenX,y:e.screenY,grabX:e.clientX/innerWidth,grabY:e.clientY/innerHeight,moved:false};suppress=false;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);}
function move(e:PointerEvent,commit=false){if(!drag)return;if(Math.hypot(e.screenX-drag.x,e.screenY-drag.y)>5)drag.moved=true;if(drag.moved){suppress=true;pending={grabX:drag.grabX,grabY:drag.grabY,commit};void movePending();}if(commit){drag=undefined;}}
function click(e:MouseEvent){
 if(suppress){e.preventDefault();e.stopImmediatePropagation();suppress=false;return;}
 if(!actor.value?.hit(e.clientX,e.clientY))return;
 cancelHover();actor.value.poke();clearTimeout(dialogueTimer);
 // A quick burst gets an immediate animation but only one conversation/model request.
 dialogueTimer=setTimeout(()=>{void invoke('companion_dialogue_click').catch(e=>error.value=String(e));},150);
 void invoke('pet_proactive_poke').catch(()=>{});
}
function menu(){clearTimeout(dialogueTimer);cancelHover();void invoke('manage_pet_instance').catch(e=>error.value=String(e));}
watch(()=>payload.value?.effect,value=>{if(value)actor.value?.effect(value);},{flush:'post'});
onMounted(async()=>{
 document.documentElement.style.background='transparent';document.body.className='pet-instance-scene';
 window.addEventListener('pet-config',config);window.addEventListener('pet-pointer',pointer);window.addEventListener('resize',resize);
 window.addEventListener('pet-presence',presence);window.addEventListener('pet-greeting-poke',greetingPoke);
 try{payload.value=await invoke<Payload>('get_pet_instance');resize();await invoke('scene_ready');}catch(e){error.value=String(e);}
});
function presence(e:Event){actor.value?.presence((e as CustomEvent<{kind:string}>).detail.kind);}
function greetingPoke(){actor.value?.poke();}
onBeforeUnmount(()=>{clearTimeout(dialogueTimer);cancelHover();window.removeEventListener('pet-greeting-poke',greetingPoke);window.removeEventListener('pet-config',config);window.removeEventListener('pet-pointer',pointer);window.removeEventListener('resize',resize);window.removeEventListener('pet-presence',presence);});
</script>
<template><div v-if="payload" class="pet-window" :style="{padding:`${8*ratio}px`}" @pointerdown="down" @pointermove="move($event)" @pointerup="move($event,true)" @pointercancel="move($event,true)" @click.capture="click" @contextmenu.prevent.stop="menu"><PetActor ref="actor" :definition="payload.definition" :size="size" :expression="payload.instance.expression" :paused="payload.paused" :fps="payload.fps" :links="payload.links&&payload.instance.links" :interactive="false"/><p v-if="error" class="pet-window-error">{{ error }}</p></div></template>
<style>
html:has(body.pet-instance-scene),body.pet-instance-scene,body.pet-instance-scene #app{margin:0;width:100%;height:100%;background:transparent!important;overflow:hidden}.pet-window{position:relative;width:max-content;box-sizing:content-box;cursor:grab;touch-action:none}.pet-window:active{cursor:grabbing}.pet-window-error{position:absolute;bottom:12px;left:12px;right:12px;border-radius:8px;padding:8px;color:#fff;background:#453a33e8;font-size:11px}
</style>

