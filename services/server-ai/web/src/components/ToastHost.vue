<script setup lang="ts">
import { dismiss, toasts } from '../toast';
import Icon from './Icon.vue';
</script>

<template>
  <div class="host" aria-live="polite" role="status">
    <TransitionGroup name="toast">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="t.kind">
        <Icon :name="t.kind === 'ok' ? 'circle-check' : 'alert'" :size="16" />
        <span>{{ t.text }}</span>
        <button type="button" class="close" aria-label="关闭提示" @click="dismiss(t.id)">
          <Icon name="x" :size="14" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.host {
  position: fixed;
  left: 50%;
  bottom: 24px;
  transform: translateX(-50%);
  z-index: 90;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  width: min(540px, calc(100vw - 28px));
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 9px;
  width: fit-content;
  max-width: 100%;
  padding: 10px 12px 10px 14px;
  border-radius: 11px;
  background: #19171a;
  color: #f3f1f5;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: var(--shadow-md);
  font-weight: 550;
  font-size: 13px;
  line-height: 1.45;
}
.toast > svg { color: #34d399; }
.toast.warn > svg { color: #fbbf24; }
.close {
  border: none;
  background: none;
  color: rgba(243, 241, 245, 0.6);
  padding: 3px;
  border-radius: 6px;
  cursor: pointer;
  display: grid;
  place-items: center;
}
.close:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.12);
}
.toast-enter-active, .toast-leave-active {
  transition: opacity 0.18s, transform 0.2s var(--ease);
}
.toast-enter-from { opacity: 0; transform: translateY(10px) scale(0.98); }
.toast-leave-to { opacity: 0; transform: translateY(6px); }
</style>
