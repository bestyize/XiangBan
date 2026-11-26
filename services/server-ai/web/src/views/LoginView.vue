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
    <button type="button" class="btn icon outline theme-toggle" :title="prefs.theme === 'dark' ? '切换浅色模式' : '切换深色模式'" @click="toggleTheme">
      <Icon :name="prefs.theme === 'dark' ? 'sun' : 'moon'" :size="15" />
    </button>

    <form class="login-card" @submit.prevent="submit">
      <div class="login-head">
        <BrandMark :size="38" />
        <div>
          <h1>小伴 AI 网关后台</h1>
          <p>管理产品档位、用户额度与上游网关</p>
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
        {{ busy ? '登录中…' : '登录' }}
      </button>

      <div class="login-foot">独立管理员鉴权 · 连续错误 5 次锁定 15 分钟</div>
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
  background: var(--canvas);
}
.theme-toggle {
  position: fixed;
  top: 18px;
  right: 18px;
}
.login-card {
  width: min(380px, 100%);
  padding: 28px 26px 24px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--r-dialog);
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.login-head {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.login-head h1 {
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.015em;
}
.login-head p {
  font-size: 12.5px;
  color: var(--ink-3);
  margin-top: 2px;
}
.pw-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.go {
  width: 100%;
  margin-top: 2px;
}
.login-foot {
  font-size: 11.5px;
  color: var(--ink-3);
  text-align: center;
  padding-top: 10px;
  border-top: 1px solid var(--line);
}
</style>
