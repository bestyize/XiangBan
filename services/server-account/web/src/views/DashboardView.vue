<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import type { Stats } from '../types';

const stats = ref<Stats | null>(null);
const error = ref('');
const loading = ref(false);

async function load() {
  loading.value = true;
  error.value = '';
  try {
    stats.value = await api.stats();
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

const KIND_LABEL: Record<string, string> = { email: '邮箱', phone: '手机号' };

const cards = computed(() => {
  const s = stats.value;
  if (!s) return [];
  return [
    { label: '用户总数', value: s.users.total, hint: `停用 ${s.users.disabled}` },
    { label: '24 小时新增', value: s.users.new_24h, hint: `7 天新增 ${s.users.new_7d}` },
    { label: '24 小时登录用户', value: s.users.logged_in_24h, hint: '按最近登录时间统计' },
    { label: '有效会话', value: s.active_sessions, hint: '未过期且未吊销的设备会话' },
    { label: '24 小时验证码发送', value: s.codes_24h.total, hint: s.codes_24h.by_kind.map((k) => `${KIND_LABEL[k.kind] ?? k.kind} ${k.count}`).join(' · ') || '暂无' },
  ];
});
</script>

<template>
  <div class="page-head">
    <h2>概览</h2>
    <button class="btn" type="button" :disabled="loading" @click="load">{{ loading ? '刷新中…' : '刷新' }}</button>
  </div>
  <p v-if="error" class="error" role="alert">{{ error }}</p>

  <div v-if="stats" class="grid">
    <div v-for="card in cards" :key="card.label" class="card stat">
      <div class="muted">{{ card.label }}</div>
      <div class="value">{{ card.value }}</div>
      <div class="muted hint">{{ card.hint }}</div>
    </div>
  </div>

  <section v-if="stats" class="card ips">
    <h3>24 小时验证码发送来源 IP（前 5）</h3>
    <p class="muted">单个 IP 发送量异常偏高，通常意味着有人在刷短信，可考虑调低 ACCOUNT_DAILY_PER_IP 或在网关封禁。</p>
    <table v-if="stats.codes_24h.top_ips.length" class="table">
      <thead><tr><th>IP</th><th>发送次数</th></tr></thead>
      <tbody>
        <tr v-for="row in stats.codes_24h.top_ips" :key="row.ip">
          <td class="mono">{{ row.ip }}</td>
          <td>{{ row.count }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else class="empty">最近 24 小时没有发送记录</div>
  </section>
</template>

<style scoped>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 14px; margin-bottom: 20px; }
.stat { padding: 16px 18px; }
.value { font-size: 30px; font-weight: 600; line-height: 1.3; }
.hint { font-size: 12.5px; }
.ips { padding: 18px 20px; }
.ips h3 { font-size: 15px; }
.ips p { margin: 4px 0 10px; }
</style>
