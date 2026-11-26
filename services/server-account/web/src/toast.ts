import { reactive } from 'vue';

export type ToastKind = 'ok' | 'error' | 'info';

export interface ToastItem {
  id: number;
  kind: ToastKind;
  text: string;
}

let nextId = 1;
export const toasts = reactive<ToastItem[]>([]);

export function showToast(text: string, kind: ToastKind = 'ok', duration = 2600) {
  const id = nextId++;
  toasts.push({ id, kind, text });
  window.setTimeout(() => {
    const idx = toasts.findIndex((t) => t.id === id);
    if (idx !== -1) toasts.splice(idx, 1);
  }, duration);
}

export async function copyWithToast(text: string, label = '已复制') {
  try {
    await navigator.clipboard.writeText(text);
    showToast(label, 'ok', 1800);
  } catch {
    showToast('复制失败，请手动复制', 'error', 2200);
  }
}
