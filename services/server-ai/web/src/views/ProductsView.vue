<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { money } from '../format';
import type { ModelOption, Product, Tier } from '../types';
import ProductDialog from '../components/ProductDialog.vue';
import TierDialog from '../components/TierDialog.vue';

const emit = defineEmits<{ go: [string]; changed: [] }>();

const products = ref<Product[]>([]);
const selectedId = ref('');
const tiers = ref<Tier[]>([]);
const models = ref<ModelOption[]>([]);
const modelsError = ref('');
const loading = ref(true);
const error = ref('');
const notice = ref<{ kind: 'ok' | 'warn'; text: string } | null>(null);

const editingProduct = ref<Product | 'new' | null>(null);
const editingTier = ref<Tier | 'new' | null>(null);

const selected = computed(() => products.value.find((p) => p.product_id === selectedId.value) ?? null);
const warnModel = computed(() => new Set(models.value.filter((m) => m.ratio_warning).map((m) => m.model_name)));
const missingModel = computed(() => (name: string) => models.value.length > 0 && !models.value.some((m) => m.model_name === name));

async function loadProducts(keep?: string) {
  loading.value = true;
  error.value = '';
  try {
    products.value = (await api.products()).items;
    const first = products.value[0]?.product_id ?? '';
    selectedId.value = keep && products.value.some((p) => p.product_id === keep) ? keep : selectedId.value && products.value.some((p) => p.product_id === selectedId.value) ? selectedId.value : first;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

async function loadTiers() {
  tiers.value = [];
  if (!selectedId.value) return;
  try {
    tiers.value = (await api.tiers(selectedId.value)).items;
  } catch (e) {
    error.value = errorMessage(e);
  }
}

async function loadModels() {
  modelsError.value = '';
  try {
    models.value = (await api.models()).items;
  } catch (e) {
    models.value = [];
    modelsError.value = errorMessage(e);
  }
}

onMounted(async () => {
  await loadProducts();
  loadModels();
});
watch(selectedId, loadTiers, { immediate: true });

async function productSaved(id: string) {
  editingProduct.value = null;
  notice.value = { kind: 'ok', text: '产品已保存' };
  await loadProducts(id);
  emit('changed');
}

async function tierSaved(warnings: string[]) {
  editingTier.value = null;
  notice.value = warnings.length ? { kind: 'warn', text: `档位已保存，但请注意：${warnings.join('；')}` } : { kind: 'ok', text: '档位已保存，新请求立即生效' };
  await Promise.all([loadTiers(), loadModels()]);
}

async function removeTier(tier: Tier) {
  if (!window.confirm(`删除档位「${tier.display_name}」？客户端再请求这个档位会收到“未知的模型档位”。`)) return;
  try {
    await api.deleteTier(tier.product_id, tier.tier);
    notice.value = { kind: 'ok', text: '档位已删除' };
    await loadTiers();
  } catch (e) {
    error.value = errorMessage(e);
  }
}
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>产品与档位</h1>
        <p>用户在客户端只会看到档位名称，背后实际调用哪个模型由你在这里决定，随时可换，不用给客户端发版。</p>
      </div>
      <button type="button" class="btn primary" @click="editingProduct = 'new'">添加产品</button>
    </div>

    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="notice" class="notice" :class="notice.kind" role="status">{{ notice.text }}</div>

    <div v-if="!loading && products.length === 0" class="panel empty">
      <h2>还没有产品</h2>
      <p class="muted">先添加一个产品（例如小伴壁纸），并关联它的客户端，用户才能使用 AI。</p>
      <button type="button" class="btn primary" @click="editingProduct = 'new'">添加第一个产品</button>
    </div>

    <template v-if="selected">
      <div class="tabs" role="tablist" aria-label="产品">
        <button
          v-for="p in products"
          :key="p.product_id"
          type="button"
          role="tab"
          class="tab"
          :class="{ active: p.product_id === selectedId }"
          :aria-selected="p.product_id === selectedId"
          @click="selectedId = p.product_id"
        >
          {{ p.name }}<span v-if="!p.enabled" class="tag">已停用</span>
        </button>
      </div>

      <section class="product panel">
        <div class="product-main">
          <h2>{{ selected.name }}</h2>
          <dl>
            <div>
              <dt>关联的客户端</dt>
              <dd>
                <template v-if="selected.client_ids.length">
                  <span v-for="c in selected.client_ids" :key="c" class="tag mono">{{ c }}</span>
                </template>
                <span v-else class="muted">还没有关联客户端，用户暂时无法使用</span>
              </dd>
            </div>
            <div>
              <dt>新用户试用</dt>
              <dd class="num">{{ selected.trial_quota > 0 ? `${money(selected.trial_quota)}，${selected.trial_days} 天内有效` : '不送试用额度' }}</dd>
            </div>
          </dl>
        </div>
        <button type="button" class="btn" @click="editingProduct = selected">编辑产品</button>
      </section>

      <section class="section">
        <div class="section-head">
          <h2>档位</h2>
          <button type="button" class="btn small" @click="editingTier = 'new'">添加档位</button>
        </div>

        <div v-if="modelsError" class="notice warn">读不到 AI 网关的模型列表，暂时无法新增或修改档位：{{ modelsError }}</div>

        <div class="panel tier-board">
          <div class="board-head">
            <div class="col-front">用户看到的</div>
            <div class="col-back">实际调用的</div>
          </div>

          <div v-if="tiers.length === 0" class="board-empty">
            <p class="muted">这个产品还没有档位。至少添加一个不要求付费的默认档位，新用户才能直接使用。</p>
          </div>

          <div v-for="t in tiers" :key="t.tier" class="tier-row" :class="{ off: !t.enabled }">
            <div class="col-front">
              <div class="tier-name">
                <strong>{{ t.display_name }}</strong>
                <span v-if="t.is_default" class="tag green">默认</span>
                <span v-if="t.requires_paid" class="tag amber">需先兑换额度</span>
                <span v-if="!t.enabled" class="tag">已停用</span>
              </div>
              <div class="tier-sub"><span class="mono">{{ t.tier }}</span><span v-if="t.description"> · {{ t.description }}</span></div>
            </div>
            <div class="col-back">
              <div class="model-line">
                <span class="mono model">{{ t.model }}</span>
                <span v-if="warnModel.has(t.model)" class="tag amber" title="倍率与 New API 未配置时的默认值相同，可能还没设置真实定价">倍率可能未配置</span>
                <span v-else-if="missingModel(t.model)" class="tag red" title="AI 网关里已经找不到这个模型">模型已不可用</span>
              </div>
              <div class="row-actions">
                <button type="button" class="btn small" @click="editingTier = t">修改</button>
                <button type="button" class="btn small ghost" @click="removeTier(t)">删除</button>
              </div>
            </div>
          </div>
        </div>
        <p class="hint under">倍率决定每次调用扣多少额度。换模型前，先到 New API 确认新模型的倍率已经设置，否则可能按默认倍率多扣或少扣。</p>
      </section>
    </template>

    <ProductDialog v-if="editingProduct" :product="editingProduct === 'new' ? null : editingProduct" @close="editingProduct = null" @saved="productSaved" />
    <TierDialog
      v-if="editingTier && selected"
      :product="selected"
      :tier="editingTier === 'new' ? null : editingTier"
      :models="models"
      @close="editingTier = null"
      @saved="tierSaved"
    />
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.empty { padding: 36px 28px; display: flex; flex-direction: column; align-items: flex-start; gap: 10px; }

.tabs { display: flex; gap: 2px; border-bottom: 1px solid var(--line-strong); margin-bottom: 18px; overflow-x: auto; }
.tab { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; border: none; background: none; cursor: pointer; color: var(--muted); white-space: nowrap; }
.tab:hover { color: var(--ink); }
.tab.active { color: var(--primary); font-weight: 600; box-shadow: inset 0 -2px 0 var(--primary); }

.product { display: flex; justify-content: space-between; align-items: flex-start; gap: 20px; padding: 18px 20px; }
.product-main { min-width: 0; }
dl { margin: 12px 0 0; display: flex; flex-direction: column; gap: 10px; }
dl > div { display: grid; grid-template-columns: 96px 1fr; gap: 12px; align-items: baseline; }
dt { color: var(--muted); }
dd { margin: 0; display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }

.tier-board { overflow: hidden; }
.board-head, .tier-row { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); }
.board-head { font-size: 13px; color: var(--muted); font-weight: 500; }
.board-head > div { padding: 9px 20px; background: #fafbfc; border-bottom: 1px solid var(--line); }
.board-head .col-back { background: var(--curtain); border-left: 1px solid var(--line); }
.tier-row { border-bottom: 1px solid var(--line); }
.tier-row:last-child { border-bottom: none; }
.tier-row .col-front { padding: 14px 20px; min-width: 0; }
.tier-row .col-back { padding: 14px 20px; background: var(--curtain); border-left: 1px solid var(--line); display: flex; flex-direction: column; gap: 10px; min-width: 0; }
.tier-row.off .col-front strong, .tier-row.off .model { color: var(--faint); }
.tier-name { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; }
.tier-name strong { font-size: 15px; }
.tier-sub { margin-top: 3px; color: var(--muted); font-size: 13px; overflow-wrap: anywhere; }
.model-line { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; }
.model { overflow-wrap: anywhere; }
.row-actions { display: flex; gap: 6px; }
.board-empty { padding: 28px 20px; }
.under { margin-top: 8px; max-width: 70ch; }

@media (max-width: 640px) {
  .board-head { display: none; }
  .tier-row { grid-template-columns: 1fr; }
  .tier-row .col-back { border-left: none; border-top: 1px solid var(--line); }
  .tier-row .col-back::before { content: '实际调用（用户看不到）'; color: var(--muted); font-size: 12.5px; }
  .product { flex-direction: column; }
  dl > div { grid-template-columns: 1fr; gap: 2px; }
}
</style>
