<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { api } from './api';
import BrandMark from './components/BrandMark.vue';
import CommandPalette, { type NavKey } from './components/CommandPalette.vue';
import CreateUserDialog from './components/CreateUserDialog.vue';
import Icon, { type IconName } from './components/Icon.vue';
import PasswordDialog from './components/PasswordDialog.vue';
import ToastHost from './components/ToastHost.vue';
import UserDrawer from './components/UserDrawer.vue';
import { session, signOut } from './session';
import { useTheme } from './theme';
import AuditView from './views/AuditView.vue';
import ClientsView from './views/ClientsView.vue';
import DashboardView from './views/DashboardView.vue';
import LoginView from './views/LoginView.vue';
import SenderView from './views/SenderView.vue';
import UsersView from './views/UsersView.vue';

interface NavItem {
  key: NavKey;
  label: string;
  icon: IconName;
  group: '工作台' | '系统与安全';
}

const navItems: readonly NavItem[] = [
  { key: 'dashboard', label: '概览', icon: 'dashboard', group: '工作台' },
  { key: 'users', label: '用户管理', icon: 'users', group: '工作台' },
  { key: 'clients', label: '接入应用', icon: 'apps', group: '工作台' },
  { key: 'sender', label: '发信与注册', icon: 'mail', group: '系统与安全' },
  { key: 'audit', label: '操作日志', icon: 'scroll', group: '系统与安全' },
];

const navGroups = computed(() => [
  { label: '工作台', items: navItems.filter((i) => i.group === '工作台') },
  { label: '系统与安全', items: navItems.filter((i) => i.group === '系统与安全') },
]);

function readHashPage(): NavKey {
  const raw = window.location.hash.replace(/^#\/?/, '');
  if (navItems.some((item) => item.key === raw)) return raw as NavKey;
  return 'dashboard';
}

const current = ref<NavKey>(readHashPage());
const sideOpen = ref(false);
const cmdOpen = ref(false);
const changingPassword = ref(false);
const quickCreateUser = ref(false);
const inspectUid = ref<string | null>(null);
const usersRefreshKey = ref(0);

const { isDark, toggleTheme } = useTheme();

const senderStatus = ref<'loading' | 'console' | 'smtp' | 'tencent' | 'error'>('loading');
const pnvsConfigured = ref(false);

const senderStatusText = computed(() => {
  switch (senderStatus.value) {
    case 'smtp':
      return pnvsConfigured.value ? 'SMTP + 阿里云短信' : 'SMTP 邮件通道';
    case 'tencent':
      return pnvsConfigured.value ? '腾讯云 + 阿里云短信' : '腾讯云发信通道';
    case 'console':
      return pnvsConfigured.value ? '仅日志邮件 + 阿里云短信' : '控制台日志模式';
    case 'error':
      return '通道状态获取失败';
    default:
      return '正在检查通道…';
  }
});

const senderStatusClass = computed(() => {
  if (senderStatus.value === 'smtp' || senderStatus.value === 'tencent') return 'ok';
  if (senderStatus.value === 'console') return 'warn';
  if (senderStatus.value === 'error') return 'bad';
  return 'checking';
});

async function refreshSenderHealth() {
  if (!session.token) return;
  senderStatus.value = 'loading';
  try {
    const cfg = await api.getSender();
    senderStatus.value = cfg.provider;
    pnvsConfigured.value = cfg.pnvs_configured;
  } catch {
    senderStatus.value = 'error';
  }
}

function go(page: NavKey) {
  current.value = page;
  sideOpen.value = false;
  if (window.location.hash !== `#/${page}`) {
    window.history.replaceState(null, '', `#/${page}`);
  }
}

function onHashChange() {
  current.value = readHashPage();
}

function onGlobalKey(e: KeyboardEvent) {
  if (!session.token) return;
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    cmdOpen.value = !cmdOpen.value;
  }
}

watch(
  () => session.token,
  (token) => {
    if (token) {
      refreshSenderHealth();
    } else {
      cmdOpen.value = false;
      sideOpen.value = false;
      changingPassword.value = false;
      quickCreateUser.value = false;
      inspectUid.value = null;
    }
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener('hashchange', onHashChange);
  window.addEventListener('keydown', onGlobalKey);
});

onUnmounted(() => {
  window.removeEventListener('hashchange', onHashChange);
  window.removeEventListener('keydown', onGlobalKey);
});

const currentLabel = computed(() => navItems.find((i) => i.key === current.value)?.label ?? '概览');
const adminInitial = computed(() => (session.username || 'A').slice(0, 1).toUpperCase());

function handlePaletteCreateUser() {
  cmdOpen.value = false;
  quickCreateUser.value = true;
}

function handlePaletteOpenUser(uid: string) {
  cmdOpen.value = false;
  inspectUid.value = uid;
}

function handlePalettePassword() {
  cmdOpen.value = false;
  changingPassword.value = true;
}

function handleUserCreated() {
  quickCreateUser.value = false;
  usersRefreshKey.value++;
  go('users');
}
</script>

