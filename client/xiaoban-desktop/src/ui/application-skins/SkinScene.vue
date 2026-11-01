<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

import type { SkinLayer, SkinSceneData } from './types.ts';
const props = withDefaults(defineProps<{ scene: SkinSceneData; paused?: boolean; interactive?: boolean }>(), { paused: false, interactive: true });
const root = ref<HTMLElement | null>(null);
const inactive = ref(false), reduced = ref(false), covered = ref<Set<string>>(new Set());
const active = ref<Record<string, boolean>>({});
const timers = new Map<string, ReturnType<typeof setTimeout>>();
let observer: ResizeObserver | undefined, mutations: MutationObserver | undefined, frame: number | undefined;
let media: MediaQueryList | undefined;
const running = computed(() => !props.paused && !inactive.value && !reduced.value);
// Only validated inline raster images can enter a scene. This also prevents requests to external hosts.
const isRaster = (url: string) => /^data:image\/(?:png|jpeg|webp);base64,[a-z0-9+/=\r\n]+$/i.test(url);
const layers = computed(() => (props.scene?.layers ?? []).filter(l => typeof l.image === 'string' && isRaster(l.image)).slice(0, 24).sort((a, b) => a.depth - b.depth));
function interaction(layer: SkinLayer) {
  return layer.interactive ? props.scene.interactions?.find(i => i.target === layer.id && i.trigger === 'click' && ['sway', 'glow'].includes(i.action) && i.label?.trim()) : undefined;
}
function idleAction(layer: SkinLayer) { return props.scene.interactions?.find(i => i.target === layer.id && i.trigger === 'idle' && ['sway', 'glow'].includes(i.action))?.action; }
function hoverAction(layer: SkinLayer) { return props.scene.interactions?.find(i => i.target === layer.id && i.trigger === 'hover' && ['sway', 'glow'].includes(i.action))?.action; }
function anchorClass(layer: SkinLayer) { return ['center', 'top-left', 'top-right', 'bottom-left', 'bottom-right'].includes(layer.anchor) ? `anchor-${layer.anchor}` : 'anchor-center'; }
function toggle(id: string) {
  if (!running.value || covered.value.has(id)) return;
  const timer = timers.get(id);
  if (timer && props.scene.interactions?.find(i => i.target === id && i.trigger === 'click')?.repeat === 'ignore') return;
  if (timer) clearTimeout(timer);
  active.value = { ...active.value, [id]: false };
  void nextTick(() => { active.value = { ...active.value, [id]: true }; timers.set(id, setTimeout(() => { active.value = { ...active.value, [id]: false }; timers.delete(id); }, 1200)); });
}
function updatePause() {
  const el = document.activeElement;
  inactive.value = document.hidden || !document.hasFocus() || !!el?.matches('input, textarea, [contenteditable="true"]');
}
function updateMotion() { reduced.value = !!media?.matches; }
// A scene hot spot is never allowed to cover real application controls, even if window size changes.
function checkCoverage() {
  frame = undefined;
  if (!root.value) return;
  const controls = [...document.querySelectorAll<HTMLElement>('button, input, textarea, select, a[href], [contenteditable="true"], [role="button"], [role="switch"], [role="tab"]')].filter(el => !root.value?.contains(el) && el.getClientRects().length > 0);
  const next = new Set<string>();
  for (const button of root.value.querySelectorAll<HTMLElement>('[data-skin-hotspot]')) {
    const id = button.dataset.skinHotspot!; const r = button.getBoundingClientRect();
    if (r.width < 24 || r.height < 24 || controls.some(el => { const c = el.getBoundingClientRect(); return r.left < c.right && r.right > c.left && r.top < c.bottom && r.bottom > c.top; })) next.add(id);
  }
  covered.value = next;
}
function scheduleCoverage() { if (frame === undefined) frame = requestAnimationFrame(checkCoverage); }
onMounted(() => {
  media = window.matchMedia?.('(prefers-reduced-motion: reduce)'); updateMotion(); media?.addEventListener?.('change', updateMotion);
  updatePause(); document.addEventListener('visibilitychange', updatePause); document.addEventListener('focusin', updatePause); document.addEventListener('focusout', updatePause); window.addEventListener('focus', updatePause); window.addEventListener('blur', updatePause); window.addEventListener('resize', scheduleCoverage); document.addEventListener('scroll', scheduleCoverage, true);
  if (typeof ResizeObserver !== 'undefined') { observer = new ResizeObserver(scheduleCoverage); observer.observe(document.documentElement); if (root.value) observer.observe(root.value); }
  if (typeof MutationObserver !== 'undefined') { mutations = new MutationObserver(scheduleCoverage); mutations.observe(document.body, { subtree: true, childList: true }); }
  scheduleCoverage();
});
watch(() => props.scene, () => { active.value = {}; void nextTick(scheduleCoverage); });
onBeforeUnmount(() => {
  media?.removeEventListener?.('change', updateMotion); document.removeEventListener('visibilitychange', updatePause); document.removeEventListener('focusin', updatePause); document.removeEventListener('focusout', updatePause); window.removeEventListener('focus', updatePause); window.removeEventListener('blur', updatePause); window.removeEventListener('resize', scheduleCoverage); document.removeEventListener('scroll', scheduleCoverage, true);
  observer?.disconnect(); mutations?.disconnect(); if (frame !== undefined) cancelAnimationFrame(frame); for (const timer of timers.values()) clearTimeout(timer);
});
</script>

