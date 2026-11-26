<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import {
  clock,
  closedLabels,
  dayKey,
  dayTitle,
  formatDate,
  formatTime,
  isSoon,
  kindLabels,
  ledgerLabels,
  money,
  relativeTime,
  timeLeft,
  xiaobanPoints,
} from '../format';
import { useModal } from '../composables/useModal';
import { notify } from '../toast';
import type { BindingDetail, Bucket, LedgerRow } from '../types';
import Avatar from './Avatar.vue';
import ConfirmDialog from './ConfirmDialog.vue';
import GrantDialog from './GrantDialog.vue';
import Icon, { type IconName } from './Icon.vue';
import Money from './Money.vue';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{ close: []; changed: []; goIssues: [] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const detail = ref<BindingDetail | null>(null);
const error = ref('');
const granting = ref(false);
const voiding = ref<Bucket | null>(null);
const busy = ref(false);
const showClosed = ref(false);
const ledgerFilter = ref<'all' | 'grant' | 'consume' | 'other'>('all');

const liveBuckets = computed(() => (detail.value?.buckets ?? []).filter((b) => !b.closed_at));
const closedBuckets = computed(() => (detail.value?.buckets ?? []).filter((b) => b.closed_at));
const paid = computed(() => liveBuckets.value.filter((b) => b.kind !== 'trial').reduce((sum, b) => sum + b.remaining, 0));
const openAnomalies = computed(() => (detail.value?.anomalies ?? []).filter((a) => !a.resolved_at));
const kindOf = computed(() => new Map((detail.value?.buckets ?? []).map((b) => [b.id, b.kind])));

const filteredLedger = computed(() => {
  const rows = detail.value?.ledger ?? [];
  if (ledgerFilter.value === 'all') return rows;
  if (ledgerFilter.value === 'grant') return rows.filter((r) => r.kind === 'grant');
  if (ledgerFilter.value === 'consume') return rows.filter((r) => r.kind === 'consume' || r.kind === 'consume_overdraft');
  return rows.filter((r) => r.kind === 'expire' || r.kind === 'void');
});

interface Group {
  key: string;
  title: string;
  rows: LedgerRow[];
}

const groups = computed<Group[]>(() => {
  const out: Group[] = [];
  for (const row of filteredLedger.value) {
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

const rowIcon: Record<string, IconName> = {
  grant: 'plus',
  consume: 'zap',
  consume_overdraft: 'zap',
  expire: 'hourglass',
  void: 'undo',
};

async function load() {
  error.value = '';
  try {
    detail.value = await api.binding(props.id);
  } catch (e) {
    error.value = errorMessage(e);
  }
}

onMounted(load);
watch(() => props.id, () => {
  detail.value = null;
  ledgerFilter.value = 'all';
  showClosed.value = false;
  load();
});

function share(b: Bucket): number {
  return b.total > 0 ? Math.max(0, Math.min(100, (b.remaining / b.total) * 100)) : 0;
}

function sourceText(b: Bucket): string {
  if (b.source.startsWith('admin:')) return `由 ${b.source.slice(6)} 补发`;
  if (b.source.startsWith('code:')) return `兑换码 #${b.source.slice(5)}`;
  return b.source === 'trial' ? '新用户试用' : b.source;
}

async function copyUid(uid: string) {
  try {
    await navigator.clipboard.writeText(uid);
    notify('已复制完整 UID');
  } catch {
    notify('复制失败，请手动复制', 'warn');
  }
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
    notify(`额度已扣回，用户减少 ${money(result.voided)}（${xiaobanPoints(result.voided)}）`);
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
        <Avatar :seed="detail.binding.uid" :size="38" />
        <div class="who-txt">
          <div class="who-top">
            <h2 id="drawer-title">绑定 #{{ detail.binding.id }}</h2>
            <span class="tag">{{ detail.binding.product_id }}</span>
          </div>
          <div class="uid-row">
            <span class="uid mono truncate" :title="detail.binding.uid">{{ detail.binding.uid }}</span>
            <button type="button" class="copy-mini" aria-label="复制完整 UID" title="复制完整 UID" @click="copyUid(detail.binding.uid)">
              <Icon name="copy" :size="12" />
            </button>
          </div>
        </div>
      </div>
      <h2 v-else id="drawer-title">用户额度明细</h2>
      <button type="button" class="btn ghost icon sm" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="16" /></button>
    </div>

    <div class="drawer-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

      <template v-if="detail">
        <div class="balance-hero">
          <div>
            <div class="k">当前可用余额</div>
            <Money :points="detail.binding.balance" size="xl" />
            <div class="sub num">
              其中付费额度 <strong>{{ money(paid) }}</strong> · 最近同步 {{ relativeTime(detail.binding.last_synced_at) }}
            </div>
          </div>
          <button type="button" class="btn primary" :disabled="detail.binding.status !== 'active'" @click="granting = true">
            <Icon name="plus" :size="14" />补发额度
          </button>
        </div>

        <div v-for="a in openAnomalies" :key="a.id" class="notice warn">
          <Icon name="alert" :size="16" />
          <span>{{ a.detail }}<span class="faint">（{{ formatTime(a.created_at) }}）</span></span>
          <button type="button" class="btn sm notice-act" @click="emit('goIssues')">去处理</button>
        </div>

        <section>
          <div class="section-head">
            <h3>有效额度桶 <span class="tag num">{{ liveBuckets.length }}</span></h3>
            <span class="hint">按到期时间先后自动扣费</span>
          </div>
          <p v-if="liveBuckets.length === 0" class="hint">现在没有可用额度桶。可点击上方「补发额度」。</p>
          <ul v-else class="buckets">
            <li v-for="b in liveBuckets" :key="b.id" class="bucket-card" :class="b.kind">
              <div class="bucket-top">
                <span class="tag" :class="b.kind === 'trial' ? 'info' : b.kind === 'monthly' ? 'warn' : 'ok'">{{ kindLabels[b.kind] ?? b.kind }}</span>
                <Money :points="b.remaining" size="md" compact-pts />
                <span class="faint num" style="font-size: 12px">/ {{ money(b.total) }}</span>
                <button type="button" class="btn sm danger void-btn" @click="voiding = b"><Icon name="undo" :size="12" />扣回</button>
              </div>
              <div class="gauge" role="img" :aria-label="`剩余 ${Math.round(share(b))}%`"><i :style="{ width: `${share(b)}%` }"></i></div>
              <div class="bucket-foot num">
                <span class="exp" :class="{ soon: isSoon(b.expires_at) }">
                  <Icon name="clock" :size="12" />{{ timeLeft(b.expires_at) }}（{{ formatDate(b.expires_at) }}）
                </span>
                <span>{{ sourceText(b) }}</span>
              </div>
            </li>
          </ul>
        </section>

        <section v-if="closedBuckets.length">
          <button type="button" class="btn sm ghost fold-btn" @click="showClosed = !showClosed">
            <Icon :name="showClosed ? 'chevron-down' : 'chevron-right'" :size="14" />
            <span>已结束的额度桶（{{ closedBuckets.length }}）</span>
          </button>
          <ul v-if="showClosed" class="done-list">
            <li v-for="b in closedBuckets" :key="b.id">
              <span class="tag">{{ kindLabels[b.kind] ?? b.kind }}</span>
              <span class="num">{{ money(b.total) }}</span>
              <span class="tag" :class="b.closed_reason === 'admin_void' ? 'danger' : ''">{{ closedLabels[b.closed_reason ?? ''] ?? '已结束' }}</span>
              <span class="faint num end">{{ formatTime(b.closed_at) }}</span>
            </li>
          </ul>
        </section>

        <section>
          <div class="section-head">
            <h3>近期流水</h3>
            <div class="seg-pills">
              <button type="button" :class="{ active: ledgerFilter === 'all' }" @click="ledgerFilter = 'all'">全部</button>
              <button type="button" :class="{ active: ledgerFilter === 'grant' }" @click="ledgerFilter = 'grant'">入账</button>
              <button type="button" :class="{ active: ledgerFilter === 'consume' }" @click="ledgerFilter = 'consume'">消耗</button>
              <button type="button" :class="{ active: ledgerFilter === 'other' }" @click="ledgerFilter = 'other'">扣回/过期</button>
            </div>
          </div>

          <p v-if="groups.length === 0" class="hint">当前筛选下没有流水记录。</p>
          <div v-for="g in groups" :key="g.key" class="day">
            <div class="dh">{{ g.title }}</div>
            <ul class="ledger-list">
              <li v-for="r in g.rows" :key="r.id" class="ledger-item" :class="r.kind">
                <span class="ledger-dot" aria-hidden="true"><Icon :name="rowIcon[r.kind] ?? 'circle-check'" :size="12" :stroke="2.2" /></span>
                <div class="what">
                  <strong>{{ rowTitle(r) }}</strong>
                  <div class="faint meta-line">
                    <span v-if="r.note">{{ r.note }} · </span>
                    <span class="num">{{ clock(r.created_at) }}</span>
                  </div>
                </div>
                <div class="amt num">
                  <Money :points="r.delta" size="sm" signed hide-pts />
                  <span class="faint after">余 {{ money(r.balance_after) }}</span>
                </div>
              </li>
            </ul>
          </div>
        </section>
      </template>

      <div v-else-if="!error" class="loading">
        <i class="skel" style="height: 84px"></i>
        <i class="skel" style="height: 110px"></i>
        <i class="skel" style="height: 140px"></i>
      </div>
    </div>
  </aside>

  <GrantDialog v-if="granting" :id="props.id" @close="granting = false" @granted="granted" />
  <ConfirmDialog
    v-if="voiding"
    title="扣回这个额度桶？"
    :desc="`将扣回这份${kindLabels[voiding.kind] ?? ''}额度里剩余的 ${money(voiding.remaining)}（${xiaobanPoints(voiding.remaining)}），用户将立即无法再用。已消耗部分不退回，此操作不可撤销。`"
    confirm-text="确认扣回"
    over
    with-note
    :busy="busy"
    @cancel="voiding = null"
    @confirm="confirmVoid"
  />
</template>

<style scoped>
.who { display: flex; align-items: center; gap: 12px; min-width: 0; }
.who-txt { min-width: 0; }
.who-top { display: flex; align-items: center; gap: 7px; }
.who-top h2 { font-size: 15px; font-weight: 700; }
.uid-row { display: flex; align-items: center; gap: 5px; margin-top: 2px; }
.uid { color: var(--ink-3); font-size: 12px; max-width: 300px; }

.balance-hero {
  padding: 16px 18px;
  border-radius: var(--r-md);
  background: var(--surface-muted);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}
.k { color: var(--ink-3); font-size: 12px; font-weight: 600; margin-bottom: 4px; }
.sub { color: var(--ink-3); font-size: 12px; margin-top: 5px; }
.sub strong { color: var(--ink); }

section h3 { font-size: 14px; font-weight: 680; display: flex; align-items: center; gap: 7px; }

.buckets { display: flex; flex-direction: column; gap: 10px; }
.bucket-card {
  padding: 13px 15px;
  border-radius: var(--r-md);
  border: 1px solid var(--line);
  background: var(--surface);
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.bucket-top { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.void-btn { margin-left: auto; }
.gauge { height: 7px; border-radius: 99px; background: var(--canvas-soft); overflow: hidden; }
.gauge i { display: block; height: 100%; border-radius: 99px; background: var(--ok); }
.bucket-card.monthly .gauge i { background: var(--warn); }
.bucket-card.trial .gauge i { background: var(--info); }
.bucket-foot { display: flex; align-items: center; justify-content: space-between; gap: 8px; flex-wrap: wrap; font-size: 12px; color: var(--ink-3); }
.exp { display: inline-flex; align-items: center; gap: 5px; }
.exp.soon { color: var(--warn); font-weight: 600; }

.fold-btn { padding-left: 0; }
.done-list { margin-top: 8px; display: flex; flex-direction: column; gap: 6px; }
.done-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--surface-muted);
  font-size: 12px;
}
.done-list .end { margin-left: auto; }

.day + .day { margin-top: 16px; }
.dh { font-size: 12px; font-weight: 680; color: var(--ink-3); margin-bottom: 6px; }
.ledger-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 9px 0;
  border-bottom: 1px solid var(--line);
}
.ledger-item:last-child { border-bottom: 0; }
.ledger-dot {
  width: 24px;
  height: 24px;
  border-radius: 7px;
  flex: none;
  display: grid;
  place-items: center;
  background: var(--canvas-soft);
  color: var(--ink-2);
  margin-top: 1px;
}
.ledger-item.grant .ledger-dot { background: var(--ok-soft); color: var(--ok); }
.ledger-item.expire .ledger-dot { background: var(--warn-soft); color: var(--warn); }
.ledger-item.void .ledger-dot, .ledger-item.consume_overdraft .ledger-dot { background: var(--danger-soft); color: var(--danger); }
.what { flex: 1; min-width: 0; }
.what strong { font-size: 13px; font-weight: 600; }
.meta-line { font-size: 12px; margin-top: 1px; }
.amt { display: flex; flex-direction: column; align-items: flex-end; gap: 1px; flex: none; }
.after { font-size: 11.5px; }
.loading { display: flex; flex-direction: column; gap: 14px; }
</style>
