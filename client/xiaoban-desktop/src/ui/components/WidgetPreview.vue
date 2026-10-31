<script setup lang="ts">
import {computed,onMounted,onBeforeUnmount,ref} from 'vue';
import {baseSize,type DesktopWidget} from '../../widgets/model.ts';
import WidgetSurface from '../../widgets/WidgetSurface.vue';
const props=defineProps<{widget:DesktopWidget;demo?:boolean}>();
const host=ref<HTMLElement>(),width=ref(0),height=ref(0);
const size=computed(()=>baseSize(props.widget.kind,props.widget.custom));
const style=computed(()=>({width:`${size.value.width}px`,height:`${size.value.height}px`,transform:`translate(-50%,-50%) scale(${Math.max(.01,Math.min((width.value-28)/size.value.width,(height.value-28)/size.value.height))})`}));
let observer:ResizeObserver|undefined;
onMounted(()=>{observer=new ResizeObserver(entries=>{const rect=entries[0]?.contentRect;if(rect){width.value=rect.width;height.value=rect.height;}});if(host.value)observer.observe(host.value);});
onBeforeUnmount(()=>observer?.disconnect());
</script>
<template><div ref="host" class="widget-preview"><div class="preview-inner" :style="style" inert aria-hidden="true"><WidgetSurface :widget="widget" preview :demo="demo"/></div><slot/></div></template>
<style scoped>
.widget-preview{position:relative;aspect-ratio:1.2;overflow:hidden;border-radius:13px;background:var(--ui-configuration-canvas)}
.preview-inner{position:absolute;left:50%;top:50%;transform-origin:center;pointer-events:none}
</style>
