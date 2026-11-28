import { ref } from 'vue';

export const toastText = ref('');
let timer: ReturnType<typeof setTimeout> | null = null;

export function toast(message: string, duration = 2400) {
  toastText.value = message;
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    toastText.value = '';
    timer = null;
  }, duration);
}
