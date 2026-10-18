<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type { Phase } from './assistant-store.ts';

const props = withDefaults(defineProps<{ phase: Phase; level?: number; active?: boolean; size?: number }>(), { level: 0, active: true, size: 48 });
const canvas = ref<HTMLCanvasElement>();

type Rgb = [number, number, number];
const palette: Record<Phase, Rgb> = {
  idle: [92, 196, 255], listening: [84, 226, 255], thinking: [158, 140, 255],
  answering: [118, 184, 255], speaking: [128, 168, 255], attention: [255, 188, 98],
};
const color: Rgb = [...palette.idle];
const rgba = (rgb: Rgb, alpha: number) => `rgba(${rgb[0] | 0},${rgb[1] | 0},${rgb[2] | 0},${Math.max(0, Math.min(1, alpha))})`;
const shift = (rgb: Rgb, by: Rgb): Rgb => [rgb[0] + by[0], rgb[1] + by[1], rgb[2] + by[2]];

let frame = 0, last = 0, clock = 0, loud = 0;
const dpr = () => Math.min(2, window.devicePixelRatio || 1);

function draw(dt: number) {
  const el = canvas.value;
  const ctx = el?.getContext('2d');
  if (!el || !ctx) return;
  const size = props.size, ratio = dpr(), px = Math.round(size * ratio);
  if (el.width !== px) { el.width = px; el.height = px; }
  const target = palette[props.phase];
  for (let i = 0; i < 3; i++) color[i] += (target[i] - color[i]) * Math.min(1, dt * 5);
  loud += ((props.phase === 'listening' ? props.level : 0) - loud) * Math.min(1, dt * 14);

  const t = clock, c = size / 2, R = size * 0.34;
  const speaking = props.phase === 'speaking' || props.phase === 'answering';
  const energy = props.phase === 'listening' ? 0.3 + 0.7 * loud
    : speaking ? 0.45 + 0.2 * Math.sin(t * 7) * Math.sin(t * 2.3)
    : props.phase === 'thinking' ? 0.5
    : props.phase === 'attention' ? 0.55 + 0.25 * Math.sin(t * 4)
    : 0.18 + 0.08 * Math.sin(t * 1.4);
  const radius = R * (1 + 0.1 * energy);

  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.clearRect(0, 0, size, size);

  const halo = ctx.createRadialGradient(c, c, radius * 0.7, c, c, size * 0.5);
  halo.addColorStop(0, rgba(color, 0.2 + 0.4 * energy));
  halo.addColorStop(1, rgba(color, 0));
  ctx.fillStyle = halo;
  ctx.fillRect(0, 0, size, size);

  if (props.phase === 'listening' || speaking) {
    for (let i = 0; i < 2; i++) {
      const p = (t * (props.phase === 'listening' ? 0.55 : 0.8) + i / 2) % 1;
      ctx.beginPath();
      const inner = radius * 1.05, reach = c - 1.5 - inner;
      ctx.arc(c, c, inner + p * reach * (0.6 + 0.4 * energy), 0, Math.PI * 2);
      ctx.strokeStyle = rgba(color, (1 - p) * (0.15 + 0.5 * energy));
      ctx.lineWidth = 1.2;
      ctx.stroke();
    }
  }

  ctx.save();
  ctx.beginPath();
  ctx.arc(c, c, radius, 0, Math.PI * 2);
  ctx.clip();
  const body = ctx.createRadialGradient(c - radius * 0.35, c - radius * 0.45, radius * 0.1, c, c, radius * 1.05);
  body.addColorStop(0, rgba(shift(color, [120, 90, 40]), 1));
  body.addColorStop(0.45, rgba(color, 1));
  body.addColorStop(1, rgba(shift(color, [-90, -80, -30]), 1));
  ctx.fillStyle = body;
  ctx.fillRect(0, 0, size, size);
  ctx.globalCompositeOperation = 'lighter';
  const speed = props.phase === 'thinking' ? 2.4 : 0.9 + energy;
  for (let i = 0; i < 3; i++) {
    const a = t * speed * (i % 2 ? -1 : 1) + i * 2.1;
    const bx = c + Math.cos(a) * radius * 0.45, by = c + Math.sin(a * 1.3) * radius * 0.45;
    const blob = ctx.createRadialGradient(bx, by, 0, bx, by, radius * (0.7 + 0.25 * energy));
    const tint = shift(color, i === 0 ? [60, -40, 60] : i === 1 ? [-30, 40, 70] : [70, 50, -20]);
    blob.addColorStop(0, rgba(tint, 0.55));
    blob.addColorStop(1, rgba(tint, 0));
    ctx.fillStyle = blob;
    ctx.fillRect(0, 0, size, size);
  }
  ctx.restore();

  ctx.beginPath();
  ctx.arc(c, c, radius, 0, Math.PI * 2);
  ctx.strokeStyle = 'rgba(255,255,255,0.35)';
  ctx.lineWidth = 0.8;
  ctx.stroke();
  const gloss = ctx.createLinearGradient(c, c - radius, c, c);
  gloss.addColorStop(0, 'rgba(255,255,255,0.55)');
  gloss.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.beginPath();
  ctx.ellipse(c - radius * 0.15, c - radius * 0.52, radius * 0.5, radius * 0.24, -0.3, 0, Math.PI * 2);
  ctx.fillStyle = gloss;
  ctx.fill();

  if (props.phase === 'thinking') {
    const a = t * 3.2;
    ctx.beginPath();
    ctx.arc(c, c, radius * 1.32, a, a + Math.PI * 0.7);
    ctx.strokeStyle = rgba(shift(color, [90, 80, 0]), 0.9);
    ctx.lineWidth = 1.8;
    ctx.lineCap = 'round';
    ctx.stroke();
  }
}

function loop(now: number) {
  frame = requestAnimationFrame(loop);
  const dt = Math.min(0.05, last ? (now - last) / 1000 : 0.016);
  last = now; clock += dt;
  draw(dt);
}
function run() { if (!frame && props.active) { last = 0; frame = requestAnimationFrame(loop); } }
function halt() { if (frame) cancelAnimationFrame(frame); frame = 0; }
watch(() => props.active, active => (active ? run() : halt()));
onMounted(() => { draw(0.016); run(); });
onBeforeUnmount(halt);
</script>

<template>
  <canvas ref="canvas" class="orb" :style="{ width: `${size}px`, height: `${size}px` }" aria-hidden="true"></canvas>
</template>

<style scoped>
.orb{display:block}
</style>
