import { ref } from 'vue'

export interface ConfirmOptions {
  title: string
  description?: string
  quote?: string
  notice?: string
  confirmLabel?: string
  cancelLabel?: string
  danger?: boolean
}

interface PendingConfirm extends ConfirmOptions {
  resolve: (confirmed: boolean) => void
}

const current = ref<PendingConfirm | null>(null)
const queue: PendingConfirm[] = []

function pump() {
  if (current.value || queue.length === 0) return
  current.value = queue.shift() ?? null
}

/**
 * 命令式二次确认弹窗（PC 与移动 H5 均居中展示 24px 卡片，替代浏览器原生 window.confirm）。
 */
export function confirmDialog(options: ConfirmOptions): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    queue.push({ ...options, resolve })
    pump()
  })
}

export function settleConfirm(confirmed: boolean) {
  const active = current.value
  if (!active) return
  current.value = null
  active.resolve(confirmed)
  pump()
}

export function useConfirmState() {
  return current
}
