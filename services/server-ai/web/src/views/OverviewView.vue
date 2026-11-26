<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { money, units } from '../format';
import { prefs } from '../prefs';
import type { FlowSegment, ModelOption, Overview, Product, Tier } from '../types';
import BarChart from '../components/BarChart.vue';
import FlowBar from '../components/FlowBar.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';

const props = defineProps<{ refreshTick?: number }>();
const emit = defineEmits<{ go: [key: string, opts?: { productId?: string; createBatch?: boolean }] }>();

const data = ref<Overview | null>(null);
const error = ref('');
const chartMode = ref<'consumed' | 'compare'>('consumed');
const setup = ref<{ products: Product[]; tiers: Tier[]; models: ModelOption[]; batches: number } | null>(null);

async function load() {
  error.value = '';
  try {
    data.value = await api.overview();
    units.perDollar = data.value.quota_per_unit;
  } catch (e) {
    error.value = errorMessage(e);
    return;
  }
  try {
    const [products, batches, models] = await Promise.all([
      api.products(),
      api.batches(),
      api.models().catch(() => ({ items: [] as ModelOption[] })),
    ]);
    const tierLists = await Promise.all(products.items.map((p) => api.tiers(p.product_id)));
    setup.value = { products: products.items, tiers: tierLists.flatMap((l) => l.items), models: models.items, batches: batches.items.length };
  } catch {
    setup.value = null;
  }
}

onMounted(load);
watch(() => props.refreshTick, () => load());

interface Step {
  key: string;
  title: string;
  desc: string;
  done: boolean;
  go?: string;
  cta?: string;
}

const steps = computed<Step[]>(() => {
  const d = data.value;
  const s = setup.value;
  if (!d || !s) return [];
  const live = s.tiers.filter((t) => t.enabled);
  const unpriced = new Set(s.models.filter((m) => m.ratio_warning).map((m) => m.model_name));
  const stuck = [...new Set(live.filter((t) => unpriced.has(t.model)).map((t) => t.model))];
  const hasDefault = s.products.some((p) => p.enabled && live.some((t) => t.product_id === p.product_id && t.is_default && !t.requires_paid));
  return [
    { key: 'gateway', title: '填写网关令牌', desc: '用于自动为用户开通账号、同步额度与读取模型列表。', done: d.newapi_configured, go: 'gateway', cta: '去填写' },
    { key: 'products', title: '配置产品与默认档', desc: '至少一个启用产品含试用可用的默认档位。', done: hasDefault, go: 'products', cta: '去设置' },
    {
      key: 'pricing',
      title: '核对上游模型倍率',
      desc: stuck.length
        ? `${stuck.slice(0, 2).join('、')}${stuck.length > 2 ? ` 等 ${stuck.length} 个` : ''} 仍为默认倍率，建议先在 New API 核对真实定价。`
        : live.length
          ? '已启用档位用到的模型均已设置倍率。'
          : '先配置档位后检查对应模型倍率。',
      done: live.length > 0 && stuck.length === 0,
      go: stuck.length ? 'gateway' : undefined,
      cta: stuck.length ? '查看模型' : undefined,
    },
    { key: 'codes', title: '生成首批兑换码', desc: '导出 .txt 卡密导入淘宝自动发货库存即可开售。', done: s.batches > 0, go: 'codes', cta: '去生成' },
  ];
});

const pending = computed(() => steps.value.filter((s) => !s.done));
const nextKey = computed(() => pending.value[0]?.key);

const today = computed(() => data.value?.series[data.value.series.length - 1]?.consumed ?? 0);
const trend = computed(() => {
  const series = data.value?.series ?? [];
  const curr = series[series.length - 1]?.consumed ?? 0;
  const prev = series[series.length - 2]?.consumed ?? 0;
  if (curr === 0 && prev === 0) return { text: '今日暂无消耗', tone: '' };
  if (prev === 0) return { text: '昨日无消耗', tone: '' };
  const change = Math.round(((curr - prev) / prev) * 100);
  if (change === 0) return { text: '与昨日持平', tone: '' };
  return { text: `${change > 0 ? '↑' : '↓'} 较昨日 ${Math.abs(change)}%`, tone: change > 0 ? 'warn' : 'ok' };
});

