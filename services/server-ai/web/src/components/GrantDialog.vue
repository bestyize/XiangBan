<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { useModal } from '../composables/useModal';
import { money, toPoints, xiaobanPoints } from '../format';
import Icon from './Icon.vue';
import KindPicker from './KindPicker.vue';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ close: []; granted: [] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const kind = ref<'topup' | 'monthly'>('topup');
const dollars = ref<number | null>(null);
const days = ref(365);
const note = ref('');
const busy = ref(false);
const error = ref('');

watch(kind, (value) => {
  days.value = value === 'monthly' ? 30 : 365;
});

const points = computed(() => (dollars.value && dollars.value > 0 ? toPoints(dollars.value) : 0));

async function submit() {
  if (points.value <= 0) return;
  busy.value = true;
  error.value = '';
  try {
    await api.grant(props.id, { quota: points.value, kind: kind.value, valid_days: Number(days.value), note: note.value.trim() });
    emit('granted');
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="scrim over" @click="emit('close')"></div>
  <form ref="root" class="dialog over" role="dialog" aria-modal="true" aria-labelledby="grant-title" @submit.prevent="submit">
    <div class="dialog-head">
      <div>
        <h2 id="grant-title">补发额度 · 绑定 #{{ id }}</h2>
        <p>补发后立即计入用户可用余额并同步至 New API，同时写入流水和操作审计。</p>
      </div>
      <button type="button" class="btn icon sm ghost" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="16" /></button>
    </div>

    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

      <KindPicker v-model="kind" />

      <div class="pair-grid">
        <div class="field">
          <label for="g-amount">金额（$1 = 100 点）</label>
          <div class="affix-wrap">
            <input id="g-amount" v-model.number="dollars" class="input num" type="number" min="0.01" step="0.01" required autofocus />
            <span class="affix">美元</span>
          </div>
        </div>
        <div class="field">
          <label for="g-days">有效期</label>
          <div class="affix-wrap">
            <input id="g-days" v-model.number="days" class="input num" type="number" min="1" max="3650" step="1" required />
            <span class="affix">天</span>
          </div>
        </div>
      </div>

      <p v-if="points > 0" class="summary-hint num">
        将补发 <strong>{{ money(points) }}</strong>（折合 <strong>{{ xiaobanPoints(points) }}</strong>），到期未用完部分自动作废。
      </p>

      <div class="field">
        <label for="g-note">备注（选填）</label>
        <input id="g-note" v-model="note" class="input" maxlength="100" placeholder="例如：客服补偿 / 淘宝订单 1234567" />
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || points <= 0">{{ busy ? '补发中…' : '确认补发' }}</button>
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