<template>
  <div ref="root" class="skin-scene" :class="{ 'is-paused': !running }" data-testid="skin-scene">
    <template v-for="layer in layers" :key="layer.id">
      <div v-if="interaction(layer) && !interactive" class="skin-object is-passive" :class="anchorClass(layer)" :style="{ zIndex: layer.depth }" aria-hidden="true"><img :src="layer.image" alt="" draggable="false" /></div>
      <button v-else-if="interaction(layer)" type="button" class="skin-object" :class="[anchorClass(layer), `hover-${hoverAction(layer)}`, { 'is-covered': covered.has(layer.id), 'is-passive': !interactive, 'is-active': active[layer.id], [`action-${interaction(layer)?.action}`]: true }]" :style="{ zIndex: layer.depth }" :data-skin-hotspot="layer.id" :aria-label="interaction(layer)?.label" :aria-hidden="covered.has(layer.id) || !interactive || undefined" :tabindex="covered.has(layer.id) || !interactive ? -1 : 0" :disabled="!interactive" @click="toggle(layer.id)"><img :src="layer.image" alt="" draggable="false" /></button>
      <div v-else class="skin-layer" :class="[anchorClass(layer), `idle-${idleAction(layer)}`]" :style="{ zIndex: layer.depth }"><img :src="layer.image" alt="" draggable="false" /></div>
    </template>
  </div>
</template>

<style scoped>
.skin-scene{position:absolute;inset:0;overflow:hidden;pointer-events:none;isolation:isolate}
.skin-layer{position:absolute;inset:0;pointer-events:none}.skin-layer img{width:100%;height:100%;object-fit:cover}.skin-layer.anchor-top-left img{object-position:left top}.skin-layer.anchor-top-right img{object-position:right top}.skin-layer.anchor-bottom-left img{object-position:left bottom}.skin-layer.anchor-bottom-right img{object-position:right bottom}
.skin-object{position:absolute;width:clamp(48px,12%,160px);aspect-ratio:1;padding:0;border:0;background:none;cursor:pointer;pointer-events:auto}.skin-object img{width:100%;height:100%;object-fit:contain;pointer-events:none}.skin-object:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px;border-radius:8px}.skin-object.is-covered{visibility:hidden;pointer-events:none}.skin-object.is-passive{pointer-events:none}
.skin-object.anchor-center{left:44%;top:44%}.skin-object.anchor-top-left{top:3%;left:3%}.skin-object.anchor-top-right{top:3%;right:3%}.skin-object.anchor-bottom-left{bottom:3%;left:3%}.skin-object.anchor-bottom-right{bottom:3%;right:3%}
.idle-sway img{animation:skin-sway 5s ease-in-out infinite}.idle-glow img{animation:skin-glow 4s ease-in-out infinite}.hover-sway:hover img,.hover-sway:focus-visible img,.action-sway.is-active img{animation:skin-sway 1.2s ease-in-out}.hover-glow:hover img,.hover-glow:focus-visible img,.action-glow.is-active img{animation:skin-glow 1.2s ease-in-out}.is-paused img{animation:none!important}
@keyframes skin-sway{0%,100%{transform:rotate(0)}25%{transform:rotate(3deg)}75%{transform:rotate(-3deg)}}@keyframes skin-glow{0%,100%{filter:brightness(1)}50%{filter:brightness(1.25)}}
</style>
