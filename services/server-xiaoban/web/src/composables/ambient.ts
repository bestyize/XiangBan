import { ref } from 'vue'

/** Page glow color. Set from whichever wallpaper the user is looking at; the CSS transition does the easing. */
export const ambientColor = ref<string | null>(null)

let timer: ReturnType<typeof setTimeout> | undefined

export function setAmbient(color: string | null, delay = 0) {
  clearTimeout(timer)
  if (delay > 0) timer = setTimeout(() => (ambientColor.value = color), delay)
  else ambientColor.value = color
}
