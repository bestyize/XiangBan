<script setup lang="ts">
import { computed, ref } from 'vue';
import { api, errorMessage } from '../api';
import { useModal } from '../composables/useModal';
import { toDollars, toPoints, xiaobanPoints } from '../format';
import type { Product } from '../types';
import Icon from './Icon.vue';
import SwitchRow from './SwitchRow.vue';

const props = defineProps<{ product: Product | null }>();
const emit = defineEmits<{ close: []; saved: [id: string] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const isNew = computed(() => props.product === null);
const productId = ref(props.product?.product_id ?? '');
const name = ref(props.product?.name ?? '');
const creditPurchaseUrl = ref(props.product?.credit_purchase_url ?? '');
const clients = ref((props.product?.client_ids ?? []).join('\n'));
const trialDollars = ref(props.product ? toDollars(props.product.trial_quota) : 0.5);
const trialDays = ref(props.product?.trial_days ?? 7);
const enabled = ref(props.product?.enabled ?? true);
const busy = ref(false);
const error = ref('');

const clientList = computed(() =>
  Array.from(
    new Set(
      clients.value
        .split(/[\s,]+/)
        .map((c) => c.trim())
        .filter(Boolean),
    ),
  ),
);

const trialPts = computed(() => toPoints(Number(trialDollars.value) || 0));

async function submit() {
  busy.value = true;
  error.value = '';
  try {
    await api.saveProduct(productId.value.trim(), {
      name: name.value.trim(),
      credit_purchase_url: creditPurchaseUrl.value.trim(),
      client_ids: clientList.value,
      trial_quota: trialPts.value,
      trial_days: Number(trialDays.value),
      enabled: enabled.value,
    });
    emit('saved', productId.value.trim());
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="scrim" @click="emit('close')"></div>
  <form ref="root" class="dialog" role="dialog" aria-modal="true" aria-labelledby="product-title" @submit.prevent="submit">
    <div class="dialog-head">
      <h2 id="product-title">{{ isNew ? '添加产品' : '编辑产品' }}</h2>
      <button type="button" class="btn icon sm ghost" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="16" /></button>
    </div>

    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

      <div class="pair-grid">
        <div class="field">
          <label for="p-id">产品标识</label>
          <input id="p-id" v-model="productId" class="input mono" :readonly="!isNew" placeholder="wallpaper" required :autofocus="isNew" />
        </div>
        <div class="field">
          <label for="p-name">产品名称</label>
          <input id="p-name" v-model="name" class="input" maxlength="40" placeholder="小伴三端" required />
        </div>
      </div>

      <div class="field">
        <label for="p-credit-link">获取额度链接（选填）</label>
        <input id="p-credit-link" v-model="creditPurchaseUrl" class="input" type="url" maxlength="2048" placeholder="https://…" />
      </div>

      <div class="field">
        <label for="p-clients">关联客户端 ID<span class="hint">每行一个</span></label>
        <textarea id="p-clients" v-model="clients" class="textarea mono" rows="3" placeholder="xiaoban-desktop-wallpaper&#10;xiaoban-android&#10;forum-web"></textarea>
      </div>

      <div class="pair-grid">
        <div class="field">
          <label for="p-trial">试用额度<span class="hint num">≈ {{ xiaobanPoints(trialPts) }}</span></label>
          <div class="affix-wrap">
            <input id="p-trial" v-model.number="trialDollars" class="input num" type="number" min="0" step="0.01" />
            <span class="affix">美元</span>
          </div>
        </div>
        <div class="field">
          <label for="p-days">试用天数</label>
          <div class="affix-wrap">
            <input id="p-days" v-model.number="trialDays" class="input num" type="number" min="1" max="365" step="1" required />
            <span class="affix">天</span>
          </div>
        </div>
      </div>

      <div class="switch-group">
        <SwitchRow v-model="enabled" title="启用产品" desc="停用后，该产品关联的所有客户端均无法调用 AI" />
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn ghost" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy">{{ busy ? '保存中…' : '保存' }}</button>
    </div>
  </form>
</template>
