<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { api } from '../api';
import { useTheme } from '../theme';
import type { UserRow } from '../types';
import Icon, { type IconName } from './Icon.vue';

export type NavKey = 'dashboard' | 'users' | 'clients' | 'sender' | 'audit';

const emit = defineEmits<{
  close: [];
  navigate: [page: NavKey];
  createUser: [];
  openUser: [uid: string];
  changePassword: [];
}>();

const { isDark, toggleTheme } = useTheme();
const query = ref('');
const inputEl = ref<HTMLInputElement | null>(null);
const users = ref<UserRow[]>([]);
const searching = ref(false);

interface ActionItem {
  id: string;
  group: string;
  label: string;
  sub: string;
  icon: IconName;
  keywords: string;
  run: () => void;
}

const staticActions = computed<ActionItem[]>(() => [
  { id: 'nav-dashboard', group: '前往页面', label: '概览', sub: '用户统计与验证码监控', icon: 'dashboard', keywords: 'dashboard 概览 统计 首页', run: () => emit('navigate', 'dashboard') },
  { id: 'nav-users', group: '前往页面', label: '用户管理', sub: '检索、建号、停用与会话', icon: 'users', keywords: 'users 用户 账号 列表', run: () => emit('navigate', 'users') },
  { id: 'nav-clients', group: '前往页面', label: '接入应用', sub: '管理 client_id 与启用状态', icon: 'apps', keywords: 'clients 应用 client_id 接入', run: () => emit('navigate', 'clients') },
  { id: 'nav-sender', group: '前往页面', label: '发信与注册设置', sub: 'SMTP、PNVS 与注册开关', icon: 'mail', keywords: 'sender smtp 邮件 短信 pnvs 注册 发信', run: () => emit('navigate', 'sender') },
  { id: 'nav-audit', group: '前往页面', label: '操作日志', sub: '管理员安全审计记录', icon: 'scroll', keywords: 'audit 日志 审计 操作', run: () => emit('navigate', 'audit') },
  { id: 'act-create-user', group: '快捷操作', label: '创建新用户', sub: '使用用户名与初始密码建号', icon: 'user-plus', keywords: 'create 新建 创建 用户', run: () => emit('createUser') },
  { id: 'act-theme', group: '快捷操作', label: isDark.value ? '切换到暖白浅色主题' : '切换到极夜深色主题', sub: '外观模式', icon: isDark.value ? 'sun' : 'moon', keywords: 'theme dark light 深色 浅色 主题 夜间', run: () => { toggleTheme(); emit('close'); } },
  { id: 'act-pwd', group: '快捷操作', label: '修改管理员密码', sub: '修改后所有管理员会话将失效', icon: 'key', keywords: 'password 密码 安全 管理员', run: () => emit('changePassword') },
]);

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return staticActions.value;
  return staticActions.value.filter(
    (item) =>
      item.label.toLowerCase().includes(q) ||
      item.sub.toLowerCase().includes(q) ||
      item.keywords.toLowerCase().includes(q),
  );
});

const grouped = computed(() => {
  const map = new Map<string, ActionItem[]>();
  for (const item of filtered.value) {
    const list = map.get(item.group) ?? [];
    list.push(item);
    map.set(item.group, list);
  }
  return Array.from(map.entries());
});

let timer: ReturnType<typeof setTimeout> | undefined;
let seq = 0;
watch(query, (val) => {
  clearTimeout(timer);
  const q = val.trim();
  if (q.length < 2) {
    users.value = [];
    searching.value = false;
    return;
  }
  timer = setTimeout(async () => {
    const mine = ++seq;
    searching.value = true;
    try {
      const res = await api.users({ q, status: '', page: 1, page_size: 5 });
      if (mine === seq) users.value = res.items;
    } catch {
      if (mine === seq) users.value = [];
    } finally {
      if (mine === seq) searching.value = false;
    }
  }, 240);
});

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}
onMounted(() => {
  window.addEventListener('keydown', onKey);
  inputEl.value?.focus();
});
onUnmounted(() => {
  clearTimeout(timer);
  window.removeEventListener('keydown', onKey);
});
</script>

<template>
  <div class="scrim over" @click="emit('close')" />
  <div class="cmd-dialog" role="dialog" aria-modal="true" aria-label="快速跳转与搜索">
    <div class="cmd-input-wrap">
      <Icon name="search" :size="16" class="faint" />
      <input
        ref="inputEl"
        v-model="query"
        type="text"
        placeholder="跳转页面、快捷操作，或直接输入用户名 / 账号 ID 查用户…"
      />
      <kbd class="cmd-kbd">ESC</kbd>
    </div>

    <div class="cmd-results">
      <template v-for="[group, items] in grouped" :key="group">
        <div class="cmd-group-h">{{ group }}</div>
        <button v-for="item in items" :key="item.id" type="button" class="cmd-item" @click="item.run()">
          <Icon :name="item.icon" :size="15" class="muted" />
          <span>{{ item.label }}</span>
          <span class="sub">{{ item.sub }}</span>
        </button>
      </template>

      <template v-if="searching || users.length">
        <div class="cmd-group-h">匹配用户</div>
        <div v-if="searching" class="cmd-hint">搜索用户中…</div>
        <button
          v-for="u in users"
          :key="u.uid"
          type="button"
          class="cmd-item"
          @click="emit('openUser', u.uid)"
        >
          <span class="mini-av">{{ u.username.slice(0, 1).toUpperCase() }}</span>
          <span>{{ u.username }}</span>
          <span class="mono faint">#{{ u.account_id }}</span>
          <span class="sub">
            <span class="tag" :class="u.status === 'active' ? 'ok' : 'danger'">
              {{ u.status === 'active' ? '正常' : '已停用' }}
            </span>
          </span>
        </button>
      </template>

      <div v-if="!grouped.length && !users.length && !searching" class="cmd-hint">
        没有找到匹配的页面、操作或用户
      </div>
    </div>
  </div>
</template>

<style scoped>
.cmd-dialog {
  position: fixed;
  top: 14%;
  left: 50%;
  transform: translateX(-50%);
  z-index: 360;
  width: min(540px, calc(100vw - 28px));
  background: var(--surface);
  border: 1px solid var(--line-strong);
  border-radius: 16px;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  animation: cmdPop 0.16s var(--ease);
}
@keyframes cmdPop {
  from {
    opacity: 0;
    transform: translate(-50%, -8px) scale(0.985);
  }
}
.cmd-input-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--line);
}
.cmd-input-wrap input {
  flex: 1;
  border: 0;
  background: none;
  outline: none;
  font-size: 14px;
}
.cmd-kbd {
  font-family: var(--mono);
  font-size: 10.5px;
  padding: 2px 6px;
  border-radius: 5px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  color: var(--ink-3);
}
.cmd-results {
  max-height: 360px;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.cmd-group-h {
  font-size: 11px;
  font-weight: 650;
  color: var(--ink-3);
  padding: 6px 10px 4px;
}
.cmd-item {
  width: 100%;
  padding: 9px 10px;
  border-radius: 9px;
  text-align: left;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--ink);
  transition: background 0.1s;
}
.cmd-item:hover {
  background: var(--canvas-soft);
}
.cmd-item .sub {
  margin-left: auto;
  font-size: 11.5px;
  color: var(--ink-3);
}
.cmd-hint {
  padding: 18px 12px;
  text-align: center;
  font-size: 12.5px;
  color: var(--ink-3);
}
.mini-av {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  background: var(--canvas-soft);
  color: var(--ink);
  font-size: 11px;
  font-weight: 700;
  display: grid;
  place-items: center;
  flex: none;
}
</style>
