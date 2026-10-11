<script setup lang="ts">
import { useToast } from '@/stores/toast'

const toast = useToast()
</script>

<template>
  <div class="toasts" role="status" aria-live="polite">
    <div v-for="t in toast.items" :key="t.id" class="toast" :class="`is-${t.tone}`">{{ t.text }}</div>
  </div>
</template>

<style scoped>
.toasts {
  position: fixed;
  left: 50%;
  bottom: 28px;
  transform: translateX(-50%);
  z-index: 60;
  display: grid;
  gap: 8px;
  justify-items: center;
  pointer-events: none;
}
.toast {
  background: var(--ink);
  color: var(--paper);
  padding: 9px 16px;
  border-radius: 20px;
  font-size: var(--fs-sm);
  font-weight: 600;
  box-shadow: var(--shadow);
  animation: pop 0.2s ease-out;
  max-width: calc(100vw - 32px);
}
.is-error {
  background: var(--danger);
  color: #fff;
}
@keyframes pop {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
}
@media (max-width: 767px) {
  .toasts {
    bottom: calc(var(--tabbar-h) + 16px + env(safe-area-inset-bottom));
  }
}
</style>
