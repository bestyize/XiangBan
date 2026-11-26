<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { formatTokens } from '../format';
import { CAPABILITIES, type CapabilityInfo } from '../media';
import { notify } from '../toast';
import type { MediaSetting, ModelOption, Product, Tier } from '../types';
import ConfirmDialog from '../components/ConfirmDialog.vue';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import MediaDialog from '../components/MediaDialog.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';
import ProductDialog from '../components/ProductDialog.vue';
import TierDialog from '../components/TierDialog.vue';

const props = defineProps<{ initialProductId?: string }>();
const emit = defineEmits<{ changed: []; selectProduct: [productId: string] }>();

const products = ref<Product[]>([]);
const selectedId = ref('');
const tiers = ref<Tier[]>([]);
const media = ref<MediaSetting[]>([]);
const models = ref<ModelOption[]>([]);
const modelsError = ref('');
const loading = ref(true);
const error = ref('');

const editingProduct = ref<Product | 'new' | null>(null);
const editingTier = ref<Tier | 'new' | null>(null);
const editingMedia = ref<CapabilityInfo | null>(null);
const confirmingDeleteTier = ref<Tier | null>(null);
const confirmingDeleteMedia = ref<CapabilityInfo | null>(null);
const deleting = ref(false);

const selected = computed(() => products.value.find((p) => p.product_id === selectedId.value) ?? null);
const modelInfo = computed(() => new Map(models.value.map((m) => [m.model_name, m])));
const mediaFor = (id: string) => media.value.find((m) => m.capability === id) ?? null;

function pickProduct(id: string) {
  selectedId.value = id;
  emit('selectProduct', id);
}

