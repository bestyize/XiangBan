<script setup lang="ts">
import {computed,onDeactivated,onMounted,onBeforeUnmount,ref} from 'vue';
import { FileCode } from 'lucide-vue-next';
import {baseSize,type DesktopWidget} from '../../widgets/model.ts';
import WidgetSurface from '../../widgets/WidgetSurface.vue';
const props=defineProps<{widget:DesktopWidget;demo?:boolean}>();
const host=ref<HTMLElement>(),width=ref(0),height=ref(0);
const hovered = ref(false), focused = ref(false);
const lazy = computed(() => props.demo && props.widget.custom?.standaloneHtml);
const size=computed(()=>baseSize(props.widget.kind,props.widget.custom));
const style=computed(()=>({width:`${size.value.width}px`,height:`${size.value.height}px`,transform:`translate(-50%,-50%) scale(${Math.max(.01,Math.min((width.value-28)/size.value.width,(height.value-28)/size.value.height))})`}));
let observer:ResizeObserver|undefined;
onMounted(()=>{observer=new ResizeObserver(entries=>{const rect=entries[0]?.contentRect;if(rect){width.value=rect.width;height.value=rect.height;}});if(host.value)observer.observe(host.value);});
onBeforeUnmount(()=>observer?.disconnect());
onDeactivated(() => { hovered.value = false; focused.value = false; });
</script>
<template>
  <div ref="host" class="widget-preview" @mouseenter="hovered = true" @mouseleave="hovered = false" @focusin="focused = true" @focusout="focused = false">
    <div v-if="!lazy || hovered || focused" class="preview-inner" :style="style" inert aria-hidden="true"><WidgetSurface :widget="widget" preview :demo="demo"/></div>
    <FileCode v-else class="preview-placeholder" :size="24" aria-hidden="true"/>
    <slot/>
  </div>
</template>
<style scoped>
.widget-preview{position:relative;aspect-ratio:1.2;overflow:hidden;border-radius:13px;background:var(--ui-configuration-canvas)}
.preview-inner{position:absolute;left:50%;top:50%;transform-origin:center;pointer-events:none}
.preview-placeholder{position:absolute;inset:0;margin:auto;color:var(--ui-text-secondary);pointer-events:none}
</style>