const activeShare = computed(() => {
  if (!data.value || data.value.bindings === 0) return 0;
  return Math.min(100, Math.round((data.value.active_24h / data.value.bindings) * 100));
});

const flow = computed<FlowSegment[]>(() => {
  const d = data.value;
  if (!d) return [];
  return [
    { key: 'used', label: '已被用掉', points: d.consumed_total, tone: 'used', note: '用户实际调用消耗，对应向上游付出的模型成本。' },
    { key: 'left', label: '还在用户手里', points: d.balance, tone: 'left', note: '未消费的有效桶余额，属于当前待履约额度。' },
    { key: 'lapsed', label: '到期或被扣回', points: d.lapsed_total, tone: 'lapsed', note: '过期清零与退款扣回部分，不会再产生上游成本。' },
  ];
});

const maxSeven = computed(() => Math.max(1, ...(data.value?.products ?? []).map((p) => p.consumed_7d)));
</script>

<template>
  <div>
    <PageHead
      title="概览"
      :desc="
        prefs.unitMode === 'both'
          ? '实时掌握各产品消耗趋势、剩余负债与异常告警。当前已开启「美元 + 小伴点」双单位对照（$1 = 100 点）。'
          : '实时掌握各产品消耗趋势、剩余负债与异常告警。'
      "
    >
      <button type="button" class="btn" @click="emit('go', 'codes')"><Icon name="ticket" :size="14" />查码 / 退单</button>
      <button type="button" class="btn primary" @click="emit('go', 'codes', { createBatch: true })"><Icon name="plus" :size="14" />生成兑换码</button>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <section v-if="pending.length" class="card setup-strip" aria-label="上线待办">
      <div class="setup-top">
        <span class="tag coral">上线检查</span>
        <strong>上线前还差 {{ pending.length }} 步</strong>
        <div class="setup-bars" role="img" :aria-label="`已完成 ${steps.length - pending.length} / ${steps.length}`">
          <i v-for="s in steps" :key="s.key" :class="{ on: s.done }"></i>
        </div>
      </div>
      <ol class="setup-grid">
        <li v-for="(s, i) in steps" :key="s.key" class="setup-step" :class="{ done: s.done, next: s.key === nextKey }">
          <div class="setup-step-head">
            <span class="step-num">
              <Icon v-if="s.done" name="check" :size="11" :stroke="2.8" />
              <template v-else>{{ i + 1 }}</template>
            </span>
            <span>{{ s.title }}</span>
          </div>
          <p>{{ s.desc }}</p>
          <button
            v-if="!s.done && s.go"
            type="button"
            class="btn sm"
            :class="{ primary: s.key === nextKey }"
            @click="emit('go', s.go, s.key === 'codes' ? { createBatch: true } : undefined)"
          >{{ s.cta }}</button>
        </li>
      </ol>
    </section>

    <div v-if="data && data.open_anomalies > 0" class="notice warn alert">
      <Icon name="alert" :size="16" />
      <span>有 <strong>{{ data.open_anomalies }} 条</strong>对账异常待人工确认：检测到 New API 钱包余额高于本地账本，系统已自动纠偏对齐。</span>
      <button type="button" class="btn sm notice-act" @click="emit('go', 'issues')">立即复核</button>
    </div>

    <div class="kpi-grid">
      <template v-if="data">
        <div class="card kpi-card">
          <div class="kpi-head"><span>用户手里还剩</span><Icon name="coins" :size="15" /></div>
          <div class="kpi-val"><Money :points="data.balance" size="xl" /></div>
          <div class="kpi-sub">分布在 <strong class="num">{{ data.funded_bindings }}</strong> 个有余额账号</div>
        </div>
        <div class="card kpi-card">
          <div class="kpi-head"><span>近 24 小时消耗</span><Icon name="zap" :size="15" /></div>
          <div class="kpi-val"><Money :points="data.consumed_24h" size="xl" /></div>
          <div class="kpi-sub"><span v-if="trend.text" class="tag" :class="trend.tone">{{ trend.text }}</span></div>
        </div>
        <div class="card kpi-card">
          <div class="kpi-head"><span>近 24 小时发放</span><Icon name="plus" :size="15" /></div>
          <div class="kpi-val"><Money :points="data.granted_24h" size="xl" /></div>
          <div class="kpi-sub">含试用、兑换码与人工补发</div>
        </div>
        <div class="card kpi-card">
          <div class="kpi-head"><span>24 小时活跃账号</span><Icon name="users" :size="15" /></div>
          <div class="kpi-val num">
            <strong>{{ data.active_24h }}</strong>
            <small>/ {{ data.bindings }}</small>
          </div>
          <div class="kpi-bar"><i :style="{ width: `${activeShare}%` }"></i></div>
          <div class="kpi-sub num">日活占比 {{ activeShare }}%</div>
        </div>
      </template>
      <template v-else>
        <div v-for="n in 4" :key="n" class="card kpi-card"><i class="skel" style="height: 76px"></i></div>
      </template>
    </div>

    <div class="overview-main">
      <section class="card chart-card" aria-label="近 14 天额度流水">
        <template v-if="data">
          <div class="chart-head">
            <div>
              <h2>近 14 天额度流水</h2>
              <div class="chart-today">
                <span>今日已消耗</span>
                <Money :points="today" size="md" compact-pts />
                <span class="faint">· 悬停柱条查看明细</span>
              </div>
            </div>
            <div class="seg-pills">
              <button type="button" :class="{ active: chartMode === 'consumed' }" @click="chartMode = 'consumed'">每日消耗</button>
              <button type="button" :class="{ active: chartMode === 'compare' }" @click="chartMode = 'compare'">消耗 vs 发放</button>
            </div>
          </div>
          <BarChart :series="data.series" :mode="chartMode" />
        </template>
        <div v-else class="skels">
          <i class="skel" style="width: 42%; height: 20px"></i>
          <i class="skel" style="width: 100%; height: 210px"></i>
        </div>
      </section>

      <section class="card flow-card" aria-label="发放额度去向">
        <template v-if="data">
          <div class="section-head" style="margin-bottom: 0">
            <h2>发放额度去向</h2>
            <span class="faint num" style="font-size: 12px">累计发放 {{ money(data.granted_total) }}</span>
          </div>
          <FlowBar :segments="flow" />
        </template>
        <i v-else class="skel" style="height: 240px"></i>
      </section>
    </div>

    <section v-if="data" class="section">
      <div class="section-head">
        <h2>各产品运行状态</h2>
        <span class="hint">点击任意产品直接进入该产品的档位与媒体配置</span>
      </div>
      <div class="table-card">
        <div class="table-scroll">
          <table class="table">
            <thead>
              <tr>
                <th>产品</th>
                <th>关联客户端</th>
                <th class="right">开通账号</th>
                <th>近 7 天消耗</th>
                <th class="right">用户手里还剩</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="p in data.products"
                :key="p.product_id"
                class="click"
                tabindex="0"
                @click="emit('go', 'products', { productId: p.product_id })"
                @keydown.enter="emit('go', 'products', { productId: p.product_id })"
              >
                <td>
                  <div class="row wrap" style="gap: 7px">
                    <strong>{{ p.name }}</strong>
                    <span v-if="p.enabled !== undefined" class="tag" :class="p.enabled ? 'ok' : ''">{{ p.enabled ? '运行中' : '已停用' }}</span>
                  </div>
                  <div class="mono faint" style="margin-top: 2px">{{ p.product_id }}</div>
                </td>
                <td>
                  <span v-if="p.client_count !== undefined" class="tag num">{{ p.client_count }} 个客户端</span>
                  <span v-else class="faint">—</span>
                </td>
                <td class="right num">{{ p.bindings.toLocaleString('en-US') }}</td>
                <td class="used-cell">
                  <Money :points="p.consumed_7d" size="sm" :muted="p.consumed_7d === 0" compact-pts />
                  <span class="meter" aria-hidden="true"><i :style="{ width: `${(p.consumed_7d / maxSeven) * 100}%` }"></i></span>
                </td>
                <td class="right"><Money :points="p.balance" size="sm" :muted="p.balance === 0" compact-pts /></td>
                <td class="right faint"><Icon name="chevron-right" :size="15" /></td>
              </tr>
              <tr v-if="data.products.length === 0">
                <td colspan="6" class="table-empty">
                  还没有产品。
                  <button type="button" class="btn sm primary" style="margin-left: 8px" @click="emit('go', 'products')">去添加产品</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.notice { margin-bottom: 16px; }

