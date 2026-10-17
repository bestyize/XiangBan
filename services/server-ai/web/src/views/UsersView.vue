<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime, money } from '../format';
import type { BindingRow, Product } from '../types';
import BindingDrawer from '../components/BindingDrawer.vue';

const emit = defineEmits<{ changed: [] }>();

const rows = ref<BindingRow[]>([]);
const products = ref<Product[]>([]);
const total = ref(0);
const page = ref(1);
const q = ref('');
const productId = ref('');
const loading = ref(false);
const error = ref('');
const openId = ref<number | null>(null);

const pageSize = 20;
const pages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));
const productName = computed(() => (id: string) => products.value.find((p) => p.product_id === id)?.name ?? id);

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const result = await api.bindings({ q: q.value.trim(), productId: productId.value, page: page.value });
    rows.value = result.items;
    total.value = result.total;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

function search() {
  page.value = 1;
  load();
}

function go(next: number) {
  page.value = Math.min(Math.max(1, next), pages.value);
  load();
}

onMounted(async () => {
  api.products().then((r) => (products.value = r.items)).catch(() => {});
  load();
});

function drawerChanged() {
  load();
  emit('changed');
}
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>用户额度</h1>
        <p>每个账号在每个产品里有一份独立的余额。点开一行可以看额度明细，补发或扣回额度。</p>
      </div>
    </div>

    <form class="filters" role="search" @submit.prevent="search">
      <div class="field grow">
        <label for="user-q">查找用户</label>
        <input id="user-q" v-model="q" class="input" placeholder="输入账号 UID 或绑定编号" />
      </div>
      <div class="field">
        <label for="user-product">产品</label>
        <select id="user-product" v-model="productId" class="select" @change="search">
          <option value="">全部产品</option>
          <option v-for="p in products" :key="p.product_id" :value="p.product_id">{{ p.name }}</option>
        </select>
      </div>
      <button type="submit" class="btn primary">查找</button>
    </form>

    <div v-if="error" class="notice error" role="alert">{{ error }}</div>

    <div class="panel">
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr>
              <th>账号</th>
              <th>产品</th>
              <th class="right">可用余额</th>
              <th>最近使用</th>
              <th>状态</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="row.id" class="click" tabindex="0" @click="openId = row.id" @keydown.enter="openId = row.id">
              <td>
                <div class="mono uid">{{ row.uid }}</div>
                <div class="faint small">绑定 #{{ row.id }}</div>
              </td>
              <td>{{ productName(row.product_id) }}</td>
              <td class="right num">
                <strong :class="{ zero: row.balance <= 0 }">{{ money(row.balance) }}</strong>
              </td>
              <td class="num">{{ formatTime(row.last_used_at) }}</td>
              <td>
                <span v-if="row.status === 'active'" class="tag green">已开通</span>
                <span v-else class="tag">未开通</span>
              </td>
            </tr>
            <tr v-if="!loading && rows.length === 0">
              <td colspan="5" class="table-empty">
                <template v-if="q || productId">没有找到符合条件的用户。</template>
                <template v-else>还没有用户使用过 AI。用户第一次发起对话时，会自动出现在这里。</template>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="total > pageSize" class="pager">
      <span class="muted num">共 {{ total }} 条，第 {{ page }} / {{ pages }} 页</span>
      <div class="pager-btns">
        <button type="button" class="btn small" :disabled="page <= 1" @click="go(page - 1)">上一页</button>
        <button type="button" class="btn small" :disabled="page >= pages" @click="go(page + 1)">下一页</button>
      </div>
    </div>

    <BindingDrawer v-if="openId !== null" :id="openId" @close="openId = null" @changed="drawerChanged" />
  </div>
</template>

<style scoped>
.filters { display: flex; align-items: flex-end; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
.filters .grow { flex: 1; min-width: 220px; }
.filters .field:not(.grow) { width: 180px; }
.notice { margin-bottom: 14px; }
.uid { overflow-wrap: anywhere; }
.small { font-size: 12.5px; }
.zero { color: var(--faint); font-weight: 500; }
.pager { display: flex; align-items: center; justify-content: space-between; margin-top: 12px; gap: 12px; }
.pager-btns { display: flex; gap: 8px; }
</style>
