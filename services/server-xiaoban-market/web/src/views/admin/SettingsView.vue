<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { admin, setSession } from '@/lib/api';
import { toast, toastError } from '@/lib/toast';

const router = useRouter();
const form = ref({ current: '', next: '', again: '' });
const busy = ref(false);

async function submit() {
  if (form.value.next !== form.value.again) return toast('两次输入的新密码不一致', 'bad');
  busy.value = true;
  try {
    await admin.changePassword(form.value.current, form.value.next);
    toast('密码已修改，请重新登录');
    setSession('');
    router.push({ name: 'admin-login' });
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="admin-page">
    <div class="page-h"><div><h1>设置</h1><div class="sub">修改后台登录密码</div></div></div>
    <form class="panel box" @submit.prevent="submit">
      <div class="field"><label for="pc">当前密码</label><input id="pc" v-model="form.current" class="input" type="password" autocomplete="current-password" required /></div>
      <div class="field"><label for="pn">新密码（至少 8 位）</label><input id="pn" v-model="form.next" class="input" type="password" autocomplete="new-password" minlength="8" required /></div>
      <div class="field"><label for="pa">再输入一次新密码</label><input id="pa" v-model="form.again" class="input" type="password" autocomplete="new-password" minlength="8" required /></div>
      <div><button class="btn btn-primary" type="submit" :disabled="busy">修改密码</button></div>
    </form>
  </div>
</template>

<style scoped>
.box { max-width: 420px; padding: 24px; display: flex; flex-direction: column; gap: 16px; }
</style>
