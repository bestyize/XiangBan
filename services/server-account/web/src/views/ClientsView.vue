<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import type { ClientRow } from '../types';

const items = ref<ClientRow[]>([]);
const error = ref('');
const loading = ref(false);
const newId = ref('');
const newName = ref('');
const creating = ref(false);

async function load() {
  loading.value = true;
  try {
    items.value = (await api.clients()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

async function run(action: () => Promise<unknown>) {
  error.value = '';
  try {
    await action();
    await load();
  } catch (e) {
    error.value = errorMessage(e);
  }
}

async function create() {
  creating.value = true;
  await run(async () => {
    await api.createClient(newId.value.trim(), newName.value.trim());
    newId.value = '';
    newName.value = '';
  });
  creating.value = false;
}

function toggle(client: ClientRow) {
  const verb = client.enabled ? '停用' : '启用';
  const tip = client.enabled ? '停用后该应用无法登录，也无法刷新令牌；已发出的访问令牌最长再存活约 30 分钟。' : '';
  if (!window.confirm(`确定${verb}「${client.name}」？${tip}`)) return;
  run(() => api.updateClient(client.client_id, { enabled: !client.enabled }));
}

function rename(client: ClientRow) {
  const name = window.prompt('新的应用名称', client.name)?.trim();
  if (name && name !== client.name) run(() => api.updateClient(client.client_id, { name }));
}
</script>

<template>
  <div class="page-head"><h2>应用</h2></div>
  <p class="muted lead">每个接入账号中心的产品是一个应用，登录时必须带上其 client_id；令牌的 aud 就是这个值。</p>
  <p v-if="error" class="error" role="alert">{{ error }}</p>

  <form class="card add" @submit.prevent="create">
    <label class="field">
      <span>client_id（小写字母、数字、. _ -）</span>
      <input v-model="newId" class="input" required minlength="2" maxlength="64" pattern="[a-z0-9][a-z0-9._\-]*" placeholder="例如 xiangban-tv" />
    </label>
    <label class="field">
      <span>应用名称</span>
      <input v-model="newName" class="input" required maxlength="40" placeholder="例如 小伴电视端" />
    </label>
    <button class="btn primary" type="submit" :disabled="creating">新增应用</button>
  </form>

  <div class="card table-wrap">
    <table class="table">
      <thead><tr><th>应用</th><th>client_id</th><th>状态</th><th>有效会话</th><th>创建时间</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="client in items" :key="client.client_id">
          <td>{{ client.name }}</td>
          <td class="mono">{{ client.client_id }}</td>
          <td><span class="badge" :class="client.enabled ? 'ok' : 'bad'">{{ client.enabled ? '启用' : '已停用' }}</span></td>
          <td>{{ client.active_sessions }}</td>
          <td>{{ formatTime(client.created_at) }}</td>
          <td class="ops">
            <button type="button" class="btn small" @click="rename(client)">重命名</button>
            <button type="button" class="btn small" :class="{ danger: client.enabled }" @click="toggle(client)">{{ client.enabled ? '停用' : '启用' }}</button>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-if="!items.length && !loading" class="empty">还没有登记任何应用</div>
  </div>
</template>

<style scoped>
.lead { margin: -6px 0 14px; }
.add { display: flex; flex-wrap: wrap; align-items: flex-end; gap: 12px; padding: 16px; margin-bottom: 16px; }
.add .field { flex: 1 1 220px; }
.ops { white-space: nowrap; display: flex; gap: 8px; }
</style>
