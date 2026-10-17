<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { closedLabels, formatDate, formatTime, isSoon, kindLabels, ledgerLabels, money, timeLeft } from '../format';
import type { Bucket, BindingDetail } from '../types';
import GrantDialog from './GrantDialog.vue';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ close: []; changed: [] }>();

const detail = ref<BindingDetail | null>(null);
const error = ref('');
const notice = ref('');
const granting = ref(false);
const voiding = ref<Bucket | null>(null);
const voidNote = ref('');
const busy = ref(false);

const liveBuckets = computed(() => (detail.value?.buckets ?? []).filter((b) => !b.closed_at));
const closedBuckets = computed(() => (detail.value?.buckets ?? []).filter((b) => b.closed_at));

async function load() {
  try {
    detail.value = await api.binding(props.id);
  } catch (e) {
    error.value = errorMessage(e);
  }
}
onMounted(load);

function share(b: Bucket): string {
  return b.total > 0 ? `${Math.max(0, Math.min(100, (b.remaining / b.total) * 100))}%` : '0%';
}

async function granted() {
  granting.value = false;
  notice.value = '额度已补发，并已同步到 AI 网关';
  await load();
  emit('changed');
}

async function confirmVoid() {
  if (!voiding.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.voidBucket(voiding.value.id, voidNote.value.trim());
    notice.value = `已扣回 ${money(result.voided)}`;
    voiding.value = null;
    voidNote.value = '';
    await load();
    emit('changed');
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="scrim" @click="emit('close')"></div>
  <aside class="drawer" role="dialog" aria-modal="true" aria-labelledby="drawer-title" @keydown.esc="emit('close')">
    <div class="drawer-head">
      <div>
        <h2 id="drawer-title">用户额度明细</h2>
        <div v-if="detail" class="muted mono uid">{{ detail.binding.uid }}</div>
      </div>
      <button type="button" class="btn ghost small" aria-label="关闭" @click="emit('close')">关闭</button>
    </div>

    <div class="drawer-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>
      <div v-if="notice" class="notice ok" role="status">{{ notice }}</div>

      <template v-if="detail">
        <div class="balance">
          <div>
            <div class="muted">可用余额</div>
            <div class="big num">{{ money(detail.binding.balance) }}</div>
          </div>
          <button type="button" class="btn primary" :disabled="detail.binding.status !== 'active'" @click="granting = true">补发额度</button>
        </div>

        <div v-for="a in detail.anomalies.filter((x) => !x.resolved_at)" :key="a.id" class="notice warn anomaly">
          {{ a.detail }}
          <span class="faint">（{{ formatTime(a.created_at) }}，到「对账异常」处理）</span>
        </div>

        <section>
          <h3>有效的额度</h3>
          <p v-if="liveBuckets.length === 0" class="muted empty">现在没有可用额度。可以点“补发额度”给用户一些。</p>
          <ul class="buckets">
            <li v-for="b in liveBuckets" :key="b.id">
              <div class="bucket-top">
                <span class="tag" :class="b.kind === 'trial' ? '' : 'green'">{{ kindLabels[b.kind] ?? b.kind }}</span>
                <strong class="num">{{ money(b.remaining) }}</strong>
                <span class="muted num">/ {{ money(b.total) }}</span>
                <button type="button" class="btn small danger void" @click="voiding = b">扣回</button>
              </div>
              <div class="gauge" :class="{ soon: isSoon(b.expires_at) }" role="img" :aria-label="`剩余 ${share(b)}`">
                <span :style="{ width: share(b) }"></span>
              </div>
              <div class="bucket-foot muted num">
                <span :class="{ soonText: isSoon(b.expires_at) }">{{ timeLeft(b.expires_at) }}（{{ formatDate(b.expires_at) }}）</span>
                <span>{{ b.source.startsWith('admin:') ? `由 ${b.source.slice(6)} 补发` : b.source === 'trial' ? '新用户试用' : b.source }}</span>
              </div>
            </li>
          </ul>
        </section>

        <section v-if="closedBuckets.length">
          <h3>已结束的额度</h3>
          <ul class="buckets closed">
            <li v-for="b in closedBuckets" :key="b.id">
              <div class="bucket-top">
                <span class="tag">{{ kindLabels[b.kind] ?? b.kind }}</span>
                <span class="num">{{ money(b.total) }}</span>
                <span class="tag">{{ closedLabels[b.closed_reason ?? ''] ?? '已结束' }}</span>
                <span class="faint num end">{{ formatTime(b.closed_at) }}</span>
              </div>
            </li>
          </ul>
        </section>

        <section>
          <h3>流水</h3>
          <div class="table-wrap">
            <table class="table compact">
              <thead>
                <tr><th>时间</th><th>类型</th><th class="right">变化</th><th class="right">变化后余额</th></tr>
              </thead>
              <tbody>
                <tr v-for="row in detail.ledger" :key="row.id">
                  <td class="num">{{ formatTime(row.created_at) }}</td>
                  <td>
                    {{ ledgerLabels[row.kind] ?? row.kind }}
                    <div v-if="row.note" class="faint note">{{ row.note }}</div>
                  </td>
                  <td class="right num" :class="row.delta >= 0 ? 'plus' : 'minus'">{{ row.delta >= 0 ? '+' : '−' }}{{ money(Math.abs(row.delta)) }}</td>
                  <td class="right num">{{ money(row.balance_after) }}</td>
                </tr>
                <tr v-if="detail.ledger.length === 0"><td colspan="4" class="table-empty">还没有流水。</td></tr>
              </tbody>
            </table>
          </div>
        </section>
      </template>
      <p v-else-if="!error" class="muted">加载中…</p>
    </div>
  </aside>

  <GrantDialog v-if="granting" :id="props.id" @close="granting = false" @granted="granted" />

  <template v-if="voiding">
    <div class="scrim over" @click="voiding = null"></div>
    <form class="dialog over" role="alertdialog" aria-modal="true" aria-labelledby="void-title" @submit.prevent="confirmVoid" @keydown.esc="voiding = null">
      <div class="dialog-head">
        <h2 id="void-title">扣回这份额度？</h2>
        <p class="hint">将扣回 {{ money(voiding.remaining) }} 的{{ kindLabels[voiding.kind] ?? '' }}额度，用户马上就不能再用。已经消耗的部分不会退回，此操作不能撤销。</p>
      </div>
      <div class="dialog-body">
        <div class="field">
          <label for="void-note">原因（选填）</label>
          <input id="void-note" v-model="voidNote" class="input" maxlength="100" placeholder="例如：淘宝订单退款" autofocus />
        </div>
      </div>
      <div class="dialog-foot">
        <button type="button" class="btn" @click="voiding = null">取消</button>
        <button type="submit" class="btn danger" :disabled="busy">{{ busy ? '扣回中…' : '确认扣回' }}</button>
      </div>
    </form>
  </template>
</template>

<style scoped>
.over { z-index: 60; }
.dialog.over { z-index: 61; }
.uid { overflow-wrap: anywhere; font-size: 12px; }
.balance { display: flex; align-items: flex-end; justify-content: space-between; gap: 12px; padding-bottom: 18px; border-bottom: 1px solid var(--line); margin-bottom: 18px; }
.big { font-size: 34px; font-weight: 650; line-height: 1.2; letter-spacing: -0.01em; }
section { margin-top: 24px; }
section h3 { margin-bottom: 10px; }
.empty { padding: 10px 0; }
.anomaly { margin-bottom: 10px; }
.notice { margin-bottom: 12px; }
.buckets { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
.buckets li { border: 1px solid var(--line); border-radius: var(--radius); padding: 12px 14px; display: flex; flex-direction: column; gap: 8px; }
.bucket-top { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.bucket-top .void { margin-left: auto; }
.bucket-top .end { margin-left: auto; }
.bucket-foot { display: flex; justify-content: space-between; gap: 12px; font-size: 12.5px; flex-wrap: wrap; }
.soonText { color: var(--amber); font-weight: 500; }
.closed li { padding: 8px 14px; background: #fafbfc; }
.compact th, .compact td { padding: 8px 10px; font-size: 13px; }
.note { font-size: 12px; line-height: 1.4; }
.plus { color: var(--primary); }
.minus { color: var(--ink); }
</style>
