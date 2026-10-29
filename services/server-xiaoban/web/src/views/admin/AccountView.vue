<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import * as api from '@/admin/api'
import { useAdmin } from '@/stores/admin'
import { toast } from '@/composables/toast'

const admin = useAdmin()

const form = reactive({ currentPassword: '', username: admin.user?.username ?? '', newPassword: '', confirmPassword: '' })
const saving = ref(false)
const error = ref<string | null>(null)

watch(
  () => admin.user?.username,
  (name) => {
    if (name && !form.username) form.username = name
  },
)

const usernameChanged = computed(() => form.username.trim() !== (admin.user?.username ?? ''))
const passwordChanged = computed(() => form.newPassword !== '')

const mismatch = computed(() => passwordChanged.value && form.confirmPassword !== '' && form.newPassword !== form.confirmPassword)
const canSave = computed(
  () =>
    !saving.value &&
    form.currentPassword !== '' &&
    (usernameChanged.value || passwordChanged.value) &&
    (!passwordChanged.value || form.newPassword === form.confirmPassword),
)

async function save() {
  if (!canSave.value) return
  saving.value = true
  error.value = null
  try {
    const user = await api.updateAccount({
      current_password: form.currentPassword,
      new_username: usernameChanged.value ? form.username.trim() : undefined,
      new_password: passwordChanged.value ? form.newPassword : undefined,
    })
    admin.user = user
    form.username = user.username
    form.currentPassword = ''
    form.newPassword = ''
    form.confirmPassword = ''
    toast(passwordChanged.value ? '已修改，其他设备上的登录已失效' : '用户名已修改')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="a-page">
    <header class="a-head">
      <h1 class="display">账户</h1>
      <p>修改登录后台用的用户名和密码。无论改哪一项，都需要先输入当前密码。</p>
    </header>

    <section class="a-section" aria-label="修改账户">
      <div v-if="admin.user?.is_default_password" class="a-banner" role="alert">
        当前仍是默认密码 123456，请尽快改成自己的密码。
      </div>

      <form class="a-form" @submit.prevent="save">
        <label class="a-field">
          当前密码
          <input v-model="form.currentPassword" class="a-input" type="password" autocomplete="current-password" required />
        </label>

        <label class="a-field">
          用户名
          <input v-model="form.username" class="a-input" autocomplete="username" autocapitalize="off" minlength="3" maxlength="32" required />
          <span class="a-hint">3 到 32 位，只能用字母、数字、下划线、点和短横线。</span>
        </label>

        <label class="a-field">
          新密码
          <input v-model="form.newPassword" class="a-input" type="password" autocomplete="new-password" minlength="8" maxlength="128" placeholder="不修改密码就留空" />
          <span class="a-hint">至少 8 位。修改后，其他设备上的登录会失效。</span>
        </label>

        <label v-if="passwordChanged" class="a-field">
          再输入一次新密码
          <input v-model="form.confirmPassword" class="a-input" type="password" autocomplete="new-password" required />
          <span v-if="mismatch" class="a-error" role="alert">两次输入的密码不一致</span>
        </label>

        <p v-if="error" class="a-error" role="alert">{{ error }}</p>
        <div class="a-actions">
          <button class="a-btn primary" type="submit" :disabled="!canSave">{{ saving ? '保存中…' : '保存修改' }}</button>
        </div>
      </form>
    </section>
  </div>
</template>
