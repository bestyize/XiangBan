import { onBeforeUnmount, ref } from 'vue'

/** 与 CSS 的窄屏断点保持一致。 */
export function useNarrow(query = '(max-width: 767px)') {
  const media = window.matchMedia(query)
  const narrow = ref(media.matches)
  const sync = () => { narrow.value = media.matches }
  media.addEventListener('change', sync)
  onBeforeUnmount(() => media.removeEventListener('change', sync))
  return narrow
}
