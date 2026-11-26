<script setup lang="ts">
defineProps<{ label: string; disabled?: boolean }>()
const checked = defineModel<boolean>({ required: true })
</script>

<template>
  <button
    class="xb-switch"
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
.xb-switch {
  position: relative;
  flex: none;
  width: 36px;
  height: 20px;
  border-radius: 999px;
  border: 0;
  background: var(--ui-control-hover);
  transition: background 0.18s;
}
.xb-switch[aria-checked='true'] {
  background: var(--ui-btn);
}
.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.18);
  transition: transform 0.2s var(--ui-ease);
}
.xb-switch[aria-checked='true'] .knob {
  transform: translateX(16px);
}
.xb-switch:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
