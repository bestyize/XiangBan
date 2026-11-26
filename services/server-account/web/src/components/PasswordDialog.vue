<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { signOut } from '../session';
import { showToast } from '../toast';
import Icon from './Icon.vue';

const emit = defineEmits<{ close: [] }>();
const current = ref('');
const next = ref('');
const confirm = ref('');
const showPwd = ref(false);
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
    showToast('管理员密码已更新，请重新登录', 'info', 3000);
    signOut();
  } catch (e) {
    error.value = errorMessage(e);
    busy.value = false;
  }
}

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape' && !busy.value) emit('close');
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
    aria-labelledby="pw-title"
    @submit.prevent="submit"
  >
    <div class="dialog-head">
      <div>
        <h3 id="pw-title">修改管理员密码</h3>
        <p>修改后所有已登录的管理员会话将立即失效，需要重新登录。</p>
      </div>
      <button type="button" class="btn ghost icon sm" :disabled="busy" aria-label="关闭" @click="emit('close')">
        <Icon name="x" :size="16" />
      </button>
    </div>

    <div class="dialog-body">
      <label class="field">
        <span class="field-label">当前密码</span>
        <div class="input-action-wrap">
          <input
            v-model="current"
            class="input"
            :type="showPwd ? 'text' : 'password'"
            autocomplete="current-password"
            placeholder="请输入当前密码"
            required
            autofocus
          />
          <button
            type="button"
            class="input-action-btn"
            :aria-label="showPwd ? '隐藏密码' : '显示密码'"
            @click="showPwd = !showPwd"
          >
            <Icon :name="showPwd ? 'eye-off' : 'eye'" :size="15" />
          </button>
        </div>
      </label>

      <label class="field">
        <span class="field-label">新密码（至少 8 位）</span>
        <input
          v-model="next"
          class="input"
          :type="showPwd ? 'text' : 'password'"
          autocomplete="new-password"
          minlength="8"
          placeholder="请输入新密码"
          required
        />
      </label>

      <label class="field">
        <span class="field-label">确认新密码</span>
        <input
          v-model="confirm"
          class="input"
          :type="showPwd ? 'text' : 'password'"
          autocomplete="new-password"
          placeholder="再次输入新密码"
          required
        />
      </label>

      <div v-if="error" class="banner danger" role="alert">
        <Icon name="alert" :size="15" class="banner-ico" />
        <span>{{ error }}</span>
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" :disabled="busy" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy">
        {{ busy ? '提交中…' : '确认修改' }}
      </button>
    </div>
  </form>
</template>