.setup-strip { padding: 16px 18px; margin-bottom: 16px; border-color: var(--line-strong); }
.setup-top { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.setup-top strong { font-size: 14px; }
.setup-bars { display: flex; gap: 5px; margin-left: auto; align-items: center; }
.setup-bars i { display: block; width: 28px; height: 5px; border-radius: 99px; background: var(--canvas-soft); border: 1px solid var(--line); }
.setup-bars i.on { background: var(--coral); border-color: var(--coral); }
.setup-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-top: 14px; }
.setup-step {
  padding: 12px 13px;
  border-radius: var(--r-md);
  border: 1px solid var(--line);
  background: var(--surface-muted);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.setup-step.next { background: var(--surface); border-color: var(--coral-line); box-shadow: var(--shadow-xs); }
.setup-step.done { opacity: 0.65; }
.setup-step-head { display: flex; align-items: center; gap: 8px; font-weight: 650; font-size: 13px; }
.step-num {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  flex: none;
  border: 1.5px solid var(--line-strong);
  font-size: 11px;
  font-weight: 700;
  display: grid;
  place-items: center;
  color: var(--ink-3);
}
.setup-step.next .step-num { border-color: var(--coral); color: var(--coral); }
.setup-step.done .step-num { border-color: var(--ok); background: var(--ok); color: #fff; }
.setup-step p { font-size: 12px; color: var(--ink-2); line-height: 1.48; }
.setup-step .btn { align-self: flex-start; margin-top: auto; }

.kpi-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
.kpi-card { padding: 16px 18px; display: flex; flex-direction: column; gap: 8px; }
.kpi-head { display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; font-weight: 600; color: var(--ink-2); }
.kpi-val { font-size: 26px; font-weight: 750; letter-spacing: -0.025em; line-height: 1.15; }
.kpi-val small { font-size: 14px; font-weight: 550; color: var(--ink-3); margin-left: 5px; }
.kpi-sub { font-size: 12px; color: var(--ink-3); display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.kpi-sub strong { color: var(--ink); }
.kpi-bar { height: 4px; border-radius: 99px; background: var(--canvas-soft); overflow: hidden; }
.kpi-bar i { display: block; height: 100%; border-radius: 99px; background: var(--ink); }

.overview-main { display: grid; grid-template-columns: minmax(0, 1.55fr) minmax(0, 1fr); gap: 14px; margin-top: 14px; }
.chart-card { padding: 18px 20px; display: flex; flex-direction: column; gap: 14px; }
.chart-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
.chart-head h2 { font-size: 14.5px; font-weight: 680; }
.chart-today { display: flex; align-items: center; gap: 8px; margin-top: 4px; font-size: 12.5px; color: var(--ink-2); flex-wrap: wrap; }
.skels { display: flex; flex-direction: column; gap: 16px; }
.flow-card { padding: 18px 20px; display: flex; flex-direction: column; gap: 16px; }

.used-cell { white-space: nowrap; }
.meter { display: inline-block; width: 72px; height: 5px; border-radius: 99px; background: var(--canvas-soft); overflow: hidden; vertical-align: middle; margin-left: 8px; }
.meter i { display: block; height: 100%; border-radius: 99px; background: var(--coral); }

@media (max-width: 1080px) {
  .kpi-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .overview-main { grid-template-columns: 1fr; }
  .setup-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
@media (max-width: 560px) {
  .kpi-grid, .setup-grid { grid-template-columns: 1fr; }
}
</style>
