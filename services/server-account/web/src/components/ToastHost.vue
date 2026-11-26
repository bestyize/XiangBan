<script setup lang="ts">
import { toasts } from '../toast';
import Icon from './Icon.vue';
</script>

<template>
  <div class="toast-host" aria-live="polite" aria-atomic="true">
    <div v-for="t in toasts" :key="t.id" class="toast" :class="t.kind">
      <Icon :name="t.kind === 'error' ? 'alert' : t.kind === 'info' ? 'info' : 'check'" :size="15" />
      <span>{{ t.text }}</span>
    </div>
  </div>
</template>

<style scoped>
.toast-host {
  position: fixed;
  left: 50%;
  bottom: 24px;
  transform: translateX(-50%);
  z-index: 450;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 15px;
  border-radius: 11px;
  background: #19171a;
  color: #f3f1f5;
  font-size: 13px;
  font-weight: 550;
  box-shadow: var(--shadow-md);
  border: 1px solid rgba(255, 255, 255, 0.1);
  animation: toastUp 0.2s var(--ease);
}
.toast.ok svg {
  color: #34d399;
}
.toast.error svg {
  color: #f87171;
}
.toast.info svg {
  color: #60a5fa;
}
@keyframes toastUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
}
</style>
