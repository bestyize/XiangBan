<script setup lang="ts">
import { useModal } from '../composables/useModal';
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { money, toPoints } from '../format';
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
      <h2 id="grant-title">补发额度</h2>
      <p class="hint">用于手动处理订单、补偿用户。补发会马上出现在用户的可用余额里，并写入流水和操作日志。</p>
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>

      <KindPicker v-model="kind" />

      <div class="pair">
        <div class="field">
          <label for="g-amount">金额</label>
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
      <p v-if="points > 0" class="hint num">将补发 {{ money(points) }}，到期未用完的部分作废。</p>

      <div class="field">
        <label for="g-note">备注（选填）</label>
        <input id="g-note" v-model="note" class="input" maxlength="100" placeholder="例如：淘宝订单 1234567" />
      </div>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || points <= 0">{{ busy ? '补发中…' : '确认补发' }}</button>
    </div>
  </form>
</template>

<style scoped>
.pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
</style>
