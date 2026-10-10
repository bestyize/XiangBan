<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { signOut } from '../session';

const emit = defineEmits<{ close: [] }>();
const current = ref('');
const next = ref('');
const confirm = ref('');
const error = ref('');
const busy = ref(false);

async function submit() {
  error.value = '';
  if (next.value !== confirm.value) {
    error.value = '两次输入的新密码不一致';
    return;
  }
  busy.value = true;
  try {
    await api.changePassword(current.value, next.value);
    signOut();
  } catch (e) {
    error.value = errorMessage(e);
    busy.value = false;
  }
}

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close');
}
onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <form class="card dialog" role="dialog" aria-modal="true" aria-labelledby="pw-title" @submit.prevent="submit">
      <h3 id="pw-title">修改管理员密码</h3>
      <p class="muted">修改后所有已登录的管理员会话会失效，需要重新登录。</p>
      <label class="field">
        <span>当前密码</span>
        <input v-model="current" class="input" type="password" autocomplete="current-password" required autofocus />
      </label>
      <label class="field">
        <span>新密码（至少 8 位）</span>
        <input v-model="next" class="input" type="password" autocomplete="new-password" minlength="8" required />
      </label>
      <label class="field">
        <span>确认新密码</span>
        <input v-model="confirm" class="input" type="password" autocomplete="new-password" required />
      </label>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <div class="actions">
        <button type="button" class="btn" @click="emit('close')">取消</button>
        <button type="submit" class="btn primary" :disabled="busy">{{ busy ? '提交中…' : '确认修改' }}</button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.overlay { position: fixed; inset: 0; background: rgba(28, 32, 51, 0.4); display: grid; place-items: center; padding: 20px; z-index: 50; }
.dialog { width: 100%; max-width: 400px; padding: 24px; display: flex; flex-direction: column; gap: 14px; }
p { margin: 0; }
.actions { display: flex; justify-content: flex-end; gap: 10px; }
</style>
