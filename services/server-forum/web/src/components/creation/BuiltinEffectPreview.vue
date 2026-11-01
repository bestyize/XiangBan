<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { builtinEffectPack } from '@/lib/builtinEffects'
import { ParticleSystem } from '@/lib/builtinEffectsEngine/sim'
import { drawEffects } from '@/lib/builtinEffectsEngine/draw'
import { defaultParams } from '@/lib/builtinEffectsEngine/types'
import { shouldAnimateEffect } from '@/lib/builtinEffectsPlayback'
const props = withDefaults(defineProps<{ id: string; playing?: boolean }>(), { playing:false })
const host = ref<HTMLElement | null>(null), canvas = ref<HTMLCanvasElement | null>(null), pack = computed(() => builtinEffectPack(props.id))
const sky = computed(() => pack.value ? `linear-gradient(145deg, ${pack.value.sky.join(', ')})` : '')
let system: ParticleSystem | null = null, resize: ResizeObserver | undefined, intersection: IntersectionObserver | undefined, media: MediaQueryList | undefined
let frame = 0, visible = false, last = 0, mounted = false
function stop() { if (frame) cancelAnimationFrame(frame); frame = 0; last = 0 }
function render() { const ctx = canvas.value?.getContext('2d'); if (ctx && system) drawEffects(ctx, system) }
function size() {
  if (!host.value || !canvas.value || !system) return
  const rect=host.value.getBoundingClientRect(), w=Math.max(1,rect.width), h=Math.max(1,rect.height), dpr=Math.min(2,window.devicePixelRatio || 1)
  canvas.value.width=Math.round(w*dpr); canvas.value.height=Math.round(h*dpr)
  canvas.value.getContext('2d')?.setTransform(dpr,0,0,dpr,0,0)
  system.configure(w,h,defaultParams)
  // Advance bursts as well, so a static card does not happen to capture an empty sky.
  for (let i=0;i<60;i++) system.advance(.1)
  if (system.layers.some(layer => layer.spec.burst)) {
    for (let i=0;i<600 && !system.layers.some(layer => layer.spec.burst && layer.order.some(particle => particle.visible && particle.alpha > .05));i++) system.advance(.1)
  }
  render()
}
function tick(now:number) { frame=0; if (!mounted || !shouldAnimateEffect(props.playing,visible,document.hidden,!!media?.matches) || !system) { last=0; return }; system.advance(last ? (now-last)/1000 : 0); last=now; render(); frame=requestAnimationFrame(tick) }
function reconcile() { stop(); if (mounted && shouldAnimateEffect(props.playing,visible,document.hidden,!!media?.matches) && system) frame=requestAnimationFrame(tick) }
function setup() {
  stop(); system = pack.value ? new ParticleSystem(pack.value) : null
  if (system) size(); else canvas.value?.getContext('2d')?.clearRect(0,0,canvas.value.width,canvas.value.height)
  reconcile()
}
onMounted(() => {
  mounted=true; media=window.matchMedia('(prefers-reduced-motion: reduce)'); media.addEventListener('change',reconcile)
  if (host.value && typeof ResizeObserver !== 'undefined') { resize=new ResizeObserver(size); resize.observe(host.value) }
  if (host.value && typeof IntersectionObserver !== 'undefined') { intersection=new IntersectionObserver(entries => { visible=entries.some(e => e.isIntersecting); reconcile() }); intersection.observe(host.value) } else visible=true
  document.addEventListener('visibilitychange',reconcile); window.addEventListener('resize',size); setup()
})
watch(() => props.id, () => { if (mounted) setup() })
watch(() => props.playing, reconcile)
onBeforeUnmount(() => { mounted=false; stop(); resize?.disconnect(); intersection?.disconnect(); media?.removeEventListener('change',reconcile); document.removeEventListener('visibilitychange',reconcile); window.removeEventListener('resize',size); system=null })
</script>
<template><div ref="host" class="effect-preview" :style="{background:sky}" :aria-label="pack ? `${pack.name}特效预览` : '特效不可用'" role="img"><canvas ref="canvas" aria-hidden="true"></canvas><span v-if="!pack">特效不可用</span></div></template>
<style scoped>.effect-preview { position:relative; width:100%; height:100%; min-height:100px; overflow:hidden; isolation:isolate; }.effect-preview canvas { position:absolute; inset:0; display:block; width:100%; height:100%; pointer-events:none; }.effect-preview span { color:var(--ink-3); }</style>
