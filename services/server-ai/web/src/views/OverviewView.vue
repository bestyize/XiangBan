<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { units } from '../format';
import type { FlowSegment, ModelOption, Overview, Product, Tier } from '../types';
import BarChart from '../components/BarChart.vue';
import FlowBar from '../components/FlowBar.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';

const emit = defineEmits<{ go: [string] }>();

const data = ref<Overview | null>(null);
const error = ref('');
const setup = ref<{ products: Product[]; tiers: Tier[]; models: ModelOption[]; batches: number } | null>(null);

async function load() {
  try {
    data.value = await api.overview();
    units.perDollar = data.value.quota_per_unit;
  } catch (e) {
    error.value = errorMessage(e);
    return;
  }
  // 上线待办只是辅助：其中任何一项取不到，都不该影响概览本身。
  try {
    const [products, batches, models] = await Promise.all([api.products(), api.batches(), api.models().catch(() => ({ items: [] as ModelOption[] }))]);
    const tierLists = await Promise.all(products.items.map((p) => api.tiers(p.product_id)));
    setup.value = { products: products.items, tiers: tierLists.flatMap((l) => l.items), models: models.items, batches: batches.items.length };
  } catch {
    setup.value = null;
  }
}
onMounted(load);

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
    { key: 'gateway', title: '填写网关令牌', desc: '本服务要靠它在 AI 网关里为用户开通账号、增减额度。', done: d.newapi_configured, go: 'gateway', cta: '去填写' },
    { key: 'products', title: '建好产品和默认档位', desc: '至少一个启用的产品，里面有一个不要求付费的默认档位，新用户才能直接用。', done: hasDefault, go: 'products', cta: '去设置' },
    {
      key: 'pricing',
      title: '给用到的模型配好价格',
      desc: stuck.length
        ? `${stuck.join('、')} 的倍率还是网关的默认值，需要到 New API 后台「模型倍率」里设置，不然扣费和成本对不上。`
        : live.length ? '档位用到的模型都已有价格。' : '先建档位，才知道要给哪些模型配价格。',
      done: live.length > 0 && stuck.length === 0,
    },
    { key: 'codes', title: '生成第一批兑换码', desc: '把码导入淘宝的自动发货库存，就可以开卖。', done: s.batches > 0, go: 'codes', cta: '去生成' },
  ];
});
const pending = computed(() => steps.value.filter((s) => !s.done));
const nextKey = computed(() => pending.value[0]?.key);

const trend = computed(() => {
  const series = data.value?.series ?? [];
  const today = series[series.length - 1]?.consumed ?? 0;
  const yesterday = series[series.length - 2]?.consumed ?? 0;
  if (today === 0 && yesterday === 0) return { text: '今天还没有消耗', tone: '' };
  if (yesterday === 0) return { text: '昨天没有消耗', tone: '' };
  const change = Math.round(((today - yesterday) / yesterday) * 100);
  if (change === 0) return { text: '和昨天持平', tone: '' };
  return { text: `比昨天${change > 0 ? '多' : '少'} ${Math.abs(change)}%`, tone: change > 0 ? 'amber' : 'green' };
});
const today = computed(() => data.value?.series[data.value.series.length - 1]?.consumed ?? 0);

const flow = computed<FlowSegment[]>(() => {
  const d = data.value;
  if (!d) return [];
  return [
    { key: 'used', label: '已被用掉', points: d.consumed_total, tone: 'used', note: '用户实际花掉的，对应你向上游付出的成本。' },
    { key: 'lapsed', label: '到期或被扣回', points: d.lapsed_total, tone: 'lapsed', note: '没用完就过期的，和你手动扣回的，这部分不会再产生成本。' },
    { key: 'left', label: '还在用户手里', points: d.balance, tone: 'left', note: '将来用户用掉才会变成成本，是你现在欠用户的额度。' },
  ];
});

const maxSeven = computed(() => Math.max(1, ...(data.value?.products ?? []).map((p) => p.consumed_7d)));
</script>