async function loadProducts(keep?: string) {
  loading.value = true;
  error.value = '';
  try {
    products.value = (await api.products()).items;
    const has = (id?: string) => !!id && products.value.some((p) => p.product_id === id);
    const next =
      (keep && has(keep) ? keep : null) ??
      (has(props.initialProductId) ? props.initialProductId! : null) ??
      (has(selectedId.value) ? selectedId.value : null) ??
      products.value[0]?.product_id ??
      '';
    pickProduct(next);
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

async function loadMedia() {
  media.value = [];
  if (!selectedId.value) return;
  try {
    media.value = (await api.media(selectedId.value)).items;
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

watch(
  () => props.initialProductId,
  (id) => {
    if (id && products.value.some((p) => p.product_id === id) && id !== selectedId.value) {
      selectedId.value = id;
    }
  },
);

watch(selectedId, () => Promise.all([loadTiers(), loadMedia()]), { immediate: true });

async function productSaved(id: string) {
  editingProduct.value = null;
  notify('产品配置已保存');
  await loadProducts(id);
  emit('changed');
}

async function tierSaved(warnings: string[]) {
  editingTier.value = null;
  if (warnings.length) notify(`档位已保存，注意：${warnings.join('；')}`, 'warn');
  else notify('档位已保存，立即生效');
  await Promise.all([loadTiers(), loadModels()]);
  emit('changed');
}

async function confirmRemoveTier() {
  const tier = confirmingDeleteTier.value;
  if (!tier) return;
  deleting.value = true;
  error.value = '';
  try {
    await api.deleteTier(tier.product_id, tier.tier);
    confirmingDeleteTier.value = null;
    notify('档位已删除');
    await loadTiers();
    emit('changed');
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    deleting.value = false;
  }
}

async function mediaSaved(warnings: string[]) {
  editingMedia.value = null;
  if (warnings.length) notify(`媒体能力已保存，注意：${warnings.join('；')}`, 'warn');
  else notify('媒体能力已保存，立即生效');
  await Promise.all([loadMedia(), loadModels()]);
}

async function confirmRemoveMedia() {
  const cap = confirmingDeleteMedia.value;
  if (!cap || !selectedId.value) return;
  deleting.value = true;
  error.value = '';
  try {
    await api.deleteMedia(selectedId.value, cap.id);
    confirmingDeleteMedia.value = null;
    notify(`${cap.label}已关闭`);
    await loadMedia();
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    deleting.value = false;
  }
}

function reasoningLabel(t: Tier): string {
  if (t.reasoning_efforts == null) return '思考: 自动';
  if (t.reasoning_efforts.length === 0) return '思考: 关';
  return `思考: ${t.reasoning_efforts.join(' / ')}`;
}

function supportsGif(m: MediaSetting | null): boolean {
  return !!m && m.capability === 'image' && /^gpt-image-2\.6-(sunburst|flare)$/.test(m.model);
}

const capEndpoints: Record<string, string> = {
  speech: '/v1/audio/speech',
  image: '/v1/images/generations',
  transcribe: '/v1/audio/transcriptions',
  realtime: '/v1/realtime (WS)',
};
</script>

<template>
  <div>
    <PageHead title="产品与档位">
      <button v-if="selected" type="button" class="btn outline" @click="editingProduct = selected"><Icon name="pencil" :size="14" />编辑产品</button>
      <button type="button" class="btn primary" @click="editingProduct = 'new'"><Icon name="plus" :size="14" />添加产品</button>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <div v-if="!loading && products.length === 0" class="card">
      <EmptyState icon="layers" title="还没有产品" desc="先添加一个产品并关联它的账号中心客户端标识，用户才能使用 AI。">
        <button type="button" class="btn primary" @click="editingProduct = 'new'"><Icon name="plus" :size="14" />添加产品</button>
      </EmptyState>
    </div>

    <template v-if="selected">
      <div v-if="products.length > 1" class="tabs-row">
        <div class="seg-pills" role="tablist" aria-label="切换产品">
          <button
            v-for="p in products"
            :key="p.product_id"
            type="button"
            role="tab"
            :class="{ active: p.product_id === selectedId }"
            :aria-selected="p.product_id === selectedId"
            @click="pickProduct(p.product_id)"
          >
            <span>{{ p.name }}</span>
            <span v-if="!p.enabled" class="cnt">停用</span>
          </button>
        </div>
      </div>

      <section class="card product-summary">
        <div class="ps-main">
          <div class="ps-title">
            <h2>{{ selected.name }}</h2>
            <span class="tag mono">{{ selected.product_id }}</span>
            <span class="tag"><i class="dot" :class="selected.enabled ? 'ok' : ''"></i>{{ selected.enabled ? '运行中' : '已停用' }}</span>
          </div>
          <div class="client-chips">
            <span v-for="c in selected.client_ids" :key="c" class="tag mono">{{ c }}</span>
            <span v-if="!selected.client_ids.length" class="hint">尚未关联客户端</span>
          </div>
        </div>

        <div class="ps-meta-grid">
          <div class="ps-meta">
            <span class="k">新用户试用</span>
            <div class="v">
              <template v-if="selected.trial_quota > 0">
                <Money :points="selected.trial_quota" size="sm" compact-pts />
                <span class="faint" style="font-weight: 500">· {{ selected.trial_days }} 天</span>
              </template>
              <span v-else class="faint">不发放</span>
            </div>
          </div>
          <div class="ps-meta">
            <span class="k">获取额度链接</span>
            <div class="v">
              <a
                v-if="selected.credit_purchase_url"
                class="ps-link"
                :href="selected.credit_purchase_url"
                target="_blank"
                rel="noopener noreferrer"
                :title="selected.credit_purchase_url"
              >
                {{ selected.credit_purchase_url.replace(/^https?:\/\//, '') }}
              </a>
              <span v-else class="faint">未配置</span>
            </div>
          </div>
        </div>
      </section>

      <div v-if="modelsError" class="notice warn">
        <Icon name="alert" :size="16" />
        <span>读不到 AI 网关模型列表，暂时无法修改档位与媒体能力：{{ modelsError }}</span>
      </div>

      <div class="section-head">
        <h2>对话档位 <span class="tag num">{{ tiers.length }}</span></h2>
        <button type="button" class="btn sm outline" :disabled="Boolean(modelsError)" @click="editingTier = 'new'">
          <Icon name="plus" :size="13" />添加档位
        </button>
      </div>

      <section class="card route-board" aria-label="档位与实际模型的路由关系">
        <div v-if="tiers.length" class="route-header">
          <span>客户端展示</span>
          <span></span>
          <span>真实模型与倍率</span>
        </div>

        <div v-for="t in tiers" :key="t.tier" class="route-row" :class="{ disabled: !t.enabled }">
          <div class="tier-client-card" :class="{ 'is-default': t.is_default }">
            <span class="tier-radio" aria-hidden="true"></span>
            <div class="tier-c-body">
              <div class="tier-c-top">
                <strong>{{ t.display_name }}</strong>
                <span v-if="t.is_default" class="tag brand">默认</span>
                <span class="tag"><i class="dot" :class="t.requires_paid ? 'warn' : 'ok'"></i>{{ t.requires_paid ? '需付费' : '试用可用' }}</span>
                <span v-if="!t.enabled" class="tag">已停用</span>
              </div>
              <div v-if="t.description" class="hint">{{ t.description }}</div>
              <div class="tier-c-foot">
                <span class="tag mono">{{ t.tier }}</span>
                <span class="tag">{{ reasoningLabel(t) }}</span>
              </div>
            </div>
          </div>

          <div class="route-arrow" aria-hidden="true"><Icon name="arrow-right" :size="14" /></div>

          <div class="tier-upstream">
            <div class="tier-up-body">
              <div class="tier-model-name">{{ t.model }}</div>
              <div class="tier-up-tags">
                <template v-if="modelInfo.get(t.model)">
                  <span class="tag outlined num">输入 {{ modelInfo.get(t.model)!.model_ratio }}</span>
                  <span class="tag outlined num">输出 ×{{ modelInfo.get(t.model)!.completion_ratio }}</span>
                  <span v-if="t.capabilities?.context_window_tokens" class="tag outlined num">{{ formatTokens(t.capabilities.context_window_tokens) }}</span>
                  <span v-if="t.capabilities?.max_output_tokens" class="tag outlined num">输出 {{ formatTokens(t.capabilities.max_output_tokens) }}</span>
                  <span v-if="modelInfo.get(t.model)!.ratio_warning" class="tag outlined" title="倍率与 New API 未配置默认值一致">
                    <i class="dot warn"></i>默认倍率
                  </span>
                </template>
                <span v-else-if="models.length" class="tag outlined" title="AI 网关里找不到这个模型">
                  <i class="dot danger"></i>模型不可用
                </span>
              </div>
            </div>
            <div class="tier-up-acts">
              <button type="button" class="btn icon sm ghost" :disabled="Boolean(modelsError)" :aria-label="`编辑档位 ${t.display_name}`" title="编辑" @click="editingTier = t">
                <Icon name="pencil" :size="14" />
              </button>
              <button type="button" class="btn icon sm ghost" :aria-label="`删除档位 ${t.display_name}`" title="删除" @click="confirmingDeleteTier = t">
                <Icon name="trash" :size="14" />
              </button>
            </div>
          </div>
        </div>

        <EmptyState v-if="!tiers.length" icon="layers" title="还没有档位" desc="至少添加一个试用可用的默认档位，新用户才能直接使用。">
          <button type="button" class="btn primary" :disabled="Boolean(modelsError)" @click="editingTier = 'new'">
            <Icon name="plus" :size="14" />添加档位
          </button>
        </EmptyState>
      </section>

      <section class="section" aria-labelledby="media-heading">
        <div class="section-head">
          <h2 id="media-heading">媒体能力</h2>
        </div>

        <div class="media-grid">
          <article
            v-for="c in CAPABILITIES"
            :key="c.id"
            class="card media-card"
            :class="mediaFor(c.id)?.enabled ? 'on' : 'off'"
          >
            <div class="mc-top">
              <span class="mc-ico"><Icon :name="c.icon" :size="16" /></span>
              <div class="mc-title">
                <strong>{{ c.label }}</strong>
                <span class="mono">{{ capEndpoints[c.id] }}</span>
              </div>
              <div class="mc-badges">
                <span class="tag">
                  <i class="dot" :class="mediaFor(c.id)?.enabled ? 'ok' : ''"></i>
                  {{ !mediaFor(c.id) ? '未开放' : mediaFor(c.id)!.enabled ? '已开放' : '已停用' }}
                </span>
                <span v-if="mediaFor(c.id)?.requires_paid" class="tag"><i class="dot warn"></i>需付费</span>
                <span v-if="supportsGif(mediaFor(c.id))" class="tag"><i class="dot brand"></i>GIF</span>
              </div>
            </div>

            <div v-if="mediaFor(c.id)" class="mc-model-box">
              <div class="mc-model-row">
                <span class="mono mc-model-name">{{ mediaFor(c.id)!.model }}</span>
                <template v-if="modelInfo.get(mediaFor(c.id)!.model)">
                  <span v-if="modelInfo.get(mediaFor(c.id)!.model)!.ratio_warning" class="tag outlined"><i class="dot warn"></i>默认倍率</span>
                  <span v-else class="tag outlined num">倍率 {{ modelInfo.get(mediaFor(c.id)!.model)!.model_ratio }}</span>
                </template>
                <span v-else-if="models.length" class="tag outlined"><i class="dot danger"></i>不可用</span>
              </div>

              <div v-if="c.id === 'speech' && mediaFor(c.id)!.voices.length" class="mc-chips">
                <span v-for="(v, i) in mediaFor(c.id)!.voices" :key="v" class="mc-chip" :class="{ def: i === 0 }">{{ v }}</span>
              </div>
              <div v-if="c.id === 'image' && mediaFor(c.id)!.sizes.length" class="mc-chips">
                <span v-for="(s, i) in mediaFor(c.id)!.sizes" :key="s" class="mc-chip" :class="{ def: i === 0 }">{{ s }}</span>
              </div>
              <div v-if="c.id === 'realtime'" class="hint">单次最长 {{ Math.round(mediaFor(c.id)!.max_secs / 60) }} 分钟</div>
            </div>
            <p v-else class="hint">{{ c.usedBy }}</p>

            <div class="mc-foot">
              <button type="button" class="btn sm" :class="{ primary: !mediaFor(c.id) }" :disabled="Boolean(modelsError)" @click="editingMedia = c">
                <Icon :name="mediaFor(c.id) ? 'pencil' : 'plus'" :size="13" />{{ mediaFor(c.id) ? '配置' : '开放' }}
              </button>
              <button v-if="mediaFor(c.id)" type="button" class="btn sm ghost danger-text" @click="confirmingDeleteMedia = c">关闭</button>
            </div>
          </article>
        </div>
      </section>
    </template>

    <ProductDialog
      v-if="editingProduct"
      :product="editingProduct === 'new' ? null : editingProduct"
      @close="editingProduct = null"
      @saved="productSaved"
    />
    <TierDialog
      v-if="editingTier && selected"
      :product="selected"
      :tier="editingTier === 'new' ? null : editingTier"
      :models="models"
      @close="editingTier = null"
      @saved="tierSaved"
    />
    <MediaDialog
      v-if="editingMedia && selected"
      :product="selected"
      :capability="editingMedia"
      :setting="mediaFor(editingMedia.id)"
      :models="models"
      @close="editingMedia = null"
      @saved="mediaSaved"
    />
    <ConfirmDialog
      v-if="confirmingDeleteTier"
      :title="`删除档位「${confirmingDeleteTier.display_name}」？`"
      desc="删除后，客户端请求该档位标识会收到错误。若线上仍有流量，建议先停用观察。"
      confirm-text="删除"
      :busy="deleting"
      @cancel="confirmingDeleteTier = null"
      @confirm="confirmRemoveTier"
    />
    <ConfirmDialog
      v-if="confirmingDeleteMedia"
      :title="`关闭「${confirmingDeleteMedia.label}」？`"
      desc="关闭后配置将被移除，客户端调用该能力会收到 503 暂未开放。"
      confirm-text="关闭"
      :busy="deleting"
      @cancel="confirmingDeleteMedia = null"
      @confirm="confirmRemoveMedia"
    />
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.tabs-row { margin-bottom: 14px; }

.product-summary {
  padding: 16px 18px;
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}
.ps-main { flex: 1; min-width: 240px; display: flex; flex-direction: column; gap: 8px; }
.ps-title { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.ps-title h2 { font-size: 16px; font-weight: 680; }
.client-chips { display: flex; flex-wrap: wrap; gap: 6px; }
.ps-meta-grid { display: flex; align-items: center; gap: 24px; flex-wrap: wrap; }
.ps-meta { display: flex; flex-direction: column; gap: 2px; }
.ps-meta .k { font-size: 11.5px; color: var(--ink-3); }
.ps-meta .v { font-size: 13px; font-weight: 600; display: flex; align-items: center; gap: 6px; }
.ps-link { max-width: 230px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.route-board { overflow: hidden; }
.route-header {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 40px minmax(0, 1.15fr);
  padding: 10px 16px;
  border-bottom: 1px solid var(--line);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-3);
}

.route-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 40px minmax(0, 1.15fr);
  padding: 14px 16px;
  border-bottom: 1px solid var(--line);
  align-items: center;
}
.route-row:last-child { border-bottom: 0; }
.route-row.disabled { opacity: 0.56; }

.tier-client-card {
  padding: 12px 14px;
  border-radius: var(--r-md);
  border: 1px solid var(--line);
  background: var(--surface);
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.tier-client-card.is-default { border-color: var(--brand); }
.tier-radio {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  margin-top: 2px;
  flex: none;
  border: 1.5px solid var(--line-strong);
  position: relative;
}
.tier-client-card.is-default .tier-radio { border-color: var(--brand); }
.tier-client-card.is-default .tier-radio::after { content: ''; position: absolute; inset: 3px; border-radius: 50%; background: var(--brand); }
.tier-c-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
.tier-c-top { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.tier-c-top strong { font-size: 13.5px; font-weight: 650; }
.tier-c-foot { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-top: 2px; }

.route-arrow { display: grid; place-items: center; color: var(--ink-4); }

.tier-upstream {
  padding: 12px 14px;
  border-radius: var(--r-md);
  background: var(--subtle);
  display: flex;
  align-items: center;
  gap: 10px;
}
.tier-up-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.tier-model-name { font-family: var(--mono); font-size: 12.5px; font-weight: 650; color: var(--ink-1); overflow-wrap: anywhere; }
.tier-up-tags { display: flex; flex-wrap: wrap; gap: 5px; align-items: center; }
.tier-up-acts { display: flex; gap: 2px; flex: none; }

.media-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.media-card { padding: 16px; display: flex; flex-direction: column; gap: 10px; }
.media-card.off { border-style: dashed; background: transparent; }
.mc-top { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.mc-ico {
  width: 32px;
  height: 32px;
  border-radius: var(--r-inner);
  flex: none;
  background: var(--subtle);
  color: var(--ink-2);
  display: grid;
  place-items: center;
}
.media-card.on .mc-ico { background: var(--brand-soft); color: var(--brand-ink); }
.mc-title { display: flex; flex-direction: column; line-height: 1.25; }
.mc-title strong { font-size: 13.5px; }
.mc-title span { font-size: 11px; color: var(--ink-3); }
.mc-badges { margin-left: auto; display: flex; gap: 5px; flex-wrap: wrap; }
.mc-model-box {
  padding: 10px 12px;
  border-radius: var(--r-inner);
  background: var(--subtle);
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.mc-model-row { display: flex; align-items: center; justify-content: space-between; gap: 8px; flex-wrap: wrap; }
.mc-model-name { font-weight: 650; font-size: 12.5px; overflow-wrap: anywhere; }
.mc-chips { display: flex; flex-wrap: wrap; gap: 5px; }
.mc-chip {
  font-family: var(--mono);
  font-size: 11px;
  padding: 1px 7px;
  border-radius: var(--r-pill);
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink-2);
}
.mc-chip.def { border-color: var(--brand); color: var(--brand-ink); font-weight: 600; }
.mc-foot { margin-top: auto; display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.danger-text { color: var(--danger); }

@media (max-width: 820px) {
  .route-header { display: none; }
  .route-row { grid-template-columns: 1fr; gap: 8px; }
  .route-arrow :deep(svg) { transform: rotate(90deg); }
  .media-grid { grid-template-columns: 1fr; }
}
</style>
