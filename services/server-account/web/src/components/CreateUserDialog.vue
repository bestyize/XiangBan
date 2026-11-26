<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref } from 'vue';
import { api, errorMessage } from '../api';
import { copyWithToast, showToast } from '../toast';
import Icon from './Icon.vue';

const emit = defineEmits<{ close: []; created: [] }>();

const form = reactive({
  request_id: crypto.randomUUID(),
  username: '',
  password: '',
  avatar_url: '',
});
const showPassword = ref(false);
const busy = ref(false);
const error = ref('');

function generatePassword() {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%';
  const arr = new Uint32Array(14);
  crypto.getRandomValues(arr);
  form.password = Array.from(arr, (n) => chars[n % chars.length]).join('');
  showPassword.value = true;
}

async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    await api.createUser({
      request_id: form.request_id,
      username: form.username.trim(),
      password: form.password,
      avatar_url: form.avatar_url.trim() || undefined,
    });
    const createdName = form.username.trim();
    form.password = '';
    showToast(`用户「${createdName}」创建成功`);
    emit('created');
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && !busy.value) emit('close');
}
onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="scrim" @click="!busy && emit('close')" />
  <form
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-labelledby="create-title"
    @submit.prevent="submit"
  >
    <div class="dialog-head">
      <div>
        <h3 id="create-title">创建用户</h3>
        <p>使用用户名与初始密码直接建号；邮箱/手机号暂不绑定，用户登录后可自行验证绑定。</p>
      </div>
      <button type="button" class="btn ghost icon sm" :disabled="busy" aria-label="关闭" @click="emit('close')">
        <Icon name="x" :size="16" />
      </button>
    </div>

    <div class="dialog-body">
      <label class="field">
        <span class="field-label">用户名</span>
        <input
          v-model="form.username"
          class="input"
          required
          maxlength="20"
          pattern="[A-Za-z\u4e00-\u9fff][A-Za-z0-9_\u4e00-\u9fff]*"
          autocomplete="off"
          placeholder="例如 xiaoban_editor"
          autofocus
        />
        <span class="hint">至少 2 个汉字或 3 个字母，最长 20（汉字按 2 计），注册后用户本人不可修改。</span>
      </label>

      <label class="field">
        <span class="field-label">
          <span>初始密码（8–128 位）</span>
          <span class="pwd-tools">
            <button type="button" class="inline-act" @click="generatePassword">
              <Icon name="sparkles" :size="12" />
              <span>随机生成</span>
            </button>
            <button
              v-if="form.password"
              type="button"
              class="inline-act"
              @click="copyWithToast(form.password, '已复制初始密码')"
            >
              <Icon name="copy" :size="12" />
              <span>复制</span>
            </button>
          </span>
        </span>
        <div class="input-action-wrap">
          <input
            v-model="form.password"
            class="input"
            :type="showPassword ? 'text' : 'password'"
            required
            minlength="8"
            maxlength="128"
            autocomplete="new-password"
            placeholder="请输入至少 8 位密码"
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

      <label class="field">
        <span class="field-label">头像地址（选填）</span>
        <input
          v-model="form.avatar_url"
          class="input"
          type="url"
          placeholder="https://…"
        />
        <span class="hint">需为不超过 512 字符的 https 链接。</span>
      </label>

      <div v-if="error" class="banner danger" role="alert">
        <Icon name="alert" :size="15" class="banner-ico" />
        <span>{{ error }}</span>
      </div>
    </div>

    <div class="dialog-foot">
      <button class="btn" type="button" :disabled="busy" @click="emit('close')">取消</button>
      <button class="btn primary" type="submit" :disabled="busy">
        {{ busy ? '创建中…' : '创建用户' }}
      </button>
    </div>
  </form>
</template>

<style scoped>
.pwd-tools {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}
.inline-act {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 550;
  color: var(--ink-2);
}
.inline-act:hover {
  color: var(--ink);
}
</style>
