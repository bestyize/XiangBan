<script setup lang="ts">
withDefaults(defineProps<{ size?: number; label?: string }>(), { size: 22, label: '加载中' })
</script>

<template>
  <span class="ui-spinner" role="status" :aria-label="label" :style="{ width: `${size}px`, height: `${size}px` }" />
</template>

<style scoped>
.ui-spinner {
  display: inline-block;
  border-radius: 50%;
  border: 2.2px solid var(--line);
  border-top-color: var(--brand-fill);
  animation: ui-spin 0.75s linear infinite;
}
@keyframes ui-spin { to { transform: rotate(360deg); } }
</style>
