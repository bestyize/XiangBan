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
    <div v-else class="preview-placeholder" aria-hidden="true">
      <span class="placeholder-icon"><FileCode :size="20"/></span>
      <b>{{ widget.custom?.name || 'HTML 组件' }}</b>
      <small>悬停实时预览</small>
    </div>
    <slot/>
  </div>
</template>
<style scoped>
.widget-preview{position:relative;aspect-ratio:1.2;overflow:hidden;border-radius:14px;background:var(--ui-configuration-canvas)}
.preview-inner{position:absolute;left:50%;top:50%;transform-origin:center;pointer-events:none}
.preview-placeholder{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:6px;padding:12px;text-align:center;color:var(--ui-text-secondary);pointer-events:none}
.placeholder-icon{width:38px;height:38px;border-radius:11px;background:var(--ui-soft);color:var(--ui-ink);display:grid;place-items:center}
.preview-placeholder b{font-size:12.5px;font-weight:600;color:var(--ui-text-primary);max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.preview-placeholder small{font-size:11px;color:var(--ui-text-tertiary)}
</style>
