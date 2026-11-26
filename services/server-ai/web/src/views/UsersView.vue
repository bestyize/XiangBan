<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { money, relativeTime, shortUid } from '../format';
import { notify } from '../toast';
import type { BindingRow, Product } from '../types';
import Avatar from '../components/Avatar.vue';
import CustomSelect from '../components/CustomSelect.vue';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';

const props = defineProps<{ initialQuery?: string; refreshTick?: number }>();
const emit = defineEmits<{ openBinding: [id: number]; consumeIntent: [] }>();

const rows = ref<BindingRow[]>([]);
const products = ref<Product[]>([]);
const total = ref(0);
const page = ref(1);
const q = ref('');
const productId = ref('');
const filter = ref<'' | 'funded' | 'empty' | 'anomaly'>('');
const sort = ref<'last_used' | 'balance' | 'created'>('last_used');
const loading = ref(true);
const error = ref('');

const pageSize = 20;
const pages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));
const productName = computed(() => (id: string) => products.value.find((p) => p.product_id === id)?.name ?? id);
const filtered = computed(() => !!q.value.trim() || !!productId.value || !!filter.value);

const productOptions = computed(() => [
  { value: '', label: '全部产品' },
  ...products.value.map((p) => ({ value: p.product_id, label: p.name })),
]);

const sortOptions = [
  { value: 'last_used', label: '按最近使用' },
  { value: 'balance', label: '按余额最高' },
  { value: 'created', label: '按最近开通' },
];

const filterTabs: readonly { key: '' | 'funded' | 'empty' | 'anomaly'; label: string }[] = [
  { key: '', label: '全部' },
  { key: 'funded', label: '有余额' },
  { key: 'empty', label: '已用完' },
  { key: 'anomaly', label: '对账异常/差额' },
];

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const result = await api.bindings({
      q: q.value.trim(),
      productId: productId.value,
      filter: filter.value,
      sort: sort.value,
      page: page.value,
    });
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

function setFilter(next: '' | 'funded' | 'empty' | 'anomaly') {
  filter.value = next;
  search();
}

function go(next: number) {
  page.value = Math.min(Math.max(1, next), pages.value);
  load();
}

async function copyUid(uid: string) {
  try {
    await navigator.clipboard.writeText(uid);
    notify('已复制完整 UID');
  } catch {
    notify('复制失败，请手动选中复制', 'warn');
  }
}

onMounted(() => {
  api.products().then((r) => (products.value = r.items)).catch(() => {});
  if (props.initialQuery) {
    q.value = props.initialQuery;
    emit('consumeIntent');
  }
  load();
});

watch(
  () => props.initialQuery,
  (val) => {
    if (val) {
      q.value = val;
      emit('consumeIntent');
      search();
    }
  },
);

watch(() => props.refreshTick, () => load());
watch([productId, sort], () => search());
</script>

