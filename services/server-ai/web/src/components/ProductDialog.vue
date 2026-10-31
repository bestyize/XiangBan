<script setup lang="ts">
import { useModal } from '../composables/useModal';
import { computed, ref } from 'vue';
import { api, errorMessage } from '../api';
import { toDollars, toPoints } from '../format';
import type { Product } from '../types';

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
  Array.from(new Set(clients.value.split(/[\s,]+/).map((c) => c.trim()).filter(Boolean))),
);

async function submit() {
  busy.value = true;
  error.value = '';
  try {
    await api.saveProduct(productId.value.trim(), {
      name: name.value.trim(),
      credit_purchase_url: creditPurchaseUrl.value.trim(),
      client_ids: clientList.value,
      trial_quota: toPoints(Number(trialDollars.value) || 0),
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
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>

      <div class="field">
        <label for="p-id">产品标识</label>
        <input id="p-id" v-model="productId" class="input mono" :readonly="!isNew" placeholder="wallpaper" required :autofocus="isNew" />
        <span class="hint">{{ isNew ? '小写字母开头，可用数字、连字符和下划线。创建后不能修改。' : '创建后不能修改。' }}</span>
      </div>
      <div class="field">
        <label for="p-name">产品名称</label>
        <input id="p-name" v-model="name" class="input" maxlength="40" placeholder="小伴壁纸" required />
      </div>
      <div class="field">
        <label for="p-credit-link">额度获取链接</label>
        <input id="p-credit-link" v-model="creditPurchaseUrl" class="input" type="url" maxlength="2048" placeholder="https://…" />
        <span class="hint">客户端“获取额度”打开此页面，可配置在线支付链接；留空时入口不可用。</span>
      </div>
      <div class="field">
        <label for="p-clients">关联的客户端</label>
        <textarea id="p-clients" v-model="clients" class="textarea mono" placeholder="xiaoban-desktop-wallpaper&#10;xiaoban-android"></textarea>
        <span class="hint">每行一个账号中心的应用标识。小伴电脑和手机须关联到同一个产品，共享试用、余额及兑换额度；一个客户端只能属于一个产品。</span>
      </div>

      <div class="pair">
        <div class="field">
          <label for="p-trial">新用户试用额度</label>
          <div class="affix-wrap">
            <input id="p-trial" v-model.number="trialDollars" class="input num" type="number" min="0" step="0.01" />
            <span class="affix">美元</span>
          </div>
        </div>
        <div class="field">
          <label for="p-days">试用有效期</label>
          <div class="affix-wrap">
            <input id="p-days" v-model.number="trialDays" class="input num" type="number" min="1" max="365" step="1" required />
            <span class="affix">天</span>
          </div>
        </div>
      </div>
      <span class="hint">用户第一次使用这个产品的 AI 时自动发放，每个账号只发一次。填 0 表示不送试用。</span>

      <label class="check"><input v-model="enabled" type="checkbox" /> 启用（停用后这个产品的所有客户端都无法使用 AI）</label>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy">{{ busy ? '保存中…' : '保存产品' }}</button>
    </div>
  </form>
</template>

<style scoped>
.pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
</style>
