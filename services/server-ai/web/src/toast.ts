import { reactive } from 'vue';

export interface ToastItem {
  id: number;
  text: string;
  kind: 'ok' | 'warn';
}

export const toasts = reactive<ToastItem[]>([]);
let next = 1;

/** 操作成功这类“知道了就行”的反馈用 Toast，需要用户处理的错误仍然留在原处用提示条。 */
export function notify(text: string, kind: ToastItem['kind'] = 'ok') {
  const id = next++;
  toasts.push({ id, text, kind });
  setTimeout(() => dismiss(id), kind === 'warn' ? 7000 : 3600);
}

export function dismiss(id: number) {
  const index = toasts.findIndex((t) => t.id === id);
  if (index >= 0) toasts.splice(index, 1);
}
