<script setup lang="ts">
import { useModal } from '../composables/useModal';
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { clock, closedLabels, dayKey, dayTitle, formatDate, formatTime, isSoon, kindLabels, ledgerLabels, money, timeLeft } from '../format';
import { notify } from '../toast';
import type { BindingDetail, Bucket, LedgerRow } from '../types';
import Avatar from './Avatar.vue';
import GrantDialog from './GrantDialog.vue';
import VoidBucketDialog from './VoidBucketDialog.vue';
import Icon, { type IconName } from './Icon.vue';
import Money from './Money.vue';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ close: []; changed: [] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const detail = ref<BindingDetail | null>(null);
const error = ref('');
const granting = ref(false);
const voiding = ref<Bucket | null>(null);
const busy = ref(false);

const liveBuckets = computed(() => (detail.value?.buckets ?? []).filter((b) => !b.closed_at));
const closedBuckets = computed(() => (detail.value?.buckets ?? []).filter((b) => b.closed_at));
const paid = computed(() => liveBuckets.value.filter((b) => b.kind !== 'trial').reduce((sum, b) => sum + b.remaining, 0));
const openAnomalies = computed(() => (detail.value?.anomalies ?? []).filter((a) => !a.resolved_at));
const kindOf = computed(() => new Map((detail.value?.buckets ?? []).map((b) => [b.id, b.kind])));

interface Group {
  key: string;
  title: string;
  rows: LedgerRow[];
}
const groups = computed<Group[]>(() => {
  const out: Group[] = [];
  for (const row of detail.value?.ledger ?? []) {
    const key = dayKey(row.created_at);
    let group = out[out.length - 1];
    if (!group || group.key !== key) {
      group = { key, title: dayTitle(key), rows: [] };
      out.push(group);
    }
    group.rows.push(row);
  }
  return out;
});

function rowTitle(row: LedgerRow): string {
  if (row.kind !== 'grant') return ledgerLabels[row.kind] ?? row.kind;
  const kind = row.bucket_id === null ? undefined : kindOf.value.get(row.bucket_id);
  if (kind === 'trial') return '新用户试用额度';
  return kind ? `${kindLabels[kind] ?? kind}入账` : '入账';
}

const rowIcon: Record<string, IconName> = { grant: 'plus', consume: 'zap', consume_overdraft: 'zap', expire: 'hourglass', void: 'undo' };

async function load() {
  try {
    detail.value = await api.binding(props.id);
  } catch (e) {
    error.value = errorMessage(e);
  }
}
onMounted(load);

function share(b: Bucket): number {
  return b.total > 0 ? Math.max(0, Math.min(100, (b.remaining / b.total) * 100)) : 0;
}

function sourceText(b: Bucket): string {
  if (b.source.startsWith('admin:')) return `由 ${b.source.slice(6)} 补发`;
  if (b.source.startsWith('code:')) return `兑换码 #${b.source.slice(5)}`;
  return b.source === 'trial' ? '新用户试用' : b.source;
}

async function granted() {
  granting.value = false;
  notify('额度已补发，并已同步到 AI 网关');
  await load();
  emit('changed');
}

