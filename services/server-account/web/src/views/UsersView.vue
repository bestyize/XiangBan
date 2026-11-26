<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api, errorMessage, formatRelativeTime, formatTime } from '../api';
import CreateUserDialog from '../components/CreateUserDialog.vue';
import Icon from '../components/Icon.vue';
import UserDrawer from '../components/UserDrawer.vue';
import { copyWithToast } from '../toast';
import type { UserList } from '../types';

const props = withDefaults(defineProps<{ refreshKey?: number }>(), { refreshKey: 0 });

const creating = ref(false);
const query = ref('');
const status = ref<'' | 'active' | 'disabled'>('');
const page = ref(1);
const data = ref<UserList | null>(null);
const error = ref('');
const loading = ref(false);
const selected = ref<string | null>(null);

const KIND_LABEL: Record<string, string> = { email: '邮箱', phone: '手机' };
const pageCount = computed(() =>
  Math.max(1, Math.ceil((data.value?.total ?? 0) / (data.value?.page_size ?? 20))),
);

const STATUS_TABS: { value: '' | 'active' | 'disabled'; label: string }[] = [
  { value: '', label: '全部' },
  { value: 'active', label: '正常' },
  { value: 'disabled', label: '已停用' },
];

let sequence = 0;
async function load() {
  const mine = ++sequence;
  loading.value = true;
  error.value = '';
  try {
    const result = await api.users({ q: query.value.trim(), status: status.value, page: page.value });
    if (mine === sequence) data.value = result;
  } catch (e) {
    if (mine === sequence) error.value = errorMessage(e);
  } finally {
    if (mine === sequence) loading.value = false;
  }
}

let timer: ReturnType<typeof setTimeout> | undefined;
watch(query, () => {
  clearTimeout(timer);
  timer = setTimeout(() => {
    page.value = 1;
    load();
  }, 280);
});
watch(status, () => {
  page.value = 1;
  load();
});
watch(
  () => props.refreshKey,
  () => {
    load();
  },
);
onMounted(load);

function go(delta: number) {
  page.value = Math.min(pageCount.value, Math.max(1, page.value + delta));
  load();
}

function clearSearch() {
  query.value = '';
  page.value = 1;
  load();
}

function avatarTone(name: string): string {
  const palette = ['#2d2a32', '#3d3448', '#2a3b45', '#42362d', '#2b3d35'];
  let sum = 0;
  for (let i = 0; i < name.length; i++) sum += name.charCodeAt(i);
  return palette[sum % palette.length];
}
</script>

