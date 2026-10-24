<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { admin, setSession } from '@/lib/api';

const route = useRoute();
const router = useRouter();
const username = ref('');
const password = ref('');
const error = ref('');
const busy = ref(false);

async function submit() {
  busy.value = true;
  error.value = '';
  try {
    const r = await admin.login(username.value.trim(), password.value);
    setSession(r.token, r.username);
    const redirect = String(route.query.redirect ?? '');
    router.replace(redirect.startsWith('/admin') ? redirect : { name: 'admin-dashboard' });
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <main class="login">
    <form class="card" @submit.prevent="submit">
      <div class="brand">
        <svg width="28" height="28" viewBox="0 0 30 30" aria-hidden="true"><path d="M4 25V7l6 4.5h10L26 7v18a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3z" fill="#141414" /><circle cx="11" cy="18" r="1.7" fill="#fff" /><circle cx="19" cy="18" r="1.7" fill="#fff" /></svg>
        <h1>小伴市场后台</h1>
      </div>
      <div class="field"><label for="u">用户名</label><input id="u" v-model="username" class="input" autocomplete="username" required autofocus /></div>
      <div class="field"><label for="p">密码</label><input id="p" v-model="password" class="input" type="password" autocomplete="current-password" required /></div>
      <div v-if="error" class="error-box" role="alert">{{ error }}</div>
      <button class="btn btn-primary" type="submit" :disabled="busy || !username || !password">{{ busy ? '登录中…' : '登录' }}</button>
    </form>
  </main>
</template>

<style scoped>
.login { min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 24px; background: var(--soft); }
.card { width: 100%; max-width: 380px; display: flex; flex-direction: column; gap: 16px; padding: 32px; background: #fff; border: 1px solid var(--line); border-radius: 20px; box-shadow: var(--shadow-card); }
.brand { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; }
h1 { font-size: 20px; font-weight: 700; }
</style>
