<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import ConfirmDialog from '../components/ConfirmDialog.vue';
import Icon from '../components/Icon.vue';
import { copyWithToast, showToast } from '../toast';
import type { ClientRow } from '../types';

const items = ref<ClientRow[]>([]);
const error = ref('');
const loading = ref(false);

// 新增应用弹窗
const showCreate = ref(false);
const newId = ref('');
const newName = ref('');
const createBusy = ref(false);
const createError = ref('');

// 重命名 / 启停弹窗
const activeClient = ref<ClientRow | null>(null);
const modalMode = ref<'rename' | 'toggle' | null>(null);
const modalBusy = ref(false);
const modalError = ref('');

async function load() {
  loading.value = true;
  error.value = '';
  try {
    items.value = (await api.clients()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const enabledCount = computed(() => items.value.filter((c) => c.enabled).length);
const totalSessions = computed(() => items.value.reduce((acc, c) => acc + c.active_sessions, 0));
const maxSessions = computed(() => Math.max(1, ...items.value.map((c) => c.active_sessions)));

function openCreate() {
  newId.value = '';
  newName.value = '';
  createError.value = '';
  showCreate.value = true;
}

async function submitCreate() {
  if (createBusy.value) return;
  createBusy.value = true;
  createError.value = '';
  try {
    await api.createClient(newId.value.trim(), newName.value.trim());
    showToast(`已登记应用「${newName.value.trim()}」`);
    showCreate.value = false;
    await load();
  } catch (e) {
    createError.value = errorMessage(e);
  } finally {
    createBusy.value = false;
  }
}

function openRename(client: ClientRow) {
  activeClient.value = client;
  modalError.value = '';
  modalMode.value = 'rename';
}

function openToggle(client: ClientRow) {
  activeClient.value = client;
  modalError.value = '';
  modalMode.value = 'toggle';
}

async function handleModalConfirm(val: string) {
  if (!activeClient.value || !modalMode.value) return;
  modalBusy.value = true;
  modalError.value = '';
  try {
    const c = activeClient.value;
    if (modalMode.value === 'rename') {
      if (val === c.name) {
        modalMode.value = null;
        return;
      }
      await api.updateClient(c.client_id, { name: val });
      showToast(`已将应用重命名为「${val}」`);
    } else {
      await api.updateClient(c.client_id, { enabled: !c.enabled });
      showToast(c.enabled ? `已停用「${c.name}」` : `已重新启用「${c.name}」`);
    }
    modalMode.value = null;
    activeClient.value = null;
    await load();
  } catch (e) {
    modalError.value = errorMessage(e);
  } finally {
    modalBusy.value = false;
  }
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && showCreate.value && !createBusy.value) showCreate.value = false;
}
onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="page-head">
    <div>
      <h1>接入应用</h1>
      <p>每个接入账号中心的产品对应一个应用；登录时必须携带其 <code>client_id</code>，签发令牌的 <code>aud</code> 即为此值。</p>
    </div>
    <div class="page-actions">
      <button type="button" class="btn" :disabled="loading" @click="load">
        <Icon name="refresh" :size="14" />
        <span>刷新</span>
      </button>
      <button type="button" class="btn primary" @click="openCreate">
        <Icon name="plus" :size="14" />
        <span>新增应用</span>
      </button>
    </div>
  </div>

  <div v-if="error" class="banner danger" style="margin-bottom: 14px" role="alert">
    <Icon name="alert" :size="15" class="banner-ico" />
    <span>{{ error }}</span>
  </div>

  <!-- 汇总指标条 -->
  <div class="summary-grid">
    <div class="card sum-card">
      <span class="sum-k">已登记应用</span>
      <strong class="sum-v num">{{ items.length }}</strong>
    </div>
    <div class="card sum-card">
      <span class="sum-k">运行启用中</span>
      <strong class="sum-v num">{{ enabledCount }}</strong>
    </div>
    <div class="card sum-card">
      <span class="sum-k">跨应用总有效会话</span>
      <strong class="sum-v num">{{ totalSessions.toLocaleString() }}</strong>
    </div>
  </div>

  <div class="table-card">
    <div class="table-scroll">
      <table class="table">
        <thead>
          <tr>
            <th>应用名称</th>
            <th>client_id</th>
            <th>状态</th>
            <th>有效会话</th>
            <th>创建时间</th>
            <th class="right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="client in items" :key="client.client_id">
            <td>
              <strong class="app-title">{{ client.name }}</strong>
            </td>
            <td>
              <div class="cid-cell">
                <code class="mono cid-chip">{{ client.client_id }}</code>
                <button
                  type="button"
                  class="copy-mini"
                  title="复制 client_id"
                  aria-label="复制 client_id"
                  @click="copyWithToast(client.client_id, `已复制 ${client.client_id}`)"
                >
                  <Icon name="copy" :size="12" />
                </button>
              </div>
            </td>
            <td>
              <span class="tag" :class="client.enabled ? 'ok' : 'danger'">
                <i class="dot" />{{ client.enabled ? '启用' : '已停用' }}
              </span>
            </td>
            <td>
              <div class="sess-cell">
                <span class="num sess-num">{{ client.active_sessions.toLocaleString() }}</span>
                <span class="meter"><i :style="{ width: `${Math.round((client.active_sessions / maxSessions) * 100)}%` }" /></span>
              </div>
            </td>
            <td class="nowrap muted num">{{ formatTime(client.created_at) }}</td>
            <td class="right">
              <div class="ops">
                <button type="button" class="btn sm" @click="openRename(client)">
                  <Icon name="edit" :size="12" />
                  <span>重命名</span>
                </button>
                <button
                  type="button"
                  class="btn sm"
                  :class="client.enabled ? 'danger' : ''"
                  @click="openToggle(client)"
                >
                  <Icon name="power" :size="12" />
                  <span>{{ client.enabled ? '停用' : '启用' }}</span>
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="!items.length && !loading" class="empty-state">
      <Icon name="apps" :size="24" class="faint" />
      <strong>还没有登记任何应用</strong>
      <span>点击右上角「新增应用」登记第一个接入产品</span>
    </div>
    <div v-else-if="!items.length && loading" class="empty-state">
      <span>正在加载应用列表…</span>
    </div>
  </div>

  <!-- 新增应用弹窗 -->
  <template v-if="showCreate">
    <div class="scrim" @click="!createBusy && (showCreate = false)" />
    <form class="dialog" role="dialog" aria-modal="true" aria-label="新增应用" @submit.prevent="submitCreate">
      <div class="dialog-head">
        <div>
          <h3>新增接入应用</h3>
          <p>登记后不可删除，停用将禁止新登录与刷新令牌。</p>
        </div>
        <button type="button" class="btn ghost icon sm" :disabled="createBusy" aria-label="关闭" @click="showCreate = false">
          <Icon name="x" :size="16" />
        </button>
      </div>
      <div class="dialog-body">
        <label class="field">
          <span class="field-label">client_id</span>
          <input
            v-model="newId"
            class="input mono"
            required
            minlength="2"
            maxlength="64"
            pattern="[a-z0-9][a-z0-9._\-]*"
            placeholder="例如 xiangban-tv"
            autofocus
          />
          <span class="hint">2–64 位小写字母、数字、点、下划线或连字符，且以字母或数字开头。</span>
        </label>
        <label class="field">
          <span class="field-label">应用名称</span>
          <input v-model="newName" class="input" required maxlength="40" placeholder="例如 小伴电视端" />
        </label>
        <div v-if="createError" class="banner danger" role="alert">
          <Icon name="alert" :size="15" class="banner-ico" />
          <span>{{ createError }}</span>
        </div>
      </div>
      <div class="dialog-foot">
        <button type="button" class="btn" :disabled="createBusy" @click="showCreate = false">取消</button>
        <button type="submit" class="btn primary" :disabled="createBusy">
          {{ createBusy ? '保存中…' : '登记应用' }}
        </button>
      </div>
    </form>
  </template>

  <!-- 重命名弹窗 -->
  <ConfirmDialog
    v-if="modalMode === 'rename' && activeClient"
    title="重命名应用"
    :description="`修改「${activeClient.client_id}」的展示名称（不影响 client_id 与现有令牌）。`"
    input-label="新的应用名称"
    input-placeholder="输入 1–40 个字符"
    :initial-value="activeClient.name"
    :max-length="40"
    confirm-text="保存名称"
    :busy="modalBusy"
    :error="modalError"
    @close="modalMode = null"
    @confirm="handleModalConfirm"
  />

  <!-- 停用 / 启用确认弹窗 -->
  <ConfirmDialog
    v-if="modalMode === 'toggle' && activeClient"
    :title="activeClient.enabled ? `停用「${activeClient.name}」` : `启用「${activeClient.name}」`"
    :description="
      activeClient.enabled
        ? '停用后该应用将无法登录，也无法刷新令牌；已发出的访问令牌最长再存活约 30 分钟。'
        : '重新启用后，该应用即可恢复正常登录与令牌刷新。'
    "
    :confirm-text="activeClient.enabled ? '确认停用' : '确认启用'"
    :danger="activeClient.enabled"
    :busy="modalBusy"
    :error="modalError"
    @close="modalMode = null"
    @confirm="handleModalConfirm"
  />
</template>

<style scoped>
.page-head code {
  font-family: var(--mono);
  font-size: 12px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--canvas-soft);
  color: var(--ink);
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}
.sum-card {
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.sum-k {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-3);
}
.sum-v {
  font-size: 24px;
  font-weight: 740;
  letter-spacing: -0.02em;
}

.app-title {
  font-weight: 620;
}
.cid-cell {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.cid-chip {
  padding: 2px 7px;
  border-radius: 6px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  color: var(--ink-2);
}

.sess-cell {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}
.sess-num {
  min-width: 36px;
  font-weight: 600;
}
.meter {
  display: inline-block;
  width: 72px;
  height: 5px;
  border-radius: 99px;
  background: var(--canvas-soft);
  overflow: hidden;
}
.meter i {
  display: block;
  height: 100%;
  border-radius: 99px;
  background: var(--ink);
}

.ops {
  display: inline-flex;
  justify-content: flex-end;
  gap: 6px;
  white-space: nowrap;
}

@media (max-width: 640px) {
  .summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
