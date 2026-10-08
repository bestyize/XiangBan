<script setup lang="ts">
import { toastMessage } from '@/composables/toast'
</script>

<template>
  <Transition name="toast">
    <div v-if="toastMessage" class="toast" role="status" aria-live="polite">{{ toastMessage }}</div>
  </Transition>
</template>

<style scoped>
.toast {
  position: fixed;
  left: 50%;
  bottom: calc(var(--tabbar-h) + var(--safe-bottom) + 20px);
  z-index: 200;
  max-width: calc(100vw - 32px);
  padding: 11px 20px;
  border-radius: 999px;
  background: var(--text);
  color: var(--bg);
  font-size: 14px;
  font-weight: 500;
  transform: translateX(-50%);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.45);
}

@media (min-width: 769px) {
  .toast {
    bottom: 32px;
  }
}

.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 0.25s,
    transform 0.35s var(--ease);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 14px);
}
</style>
