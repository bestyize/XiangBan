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
    :aria-description="desc"
    :title="desc || title"
    :disabled="disabled"
    @click="toggle"
  >
    <span class="switch-title">{{ title }}</span>
    <span class="switch-track"><i></i></span>
  </button>
</template>

<style scoped>
.switch-row {
  width: 100%;
  min-height: 40px;
  padding: 0 14px;
  text-align: left;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-1);
  border-bottom: 1px solid var(--line);
  transition: background 120ms;
}
.switch-row:last-child { border-bottom: 0; }
.switch-row:hover:not(:disabled) { background: var(--hover); }
.switch-row:disabled { opacity: 0.45; cursor: not-allowed; }
.switch-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.switch-track {
  width: 34px;
  height: 20px;
  border-radius: var(--r-pill);
  flex: none;
  padding: 2px;
  background: var(--line-strong);
  transition: background 180ms var(--ease);
  display: flex;
  align-items: center;
}
.switch-track i {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #ffffff;
  transition: transform 180ms var(--ease);
}
.switch-row[aria-checked='true'] .switch-track { background: var(--brand); }
.switch-row[aria-checked='true'] .switch-track i { transform: translateX(14px); }
</style>
