import { computed, ref } from 'vue'

interface BeforeInstallPromptEvent extends Event {
  prompt(): Promise<void>
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>
}

// The browser fires this once, early, so it is captured at startup rather than when Settings opens.
const deferred = ref<BeforeInstallPromptEvent | null>(null)

window.addEventListener('beforeinstallprompt', (e) => {
  e.preventDefault()
  deferred.value = e as BeforeInstallPromptEvent
})

window.addEventListener('appinstalled', () => {
  deferred.value = null
})

export const isStandalone =
  window.matchMedia('(display-mode: standalone)').matches ||
  (navigator as Navigator & { standalone?: boolean }).standalone === true

export const isIOS = /iphone|ipad|ipod/i.test(navigator.userAgent)

export const canInstall = computed(() => deferred.value !== null)

export async function promptInstall(): Promise<boolean> {
  const event = deferred.value
  if (!event) return false
  await event.prompt()
  const { outcome } = await event.userChoice
  deferred.value = null
  return outcome === 'accepted'
}
