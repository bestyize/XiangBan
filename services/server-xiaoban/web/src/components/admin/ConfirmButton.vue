<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

const props = defineProps<{ label: string; confirmLabel?: string; busy?: boolean }>()
const emit = defineEmits<{ confirm: [] }>()

const armed = ref(false)
let timer: ReturnType<typeof setTimeout> | undefined

function press() {
  if (armed.value) {
    clearTimeout(timer)
    armed.value = false
    emit('confirm')
    return
  }
  armed.value = true
  timer = setTimeout(() => (armed.value = false), 3000)
}

onBeforeUnmount(() => clearTimeout(timer))
</script>

<template>
  <button
    class="is-sm"
    :class="armed ? 'xb-btn-soft is-danger' : 'xb-btn-ghost is-danger'"
    type="button"
    :disabled="busy"
    @click="press"
  >
    {{ armed ? (props.confirmLabel ?? '再点一次确认') : label }}
  </button>
</template>
