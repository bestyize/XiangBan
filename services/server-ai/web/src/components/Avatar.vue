<script setup lang="ts">
import { computed } from 'vue';
import { prefs } from '../prefs';

const props = withDefaults(defineProps<{ seed: string; size?: number }>(), { size: 32 });

const hue = computed(() => [...props.seed].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 360, 7));
const initials = computed(() => props.seed.replace(/[^0-9a-z]/gi, '').slice(0, 2).toUpperCase() || '?');
const style = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  fontSize: `${Math.round(props.size * 0.36)}px`,
  background: prefs.theme === 'dark' ? `hsl(${hue.value} 32% 24%)` : `hsl(${hue.value} 42% 91%)`,
  color: prefs.theme === 'dark' ? `hsl(${hue.value} 72% 82%)` : `hsl(${hue.value} 48% 28%)`,
}));
</script>

<template>
  <span class="avatar" :style="style" aria-hidden="true">{{ initials }}</span>
</template>

<style scoped>
.avatar {
  display: inline-grid;
  place-items: center;
  border-radius: 9px;
  font-weight: 700;
  letter-spacing: 0.02em;
  flex: none;
  user-select: none;
}
</style>
