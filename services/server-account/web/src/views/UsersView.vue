<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import type { UserList } from '../types';
import UserDrawer from '../components/UserDrawer.vue';
import CreateUserDialog from '../components/CreateUserDialog.vue';

const creating = ref(false);
const query = ref('');
const status = ref('');
const page = ref(1);
const data = ref<UserList | null>(null);
const error = ref('');
const loading = ref(false);
const selected = ref<string | null>(null);

const KIND_LABEL: Record<string, string> = { email: '邮箱', phone: '手机' };
const pageCount = computed(() => Math.max(1, Math.ceil((data.value?.total ?? 0) / (data.value?.page_size ?? 20))));

let sequence = 0;
async function load() {
  const mine = ++sequence;
  loading.value = true;
  error.value = '';
  try {
    const result = await api.users({ q: query.value.trim(), status: status.value, page: page.value });
    if (mine === sequence) data.value = result;
  } catch (e) {
    if (mine === sequence) error.value = errorMessage(e);
  } finally {
    if (mine === sequence) loading.value = false;
  }
}

let timer: ReturnType<typeof setTimeout> | undefined;
watch(query, () => {
  clearTimeout(timer);
  timer = setTimeout(() => {
    page.value = 1;
    load();
  }, 300);
});
watch(status, () => {
  page.value = 1;
  load();
});
onMounted(load);

function go(delta: number) {
  page.value = Math.min(pageCount.value, Math.max(1, page.value + delta));
  load();
}
</script>

<template>
  <div class="page-head"><h2>用户</h2><button class="btn primary" @click="creating = true">创建用户</button></div>

  <div class="toolbar">
    <input v-model="query" class="input" type="search" aria-label="搜索用户" placeholder="账号 ID / 用户名 / 完整邮箱 / 完整手机号 / UID" />
    <select v-model="status" class="input" aria-label="按状态筛选">
      <option value="">全部状态</option>
      <option value="active">正常</option>
      <option value="disabled">已停用</option>
    </select>
  </div>
  <p v-if="error" class="error" role="alert">{{ error }}</p>

  <div class="card table-wrap">
    <table class="table">
      <thead><tr><th>账号 ID</th><th>用户名</th><th>登录标识</th><th>状态</th><th>注册时间</th><th>最近登录</th></tr></thead>
      <tbody>
        <tr v-for="user in data?.items" :key="user.uid" class="clickable" tabindex="0" @click="selected = user.uid" @keydown.enter="selected = user.uid">
          <td class="mono">{{ user.account_id }}</td>
          <td>{{ user.username }}</td>
          <td>
            <div v-for="(identity, index) in user.identities" :key="index" class="ident">
              <span class="muted">{{ KIND_LABEL[identity.kind] ?? identity.kind }}</span> <span class="mono">{{ identity.masked }}</span>
            </div>
          </td>
          <td><span class="badge" :class="user.status === 'active' ? 'ok' : 'bad'">{{ user.status === 'active' ? '正常' : '已停用' }}</span></td>
          <td>{{ formatTime(user.created_at) }}</td>
          <td>{{ formatTime(user.last_login_at) }}</td>
        </tr>
      </tbody>
    </table>
    <div v-if="data && !data.items.length" class="empty">没有符合条件的用户</div>
    <div v-else-if="!data && loading" class="empty">加载中…</div>
  </div>

  <div v-if="data && data.total > 0" class="pager">
    <span class="muted">共 {{ data.total }} 位用户</span>
    <button type="button" class="btn small" :disabled="page <= 1 || loading" @click="go(-1)">上一页</button>
    <span>{{ page }} / {{ pageCount }}</span>
    <button type="button" class="btn small" :disabled="page >= pageCount || loading" @click="go(1)">下一页</button>
  </div>

  <CreateUserDialog v-if="creating" @close="creating = false" @created="creating = false; load()" />
  <UserDrawer v-if="selected" :uid="selected" @close="selected = null" @changed="load" />
</template>

<style scoped>
.ident { white-space: nowrap; }
.pager { display: flex; align-items: center; justify-content: flex-end; gap: 12px; margin-top: 14px; }
</style>

