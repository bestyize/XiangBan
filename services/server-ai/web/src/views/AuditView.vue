<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { auditLabels, formatTime } from '../format';
import type { AuditRow } from '../types';

const items = ref<AuditRow[]>([]);
const error = ref('');
const loading = ref(true);

onMounted(async () => {
  try {
    items.value = (await api.audit()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>操作日志</h1>
        <p>管理员做过的改动，最近 100 条。改档位、补发和扣回额度都会记在这里。</p>
      </div>
    </div>

    <div v-if="error" class="notice error" role="alert">{{ error }}</div>

    <div class="panel">
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr><th>时间</th><th>管理员</th><th>操作</th><th>对象</th><th>详情</th></tr>
          </thead>
          <tbody>
            <tr v-for="(row, i) in items" :key="i">
              <td class="num nowrap">{{ formatTime(row.created_at) }}</td>
              <td>{{ row.admin }}</td>
              <td>{{ auditLabels[row.action] ?? row.action }}</td>
              <td class="mono">{{ row.target ?? '—' }}</td>
              <td class="muted detail mono">{{ row.detail ?? '' }}</td>
            </tr>
            <tr v-if="!loading && items.length === 0">
              <td colspan="5" class="table-empty">还没有操作记录。</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.nowrap { white-space: nowrap; }
.detail { max-width: 46ch; overflow-wrap: anywhere; }
</style>
