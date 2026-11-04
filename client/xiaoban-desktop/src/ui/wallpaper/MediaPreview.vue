<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import { bindMedia, mediaMarkup } from "../../wallpaper-media.ts";
import { invoke } from "../../bridge.ts";
const props=defineProps<{kind:string;source:string;poster:string;title:string;controls?:boolean;fit?:string;rate?:number;coverId?:string}>();
const host=ref<HTMLElement>();let generation=0;let disposed=false;let cleanup:(()=>void)|undefined;let card:HTMLElement|null=null,hoverHost:HTMLElement|null=null;
const capturedPoster=ref('');
let coverObserver: IntersectionObserver | undefined;
const live=ref(false);
async function render() {const token=++generation;cleanup?.();await nextTick();if(disposed || token!==generation || !host.value)return;host.value.innerHTML=mediaMarkup(props.kind,props.source,props.poster||capturedPoster.value,props.title,props.controls||live.value);cleanup=bindMedia(host.value);update();}
function update(){ const element=host.value?.querySelector<HTMLVideoElement>("video");if(element)element.playbackRate=props.rate??1; }
function start(){if(props.kind==='web'&&!props.controls&&props.source&&!live.value){live.value=true;void render();}}
function stop(){if(live.value){live.value=false;void render();}}
watch(()=>[props.kind,props.source,props.poster,props.controls],()=>{live.value=false;void render();},{immediate:true});watch(()=>props.rate,update);
watch(capturedPoster, render);
onMounted(()=>{
  if (!host.value || !props.coverId || props.poster) return;
  coverObserver = new IntersectionObserver(entries => {
    const id = props.coverId;
    if (!id || !entries.some(entry => entry.isIntersecting)) return;
    coverObserver?.disconnect();
    if (props.poster) return;
    void invoke<string>('wallpaper_ensure_cover', { id }).then(url => {
      if (!disposed && props.coverId === id) capturedPoster.value = url;
    }).catch(() => { /* A missing cover still has a live hover preview. */ });
  });
  coverObserver.observe(host.value);
});
onMounted(()=>{card=host.value?.closest<HTMLElement>('.wallpaper-card')??null;hoverHost=host.value?.closest<HTMLElement>('.wc')??card;hoverHost?.addEventListener('mouseenter',start);hoverHost?.addEventListener('mouseleave',stop);card?.addEventListener('focus',start);card?.addEventListener('blur',stop);});
onBeforeUnmount(()=>{disposed=true;generation++;coverObserver?.disconnect();cleanup?.();hoverHost?.removeEventListener('mouseenter',start);hoverHost?.removeEventListener('mouseleave',stop);card?.removeEventListener('focus',start);card?.removeEventListener('blur',stop);});
</script>
<template><div ref="host" class="media-preview" :class="{'media-preview-card':!controls}" :style="{'--media-fit':fit || 'cover'}" /></template>
<style scoped>
.media-preview{position:relative;overflow:hidden;width:100%;height:100%;background:var(--ui-soft);display:grid;place-items:center}.media-preview :deep(img),.media-preview :deep(video){width:100%;height:100%;object-fit:var(--media-fit)}.media-preview :deep(iframe){width:100%;height:100%;border:0;display:block}.media-preview :deep(.media-status){position:absolute;bottom:8px;left:8px;right:8px;color:var(--ui-text-secondary);font-size:12px;background:color-mix(in srgb, var(--ui-surface) 86%, transparent);padding:4px;border-radius:5px}.media-preview :deep(.media-status[hidden]){display:none}.media-preview :deep(.fallback-art){font-size:34px;color:var(--ui-text-secondary)}
.media-preview-card :deep(iframe){pointer-events:none}
</style>

