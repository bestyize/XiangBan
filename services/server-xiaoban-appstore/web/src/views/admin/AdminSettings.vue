<template>
  <div class="admin-settings-page">
    <div class="settings-card">
      <div class="card-header">
        <div class="header-icon">
          <Shield :size="20" />
        </div>
        <div>
          <h3>管理员账号与安全设置</h3>
          <p class="header-desc">修改管理后台的登录账号名称与登录密码</p>
        </div>
      </div>

      <!-- 提示消息 -->
      <div v-if="toastMessage" class="alert-box" :class="'alert-' + toastType">
        {{ toastMessage }}
      </div>

      <form class="settings-form" @submit.prevent="handleSubmit">
        <!-- 当前账号展示 -->
        <div class="form-item">
          <label class="item-label">当前登录账号</label>
          <div class="readonly-value">
            <User :size="15" class="val-icon" />
            <span>{{ authStore.username || 'admin' }}</span>
          </div>
        </div>

        <div class="divider"></div>

        <!-- 身份验证：原密码 -->
        <div class="form-item">
          <label class="item-label">当前原密码 <span class="required">*</span></label>
          <input
            v-model="currentPassword"
            type="password"
            required
            placeholder="请输入当前原密码进行身份校验"
            autocomplete="current-password"
          />
          <span class="field-hint">为保障安全，修改账号或密码前必须验证当前有效密码</span>
        </div>

        <!-- 修改用户名 -->
        <div class="form-item">
          <label class="item-label">新管理员账号名称</label>
          <input
            v-model="newUsername"
            type="text"
            :placeholder="authStore.username || '留空则保持原账号不变'"
            autocomplete="username"
          />
          <span class="field-hint">3~32 个字符，支持英文、数字与下划线</span>
        </div>

        <!-- 修改新密码 -->
        <div class="form-item">
          <label class="item-label">设置新密码</label>
          <input
            v-model="newPassword"
            type="password"
            placeholder="留空则保持原密码不变"
            autocomplete="new-password"
          />
          <span class="field-hint">不少于 6 个字符</span>
        </div>

        <!-- 确认新密码 -->
        <div class="form-item" v-if="newPassword">
          <label class="item-label">再次确认新密码 <span class="required">*</span></label>
          <input
            v-model="confirmPassword"
            type="password"
            required
            placeholder="请再次输入新密码"
            autocomplete="new-password"
          />
          <span v-if="newPassword && confirmPassword && newPassword !== confirmPassword" class="error-hint">
            两次输入的密码不一致
          </span>
        </div>

        <div class="form-actions">
          <button type="submit" class="btn-primary" :disabled="submitting">
            <Save :size="15" />
            <span>{{ submitting ? '保存中...' : '保存修改' }}</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useAuthStore } from '../../stores/auth';
import { api } from '../../api';
import { Shield, User, Save } from 'lucide-vue-next';

const authStore = useAuthStore();

const currentPassword = ref('');
const newUsername = ref('');
const newPassword = ref('');
const confirmPassword = ref('');
const submitting = ref(false);

const toastMessage = ref('');
const toastType = ref<'success' | 'error'>('success');

function showToast(msg: string, type: 'success' | 'error' = 'success') {
  toastMessage.value = msg;
  toastType.value = type;
  setTimeout(() => {
    toastMessage.value = '';
  }, 4000);
}

async function handleSubmit() {
  if (!currentPassword.value) {
    showToast('请输入当前原密码', 'error');
    return;
  }

  if (newPassword.value) {
    if (newPassword.value.length < 6) {
      showToast('新密码长度不能少于 6 个字符', 'error');
      return;
    }
    if (newPassword.value !== confirmPassword.value) {
      showToast('两次输入的新密码不一致，请核对', 'error');
      return;
    }
  }

  if (!newUsername.value && !newPassword.value) {
    showToast('请至少输入新的账号名称或新密码', 'error');
    return;
  }

  submitting.value = true;
  try {
    const res = await api.changeAdminCredentials({
      current_password: currentPassword.value,
      new_username: newUsername.value.trim() || undefined,
      new_password: newPassword.value.trim() || undefined,
    });

    // 更新前端持久化凭证与当前登录状态
    authStore.updateCredentials(res.username, res.token);

    // 清空表单
    currentPassword.value = '';
    newPassword.value = '';
    confirmPassword.value = '';
    newUsername.value = '';

    showToast(res.message || '管理员账号和密码修改成功！', 'success');
  } catch (err: any) {
    showToast(err.message || '修改失败，请检查原密码是否正确', 'error');
  } finally {
    submitting.value = false;
  }
}
</script>

<style scoped>
.admin-settings-page {
  max-width: 640px;
}

.settings-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 28px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 24px;
}

.header-icon {
  width: 42px;
  height: 42px;
  border-radius: 10px;
  background: #eff6ff;
  color: #2563eb;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-header h3 {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
}

.header-desc {
  font-size: 13px;
  color: #64748b;
  margin-top: 2px;
}

.alert-box {
  padding: 11px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 20px;
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.alert-success {
  background: #ecfdf5;
  color: #047857;
  border: 1px solid #a7f3d0;
}

.alert-error {
  background: #fef2f2;
  color: #b91c1c;
  border: 1px solid #fecaca;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.item-label {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
}

.required {
  color: #ef4444;
}

.readonly-value {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.val-icon {
  color: #2563eb;
}

.divider {
  height: 1px;
  background: #f1f5f9;
  margin: 4px 0;
}

.form-item input {
  padding: 10px 14px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-size: 14px;
  color: #0f172a;
  outline: none;
  transition: all 0.15s ease;
}

.form-item input:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
}

.field-hint {
  font-size: 12px;
  color: #94a3b8;
}

.error-hint {
  font-size: 12px;
  color: #ef4444;
  font-weight: 500;
}

.form-actions {
  display: flex;
  justify-content: flex-start;
  margin-top: 10px;
  padding-top: 18px;
  border-top: 1px solid #f1f5f9;
}
</style>
