<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { money, toPoints } from '../format';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ close: []; granted: [] }>();

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
  <form class="dialog over" role="dialog" aria-modal="true" aria-labelledby="grant-title" @submit.prevent="submit" @keydown.esc="emit('close')">
    <div class="dialog-head">
      <h2 id="grant-title">补发额度</h2>
      <p class="hint">用于手动处理订单、补偿用户。补发会马上出现在用户的可用余额里，并写入流水和操作日志。</p>
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>

      <fieldset class="kinds">
        <legend class="sr">额度类型</legend>
        <label class="kind" :class="{ on: kind === 'topup' }">
          <input v-model="kind" type="radio" value="topup" />
          <span><strong>加油包</strong><small>长期有效，默认 1 年</small></span>
        </label>
        <label class="kind" :class="{ on: kind === 'monthly' }">
          <input v-model="kind" type="radio" value="monthly" />
          <span><strong>月卡</strong><small>限时额度，默认 30 天</small></span>
        </label>
      </fieldset>

      <div class="pair">
        <div class="field">
          <label for="g-amount">金额</label>
          <div class="input-affix">
            <input id="g-amount" v-model.number="dollars" class="input num" type="number" min="0.01" step="0.01" required autofocus />
            <span class="affix">美元</span>
          </div>
        </div>
        <div class="field">
          <label for="g-days">有效期</label>
          <div class="input-affix">
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
.over { z-index: 60; }
.dialog.over { z-index: 61; }
.pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.kinds { border: none; margin: 0; padding: 0; display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
.kind { display: flex; gap: 10px; align-items: flex-start; padding: 10px 12px; border: 1px solid var(--line-strong); border-radius: var(--radius); cursor: pointer; }
.kind:hover { border-color: var(--faint); }
.kind.on { border-color: var(--primary); background: var(--primary-soft); }
.kind input { margin-top: 3px; accent-color: var(--primary); }
.kind span { display: flex; flex-direction: column; }
.kind small { color: var(--muted); font-size: 12.5px; }
</style>
