<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime, relativeTime } from '../format';
import { notify } from '../toast';
import type { Anomaly } from '../types';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import PageHead from '../components/PageHead.vue';

const props = defineProps<{ refreshTick?: number }>();
const emit = defineEmits<{ changed: []; openBinding: [id: number] }>();

const items = ref<Anomaly[]>([]);
const error = ref('');
const loading = ref(true);

const open = computed(() => items.value.filter((i) => !i.resolved_at));
const done = computed(() => items.value.filter((i) => i.resolved_at));

async function load() {
  error.value = '';
  try {
    items.value = (await api.anomalies()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch(() => props.refreshTick, () => load());

async function resolve(item: Anomaly) {
  error.value = '';
  try {
    await api.resolveAnomaly(item.id);
    notify('已标记为已处理');
    await load();
    emit('changed');
  } catch (e) {
    error.value = errorMessage(e);
  }
}

const kindLabel: Record<string, string> = { external_credit: 'New API 钱包余额高于预期（外部调额）' };
</script>

<template>
  <div>
    <PageHead
      title="对账异常"
      desc="系统每分钟自动核对 New API 钱包与本地账本。检测到上游被手工加额时，系统已自动按本地账本纠偏，此处用于人工复核与确认。"
    >
      <span class="tag num" :class="open.length ? 'warn' : 'ok'">待处理 {{ open.length }}</span>
      <span class="tag num">已处理 {{ done.length }}</span>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <div v-if="loading" class="issue-list"><i class="skel" style="height: 76px"></i><i class="skel" style="height: 76px"></i></div>

    <template v-else>
      <div v-if="items.length === 0" class="card">
        <EmptyState icon="shield" title="账目全部对齐，无待处理异常" desc="New API 用户钱包与本地额度桶账本完全一致。" />
      </div>

      <template v-if="open.length">
        <div class="section-head"><h2>待确认复核</h2></div>
        <ul class="issue-list">
          <li v-for="i in open" :key="i.id" class="card issue-card open">
            <span class="issue-ico"><Icon name="alert" :size="17" /></span>
            <div class="issue-body">
              <strong>{{ kindLabel[i.kind] ?? i.kind }}</strong>
              <p>{{ i.detail }}</p>
              <div class="issue-meta num">
                <span>异常 #{{ i.id }}</span>
                <span>·</span>
                <span>绑定账号 #{{ i.binding_id }}</span>
                <span>·</span>
                <span :title="formatTime(i.created_at)">{{ relativeTime(i.created_at) }}发现</span>
              </div>
            </div>
            <div class="issue-acts">
              <button type="button" class="btn sm" @click="emit('openBinding', i.binding_id)">
                <Icon name="arrow-up-right" :size="13" />查看用户 #{{ i.binding_id }} 账本
              </button>
              <button type="button" class="btn sm primary" @click="resolve(i)">
                <Icon name="check" :size="13" />标记已处理
              </button>
            </div>
          </li>
        </ul>
      </template>

      <div v-else-if="items.length > 0" class="card ok-card">
        <span class="ok-ico"><Icon name="shield" :size="20" /></span>
        <div>
          <strong>当前没有待处理异常</strong>
          <p class="hint">以下为历史已确认的对账异常记录。</p>
        </div>
      </div>

      <section v-if="done.length" class="section">
        <div class="section-head"><h2>已处理记录</h2></div>
        <ul class="issue-list">
          <li v-for="i in done" :key="i.id" class="card issue-card resolved">
            <span class="issue-ico ok"><Icon name="circle-check" :size="17" /></span>
            <div class="issue-body">
              <strong>{{ kindLabel[i.kind] ?? i.kind }}</strong>
              <p>{{ i.detail }}</p>
              <div class="issue-meta num">
                <span>绑定 #{{ i.binding_id }}</span>
                <span>·</span>
                <span>{{ formatTime(i.resolved_at) }} 已确认</span>
              </div>
            </div>
            <div class="issue-acts">
              <button type="button" class="btn sm ghost" @click="emit('openBinding', i.binding_id)">查账</button>
            </div>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.issue-list { display: flex; flex-direction: column; gap: 10px; }
.issue-card {
  padding: 15px 18px;
  display: flex;
  align-items: flex-start;
  gap: 14px;
}
.issue-card.open { border-color: var(--warn-line); }
.issue-card.resolved { opacity: 0.72; background: var(--surface-muted); }
.issue-ico {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  flex: none;
  display: grid;
  place-items: center;
  background: var(--warn-soft);
  color: var(--warn);
}
.issue-ico.ok { background: var(--ok-soft); color: var(--ok); }
.issue-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
.issue-body p { font-size: 13px; color: var(--ink-2); max-width: 72ch; }
.issue-meta { font-size: 12px; color: var(--ink-3); display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.issue-acts { display: flex; align-items: center; gap: 7px; flex: none; flex-wrap: wrap; }

.ok-card { padding: 16px 18px; display: flex; align-items: center; gap: 12px; }
.ok-ico {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--ok-soft);
  color: var(--ok);
  display: grid;
  place-items: center;
  flex: none;
}

@media (max-width: 680px) {
  .issue-card { flex-wrap: wrap; }
  .issue-acts { width: 100%; padding-left: 48px; }
}
</style>
