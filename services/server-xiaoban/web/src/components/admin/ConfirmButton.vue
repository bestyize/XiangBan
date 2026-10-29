<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

const props = defineProps<{ label: string; confirmLabel?: string; busy?: boolean }>()
const emit = defineEmits<{ confirm: [] }>()

const armed = ref(false)
let timer: ReturnType<typeof setTimeout> | undefined

// The first press only arms the button; a second press within three seconds does the destructive thing.
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
  <button class="a-btn small" :class="{ danger: armed }" type="button" :disabled="busy" @click="press">
    {{ armed ? (props.confirmLabel ?? '再点一次确认') : label }}
  </button>
</template>
