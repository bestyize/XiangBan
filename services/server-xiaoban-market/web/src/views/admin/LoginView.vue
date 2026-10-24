<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { accountPasswordLogin, admin, setSession, type LoginConfig } from '@/lib/api';

const route = useRoute();
const router = useRouter();
const mode = ref<'local' | 'account'>('local');
const config = ref<LoginConfig['account'] | null>(null);
const form = ref({ username: '', password: '', identifier: '', accountPassword: '' });
const error = ref('');
const busy = ref(false);

onMounted(async () => {
  try {
    const c = await admin.config();
    config.value = c.account.enabled ? c.account : null;
  } catch {
    config.value = null; // 读不到配置时只提供本地登录
  }
});

function done(token: string, username: string) {
  setSession(token, username);
  const redirect = String(route.query.redirect ?? '');
  router.replace(redirect.startsWith('/admin') ? redirect : { name: 'admin-dashboard' });
}

async function submit() {
  busy.value = true;
  error.value = '';
  try {
    if (mode.value === 'local') {
      const r = await admin.login(form.value.username.trim(), form.value.password);
      done(r.token, r.username);
    } else {
      const c = config.value!;
      const accessToken = await accountPasswordLogin(c.url, c.clientId, form.value.identifier, form.value.accountPassword);
      const r = await admin.loginWithAccount(accessToken);
      done(r.token, r.username);
    }
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

      <div v-if="config" class="seg-tabs" role="group" aria-label="登录方式">
        <button type="button" :class="{ on: mode === 'account' }" @click="mode = 'account'; error = ''">相伴账号</button>
        <button type="button" :class="{ on: mode === 'local' }" @click="mode = 'local'; error = ''">本地管理员</button>
      </div>

      <template v-if="mode === 'local'">
        <div class="field"><label for="u">用户名</label><input id="u" v-model="form.username" class="input" autocomplete="username" required autofocus /></div>
        <div class="field"><label for="p">密码</label><input id="p" v-model="form.password" class="input" type="password" autocomplete="current-password" required /></div>
      </template>
      <template v-else>
        <div class="field"><label for="ai">账号 / 邮箱 / 手机号</label><input id="ai" v-model="form.identifier" class="input" autocomplete="username" required autofocus /></div>
        <div class="field"><label for="ap">账号密码</label><input id="ap" v-model="form.accountPassword" class="input" type="password" autocomplete="current-password" required /></div>
        <p class="hint">需要先由本地管理员在「设置」里把这个账号绑定为后台管理员。密码只发给账号中心。</p>
      </template>

      <div v-if="error" class="error-box" role="alert">{{ error }}</div>
      <button class="btn btn-primary" type="submit" :disabled="busy || (mode === 'local' ? !form.username || !form.password : !form.identifier || !form.accountPassword)">{{ busy ? '登录中…' : '登录' }}</button>
    </form>
  </main>
</template>

<style scoped>
.login { min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 24px; background: var(--soft); }
.card { width: 100%; max-width: 380px; display: flex; flex-direction: column; gap: 16px; padding: 32px; background: #fff; border: 1px solid var(--line); border-radius: 20px; box-shadow: var(--shadow-card); }
.brand { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; }
h1 { font-size: 20px; font-weight: 700; }
.seg-tabs { display: flex; }
.seg-tabs button { flex: 1; }
</style>
