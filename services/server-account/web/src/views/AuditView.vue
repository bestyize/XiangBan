<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import type { AuditRow } from '../types';

const ACTION_LABEL: Record<string, string> = {
  'user.disabled': '停用用户',
  'user.active': '启用用户',
  'user.revoke_sessions': '下线全部设备',
  'client.create': '新增应用',
  'client.enable': '启用应用',
  'client.disable': '停用应用',
  'client.rename': '重命名应用',
  'admin.password_change': '修改管理员密码',
};

const items = ref<AuditRow[]>([]);
const error = ref('');
const loading = ref(false);

async function load() {
  loading.value = true;
  error.value = '';
  try {
    items.value = (await api.audit()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);
</script>

<template>
  <div class="page-head">
    <h2>操作日志</h2>
    <button class="btn" type="button" :disabled="loading" @click="load">刷新</button>
  </div>
  <p v-if="error" class="error" role="alert">{{ error }}</p>
  <div class="card table-wrap">
    <table class="table">
      <thead><tr><th>时间</th><th>管理员</th><th>操作</th><th>对象</th><th>备注</th></tr></thead>
      <tbody>
        <tr v-for="(row, index) in items" :key="index">
          <td>{{ formatTime(row.created_at) }}</td>
          <td>{{ row.admin }}</td>
          <td>{{ ACTION_LABEL[row.action] ?? row.action }}</td>
          <td class="mono">{{ row.target ?? '—' }}</td>
          <td>{{ row.detail ?? '—' }}</td>
        </tr>
      </tbody>
    </table>
    <div v-if="!items.length && !loading" class="empty">暂无操作记录</div>
  </div>
  <p class="muted note">仅显示最近 100 条。</p>
</template>

<style scoped>
.note { margin-top: 10px; font-size: 12.5px; }
</style>
