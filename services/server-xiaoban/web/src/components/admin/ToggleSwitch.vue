<script setup lang="ts">
defineProps<{ label: string; disabled?: boolean }>()
const checked = defineModel<boolean>({ required: true })
</script>

<template>
  <button
    class="switch"
    type="button"
    role="switch"
    :aria-checked="checked"
    :aria-label="label"
    :title="label"
    :disabled="disabled"
    @click="checked = !checked"
  >
    <span class="knob" />
  </button>
</template>

<style scoped>
.switch {
  position: relative;
  flex: none;
  width: 40px;
  height: 24px;
  border-radius: 999px;
  border: 1px solid var(--line-strong);
  background: var(--surface-2);
  transition:
    background 0.2s,
    border-color 0.2s;
}

.switch[aria-checked='true'] {
  background: var(--text);
  border-color: var(--text);
}

.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--text-2);
  transition:
    transform 0.25s var(--ease),
    background 0.2s;
}

.switch[aria-checked='true'] .knob {
  background: var(--bg);
  transform: translateX(16px);
}

.switch:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
