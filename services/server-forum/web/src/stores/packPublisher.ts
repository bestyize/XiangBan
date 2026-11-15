import { computed, ref, shallowRef, watch } from 'vue'
import { defineStore } from 'pinia'
import { track } from '@/lib/analytics'
import { useSession } from './session'

export const usePackPublisher = defineStore('packPublisher', () => {
  const session = useSession()
  const target = shallowRef<{ packId?: number } | null>(null)
  const owner = ref<string | null>(null)
  const saved = shallowRef<{ id: number } | null>(null)
  const visible = computed(() => !!target.value && session.loggedIn && !session.loginOpen)
  const ready = computed(() => visible.value && !!owner.value && session.user?.uid === owner.value)

  function open(packId?: number) {
    if (target.value) return
    owner.value = session.user?.uid ?? null
    target.value = { packId }
    track('publish_start')
    session.requireLogin()
  }

  function close() {
    target.value = null
    owner.value = null
  }

  function complete(id: number) {
    saved.value = { id }
    close()
  }

  watch(() => session.user?.uid, uid => {
    if (!target.value || !uid) return
    if (owner.value && uid !== owner.value) close()
    else owner.value = uid
  }, { flush: 'sync' })
  watch(() => session.loggedIn, loggedIn => {
    if (!loggedIn && target.value) session.requireLogin()
  })
  watch(() => session.loginOpen, open => {
    if (!open && !session.loggedIn) close()
  })

  return { target, visible, ready, saved, open, close, complete }
})
