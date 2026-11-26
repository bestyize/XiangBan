<script setup lang="ts">
import { ref } from 'vue';
import { api, errorMessage } from '../api';
import BrandMark from '../components/BrandMark.vue';
import Icon from '../components/Icon.vue';
import { signIn } from '../session';
import { useTheme } from '../theme';

const username = ref('');
const password = ref('');
const showPassword = ref(false);
const error = ref('');
const busy = ref(false);

const { isDark, toggleTheme } = useTheme();

async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.login(username.value.trim(), password.value);
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
    <button
      type="button"
      class="theme-corner btn ghost sm"
      :aria-label="isDark ? '切换浅色主题' : '切换深色主题'"
      @click="toggleTheme"
    >
      <Icon :name="isDark ? 'sun' : 'moon'" :size="15" />
      <span>{{ isDark ? '暖白浅色' : '极夜深色' }}</span>
    </button>

    <form class="login-card" @submit.prevent="submit">
      <div class="login-head">
        <BrandMark :size="40" />
        <div>
          <h1>相伴账号中心</h1>
          <p>管理员控制台登录 · 统一身份与安全管理</p>
        </div>
      </div>

      <label class="field">
        <span class="field-label">管理员用户名</span>
        <input
          v-model="username"
          class="input"
          autocomplete="username"
          placeholder="请输入管理员用户名"
          required
          autofocus
        />
      </label>

      <label class="field">
        <span class="field-label">密码</span>
        <div class="input-action-wrap">
          <input
            v-model="password"
            class="input"
            :type="showPassword ? 'text' : 'password'"
            autocomplete="current-password"
            placeholder="请输入密码"
            required
          />
          <button
            type="button"
            class="input-action-btn"
            :aria-label="showPassword ? '隐藏密码' : '显示密码'"
            @click="showPassword = !showPassword"
          >
            <Icon :name="showPassword ? 'eye-off' : 'eye'" :size="15" />
          </button>
        </div>
      </label>

      <div v-if="error" class="banner danger" role="alert">
        <Icon name="alert" :size="15" class="banner-ico" />
        <span>{{ error }}</span>
      </div>

      <button class="btn primary lg submit-btn" type="submit" :disabled="busy">
        <Icon name="shield" :size="15" />
        <span>{{ busy ? '正在验证身份…' : '登录控制台' }}</span>
      </button>

      <div class="login-foot">
        独立管理员鉴权体系 · 连续输错 5 次将自动锁定 15 分钟
      </div>
    </form>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: 40px 20px;
  position: relative;
  background:
    radial-gradient(640px 360px at 50% 0%, var(--coral-soft), transparent 70%),
    var(--canvas);
}

.theme-corner {
  position: fixed;
  top: 16px;
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
  gap: 18px;
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
  line-height: 1.25;
}
.login-head p {
  font-size: 13px;
  color: var(--ink-2);
  margin-top: 3px;
}

.submit-btn {
  width: 100%;
  margin-top: 2px;
}

.login-foot {
  font-size: 12px;
  color: var(--ink-3);
  text-align: center;
  padding-top: 12px;
  border-top: 1px solid var(--line);
}
</style>
