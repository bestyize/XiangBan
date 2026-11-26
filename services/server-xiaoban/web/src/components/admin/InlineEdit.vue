<script setup lang="ts">
import { nextTick, ref } from 'vue'

const props = defineProps<{ value: string; label: string }>()
const emit = defineEmits<{ save: [value: string] }>()

const editing = ref(false)
const draft = ref('')
const input = ref<HTMLInputElement>()

async function start() {
  draft.value = props.value
  editing.value = true
  await nextTick()
  input.value?.select()
}

function finish(save: boolean) {
  if (!editing.value) return
  editing.value = false
  const next = draft.value.trim()
  if (save && next && next !== props.value) emit('save', next)
}
</script>

<template>
  <input
    v-if="editing"
    ref="input"
    v-model="draft"
    class="edit"
    :aria-label="label"
    maxlength="40"
    @keydown.enter.prevent="finish(true)"
    @keydown.esc.prevent="finish(false)"
    @blur="finish(true)"
  />
  <button v-else class="text" type="button" :title="`点击修改${label}`" @click="start">{{ value }}</button>
</template>

<style scoped>
.text {
  padding: 2px 6px;
  margin: -2px -6px;
  border-radius: 6px;
  font-weight: 550;
  color: var(--ui-text-primary);
  text-align: left;
  transition: background 0.14s;
}
.text:hover {
  background: var(--ui-surface-hover);
}
.edit {
  width: 14em;
  max-width: 100%;
  height: 28px;
  padding: 0 8px;
  border-radius: 7px;
  border: 0;
  background: var(--ui-surface);
  color: var(--ui-text-primary);
  box-shadow: 0 0 0 2px var(--ui-btn);
  font-weight: 550;
  outline: none;
}
</style>
