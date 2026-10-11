<script setup lang="ts">
// 操作日志：最近的管理操作，最新在前。
import { onMounted, ref } from 'vue'
import { adminApi, AdminApiError, type AuditRow } from '@/api/admin'
import { AUDIT_ACTIONS, AUDIT_TARGETS, errorText, fmtTime } from './shared'

const limit = ref(50)
const rows = ref<AuditRow[]>([])
const loading = ref(true)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    rows.value = await adminApi.audit(limit.value)
  } catch (e) {
    if (!(e instanceof AdminApiError && e.status === 401)) error.value = errorText(e)
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <select v-model.number="limit" class="select" aria-label="显示条数" @change="load">
      <option :value="50">最近 50 条</option>
      <option :value="100">最近 100 条</option>
      <option :value="200">最近 200 条</option>
    </select>
    <button class="btn" :disabled="loading" @click="load">刷新</button>
  </div>

  <div class="panel adm-card">
    <p v-if="error" class="adm-state error-text">{{ error }}</p>
    <p v-else-if="loading && !rows.length" class="adm-state muted">加载中</p>
    <p v-else-if="!rows.length" class="adm-empty">还没有操作记录</p>
    <table v-else class="adm-table">
      <thead>
        <tr>
          <th>时间</th>
          <th>管理员</th>
          <th>操作</th>
          <th>对象</th>
          <th>详情</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="r in rows" :key="r.id">
          <td data-label="时间" class="shrink">{{ fmtTime(r.created_at) }}</td>
          <td data-label="管理员">{{ r.admin }}</td>
          <td data-label="操作" class="shrink">
            <span class="adm-title">{{ AUDIT_ACTIONS[r.action] ?? r.action }}</span>
          </td>
          <td data-label="对象" class="adm-wrap">{{ AUDIT_TARGETS[r.target_type] ?? r.target_type }} {{ r.target_id }}</td>
          <td data-label="详情" class="adm-wrap adm-note">{{ r.detail || '-' }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
