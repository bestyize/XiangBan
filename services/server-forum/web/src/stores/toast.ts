import { ref } from 'vue'
import { defineStore } from 'pinia'

export interface Toast {
  id: number
  text: string
  tone: 'info' | 'error'
  action?: { label: string; to: string }
}

export const useToast = defineStore('toast', () => {
  const items = ref<Toast[]>([])
  let seq = 0

  function show(text: string, tone: Toast['tone'] = 'info', action?: Toast['action']) {
    const id = ++seq
    items.value.push({ id, text, tone, action })
    setTimeout(() => {
      items.value = items.value.filter(t => t.id !== id)
    }, action ? 8000 : 3200)
  }

  function error(e: unknown) {
    show(e instanceof Error ? e.message : '操作失败', 'error')
  }

  return { items, show, error }
})
