<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { relativeTime, shortUid } from '../format';
import type { BindingRow, Product } from '../types';
import Avatar from '../components/Avatar.vue';
import BindingDrawer from '../components/BindingDrawer.vue';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';

const emit = defineEmits<{ changed: [] }>();

const rows = ref<BindingRow[]>([]);
const products = ref<Product[]>([]);
const total = ref(0);
const page = ref(1);
const q = ref('');
const productId = ref('');
const loading = ref(true);
const error = ref('');
const openId = ref<number | null>(null);

const pageSize = 20;
const pages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));
const productName = computed(() => (id: string) => products.value.find((p) => p.product_id === id)?.name ?? id);
const filtered = computed(() => !!q.value.trim() || !!productId.value);

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

onMounted(() => {
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
    <PageHead title="用户额度" desc="每个账号在每个产品里有一份独立的余额。点开一行看额度明细，补发或扣回额度。">
      <span v-if="!loading" class="muted num total">共 {{ total }} 个账号</span>
    </PageHead>

    <form class="tools" role="search" @submit.prevent="search">
      <div class="search-wrap grow">
        <Icon name="search" :size="16" />
        <label class="sr" for="user-q">查找用户</label>
        <input id="user-q" v-model="q" class="input" placeholder="输入账号 UID 或绑定编号" autocomplete="off" />
      </div>
      <div class="sel">
        <label class="sr" for="user-product">产品</label>
        <select id="user-product" v-model="productId" class="select" @change="search">
          <option value="">全部产品</option>
          <option v-for="p in products" :key="p.product_id" :value="p.product_id">{{ p.name }}</option>
        </select>
      </div>
      <button type="submit" class="btn primary">查找</button>
    </form>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr><th>账号</th><th>产品</th><th class="right">可用余额</th><th>最近使用</th><th>状态</th></tr>
        </thead>
        <tbody>
          <template v-if="loading && rows.length === 0">
            <tr v-for="n in 6" :key="n"><td colspan="5"><i class="skel" style="display: block; height: 34px"></i></td></tr>
          </template>
          <tr v-for="row in rows" :key="row.id" class="click" tabindex="0" @click="openId = row.id" @keydown.enter="openId = row.id">
            <td>
              <div class="who">
                <Avatar :seed="row.uid" :size="34" />
                <div>
                  <div class="mono id">{{ shortUid(row.uid) }}</div>
                  <div class="faint small">绑定 #{{ row.id }}</div>
                </div>
              </div>
            </td>
            <td>{{ productName(row.product_id) }}</td>
            <td class="right"><Money :points="row.balance" size="md" :muted="row.balance <= 0" /></td>
            <td class="muted nowrap" :title="row.last_used_at ?? ''">{{ relativeTime(row.last_used_at) }}</td>
            <td>
              <span v-if="row.status === 'active'" class="tag green">已开通</span>
              <span v-else class="tag">未开通</span>
            </td>
          </tr>
          <tr v-if="!loading && rows.length === 0">
            <td colspan="5" class="nopad">
              <EmptyState
                :icon="filtered ? 'search' : 'users'"
                :title="filtered ? '没有找到符合条件的用户' : '还没有用户使用过 AI'"
                :desc="filtered ? '换个 UID 或产品再试试。' : '用户第一次发起对话时，会自动出现在这里。'"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="total > pageSize" class="pager">
      <span class="muted num">第 {{ page }} / {{ pages }} 页</span>
      <div class="row">
        <button type="button" class="btn small" :disabled="page <= 1" @click="go(page - 1)"><Icon name="chevron-left" :size="15" />上一页</button>
        <button type="button" class="btn small" :disabled="page >= pages" @click="go(page + 1)">下一页<Icon name="chevron-right" :size="15" /></button>
      </div>
    </div>

    <BindingDrawer v-if="openId !== null" :id="openId" @close="openId = null" @changed="drawerChanged" />
  </div>
</template>

<style scoped>
.total { padding-bottom: 4px; }
.tools { display: flex; align-items: center; gap: 10px; margin-bottom: 16px; flex-wrap: wrap; }
.tools .grow { min-width: 240px; }
.sel { width: 180px; }
.notice { margin-bottom: 14px; }
.who { display: flex; align-items: center; gap: 12px; }
.id { font-size: 13px; font-weight: 600; color: var(--text); }
.small { font-size: 12px; }
.nopad { padding: 0 !important; }
.pager { display: flex; align-items: center; justify-content: space-between; margin-top: 14px; gap: 12px; }

@media (max-width: 560px) {
  .sel { width: 100%; }
}
</style>
