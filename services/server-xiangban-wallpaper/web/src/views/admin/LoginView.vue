<script setup lang="ts">
import '@/styles/admin.css'
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAdmin } from '@/stores/admin'

const route = useRoute()
const router = useRouter()
const admin = useAdmin()

const username = ref('')
const password = ref('')
const busy = ref(false)
const error = ref<string | null>(null)

async function submit() {
  if (busy.value) return
  busy.value = true
  error.value = null
  try {
    await admin.login(username.value.trim(), password.value)
    const target = route.query.redirect
    router.replace(typeof target === 'string' && target.startsWith('/admin/') ? target : '/admin/keys')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '登录失败，请稍后再试'
    password.value = ''
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <main class="admin login">
    <div class="panel">
      <h1 class="display">Wallpapers</h1>
      <p class="sub">管理后台</p>

      <div v-if="admin.unavailable" class="a-banner" role="alert">{{ admin.unavailable }}</div>

      <form v-else class="a-form" @submit.prevent="submit">
        <label class="a-field">
          用户名
          <input v-model="username" class="a-input" name="username" autocomplete="username" autocapitalize="off" required autofocus />
        </label>
        <label class="a-field">
          密码
          <input v-model="password" class="a-input" name="password" type="password" autocomplete="current-password" required />
        </label>
        <p v-if="error" class="a-error" role="alert">{{ error }}</p>
        <button class="a-btn primary" type="submit" :disabled="busy">{{ busy ? '登录中…' : '登录' }}</button>
      </form>

      <RouterLink to="/" class="a-link back">返回前台</RouterLink>
    </div>
  </main>
</template>

<style scoped>
.login {
  display: grid;
  min-height: 100vh;
  min-height: 100dvh;
  place-items: center;
  padding: 24px var(--gutter);
}

.panel {
  display: grid;
  gap: 18px;
  width: 100%;
  max-width: 360px;
}

h1 {
  font-size: 56px;
  line-height: 1;
}

.sub {
  margin-top: -12px;
  color: var(--text-3);
}

.back {
  justify-self: start;
  font-size: 13px;
}
</style>
