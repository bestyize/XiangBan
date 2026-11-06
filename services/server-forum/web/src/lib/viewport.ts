import { onBeforeUnmount, ref } from 'vue'

/** 手机宽度（<768px）：评论改用底部栏 + 弹出面板，桌面保持页内输入框。 */
export function useNarrow(query = '(max-width: 767px)') {
  const media = window.matchMedia(query)
  const narrow = ref(media.matches)
  const sync = () => { narrow.value = media.matches }
  media.addEventListener('change', sync)
  onBeforeUnmount(() => media.removeEventListener('change', sync))
  return narrow
}
