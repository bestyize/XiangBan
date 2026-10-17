<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime } from '../format';
import type { Anomaly } from '../types';

const emit = defineEmits<{ changed: [] }>();

const items = ref<Anomaly[]>([]);
const error = ref('');
const loading = ref(true);

async function load() {
  try {
    items.value = (await api.anomalies()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

async function resolve(item: Anomaly) {
  error.value = '';
  try {
    await api.resolveAnomaly(item.id);
    await load();
    emit('changed');
  } catch (e) {
    error.value = errorMessage(e);
  }
}

const kindLabel: Record<string, string> = { external_credit: '网关里的额度比预期多' };
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>对账异常</h1>
        <p>系统每分钟会核对 AI 网关里的用户额度。核对不上时记在这里；系统已经按我们的账本把网关里的额度改回正确的数，处理只是确认你知道了。</p>
      </div>
    </div>

    <div v-if="error" class="notice error" role="alert">{{ error }}</div>

    <div class="panel">
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr><th>发现时间</th><th>情况</th><th>绑定</th><th class="right">状态</th></tr>
          </thead>
          <tbody>
            <tr v-for="item in items" :key="item.id" :class="{ done: item.resolved_at }">
              <td class="num nowrap">{{ formatTime(item.created_at) }}</td>
              <td>
                <strong>{{ kindLabel[item.kind] ?? item.kind }}</strong>
                <div class="muted detail">{{ item.detail }}</div>
              </td>
              <td class="num">#{{ item.binding_id }}</td>
              <td class="right">
                <span v-if="item.resolved_at" class="tag">已处理</span>
                <button v-else type="button" class="btn small" @click="resolve(item)">标记为已处理</button>
              </td>
            </tr>
            <tr v-if="!loading && items.length === 0">
              <td colspan="4" class="table-empty">没有异常，账目是对的。</td>
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
.detail { max-width: 52ch; margin-top: 2px; }
.done { opacity: 0.6; }
</style>
