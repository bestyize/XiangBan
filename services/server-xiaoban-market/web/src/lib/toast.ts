import { reactive } from 'vue';

export const toasts = reactive<{ id: number; text: string; tone: 'ok' | 'bad' }[]>([]);
let next = 1;

export function toast(text: string, tone: 'ok' | 'bad' = 'ok', ms = 4000) {
  const id = next++;
  toasts.push({ id, text, tone });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, tone === 'bad' ? Math.max(ms, 7000) : ms);
}

export const toastError = (e: unknown) => toast((e as Error).message || '操作失败', 'bad');
