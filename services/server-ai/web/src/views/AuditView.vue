<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { auditCategory, auditLabels, clock, dayKey, dayTitle, parseAuditDetail, type AuditCategory } from '../format';
import type { AuditRow } from '../types';
import EmptyState from '../components/EmptyState.vue';
import Icon, { type IconName } from '../components/Icon.vue';
import PageHead from '../components/PageHead.vue';

const items = ref<AuditRow[]>([]);
const error = ref('');
const loading = ref(true);
const category = ref<'all' | AuditCategory>('all');
const query = ref('');
const showRaw = ref(false);

const icons: Record<string, IconName> = {
  'admin.password_change': 'key',
  'product.save': 'layers',
  'tier.save': 'layers',
  'tier.delete': 'trash',
  'media.save': 'layers',
  'media.delete': 'trash',
  'binding.grant': 'plus',
  'bucket.void': 'undo',
  'anomaly.resolve': 'circle-check',
  'batch.create': 'ticket',
  'batch.void': 'x',
  'code.void': 'x',
  'settings.newapi_token': 'plug',
};

const tabs: readonly { key: 'all' | AuditCategory; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'product', label: '产品与档位' },
  { key: 'quota', label: '额度调整' },
  { key: 'code', label: '兑换码' },
  { key: 'system', label: '系统安全' },
];

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  return items.value.filter((row) => {
    if (category.value !== 'all' && auditCategory(row.action) !== category.value) return false;
    if (!q) return true;
    const label = (auditLabels[row.action] ?? row.action).toLowerCase();
    return (
      label.includes(q) ||
      row.admin.toLowerCase().includes(q) ||
      (row.target ?? '').toLowerCase().includes(q) ||
      (row.detail ?? '').toLowerCase().includes(q)
    );
  });
});

interface Group {
  key: string;
  title: string;
  rows: AuditRow[];
}

const groups = computed<Group[]>(() => {
  const out: Group[] = [];
  for (const row of filtered.value) {
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

onMounted(async () => {
  try {
    items.value = (await api.audit()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div>
    <PageHead title="操作日志" desc="记录最近 150 条管理员写操作，自动将参数结构化为易读标签，便于排障与财务审计。">
      <button type="button" class="btn sm" :class="{ primary: showRaw }" @click="showRaw = !showRaw">
        {{ showRaw ? '切回结构化标签' : '查看原始参数' }}
      </button>
    </PageHead>

    <div class="toolbar">
      <div class="seg-pills" role="tablist" aria-label="按操作分类筛选">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          role="tab"
          :class="{ active: category === tab.key }"
          :aria-selected="category === tab.key"
          @click="category = tab.key"
        >{{ tab.label }}</button>
      </div>
      <div class="search-wrap search-box">
        <Icon name="search" :size="14" />
        <label class="sr" for="audit-q">筛选日志</label>
        <input id="audit-q" v-model="query" class="input" placeholder="筛选操作人、目标或详情" autocomplete="off" />
      </div>
    </div>

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

    <div v-if="loading" class="skels"><i v-for="n in 5" :key="n" class="skel" style="height: 52px"></i></div>

    <div v-else-if="groups.length === 0" class="card">
      <EmptyState
        icon="scroll"
        :title="items.length === 0 ? '还没有操作记录' : '没有匹配的操作日志'"
        :desc="items.length === 0 ? '你在后台做的每一次改动都会按时间记录在这里。' : '试试切换分类或清空搜索关键词。'"
      />
    </div>

    <div v-for="g in groups" :key="g.key" class="audit-day">
      <h2 class="dh">{{ g.title }}</h2>
      <ul class="card audit-list">
        <li v-for="(r, i) in g.rows" :key="`${g.key}-${i}`" class="audit-row">
          <span class="audit-ico" aria-hidden="true"><Icon :name="icons[r.action] ?? 'circle-check'" :size="14" /></span>
          <div class="audit-main">
            <div class="audit-head">
              <strong>{{ auditLabels[r.action] ?? r.action }}</strong>
              <code v-if="r.target" class="tag mono">{{ r.target }}</code>
            </div>
            <template v-if="r.detail">
              <p v-if="showRaw" class="raw mono faint">{{ r.detail }}</p>
              <div v-else class="kv-list">
                <span v-for="(kv, ki) in parseAuditDetail(r.detail)" :key="ki" class="kv">{{ kv }}</span>
              </div>
            </template>
          </div>
          <div class="audit-right num">
            <strong>{{ clock(r.created_at) }}</strong>
            <span class="faint">{{ r.admin }}</span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.toolbar { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 16px; flex-wrap: wrap; }
.search-box { width: 260px; max-width: 100%; }
.notice { margin-bottom: 14px; }
.skels { display: flex; flex-direction: column; gap: 10px; }

.audit-day + .audit-day { margin-top: 22px; }
.dh { font-size: 12px; font-weight: 680; color: var(--ink-3); margin-bottom: 10px; letter-spacing: 0.02em; }
.audit-list { overflow: hidden; }
.audit-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--line);
}
.audit-row:last-child { border-bottom: 0; }
.audit-ico {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  flex: none;
  background: var(--canvas-soft);
  color: var(--ink-2);
  display: grid;
  place-items: center;
  margin-top: 1px;
}
.audit-main { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 5px; }
.audit-head { display: flex; align-items: center; flex-wrap: wrap; gap: 6px 8px; }
.kv-list { display: flex; flex-wrap: wrap; gap: 5px; }
.kv {
  font-size: 11.5px;
  padding: 1px 7px;
  border-radius: 5px;
  background: var(--surface-muted);
  border: 1px solid var(--line);
  color: var(--ink-2);
}
.raw { overflow-wrap: anywhere; max-width: 78ch; font-size: 12px; }
.audit-right {
  flex: none;
  text-align: right;
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
</style>
