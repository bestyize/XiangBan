import { ref } from 'vue'

export const toastMessage = ref<string | null>(null)

let timer: ReturnType<typeof setTimeout> | undefined

export function toast(message: string, ms = 2200) {
  toastMessage.value = message
  clearTimeout(timer)
  timer = setTimeout(() => (toastMessage.value = null), ms)
}
