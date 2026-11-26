<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 30 });
</script>

<template>
  <span
    class="brand-mark"
    :style="{ width: `${size}px`, height: `${size}px`, borderRadius: `${Math.round(size * 0.28)}px` }"
    role="img"
    aria-label="小伴 AI"
  >
    <svg :width="Math.round(size * 0.62)" :height="Math.round(size * 0.62)" viewBox="0 0 24 24" fill="none">
      <path d="M9.5 3.8l1.6 4.4 4.4 1.6-4.4 1.6-1.6 4.4-1.6-4.4-4.4-1.6 4.4-1.6 1.6-4.4z" fill="#fff" />
      <path d="M17.4 12.6l.9 2.5 2.5.9-2.5.9-.9 2.5-.9-2.5-2.5-.9 2.5-.9.9-2.5z" fill="rgba(255,255,255,0.85)" />
    </svg>
  </span>
</template>

<style scoped>
.brand-mark {
  flex: none;
  background: linear-gradient(135deg, var(--brand) 0%, var(--brand-hi) 100%);
  display: grid;
  place-items: center;
}
</style>
