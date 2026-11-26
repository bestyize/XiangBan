<script setup lang="ts">
import { useToast } from '@/stores/toast'

const toast = useToast()
</script>

<template>
  <div class="toasts" role="status" aria-live="polite">
    <div v-for="t in toast.items" :key="t.id" class="toast" :class="`is-${t.tone}`">
      <span>{{ t.text }}</span>
      <RouterLink v-if="t.action" class="action" :to="t.action.to">{{ t.action.label }}</RouterLink>
    </div>
  </div>
</template>

<style scoped>
.toasts {
  position: fixed;
  left: 50%;
  bottom: 28px;
  transform: translateX(-50%);
  z-index: var(--z-toast);
  display: grid;
  gap: 8px;
  justify-items: center;
  pointer-events: none;
}
.toast {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line);
  padding: 10px 16px;
  border-radius: var(--r-pill);
  font-size: var(--fs-secondary);
  font-weight: 550;
  box-shadow: var(--ui-shadow-toast);
  animation: ui-toast-in var(--dur-toast-in) var(--ease-ui);
  max-width: calc(100vw - 32px);
}
.is-error {
  background: var(--ui-danger);
  border-color: transparent;
  color: #fff;
}
.action {
  color: var(--brand);
  font-weight: 650;
  text-decoration: underline;
  text-underline-offset: 2px;
  pointer-events: auto;
}
.is-error .action { color: #fff; }
@keyframes ui-toast-in {
  from { opacity: 0; transform: translateY(8px); }
}
@media (max-width: 767px) {
  .toasts {
    bottom: calc(var(--tabbar-h) + 16px + env(safe-area-inset-bottom));
  }
}
</style>
