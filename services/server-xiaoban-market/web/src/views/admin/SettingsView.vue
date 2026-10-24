<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { accountPasswordLogin, admin, setSession, type AccountBinding } from '@/lib/api';
import { formatTime } from '@/lib/format';
import { toast, toastError } from '@/lib/toast';

const router = useRouter();
const form = ref({ current: '', next: '', again: '' });
const busy = ref(false);

const account = ref<{ url: string; clientId: string } | null>(null);
const bindings = ref<AccountBinding[]>([]);
const bind = ref({ identifier: '', password: '', label: '' });

async function loadBindings() {
  try {
    const [cfg, list] = await Promise.all([admin.config(), admin.bindings()]);
    account.value = cfg.account.enabled ? cfg.account : null;
    bindings.value = list.results;
  } catch (e) {
    toastError(e);
  }
}

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

async function doBind() {
  if (!account.value) return;
  busy.value = true;
  try {
    const token = await accountPasswordLogin(account.value.url, account.value.clientId, bind.value.identifier, bind.value.password);
    await admin.bindAccount(token, bind.value.label.trim() || bind.value.identifier.trim());
    toast('已绑定');
    bind.value = { identifier: '', password: '', label: '' };
    await loadBindings();
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}

async function unbind(b: AccountBinding) {
  if (!window.confirm(`确定解除「${b.label}」的后台管理员权限吗？它当前已登录的会话会立刻失效。`)) return;
  try {
    await admin.unbindAccount(b.id);
    toast('已解绑');
    await loadBindings();
  } catch (e) {
    toastError(e);
  }
}

onMounted(loadBindings);
</script>

<template>
  <div class="admin-page">
    <div class="page-h"><div><h1>设置</h1><div class="sub">后台登录与管理员</div></div></div>

    <form class="panel box" @submit.prevent="submit">
      <h2>修改本地管理员密码</h2>
      <div class="field"><label for="pc">当前密码</label><input id="pc" v-model="form.current" class="input" type="password" autocomplete="current-password" required /></div>
      <div class="field"><label for="pn">新密码（至少 8 位）</label><input id="pn" v-model="form.next" class="input" type="password" autocomplete="new-password" minlength="8" required /></div>
      <div class="field"><label for="pa">再输入一次新密码</label><input id="pa" v-model="form.again" class="input" type="password" autocomplete="new-password" minlength="8" required /></div>
      <div><button class="btn btn-primary" type="submit" :disabled="busy">修改密码</button></div>
    </form>

    <section v-if="account" class="panel wide">
      <div class="panel-h"><h2>相伴账号管理员</h2><span class="hint">绑定后可以用相伴账号密码登录后台</span></div>
      <div v-if="!bindings.length" class="empty">还没有绑定任何相伴账号。</div>
      <div v-else class="tbl-wrap">
        <table class="tbl">
          <thead><tr><th>名称</th><th>账号 ID</th><th>绑定人</th><th>绑定时间</th><th /></tr></thead>
          <tbody>
            <tr v-for="b in bindings" :key="b.id">
              <td>{{ b.label }}</td>
              <td class="num">{{ b.accountId }}</td>
              <td>{{ b.boundBy }}</td>
              <td>{{ formatTime(b.createdAt) }}</td>
              <td><button class="btn btn-sm btn-danger" type="button" @click="unbind(b)">解绑</button></td>
            </tr>
          </tbody>
        </table>
      </div>
      <form class="bind" @submit.prevent="doBind">
        <div class="field"><label for="bi">要绑定的账号 / 邮箱 / 手机号</label><input id="bi" v-model="bind.identifier" class="input" autocomplete="off" required /></div>
        <div class="field"><label for="bp">该账号的密码</label><input id="bp" v-model="bind.password" class="input" type="password" autocomplete="off" required /></div>
        <div class="field"><label for="bl">显示名称（可选）</label><input id="bl" v-model="bind.label" class="input" autocomplete="off" /></div>
        <button class="btn btn-primary" type="submit" :disabled="busy || !bind.identifier || !bind.password">绑定为管理员</button>
        <p class="hint full">这个账号的密码只会发给账号中心，用来换取一个访问令牌；小伴市场只保存账号 ID。应用 <code>{{ account.clientId }}</code> 需要已在账号中心登记。</p>
      </form>
    </section>
  </div>
</template>

<style scoped>
.box { max-width: 420px; padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.box h2 { font-size: 16px; font-weight: 700; }
.wide { max-width: 860px; }
.bind { display: flex; flex-wrap: wrap; align-items: flex-end; gap: 12px; padding: 16px 20px; border-top: 1px solid var(--line); }
.bind .field { flex: 1 1 200px; }
.full { flex: 1 1 100%; }
</style>
