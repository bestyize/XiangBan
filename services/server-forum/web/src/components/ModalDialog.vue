<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps<{ open: boolean; title: string; width?: number; scrollable?: boolean; keepOnBackdrop?: boolean; closeDisabled?: boolean }>()
const emit = defineEmits<{ close: [] }>()

const dialog = ref<HTMLDialogElement | null>(null)
onBeforeUnmount(() => dialog.value?.close())

// 用原生 <dialog>：自带焦点圈定、Esc 关闭和无障碍语义。
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
function requestClose() { if (!props.closeDisabled) emit('close') }
</script>

<template>
  <dialog
    ref="dialog"
    class="dialog"
    :class="{ scrollable }"
    :style="{ '--w': `${width ?? 420}px` }"
    :aria-label="title"
    @cancel.prevent="requestClose"
    @close="open && !dialog?.open && emit('close')"
    @click="onBackdrop"
  >
    <div class="body">
      <header>
        <h2>{{ title }}</h2>
        <button type="button" class="close" aria-label="关闭" :disabled="closeDisabled" @click="requestClose">
          <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M5 5l10 10M15 5 5 15" /></svg>
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
  border-radius: 14px;
  background: var(--surface);
  color: var(--ink);
  box-shadow: 0 24px 60px -20px rgba(0, 0, 0, 0.45);
}
.dialog::backdrop {
  background: rgba(0, 0, 0, 0.42);
}
.dialog[open] {
  animation: rise 0.18s ease-out;
}
@keyframes rise {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
}
.body {
  padding: 20px 22px 22px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
h2 {
  margin: 0;
  font-size: var(--fs-lg);
}
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
.close:hover:not(:disabled) {
  background: var(--surface-2);
}
.close:disabled { opacity: 0.45; }
.close svg {
  width: 16px;
  height: 16px;
  stroke: var(--ink-2);
  stroke-width: 1.8;
  stroke-linecap: round;
}
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
  .dialog {
    margin: auto auto 0;
    width: 100vw;
    max-width: 100vw;
    border-radius: 16px 16px 0 0;
    border-bottom: 0;
  }
  .body {
    padding-bottom: calc(22px + env(safe-area-inset-bottom));
  }
  .dialog.scrollable { --dialog-max-height: calc(100dvh - 12px); }
  .scrollable header { padding-inline: 16px; }
}
</style>
