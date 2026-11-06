<script setup lang="ts">
// 底部栏上的图标按钮（赞、收藏等）；图标由默认 slot 提供，按下后图标变实心并染品牌色。
defineProps<{ label: string; text?: string | number; active?: boolean; pressed?: boolean }>()
</script>

<template>
  <button type="button" class="dock-btn" :class="{ on: active }" :aria-label="label" :aria-pressed="pressed ?? active"><slot /><span v-if="text !== undefined && text !== ''">{{ text }}</span></button>
</template>

<style scoped>
.dock-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 3px;
  min-width: 44px;
  height: 44px;
  padding: 0 5px;
  border: 0;
  background: none;
  color: var(--ink-2);
  font-size: 12px;
  white-space: nowrap;
}
.dock-btn.on { color: var(--brand); }
.dock-btn.on :deep(svg) { fill: currentColor; }
</style>
