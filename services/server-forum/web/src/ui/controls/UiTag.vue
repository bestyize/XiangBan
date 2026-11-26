<script setup lang="ts">
/**
 * 状态 / 语义标签（不使用彩色糖果底）：
 * 统一中性灰底（或白底细描边）+ 可选 6px 语义色圆点 + 中性文字。
 */
withDefaults(
  defineProps<{
    tone?: 'neutral' | 'success' | 'warn' | 'danger' | 'info'
    outline?: boolean
  }>(),
  { tone: 'neutral', outline: false },
)
</script>

<template>
  <span class="ui-tag" :class="[`is-${tone}`, { 'is-outline': outline }]">
    <i v-if="tone !== 'neutral'" class="ui-tag-dot" aria-hidden="true" />
    <slot />
  </span>
</template>

<style scoped>
.ui-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: var(--r-pill);
  font-size: var(--fs-caption);
  font-weight: 550;
  line-height: 16px;
  background: var(--surface-2);
  color: var(--ink);
  white-space: nowrap;
}
.ui-tag.is-outline {
  background: var(--surface);
  color: var(--ink-2);
  border: 1px solid var(--line);
}
.ui-tag-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex: none;
}
.ui-tag.is-success .ui-tag-dot { background: var(--ui-success); }
.ui-tag.is-warn .ui-tag-dot { background: var(--ui-warn); }
.ui-tag.is-danger .ui-tag-dot { background: var(--ui-danger); }
.ui-tag.is-info .ui-tag-dot { background: var(--ui-info); }
</style>
