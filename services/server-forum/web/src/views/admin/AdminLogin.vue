<script setup lang="ts">
// 管理员登录：成功后令牌写入 localStorage，AdminView 随之切换到后台界面。
import { ref } from 'vue'
import { adminApi, adminAuth, setAdminToken } from '@/api/admin'
import BrandMark from '@/components/BrandMark.vue'
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
    <div class="adm-login-box">
      <div class="adm-login-brand">
        <span class="adm-brand-logo"><BrandMark /></span>
        <div>
          <h1>超级小伴 · 运营管理台</h1>
          <p>chat.xiangban.xyz 独立管理员通道</p>
        </div>
      </div>

      <form @submit.prevent="submit">
        <label class="field">
          <span>管理员账号</span>
          <input v-model="username" class="input" autocomplete="username" autocapitalize="none" placeholder="请输入用户名" autofocus />
        </label>
        <label class="field">
          <span>登录密码</span>
          <input v-model="password" class="input" type="password" autocomplete="current-password" placeholder="请输入密码" />
        </label>
        <div class="adm-login-meta">
          <span>独立会话 · 操作全程审计</span>
          <span>连续输错 5 次锁 15 分钟</span>
        </div>
        <p v-if="error" class="error-text" role="alert" style="margin: 0">{{ error }}</p>
        <button class="btn btn-primary" type="submit" :disabled="busy">{{ busy ? '登录中…' : '进入管理后台' }}</button>
      </form>

      <div class="adm-login-foot">
        <RouterLink to="/">← 返回社区前台</RouterLink>
        <span>超级小伴社区</span>
      </div>
    </div>
  </div>
</template>
