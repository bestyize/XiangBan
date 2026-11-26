<script setup lang="ts">
const model = defineModel<boolean>({ required: true });

withDefaults(
  defineProps<{
    title: string;
    desc?: string;
    disabled?: boolean;
  }>(),
  { disabled: false },
);

function toggle() {
  model.value = !model.value;
}
</script>

<template>
  <button
    type="button"
    class="switch-row"
    role="switch"
    :aria-checked="model"
    :disabled="disabled"
    @click="toggle"
  >
    <span class="switch-txt">
      <strong>{{ title }}</strong>
      <span v-if="desc">{{ desc }}</span>
    </span>
    <span class="switch-track"><i></i></span>
  </button>
</template>

<style scoped>
.switch-row {
  width: 100%;
  padding: 11px 14px;
  text-align: left;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  border-bottom: 1px solid var(--line);
  transition: background 0.12s;
}
.switch-row:last-child { border-bottom: 0; }
.switch-row:hover:not(:disabled) { background: var(--surface-hover); }
.switch-row:disabled { opacity: 0.52; cursor: not-allowed; }
.switch-txt { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.switch-txt strong { font-size: 13px; font-weight: 600; color: var(--ink); }
.switch-txt span { font-size: 11.5px; color: var(--ink-3); line-height: 1.45; }
.switch-track {
  width: 36px;
  height: 20px;
  border-radius: 99px;
  flex: none;
  padding: 2px;
  background: var(--line-strong);
  transition: background 0.18s var(--ease);
  display: flex;
  align-items: center;
}
.switch-track i {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.22);
  transition: transform 0.18s var(--ease);
}
.switch-row[aria-checked='true'] .switch-track { background: var(--btn-primary-bg); }
:global([data-theme='dark']) .switch-row[aria-checked='true'] .switch-track { background: var(--coral); }
.switch-row[aria-checked='true'] .switch-track i { transform: translateX(16px); }
</style>
