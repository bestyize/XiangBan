<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { closedLabels, codeStatusLabels, formatDate, formatTime, kindLabels, money, relativeTime, xiaobanPoints } from '../format';
import { notify } from '../toast';
import type { Batch, CodeLookup, Product } from '../types';
import BatchDialog from '../components/BatchDialog.vue';
import CodesRevealDialog from '../components/CodesRevealDialog.vue';
import ConfirmDialog from '../components/ConfirmDialog.vue';
import CustomSelect from '../components/CustomSelect.vue';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';

const props = defineProps<{ initialQuery?: string; autoCreateBatch?: boolean; refreshTick?: number }>();
const emit = defineEmits<{ changed: []; go: [string]; openBinding: [id: number]; consumeIntent: [] }>();

const batches = ref<Batch[]>([]);
const products = ref<Product[]>([]);
const loading = ref(true);
const error = ref('');

const creating = ref(false);
const revealed = ref<{ batch: Batch; codes: string[] } | null>(null);

const query = ref('');
const looking = ref(false);
const found = ref<CodeLookup | null>(null);
const lookupError = ref('');
const acting = ref(false);

const batchProduct = ref('');
const batchFilter = ref<'all' | 'has_unused' | 'exhausted'>('all');

const confirmAction = ref<
  | { type: 'voidBatch'; batch: Batch }
  | { type: 'voidCode'; code: CodeLookup }
  | { type: 'clawback'; code: CodeLookup }
  | null
>(null);

const productName = computed(() => (id: string) => products.value.find((p) => p.product_id === id)?.name ?? id);
const productOptions = computed(() => [
  { value: '', label: '全部产品' },
  ...products.value.map((p) => ({ value: p.product_id, label: p.name })),
]);

const filteredBatches = computed(() => {
  return batches.value.filter((b) => {
    if (batchProduct.value && b.product_id !== batchProduct.value) return false;
    if (batchFilter.value === 'has_unused' && b.unused <= 0) return false;
    if (batchFilter.value === 'exhausted' && b.unused > 0) return false;
    return true;
  });
});

const totalUnused = computed(() => batches.value.reduce((s, b) => s + b.unused, 0));
const totalRedeemed = computed(() => batches.value.reduce((s, b) => s + b.redeemed, 0));

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const [b, p] = await Promise.all([api.batches(), api.products()]);
    batches.value = b.items;
    products.value = p.items;
    if (props.autoCreateBatch && products.value.length > 0) {
      creating.value = true;
      emit('consumeIntent');
    }
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

async function search() {
  const code = query.value.trim();
  if (!code) return;
  looking.value = true;
  lookupError.value = '';
  found.value = null;
  try {
    found.value = (await api.lookupCode(code)).code;
  } catch (e) {
    lookupError.value = errorMessage(e);
  } finally {
    looking.value = false;
  }
}

async function refreshFound() {
  if (found.value && query.value.trim()) {
    found.value = (await api.lookupCode(query.value.trim())).code;
  }
}

function clearLookup() {
  query.value = '';
  found.value = null;
  lookupError.value = '';
}

function created(batch: Batch, codes: string[]) {
  creating.value = false;
  revealed.value = { batch, codes };
  load();
  emit('changed');
}

async function handleConfirm(note: string) {
  const action = confirmAction.value;
  if (!action) return;
  acting.value = true;
  error.value = '';
  lookupError.value = '';
  try {
    if (action.type === 'voidBatch') {
      const result = await api.voidBatch(action.batch.id);
      confirmAction.value = null;
      notify(`已作废批次 #${action.batch.id} 剩余的 ${result.voided} 个码`);
      await load();
    } else if (action.type === 'voidCode') {
      await api.voidCode(action.code.id);
      confirmAction.value = null;
      notify('兑换码已作废');
      await Promise.all([refreshFound(), load()]);
    } else if (action.type === 'clawback' && action.code.bucket_id) {
      const reason = note || `兑换码 #${action.code.id} 退款`;
      const result = await api.voidBucket(action.code.bucket_id, reason);
      confirmAction.value = null;
      notify(`额度已扣回，用户减少 ${money(result.voided)}（${xiaobanPoints(result.voided)}）`);
      await refreshFound();
      emit('changed');
    }
  } catch (e) {
    const msg = errorMessage(e);
    if (action.type === 'voidBatch') error.value = msg;
    else lookupError.value = msg;
    confirmAction.value = null;
  } finally {
    acting.value = false;
  }
}

