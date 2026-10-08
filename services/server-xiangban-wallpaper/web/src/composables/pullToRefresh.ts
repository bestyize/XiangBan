import { computed, onActivated, onDeactivated, ref } from 'vue'

const THRESHOLD = 72
const MAX_PULL = 110

/** Touch-only pull-down-to-refresh for pages scrolled to the very top. */
export function usePullToRefresh(onRefresh: () => Promise<void> | void) {
  const pull = ref(0)
  const refreshing = ref(false)
  let startY = 0
  let tracking = false

  function onStart(e: TouchEvent) {
    if (refreshing.value || window.scrollY > 0 || e.touches.length !== 1) return
    startY = e.touches[0].clientY
    tracking = true
  }

  function onMove(e: TouchEvent) {
    if (!tracking) return
    const dy = e.touches[0].clientY - startY
    if (dy <= 0 || window.scrollY > 0) {
      pull.value = 0
      return
    }
    pull.value = Math.min(dy * 0.5, MAX_PULL)
  }

  async function onEnd() {
    if (!tracking) return
    tracking = false
    if (pull.value >= THRESHOLD) {
      refreshing.value = true
      pull.value = THRESHOLD
      try {
        await onRefresh()
      } finally {
        refreshing.value = false
        pull.value = 0
      }
    } else {
      pull.value = 0
    }
  }

  // Views are kept alive between tabs, so listeners follow activation rather than mount.
  onActivated(() => {
    window.addEventListener('touchstart', onStart, { passive: true })
    window.addEventListener('touchmove', onMove, { passive: true })
    window.addEventListener('touchend', onEnd)
    window.addEventListener('touchcancel', onEnd)
  })
  onDeactivated(() => {
    window.removeEventListener('touchstart', onStart)
    window.removeEventListener('touchmove', onMove)
    window.removeEventListener('touchend', onEnd)
    window.removeEventListener('touchcancel', onEnd)
    pull.value = 0
    tracking = false
  })

  const ready = computed(() => pull.value >= THRESHOLD)
  return { pull, refreshing, ready }
}
