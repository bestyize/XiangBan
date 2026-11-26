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

const kindLabel: Record<string, string> = { external_credit: 'New API 钱包余额高于预期' };
</script>

<template>
  <div>
    <PageHead title="对账异常">
      <span class="tag outlined num"><i class="dot" :class="open.length ? 'warn' : 'ok'"></i>待处理 {{ open.length }}</span>
      <span class="tag outlined num">已处理 {{ done.length }}</span>
    </PageHead>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <div v-if="loading" class="issue-list"><i class="skel" style="height: 72px"></i><i class="skel" style="height: 72px"></i></div>

    <template v-else>
      <div v-if="items.length === 0" class="card">
        <EmptyState icon="shield" title="账目全部对齐" desc="New API 用户钱包与本地额度桶账本一致。" />
      </div>

      <template v-if="open.length">
        <div class="section-head"><h2>待复核</h2></div>
        <ul class="issue-list">
          <li v-for="i in open" :key="i.id" class="card issue-card open">
            <span class="issue-ico warn"><Icon name="alert" :size="16" /></span>
            <div class="issue-body">
              <strong>{{ kindLabel[i.kind] ?? i.kind }}</strong>
              <p>{{ i.detail }}</p>
              <div class="issue-meta num">
                <span>异常 #{{ i.id }}</span>
                <span>·</span>
                <span>绑定 #{{ i.binding_id }}</span>
                <span>·</span>
                <span :title="formatTime(i.created_at)">{{ relativeTime(i.created_at) }}</span>
              </div>
            </div>
            <div class="issue-acts">
              <button type="button" class="btn sm outline" @click="emit('openBinding', i.binding_id)">
                <Icon name="arrow-up-right" :size="13" />查账 #{{ i.binding_id }}
              </button>
              <button type="button" class="btn sm primary" @click="resolve(i)">
                <Icon name="check" :size="13" />标记已处理
              </button>
            </div>
          </li>
        </ul>
      </template>

      <div v-else-if="items.length > 0" class="card ok-card">
        <span class="issue-ico ok"><Icon name="shield" :size="18" /></span>
        <div>
          <strong>当前无待处理异常</strong>
          <p class="hint">以下为历史已确认记录。</p>
        </div>
      </div>

      <section v-if="done.length" class="section">
        <div class="section-head"><h2>已处理</h2></div>
        <ul class="issue-list">
          <li v-for="i in done" :key="i.id" class="card issue-card resolved">
            <span class="issue-ico ok"><Icon name="circle-check" :size="16" /></span>
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
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.issue-card.open { border-left: 3px solid var(--warn); }
.issue-card.resolved { opacity: 0.65; }
.issue-ico {
  width: 32px;
  height: 32px;
  border-radius: var(--r-inner);
  flex: none;
  display: grid;
  place-items: center;
  background: var(--subtle);
  color: var(--ink-2);
}
.issue-ico.warn { color: var(--warn); }
.issue-ico.ok { color: var(--ok); }
.issue-body { flex: 1; min-width: 240px; display: flex; flex-direction: column; gap: 3px; }
.issue-body p { font-size: 13px; color: var(--ink-2); max-width: 72ch; }
.issue-meta { font-size: 12px; color: var(--ink-3); display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.issue-acts { display: flex; align-items: center; gap: 8px; flex: none; flex-wrap: wrap; }
.ok-card { padding: 14px 16px; display: flex; align-items: center; gap: 12px; }
</style>
