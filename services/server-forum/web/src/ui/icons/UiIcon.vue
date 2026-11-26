<script setup lang="ts">
import { computed } from 'vue'
import { UI_FILLED_ICONS, UI_STROKE_ICONS, type UiFilledIconName, type UiIconName, type UiStrokeIconName } from './icons'

/**
 * 强类型通用 UI 图标组件：统一 24×24 viewBox、1.8 线宽、圆头圆角与 currentColor 语义着墨。
 * 推荐尺寸档位：14 / 16 / 20 / 24px（独立点击按钮外层热区仍应 >= 44×44px）。
 */
const props = withDefaults(
  defineProps<{
    name: UiIconName
    size?: number
    strokeWidth?: number
    label?: string
  }>(),
  { size: 20, strokeWidth: 1.8 },
)

const isFilled = computed(() => props.name in UI_FILLED_ICONS)
const paths = computed(() =>
  isFilled.value
    ? UI_FILLED_ICONS[props.name as UiFilledIconName]
    : UI_STROKE_ICONS[props.name as UiStrokeIconName],
)
</script>

<template>
  <svg
    v-if="isFilled"
    class="ui-icon"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="currentColor"
    fill-rule="evenodd"
    :aria-label="label"
    :aria-hidden="!label"
  >
    <path v-for="(d, i) in paths" :key="i" :d="d" />
  </svg>
  <svg
    v-else
    class="ui-icon"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    :aria-label="label"
    :aria-hidden="!label"
  >
    <path v-for="(d, i) in paths" :key="i" :d="d" />
  </svg>
</template>

<style scoped>
.ui-icon {
  display: block;
  flex: none;
}
</style>
