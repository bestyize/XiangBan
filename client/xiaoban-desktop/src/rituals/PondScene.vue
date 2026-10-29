<script setup lang="ts">
import { onActivated, onDeactivated, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import createPond, { type PondRenderer } from '../../public/wallpapers/builtin/wish-pond.js';
import type { Wish } from './journal.ts';
const props = withDefaults(defineProps<{ paused?: boolean; fps?: 30 | 60; wishes?: Wish[]; interactive?: boolean }>(), { paused: false, fps: 30, wishes: () => [], interactive: false });
const emit = defineEmits<{ water: [point: { x: number; y: number }] }>();
const canvas = ref<HTMLCanvasElement>();
let renderer: PondRenderer | null = null, observer: ResizeObserver | null = null, frame = 0, previous = 0, elapsed = 0;
let active = true;
const reduce = window.matchMedia('(prefers-reduced-motion: reduce)');
function draw() { renderer?.render({ time: elapsed }); }
function resize() {
  if (!canvas.value) return;
  const rect = canvas.value.getBoundingClientRect(), dpr = Math.min(window.devicePixelRatio || 1, 2);
  canvas.value.width = Math.max(1, Math.round(rect.width * dpr)); canvas.value.height = Math.max(1, Math.round(rect.height * dpr));
  renderer?.resize({ dpr }); draw();
}
function tick(now: number) {
  frame = 0;
  if (!active || props.paused || document.hidden || reduce.matches) return;
  if (!previous || now - previous >= 1000 / props.fps - 1) { elapsed += previous ? Math.min((now - previous) / 1000, .1) : 0; previous = now; draw(); }
  frame = requestAnimationFrame(tick);
}
function sync() { if (frame) cancelAnimationFrame(frame); frame = 0; previous = 0; if (active) draw(); if (active && !props.paused && !document.hidden && !reduce.matches) frame = requestAnimationFrame(tick); }
watch(() => [props.paused, props.fps], sync);
watch(() => props.wishes, wishes => { renderer?.setWishes(wishes); draw(); }, { deep: true });
function click(event: MouseEvent) {
  if (!props.interactive || !canvas.value) return;
  const rect = canvas.value.getBoundingClientRect(), x = (event.clientX - rect.left) / rect.width, y = (event.clientY - rect.top) / rect.height;
  if (x < .1 || x > .9 || y < .38 || y > .85) return;
  renderer?.ripple(x, y, elapsed); draw(); emit('water', { x, y });
}
onMounted(() => {
  renderer = createPond({ canvas: canvas.value!, settings: { speed: .6 } }); renderer.setWishes(props.wishes);
  observer = new ResizeObserver(resize); observer.observe(canvas.value!); resize(); sync();
  document.addEventListener('visibilitychange', sync); reduce.addEventListener('change', sync);
});
onActivated(() => { active = true; sync(); });
onDeactivated(() => { active = false; sync(); });
onBeforeUnmount(() => { if (frame) cancelAnimationFrame(frame); observer?.disconnect(); renderer?.destroy(); document.removeEventListener('visibilitychange', sync); reduce.removeEventListener('change', sync); });
</script>
<template><canvas ref="canvas" class="pond-canvas" aria-hidden="true" @click="click" /></template>
<style scoped>.pond-canvas{position:absolute;inset:0;width:100%;height:100%;display:block}</style>
