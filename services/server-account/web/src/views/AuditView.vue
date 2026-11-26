<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import Icon, { type IconName } from '../components/Icon.vue';
import { copyWithToast } from '../toast';
import type { AuditRow } from '../types';

const emit = defineEmits<{ openUser: [uid: string] }>();

type Category = 'all' | 'user' | 'client' | 'system';

interface ActionMeta {
  label: string;
  cat: Exclude<Category, 'all'>;
  tone: '' | 'ok' | 'warn' | 'danger' | 'info';
  icon: IconName;
}

const ACTION_META: Record<string, ActionMeta> = {
  'user.create': { label: '创建用户', cat: 'user', tone: 'ok', icon: 'user-plus' },
  'user.disabled': { label: '停用用户', cat: 'user', tone: 'danger', icon: 'power' },
  'user.active': { label: '启用用户', cat: 'user', tone: 'ok', icon: 'check' },
  'user.rename': { label: '修改用户名', cat: 'user', tone: 'warn', icon: 'edit' },
  'user.revoke_sessions': { label: '下线全部设备', cat: 'user', tone: 'warn', icon: 'log-out' },
  'client.create': { label: '新增应用', cat: 'client', tone: 'ok', icon: 'plus' },
  'client.enable': { label: '启用应用', cat: 'client', tone: 'ok', icon: 'check' },
  'client.disable': { label: '停用应用', cat: 'client', tone: 'danger', icon: 'power' },
  'client.rename': { label: '重命名应用', cat: 'client', tone: 'info', icon: 'edit' },
  'sender.update': { label: '更新发信通道', cat: 'system', tone: 'info', icon: 'mail' },
  'signup_methods.update': { label: '调整注册方式', cat: 'system', tone: 'info', icon: 'shield' },
  'admin.password_change': { label: '修改管理员密码', cat: 'system', tone: 'warn', icon: 'key' },
};

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function metaOf(action: string): ActionMeta {
  return (
    ACTION_META[action] ?? {
      label: action,
      cat: action.startsWith('user.') ? 'user' : action.startsWith('client.') ? 'client' : 'system',
      tone: '',
      icon: 'scroll',
    }
  );
}

const items = ref<AuditRow[]>([]);
const error = ref('');
const loading = ref(false);
const category = ref<Category>('all');
const query = ref('');

async function load() {
  loading.value = true;
  error.value = '';
  try {
    items.value = (await api.audit(100)).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const CAT_TABS: { value: Category; label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'user', label: '用户' },
  { value: 'client', label: '应用' },
  { value: 'system', label: '发信与安全' },
];

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  return items.value.filter((row) => {
    const meta = metaOf(row.action);
    if (category.value !== 'all' && meta.cat !== category.value) return false;
    if (!q) return true;
    return (
      row.admin.toLowerCase().includes(q) ||
      row.action.toLowerCase().includes(q) ||
      meta.label.toLowerCase().includes(q) ||
      (row.target ?? '').toLowerCase().includes(q) ||
      (row.detail ?? '').toLowerCase().includes(q)
    );
  });
});

