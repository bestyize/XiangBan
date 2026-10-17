<script setup lang="ts">
import { dismiss, toasts } from '../toast';
import Icon from './Icon.vue';
</script>

<template>
  <div class="host" aria-live="polite" role="status">
    <TransitionGroup name="toast">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="t.kind">
        <Icon :name="t.kind === 'ok' ? 'circle-check' : 'alert'" :size="18" />
        <span>{{ t.text }}</span>
        <button type="button" class="close" aria-label="关闭提示" @click="dismiss(t.id)"><Icon name="x" :size="15" /></button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.host { position: fixed; left: 50%; bottom: 26px; transform: translateX(-50%); z-index: 90; display: flex; flex-direction: column; gap: 10px; align-items: center; width: min(560px, calc(100vw - 28px)); pointer-events: none; }
.toast { pointer-events: auto; display: flex; align-items: center; gap: 10px; width: fit-content; max-width: 100%; padding: 11px 12px 11px 15px; border-radius: 12px; background: var(--ink); color: #eaf6f2; box-shadow: var(--shadow-2); font-weight: 600; font-size: 13.5px; line-height: 1.5; }
.toast > svg { color: var(--mint); }
.toast.warn > svg { color: #f2c46d; }
.close { border: none; background: none; color: #93a8a8; padding: 4px; border-radius: 6px; cursor: pointer; display: grid; place-items: center; }
.close:hover { color: #fff; background: rgba(255, 255, 255, 0.1); }
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.22s cubic-bezier(0.2, 0.7, 0.2, 1); }
.toast-enter-from { opacity: 0; transform: translateY(12px) scale(0.98); }
.toast-leave-to { opacity: 0; transform: translateY(6px); }
</style>
