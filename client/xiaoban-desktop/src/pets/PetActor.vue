<script setup lang="ts">
import {computed,onActivated,onDeactivated,onMounted,onBeforeUnmount,ref,shallowRef,watch} from 'vue';
import {PetView} from '../pet.ts';
import {defaults} from '../bridge.ts';
import type {PetDefinition} from './model.ts';
import type {ExpressionName} from '../companion/expressions.ts';
import {presenceBeats,isPresenceKind} from '../companion/presence.ts';
import {isMingyue,type PetMotion} from './mingyue/actions.ts';
import {correctedMingyueSprites} from './mingyue/fallback.ts';
import type {MingyueRenderer} from './mingyue/renderer.js';
const props=withDefaults(defineProps<{definition:PetDefinition;themeId?:string;motion?:PetMotion;size?:number;expression?:ExpressionName;paused?:boolean;fps?:30|60;links?:boolean;interactive?:boolean;gallery?:boolean}>(),{size:200,expression:'idle',paused:false,fps:30,links:true,interactive:true});
const host=ref<HTMLElement>();let pet:PetView|undefined;
const usesModel=computed(()=>isMingyue(props.themeId,props.definition.id));
const sprites=shallowRef<PetDefinition>();
watch(()=>[usesModel.value,props.definition] as const,([mingyue,definition],_,onCleanup)=>{
  sprites.value=undefined;
  if(!mingyue)return;
  let current=true;
  onCleanup(()=>{current=false;});
  void correctedMingyueSprites(definition).then(updated=>{
    if(current&&updated!==definition)sprites.value=updated;
  }).catch(error=>console.warn('溟月备用姿态更新失败',error));
},{immediate:true});
const canvas=ref<HTMLCanvasElement>(), modelReady=ref(false);
let model:MingyueRenderer|undefined;
let active=true;
const settings=()=>({...defaults,size:props.size,expression:props.expression,paused:props.paused||!active,fps:props.fps});
function configure(){if(!pet)return;pet.setDefinition(sprites.value??props.definition);pet.update(settings());pet.reactions.setEnabled(props.links);pet.reactions.setPaused(props.paused||!active);if(props.gallery&&!props.definition.poses)host.value?.querySelector('svg')?.setAttribute('viewBox','0 -72 320 352');}
onMounted(()=>{if(host.value){pet=new PetView(host.value,settings(),undefined,props.definition);configure();}});
onActivated(()=>{active=true;configure();});
onDeactivated(()=>{active=false;configure();});
watch(()=>[props.definition,sprites.value,props.size,props.expression,props.paused,props.fps,props.links,props.motion],configure,{flush:'post'});

watch(canvas,(element,_,onCleanup)=>{
  if(!element)return;
  const loading=new AbortController();
  const resize=new ResizeObserver(()=>pet?.redrawArtwork());
  resize.observe(element);
  function useFallback(){modelReady.value=false;pet?.setArtwork();}
  function attach(){
    if(!pet||!model||loading.signal.aborted||element?.getContext('webgl2')?.isContextLost())return;
    const rig=model;
    pet.setArtwork({render:(dt,mood)=>rig.render(dt,mood,props.motion),hit:rig.hit,poke:rig.poke});
    modelReady.value=true;
  }
  function lost(event:Event){event.preventDefault();useFallback();}
  function restored(){queueMicrotask(attach);}
  element.addEventListener('webglcontextlost',lost);
  element.addEventListener('webglcontextrestored',restored);
  const start=async()=>{
    const {loadMingyue}=await import('./mingyue/renderer.js');
    if(loading.signal.aborted)return;
    const rig=await loadMingyue(element,loading.signal);
    if(loading.signal.aborted){rig.destroy();return;}
    model=rig;attach();
  };
  const timer=window.setTimeout(()=>{
    void start().catch(error=>{
      if(!loading.signal.aborted){useFallback();console.warn('溟月已使用平面形象',error);}
    });
  },props.gallery?250:0);
  onCleanup(()=>{
    clearTimeout(timer);
    loading.abort();
    resize.disconnect();
    element.removeEventListener('webglcontextlost',lost);
    element.removeEventListener('webglcontextrestored',restored);
    useFallback();
    model?.destroy();model=undefined;
  });
},{flush:'post'});

function setDragging(value:boolean){
  if(!model||(value&&props.paused))return;
  if(value)pet?.interact();
  model.setDragging(value);pet?.redrawArtwork();
}
onBeforeUnmount(()=>pet?.destroy());
defineExpose({setDragging,poke:()=>pet?.poke(),hit:(x:number,y:number)=>pet?.hit(x,y)??false,look:(x:number,y:number)=>pet?.look(x,y),presence:(kind:unknown)=>{if(props.links&&isPresenceKind(kind))pet?.playBeats(presenceBeats(kind));},effect:(selected:string)=>pet?.reactions.setEffect(selected)});
</script>
<template><div ref="host" class="pet-actor" :class="{'is-gallery':gallery,'has-model':modelReady}" :style="{width:`${size}px`,height:`${size}px`}" @pointermove="interactive&&pet?.look($event.clientX,$event.clientY)" @click="interactive&&pet?.poke()"><canvas v-if="usesModel" ref="canvas" class="pet-model" aria-hidden="true"/></div></template>
<style scoped>
.pet-actor{position:relative;flex:none;user-select:none;touch-action:none}.pet-actor :deep(.pet-stage),.pet-actor :deep(.pet-art){width:100%;height:100%;position:relative;display:block}.pet-actor :deep(svg){width:100%;height:100%;display:block;overflow:visible}.pet-actor :deep(.pet-art:focus-visible){outline:2px solid var(--ui-primary,#bd8c70);outline-offset:4px;border-radius:30px}
/* PetView sets an inline pixel height for desktop windows. Gallery previews must fit their responsive host. */
.pet-actor.is-gallery :deep(.pet-stage){width:100%!important;height:100%!important}
.pet-model{position:absolute;inset:0;z-index:1;width:100%;height:100%;pointer-events:none;visibility:hidden}.has-model .pet-model{visibility:visible}.has-model :deep(.pet-art svg){visibility:hidden}
</style>
