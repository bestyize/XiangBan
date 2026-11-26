<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import UiIcon from '../icons/UiIcon.vue'

/**
 * 统一 24px 圆角居中卡片弹窗（PC 与移动 H5 均保持居中，不被压到底部）。
 * 入场 220ms 标准减速，遮罩 200ms 淡入，自带原生 <dialog> 焦点圈定与 Esc 关闭。
 */
const props = withDefaults(
  defineProps<{
    open: boolean
    title?: string
    width?: number
    keepOnBackdrop?: boolean
    closeDisabled?: boolean
    hideClose?: boolean
  }>(),
  { width: 360, keepOnBackdrop: false, closeDisabled: false, hideClose: false },
)
const emit = defineEmits<{ close: [] }>()
const dialog = ref<HTMLDialogElement | null>(null)

onBeforeUnmount(() => dialog.value?.close())

watch(
  () => props.open,
  async () => {
    await nextTick()
    const el = dialog.value
    if (!el) return
    if (props.open && !el.open) el.showModal()
    if (!props.open && el.open) el.close()
  },
  { immediate: true },
)

function requestClose() {
  if (!props.closeDisabled) emit('close')
}
function onBackdrop(e: MouseEvent) {
  if (!props.keepOnBackdrop && e.target === dialog.value) requestClose()
}
</script>

<template>
  <dialog
    ref="dialog"
    class="ui-center-dialog"
    :style="{ '--ui-dialog-w': `${width}px` }"
    :aria-label="title || '确认弹窗'"
    @cancel.prevent="requestClose"
    @close="open && !dialog?.open && emit('close')"
    @click="onBackdrop"
  >
    <div class="ui-center-card">
      <button
        v-if="!hideClose"
        type="button"
        class="ui-center-close"
        aria-label="关闭"
        :disabled="closeDisabled"
        @click="requestClose"
      >
        <span>
          <UiIcon name="close" :size="15" :stroke-width="2.1" />
        </span>
      </button>
      <div class="ui-center-scroll"><slot /></div>
      <div v-if="$slots.footer" class="ui-center-footer"><slot name="footer" /></div>
    </div>
  </dialog>
</template>

<style scoped>
.ui-center-dialog {
  margin: auto;
  padding: 0;
  border: 0;
  background: transparent;
  width: min(var(--ui-dialog-w), calc(100vw - 48px));
  max-width: 100%;
  max-height: calc(100dvh - 48px);
  overflow: visible;
  color: var(--ink);
}
.ui-center-dialog::backdrop {
  background: var(--ui-scrim);
  backdrop-filter: blur(2.5px);
  -webkit-backdrop-filter: blur(2.5px);
  animation: ui-fade var(--dur-normal) var(--ease-ui);
}
.ui-center-dialog[open] .ui-center-card {
  animation: ui-dialog-in var(--dur-dialog-in) var(--ease-ui);
}
@keyframes ui-fade { from { opacity: 0; } }
@keyframes ui-dialog-in {
  from { opacity: 0; transform: translateY(8px) scale(0.95); }
}

.ui-center-card {
  position: relative;
  display: flex;
  flex-direction: column;
  max-height: calc(100dvh - 48px);
  border-radius: var(--r-dialog);
  background: var(--surface);
  box-shadow: var(--ui-shadow-dialog);
  overflow: hidden;
}
.ui-center-close {
  position: absolute;
  top: 4px;
  right: 4px;
  z-index: 2;
  width: var(--h-touch);
  height: var(--h-touch);
  border: 0;
  background: none;
  display: grid;
  place-items: center;
  border-radius: 50%;
}
.ui-center-close span {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: transparent;
  color: var(--ink-2);
  display: grid;
  place-items: center;
  transition: background-color var(--dur-fast) var(--ease-ui), color var(--dur-fast) var(--ease-ui);
}
.ui-center-close:hover:not(:disabled) span { background: var(--surface-2); color: var(--ink); }
.ui-center-close:disabled { opacity: 0.45; cursor: not-allowed; }
.ui-center-close svg {
  width: 15px;
  height: 15px;
  stroke: currentColor;
  stroke-width: 2.1;
  stroke-linecap: round;
}
.ui-center-scroll {
  padding: 22px 18px 0;
  overflow-y: auto;
  min-height: 0;
}
.ui-center-footer {
  padding: 16px 18px 12px;
  display: grid;
  gap: 4px;
  flex: none;
}
</style>
