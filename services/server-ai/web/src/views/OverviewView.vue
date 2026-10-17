<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { money, units } from '../format';
import type { Overview } from '../types';

const emit = defineEmits<{ go: [string] }>();

const data = ref<Overview | null>(null);
const error = ref('');

onMounted(async () => {
  try {
    data.value = await api.overview();
    units.perDollar = data.value.quota_per_unit;
  } catch (e) {
    error.value = errorMessage(e);
  }
});
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>概览</h1>
        <p>先看有没有需要处理的事，再看最近一天的额度进出。</p>
      </div>
    </div>

    <div v-if="error" class="notice error" role="alert">{{ error }}</div>

    <template v-if="data">
      <div class="alerts">
        <div v-if="!data.newapi_configured" class="notice error" role="alert">
          还没有设置 AI 网关的管理令牌。设置之前，用户无法开通，也不能调用 AI。
          <button type="button" class="linkbtn" @click="emit('go', 'gateway')">去填写</button>
        </div>
        <div v-if="data.open_anomalies > 0" class="notice warn">
          有 {{ data.open_anomalies }} 条对账异常没处理，可能是有人在 New API 后台手工改过用户额度。
          <button type="button" class="linkbtn" @click="emit('go', 'issues')">去处理</button>
        </div>
        <div v-if="data.newapi_configured && data.open_anomalies === 0" class="notice ok">AI 网关已连接，对账正常，没有待处理的异常。</div>
      </div>

      <section class="section">
        <div class="section-head"><h2>最近 24 小时</h2></div>
        <div class="flow panel">
          <div class="flow-cell">
            <div class="muted">用户用掉</div>
            <div class="figure num">{{ money(data.consumed_24h) }}</div>
          </div>
          <div class="flow-cell">
            <div class="muted">发放给用户</div>
            <div class="figure num">{{ money(data.granted_24h) }}</div>
            <div class="hint">含试用、兑换和你手动补发的</div>
          </div>
          <div class="flow-cell">
            <div class="muted">用户手里还剩</div>
            <div class="figure num">{{ money(data.balance) }}</div>
            <div class="hint">这是你对用户的未用额度</div>
          </div>
        </div>
      </section>

      <section class="section">
        <div class="section-head"><h2>用户</h2></div>
        <div class="panel table-wrap">
          <table class="table">
            <thead>
              <tr><th>产品</th><th class="right">用过 AI 的账号</th></tr>
            </thead>
            <tbody>
              <tr v-for="p in data.products" :key="p.product_id">
                <td>{{ p.name }}</td>
                <td class="right num">{{ p.bindings }}</td>
              </tr>
              <tr v-if="data.products.length === 0">
                <td colspan="2" class="table-empty">
                  还没有产品。
                  <button type="button" class="linkbtn" @click="emit('go', 'products')">去添加产品</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <p class="hint under num">
          共 {{ data.bindings }} 个账号与产品的组合，其中 {{ data.active_bindings }} 个已开通，{{ data.funded_bindings }} 个现在还有可用额度。
        </p>
      </section>
    </template>
    <p v-else-if="!error" class="muted">加载中…</p>
  </div>
</template>

<style scoped>
.alerts { display: flex; flex-direction: column; gap: 10px; }
.flow { display: grid; grid-template-columns: repeat(3, 1fr); }
.flow-cell { padding: 18px 22px; display: flex; flex-direction: column; gap: 2px; }
.flow-cell + .flow-cell { border-left: 1px solid var(--line); }
.figure { font-size: 28px; font-weight: 650; letter-spacing: -0.01em; line-height: 1.3; }
.under { margin-top: 10px; }
.linkbtn { border: none; background: none; padding: 0; margin-left: 6px; color: var(--primary); font-weight: 600; cursor: pointer; text-decoration: underline; }

@media (max-width: 640px) {
  .flow { grid-template-columns: 1fr; }
  .flow-cell + .flow-cell { border-left: none; border-top: 1px solid var(--line); }
}
</style>
