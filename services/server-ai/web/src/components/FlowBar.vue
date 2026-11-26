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
        <span
          v-for="s in segments"
          :key="s.key"
          class="seg"
          :class="s.tone"
          :style="{ width: `${share(s.points)}%` }"
        ></span>
      </template>
      <span v-else class="seg none" style="width: 100%"></span>
    </div>
    <ul class="legend">
      <li v-for="s in segments" :key="s.key">
        <i class="sw" :class="s.tone"></i>
        <div class="body">
          <div class="top">
            <span class="title"><strong>{{ s.label }}</strong><span class="pct num">{{ percent(s.points) }}</span></span>
            <Money :points="s.points" size="sm" compact-pts />
          </div>
          <span class="note">{{ s.note }}</span>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.flow { display: flex; flex-direction: column; gap: 16px; }
.track {
  display: flex;
  height: 12px;
  border-radius: 99px;
  overflow: hidden;
  background: var(--canvas-soft);
  gap: 2px;
}
.seg { height: 100%; min-width: 3px; border-radius: 2px; transform-origin: left; animation: grow 0.6s var(--ease) both; }
.seg.used, .sw.used { background: var(--ink); }
.seg.left, .sw.left { background: var(--coral); }
.seg.lapsed, .sw.lapsed { background: var(--ink-4); opacity: 0.5; }
.seg.none { background: var(--canvas-soft); animation: none; }
@keyframes grow { from { transform: scaleX(0); } }

.legend { display: flex; flex-direction: column; gap: 12px; }
.legend li {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--line);
}
.legend li:last-child { border-bottom: 0; padding-bottom: 0; }
.sw { width: 10px; height: 10px; border-radius: 3px; margin-top: 4px; flex: none; }
.body { flex: 1; min-width: 0; }
.top { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; flex-wrap: wrap; }
.title { display: inline-flex; align-items: baseline; gap: 6px; }
.title strong { font-size: 13px; }
.pct { font-size: 11.5px; color: var(--ink-3); font-weight: 600; }
.note { display: block; color: var(--ink-3); font-size: 12px; line-height: 1.48; margin-top: 2px; }
</style>
