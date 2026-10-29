import { ref } from 'vue'
import { defineStore } from 'pinia'
import { ApiError } from '@/api'
import * as adminApi from '@/admin/api'
import type { AdminUser } from '@/admin/types'

export const useAdmin = defineStore('admin', () => {
  const user = ref<AdminUser | null>(null)
  const checked = ref(false)
  /** Set when the server has no PostgreSQL, in which case the console cannot work at all. */
  const unavailable = ref<string | null>(null)

  async function refresh() {
    try {
      user.value = await adminApi.me()
      unavailable.value = null
    } catch (e) {
      user.value = null
      unavailable.value = e instanceof ApiError && e.status === 503 ? e.message : null
    } finally {
      checked.value = true
    }
  }

  async function login(username: string, password: string) {
    user.value = await adminApi.login(username, password)
    unavailable.value = null
    checked.value = true
  }

  async function logout() {
    try {
      await adminApi.logout()
    } finally {
      user.value = null
    }
  }

  function clear() {
    user.value = null
  }

  return { user, checked, unavailable, refresh, login, logout, clear }
})
