<script setup lang="ts">
// 管理员登录：成功后令牌写入 localStorage，AdminView 随之切换到后台界面。
import { ref } from 'vue'
import { adminApi, adminAuth, setAdminToken } from '@/api/admin'
import { errorText } from './shared'

const username = ref('')
const password = ref('')
const busy = ref(false)
const error = ref('')

async function submit() {
  if (busy.value) return
  error.value = ''
  if (!username.value.trim() || !password.value) {
    error.value = '请输入用户名和密码'
    return
  }
  busy.value = true
  try {
    const res = await adminApi.login(username.value.trim(), password.value)
    adminAuth.username = res.admin.username
    setAdminToken(res.token)
    password.value = ''
  } catch (e) {
    error.value = errorText(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="adm-login">
    <div class="panel adm-login-box">
      <h1>
        <span class="adm-brand-mark" aria-hidden="true" />
        超级小伴
      </h1>
      <p>管理后台</p>
      <form @submit.prevent="submit">
        <label class="field">
          <span>用户名</span>
          <input v-model="username" class="input" autocomplete="username" autocapitalize="none" autofocus />
        </label>
        <label class="field">
          <span>密码</span>
          <input v-model="password" class="input" type="password" autocomplete="current-password" />
        </label>
        <p v-if="error" class="error-text" role="alert" style="margin: 0">{{ error }}</p>
        <button class="btn btn-primary" type="submit" :disabled="busy">{{ busy ? '登录中' : '登录' }}</button>
      </form>
    </div>
  </div>
</template>