<template>
  <LoginView v-if="!session.token" />

  <div v-else class="shell">
    <!-- 移动端顶栏 -->
    <header class="mobile-bar">
      <button type="button" class="btn ghost icon" aria-label="打开导航菜单" @click="sideOpen = true">
        <Icon name="menu" :size="18" />
      </button>
      <div class="mb-brand">
        <BrandMark :size="24" />
        <strong>{{ currentLabel }}</strong>
      </div>
      <div class="mb-acts">
        <button type="button" class="btn ghost icon sm" aria-label="快捷搜索" @click="cmdOpen = true">
          <Icon name="search" :size="16" />
        </button>
        <button
          type="button"
          class="btn ghost icon sm"
          :aria-label="isDark ? '切换浅色主题' : '切换深色主题'"
          @click="toggleTheme"
        >
          <Icon :name="isDark ? 'sun' : 'moon'" :size="16" />
        </button>
      </div>
    </header>

    <!-- 侧栏遮罩（移动/平板） -->
    <div v-if="sideOpen" class="side-scrim" @click="sideOpen = false" />

    <!-- 左侧导轨 -->
    <aside class="sidebar" :class="{ open: sideOpen }">
      <div class="sb-brand">
        <BrandMark :size="32" />
        <div class="sb-brand-txt">
          <strong>相伴账号中心</strong>
          <span>统一身份与安全控制台</span>
        </div>
        <button type="button" class="btn ghost icon sm sb-close" aria-label="关闭导航" @click="sideOpen = false">
          <Icon name="x" :size="16" />
        </button>
      </div>

      <button type="button" class="cmd-trigger" @click="cmdOpen = true">
        <Icon name="search" :size="14" />
        <span>快速跳转或查用户…</span>
        <kbd>⌘K</kbd>
      </button>

      <nav class="nav-scroll" aria-label="主导航">
        <div v-for="group in navGroups" :key="group.label" class="nav-group">
          <div class="nav-group-label">{{ group.label }}</div>
          <div class="nav-list">
            <button
              v-for="item in group.items"
              :key="item.key"
              type="button"
              class="nav-item"
              :class="{ active: current === item.key }"
              :aria-current="current === item.key ? 'page' : undefined"
              @click="go(item.key)"
            >
              <Icon :name="item.icon" :size="16" />
              <span>{{ item.label }}</span>
            </button>
          </div>
        </div>
      </nav>

      <div class="sb-foot">
        <button
          type="button"
          class="status-pill"
          :class="senderStatusClass"
          :title="senderStatusText"
          @click="go('sender')"
        >
          <i class="status-dot" aria-hidden="true" />
          <span class="truncate">{{ senderStatusText }}</span>
          <Icon name="chevron-right" :size="13" class="faint" />
        </button>

        <div class="sb-prefs">
          <button type="button" class="pref-btn" @click="toggleTheme">
            <Icon :name="isDark ? 'sun' : 'moon'" :size="14" />
            <span>{{ isDark ? '暖白浅色' : '极夜深色' }}</span>
          </button>
        </div>

        <div class="me-row">
          <span class="me-av" aria-hidden="true">{{ adminInitial }}</span>
          <div class="me-meta">
            <strong class="truncate">{{ session.username }}</strong>
            <span>管理员</span>
          </div>
          <button
            type="button"
            class="me-btn"
            aria-label="修改密码"
            title="修改密码"
            @click="changingPassword = true"
          >
            <Icon name="key" :size="15" />
          </button>
          <button type="button" class="me-btn" aria-label="退出登录" title="退出登录" @click="signOut">
            <Icon name="log-out" :size="15" />
          </button>
        </div>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="main-col">
      <div class="main-inner">
        <DashboardView v-if="current === 'dashboard'" @navigate="go" @open-user="(uid) => (inspectUid = uid)" />
        <UsersView v-else-if="current === 'users'" :refresh-key="usersRefreshKey" />
        <ClientsView v-else-if="current === 'clients'" />
        <SenderView v-else-if="current === 'sender'" @updated="refreshSenderHealth" />
        <AuditView v-else-if="current === 'audit'" @open-user="(uid) => (inspectUid = uid)" />
      </div>
    </main>

    <CommandPalette
      v-if="cmdOpen"
      @close="cmdOpen = false"
      @navigate="(p) => { cmdOpen = false; go(p); }"
      @create-user="handlePaletteCreateUser"
      @open-user="handlePaletteOpenUser"
      @change-password="handlePalettePassword"
    />
    <CreateUserDialog
      v-if="quickCreateUser"
      @close="quickCreateUser = false"
      @created="handleUserCreated"
    />
    <UserDrawer
      v-if="inspectUid"
      :uid="inspectUid"
      @close="inspectUid = null"
      @changed="usersRefreshKey++"
    />
    <PasswordDialog v-if="changingPassword" @close="changingPassword = false" />
  </div>

  <ToastHost />
</template>

<style scoped>
.shell {
  display: grid;
  grid-template-columns: 238px minmax(0, 1fr);
  min-height: 100vh;
}