<template>
  <div>
    <PageHead title="概览" desc="先看有没有需要处理的事，再看额度的进出和去向。" />

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

    <section v-if="pending.length" class="setup panel" aria-label="上线待办">
      <header>
        <h2>上线前还差 {{ pending.length }} 步</h2>
        <div class="progress" role="img" :aria-label="`已完成 ${steps.length - pending.length} / ${steps.length}`">
          <span v-for="s in steps" :key="s.key" :class="{ on: s.done }"></span>
        </div>
      </header>
      <ol>
        <li v-for="(s, i) in steps" :key="s.key" :class="{ done: s.done, next: s.key === nextKey }">
          <span class="mark" :aria-label="s.done ? '已完成' : `第 ${i + 1} 步`">
            <Icon v-if="s.done" name="check" :size="14" :stroke="2.6" />
            <template v-else>{{ i + 1 }}</template>
          </span>
          <div class="body">
            <strong>{{ s.title }}</strong>
            <p>{{ s.desc }}</p>
            <button v-if="!s.done && s.go" type="button" class="btn small" :class="{ primary: s.key === nextKey }" @click="emit('go', s.go)">{{ s.cta }}</button>
          </div>
        </li>
      </ol>
    </section>

    <div v-if="data && data.open_anomalies > 0" class="notice warn alert">
      <Icon name="alert" :size="17" />
      <span>有 {{ data.open_anomalies }} 条对账异常没处理，可能是有人在 New API 后台手工改过用户额度。<button type="button" class="linkbtn" @click="emit('go', 'issues')">去处理</button></span>
    </div>

    <div class="hero">
      <section class="panel chart" aria-label="近 14 天额度流水">
        <template v-if="data">
          <header>
            <div>
              <h2>近 14 天额度流水</h2>
              <p class="today"><span class="muted">今天已用掉</span><Money :points="today" size="lg" /><span v-if="trend.text" class="tag" :class="trend.tone">{{ trend.text }}</span></p>
            </div>
          </header>
          <BarChart :series="data.series" />
        </template>
        <div v-else class="skels"><i class="skel" style="width: 38%; height: 20px"></i><i class="skel" style="width: 100%; height: 190px"></i></div>
      </section>

      <aside class="stats" aria-label="关键数字">
        <template v-if="data">
          <div class="stat lead">
            <span class="k">用户手里还剩</span>
            <Money :points="data.balance" size="xl" />
            <span class="sub num">分布在 {{ data.funded_bindings }} 个有额度的账号里</span>
          </div>
          <div class="stat"><span class="k">近 24 小时用掉</span><Money :points="data.consumed_24h" size="lg" /></div>
          <div class="stat"><span class="k">近 24 小时发放</span><Money :points="data.granted_24h" size="lg" /><span class="sub">含试用、兑换码和你手动补发的</span></div>
          <div class="stat"><span class="k">24 小时内活跃</span><span class="count num"><b>{{ data.active_24h }}</b><small>/ {{ data.bindings }} 个账号</small></span></div>
        </template>
        <template v-else><i v-for="n in 4" :key="n" class="skel" style="height: 62px"></i></template>
      </aside>
    </div>

    <section v-if="data" class="section">
      <div class="section-head">
        <h2>发放的额度去哪了</h2>
        <span class="muted num">累计发放 <Money :points="data.granted_total" size="md" /></span>
      </div>
      <div class="panel flowpanel"><FlowBar :segments="flow" /></div>
    </section>

    <section v-if="data" class="section">
      <div class="section-head"><h2>各产品</h2></div>
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr><th>产品</th><th class="right">用过 AI 的账号</th><th>近 7 天用掉</th><th class="right">用户手里还剩</th></tr>
          </thead>
          <tbody>
            <tr v-for="p in data.products" :key="p.product_id" class="click" tabindex="0" @click="emit('go', 'products')" @keydown.enter="emit('go', 'products')">
              <td><strong>{{ p.name }}</strong><div class="faint mono">{{ p.product_id }}</div></td>
              <td class="right num">{{ p.bindings }}</td>
              <td class="used">
                <Money :points="p.consumed_7d" size="md" :muted="p.consumed_7d === 0" />
                <span class="meter" aria-hidden="true"><i :style="{ width: `${(p.consumed_7d / maxSeven) * 100}%` }"></i></span>
              </td>
              <td class="right"><Money :points="p.balance" size="md" :muted="p.balance === 0" /></td>
            </tr>
            <tr v-if="data.products.length === 0">
              <td colspan="4" class="table-empty">还没有产品。<button type="button" class="linkbtn tideLink" @click="emit('go', 'products')">去添加产品</button></td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </div>
