<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { notify } from '../toast';
import type { MediaSetting, ModelOption, Product, Tier } from '../types';
import { CAPABILITIES, type CapabilityInfo } from '../media';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';
import ProductDialog from '../components/ProductDialog.vue';
import MediaDialog from '../components/MediaDialog.vue';
import TierDialog from '../components/TierDialog.vue';

const emit = defineEmits<{ go: [string]; changed: [] }>();

const products = ref<Product[]>([]);
const selectedId = ref('');
const tiers = ref<Tier[]>([]);
const models = ref<ModelOption[]>([]);
const modelsError = ref('');
const loading = ref(true);
const error = ref('');

const editingProduct = ref<Product | 'new' | null>(null);
const editingTier = ref<Tier | 'new' | null>(null);
const media = ref<MediaSetting[]>([]);
const editingMedia = ref<CapabilityInfo | null>(null);

const selected = computed(() => products.value.find((p) => p.product_id === selectedId.value) ?? null);
const modelInfo = computed(() => new Map(models.value.map((m) => [m.model_name, m])));
const span = computed(() => Math.max(tiers.value.length, 1) + 1);
const mediaFor = (id: string) => media.value.find((m) => m.capability === id) ?? null;

async function loadProducts(keep?: string) {
  loading.value = true;
  error.value = '';
  try {
    products.value = (await api.products()).items;
    const has = (id: string) => products.value.some((p) => p.product_id === id);
    selectedId.value = keep && has(keep) ? keep : has(selectedId.value) ? selectedId.value : (products.value[0]?.product_id ?? '');
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

/** 卡片上那一行：语音的音色、图片的尺寸、实时语音的时长，识别没有额外选项。 */
function mediaDetail(m: MediaSetting): string {
  if (m.capability === 'speech') return `音色：${m.voices.join('、')}（默认 ${m.voices[0] ?? '—'}）`;
  if (m.capability === 'image') return `尺寸：${m.sizes.join('、')}`;
  if (m.capability === 'realtime') return `单次最长 ${Math.round(m.max_secs / 60)} 分钟`;
  return '';
}

async function mediaSaved(warnings: string[]) {
  editingMedia.value = null;
  if (warnings.length) notify(`已保存，但请注意：${warnings.join('；')}`, 'warn');
  else notify('已保存，新请求立即生效');
  await Promise.all([loadMedia(), loadModels()]);
}

async function removeMedia(capability: CapabilityInfo) {
  if (!window.confirm(`关闭「${capability.label}」？客户端再使用会收到“暂未开放”。`)) return;
  try {
    await api.deleteMedia(selectedId.value, capability.id);
    notify(`${capability.label}已关闭`);
    await loadMedia();
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
watch(selectedId, () => Promise.all([loadTiers(), loadMedia()]), { immediate: true });

async function productSaved(id: string) {
  editingProduct.value = null;
  notify('产品已保存');
  await loadProducts(id);
  emit('changed');
}

async function tierSaved(warnings: string[]) {
  editingTier.value = null;
  if (warnings.length) notify(`档位已保存，但请注意：${warnings.join('；')}`, 'warn');
  else notify('档位已保存，新请求立即生效');
  await Promise.all([loadTiers(), loadModels()]);
}

async function removeTier(tier: Tier) {
  if (!window.confirm(`删除档位「${tier.display_name}」？客户端再请求这个档位会收到“未知的模型档位”。`)) return;
  try {
    await api.deleteTier(tier.product_id, tier.tier);
    notify('档位已删除');
    await loadTiers();
  } catch (e) {
    error.value = errorMessage(e);
  }
}
</script>

<template>
  <div>
    <PageHead title="产品与档位" desc="用户在客户端只会看到档位名称，背后实际调用哪个模型由你决定，随时可换，不用给客户端发版。">
      <button type="button" class="btn primary" @click="editingProduct = 'new'"><Icon name="plus" :size="16" />添加产品</button>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

    <div v-if="!loading && products.length === 0" class="panel">
      <EmptyState icon="layers" title="还没有产品" desc="先添加一个产品（例如小伴壁纸），并关联它的客户端，用户才能使用 AI。">
        <button type="button" class="btn primary" @click="editingProduct = 'new'"><Icon name="plus" :size="16" />添加第一个产品</button>
      </EmptyState>
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
        >{{ p.name }}<span v-if="!p.enabled" class="tag">已停用</span></button>
      </div>

      <section class="summary">
        <div class="who">
          <h2>{{ selected.name }}</h2>
          <div class="chips">
            <span v-for="c in selected.client_ids" :key="c" class="chip mono">{{ c }}</span>
            <span v-if="!selected.client_ids.length" class="muted">还没有关联客户端，用户暂时无法使用</span>
          </div>
        </div>
        <div class="trial">
          <span class="k">新用户试用</span>
          <template v-if="selected.trial_quota > 0"><Money :points="selected.trial_quota" size="lg" /><span class="muted">{{ selected.trial_days }} 天内有效</span></template>
          <span v-else class="muted">不送试用额度</span>
        </div>
        <button type="button" class="btn small" @click="editingProduct = selected"><Icon name="pencil" :size="14" />编辑产品</button>
      </section>

      <div v-if="modelsError" class="notice warn"><Icon name="alert" :size="17" /><span>读不到 AI 网关的模型列表，暂时无法新增或修改档位：{{ modelsError }}</span></div>

      <section class="stage" :style="{ '--span': span }" aria-label="档位与实际模型的对应关系">
        <div class="bg phone"></div>
        <div class="bg curtain" aria-hidden="true"></div>
        <div class="bg backstage"></div>

        <header class="h front"><Icon name="eye" :size="16" />用户在客户端看到的</header>
        <header class="h back"><Icon name="eye-off" :size="16" />幕后实际调用的<span class="hide">用户看不到</span></header>

        <template v-for="(t, i) in tiers" :key="t.tier">
          <div class="cell front" :style="{ '--r': i + 2 }">
            <div class="opt" :class="{ sel: t.is_default, off: !t.enabled, paid: t.requires_paid }">
              <span class="radio" :class="{ on: t.is_default }" aria-hidden="true"></span>
              <div class="txt">
                <div class="nm">
                  <strong>{{ t.display_name }}</strong>
                  <span v-if="t.is_default" class="tag green">默认</span>
                  <span v-if="!t.enabled" class="tag">已停用</span>
                </div>
                <div v-if="t.description" class="ds">{{ t.description }}</div>
              </div>
              <span v-if="t.requires_paid" class="lock" title="需要先兑换额度才能使用"><Icon name="lock" :size="14" />需兑换额度</span>
            </div>
          </div>

          <div class="cell wire" :class="{ off: !t.enabled }" :style="{ '--r': i + 2 }" aria-hidden="true"><i></i></div>

          <div class="cell back" :class="{ off: !t.enabled }" :style="{ '--r': i + 2 }">
            <div class="model">
              <div class="mn mono">{{ t.model }}</div>
              <div class="mm">
                <template v-if="modelInfo.get(t.model)">
                  <span class="tag">输入倍率 {{ modelInfo.get(t.model)!.model_ratio }}</span>
                  <span class="tag">输出 ×{{ modelInfo.get(t.model)!.completion_ratio }}</span>
                  <span v-if="modelInfo.get(t.model)!.ratio_warning" class="tag amber" title="倍率与 New API 未配置时的默认值相同，可能还没设置真实定价"><Icon name="alert" :size="12" />倍率可能未配置</span>
                </template>
                <span v-else-if="models.length" class="tag rose" title="AI 网关里已经找不到这个模型"><Icon name="circle-alert" :size="12" />模型已不可用</span>
              </div>
              <div class="slug">客户端请求里 model 填 <code>{{ t.tier }}</code></div>
            </div>
            <div class="acts">
              <button type="button" class="btn icon small quiet" :aria-label="`修改档位 ${t.display_name}`" title="修改" @click="editingTier = t"><Icon name="pencil" :size="15" /></button>
              <button type="button" class="btn icon small quiet" :aria-label="`删除档位 ${t.display_name}`" title="删除" @click="removeTier(t)"><Icon name="trash" :size="15" /></button>
            </div>
          </div>
        </template>

        <div v-if="!tiers.length" class="none">
          <EmptyState icon="layers" title="这个产品还没有档位" desc="至少添加一个不要求付费的默认档位，新用户才能直接使用。">
            <button type="button" class="btn primary" @click="editingTier = 'new'"><Icon name="plus" :size="16" />添加档位</button>
          </EmptyState>
        </div>
      </section>

      <div class="below">
        <p class="hint">倍率决定每次调用扣多少额度。换模型前，先到 New API 确认新模型的倍率已经设置，否则会按默认倍率多扣或少扣。</p>
        <button v-if="tiers.length" type="button" class="btn" @click="editingTier = 'new'"><Icon name="plus" :size="16" />添加档位</button>
      </div>

      <section class="media" aria-labelledby="media-heading">
        <div class="media-head">
          <h2 id="media-heading">媒体能力</h2>
          <p class="hint">语音、图片、识别和实时语音各用哪个模型。客户端只知道开放了哪些能力、能选哪些音色和尺寸，看不到模型名；没开放的能力会收到“暂未开放”。</p>
        </div>
        <div class="media-grid">
          <article v-for="c in CAPABILITIES" :key="c.id" class="panel media-card" :class="{ off: !mediaFor(c.id)?.enabled }">
            <div class="mc-head">
              <span class="mc-icon"><Icon :name="c.icon" :size="17" /></span>
              <strong>{{ c.label }}</strong>
              <span v-if="!mediaFor(c.id)" class="tag">未开放</span>
              <span v-else-if="!mediaFor(c.id)!.enabled" class="tag">已停用</span>
              <span v-else class="tag green">已开放</span>
              <span v-if="mediaFor(c.id)?.requires_paid" class="tag amber" title="需要先兑换额度才能使用"><Icon name="lock" :size="12" />需兑换额度</span>
            </div>
            <template v-if="mediaFor(c.id)">
              <div class="mn mono">{{ mediaFor(c.id)!.model }}</div>
              <div class="mm">
                <template v-if="modelInfo.get(mediaFor(c.id)!.model)">
                  <span class="tag">倍率 {{ modelInfo.get(mediaFor(c.id)!.model)!.model_ratio }}</span>
                  <span v-if="modelInfo.get(mediaFor(c.id)!.model)!.ratio_warning" class="tag amber" title="倍率与 New API 未配置时的默认值相同，可能还没设置真实定价"><Icon name="alert" :size="12" />倍率可能未配置</span>
                </template>
                <span v-else-if="models.length" class="tag rose" title="AI 网关里已经找不到这个模型"><Icon name="circle-alert" :size="12" />模型已不可用</span>
              </div>
              <div v-if="mediaDetail(mediaFor(c.id)!)" class="mc-detail">{{ mediaDetail(mediaFor(c.id)!) }}</div>
            </template>
            <p v-else class="mc-detail muted">{{ c.usedBy }}</p>
            <div class="mc-acts">
              <button type="button" class="btn small" :disabled="Boolean(modelsError)" @click="editingMedia = c">
                <Icon :name="mediaFor(c.id) ? 'pencil' : 'plus'" :size="14" />{{ mediaFor(c.id) ? '修改' : '开放' }}
              </button>
              <button v-if="mediaFor(c.id)" type="button" class="btn icon small quiet" :aria-label="`关闭${c.label}`" title="关闭" @click="removeMedia(c)"><Icon name="trash" :size="15" /></button>
            </div>
          </article>
        </div>
      </section>
    </template>

    <ProductDialog v-if="editingProduct" :product="editingProduct === 'new' ? null : editingProduct" @close="editingProduct = null" @saved="productSaved" />
    <MediaDialog
      v-if="editingMedia && selected"
      :product="selected"
      :capability="editingMedia"
      :setting="mediaFor(editingMedia.id)"
      :models="models"
      @close="editingMedia = null"
      @saved="mediaSaved"
    />
    <TierDialog v-if="editingTier && selected" :product="selected" :tier="editingTier === 'new' ? null : editingTier" :models="models" @close="editingTier = null" @saved="tierSaved" />
  </div>
</template>

<style scoped>
.notice { margin-bottom: 16px; }

.tabs { display: inline-flex; gap: 4px; padding: 4px; margin-bottom: 18px; background: #e6ecec; border-radius: 12px; max-width: 100%; overflow-x: auto; }
.tab { display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; border: none; border-radius: 9px; background: none; color: var(--muted); font-weight: 600; cursor: pointer; white-space: nowrap; transition: background-color 0.14s, color 0.14s; }
.tab:hover { color: var(--text); }
.tab.active { background: var(--surface); color: var(--text); box-shadow: var(--shadow-1), 0 0 0 1px rgba(16, 28, 34, 0.05); }

.summary { display: flex; align-items: center; gap: 28px; flex-wrap: wrap; margin-bottom: 20px; }
.who { flex: 1; min-width: 260px; display: flex; flex-direction: column; gap: 9px; }
.chips { display: flex; flex-wrap: wrap; gap: 6px; }
.chip { padding: 3px 9px; border-radius: 999px; background: var(--surface); box-shadow: inset 0 0 0 1px var(--line-strong); color: var(--muted); font-size: 12px; }
.trial { display: flex; align-items: baseline; gap: 10px; }
.k { color: var(--muted); font-weight: 600; font-size: 13px; }

/* ---- 舞台 ---- */
.stage { --curtain-w: 148px; position: relative; display: grid; grid-template-columns: minmax(0, 1fr) var(--curtain-w) minmax(0, 1fr); border-radius: 20px; overflow: hidden; background: var(--surface); box-shadow: var(--shadow-1), 0 0 0 1px var(--line-strong); isolation: isolate; }
.bg { grid-row: 1 / span var(--span); z-index: 0; }
.phone { grid-column: 1; background: linear-gradient(180deg, #fff, #fafcfc); }
.curtain { grid-column: 2; background: var(--curtain-dark); box-shadow: inset 14px 0 22px -14px rgba(0, 0, 0, 0.7), inset -14px 0 22px -14px rgba(0, 0, 0, 0.7); }
.backstage { grid-column: 3; background: var(--curtain-light); box-shadow: inset 16px 0 22px -16px rgba(16, 28, 34, 0.35); }

.h { position: relative; z-index: 1; grid-row: 1; display: flex; align-items: center; gap: 8px; padding: 18px 24px 14px; font-weight: 700; font-size: 13.5px; }
.h.front { grid-column: 1; border-bottom: 1px solid var(--line); color: var(--text); }
.h.back { grid-column: 3; border-bottom: 1px solid rgba(16, 28, 34, 0.1); color: #34464c; }
.hide { margin-left: auto; font-weight: 600; font-size: 12px; color: var(--muted); }

.cell { position: relative; z-index: 1; grid-row: var(--r); display: flex; align-items: center; min-height: 92px; }
.cell.front { grid-column: 1; padding: 14px 22px; border-bottom: 1px solid var(--line); }
.cell.back { grid-column: 3; padding: 14px 14px 14px 24px; gap: 8px; border-bottom: 1px solid rgba(16, 28, 34, 0.09); }
.cell.wire { grid-column: 2; }

/* 穿过幕布的线：两端各一个节点 */
.wire i { position: relative; display: block; width: 100%; height: 2px; background: var(--mint); box-shadow: 0 0 14px rgba(111, 216, 194, 0.55); }
.wire i::before, .wire i::after { content: ''; position: absolute; top: 50%; width: 10px; height: 10px; border-radius: 50%; background: var(--mint); transform: translateY(-50%); box-shadow: 0 0 0 4px rgba(111, 216, 194, 0.22); }
.wire i::before { left: -5px; }
.wire i::after { right: -5px; }
.wire.off i { background: repeating-linear-gradient(90deg, #5f7a76 0 6px, transparent 6px 11px); box-shadow: none; }
.wire.off i::before, .wire.off i::after { background: #5f7a76; box-shadow: none; }

/* 前台：像客户端里的一个可选项 */
.opt { width: 100%; display: flex; align-items: center; gap: 13px; padding: 13px 15px; border: 1.5px solid var(--line); border-radius: 14px; background: var(--surface); box-shadow: var(--shadow-1); }
.opt.sel { border-color: var(--tide); background: var(--tide-soft); box-shadow: 0 0 0 3px rgba(10, 107, 94, 0.1); }
.opt.off { border-style: dashed; background: transparent; box-shadow: none; opacity: 0.65; }
.radio { flex: none; width: 18px; height: 18px; border-radius: 50%; border: 2px solid var(--line-strong); background: #fff; position: relative; }
.radio.on { border-color: var(--tide); }
.radio.on::after { content: ''; position: absolute; inset: 3px; border-radius: 50%; background: var(--tide); }
.txt { flex: 1; min-width: 0; }
.nm { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.nm strong { font-size: 15px; }
.ds { color: var(--muted); font-size: 13px; margin-top: 2px; }
.lock { flex: none; display: inline-flex; align-items: center; gap: 5px; padding: 4px 9px; border-radius: 999px; background: var(--amber-soft); color: var(--amber); font-size: 12px; font-weight: 700; }

/* 幕后：真实的模型 */
.model { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 7px; }
.mn { font-size: 14px; font-weight: 600; color: var(--text); overflow-wrap: anywhere; letter-spacing: -0.005em; }
.mm { display: flex; flex-wrap: wrap; gap: 5px; }
.mm .tag { background: rgba(255, 255, 255, 0.72); }
.mm .tag.amber { background: var(--amber-soft); }
.mm .tag.rose { background: var(--rose-soft); }
.slug { color: var(--muted); font-size: 12.5px; }
.slug code { font-family: var(--mono); font-size: 12px; padding: 1px 6px; border-radius: 5px; background: rgba(255, 255, 255, 0.75); color: var(--text); }
.cell.back.off .mn { color: var(--faint); }
.acts { flex: none; display: flex; flex-direction: column; gap: 2px; }

.none { grid-column: 1 / -1; grid-row: 2; position: relative; z-index: 1; background: var(--surface); }

.below { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-top: 14px; flex-wrap: wrap; }
.below .hint { max-width: 66ch; }

/* ---- 媒体能力 ---- */
.media { margin-top: 34px; }
.media-head { display: flex; flex-direction: column; gap: 4px; margin-bottom: 14px; }
.media-head .hint { max-width: 76ch; }
.media-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 14px; }
.media-card { display: flex; flex-direction: column; gap: 9px; padding: 16px; min-width: 0; }
.media-card.off { background: transparent; box-shadow: none; border-style: dashed; }
.mc-head { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.mc-icon { display: inline-flex; padding: 6px; border-radius: 9px; background: var(--tide-soft); color: var(--tide); }
.media-card.off .mc-icon { background: #eaeff0; color: var(--muted); }
.mc-detail { font-size: 13px; color: var(--muted); overflow-wrap: anywhere; }
.mc-acts { margin-top: auto; display: flex; align-items: center; gap: 6px; padding-top: 4px; }

@media (max-width: 860px) {
  .stage { grid-template-columns: 1fr; border-radius: 16px; }
  .bg { display: none; }
  .h { display: none; }
  .cell { grid-row: auto; min-height: 0; }
  .cell.front, .cell.wire, .cell.back { grid-column: 1; }
  .cell.front { border-bottom: none; padding: 16px 16px 0; }
  .cell.front::before { content: '用户看到的'; position: absolute; top: 3px; left: 18px; font-size: 11.5px; font-weight: 700; color: var(--faint); }
  .cell.front { padding-top: 26px; }
  .cell.wire { height: 34px; background: var(--curtain-dark); margin-top: 16px; }
  .wire i { width: 2px; height: 100%; margin: 0 auto; }
  .wire i::before, .wire i::after { left: 50%; top: auto; transform: translateX(-50%); }
  .wire i::before { top: -5px; }
  .wire i::after { bottom: -5px; right: auto; }
  .cell.back { background: var(--curtain-light); padding: 26px 16px 16px; border-bottom: 6px solid var(--paper); }
  .cell.back::before { content: '幕后实际调用（用户看不到）'; position: absolute; top: 5px; left: 18px; font-size: 11.5px; font-weight: 700; color: var(--muted); }
  .none { grid-row: auto; }
  .summary { gap: 14px; }
}
</style>