<template>
  <div class="page-head">
    <div>
      <h1>用户管理</h1>
      <p>按账号 ID、用户名、完整邮箱 / 手机号或 UID 检索用户，点击行查看会话与安全操作。</p>
    </div>
    <div class="page-actions">
      <button type="button" class="btn" :disabled="loading" @click="load">
        <Icon name="refresh" :size="14" />
        <span>刷新</span>
      </button>
      <button type="button" class="btn primary" @click="creating = true">
        <Icon name="user-plus" :size="14" />
        <span>创建用户</span>
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
        aria-label="搜索用户"
        placeholder="搜索账号 ID / 用户名 / 完整邮箱 / 完整手机号 / UID…"
      />
      <button v-if="query" type="button" class="search-clear" aria-label="清除搜索" @click="clearSearch">
        <Icon name="x" :size="13" />
      </button>
    </div>

    <div class="seg-pills" role="tablist" aria-label="按状态筛选">
      <button
        v-for="tab in STATUS_TABS"
        :key="tab.value"
        type="button"
        role="tab"
        :aria-selected="status === tab.value"
        :class="{ active: status === tab.value }"
        @click="status = tab.value"
      >
        {{ tab.label }}
      </button>
    </div>

    <span v-if="data" class="faint num result-count">共 {{ data.total.toLocaleString() }} 位</span>
  </div>

  <div v-if="error" class="banner danger" style="margin-bottom: 14px" role="alert">
    <Icon name="alert" :size="15" class="banner-ico" />
    <span>{{ error }}</span>
  </div>

  <div class="table-card">
    <div class="table-scroll">
      <table class="table">
        <thead>
          <tr>
            <th>用户</th>
            <th>账号 ID</th>
            <th>登录标识</th>
            <th>状态</th>
            <th>注册时间</th>
            <th>最近登录</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="user in data?.items"
            :key="user.uid"
            class="clickable"
            tabindex="0"
            @click="selected = user.uid"
            @keydown.enter="selected = user.uid"
          >
            <td>
              <div class="user-cell">
                <img
                  v-if="user.avatar_url"
                  :src="user.avatar_url"
                  :alt="user.username"
                  class="avatar-img"
                  @error="($event.target as HTMLImageElement).style.display = 'none'"
                />
                <span v-else class="avatar-txt" :style="{ background: avatarTone(user.username) }">
                  {{ user.username.slice(0, 1).toUpperCase() }}
                </span>
                <div class="user-name-wrap">
                  <strong class="uname">{{ user.username }}</strong>
                  <span class="mono faint uid-short">{{ user.uid.slice(0, 8) }}</span>
                </div>
              </div>
            </td>
            <td>
              <div class="id-cell">
                <span class="mono num">{{ user.account_id }}</span>
                <button
                  type="button"
                  class="copy-mini"
                  title="复制账号 ID"
                  aria-label="复制账号 ID"
                  @click.stop="copyWithToast(user.account_id, `已复制账号 ID ${user.account_id}`)"
                >
                  <Icon name="copy" :size="12" />
                </button>
              </div>
            </td>
            <td>
              <div v-if="user.identities.length" class="idents">
                <div v-for="(identity, index) in user.identities" :key="index" class="ident-line">
                  <Icon :name="identity.kind === 'phone' ? 'phone' : 'mail'" :size="12" class="faint" />
                  <span class="faint">{{ KIND_LABEL[identity.kind] ?? identity.kind }}</span>
                  <span class="mono">{{ identity.masked }}</span>
                </div>
              </div>
              <span v-else class="tag">仅密码登录</span>
            </td>
            <td>
              <span class="tag" :class="user.status === 'active' ? 'ok' : 'danger'">
                <i class="dot" />{{ user.status === 'active' ? '正常' : '已停用' }}
              </span>
            </td>
            <td class="nowrap muted" :title="formatTime(user.created_at)">
              {{ formatRelativeTime(user.created_at) }}
            </td>
            <td class="nowrap muted" :title="formatTime(user.last_login_at)">
              {{ formatRelativeTime(user.last_login_at) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="data && !data.items.length" class="empty-state">
      <Icon name="users" :size="24" class="faint" />
      <strong>没有符合条件的用户</strong>
      <span>尝试更换搜索关键词或切换状态筛选</span>
    </div>
    <div v-else-if="!data && loading" class="empty-state">
      <span>正在加载用户列表…</span>
    </div>
  </div>

  <div v-if="data && data.total > 0" class="pager">
    <span class="faint num">
      第 {{ page }} / {{ pageCount }} 页 · 共 {{ data.total.toLocaleString() }} 位用户
    </span>
    <div class="pager-btns">
      <button type="button" class="btn sm" :disabled="page <= 1 || loading" @click="go(-1)">上一页</button>
      <button type="button" class="btn sm" :disabled="page >= pageCount || loading" @click="go(1)">下一页</button>
    </div>
  </div>

  <CreateUserDialog
    v-if="creating"
    @close="creating = false"
    @created="creating = false; load()"
  />
  <UserDrawer
    v-if="selected"
    :uid="selected"
    @close="selected = null"
    @changed="load"
  />
</template>

<style scoped>
.result-count {
  margin-left: auto;
  font-size: 12.5px;
}

.user-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}
.avatar-img,
.avatar-txt {
  width: 32px;
  height: 32px;
  border-radius: 9px;
  flex: none;
}
.avatar-img {
  object-fit: cover;
  border: 1px solid var(--line);
}
.avatar-txt {
  display: grid;
  place-items: center;
  color: #ffffff;
  font-size: 12.5px;
  font-weight: 700;
}
.user-name-wrap {
  display: flex;
  flex-direction: column;
  line-height: 1.25;
  min-width: 0;
}
.uname {
  font-weight: 620;
  font-size: 13.5px;
}
.uid-short {
  font-size: 11px;
}

.id-cell {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.idents {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.ident-line {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
  font-size: 12.5px;
}

.pager {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 14px;
  font-size: 12.5px;
}
.pager-btns {
  display: flex;
  gap: 6px;
}
</style>
