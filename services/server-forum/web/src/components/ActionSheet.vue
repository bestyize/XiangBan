<script setup lang="ts">
import { watch } from 'vue'

const props = defineProps<{ open: boolean; label: string }>()
const emit = defineEmits<{ close: [] }>()

// 打开时锁住页面滚动；行内容由 slot 提供。
watch(() => props.open, (on, _, cleanup) => {
  if (!on) return
  const root = document.documentElement, previous = root.style.overflow
  root.style.overflow = 'hidden'
  cleanup(() => { root.style.overflow = previous })
})
</script>

<template>
  <Teleport to="body">
    <template v-if="open">
      <div class="sheet-backdrop" @click="emit('close')" />
      <div class="action-sheet" role="dialog" :aria-label="label" @keydown.esc="emit('close')"><slot /></div>
    </template>
  </Teleport>
</template>

<style scoped>
.sheet-backdrop {
  position: fixed;
  inset: 0;
  z-index: 39;
  background: rgba(0, 0, 0, 0.35);
}
.action-sheet {
  position: fixed;
  z-index: 40;
  left: 0;
  right: 0;
  bottom: 0;
  box-sizing: border-box;
  max-height: 80dvh;
  overflow: auto;
  padding-bottom: env(safe-area-inset-bottom);
  background: var(--surface);
  border-radius: 16px 16px 0 0;
  box-shadow: 0 -6px 24px rgba(0, 0, 0, 0.14);
}
</style>
