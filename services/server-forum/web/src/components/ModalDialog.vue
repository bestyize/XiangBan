<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'

const props = defineProps<{ open: boolean; title: string; width?: number }>()
const emit = defineEmits<{ close: [] }>()

const dialog = ref<HTMLDialogElement | null>(null)

// 用原生 <dialog>：自带焦点圈定、Esc 关闭和无障碍语义。
watch(
  () => props.open,
  async open => {
    await nextTick()
    const el = dialog.value
    if (!el) return
    if (open && !el.open) el.showModal()
    if (!open && el.open) el.close()
  },
  { immediate: true },
)

function onBackdrop(e: MouseEvent) {
  if (e.target === dialog.value) emit('close')
}
</script>

<template>
  <dialog
    ref="dialog"
    class="dialog"
    :style="{ '--w': `${width ?? 420}px` }"
    :aria-label="title"
    @close="emit('close')"
    @click="onBackdrop"
  >
    <div class="body">
      <header>
        <h2>{{ title }}</h2>
        <button type="button" class="close" aria-label="关闭" @click="emit('close')">
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
.close:hover {
  background: var(--surface-2);
}
.close svg {
  width: 16px;
  height: 16px;
  stroke: var(--ink-2);
  stroke-width: 1.8;
  stroke-linecap: round;
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
}
</style>
