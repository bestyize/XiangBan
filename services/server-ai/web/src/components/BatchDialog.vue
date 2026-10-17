<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { money, toPoints } from '../format';
import type { Batch, Product } from '../types';

const props = defineProps<{ products: Product[] }>();
const emit = defineEmits<{ close: []; created: [batch: Batch, codes: string[]] }>();

const productId = ref(props.products.find((p) => p.enabled)?.product_id ?? props.products[0]?.product_id ?? '');
const kind = ref<'topup' | 'monthly'>('topup');
const dollars = ref<number | null>(5);
const days = ref(365);
const count = ref<number | null>(10);
const note = ref('');
const busy = ref(false);
const error = ref('');

watch(kind, (value) => {
  days.value = value === 'monthly' ? 30 : 365;
});

const points = computed(() => (dollars.value && dollars.value > 0 ? toPoints(dollars.value) : 0));
const totalPoints = computed(() => points.value * (count.value && count.value > 0 ? count.value : 0));
const ready = computed(() => !!productId.value && points.value > 0 && !!count.value && count.value >= 1 && count.value <= 2000 && Number(days.value) >= 1);

async function submit() {
  busy.value = true;
  error.value = '';
  try {
    const result = await api.createBatch({
      product_id: productId.value,
      kind: kind.value,
      quota: points.value,
      valid_days: Number(days.value),
      count: Number(count.value),
      note: note.value.trim(),
    });
    emit('created', result.batch, result.codes);
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="scrim" @click="emit('close')"></div>
  <form class="dialog" role="dialog" aria-modal="true" aria-labelledby="batch-title" @submit.prevent="submit" @keydown.esc="emit('close')">
    <div class="dialog-head">
      <h2 id="batch-title">生成兑换码</h2>
      <p class="hint">一批对应淘宝上的一个商品：同一个产品、同样的类型和面额。</p>
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>

      <div class="field">
        <label for="b-product">用在哪个产品</label>
        <select id="b-product" v-model="productId" class="select" required>
          <option v-for="p in products" :key="p.product_id" :value="p.product_id">{{ p.name }}{{ p.enabled ? '' : '（已停用）' }}</option>
        </select>
        <span class="hint">码只能在这个产品的客户端里兑换。</span>
      </div>

      <fieldset class="kinds">
        <legend class="sr">类型</legend>
        <label class="kind" :class="{ on: kind === 'topup' }">
          <input v-model="kind" type="radio" value="topup" />
          <span><strong>加油包</strong><small>长期有效，适合按量买</small></span>
        </label>
        <label class="kind" :class="{ on: kind === 'monthly' }">
          <input v-model="kind" type="radio" value="monthly" />
          <span><strong>月卡</strong><small>限时额度，到期没用完就作废</small></span>
        </label>
      </fieldset>

      <div class="trio">
        <div class="field">
          <label for="b-amount">每个码的额度</label>
          <div class="input-affix">
            <input id="b-amount" v-model.number="dollars" class="input num" type="number" min="0.01" step="0.01" required />
            <span class="affix">美元</span>
          </div>
        </div>
        <div class="field">
          <label for="b-days">兑换后有效</label>
          <div class="input-affix">
            <input id="b-days" v-model.number="days" class="input num" type="number" min="1" max="3650" step="1" required />
            <span class="affix">天</span>
          </div>
        </div>
        <div class="field">
          <label for="b-count">生成数量</label>
          <div class="input-affix">
            <input id="b-count" v-model.number="count" class="input num" type="number" min="1" max="2000" step="1" required />
            <span class="affix">个</span>
          </div>
        </div>
      </div>
      <p v-if="ready" class="hint num">共 {{ count }} 个，全部兑换完会发放 {{ money(totalPoints) }} 的额度。有效期从用户兑换那天开始算。</p>

      <div class="field">
        <label for="b-note">备注（选填）</label>
        <input id="b-note" v-model="note" class="input" maxlength="100" placeholder="例如：淘宝「小伴壁纸 AI 5 美元加油包」" />
        <span class="hint">写上对应的淘宝商品，以后好分辨。</span>
      </div>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || !ready">{{ busy ? '生成中…' : '生成' }}</button>
    </div>
  </form>
</template>

<style scoped>
.trio { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.kinds { border: none; margin: 0; padding: 0; display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
.kind { display: flex; gap: 10px; align-items: flex-start; padding: 10px 12px; border: 1px solid var(--line-strong); border-radius: var(--radius); cursor: pointer; }
.kind:hover { border-color: var(--faint); }
.kind.on { border-color: var(--primary); background: var(--primary-soft); }
.kind input { margin-top: 3px; accent-color: var(--primary); }
.kind span { display: flex; flex-direction: column; }
.kind small { color: var(--muted); font-size: 12.5px; }

@media (max-width: 520px) {
  .trio { grid-template-columns: 1fr; }
}
</style>
