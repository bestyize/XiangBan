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
      <li v-for="s in segments" :key="s.key" :title="s.note">
        <i class="dot" :class="s.tone === 'left' ? 'brand' : s.tone === 'used' ? 'ink' : ''"></i>
        <span class="title"><strong>{{ s.label }}</strong><span class="pct num">{{ percent(s.points) }}</span></span>
        <Money :points="s.points" size="sm" compact-pts />
      </li>
    </ul>
  </div>
</template>

<style scoped>
.flow { display: flex; flex-direction: column; gap: 14px; }
.track {
  display: flex;
  height: 10px;
  border-radius: 99px;
  overflow: hidden;
  background: var(--subtle);
  gap: 2px;
}
.seg { height: 100%; min-width: 3px; border-radius: 2px; transform-origin: left; animation: grow 500ms var(--ease) both; }
.seg.used { background: var(--ink-2); }
.seg.left { background: var(--brand); }
.seg.lapsed { background: var(--ink-4); opacity: 0.45; }
.seg.none { background: var(--subtle); animation: none; }
@keyframes grow { from { transform: scaleX(0); } }

.legend { display: flex; flex-direction: column; }
.legend li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 0;
  border-bottom: 1px solid var(--line);
}
.legend li:last-child { border-bottom: 0; padding-bottom: 0; }
.dot.ink { background: var(--ink-2); }
.title { flex: 1; display: inline-flex; align-items: baseline; gap: 6px; min-width: 0; }
.title strong { font-size: 13px; font-weight: 600; }
.pct { font-size: 12px; color: var(--ink-3); }
</style>
