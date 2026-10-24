<script setup lang="ts" generic="T extends string">
defineProps<{ options: { value: T; label: string }[]; label: string }>();
const model = defineModel<T>({ required: true });
</script>

<template>
  <div class="seg" role="group" :aria-label="label">
    <button v-for="o in options" :key="o.value" type="button" class="seg-item" :class="{ on: model === o.value }" :aria-pressed="model === o.value" @click="model = o.value">{{ o.label }}</button>
  </div>
</template>

<style scoped>
.seg { display: inline-flex; padding: 3px; border-radius: 12px; background: var(--wash); }
.seg-item { min-height: 38px; min-width: 60px; padding: 0 16px; border: 0; border-radius: 9px; background: transparent; color: var(--ink-2); font-size: 14px; cursor: pointer; }
.seg-item.on { background: #fff; color: var(--ink); font-weight: 600; box-shadow: 0 1px 2px rgba(20, 20, 20, 0.08), 0 2px 6px rgba(20, 20, 20, 0.06); }
</style>
