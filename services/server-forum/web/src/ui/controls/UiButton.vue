<script setup lang="ts">
/**
 * 通用胶囊按钮（不使用同色系浅底深字搭配）：
 * primary（主题色实心+白字）、outline（白底细描边+深字）、secondary（中性浅灰底+深字）、
 * ghost（透明弱操作）、danger（固定危险红实心+白字）、danger-outline（白底细描边+固定危险红字）。
 */
withDefaults(
  defineProps<{
    variant?: 'primary' | 'outline' | 'secondary' | 'ghost' | 'danger' | 'danger-outline'
    size?: 'sm' | 'md' | 'lg'
    block?: boolean
    disabled?: boolean
    loading?: boolean
    type?: 'button' | 'submit' | 'reset'
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    block: false,
    disabled: false,
    loading: false,
    type: 'button',
  },
)
</script>

<template>
  <button
    :type="type"
    class="ui-btn"
    :class="[`is-${variant}`, `is-${size}`, { 'is-block': block }]"
    :disabled="disabled || loading"
  >
    <span v-if="loading" class="ui-btn-spin" aria-hidden="true" />
    <slot />
  </button>
</template>

<style scoped>
.ui-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px solid transparent;
  border-radius: var(--r-pill);
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
  transition: background-color var(--dur-fast) var(--ease-ui), border-color var(--dur-fast) var(--ease-ui), filter var(--dur-fast) var(--ease-ui), opacity var(--dur-fast) var(--ease-ui);
}
.ui-btn.is-lg { min-height: var(--h-btn-lg); padding: 0 20px; font-size: var(--fs-body); }
.ui-btn.is-md { min-height: var(--h-btn-md); padding: 0 16px; font-size: 14px; }
.ui-btn.is-sm { min-height: var(--h-btn-sm); padding: 0 14px; font-size: var(--fs-secondary); }
.ui-btn.is-block { width: 100%; }

.ui-btn.is-primary { background: var(--brand-fill); color: var(--brand-ink); }
.ui-btn.is-primary:hover:not(:disabled) { filter: brightness(0.94); }

.ui-btn.is-outline { background: var(--surface); color: var(--ink); border-color: var(--line-strong); font-weight: 550; }
.ui-btn.is-outline:hover:not(:disabled) { background: var(--surface-2); }

.ui-btn.is-secondary { background: var(--surface-2); color: var(--ink); font-weight: 550; }
.ui-btn.is-secondary:hover:not(:disabled) { filter: brightness(0.96); }

.ui-btn.is-ghost { background: transparent; color: var(--ink-2); font-weight: 500; }
.ui-btn.is-ghost:hover:not(:disabled) { background: var(--surface-2); }

.ui-btn.is-danger { background: var(--ui-danger); color: #fff; }
.ui-btn.is-danger:hover:not(:disabled) { filter: brightness(0.94); }

.ui-btn.is-danger-outline { background: var(--surface); color: var(--ui-danger); border-color: var(--line); }
.ui-btn.is-danger-outline:hover:not(:disabled) { background: var(--surface-2); }

.ui-btn:disabled { opacity: 0.48; cursor: not-allowed; }
.ui-btn-spin {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid currentColor;
  border-top-color: transparent;
  animation: ui-spin 0.7s linear infinite;
}
@keyframes ui-spin { to { transform: rotate(360deg); } }
</style>
