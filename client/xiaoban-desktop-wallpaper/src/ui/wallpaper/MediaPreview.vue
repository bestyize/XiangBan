<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import { bindMedia, mediaMarkup } from "../../wallpaper-media.ts";
const props=defineProps<{kind:string;source:string;poster:string;title:string;controls?:boolean;fit?:string;rate?:number}>();
const host=ref<HTMLElement>();let generation=0;let disposed=false;let cleanup:(()=>void)|undefined;let card:HTMLElement|null=null;
const live=ref(false);
async function render() {const token=++generation;cleanup?.();await nextTick();if(disposed || token!==generation || !host.value)return;host.value.innerHTML=mediaMarkup(props.kind,props.source,props.poster,props.title,props.controls||live.value);cleanup=bindMedia(host.value);update();}
function update(){ const element=host.value?.querySelector<HTMLVideoElement>("video");if(element)element.playbackRate=props.rate??1; }
function start(){if(props.kind==='web'&&!props.controls&&props.source&&!live.value){live.value=true;void render();}}
function stop(){if(live.value){live.value=false;void render();}}
watch(()=>[props.kind,props.source,props.poster,props.controls],()=>{live.value=false;void render();},{immediate:true});watch(()=>props.rate,update);
onMounted(()=>{card=host.value?.closest<HTMLElement>('button.wallpaper-card')??null;card?.addEventListener('mouseenter',start);card?.addEventListener('mouseleave',stop);card?.addEventListener('focus',start);card?.addEventListener('blur',stop);});
onBeforeUnmount(()=>{disposed=true;generation++;cleanup?.();card?.removeEventListener('mouseenter',start);card?.removeEventListener('mouseleave',stop);card?.removeEventListener('focus',start);card?.removeEventListener('blur',stop);});
</script>
<template><div ref="host" class="media-preview" :class="{'media-preview-card':!controls}" :style="{'--media-fit':fit || 'cover'}" /></template>
<style scoped>
.media-preview{position:relative;overflow:hidden;width:100%;height:100%;background:var(--ui-primary-soft);display:grid;place-items:center}.media-preview :deep(img),.media-preview :deep(video){width:100%;height:100%;object-fit:var(--media-fit)}.media-preview :deep(iframe){width:100%;height:100%;border:0;display:block}.media-preview :deep(.media-status){position:absolute;bottom:8px;left:8px;right:8px;color:var(--ui-text-secondary);font-size:12px;background:color-mix(in srgb, var(--ui-surface) 86%, transparent);padding:4px;border-radius:5px}.media-preview :deep(.media-status[hidden]){display:none}.media-preview :deep(.fallback-art){font-size:34px;color:var(--ui-text-secondary)}
.media-preview-card :deep(iframe){pointer-events:none}
</style>

