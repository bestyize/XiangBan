<script setup lang="ts">
import { ref } from 'vue';
import { api, errorMessage } from '../api';
import { prefs, toggleTheme } from '../prefs';
import { signIn } from '../session';
import BrandMark from '../components/BrandMark.vue';
import Icon from '../components/Icon.vue';

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
    signIn(result.token, result.admin.username);
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="login-page">
    <button type="button" class="btn icon theme-toggle" :title="prefs.theme === 'dark' ? '切换暖白浅色' : '切换极夜深色'" @click="toggleTheme">
      <Icon :name="prefs.theme === 'dark' ? 'sun' : 'moon'" :size="16" />
    </button>

    <form class="login-card" @submit.prevent="submit">
      <div class="login-head">
        <BrandMark :size="42" />
        <div>
          <h1>登录小伴 AI 网关后台</h1>
          <p>统一管理产品档位路由、用户额度账本、兑换码与上游网关。</p>
        </div>
      </div>

      <div v-if="error" class="notice error" role="alert">
        <Icon name="circle-alert" :size="16" />
        <span>{{ error }}</span>
      </div>

      <div class="field">
        <label for="login-user">管理员用户名</label>
        <input id="login-user" v-model="username" class="input" autocomplete="username" required autofocus />
      </div>

      <div class="field">
        <label for="login-pass">密码</label>
        <div class="pw-row">
          <input
            id="login-pass"
            v-model="password"
            class="input"
            :type="reveal ? 'text' : 'password'"
            autocomplete="current-password"
            required
          />
          <button type="button" class="btn icon" :aria-label="reveal ? '隐藏密码' : '显示密码'" @click="reveal = !reveal">
            <Icon :name="reveal ? 'eye-off' : 'eye'" :size="15" />
          </button>
        </div>
      </div>

      <button class="btn primary lg go" type="submit" :disabled="busy || !username || !password">
        {{ busy ? '登录中…' : '进入控制台' }}
      </button>

      <div class="login-foot">独立管理员鉴权 · 连续错误 5 次自动锁定 15 分钟</div>
    </form>
  </div>
</template>

<style scoped>
.login-page {
  position: relative;
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: 40px 20px;
  background: radial-gradient(640px 360px at 50% 0%, var(--coral-soft), transparent 70%), var(--canvas);
}
.theme-toggle {
  position: fixed;
  top: 18px;
  right: 18px;
}
.login-card {
  width: min(400px, 100%);
  padding: 32px 30px 26px;
  background: var(--surface);
  border: 1px solid var(--line-strong);
  border-radius: 20px;
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
  gap: 17px;
}
.login-head {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.login-head h1 {
  font-size: 21px;
  font-weight: 720;
  letter-spacing: -0.02em;
}
.login-head p {
  font-size: 13px;
  color: var(--ink-2);
  margin-top: 3px;
}
.pw-row {
  display: flex;
  gap: 8px;
}
.go {
  width: 100%;
  margin-top: 2px;
}
.login-foot {
  font-size: 12px;
  color: var(--ink-3);
  text-align: center;
  padding-top: 10px;
  border-top: 1px solid var(--line);
}
</style>
