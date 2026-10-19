<script setup lang="ts">
import {onMounted,onBeforeUnmount,ref,watch} from 'vue';
import {PetView} from '../pet.ts';
import {defaults} from '../bridge.ts';
import type {PetDefinition} from './model.ts';
import type {ExpressionName} from '../companion/expressions.ts';
import {presenceBeats,isPresenceKind} from '../companion/presence.ts';
const props=withDefaults(defineProps<{definition:PetDefinition;size?:number;expression?:ExpressionName;paused?:boolean;fps?:30|60;links?:boolean;interactive?:boolean}>(),{size:200,expression:'idle',paused:false,fps:30,links:true,interactive:true});
const host=ref<HTMLElement>();let pet:PetView|undefined;
const settings=()=>({...defaults,size:props.size,expression:props.expression,paused:props.paused,fps:props.fps});
function configure(){if(!pet)return;pet.setDefinition(props.definition);pet.update(settings());pet.reactions.setEnabled(props.links);pet.reactions.setPaused(props.paused);}
onMounted(()=>{if(host.value){pet=new PetView(host.value,settings(),undefined,props.definition);configure();}});
watch(()=>[props.definition,props.size,props.expression,props.paused,props.fps,props.links],configure);
onBeforeUnmount(()=>pet?.destroy());
defineExpose({poke:()=>pet?.poke(),hit:(x:number,y:number)=>pet?.hit(x,y)??false,look:(x:number,y:number)=>pet?.look(x,y),presence:(kind:unknown)=>{if(props.links&&isPresenceKind(kind))pet?.playBeats(presenceBeats(kind));},effect:(selected:string)=>pet?.reactions.setEffect(selected)});
</script>
<template><div ref="host" class="pet-actor" :style="{width:`${size}px`,height:`${size}px`}" @pointermove="interactive&&pet?.look($event.clientX,$event.clientY)" @click="interactive&&pet?.poke()"/></template>
<style scoped>
.pet-actor{position:relative;flex:none;user-select:none;touch-action:none}.pet-actor :deep(.pet-stage),.pet-actor :deep(.pet-art){width:100%;height:100%;position:relative;display:block}.pet-actor :deep(svg){width:100%;height:100%;display:block;overflow:visible}.pet-actor :deep(.pet-art:focus-visible){outline:2px solid var(--ui-primary,#bd8c70);outline-offset:4px;border-radius:30px}
</style>
