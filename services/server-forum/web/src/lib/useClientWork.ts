import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { clientWorkLink, workClientPlatform, type ClientWork } from './clientWork'
import { inDesktopCommunity } from '@/stores/session'

export function useClientWork(target: () => ClientWork | undefined) {
  const platform = inDesktopCommunity ? undefined : workClientPlatform(navigator.userAgent, navigator.platform, navigator.maxTouchPoints)
  const mobile = platform === 'android'
  const label = mobile ? '在小伴手机版打开' : '在小伴桌面打开'
  const icon = mobile ? 'client-phone' : 'client-monitor'
  const download = 'https://wallpaper.xiangban.xyz/download#desktop-downloads'
  const href = computed(() => platform ? clientWorkLink(target()) : undefined)
  const help = ref(false)
  let trigger: HTMLElement | null = null
  function dismiss() { help.value = false; trigger = null }
  function closeHint() {
    trigger?.focus({ preventScroll: true })
    dismiss()
  }
  function hideWhenAway() { if (document.hidden) dismiss() }
  watch(href, dismiss)
  onMounted(() => document.addEventListener('visibilitychange', hideWhenAway))
  onBeforeUnmount(() => document.removeEventListener('visibilitychange', hideWhenAway))
  function open() {
    if (!href.value) return
    trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null
    help.value = true
    // Keep the current work and browser history in place if the app is absent or the prompt is cancelled.
    try { window.location.assign(href.value) } catch { /* The download link stays available. */ }
  }
  return { href, mobile, label, icon, download, help, open, dismiss, closeHint }
}
