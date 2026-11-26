<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import UiIcon from '../icons/UiIcon.vue'

/**
 * 响应式业务弹窗（升级原 ModalDialog）：
 * - PC 端（≥768px）：居中弹窗（16px 圆角）
 * - 移动 H5（≤767px）：通过 mobileMode 显式选择 'sheet'（20px 顶圆角贴底）或 'center'（24px 居中卡）
 */
const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    width?: number
    scrollable?: boolean
    keepOnBackdrop?: boolean
    closeDisabled?: boolean
    hideHeader?: boolean
    mobileMode?: 'sheet' | 'center'
  }>(),
  { width: 420, scrollable: false, keepOnBackdrop: false, closeDisabled: false, hideHeader: false, mobileMode: 'sheet' },
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

function onBackdrop(e: MouseEvent) {
  if (!props.keepOnBackdrop && e.target === dialog.value) requestClose()
}
function requestClose() {
  if (!props.closeDisabled) emit('close')
}
</script>

<template>
  <dialog
    ref="dialog"
    class="dialog"
    :class="[`mobile-${mobileMode}`, { scrollable }]"
    :style="{ '--w': `${width}px` }"
    :aria-label="title"
    @cancel.prevent="requestClose"
    @close="open && !dialog?.open && emit('close')"
    @click="onBackdrop"
  >
    <div class="body">
      <header v-if="!hideHeader">
        <h2>{{ title }}</h2>
        <button type="button" class="close" aria-label="关闭" :disabled="closeDisabled" @click="requestClose">
          <UiIcon name="close" :size="16" />
        </button>
      </header>
      <slot />
    </div>
  </dialog>
</template>

<style scoped>
.dialog {
  width: min(var(--w), calc(100vw - 24px));
  padding: 0;
  border: 1px solid var(--line);
  border-radius: var(--r-card);
  background: var(--surface);
  color: var(--ink);
  box-shadow: var(--ui-shadow-dialog);
}
.dialog::backdrop {
  background: var(--ui-scrim);
  animation: ui-fade var(--dur-normal) var(--ease-ui);
}
.dialog[open] {
  animation: ui-rise var(--dur-dialog-in) var(--ease-ui);
}
@keyframes ui-fade { from { opacity: 0; } }
@keyframes ui-rise {
  from { opacity: 0; transform: translateY(8px) scale(0.96); }
}
.body { padding: 20px 22px 22px; }
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
h2 { margin: 0; font-size: var(--fs-lg); }
.close {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 0;
  background: none;
  border-radius: 50%;
  margin-right: -6px;
}
.close:hover:not(:disabled) { background: var(--surface-2); }
.close:disabled { opacity: 0.45; }
.close svg { width: 16px; height: 16px; stroke: var(--ink-2); stroke-width: 1.8; stroke-linecap: round; }

.dialog.scrollable {
  --dialog-max-height: min(860px, calc(100dvh - 32px));
  max-height: var(--dialog-max-height);
  overflow: hidden;
}
.scrollable .body {
  display: flex;
  flex-direction: column;
  max-height: calc(var(--dialog-max-height) - 2px);
  padding: 0;
}
.scrollable header {
  flex: none;
  margin: 0;
  padding: 14px 22px;
  border-bottom: 1px solid var(--line);
}

@media (max-width: 767px) {
  .dialog.mobile-sheet {
    margin: auto auto 0;
    width: 100vw;
    max-width: 100vw;
    border-radius: var(--r-sheet) var(--r-sheet) 0 0;
    border-bottom: 0;
    box-shadow: var(--ui-shadow-sheet);
    animation: ui-sheet-rise var(--dur-sheet-in) var(--ease-ui);
  }
  .dialog.mobile-sheet .body { padding-bottom: calc(22px + env(safe-area-inset-bottom)); }
  .dialog.mobile-sheet.scrollable { --dialog-max-height: calc(100dvh - 12px); }
  .dialog.mobile-sheet.scrollable header { padding-inline: 16px; }

  .dialog.mobile-center {
    margin: auto;
    width: min(var(--w), calc(100vw - 48px));
    border-radius: var(--r-dialog);
  }
}
@keyframes ui-sheet-rise { from { transform: translateY(100%); } }
</style>
