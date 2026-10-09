<script setup lang="ts">
import { ref, watch, onBeforeUnmount, nextTick } from "vue";
import { bindMedia, mediaMarkup } from "../../wallpaper-media.ts";
import { WallpaperEffects, type WallpaperEffectsOptions } from "../../wallpaper-effects.ts";
const props=defineProps<{kind:string;source:string;poster:string;title:string;controls?:boolean;fit?:string;rate?:number;effects?:WallpaperEffectsOptions;paused?:boolean;fps?:30|60}>();
const host=ref<HTMLElement>();let generation=0;let disposed=false;let cleanup:(()=>void)|undefined;let renderer:WallpaperEffects|undefined;
async function render() {const token=++generation;cleanup?.();renderer?.destroy();await nextTick();if(disposed || token!==generation || !host.value)return;host.value.innerHTML=mediaMarkup(props.kind,props.source,props.poster,props.title,props.controls);cleanup=bindMedia(host.value);renderer=new WallpaperEffects(host.value,props.kind,props.effects,{paused:props.paused??false,fps:props.fps??60});update();}
function update(){ renderer?.update(props.kind,props.effects,{paused:props.paused??false,fps:props.fps??60}); const element=host.value?.querySelector<HTMLVideoElement>("video");if(element)element.playbackRate=props.rate??1; }
watch(()=>[props.kind,props.source,props.poster,props.controls],render,{immediate:true});watch(()=>[props.effects,props.rate,props.paused,props.fps],update,{deep:true});
onBeforeUnmount(()=>{disposed=true;generation++;cleanup?.();renderer?.destroy();});
</script>
<template><div ref="host" class="media-preview" :style="{'--media-fit':fit || 'cover'}" /></template>
<style scoped>
.media-preview{position:relative;overflow:hidden;width:100%;height:100%;background:#e9eef5;display:grid;place-items:center}.media-preview :deep(img),.media-preview :deep(video){width:100%;height:100%;object-fit:var(--media-fit)}.media-preview :deep(.media-status){position:absolute;bottom:8px;left:8px;right:8px;color:#637186;font-size:11px;background:#ffffffdb;padding:4px;border-radius:5px}.media-preview :deep(.media-status[hidden]){display:none}.media-preview :deep(.fallback-art){font-size:34px;color:#7b96b9}
</style>
