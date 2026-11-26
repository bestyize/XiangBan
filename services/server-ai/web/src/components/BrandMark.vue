<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 32 });
</script>

<template>
  <span
    class="brand-mark"
    :style="{ width: `${size}px`, height: `${size}px`, borderRadius: `${Math.round(size * 0.28)}px` }"
    role="img"
    aria-label="小伴 AI"
  >
    <svg :width="Math.round(size * 0.64)" :height="Math.round(size * 0.64)" viewBox="0 0 24 24" fill="none">
      <path d="M9.5 3.8l1.6 4.4 4.4 1.6-4.4 1.6-1.6 4.4-1.6-4.4-4.4-1.6 4.4-1.6 1.6-4.4z" fill="#fff" />
      <path d="M17.4 12.6l.9 2.5 2.5.9-2.5.9-.9 2.5-.9-2.5-2.5-.9 2.5-.9.9-2.5z" fill="rgba(255,255,255,0.84)" />
    </svg>
  </span>
</template>

<style scoped>
.brand-mark {
  flex: none;
  background: var(--coral-grad);
  display: grid;
  place-items: center;
  box-shadow: 0 4px 12px rgba(238, 79, 109, 0.28);
}
</style>
