<script setup lang="ts">
import { computed } from 'vue';
import type { FlowSegment } from '../types';
import Money from './Money.vue';

const props = defineProps<{ segments: FlowSegment[] }>();

const total = computed(() => props.segments.reduce((sum, s) => sum + Math.max(0, s.points), 0));

function share(points: number): number {
  return total.value > 0 ? (Math.max(0, points) / total.value) * 100 : 0;
}

function percent(points: number): string {
  const value = share(points);
  return value > 0 && value < 1 ? '<1%' : `${Math.round(value)}%`;
}
</script>

<template>
  <div class="flow">
    <div class="track" role="img" :aria-label="segments.map((s) => `${s.label} ${percent(s.points)}`).join('，')">
      <template v-if="total > 0">
        <span v-for="s in segments" :key="s.key" class="seg" :class="s.tone" :style="{ width: `${share(s.points)}%` }"></span>
      </template>
      <span v-else class="seg none" style="width: 100%"></span>
    </div>
    <ul class="legend">
      <li v-for="s in segments" :key="s.key">
        <i class="sw" :class="s.tone"></i>
        <span class="name">{{ s.label }}</span>
        <Money :points="s.points" size="md" />
        <span class="pct num">{{ percent(s.points) }}</span>
        <span class="note">{{ s.note }}</span>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.flow { display: flex; flex-direction: column; gap: 18px; }
.track { display: flex; height: 16px; border-radius: 8px; overflow: hidden; background: var(--line); gap: 2px; }
.seg { height: 100%; min-width: 3px; transform-origin: left; animation: grow 0.7s cubic-bezier(0.2, 0.7, 0.2, 1) both; }
.seg:first-child { border-radius: 8px 0 0 8px; }
.seg:last-child { border-radius: 0 8px 8px 0; }
.seg.used, .sw.used { background: var(--tide); }
.seg.lapsed, .sw.lapsed { background: #c3cdd1; }
.seg.left, .sw.left { background: var(--mint); }
.seg.none { background: var(--line); animation: none; }
@keyframes grow { from { transform: scaleX(0); } }

.legend { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px 28px; }
.legend li { display: grid; grid-template-columns: auto 1fr auto; column-gap: 8px; row-gap: 2px; align-items: baseline; }
.sw { width: 10px; height: 10px; border-radius: 3px; align-self: center; }
.name { font-weight: 600; }
.pct { color: var(--faint); font-size: 12.5px; font-weight: 600; }
.note { grid-column: 2 / -1; color: var(--muted); font-size: 12.5px; line-height: 1.5; }
.legend li :deep(.money) { grid-column: 3; grid-row: 1; }
.legend li .pct { display: none; }
@media (max-width: 760px) { .legend { grid-template-columns: 1fr; } }
</style>
