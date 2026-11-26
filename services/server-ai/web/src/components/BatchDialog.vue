<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { useModal } from '../composables/useModal';
import { money, toPoints, xiaobanPoints } from '../format';
import type { Batch, Product } from '../types';
import CustomSelect from './CustomSelect.vue';
import Icon from './Icon.vue';
import KindPicker from './KindPicker.vue';

const props = defineProps<{ products: Product[] }>();
const emit = defineEmits<{ close: []; created: [batch: Batch, codes: string[]] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const productId = ref(props.products.find((p) => p.enabled)?.product_id ?? props.products[0]?.product_id ?? '');
const kind = ref<'topup' | 'monthly'>('topup');
const dollars = ref<number | null>(5);
const days = ref(365);
const count = ref<number | null>(20);
const note = ref('');
const busy = ref(false);
const error = ref('');

const productOptions = computed(() =>
  props.products.map((p) => ({
    value: p.product_id,
    label: `${p.name}${p.enabled ? '' : '（已停用）'}`,
    desc: p.product_id,
  })),
);

watch(kind, (value) => {
  days.value = value === 'monthly' ? 30 : 365;
});

const points = computed(() => (dollars.value && dollars.value > 0 ? toPoints(dollars.value) : 0));
const totalPoints = computed(() => points.value * (count.value && count.value > 0 ? count.value : 0));
const ready = computed(
  () => !!productId.value && points.value > 0 && !!count.value && count.value >= 1 && count.value <= 2000 && Number(days.value) >= 1,
);

async function submit() {
  if (!ready.value) return;
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
  <form ref="root" class="dialog" role="dialog" aria-modal="true" aria-labelledby="batch-title" @submit.prevent="submit">
    <div class="dialog-head">
      <div>
        <h2 id="batch-title">生成兑换码批次</h2>
        <p>一批对应淘宝上的一个商品规格；明文仅在生成后展示一次。</p>
      </div>
      <button type="button" class="btn icon sm ghost" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="16" /></button>
    </div>

    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

      <div class="field">
        <label>所属产品</label>
        <CustomSelect v-model="productId" :options="productOptions" aria-label="所属产品" />
        <span class="hint">码只能在该产品关联的客户端里兑换。</span>
      </div>

      <KindPicker v-model="kind" />

      <div class="trio-grid">
        <div class="field">
          <label for="b-amount">单码额度</label>
          <div class="affix-wrap">
            <input id="b-amount" v-model.number="dollars" class="input num" type="number" min="0.01" step="0.01" required />
            <span class="affix">美元</span>
          </div>
        </div>
        <div class="field">
          <label for="b-days">兑换后有效</label>
          <div class="affix-wrap">
            <input id="b-days" v-model.number="days" class="input num" type="number" min="1" max="3650" step="1" required />
            <span class="affix">天</span>
          </div>
        </div>
        <div class="field">
          <label for="b-count">生成数量</label>
          <div class="affix-wrap">
            <input id="b-count" v-model.number="count" class="input num" type="number" min="1" max="2000" step="1" required />
            <span class="affix">个</span>
          </div>
        </div>
      </div>

      <p v-if="ready" class="summary-hint num">
        单码折合 <strong>{{ xiaobanPoints(points) }}</strong>，本批共 <strong>{{ count }}</strong> 个，合计发行 <strong>{{ money(totalPoints) }}</strong>（有效期从兑换当日计算）。
      </p>

      <div class="field">
        <label for="b-note">商品备注（选填）</label>
        <input id="b-note" v-model="note" class="input" maxlength="100" placeholder="例如：淘宝「小伴 AI 500 点加油包」" />
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || !ready">{{ busy ? '生成中…' : '立即生成' }}</button>
    </div>
  </form>
</template>

<style scoped>
.summary-hint {
  font-size: 12.5px;
  color: var(--ink-2);
  padding: 9px 12px;
  border-radius: 9px;
  background: var(--surface-muted);
  border: 1px solid var(--line);
}
.summary-hint strong { color: var(--ink); }
</style>
