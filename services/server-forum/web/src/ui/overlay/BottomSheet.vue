<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import UiIcon from '../icons/UiIcon.vue'

/**
 * 统一底部弹层：仅顶角 20px 圆角、最高 80dvh、内置页面滚动锁、安全区与 visualViewport 软键盘避让。
 * 入场 260ms 标准减速（无过冲）。
 */
const props = withDefaults(
  defineProps<{
    open: boolean
    title?: string
    subtitle?: string
    label?: string
    showHandle?: boolean
    flush?: boolean
    hideClose?: boolean
    divider?: boolean
    cancelLabel?: string
    confirmLabel?: string
  }>(),
  { showHandle: true, flush: false, hideClose: false, divider: false },
)
const emit = defineEmits<{ close: []; confirm: [] }>()
const imeInset = ref(0)

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

watch(
  () => props.open,
  (on, _, cleanup) => {
    if (!on) return
    const root = document.documentElement
    const prev = root.style.overflow
    root.style.overflow = 'hidden'
    window.addEventListener('keydown', onKeydown)

    const viewport = window.visualViewport
    const sync = () => {
      if (!viewport) return
      imeInset.value = Math.max(0, Math.round(window.innerHeight - viewport.height - viewport.offsetTop))
    }
    sync()
    viewport?.addEventListener('resize', sync)
    viewport?.addEventListener('scroll', sync)

    cleanup(() => {
      root.style.overflow = prev
      window.removeEventListener('keydown', onKeydown)
      viewport?.removeEventListener('resize', sync)
      viewport?.removeEventListener('scroll', sync)
      imeInset.value = 0
    })
  },
  { immediate: true },
)

onBeforeUnmount(() => { imeInset.value = 0 })
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="ui-sheet-root" :style="{ '--ui-ime': `${imeInset}px` }">
      <div class="ui-sheet-scrim" @click="emit('close')" />
      <div class="ui-sheet-panel" role="dialog" aria-modal="true" :aria-label="label || title || '底部弹层'">
        <div v-if="showHandle" class="ui-sheet-handle" aria-hidden="true" />
        <header v-if="title || $slots.header" class="ui-sheet-head" :class="{ 'has-divider': divider }">
          <slot name="header">
            <div class="ui-sheet-titles">
              <h3>{{ title }}</h3>
              <p v-if="subtitle">{{ subtitle }}</p>
            </div>
            <button v-if="!hideClose && !cancelLabel && !confirmLabel" type="button" class="ui-sheet-close" aria-label="关闭" @click="emit('close')">
              <UiIcon name="close" :size="15" :stroke-width="2.1" />
            </button>
          </slot>
        </header>
        <div class="ui-sheet-body" :class="{ 'is-padded': !flush }">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="ui-sheet-footer">
          <slot name="footer" />
        </footer>
        <div v-else-if="cancelLabel && confirmLabel" class="ui-sheet-actions-2">
          <button type="button" class="is-cancel" @click="emit('close')">{{ cancelLabel }}</button>
          <button type="button" class="is-confirm" @click="emit('confirm')">{{ confirmLabel }}</button>
        </div>
        <button v-else-if="cancelLabel" type="button" class="ui-sheet-cancel-row" @click="emit('close')">
          {{ cancelLabel }}
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.ui-sheet-root {
  position: fixed;
  inset: 0;
  z-index: var(--z-sheet);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding-bottom: var(--ui-ime, 0px);
}
.ui-sheet-scrim {
  position: fixed;
  inset: 0;
  background: var(--ui-scrim);
  animation: ui-fade var(--dur-normal) var(--ease-ui);
}
.ui-sheet-panel {
  position: relative;
  z-index: 1;
  width: 100%;
  max-height: min(80dvh, calc(100dvh - var(--ui-ime, 0px) - 24px));
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface);
  color: var(--ink);
  border-radius: var(--r-sheet) var(--r-sheet) 0 0;
  box-shadow: var(--ui-shadow-sheet);
  padding-bottom: env(safe-area-inset-bottom);
  animation: ui-sheet-up var(--dur-sheet-in) var(--ease-ui);
}
@keyframes ui-fade { from { opacity: 0; } }
@keyframes ui-sheet-up { from { transform: translateY(100%); } }

.ui-sheet-handle {
  padding: 10px 0 4px;
  display: flex;
  justify-content: center;
  flex: none;
}
.ui-sheet-handle::before {
  content: '';
  width: 36px;
  height: 4px;
  border-radius: 4px;
  background: var(--line-strong);
}
.ui-sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px 12px;
  flex: none;
}
.ui-sheet-head.has-divider { border-bottom: 1px solid var(--line); }
.ui-sheet-titles h3 { margin: 0; font-size: var(--fs-section); line-height: 22px; font-weight: 650; }
.ui-sheet-titles p { margin: 2px 0 0; font-size: var(--fs-caption); color: var(--ink-2); }
.ui-sheet-close {
  width: 36px;
  height: 36px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--ink-2);
  display: grid;
  place-items: center;
  flex: none;
  transition: background-color var(--dur-fast) var(--ease-ui), color var(--dur-fast) var(--ease-ui);
}
.ui-sheet-close:hover { background: var(--surface-2); color: var(--ink); }
.ui-sheet-close svg { width: 15px; height: 15px; stroke: currentColor; stroke-width: 2.1; stroke-linecap: round; }
.ui-sheet-body { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; }
.ui-sheet-body.is-padded { padding: 4px 16px 20px; }
.ui-sheet-footer {
  padding: 12px 16px 16px;
  border-top: 1px solid var(--line);
  display: flex;
  gap: 10px;
  flex: none;
}
.ui-sheet-cancel-row {
  position: relative;
  width: 100%;
  min-height: 54px;
  padding: 12px 16px 16px;
  border: 0;
  background: transparent;
  color: var(--ink-2);
  font-size: var(--fs-body);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
  transition: background-color var(--dur-fast) var(--ease-ui), color var(--dur-fast) var(--ease-ui);
}
.ui-sheet-cancel-row::before {
  content: '';
  position: absolute;
  top: 0;
  left: 16px;
  right: 16px;
  height: 0.5px;
  background: var(--line);
}
.ui-sheet-cancel-row:hover {
  background: var(--surface-2);
  color: var(--ink);
}
.ui-sheet-actions-2 {
  position: relative;
  display: grid;
  grid-template-columns: 1fr 1fr;
  flex: none;
}
.ui-sheet-actions-2::before {
  content: '';
  position: absolute;
  top: 0;
  left: 16px;
  right: 16px;
  height: 0.5px;
  background: var(--line);
}
.ui-sheet-actions-2 button {
  position: relative;
  min-height: 54px;
  padding: 12px 16px 16px;
  border: 0;
  background: transparent;
  font-size: var(--fs-body);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color var(--dur-fast) var(--ease-ui), color var(--dur-fast) var(--ease-ui);
}
.ui-sheet-actions-2 button:hover { background: var(--surface-2); }
.ui-sheet-actions-2 .is-cancel { color: var(--ink-2); font-weight: 500; }
.ui-sheet-actions-2 .is-confirm { color: var(--ink); font-weight: 650; }
.ui-sheet-actions-2 .is-confirm::before {
  content: '';
  position: absolute;
  left: 0;
  top: 14px;
  bottom: 18px;
  width: 0.5px;
  background: var(--line);
}
</style>
