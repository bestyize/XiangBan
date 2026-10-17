<script setup lang="ts">
import { ref } from 'vue';
import { api, errorMessage } from '../api';
import { signIn } from '../session';

const username = ref('');
const password = ref('');
const busy = ref(false);
const error = ref('');

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
  <div class="login">
    <div class="curtain" aria-hidden="true">
      <div class="phrase">
        <strong>用户看到档位，</strong>
        <span>你决定背后用哪个模型。</span>
      </div>
    </div>
    <form class="form" @submit.prevent="submit">
      <h1>登录相伴 AI 控制台</h1>
      <p class="muted">使用管理员账号登录，管理产品档位和用户额度。</p>

      <div v-if="error" class="notice error" role="alert">{{ error }}</div>

      <div class="field">
        <label for="login-user">管理员用户名</label>
        <input id="login-user" v-model="username" class="input" autocomplete="username" required autofocus />
      </div>
      <div class="field">
        <label for="login-pass">密码</label>
        <input id="login-pass" v-model="password" class="input" type="password" autocomplete="current-password" required />
      </div>
      <button class="btn primary" type="submit" :disabled="busy || !username || !password">{{ busy ? '登录中…' : '登录' }}</button>
    </form>
  </div>
</template>

<style scoped>
.login { min-height: 100vh; display: grid; grid-template-columns: minmax(0, 1.1fr) minmax(360px, 1fr); }
.curtain { background: var(--curtain); border-right: 1px solid var(--line-strong); display: flex; align-items: flex-end; padding: 56px; }
.phrase { font-size: 26px; line-height: 1.5; font-weight: 400; color: var(--ink); max-width: 14em; }
.phrase strong { display: block; font-weight: 650; }
.phrase span { color: var(--muted); }
.form { align-self: center; justify-self: center; width: min(360px, calc(100vw - 48px)); display: flex; flex-direction: column; gap: 16px; padding: 32px 0; }

@media (max-width: 760px) {
  .login { grid-template-columns: 1fr; grid-template-rows: auto 1fr; }
  .curtain { padding: 28px 24px; border-right: none; border-bottom: 1px solid var(--line-strong); }
  .phrase { font-size: 20px; }
}
</style>
