<script setup lang="ts">
import { reactive, ref } from 'vue';
import { api, errorMessage } from '../api';
const emit = defineEmits<{ close: []; created: [] }>();
const form = reactive({ request_id: crypto.randomUUID(), username: '', password: '', avatar_url: '' });
const busy = ref(false); const error = ref('');
async function submit() {
  busy.value = true; error.value = '';
  try { await api.createUser(form); form.password = ''; emit('created'); }
  catch (e) { error.value = errorMessage(e); }
  finally { busy.value = false; }
}
</script>
<template>
  <div class="overlay" @click.self="!busy && emit('close')">
    <form class="card create-dialog" role="dialog" aria-modal="true" aria-labelledby="create-title" @submit.prevent="submit">
      <h3 id="create-title">创建用户</h3>
      <p class="muted">使用用户名和密码登录。邮箱、手机号暂不绑定，用户可登录后自行验证绑定。</p>
      <label class="field"><span>用户名（4–20 位，以字母开头）</span><input v-model="form.username" class="input" required minlength="4" maxlength="20" pattern="[A-Za-z][A-Za-z0-9_]{3,19}" autocomplete="off" /></label>
      <label class="field"><span>初始密码（8–128 位）</span><input v-model="form.password" class="input" type="password" required minlength="8" maxlength="128" autocomplete="new-password" /></label>
      <label class="field"><span>头像地址（选填）</span><input v-model="form.avatar_url" class="input" type="url" placeholder="https://…" /></label>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <div class="actions"><button class="btn" type="button" :disabled="busy" @click="emit('close')">取消</button><button class="btn primary" :disabled="busy">{{ busy ? '创建中…' : '创建用户' }}</button></div>
    </form>
  </div>
</template>
<style scoped>
.overlay{position:fixed;inset:0;background:#0006;display:grid;place-items:center;padding:20px;z-index:50}.create-dialog{width:100%;max-width:440px;padding:24px;display:grid;gap:14px}.actions{display:flex;justify-content:flex-end;gap:10px}p,h3{margin:0}
</style>
