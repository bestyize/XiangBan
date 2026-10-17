<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { closedLabels, codeStatusLabels, formatDate, formatTime, kindLabels, money } from '../format';
import type { Batch, CodeLookup, Product } from '../types';
import BatchDialog from '../components/BatchDialog.vue';
import CodesRevealDialog from '../components/CodesRevealDialog.vue';

const emit = defineEmits<{ changed: [] }>();

const batches = ref<Batch[]>([]);
const products = ref<Product[]>([]);
const loading = ref(true);
const error = ref('');
const notice = ref('');

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
    notice.value = `已作废 ${result.voided} 个码`;
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
    notice.value = '兑换码已作废';
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
    notice.value = `已扣回 ${money(result.voided)}`;
    await refreshFound();
    emit('changed');
  } catch (e) {
    lookupError.value = errorMessage(e);
  } finally {
    acting.value = false;
  }
}

const statusTag = (status: string) => (status === 'unused' ? 'green' : status === 'void' ? 'red' : '');
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>兑换码</h1>
        <p>在淘宝上卖的额度，用兑换码发给买家。买家在客户端输入码，额度就进他的账户。</p>
      </div>
      <button type="button" class="btn primary" :disabled="products.length === 0" @click="creating = true">生成兑换码</button>
    </div>

    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="notice" class="notice ok" role="status">{{ notice }}</div>

    <section class="panel lookup">
      <div class="lookup-head">
        <h2>查询一个码</h2>
        <p class="muted">买家来退款或反馈问题时，把他给你的码粘贴到这里：看是否被用过、被谁用、还剩多少，并直接处理。</p>
      </div>
      <form class="lookup-form" role="search" @submit.prevent="search">
        <label class="sr" for="code-q">兑换码</label>
        <input id="code-q" v-model="query" class="input mono" placeholder="XXXX-XXXX-XXXX-XXXX" autocomplete="off" spellcheck="false" />
        <button type="submit" class="btn" :disabled="looking || !query.trim()">{{ looking ? '查询中…' : '查询' }}</button>
      </form>

      <div v-if="lookupError" class="notice error result" role="alert">{{ lookupError }}</div>

      <div v-if="found" class="result found">
        <div class="found-top">
          <span class="tag" :class="statusTag(found.status)">{{ codeStatusLabels[found.status] }}</span>
          <strong class="num">{{ kindLabels[found.kind] }} {{ money(found.quota) }}</strong>
          <span class="muted">{{ productName(found.product_id) }} · 批次 #{{ found.batch_id }}<template v-if="found.note">（{{ found.note }}）</template></span>
        </div>

        <dl>
          <template v-if="found.status === 'redeemed'">
            <div><dt>兑换账号</dt><dd class="mono">{{ found.redeemed_by }}</dd></div>
            <div><dt>兑换时间</dt><dd class="num">{{ formatTime(found.redeemed_at) }}</dd></div>
            <div>
              <dt>这份额度</dt>
              <dd class="num">
                <template v-if="found.bucket_closed_reason">{{ closedLabels[found.bucket_closed_reason] ?? '已结束' }}</template>
                <template v-else>还剩 {{ money(found.bucket_remaining ?? 0) }}，{{ formatDate(found.bucket_expires_at) }} 到期</template>
              </dd>
            </div>
          </template>
          <div v-else-if="found.status === 'void'"><dt>作废时间</dt><dd class="num">{{ formatTime(found.voided_at) }}</dd></div>
          <div v-else><dt>有效期</dt><dd>兑换后 {{ found.valid_days }} 天</dd></div>
        </dl>

        <div class="found-actions">
          <button v-if="found.status === 'unused'" type="button" class="btn danger" :disabled="acting" @click="voidFoundCode">作废这个码</button>
          <button v-if="found.status === 'redeemed' && !found.bucket_closed_reason" type="button" class="btn danger" :disabled="acting" @click="clawBack">扣回这份额度</button>
          <span v-if="found.status === 'redeemed' && found.bucket_closed_reason" class="muted">额度已经结束，没有可处理的。</span>
        </div>
      </div>
    </section>

    <section class="section">
      <div class="section-head"><h2>已生成的批次</h2></div>
      <div class="panel">
        <div class="table-wrap">
          <table class="table">
            <thead>
              <tr>
                <th>批次</th>
                <th>产品</th>
                <th>内容</th>
                <th class="right">未兑换</th>
                <th class="right">已兑换</th>
                <th class="right">已作废</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="b in batches" :key="b.id">
                <td>
                  <div class="num">#{{ b.id }} <span class="faint">{{ formatTime(b.created_at) }}</span></div>
                  <div v-if="b.note" class="note">{{ b.note }}</div>
                </td>
                <td>{{ productName(b.product_id) }}</td>
                <td class="num">
                  <span class="tag" :class="b.kind === 'monthly' ? 'amber' : 'green'">{{ kindLabels[b.kind] }}</span>
                  {{ money(b.quota) }}，{{ b.valid_days }} 天
                </td>
                <td class="right num"><strong>{{ b.unused }}</strong></td>
                <td class="right num">{{ b.redeemed }}</td>
                <td class="right num faint">{{ b.void }}</td>
                <td class="right">
                  <button v-if="b.unused > 0" type="button" class="btn small ghost" @click="voidBatch(b)">作废剩余</button>
                </td>
              </tr>
              <tr v-if="!loading && batches.length === 0">
                <td colspan="7" class="table-empty">
                  <template v-if="products.length === 0">先到「产品与档位」添加一个产品，再回来生成兑换码。</template>
                  <template v-else>还没有生成过兑换码。点「生成兑换码」，把码导入淘宝的自动发货库存后就可以开卖。</template>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>

    <BatchDialog v-if="creating" :products="products" @close="creating = false" @created="created" />
    <CodesRevealDialog v-if="revealed" :batch="revealed.batch" :codes="revealed.codes" @close="revealed = null" />
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.lookup { padding: 18px 20px; display: flex; flex-direction: column; gap: 14px; }
.lookup-head p { margin-top: 3px; max-width: 64ch; }
.lookup-form { display: flex; gap: 10px; max-width: 520px; }
.sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
.result { margin: 0; }
.found { border-top: 1px solid var(--line); padding-top: 14px; display: flex; flex-direction: column; gap: 12px; }
.found-top { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; }
dl { margin: 0; display: flex; flex-direction: column; gap: 6px; }
dl > div { display: grid; grid-template-columns: 84px 1fr; gap: 12px; align-items: baseline; }
dt { color: var(--muted); }
dd { margin: 0; overflow-wrap: anywhere; }
.found-actions { display: flex; align-items: center; gap: 12px; }
.note { color: var(--muted); font-size: 12.5px; margin-top: 2px; max-width: 28ch; overflow-wrap: anywhere; }
</style>
