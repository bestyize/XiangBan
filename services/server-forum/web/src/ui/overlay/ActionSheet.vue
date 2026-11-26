<script setup lang="ts">
import BottomSheet from './BottomSheet.vue'
import UiIcon from '../icons/UiIcon.vue'
import type { UiIconName } from '../icons/icons'

export interface ActionSheetOption {
  label: string
  icon?: UiIconName
  danger?: boolean
  disabled?: boolean
  onSelect?: () => void
}

/**
 * 底部操作菜单（与 BottomSheet 完全一致）：
 * 20px 顶圆角纯白壳、顶部 36×4 拖拽条、轻标题、左对齐 52px 操作行 + 左右 16px 内缩细分隔线，
 * 底部以同样细线分隔居中无底色「取消」行。
 */
withDefaults(
  defineProps<{
    open: boolean
    label: string
    title?: string
    context?: string
    items?: ActionSheetOption[]
    cancelLabel?: string
    hideCancel?: boolean
  }>(),
  { cancelLabel: '取消', hideCancel: false },
)
const emit = defineEmits<{ close: [] }>()

function pick(item: ActionSheetOption) {
  if (item.disabled) return
  emit('close')
  item.onSelect?.()
}
</script>

<template>
  <BottomSheet
    :open="open"
    :label="label"
    :title="title || label"
    :subtitle="context"
    :show-handle="true"
    :cancel-label="hideCancel ? undefined : cancelLabel"
    hide-close
    flush
    @close="emit('close')"
  >
    <div class="ui-as-list">
      <template v-if="items?.length">
        <button
          v-for="(item, idx) in items"
          :key="idx"
          type="button"
          class="ui-as-row"
          :class="{ 'is-danger': item.danger }"
          :disabled="item.disabled"
          @click="pick(item)"
        >
          <UiIcon v-if="item.icon" :name="item.icon" :size="20" />
          <span>{{ item.label }}</span>
        </button>
      </template>
      <slot />
    </div>
  </BottomSheet>
</template>

<style scoped>
.ui-as-row {
  position: relative;
  width: 100%;
  min-height: var(--h-sheet-row);
  padding: 12px 16px;
  border: 0;
  background: transparent;
  color: var(--ink);
  font-size: var(--fs-body);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 12px;
  text-align: left;
  transition: background-color var(--dur-fast) var(--ease-ui);
}
.ui-as-row + .ui-as-row::before {
  content: '';
  position: absolute;
  top: 0;
  left: 16px;
  right: 16px;
  height: 0.5px;
  background: var(--line);
}
.ui-as-row:hover:not(:disabled) { background: var(--surface-2); }
.ui-as-row.is-danger { color: var(--ui-danger); font-weight: 600; }
.ui-as-row:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
