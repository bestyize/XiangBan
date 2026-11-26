<script setup lang="ts">
import { dismiss, toasts } from '../toast';
import Icon from './Icon.vue';
</script>

<template>
  <div class="host" aria-live="polite" role="status">
    <TransitionGroup name="toast">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="t.kind">
        <i class="dot" :class="t.kind === 'ok' ? 'ok' : 'warn'" aria-hidden="true"></i>
        <span>{{ t.text }}</span>
        <button type="button" class="close" aria-label="关闭提示" @click="dismiss(t.id)">
          <Icon name="x" :size="13" />
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
  width: min(520px, calc(100vw - 28px));
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  width: fit-content;
  max-width: 100%;
  padding: 9px 12px 9px 15px;
  border-radius: var(--r-pill);
  background: var(--surface);
  color: var(--ink-1);
  border: 1px solid var(--line);
  box-shadow: var(--shadow-toast);
  font-weight: 550;
  font-size: 13px;
  line-height: 1.4;
}
.close {
  border: none;
  background: none;
  color: var(--ink-3);
  width: 22px;
  height: 22px;
  border-radius: 50%;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: background 120ms, color 120ms;
}
.close:hover {
  color: var(--ink-1);
  background: var(--subtle);
}
.toast-enter-active, .toast-leave-active {
  transition: opacity 180ms, transform 180ms var(--ease);
}
.toast-enter-from { opacity: 0; transform: translateY(10px) scale(0.98); }
.toast-leave-to { opacity: 0; transform: translateY(6px); }
</style>
