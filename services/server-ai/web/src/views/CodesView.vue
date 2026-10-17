<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { closedLabels, codeStatusLabels, formatDate, formatTime, kindLabels, money, relativeTime } from '../format';
import { notify } from '../toast';
import type { Batch, CodeLookup, Product } from '../types';
import BatchDialog from '../components/BatchDialog.vue';
import CodesRevealDialog from '../components/CodesRevealDialog.vue';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import Money from '../components/Money.vue';
import PageHead from '../components/PageHead.vue';

const emit = defineEmits<{ changed: []; go: [string] }>();

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

const productName = computed(() => (id: string) => products.value.find((p) => p.product_id === id)?.name ?? id);

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const [b, p] = await Promise.all([api.batches(), api.products()]);
    batches.value = b.items;
    products.value = p.items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

function created(batch: Batch, codes: string[]) {
  creating.value = false;
  revealed.value = { batch, codes };
  load();
}

async function voidBatch(batch: Batch) {
  if (!window.confirm(`作废批次 #${batch.id} 里还没兑换的 ${batch.unused} 个码？已经兑换的不受影响，作废后不能恢复。`)) return;
  error.value = '';
  try {
    const result = await api.voidBatch(batch.id);
    notify(`已作废 ${result.voided} 个码`);
    await load();
  } catch (e) {
    error.value = errorMessage(e);
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
  if (found.value) found.value = (await api.lookupCode(query.value.trim())).code;
}

async function voidFoundCode() {
  if (!found.value || !window.confirm('作废这个码？作废后没人能再兑换它。')) return;
  acting.value = true;
  lookupError.value = '';
  try {
    await api.voidCode(found.value.id);
    notify('兑换码已作废');
    await Promise.all([refreshFound(), load()]);
  } catch (e) {
    lookupError.value = errorMessage(e);
  } finally {
    acting.value = false;
  }
}

async function clawBack() {
  const hit = found.value;
  if (!hit?.bucket_id || !window.confirm(`扣回这个码发放的额度（现在还剩 ${money(hit.bucket_remaining ?? 0)}）？已经用掉的不会退回，此操作不能撤销。`)) return;
  acting.value = true;
  lookupError.value = '';
  try {
    const result = await api.voidBucket(hit.bucket_id, `兑换码 #${hit.id} 退款`);
    notify(`额度已扣回，用户少了 ${money(result.voided)}`);
    await refreshFound();
    emit('changed');
  } catch (e) {
    lookupError.value = errorMessage(e);
  } finally {
    acting.value = false;
  }
}

const statusTag = (status: string) => (status === 'unused' ? 'green' : status === 'void' ? 'rose' : 'sky');

function share(count: number, b: Batch): string {
  return b.total > 0 ? `${(count / b.total) * 100}%` : '0%';
}
</script>

<template>
  <div>
    <PageHead title="兑换码" desc="在淘宝上卖的额度，用兑换码发给买家。买家在客户端输入码，额度就进他的账户。">
      <button type="button" class="btn primary" :disabled="products.length === 0" @click="creating = true"><Icon name="plus" :size="16" />生成兑换码</button>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

    <section class="panel lookup" aria-label="查询一个码">
      <div class="lead">
        <span class="ic"><Icon name="search" :size="18" /></span>
        <div>
          <h2>查询一个码</h2>
          <p class="muted">买家来退款或反馈问题时，把他给你的码粘贴到这里：看有没有被用过、被谁用、还剩多少，并直接处理。</p>
        </div>
      </div>
      <form class="qform" role="search" @submit.prevent="search">
        <label class="sr" for="code-q">兑换码</label>
        <input id="code-q" v-model="query" class="input code" placeholder="XXXX-XXXX-XXXX-XXXX" autocomplete="off" spellcheck="false" />
        <button type="submit" class="btn primary" :disabled="looking || !query.trim()">{{ looking ? '查询中…' : '查询' }}</button>
      </form>

      <div v-if="lookupError" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ lookupError }}</span></div>

      <div v-if="found" class="found">
        <div class="ftop">
          <span class="tag" :class="statusTag(found.status)">{{ codeStatusLabels[found.status] }}</span>
          <Money :points="found.quota" size="lg" />
          <span class="tag" :class="found.kind === 'monthly' ? 'amber' : 'green'">{{ kindLabels[found.kind] }}</span>
          <span class="muted">{{ productName(found.product_id) }}，批次 #{{ found.batch_id }}<template v-if="found.note">（{{ found.note }}）</template></span>
        </div>

        <dl>
          <template v-if="found.status === 'redeemed'">
            <div><dt>兑换账号</dt><dd class="mono">{{ found.redeemed_by }}</dd></div>
            <div><dt>兑换时间</dt><dd class="num">{{ formatTime(found.redeemed_at) }}（{{ relativeTime(found.redeemed_at) }}）</dd></div>
            <div>
              <dt>这份额度</dt>
              <dd class="num">
                <template v-if="found.bucket_closed_reason">{{ closedLabels[found.bucket_closed_reason] ?? '已结束' }}</template>
                <template v-else>还剩 <Money :points="found.bucket_remaining ?? 0" size="sm" />，{{ formatDate(found.bucket_expires_at) }} 到期</template>
              </dd>
            </div>
          </template>
          <div v-else-if="found.status === 'void'"><dt>作废时间</dt><dd class="num">{{ formatTime(found.voided_at) }}</dd></div>
          <div v-else><dt>有效期</dt><dd>兑换后 {{ found.valid_days }} 天</dd></div>
        </dl>

        <div class="fact">
          <button v-if="found.status === 'unused'" type="button" class="btn danger" :disabled="acting" @click="voidFoundCode"><Icon name="x" :size="15" />作废这个码</button>
          <button v-if="found.status === 'redeemed' && !found.bucket_closed_reason" type="button" class="btn danger" :disabled="acting" @click="clawBack"><Icon name="undo" :size="15" />扣回这份额度</button>
          <span v-if="found.status === 'redeemed' && found.bucket_closed_reason" class="muted">额度已经结束，没有可处理的。</span>
        </div>
      </div>
    </section>

    <section class="section">
      <div class="section-head"><h2>已生成的批次</h2><span v-if="batches.length" class="muted num">最近 {{ batches.length }} 批</span></div>
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr><th>批次</th><th>产品</th><th>内容</th><th>兑换进度</th><th></th></tr>
          </thead>
          <tbody>
            <tr v-for="b in batches" :key="b.id">
              <td>
                <div class="bn num"><strong>#{{ b.id }}</strong><span class="faint">{{ relativeTime(b.created_at) }}</span></div>
                <div v-if="b.note" class="note">{{ b.note }}</div>
              </td>
              <td>{{ productName(b.product_id) }}</td>
              <td>
                <div class="what"><span class="tag" :class="b.kind === 'monthly' ? 'amber' : 'green'">{{ kindLabels[b.kind] }}</span><Money :points="b.quota" size="md" /></div>
                <div class="faint small">兑换后 {{ b.valid_days }} 天有效</div>
              </td>
              <td class="prog">
                <div class="bar" role="img" :aria-label="`未兑换 ${b.unused}，已兑换 ${b.redeemed}，已作废 ${b.void}`">
                  <i class="r" :style="{ width: share(b.redeemed, b) }"></i><i class="u" :style="{ width: share(b.unused, b) }"></i><i class="v" :style="{ width: share(b.void, b) }"></i>
                </div>
                <div class="legend num">
                  <span><i class="sw u"></i><b>{{ b.unused }}</b> 待售</span>
                  <span><i class="sw r"></i><b>{{ b.redeemed }}</b> 已兑换</span>
                  <span v-if="b.void"><i class="sw v"></i><b>{{ b.void }}</b> 作废</span>
                </div>
              </td>
              <td class="right"><button v-if="b.unused > 0" type="button" class="btn small quiet" @click="voidBatch(b)">作废剩余</button></td>
            </tr>
            <tr v-if="loading && batches.length === 0"><td colspan="5"><i class="skel" style="display: block; height: 40px"></i></td></tr>
            <tr v-if="!loading && batches.length === 0">
              <td colspan="5" class="nopad">
                <EmptyState icon="ticket" :title="products.length === 0 ? '先添加一个产品' : '还没有生成过兑换码'" :desc="products.length === 0 ? '兑换码要绑定到某个产品，先到「产品与档位」添加。' : '点「生成兑换码」，把码导入淘宝的自动发货库存后就可以开卖。'">
                  <button v-if="products.length === 0" type="button" class="btn primary" @click="emit('go', 'products')">去添加产品</button>
                </EmptyState>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <BatchDialog v-if="creating" :products="products" @close="creating = false" @created="created" />
    <CodesRevealDialog v-if="revealed" :batch="revealed.batch" :codes="revealed.codes" @close="revealed = null" />
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.lookup { padding: 22px 24px; display: flex; flex-direction: column; gap: 16px; }
.lead { display: flex; gap: 14px; align-items: flex-start; }
.ic { flex: none; display: grid; place-items: center; width: 38px; height: 38px; border-radius: 11px; background: var(--tide-soft); color: var(--tide); }
.lead p { margin-top: 2px; max-width: 62ch; }
.qform { display: flex; gap: 10px; max-width: 560px; }
.input.code { font-family: var(--mono); font-size: 15px; letter-spacing: 0.06em; min-height: 42px; }
.qform .btn { min-height: 42px; padding: 0 22px; }
.found { border-top: 1px solid var(--line); padding-top: 18px; display: flex; flex-direction: column; gap: 14px; }
.ftop { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; }
dl { margin: 0; display: flex; flex-direction: column; gap: 7px; }
dl > div { display: grid; grid-template-columns: 84px 1fr; gap: 12px; align-items: baseline; }
dt { color: var(--muted); font-weight: 600; font-size: 13px; }
dd { margin: 0; overflow-wrap: anywhere; }
.fact { display: flex; align-items: center; gap: 12px; }

.bn { display: flex; align-items: baseline; gap: 9px; }
.note { color: var(--muted); font-size: 12.5px; margin-top: 2px; max-width: 30ch; overflow-wrap: anywhere; }
.what { display: flex; align-items: center; gap: 9px; }
.small { font-size: 12.5px; margin-top: 3px; }
.prog { min-width: 220px; }
.bar { display: flex; height: 8px; border-radius: 4px; overflow: hidden; background: var(--line); gap: 2px; }
.bar i { display: block; height: 100%; }
.bar .r, .sw.r { background: var(--tide); }
.bar .u, .sw.u { background: #a9dccf; }
.bar .v, .sw.v { background: #c3cdd1; }
.legend { display: flex; flex-wrap: wrap; gap: 4px 14px; margin-top: 8px; font-size: 12.5px; color: var(--muted); }
.legend b { color: var(--text); }
.sw { display: inline-block; width: 8px; height: 8px; border-radius: 2px; margin-right: 5px; }
.nopad { padding: 0 !important; }

@media (max-width: 560px) { .qform { flex-direction: column; } }
</style>
