<script setup lang="ts">
import { useModal } from '../composables/useModal';
import { ref } from 'vue';
import { api, errorMessage } from '../api';
import { signOut } from '../session';

const emit = defineEmits<{ close: [] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const current = ref('');
const next = ref('');
const confirm = ref('');
const busy = ref(false);
const error = ref('');

async function submit() {
  error.value = '';
  if (next.value !== confirm.value) {
    error.value = '两次输入的新密码不一致';
    return;
  }
  busy.value = true;
  try {
    await api.changePassword(current.value, next.value);
    // 改密码会让所有已登录的管理员令牌失效，直接回到登录页。
    signOut();
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="scrim" @click="emit('close')"></div>
  <form ref="root" class="dialog" role="dialog" aria-modal="true" aria-labelledby="pw-title" @submit.prevent="submit">
    <div class="dialog-head">
      <h2 id="pw-title">修改管理员密码</h2>
      <p class="hint">修改后需要重新登录，其他设备上的登录也会失效。</p>
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>
      <div class="field">
        <label for="pw-current">当前密码</label>
        <input id="pw-current" v-model="current" class="input" type="password" autocomplete="current-password" required autofocus />
      </div>
      <div class="field">
        <label for="pw-next">新密码</label>
        <input id="pw-next" v-model="next" class="input" type="password" autocomplete="new-password" minlength="8" required />
        <span class="hint">至少 8 位。</span>
      </div>
      <div class="field">
        <label for="pw-confirm">再输入一次新密码</label>
        <input id="pw-confirm" v-model="confirm" class="input" type="password" autocomplete="new-password" required />
      </div>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy">{{ busy ? '保存中…' : '保存新密码' }}</button>
    </div>
  </form>
</template>