async function confirmVoid(note: string) {
  if (!voiding.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.voidBucket(voiding.value.id, note);
    notify(`额度已扣回，用户少了 ${money(result.voided)}`);
    voiding.value = null;
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
  <aside ref="root" class="drawer" role="dialog" aria-modal="true" aria-labelledby="drawer-title">
    <div class="drawer-head">
      <div v-if="detail" class="who">
        <Avatar :seed="detail.binding.uid" :size="42" />
        <div>
          <h2 id="drawer-title">用户额度明细</h2>
          <div class="uid mono">{{ detail.binding.uid }}</div>
        </div>
      </div>
      <h2 v-else id="drawer-title">用户额度明细</h2>
      <button type="button" class="btn quiet icon small" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="18" /></button>
    </div>

    <div class="drawer-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

      <template v-if="detail">
        <div class="balance">
          <div>
            <div class="k">可用余额</div>
            <Money :points="detail.binding.balance" size="xl" />
            <div class="sub num">其中付费额度 <Money :points="paid" size="sm" /></div>
          </div>
          <button type="button" class="btn primary" :disabled="detail.binding.status !== 'active'" @click="granting = true"><Icon name="plus" :size="16" />补发额度</button>
        </div>

        <div v-for="a in openAnomalies" :key="a.id" class="notice warn anomaly">
          <Icon name="alert" :size="17" />
          <span>{{ a.detail }}<span class="faint">（{{ formatTime(a.created_at) }}，到「对账异常」处理）</span></span>
        </div>

        <section>
          <h3>有效的额度</h3>
          <p v-if="liveBuckets.length === 0" class="muted none">现在没有可用额度。可以点“补发额度”给用户一些。</p>
          <ul class="buckets">
            <li v-for="b in liveBuckets" :key="b.id" :class="b.kind">
              <div class="top">
                <span class="tag" :class="b.kind === 'trial' ? 'sky' : b.kind === 'monthly' ? 'amber' : 'green'">{{ kindLabels[b.kind] ?? b.kind }}</span>
                <Money :points="b.remaining" size="lg" />
                <span class="of num muted">共 <Money :points="b.total" size="sm" /></span>
                <button type="button" class="btn small danger void" @click="voiding = b"><Icon name="undo" :size="14" />扣回</button>
              </div>
              <div class="gauge" role="img" :aria-label="`剩余 ${Math.round(share(b))}%`"><i :style="{ width: `${share(b)}%` }"></i></div>
              <div class="foot num">
                <span class="exp" :class="{ soon: isSoon(b.expires_at) }"><Icon name="clock" :size="13" />{{ timeLeft(b.expires_at) }}（{{ formatDate(b.expires_at) }}）</span>
                <span class="muted">{{ sourceText(b) }}</span>
              </div>
            </li>
          </ul>
        </section>

        <section v-if="closedBuckets.length">
          <h3>已结束的额度</h3>
          <ul class="done">
            <li v-for="b in closedBuckets" :key="b.id">
              <span class="tag">{{ kindLabels[b.kind] ?? b.kind }}</span>
              <Money :points="b.total" size="sm" muted />
              <span class="tag" :class="b.closed_reason === 'admin_void' ? 'rose' : ''">{{ closedLabels[b.closed_reason ?? ''] ?? '已结束' }}</span>
              <span class="faint num end">{{ formatTime(b.closed_at) }}</span>
            </li>
          </ul>
        </section>

        <section>
          <h3>流水</h3>
          <p v-if="groups.length === 0" class="muted none">还没有流水。</p>
          <div v-for="g in groups" :key="g.key" class="day">
            <div class="dh">{{ g.title }}</div>
            <ul class="tl">
              <li v-for="r in g.rows" :key="r.id" :class="r.kind">
                <span class="node" aria-hidden="true"><Icon :name="rowIcon[r.kind] ?? 'circle-check'" :size="13" :stroke="2.2" /></span>
                <div class="what">
                  <strong>{{ rowTitle(r) }}</strong>
                  <span v-if="r.note" class="note">{{ r.note }}</span>
                  <span class="time faint num">{{ clock(r.created_at) }}</span>
                </div>
                <div class="amt num">
                  <Money :points="r.delta" size="md" signed :class="r.delta >= 0 ? 'plus' : ''" />
                  <span class="after faint">余额 <Money :points="r.balance_after" size="sm" /></span>
                </div>
              </li>
            </ul>
          </div>
        </section>
      </template>
      <div v-else-if="!error" class="loading"><i class="skel" style="height: 74px"></i><i class="skel" style="height: 120px"></i><i class="skel" style="height: 120px"></i></div>
    </div>
  </aside>

  <GrantDialog v-if="granting" :id="props.id" @close="granting = false" @granted="granted" />

  <VoidBucketDialog v-if="voiding" :bucket="voiding" :busy="busy" @cancel="voiding = null" @confirm="confirmVoid" />
</template>

<style scoped>
.who { display: flex; align-items: center; gap: 13px; min-width: 0; }
.uid { color: var(--muted); overflow-wrap: anywhere; font-size: 12px; margin-top: 2px; }
.drawer-body { display: flex; flex-direction: column; gap: 4px; }
.notice { margin-bottom: 12px; }

.balance { display: flex; align-items: flex-end; justify-content: space-between; gap: 14px; padding: 4px 0 22px; border-bottom: 1px solid var(--line); margin-bottom: 4px; flex-wrap: wrap; }
.k { color: var(--muted); font-size: 13px; font-weight: 600; margin-bottom: 4px; }
.sub { color: var(--muted); font-size: 13px; margin-top: 6px; display: flex; align-items: baseline; gap: 6px; }

section { margin-top: 26px; }
section h3 { margin-bottom: 12px; }
.none { padding: 6px 0; }
.anomaly { margin-top: 14px; }

.buckets { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 12px; }
.buckets li { --fill: var(--tide); padding: 15px 16px 13px; border: 1px solid var(--line); border-radius: 14px; display: flex; flex-direction: column; gap: 11px; background: #fcfdfd; }
.buckets li.trial { --fill: #3a86b4; }
.buckets li.monthly { --fill: var(--amber-bright); }
.top { display: flex; align-items: center; gap: 11px; flex-wrap: wrap; }
.top .of { display: inline-flex; align-items: baseline; gap: 5px; font-size: 13px; }
.void { margin-left: auto; }
.gauge { height: 10px; border-radius: 6px; background: #e6edee; overflow: hidden; }
.gauge i { display: block; height: 100%; border-radius: 6px; background: linear-gradient(90deg, color-mix(in srgb, var(--fill) 78%, #fff), var(--fill)); transform-origin: left; animation: fill 0.7s cubic-bezier(0.2, 0.7, 0.2, 1) both; }
@keyframes fill { from { transform: scaleX(0); } }
.foot { display: flex; justify-content: space-between; gap: 12px; flex-wrap: wrap; font-size: 12.5px; }
.exp { display: inline-flex; align-items: center; gap: 5px; color: var(--muted); font-weight: 600; }
.exp.soon { color: var(--amber); }

.done { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; }
.done li { display: flex; align-items: center; gap: 10px; padding: 9px 0; border-bottom: 1px solid var(--line); }
.done li:last-child { border-bottom: none; }
.done .end { margin-left: auto; font-size: 12.5px; }

.day + .day { margin-top: 18px; }
.dh { font-size: 12.5px; font-weight: 700; color: var(--muted); padding-bottom: 8px; }
.tl { list-style: none; margin: 0; padding: 0 0 0 4px; position: relative; }
.tl::before { content: ''; position: absolute; left: 15px; top: 14px; bottom: 14px; width: 2px; background: var(--line); border-radius: 1px; }
.tl li { position: relative; display: flex; align-items: flex-start; gap: 12px; padding: 7px 0; }
.node { position: relative; z-index: 1; flex: none; display: grid; place-items: center; width: 24px; height: 24px; margin-top: 1px; border-radius: 50%; background: #e9eff0; color: var(--muted); box-shadow: 0 0 0 4px var(--surface); }
.tl li.grant .node { background: var(--tide-soft); color: var(--tide); }
.tl li.expire .node { background: var(--amber-soft); color: var(--amber); }
.tl li.void .node, .tl li.consume_overdraft .node { background: var(--rose-soft); color: var(--rose); }
.what { flex: 1; min-width: 0; display: flex; flex-wrap: wrap; align-items: baseline; gap: 2px 10px; }
.what strong { font-size: 13.5px; }
.note { color: var(--muted); font-size: 12.5px; flex-basis: 100%; order: 3; }
.time { font-size: 12px; }
.amt { display: flex; flex-direction: column; align-items: flex-end; gap: 1px; text-align: right; }
.amt .plus { color: var(--tide); }
.after { display: inline-flex; align-items: baseline; gap: 4px; font-size: 12px; }
.loading { display: flex; flex-direction: column; gap: 14px; }
</style>