function share(count: number, b: Batch): string {
  return b.total > 0 ? `${(count / b.total) * 100}%` : '0%';
}

const statusTone = (status: string) => (status === 'unused' ? 'ok' : status === 'void' ? 'danger' : 'info');

onMounted(() => {
  load();
  if (props.initialQuery) {
    query.value = props.initialQuery;
    emit('consumeIntent');
    search();
  }
});

watch(
  () => props.initialQuery,
  (val) => {
    if (val) {
      query.value = val;
      emit('consumeIntent');
      search();
    }
  },
);

watch(
  () => props.autoCreateBatch,
  (val) => {
    if (val && products.value.length > 0) {
      creating.value = true;
      emit('consumeIntent');
    }
  },
);

watch(() => props.refreshTick, () => {
  load();
  refreshFound();
});
</script>

<template>
  <div>
    <PageHead title="兑换码" desc="面向淘宝卡密发售的批次管理与售后查码退款工作台。明文仅在生成时展示一次，数据库仅存 HMAC 摘要。">
      <button type="button" class="btn primary" :disabled="products.length === 0" @click="creating = true">
        <Icon name="plus" :size="14" />生成兑换码
      </button>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <!-- 查码与售后工作台 -->
    <section class="card lookup-card" aria-label="查询一个码">
      <div>
        <h2 class="sub-h">查询兑换码（客服与退款）</h2>
        <p class="hint">粘贴买家提供的 16 位兑换码，查看归属账号与剩余额度，并可直接作废、扣回或打开用户账本。</p>
      </div>

      <form class="lookup-row" role="search" @submit.prevent="search">
        <label class="sr" for="code-q">兑换码</label>
        <input
          id="code-q"
          v-model="query"
          class="input lookup-input"
          placeholder="XXXX-XXXX-XXXX-XXXX"
          autocomplete="off"
          spellcheck="false"
        />
        <button type="submit" class="btn primary lg" :disabled="looking || !query.trim()">
          <Icon name="search" :size="15" />{{ looking ? '查询中…' : '查询状态' }}
        </button>
        <button v-if="found || lookupError" type="button" class="btn ghost" @click="clearLookup">清空</button>
      </form>

      <div v-if="lookupError" class="notice error" role="alert">
        <Icon name="circle-alert" :size="16" /><span>{{ lookupError }}</span>
      </div>

      <div v-if="found" class="lookup-result">
        <div class="lr-top">
          <span class="tag" :class="statusTone(found.status)">{{ codeStatusLabels[found.status] }}</span>
          <Money :points="found.quota" size="md" />
          <span class="tag" :class="found.kind === 'monthly' ? 'warn' : 'ok'">{{ kindLabels[found.kind] }}</span>
          <span class="muted">{{ productName(found.product_id) }} · 批次 #{{ found.batch_id }}<template v-if="found.note">（{{ found.note }}）</template></span>
        </div>

        <div class="lr-grid">
          <template v-if="found.status === 'redeemed'">
            <div class="lr-kv"><span>兑换账号 UID</span><strong class="mono">{{ found.redeemed_by }}</strong></div>
            <div class="lr-kv"><span>兑换时间</span><strong class="num">{{ formatTime(found.redeemed_at) }}（{{ relativeTime(found.redeemed_at) }}）</strong></div>
            <div class="lr-kv">
              <span>对应额度桶状态</span>
              <strong class="num">
                <template v-if="found.bucket_closed_reason">{{ closedLabels[found.bucket_closed_reason] ?? '已结束' }}</template>
                <template v-else>还剩 {{ money(found.bucket_remaining ?? 0) }}（{{ xiaobanPoints(found.bucket_remaining ?? 0) }}），{{ formatDate(found.bucket_expires_at) }} 到期</template>
              </strong>
            </div>
          </template>
          <div v-else-if="found.status === 'void'" class="lr-kv">
            <span>作废时间</span><strong class="num">{{ formatTime(found.voided_at) }}</strong>
          </div>
          <div v-else class="lr-kv">
            <span>有效期规则</span><strong>用户兑换后 {{ found.valid_days }} 天内有效</strong>
          </div>
        </div>

        <div class="lr-acts">
          <button v-if="found.binding_id" type="button" class="btn sm" @click="emit('openBinding', found.binding_id)">
            <Icon name="arrow-up-right" :size="13" />查看用户 #{{ found.binding_id }} 完整账本
          </button>
          <button v-if="found.status === 'unused'" type="button" class="btn sm danger" :disabled="acting" @click="confirmAction = { type: 'voidCode', code: found }">
            <Icon name="x" :size="13" />作废这个码
          </button>
          <button
            v-if="found.status === 'redeemed' && !found.bucket_closed_reason && found.bucket_id"
            type="button"
            class="btn sm danger"
            :disabled="acting"
            @click="confirmAction = { type: 'clawback', code: found }"
          >
            <Icon name="undo" :size="13" />退款并扣回剩余额度
          </button>
          <span v-if="found.status === 'redeemed' && found.bucket_closed_reason" class="hint">额度桶已结束，无可扣回余额。</span>
        </div>
      </div>
    </section>

    <!-- 批次库存 -->
    <section class="section">
      <div class="section-head">
        <h2>
          <span>已生成批次</span>
          <span v-if="batches.length" class="tag num">待售 {{ totalUnused.toLocaleString('en-US') }}</span>
          <span v-if="batches.length" class="tag num">已兑 {{ totalRedeemed.toLocaleString('en-US') }}</span>
        </h2>
        <div v-if="batches.length" class="batch-filters">
          <div v-if="products.length > 1" style="width: 180px">
            <CustomSelect v-model="batchProduct" :options="productOptions" size="sm" aria-label="按产品筛选批次" />
          </div>
          <div class="seg-pills">
            <button type="button" :class="{ active: batchFilter === 'all' }" @click="batchFilter = 'all'">全部</button>
            <button type="button" :class="{ active: batchFilter === 'has_unused' }" @click="batchFilter = 'has_unused'">仍有待售</button>
            <button type="button" :class="{ active: batchFilter === 'exhausted' }" @click="batchFilter = 'exhausted'">已兑完</button>
          </div>
        </div>
      </div>

      <div class="table-card">
        <div class="table-scroll">
          <table class="table">
            <thead>
              <tr>
                <th>批次 / 商品备注</th>
                <th>产品</th>
                <th>面额与类型</th>
                <th>库存与兑换进度</th>
                <th class="right">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="b in filteredBatches" :key="b.id">
                <td>
                  <div class="bn num"><strong>#{{ b.id }}</strong><span class="faint">{{ relativeTime(b.created_at) }} · {{ b.created_by }}</span></div>
                  <div v-if="b.note" class="note">{{ b.note }}</div>
                </td>
                <td>{{ productName(b.product_id) }}</td>
                <td>
                  <div class="what">
                    <span class="tag" :class="b.kind === 'monthly' ? 'warn' : 'ok'">{{ kindLabels[b.kind] }}</span>
                    <Money :points="b.quota" size="sm" compact-pts />
                  </div>
                  <div class="faint small">兑换后 {{ b.valid_days }} 天有效 · 共 {{ b.total }} 个</div>
                </td>
                <td class="prog">
                  <div class="bar" role="img" :aria-label="`待售 ${b.unused}，已兑换 ${b.redeemed}，作废 ${b.void}`">
                    <i class="r" :style="{ width: share(b.redeemed, b) }"></i>
                    <i class="u" :style="{ width: share(b.unused, b) }"></i>
                    <i class="v" :style="{ width: share(b.void, b) }"></i>
                  </div>
                  <div class="legend num">
                    <span><b>{{ b.unused }}</b> 待售</span>
                    <span><b>{{ b.redeemed }}</b> 已兑</span>
                    <span v-if="b.void"><b>{{ b.void }}</b> 作废</span>
                  </div>
                </td>
                <td class="right">
                  <button
                    v-if="b.unused > 0"
                    type="button"
                    class="btn sm ghost danger-text"
                    @click="confirmAction = { type: 'voidBatch', batch: b }"
                  >作废剩余 {{ b.unused }} 个</button>
                  <span v-else class="faint small">已清空</span>
                </td>
              </tr>
              <tr v-if="loading && batches.length === 0">
                <td colspan="5"><i class="skel" style="height: 40px"></i></td>
              </tr>
              <tr v-if="!loading && filteredBatches.length === 0">
                <td colspan="5" class="nopad">
                  <EmptyState
                    icon="ticket"
                    :title="products.length === 0 ? '先添加一个产品' : batches.length ? '当前筛选下没有批次' : '还没有生成过兑换码'"
                    :desc="
                      products.length === 0
                        ? '兑换码需绑定到具体产品，请先到「产品与档位」创建产品。'
                        : batches.length
                          ? '可切换产品或状态筛选查看全部批次。'
                          : '点击右上角「生成兑换码」，导出 .txt 导入淘宝自动发货库存即可开售。'
                    "
                  >
                    <button v-if="products.length === 0" type="button" class="btn primary" @click="emit('go', 'products')">去添加产品</button>
                  </EmptyState>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>

    <BatchDialog v-if="creating" :products="products" @close="creating = false" @created="created" />
    <CodesRevealDialog v-if="revealed" :batch="revealed.batch" :codes="revealed.codes" @close="revealed = null" />
    <ConfirmDialog
      v-if="confirmAction"
      :title="
        confirmAction.type === 'voidBatch'
          ? `作废批次 #${confirmAction.batch.id} 剩余的 ${confirmAction.batch.unused} 个码？`
          : confirmAction.type === 'voidCode'
            ? '作废这个兑换码？'
            : '退款并扣回这份额度？'
      "
      :desc="
        confirmAction.type === 'voidBatch'
          ? '已经兑换的码和用户余额不受影响；未兑换的码作废后无法恢复。'
          : confirmAction.type === 'voidCode'
            ? '作废后无人能再兑换该码，此操作不可恢复。'
            : `将扣回该码对应额度桶里剩余的 ${money(confirmAction.code.bucket_remaining ?? 0)}（${xiaobanPoints(confirmAction.code.bucket_remaining ?? 0)}），已消耗部分不退回，此操作不可撤销。`
      "
      :confirm-text="confirmAction.type === 'clawback' ? '确认扣回' : '确认作废'"
      :with-note="confirmAction.type === 'clawback'"
      :note-placeholder="confirmAction.type === 'clawback' ? `兑换码 #${confirmAction.code.id} 退款` : ''"
      :busy="acting"
      @cancel="confirmAction = null"
      @confirm="handleConfirm"
    />
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.lookup-card { padding: 18px 20px; display: flex; flex-direction: column; gap: 14px; }
.sub-h { font-size: 15px; font-weight: 680; margin-bottom: 3px; }
.lookup-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.lookup-input { flex: 1; min-width: 240px; max-width: 440px; height: 40px; font-family: var(--mono); font-size: 14.5px; letter-spacing: 0.06em; }

