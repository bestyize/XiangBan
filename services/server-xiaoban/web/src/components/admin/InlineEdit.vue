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
  padding: 0 4px;
  margin: 0 -4px;
  border-radius: 6px;
  font-weight: 500;
  text-align: left;
}

.text:hover {
  background: var(--surface-2);
}

.edit {
  width: 14em;
  max-width: 100%;
  height: 28px;
  padding: 0 8px;
  border-radius: 8px;
  border: 1px solid var(--line-strong);
  background: var(--field, #16161a);
  font-weight: 500;
}
</style>