</template>

<style scoped>
.panel + .notice, .notice.alert { margin-top: 14px; }

.setup { padding: 20px 22px 22px; margin-bottom: 22px; }
.setup header { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: 16px; }
.progress { display: flex; gap: 5px; }
.progress span { width: 30px; height: 6px; border-radius: 3px; background: var(--line); }
.progress span.on { background: var(--tide); }
.setup ol { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
.setup li { display: flex; gap: 12px; padding: 14px; border: 1px solid var(--line); border-radius: 12px; background: #fbfdfd; }
.setup li.next { border-color: var(--tide-line); background: var(--tide-soft); box-shadow: inset 0 0 0 1px var(--tide-line); }
.setup li.done { background: transparent; }
.setup li.done strong { color: var(--muted); font-weight: 600; }
.mark { flex: none; display: grid; place-items: center; width: 24px; height: 24px; border-radius: 50%; border: 1.5px solid var(--line-strong); font-size: 12px; font-weight: 700; color: var(--muted); }
.setup li.next .mark { border-color: var(--tide); color: var(--tide); background: #fff; }
.setup li.done .mark { border-color: var(--tide); background: var(--tide); color: #fff; }
.body { display: flex; flex-direction: column; gap: 4px; align-items: flex-start; min-width: 0; }
.body strong { font-size: 14px; line-height: 1.4; }
.body p { color: var(--muted); font-size: 12.5px; line-height: 1.55; }
.body .btn { margin-top: 8px; }

.hero { display: grid; grid-template-columns: minmax(0, 1fr) 320px; gap: 22px; margin-top: 22px; align-items: stretch; }
.setup + .hero, .alert + .hero { margin-top: 22px; }
.chart { padding: 20px 22px 18px; }
.chart header { margin-bottom: 4px; }
.today { display: flex; align-items: baseline; gap: 10px; margin-top: 6px; flex-wrap: wrap; }
.today .tag { align-self: center; }
.skels { display: flex; flex-direction: column; gap: 18px; }

.stats { display: flex; flex-direction: column; border-left: 1px solid var(--line); padding-left: 22px; }
.stat { display: flex; flex-direction: column; gap: 4px; padding: 16px 0; border-bottom: 1px solid var(--line); }
.stat:first-child { padding-top: 2px; }
.stat:last-child { border-bottom: none; }
.stat.lead { padding-bottom: 20px; }
.k { color: var(--muted); font-size: 13px; font-weight: 600; }
.sub { color: var(--muted); font-size: 12.5px; }
.count b { font-size: 26px; font-weight: 700; letter-spacing: -0.02em; }
.count small { color: var(--muted); font-size: 13px; margin-left: 6px; font-weight: 600; }

.flowpanel { padding: 22px 24px; }

.used { min-width: 190px; }
.used .money { display: inline-flex; min-width: 76px; }
.meter { display: inline-block; vertical-align: middle; width: 90px; height: 6px; margin-left: 10px; border-radius: 3px; background: var(--line); overflow: hidden; }
.meter i { display: block; height: 100%; border-radius: 3px; background: var(--tide); }
.tideLink { color: var(--tide); margin-left: 6px; }

@media (max-width: 1100px) { .setup ol { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 960px) {
  .hero { grid-template-columns: 1fr; }
  .stats { border-left: none; padding-left: 0; display: grid; grid-template-columns: 1fr 1fr; column-gap: 24px; }
  .stat.lead { grid-column: 1 / -1; }
  .stat:nth-child(odd):not(.lead) { border-bottom: 1px solid var(--line); }
}
@media (max-width: 560px) {
  .setup ol { grid-template-columns: 1fr; }
  .setup header { flex-direction: column; align-items: flex-start; gap: 10px; }
  .stats { grid-template-columns: 1fr; }
}
</style>