.mobile-bar {
  display: none;
}
.side-scrim {
  display: none;
}
.sb-close {
  display: none;
  margin-left: auto;
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  padding: 16px 12px 14px;
  overflow-y: auto;
  z-index: 60;
}

.sb-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 2px 6px 14px;
}
.sb-brand-txt {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
  min-width: 0;
}
.sb-brand-txt strong {
  font-size: 14.5px;
  font-weight: 700;
  letter-spacing: -0.01em;
}
.sb-brand-txt span {
  font-size: 11px;
  color: var(--ink-3);
  margin-top: 2px;
}

.cmd-trigger {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  height: 34px;
  padding: 0 9px;
  margin-bottom: 16px;
  border-radius: 9px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  color: var(--ink-3);
  font-size: 12.5px;
  box-shadow: var(--shadow-xs);
  white-space: nowrap;
  transition: border-color 0.14s, color 0.14s;
}
.cmd-trigger > span {
  overflow: hidden;
  text-overflow: ellipsis;
}
.cmd-trigger:hover {
  border-color: var(--line-focus);
  color: var(--ink-2);
}
.cmd-trigger kbd {
  margin-left: auto;
  font-family: var(--mono);
  font-size: 10.5px;
  padding: 1px 5px;
  border-radius: 5px;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  color: var(--ink-3);
}

.nav-scroll {
  flex: 1;
  overflow-y: auto;
}
.nav-group {
  margin-bottom: 16px;
}
.nav-group-label {
  font-size: 11px;
  font-weight: 650;
  color: var(--ink-3);
  padding: 0 10px 6px;
  letter-spacing: 0.03em;
}
.nav-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  height: 36px;
  padding: 0 10px;
  border-radius: 9px;
  color: var(--ink-2);
  font-size: 13.5px;
  font-weight: 520;
  text-align: left;
  transition: background 0.14s, color 0.14s;
}
.nav-item:hover {
  background: rgba(25, 23, 26, 0.045);
  color: var(--ink);
}
:root[data-theme='dark'] .nav-item:hover {
  background: rgba(255, 255, 255, 0.055);
}
.nav-item.active {
  background: var(--surface);
  color: var(--ink);
  font-weight: 650;
  box-shadow: var(--shadow-xs), inset 0 0 0 1px var(--line);
}
.nav-item.active svg {
  color: var(--coral);
}

.sb-foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--line);
}

.status-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border-radius: 9px;
  border: 1px solid var(--line);
  background: var(--surface);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-2);
  text-align: left;
  transition: border-color 0.14s, color 0.14s;
}
.status-pill:hover {
  border-color: var(--line-strong);
  color: var(--ink);
}
.status-pill > span {
  flex: 1;
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex: none;
  background: var(--ink-3);
}
.status-pill.ok .status-dot {
  background: var(--ok);
  box-shadow: 0 0 0 3px var(--ok-soft);
}
.status-pill.warn .status-dot {
  background: var(--warn);
  box-shadow: 0 0 0 3px var(--warn-soft);
}
.status-pill.bad .status-dot {
  background: var(--danger);
  box-shadow: 0 0 0 3px var(--danger-soft);
}

.sb-prefs {
  display: flex;
  gap: 6px;
}
.pref-btn {
  flex: 1;
  height: 30px;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--surface);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: border-color 0.14s, color 0.14s;
}
.pref-btn:hover {
  border-color: var(--line-strong);
  color: var(--ink);
}

.me-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 4px 0;
}
.me-av {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  flex: none;
  background: var(--btn-primary-bg);
  color: var(--btn-primary-ink);
  font-size: 12px;
  font-weight: 700;
  display: grid;
  place-items: center;
}
.me-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}
.me-meta strong {
  font-size: 12.5px;
  font-weight: 650;
}
.me-meta span {
  font-size: 11px;
  color: var(--ink-3);
}
.me-btn {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  color: var(--ink-3);
  display: grid;
  place-items: center;
  transition: background 0.14s, color 0.14s;
}
.me-btn:hover {
  background: var(--canvas-soft);
  color: var(--ink);
}

.main-col {
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.main-inner {
  width: 100%;
  max-width: 1160px;
  margin: 0 auto;
  padding: 28px 36px 72px;
}

@media (max-width: 860px) {
  .shell {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
  }
  .mobile-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    position: sticky;
    top: 0;
    z-index: 40;
    height: 52px;
    padding: 0 12px;
    background: var(--surface-glass);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    border-bottom: 1px solid var(--line);
  }
  .mb-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14.5px;
    font-weight: 680;
  }
  .mb-acts {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .side-scrim {
    display: block;
    position: fixed;
    inset: 0;
    background: var(--scrim);
    z-index: 55;
  }
  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 268px;
    transform: translateX(-104%);
    transition: transform 0.22s var(--ease);
    box-shadow: var(--shadow-lg);
  }
  .sidebar.open {
    transform: translateX(0);
  }
  .sb-close {
    display: inline-flex;
  }
  .main-inner {
    padding: 20px 16px 56px;
  }
}
</style>
