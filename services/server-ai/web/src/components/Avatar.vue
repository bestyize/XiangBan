<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{ seed: string; size?: number }>(), { size: 32 });

const initials = computed(() => props.seed.replace(/[^0-9a-z]/gi, '').slice(0, 2).toUpperCase() || '?');
const style = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  fontSize: `${Math.round(props.size * 0.36)}px`,
}));
</script>

<template>
  <span class="avatar" :style="style" aria-hidden="true">{{ initials }}</span>
</template>

<style scoped>
.avatar {
  display: inline-grid;
  place-items: center;
  border-radius: 50%;
  font-weight: 650;
  letter-spacing: 0.02em;
  background: var(--subtle);
  color: var(--ink-2);
  flex: none;
  user-select: none;
}
</style>
