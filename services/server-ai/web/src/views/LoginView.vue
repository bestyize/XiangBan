<script setup lang="ts">
import { ref } from 'vue';
import { api, errorMessage } from '../api';
import { signIn } from '../session';
import BrandMark from '../components/BrandMark.vue';
import Icon from '../components/Icon.vue';

const emit = defineEmits<{ entered: [] }>();

const username = ref('');
const password = ref('');
const reveal = ref(false);
const busy = ref(false);
const error = ref('');

async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.login(username.value, password.value);
    emit('entered');
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
    <section class="stage" aria-hidden="true">
      <div class="brand"><BrandMark :size="36" /><span>相伴 AI</span></div>
      <div class="pitch">
        <p class="big">用户看到档位，</p>
        <p class="big dim">你决定背后用哪个模型。</p>
      </div>
    </section>

    <section class="panel-side">
      <form class="form" @submit.prevent="submit">
        <div class="mark-m"><BrandMark :size="40" /></div>
        <div>
          <h1>登录控制台</h1>
          <p class="muted">管理产品档位、用户额度和兑换码。</p>
        </div>

        <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

        <div class="field">
          <label for="login-user">管理员用户名</label>
          <input id="login-user" v-model="username" class="input" autocomplete="username" required autofocus />
        </div>
        <div class="field">
          <label for="login-pass">密码</label>
          <div class="pw">
            <input id="login-pass" v-model="password" class="input" :type="reveal ? 'text' : 'password'" autocomplete="current-password" required />
            <button type="button" class="eye" :aria-label="reveal ? '隐藏密码' : '显示密码'" @click="reveal = !reveal"><Icon :name="reveal ? 'eye-off' : 'eye'" :size="17" /></button>
          </div>
        </div>
        <button class="btn primary go" type="submit" :disabled="busy || !username || !password">{{ busy ? '登录中…' : '登录' }}</button>
      </form>
    </section>
  </div>
</template>

<style scoped>
.login { min-height: 100vh; display: grid; grid-template-columns: minmax(0, 1.08fr) minmax(380px, 1fr); }
.stage { position: relative; background: var(--curtain-dark); color: #e6f5f1; padding: 44px 56px; display: flex; flex-direction: column; justify-content: space-between; overflow: hidden; }
.stage::after { content: ''; position: absolute; inset: 0; background: radial-gradient(120% 70% at 30% 100%, rgba(111, 216, 194, 0.14), transparent 60%), linear-gradient(180deg, rgba(13, 24, 29, 0.55), transparent 28%); pointer-events: none; }
.brand { position: relative; z-index: 1; display: flex; align-items: center; gap: 12px; font-weight: 700; font-size: 17px; letter-spacing: -0.005em; }
.pitch { position: relative; z-index: 1; }
.big { font-size: clamp(26px, 2.9vw, 38px); line-height: 1.45; font-weight: 700; letter-spacing: -0.02em; text-wrap: balance; }
.big.dim { color: var(--mint); }

.panel-side { display: grid; place-items: center; padding: 40px 24px; }
.form { width: min(372px, 100%); display: flex; flex-direction: column; gap: 18px; }
.mark-m { display: none; }
.form h1 { font-size: 26px; }
.form .muted { margin-top: 6px; }
.pw { position: relative; }
.pw .input { padding-right: 44px; }
.eye { position: absolute; right: 5px; top: 50%; transform: translateY(-50%); width: 32px; height: 32px; display: grid; place-items: center; border: none; background: none; color: var(--faint); border-radius: 8px; cursor: pointer; }
.eye:hover { color: var(--text); background: rgba(16, 28, 34, 0.06); }
.go { min-height: 42px; margin-top: 2px; font-size: 14.5px; }

@media (max-width: 820px) {
  .login { grid-template-columns: 1fr; }
  .stage { display: none; }
  .mark-m { display: block; }
  .panel-side { align-items: start; padding-top: 64px; }
}
</style>