<template>
  <div>
    <PageHead title="用户额度" desc="额度按「账号 × 产品」隔离。点击任意行滑出右侧账本抽屉，查看额度桶、流水或执行补发/扣回。">
      <span v-if="!loading" class="tag num">共 {{ total.toLocaleString('en-US') }} 个账号</span>
    </PageHead>

    <form class="toolbar" role="search" @submit.prevent="search">
      <div class="search-wrap grow">
        <Icon name="search" :size="15" />
        <label class="sr" for="user-q">查找用户</label>
        <input id="user-q" v-model="q" class="input" placeholder="输入账号 UID（支持前缀）或绑定编号" autocomplete="off" />
      </div>
      <div class="sel-prod">
        <CustomSelect v-model="productId" :options="productOptions" aria-label="按产品筛选" />
      </div>
      <div class="sel-sort">
        <CustomSelect v-model="sort" :options="sortOptions" aria-label="排序方式" />
      </div>
      <button type="submit" class="btn primary">查找</button>
    </form>

    <div class="filter-row">
      <div class="seg-pills" role="tablist" aria-label="按余额状态筛选">
        <button
          v-for="tab in filterTabs"
          :key="tab.key"
          type="button"
          role="tab"
          :class="{ active: filter === tab.key }"
          :aria-selected="filter === tab.key"
          @click="setFilter(tab.key)"
        >{{ tab.label }}</button>
      </div>
    </div>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <div class="table-card">
      <div class="table-scroll">
        <table class="table">
          <thead>
            <tr>
              <th>账号 / 绑定编号</th>
              <th>产品</th>
              <th class="right">可用余额</th>
              <th>对账同步</th>
              <th>最近使用</th>
              <th>状态</th>
            </tr>
          </thead>
          <tbody>
            <template v-if="loading && rows.length === 0">
              <tr v-for="n in 6" :key="n"><td colspan="6"><i class="skel" style="height: 34px"></i></td></tr>
            </template>
            <tr
              v-for="row in rows"
              :key="row.id"
              class="click"
              tabindex="0"
              @click="emit('openBinding', row.id)"
              @keydown.enter="emit('openBinding', row.id)"
            >
              <td>
                <div class="user-cell">
                  <Avatar :seed="row.uid" :size="32" />
                  <div>
                    <div class="uid-line">
                      <span class="mono id" :title="row.uid">{{ shortUid(row.uid) }}</span>
                      <button type="button" class="copy-mini" aria-label="复制完整 UID" title="复制完整 UID" @click.stop="copyUid(row.uid)">
                        <Icon name="copy" :size="12" />
                      </button>
                    </div>
                    <div class="faint sub">绑定 #{{ row.id }}</div>
                  </div>
                </div>
              </td>
              <td>{{ productName(row.product_id) }}</td>
              <td class="right"><Money :points="row.balance" size="sm" :muted="row.balance <= 0" compact-pts /></td>
              <td>
                <span v-if="row.open_anomalies && row.open_anomalies > 0" class="tag warn">
                  <Icon name="alert" :size="11" />{{ row.open_anomalies }} 条未处理异常
                </span>
                <span v-else-if="row.synced_quota !== row.balance" class="tag warn">
                  <Icon name="alert" :size="11" />差额 {{ money(row.synced_quota - row.balance) }}
                </span>
                <span v-else class="faint sub">已对齐 · {{ relativeTime(row.last_synced_at) }}</span>
              </td>
              <td class="muted nowrap" :title="row.last_used_at ?? ''">{{ relativeTime(row.last_used_at) }}</td>
              <td>
                <span v-if="row.status === 'active'" class="tag ok"><i class="dot"></i>已开通</span>
                <span v-else class="tag">未开通</span>
              </td>
            </tr>
            <tr v-if="!loading && rows.length === 0">
              <td colspan="6" class="nopad">
                <EmptyState
                  :icon="filtered ? 'search' : 'users'"
                  :title="filtered ? '没有找到符合条件的用户' : '还没有用户使用过 AI'"
                  :desc="filtered ? '试试切换筛选条件或搜索其他 UID。' : '用户第一次发起对话时，会自动出现在这里。'"
                />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="total > pageSize" class="pager">
      <span class="muted num">第 {{ page }} / {{ pages }} 页</span>
      <div class="row">
        <button type="button" class="btn sm" :disabled="page <= 1" @click="go(page - 1)"><Icon name="chevron-left" :size="14" />上一页</button>
        <button type="button" class="btn sm" :disabled="page >= pages" @click="go(page + 1)">下一页<Icon name="chevron-right" :size="14" /></button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.toolbar { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.toolbar .grow { min-width: 220px; }
.sel-prod { width: 200px; }
.sel-sort { width: 136px; }
.filter-row { margin-bottom: 14px; }
.notice { margin-bottom: 14px; }
.user-cell { display: flex; align-items: center; gap: 10px; }
.uid-line { display: flex; align-items: center; gap: 5px; }
.id { font-size: 12.5px; font-weight: 650; color: var(--ink); }
.sub { font-size: 11.5px; }
.nopad { padding: 0 !important; }
.pager { display: flex; align-items: center; justify-content: space-between; margin-top: 14px; gap: 12px; font-size: 12.5px; }

@media (max-width: 640px) {
  .sel-prod, .sel-sort { width: 100%; }
}
</style>
