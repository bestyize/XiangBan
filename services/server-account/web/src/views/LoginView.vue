<script setup lang="ts">
import { ref } from 'vue';
import { api, errorMessage } from '../api';
import { signIn } from '../session';

const username = ref('');
const password = ref('');
const error = ref('');
const busy = ref(false);

async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.login(username.value, password.value);
    signIn(result.token, result.admin.username);
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="wrap">
    <form class="card box" @submit.prevent="submit">
      <h1>相伴账号中心</h1>
      <p class="muted">管理后台登录</p>
      <label class="field">
        <span>用户名</span>
        <input v-model="username" class="input" autocomplete="username" required autofocus />
      </label>
      <label class="field">
        <span>密码</span>
        <input v-model="password" class="input" type="password" autocomplete="current-password" required />
      </label>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <button class="btn primary" type="submit" :disabled="busy">{{ busy ? '登录中…' : '登录' }}</button>
    </form>
  </div>
</template>

<style scoped>
.wrap { min-height: 100vh; display: grid; place-items: center; padding: 20px; }
.box { width: 100%; max-width: 360px; padding: 30px; display: flex; flex-direction: column; gap: 16px; }
h1 { font-size: 22px; }
p { margin: -10px 0 0; }
p.error { margin: 0; }
.btn { height: 38px; }
</style>
