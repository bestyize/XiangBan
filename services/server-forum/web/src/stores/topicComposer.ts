import { computed, ref, shallowRef, watch } from 'vue'
import { defineStore } from 'pinia'
import type { PostingSectionKey } from '@/lib/postingStructure'
import { useSession } from './session'

interface ComposerTarget {
  section?: PostingSectionKey
  category?: string
  editId?: number
}

export const useTopicComposer = defineStore('topicComposer', () => {
  const session = useSession()
  const target = shallowRef<ComposerTarget | null>(null)
  const owner = ref<string | null>(null)
  const saved = shallowRef<{ id: number } | null>(null)
  const visible = computed(() => !!target.value && session.loggedIn && !session.loginOpen)
  const ready = computed(() => visible.value && !!owner.value && session.user?.uid === owner.value)

  function open(options: ComposerTarget = {}) {
    owner.value = session.user?.uid ?? null
    target.value = options
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
  })
  watch(() => session.loggedIn, loggedIn => {
    if (!loggedIn && target.value) session.requireLogin()
  })
  watch(() => session.loginOpen, open => {
    if (!open && !session.loggedIn) close()
  })

  return { target, visible, ready, saved, open, close, complete }
})