function dayHeader(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '更早';
  const now = new Date();
  const sameDay = (a: Date, b: Date) =>
    a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
  if (sameDay(d, now)) return '今天';
  const yesterday = new Date(now.getTime() - 86400000);
  if (sameDay(d, yesterday)) return '昨天';
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

const groupedByDay = computed(() => {
  const groups: { day: string; rows: AuditRow[] }[] = [];
  for (const row of filtered.value) {
    const day = dayHeader(row.created_at);
    const last = groups[groups.length - 1];
    if (last && last.day === day) last.rows.push(row);
    else groups.push({ day, rows: [row] });
  }
  return groups;
});
</script>

<template>
  <div class="page-head">
    <div>
      <h1>操作日志</h1>
      <p>记录管理员对用户、应用、发信通道与安全凭证的每次变更（展示最近 100 条）。</p>
    </div>
    <div class="page-actions">
      <button type="button" class="btn" :disabled="loading" @click="load">
        <Icon name="refresh" :size="14" />
        <span>{{ loading ? '刷新中…' : '刷新日志' }}</span>
      </button>
    </div>
  </div>

  <div class="toolbar">
    <div class="search-box">
      <Icon name="search" :size="15" class="search-ico" />
      <input
        v-model="query"
        class="input"
        type="text"
        aria-label="筛选日志"
        placeholder="按管理员、动作、UID、client_id 或备注检索…"
      />
      <button v-if="query" type="button" class="search-clear" aria-label="清除筛选" @click="query = ''">
        <Icon name="x" :size="13" />
      </button>
    </div>

    <div class="seg-pills" role="tablist" aria-label="按类型筛选">
      <button
        v-for="t in CAT_TABS"
        :key="t.value"
        type="button"
        role="tab"
        :aria-selected="category === t.value"
        :class="{ active: category === t.value }"
        @click="category = t.value"
      >
        {{ t.label }}
      </button>
    </div>

    <span class="faint num count-label">{{ filtered.length }} / {{ items.length }} 条</span>
  </div>

  <div v-if="error" class="banner danger" style="margin-bottom: 14px" role="alert">
    <Icon name="alert" :size="15" class="banner-ico" />
    <span>{{ error }}</span>
  </div>

  <div v-if="loading && !items.length" class="card">
    <div class="empty-state">正在加载操作日志…</div>
  </div>

  <div v-else-if="!groupedByDay.length" class="card">
    <div class="empty-state">
      <Icon name="scroll" :size="24" class="faint" />
      <strong>暂无匹配的操作记录</strong>
      <span>尝试清空检索词或切换分类</span>
    </div>
  </div>

  <div v-else class="audit-days">
    <section v-for="group in groupedByDay" :key="group.day" class="audit-day">
      <div class="audit-day-h">{{ group.day }} · {{ group.rows.length }} 条</div>
      <div class="card audit-list">
        <div v-for="(row, idx) in group.rows" :key="`${row.created_at}-${idx}`" class="audit-row">
          <div class="audit-ico" :class="metaOf(row.action).tone">
            <Icon :name="metaOf(row.action).icon" :size="14" />
          </div>
          <div class="audit-main">
            <div class="audit-head">
              <span class="tag" :class="metaOf(row.action).tone">{{ metaOf(row.action).label }}</span>
              <span class="mono faint">{{ row.action }}</span>
            </div>
            <div v-if="row.target || row.detail" class="audit-kv-list">
              <span v-if="row.target" class="audit-kv">
                <span class="faint">对象：</span>
                <button
                  v-if="UUID_RE.test(row.target)"
                  type="button"
                  class="mono uid-jump"
                  title="点击查看该用户详情"
                  @click="emit('openUser', row.target)"
                >
                  {{ row.target }}
                </button>
                <span v-else class="mono">{{ row.target }}</span>
                <button
                  type="button"
                  class="copy-mini"
                  title="复制对象标识"
                  @click="copyWithToast(row.target, '已复制对象标识')"
                >
                  <Icon name="copy" :size="11" />
                </button>
              </span>
              <span v-if="row.detail" class="audit-kv">
                <span class="faint">备注：</span>
                <span>{{ row.detail }}</span>
              </span>
            </div>
          </div>
          <div class="audit-right">
            <strong class="admin-who">{{ row.admin }}</strong>
            <span class="faint num">{{ formatTime(row.created_at) }}</span>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.count-label {
  margin-left: auto;
  font-size: 12.5px;
}

.audit-days {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.audit-day-h {
  font-size: 12px;
  font-weight: 680;
  color: var(--ink-3);
  margin-bottom: 8px;
  letter-spacing: 0.02em;
}

.audit-list {
  overflow: hidden;
}
.audit-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 13px 16px;
  border-bottom: 1px solid var(--line);
}
.audit-row:last-child {
  border-bottom: 0;
}

.audit-ico {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  flex: none;
  background: var(--canvas-soft);
  color: var(--ink-2);
  display: grid;
  place-items: center;
  margin-top: 1px;
}
.audit-ico.ok {
  background: var(--ok-soft);
  color: var(--ok);
}
.audit-ico.warn {
  background: var(--warn-soft);
  color: var(--warn);
}
.audit-ico.danger {
  background: var(--danger-soft);
  color: var(--danger);
}
.audit-ico.info {
  background: var(--info-soft);
  color: var(--info);
}

.audit-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.audit-head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.audit-kv-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.audit-kv {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--surface-muted);
  border: 1px solid var(--line);
  color: var(--ink-2);
  max-width: 100%;
  word-break: break-all;
}
.uid-jump {
  color: var(--info);
  text-decoration: underline;
  text-underline-offset: 2px;
}
.uid-jump:hover {
  opacity: 0.8;
}

.audit-right {
  text-align: right;
  flex: none;
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.admin-who {
  font-weight: 620;
  color: var(--ink);
}

@media (max-width: 680px) {
  .audit-row {
    flex-wrap: wrap;
  }
  .audit-right {
    width: 100%;
    flex-direction: row;
    justify-content: space-between;
    text-align: left;
    padding-left: 42px;
  }
}
</style>
