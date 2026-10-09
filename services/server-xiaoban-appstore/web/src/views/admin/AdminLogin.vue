<template>
  <div class="login-wrapper">
    <div class="login-card">
      <div class="login-header">
        <span class="brand-emoji">❖</span>
        <h2>小伴精选管理后台</h2>
        <p>登录后可进行应用一键上架、下架及配置</p>
      </div>

      <form class="login-form" @submit.prevent="handleLogin">
        <div v-if="errorMessage" class="error-alert">
          {{ errorMessage }}
        </div>

        <div class="form-item">
          <label>管理员账号</label>
          <div class="input-wrap">
            <User :size="16" class="input-icon" />
            <input
              v-model="username"
              type="text"
              required
              placeholder="请输入管理员账号 (默认: admin)"
            />
          </div>
        </div>

        <div class="form-item">
          <label>密码</label>
          <div class="input-wrap">
            <Lock :size="16" class="input-icon" />
            <input
              v-model="password"
              type="password"
              required
              placeholder="请输入密码 (默认: admin123)"
            />
          </div>
        </div>

        <button type="submit" class="btn-login" :disabled="loading">
          {{ loading ? '正在验证凭证...' : '立即登录' }}
        </button>

        <div class="login-footer">
          <router-link to="/" class="back-link">
            ← 返回前台应用商店
          </router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../../stores/auth';
import { User, Lock } from 'lucide-vue-next';

const router = useRouter();
const authStore = useAuthStore();

const username = ref('admin');
const password = ref('admin123');
const loading = ref(false);
const errorMessage = ref('');

async function handleLogin() {
  loading.value = true;
  errorMessage.value = '';
  try {
    await authStore.login({
      username: username.value,
      password: password.value,
    });
    router.push('/admin/apps');
  } catch (err: any) {
    errorMessage.value = err.message || '登录失败，请检查账号密码';
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.login-wrapper {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 420px;
  background: #ffffff;
  border-radius: 16px;
  padding: 36px 32px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.25);
}

.login-header {
  text-align: center;
  margin-bottom: 28px;
}

.brand-emoji {
  font-size: 40px;
  display: inline-block;
  margin-bottom: 8px;
}

.login-header h2 {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 6px;
}

.login-header p {
  font-size: 13px;
  color: #64748b;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.error-alert {
  background: #fef2f2;
  border: 1px solid #fecaca;
  color: #dc2626;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-item label {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
}

.input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.input-icon {
  position: absolute;
  left: 12px;
  color: #94a3b8;
}

.input-wrap input {
  width: 100%;
  padding: 10px 14px 10px 38px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-size: 14px;
  color: #1e293b;
  outline: none;
  transition: all 0.15s ease;
}

.input-wrap input:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
}

.btn-login {
  background: #2563eb;
  color: #ffffff;
  padding: 11px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  margin-top: 6px;
  box-shadow: 0 2px 8px rgba(37, 99, 235, 0.25);
  transition: all 0.2s ease;
}

.btn-login:hover:not(:disabled) {
  background: #1d4ed8;
  transform: translateY(-1px);
}

.btn-login:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.login-footer {
  text-align: center;
  margin-top: 8px;
}

.back-link {
  font-size: 13px;
  color: #64748b;
  transition: color 0.15s ease;
}

.back-link:hover {
  color: #2563eb;
}
</style>
