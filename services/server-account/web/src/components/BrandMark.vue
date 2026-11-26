<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{ size?: number }>(), { size: 32 });
const radius = computed(() => Math.round(props.size * 0.28));
const iconSize = computed(() => Math.round(props.size * 0.64));
</script>

<template>
  <span
    class="brand-mark"
    :style="{ width: `${size}px`, height: `${size}px`, borderRadius: `${radius}px` }"
    aria-hidden="true"
  >
    <svg :width="iconSize" :height="iconSize" viewBox="0 0 24 24" fill="none">
      <path d="M9.5 3.8l1.6 4.4 4.4 1.6-4.4 1.6-1.6 4.4-1.6-4.4-4.4-1.6 4.4-1.6 1.6-4.4z" fill="#fff" />
      <path
        d="M17.4 12.6l.9 2.5 2.5.9-2.5.9-.9 2.5-.9-2.5-2.5-.9 2.5-.9.9-2.5z"
        fill="rgba(255,255,255,0.84)"
      />
    </svg>
  </span>
</template>

<style scoped>
.brand-mark {
  display: grid;
  place-items: center;
  flex: none;
  background: var(--coral-grad);
  box-shadow: 0 4px 12px rgba(238, 79, 109, 0.28);
}
</style>
