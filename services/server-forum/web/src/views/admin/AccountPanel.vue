<script setup lang="ts">
// 账号：修改管理员密码。成功后旧令牌立即失效，用接口返回的新令牌继续使用。
import { reactive, ref } from 'vue'
import { adminApi, adminAuth, setAdminToken } from '@/api/admin'
import { errorText, notify } from './shared'

const form = reactive({ current: '', next: '', again: '' })
const busy = ref(false)
const error = ref('')

async function submit() {
  error.value = ''
  if (form.next.length < 8 || form.next.length > 128) {
    error.value = '新密码长度需为 8-128 位'
    return
  }
  if (form.next !== form.again) {
    error.value = '两次输入的新密码不一致'
    return
  }
  busy.value = true
  try {
    const res = await adminApi.changePassword(form.current, form.next)
    setAdminToken(res.token)
    form.current = form.next = form.again = ''
    notify('密码已修改')
  } catch (e) {
    error.value = errorText(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="panel adm-card adm-account">
    <form class="adm-form" @submit.prevent="submit">
      <p style="margin: 0" class="muted">当前账号：{{ adminAuth.username || '管理员' }}</p>
      <label class="field">
        <span>当前密码</span>
        <input v-model="form.current" class="input" type="password" autocomplete="current-password" required />
      </label>
      <label class="field">
        <span>新密码</span>
        <input v-model="form.next" class="input" type="password" autocomplete="new-password" required minlength="8" maxlength="128" />
        <small>8-128 位。修改后其他设备上的登录会失效。</small>
      </label>
      <label class="field">
        <span>再输入一次新密码</span>
        <input v-model="form.again" class="input" type="password" autocomplete="new-password" required />
      </label>
      <p v-if="error" class="error-text" role="alert" style="margin: 0">{{ error }}</p>
      <div class="adm-form-actions">
        <button class="btn btn-primary" type="submit" :disabled="busy">修改密码</button>
      </div>
    </form>
  </div>
</template>
