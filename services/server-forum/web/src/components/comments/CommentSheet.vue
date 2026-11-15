<script setup lang="ts">
import { onBeforeUnmount, ref, watchPostEffect } from 'vue'
import ToastHost from '@/components/ToastHost.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const dialog = ref<HTMLDialogElement | null>(null)
const inset = ref(0)
const visibleHeight = ref(0)
let backdropDown = false

watchPostEffect((cleanup) => {
  const el = dialog.value
  if (!el) return
  if (!props.open) {
    if (el.open) el.close()
    return
  }
  if (!el.open) el.showModal()

  // iOS and Android browsers can shrink only the visual viewport when the keyboard opens.
  const viewport = window.visualViewport
  if (!viewport) return
  const sync = () => {
    visibleHeight.value = Math.round(viewport.height)
    inset.value = Math.max(0, Math.round(window.innerHeight - viewport.height - viewport.offsetTop))
  }
  sync()
  viewport.addEventListener('resize', sync)
  viewport.addEventListener('scroll', sync)
  cleanup(() => {
    viewport.removeEventListener('resize', sync)
    viewport.removeEventListener('scroll', sync)
  })
})

onBeforeUnmount(() => dialog.value?.close())

function backdropClick(event: MouseEvent) {
  if (backdropDown && event.target === dialog.value) emit('close')
  backdropDown = false
}
</script>

<template>
  <Teleport to="body">
    <dialog
      ref="dialog"
      class="comment-sheet"
      aria-label="评论编辑"
      :style="{ '--ime-inset': inset + 'px', '--visible-height': visibleHeight ? visibleHeight + 'px' : undefined, '--sheet-safe-bottom': inset ? '0px' : undefined }"
      @cancel.prevent="emit('close')"
      @close="open && !dialog?.open && emit('close')"
      @pointerdown="backdropDown = $event.target === dialog"
      @click="backdropClick"
    >
      <div class="sheet-body"><slot /></div>
      <ToastHost v-if="open" class="sheet-toasts" />
    </dialog>
  </Teleport>
</template>

<style scoped>
:global(html:has(.comment-sheet[open])) { overflow: hidden; }
.comment-sheet {
  --sheet-safe-bottom: env(safe-area-inset-bottom, 0px);
  --sheet-max-height: min(640px, calc(var(--visible-height, 100dvh) - 40px));
  width: min(560px, calc(100vw - 32px));
  max-width: none;
  max-height: var(--sheet-max-height);
  padding: 0;
  border: 1px solid var(--line);
  border-radius: 16px;
  overflow: hidden;
  background: var(--surface);
  color: var(--ink);
  box-shadow: 0 24px 64px -16px #0006;
}
.comment-sheet::backdrop { background: #0006; }
.sheet-body {
  max-height: calc(var(--sheet-max-height) - 2px);
  padding-bottom: var(--sheet-safe-bottom);
  overflow: auto;
  overscroll-behavior: contain;
}
.comment-sheet .sheet-toasts { position: absolute; bottom: calc(10px + var(--sheet-safe-bottom)); width: calc(100% - 32px); }
@media (max-width: 767px) {
  .comment-sheet {
    --sheet-max-height: min(640px, calc(var(--visible-height, 100dvh) - 12px));
    inset: auto 0 var(--ime-inset, 0px);
    margin: 0;
    width: 100%;
    border: 0;
    border-radius: 16px 16px 0 0;
  }
}
</style>
