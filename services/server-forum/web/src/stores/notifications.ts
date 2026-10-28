import { ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/api/forum'
import { useSession } from './session'

/** 未读通知数：登录后每分钟刷新一次，页面回到前台时也刷新；失败就保持原数，不打扰用户。 */
export const useNotifications = defineStore('notifications', () => {
  const unread = ref(0)
  const session = useSession()
  let timer: number | undefined

  async function refresh() {
    if (!session.loggedIn) {
      unread.value = 0
      return
    }
    try {
      unread.value = (await api.unreadCount()).unread
    } catch {
      // 网络波动时保持上次的数字
    }
  }

  function start() {
    stop()
    void refresh()
    timer = window.setInterval(refresh, 60_000)
    document.addEventListener('visibilitychange', onVisible)
  }
  function stop() {
    if (timer !== undefined) window.clearInterval(timer)
    timer = undefined
    document.removeEventListener('visibilitychange', onVisible)
  }
  function onVisible() {
    if (document.visibilityState === 'visible') void refresh()
  }

  watch(() => session.loggedIn, on => (on ? start() : (stop(), (unread.value = 0))), { immediate: true })

  return { unread, refresh }
})
