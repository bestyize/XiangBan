<script setup lang="ts">
import UiButton from './UiButton.vue'

defineProps<{ title: string; desc?: string; actionLabel?: string }>()
const emit = defineEmits<{ action: [] }>()
</script>

<template>
  <div class="ui-empty">
    <div v-if="$slots.icon" class="ui-empty-icon"><slot name="icon" /></div>
    <strong class="ui-empty-title">{{ title }}</strong>
    <p v-if="desc" class="ui-empty-desc">{{ desc }}</p>
    <UiButton v-if="actionLabel" variant="outline" size="sm" @click="emit('action')">{{ actionLabel }}</UiButton>
  </div>
</template>

<style scoped>
.ui-empty {
  padding: 28px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}
.ui-empty-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--r-card);
  background: var(--surface);
  border: 1px solid var(--line);
  display: grid;
  place-items: center;
  color: var(--ink-2);
}
.ui-empty-title { font-size: var(--fs-section); font-weight: 650; color: var(--ink); }
.ui-empty-desc { margin: 0; font-size: var(--fs-secondary); color: var(--ink-2); max-width: 280px; }
</style>
