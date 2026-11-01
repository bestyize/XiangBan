<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import WidgetSurface from './widget-preview/WidgetSurface.vue'
import { baseSize, type DesktopWidget, type WidgetKind } from './widget-preview/model'
import { emptyNote, emptyCountdown } from './widget-preview/content'
import { dateKey } from './widget-preview/calendar'
import pelican from './widget-preview/pelican-bike.json'
import { BUILTIN_WIDGETS } from '@/lib/builtinWidgets'
const props = defineProps<{ id: string }>()
const host = ref<HTMLElement>()
const dimensions = ref({ width: 0, height: 0 })
const valid = computed(() => BUILTIN_WIDGETS.some(item => item.id === props.id))
const isPelican = computed(() => props.id === 'pelican-bike')
const widget = computed<DesktopWidget>(() => {
  const kind = (valid.value && !isPelican.value ? props.id : 'clock') as WidgetKind
  return { id:`demo${kind}`, kind, theme:'ivory', scale:1, x:0, y:0, enabled:true, city:null,
    ...(kind === 'notes' ? { note:{ ...emptyNote(), items:[{id:'task1',text:'完成产品周报',done:true,time:'09:00'}, {id:'task2',text:'准备下周的设计评审',done:false,time:'14:00'}, {id:'task3',text:'阅读一本书',done:false,time:'20:00'}] } } : {}),
    ...(kind === 'reminders' ? { living:{ symbol:'1.000001',count:0,reminders:[{id:'demo1',title:'产品设计方案评审',at:dateKey(new Date())+'T10:00',end:'11:30',location:'会议室 A302',done:false}, {id:'demo2',title:'与开发同学同步进度',at:dateKey(new Date())+'T14:00',end:'15:00',done:false}] } } : {}),
    ...(kind === 'woodfish' ? { living:{ count:108,symbol:'1.000001',reminders:[] } } : {}),
    ...(kind === 'countdown' ? { countdown:{...emptyCountdown(),title:'我们的纪念日',date:dateKey(new Date(Date.now()+79*86400000))} } : {}) }
})
const size = computed(() => isPelican.value ? { width:300,height:260 } : baseSize(widget.value.kind))
const scale = computed(() => Math.max(.01, Math.min((dimensions.value.width-24)/size.value.width, (dimensions.value.height-24)/size.value.height)))
const pelicanDocument = computed(() => pelican.html.replace('<head>', `<head><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:; connect-src 'none'; base-uri 'none'; form-action 'none'">`))
let observer: ResizeObserver | undefined
onMounted(() => { observer = new ResizeObserver(entries => { const r=entries[0]?.contentRect; if(r) dimensions.value={width:r.width,height:r.height} }); if(host.value) observer.observe(host.value) })
onBeforeUnmount(() => observer?.disconnect())
</script>
<template>
  <div ref="host" class="builtin-widget-preview" :aria-label="valid ? `${BUILTIN_WIDGETS.find(item => item.id === id)?.name}，演示预览` : '组件预览不可用'">
    <div v-if="valid" class="preview-canvas" :style="{width:`${size.width}px`,height:`${size.height}px`,transform:`translate(-50%,-50%) scale(${scale})`}" aria-hidden="true" inert>
      <iframe v-if="isPelican" :srcdoc="pelicanDocument" title="鹈鹕骑车演示" sandbox="allow-scripts" tabindex="-1" />
      <WidgetSurface v-else :widget="widget" preview demo />
    </div>
    <span v-if="valid" class="demo-label">演示</span><span v-else class="unavailable">预览不可用</span>
  </div>
</template>
<style scoped>
.builtin-widget-preview{position:relative;width:100%;height:100%;min-height:0;overflow:hidden;background:#eeece8;border-radius:12px;isolation:isolate}.preview-canvas{position:absolute;left:50%;top:50%;transform-origin:center;pointer-events:none}.preview-canvas iframe{width:100%;height:100%;border:0;display:block}.demo-label{position:absolute;right:8px;bottom:8px;z-index:2;background:#fffdf5e8;color:#666;padding:2px 8px;border-radius:12px;font-size:12px;line-height:1.5}.unavailable{position:absolute;inset:0;display:grid;place-items:center;color:#666;font-size:14px}
</style>