.lookup-result {
  padding: 15px 16px;
  border-radius: var(--r-md);
  border: 1px solid var(--line-strong);
  background: var(--surface-muted);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.lr-top { display: flex; align-items: center; gap: 9px; flex-wrap: wrap; }
.lr-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px 16px; }
.lr-kv { display: flex; flex-direction: column; gap: 2px; }
.lr-kv span { font-size: 11.5px; color: var(--ink-3); }
.lr-kv strong { font-size: 13px; overflow-wrap: anywhere; }
.lr-acts { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; padding-top: 8px; border-top: 1px solid var(--line); }

.batch-filters { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.bn { display: flex; align-items: baseline; gap: 8px; }
.bn span { font-size: 12px; }
.note { color: var(--ink-2); font-size: 12px; margin-top: 2px; max-width: 34ch; overflow-wrap: anywhere; }
.what { display: flex; align-items: center; gap: 7px; }
.small { font-size: 11.5px; margin-top: 3px; }
.prog { min-width: 190px; }
.bar { display: flex; height: 6px; border-radius: 99px; overflow: hidden; background: var(--canvas-soft); gap: 1.5px; }
.bar i { display: block; height: 100%; }
.bar .r { background: var(--ink); }
.bar .u { background: var(--ok); opacity: 0.6; }
.bar .v { background: var(--ink-4); opacity: 0.4; }
.legend { display: flex; gap: 12px; margin-top: 6px; font-size: 11.5px; color: var(--ink-3); flex-wrap: wrap; }
.legend b { color: var(--ink); }
.danger-text { color: var(--danger); }
.nopad { padding: 0 !important; }

@media (max-width: 720px) {
  .lr-grid { grid-template-columns: 1fr; }
}
</style>
