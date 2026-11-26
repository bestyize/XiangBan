<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { api } from './api';
import { units } from './format';
import { applyPrefs, prefs, toggleTheme, toggleUnitMode } from './prefs';
import { session, signOut } from './session';
import { notify } from './toast';
import BindingDrawer from './components/BindingDrawer.vue';
import BrandMark from './components/BrandMark.vue';
import CommandPalette, { type PalettePage } from './components/CommandPalette.vue';
import Icon, { type IconName } from './components/Icon.vue';
import PasswordDialog from './components/PasswordDialog.vue';
import ToastHost from './components/ToastHost.vue';
import AuditView from './views/AuditView.vue';
import CodesView from './views/CodesView.vue';
import GatewayView from './views/GatewayView.vue';
import IssuesView from './views/IssuesView.vue';
import LoginView from './views/LoginView.vue';
import OverviewView from './views/OverviewView.vue';
import ProductsView from './views/ProductsView.vue';
import UsersView from './views/UsersView.vue';

const pages = [
  { key: 'overview', group: '运营', label: '概览', desc: '核心 KPI、14 天流水与额度去向', icon: 'dashboard', component: OverviewView },
  { key: 'users', group: '运营', label: '用户额度', desc: '查询账号余额、补发与扣回', icon: 'users', component: UsersView },
  { key: 'codes', group: '运营', label: '兑换码', desc: '查码退款与淘宝卡密批次管理', icon: 'ticket', component: CodesView },
  { key: 'products', group: '路由配置', label: '产品与档位', desc: '档位路由映射与媒体能力配置', icon: 'layers', component: ProductsView },
  { key: 'gateway', group: '路由配置', label: '网关连接', desc: '管理令牌与上游模型定价自检', icon: 'plug', component: GatewayView },
  { key: 'issues', group: '风控与审计', label: '对账异常', desc: '核对 New API 钱包与本地账本差额', icon: 'alert', component: IssuesView },
  { key: 'audit', group: '风控与审计', label: '操作日志', desc: '管理员写操作留痕审计', icon: 'scroll', component: AuditView },
] as const satisfies readonly {
  key: string;
  group: string;
  label: string;
  desc: string;
  icon: IconName;
  component: unknown;
}[];

type PageKey = (typeof pages)[number]['key'];
const PAGE_KEYS = new Set<string>(pages.map((p) => p.key));

const navGroups = ['运营', '路由配置', '风控与审计'].map((group) => ({
  label: group,
  items: pages.filter((p) => p.group === group),
}));

const palettePages: readonly PalettePage[] = pages.map((p) => ({
  key: p.key,
  label: p.label,
  desc: p.desc,
  icon: p.icon,
}));

function parseHash(): { page: PageKey; product?: string } {
  const raw = window.location.hash.replace(/^#\/?/, '');
  const [name, qs] = raw.split('?');
  const page: PageKey = PAGE_KEYS.has(name) ? (name as PageKey) : 'overview';
  const params = new URLSearchParams(qs ?? '');
  return { page, product: params.get('product') ?? undefined };
}

const initialHash = parseHash();
const current = ref<PageKey>(initialHash.page);
const targetProductId = ref<string>(initialHash.product ?? '');
const pendingUserQuery = ref('');
const pendingCodeQuery = ref('');
const pendingCreateBatch = ref(false);
const drawerBindingId = ref<number | null>(null);
const refreshTick = ref(0);

const view = computed(() => pages.find((p) => p.key === current.value)!.component);
const changingPassword = ref(false);
const cmdOpen = ref(false);
const openIssues = ref(0);

type GatewayState = 'checking' | 'ok' | 'bad' | 'unset';
const gateway = ref<GatewayState>('checking');
const gatewayLatency = ref<number | null>(null);
const gatewayText = computed(
  () =>
    ({
      checking: '正在检测网关…',
      ok: 'New API 已连接',
      bad: '网关连接异常',
      unset: '待配置网关令牌',
    })[gateway.value],
);

function syncHash() {
  const suffix = current.value === 'products' && targetProductId.value ? `?product=${encodeURIComponent(targetProductId.value)}` : '';
  const nextHash = `#${current.value}${suffix}`;
  if (window.location.hash !== nextHash) {
    window.history.replaceState(null, '', nextHash);
  }
}

function goPage(key: string, opts?: { productId?: string; createBatch?: boolean; userQuery?: string; codeQuery?: string }) {
  if (!PAGE_KEYS.has(key)) return;
  if (opts?.productId !== undefined) targetProductId.value = opts.productId;
  if (opts?.createBatch) pendingCreateBatch.value = true;
  if (opts?.userQuery !== undefined) pendingUserQuery.value = opts.userQuery;
  if (opts?.codeQuery !== undefined) pendingCodeQuery.value = opts.codeQuery;
  current.value = key as PageKey;
  syncHash();
}

function onSelectProduct(productId: string) {
  targetProductId.value = productId;
  if (current.value === 'products') syncHash();
}

function consumeIntent() {
  pendingUserQuery.value = '';
  pendingCodeQuery.value = '';
  pendingCreateBatch.value = false;
}

async function refreshBadge() {
  try {
    const overview = await api.overview();
    units.perDollar = overview.quota_per_unit;
    openIssues.value = overview.open_anomalies;
  } catch {
    /* 角标获取失败不影响主界面 */
  }
}

async function refreshGateway() {
  gateway.value = 'checking';
  gatewayLatency.value = null;
  try {
    const { newapi } = await api.settings();
    if (!newapi.configured) {
      gateway.value = 'unset';
      return;
    }
    const t0 = performance.now();
    const res = await api.testGateway();
    gatewayLatency.value = Math.max(1, Math.round(performance.now() - t0));
    gateway.value = res.ok ? 'ok' : 'bad';
  } catch {
    gateway.value = 'bad';
  }
}

function refreshAll() {
  refreshBadge();
  refreshGateway();
}

function onDrawerChanged() {
  refreshTick.value += 1;
  refreshBadge();
}

function onToggleUnit() {
  toggleUnitMode();
  notify(prefs.unitMode === 'both' ? '已开启「美元 + 小伴点」双单位对照' : '已切换为仅美元显示');
}

function onGlobalKeydown(e: KeyboardEvent) {
  if (!session.token) return;
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    cmdOpen.value = !cmdOpen.value;
  }
}

function onHashChange() {
  const parsed = parseHash();
  current.value = parsed.page;
  if (parsed.product) targetProductId.value = parsed.product;
}

onMounted(() => {
  applyPrefs();
  if (session.token) refreshAll();
  window.addEventListener('keydown', onGlobalKeydown);
  window.addEventListener('hashchange', onHashChange);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onGlobalKeydown);
  window.removeEventListener('hashchange', onHashChange);
});

watch(
  () => session.token,
  (token) => {
    if (token) refreshAll();
    else drawerBindingId.value = null;
  },
);
watch(current, () => {
  syncHash();
  if (session.token) refreshBadge();
});
</script>

<template>
  <LoginView v-if="!session.token" />
  <div v-else class="shell">
    <aside class="sidebar">
      <div class="sb-brand">
        <BrandMark :size="32" />
        <div class="sb-brand-txt">
          <strong>小伴 AI 网关</strong>
          <span>运营与额度控制台</span>
        </div>
      </div>

      <button type="button" class="cmd-trigger" @click="cmdOpen = true">
        <Icon name="search" :size="14" />
        <span>快速查码、找用户…</span>
        <kbd>⌘K</kbd>
      </button>

      <nav class="nav-groups" aria-label="主导航">
        <div v-for="group in navGroups" :key="group.label" class="nav-group">
          <div class="nav-group-label">{{ group.label }}</div>
          <div class="nav-list">
            <button
              v-for="page in group.items"
              :key="page.key"
              type="button"
              class="nav-item"
              :class="{ active: current === page.key }"
              :aria-current="current === page.key ? 'page' : undefined"
              @click="goPage(page.key)"
            >
              <Icon :name="page.icon" :size="16" />
              <span class="label">{{ page.label }}</span>
              <span v-if="page.key === 'issues' && openIssues > 0" class="nav-badge num" :aria-label="`${openIssues} 条未处理`">{{ openIssues }}</span>
            </button>
          </div>
        </div>
      </nav>

      <div class="sb-foot">
        <button type="button" class="gw-pill" :class="gateway" :title="gatewayText" @click="goPage('gateway')">
          <i class="gw-dot" aria-hidden="true"></i>
          <span class="truncate">{{ gatewayText }}</span>
          <span v-if="gateway === 'ok' && gatewayLatency !== null" class="gw-ms num">{{ gatewayLatency }}ms</span>
        </button>

        <div class="sb-prefs">
          <button
            type="button"
            class="pref-btn"
            :class="{ active: prefs.unitMode === 'both' }"
            title="切换金额显示单位：美元 + 小伴点对照 / 仅美元"
            @click="onToggleUnit"
          >
            <Icon name="coins" :size="13" />
            <span>{{ prefs.unitMode === 'both' ? '$ + 小伴点' : '仅美元 $' }}</span>
          </button>
          <button type="button" class="pref-btn ico-only" :title="prefs.theme === 'dark' ? '切换暖白浅色' : '切换极夜深色'" @click="toggleTheme">
            <Icon :name="prefs.theme === 'dark' ? 'sun' : 'moon'" :size="14" />
          </button>
        </div>

        <div class="me-row">
          <span class="me-av" aria-hidden="true">{{ (session.username || 'A').slice(0, 1).toUpperCase() }}</span>
          <span class="me-name">{{ session.username }}</span>
          <button type="button" class="me-btn" aria-label="修改密码" title="修改密码" @click="changingPassword = true">
            <Icon name="key" :size="14" />
          </button>
          <button type="button" class="me-btn" aria-label="退出登录" title="退出登录" @click="signOut">
            <Icon name="log-out" :size="14" />
          </button>
        </div>
      </div>
    </aside>

    <div class="main-col">
      <header class="mobile-bar">
        <div class="mb-top">
          <div class="mb-brand">
            <BrandMark :size="26" />
            <strong>小伴 AI 网关</strong>
          </div>
          <div class="mb-actions">
            <button type="button" class="btn sm ghost" aria-label="快速检索" @click="cmdOpen = true"><Icon name="search" :size="14" /></button>
            <button type="button" class="btn sm ghost" @click="onToggleUnit">{{ prefs.unitMode === 'both' ? '$+点' : '$' }}</button>
            <button type="button" class="btn sm ghost" aria-label="切换主题" @click="toggleTheme"><Icon :name="prefs.theme === 'dark' ? 'sun' : 'moon'" :size="14" /></button>
            <button type="button" class="btn sm ghost" aria-label="修改密码" @click="changingPassword = true"><Icon name="key" :size="14" /></button>
            <button type="button" class="btn sm ghost" aria-label="退出登录" @click="signOut"><Icon name="log-out" :size="14" /></button>
          </div>
        </div>
        <div class="mb-tabs" role="tablist">
          <button
            v-for="page in pages"
            :key="page.key"
            type="button"
            :class="{ active: current === page.key }"
            @click="goPage(page.key)"
          >
            {{ page.label }}
            <span v-if="page.key === 'issues' && openIssues > 0" class="mb-dot">{{ openIssues }}</span>
          </button>
        </div>
      </header>

      <main class="main-inner">
        <component
          :is="view"
          :initial-product-id="targetProductId"
          :initial-query="current === 'users' ? pendingUserQuery : pendingCodeQuery"
          :auto-create-batch="pendingCreateBatch"
          :refresh-tick="refreshTick"
          @go="goPage"
          @select-product="onSelectProduct"
          @open-binding="(id: number) => (drawerBindingId = id)"
          @consume-intent="consumeIntent"
          @changed="refreshAll"
        />
      </main>
    </div>

    <BindingDrawer
      v-if="drawerBindingId !== null"
      :id="drawerBindingId"
      @close="drawerBindingId = null"
      @go-issues="() => { drawerBindingId = null; goPage('issues'); }"
      @changed="onDrawerChanged"
    />
    <PasswordDialog v-if="changingPassword" @close="changingPassword = false" />
    <CommandPalette
      v-if="cmdOpen"
      :pages="palettePages"
      @close="cmdOpen = false"
      @go="(key) => { cmdOpen = false; goPage(key); }"
      @open-binding="(id) => { cmdOpen = false; drawerBindingId = id; }"
      @search-user="(q) => { cmdOpen = false; goPage('users', { userQuery: q }); }"
      @lookup-code="(code) => { cmdOpen = false; goPage('codes', { codeQuery: code }); }"
    />
  </div>

  <ToastHost />
</template>

<style scoped>
.shell {
  display: grid;
  grid-template-columns: 238px minmax(0, 1fr);
  min-height: 100vh;
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
}
.sb-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 2px 6px 14px;
}
.sb-brand-txt { display: flex; flex-direction: column; line-height: 1.2; min-width: 0; }
.sb-brand-txt strong { font-size: 14.5px; font-weight: 700; letter-spacing: -0.01em; }
.sb-brand-txt span { font-size: 11px; color: var(--ink-3); margin-top: 2px; }

.cmd-trigger {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  height: 34px;
  padding: 0 9px;
  margin-bottom: 14px;
  border-radius: 9px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  color: var(--ink-3);
  font-size: 12.5px;
  box-shadow: var(--shadow-xs);
  white-space: nowrap;
  transition: border-color 0.14s, color 0.14s;
}
.cmd-trigger > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cmd-trigger:hover { border-color: var(--line-focus); color: var(--ink-2); }
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

.nav-groups { display: flex; flex-direction: column; gap: 14px; }
.nav-group-label {
  font-size: 11px;
  font-weight: 650;
  color: var(--ink-3);
  padding: 0 10px 6px;
  letter-spacing: 0.03em;
}
.nav-list { display: flex; flex-direction: column; gap: 2px; }
.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  height: 35px;
  padding: 0 10px;
  border-radius: 9px;
  color: var(--ink-2);
  font-size: 13.5px;
  font-weight: 520;
  text-align: left;
  transition: background 0.14s, color 0.14s;
}
.nav-item:hover { background: rgba(25, 23, 26, 0.045); color: var(--ink); }
:global([data-theme='dark']) .nav-item:hover { background: rgba(255, 255, 255, 0.055); }
.nav-item.active {
  background: var(--surface);
  color: var(--ink);
  font-weight: 650;
  box-shadow: var(--shadow-xs), inset 0 0 0 1px var(--line);
}
.nav-item.active :deep(svg) { color: var(--coral); }
.label { flex: 1; }
.nav-badge {
  min-width: 19px;
  height: 19px;
  padding: 0 6px;
  border-radius: 999px;
  background: var(--danger);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  display: grid;
  place-items: center;
}

.sb-foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--line);
}
.gw-pill {
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
  transition: border-color 0.14s, color 0.14s;
}
.gw-pill:hover { border-color: var(--line-strong); color: var(--ink); }
.gw-dot { width: 8px; height: 8px; border-radius: 50%; flex: none; background: var(--ink-4); }
.gw-pill.ok .gw-dot { background: var(--ok); box-shadow: 0 0 0 3px var(--ok-soft); }
.gw-pill.bad .gw-dot { background: var(--danger); box-shadow: 0 0 0 3px var(--danger-soft); }
.gw-pill.unset .gw-dot { background: var(--warn); box-shadow: 0 0 0 3px var(--warn-soft); }
.gw-pill.checking .gw-dot { animation: blink 1.1s ease-in-out infinite; }
.gw-ms { margin-left: auto; font-size: 11px; color: var(--ink-3); font-weight: 500; }
@keyframes blink { 50% { opacity: 0.3; } }

.sb-prefs { display: grid; grid-template-columns: 1fr auto; gap: 6px; }
.pref-btn {
  height: 30px;
  padding: 0 9px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--surface);
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  transition: border-color 0.14s, color 0.14s;
}
.pref-btn.ico-only { width: 34px; padding: 0; }
.pref-btn:hover { border-color: var(--line-strong); color: var(--ink); }
.pref-btn.active { border-color: var(--line-strong); color: var(--ink); }

.me-row { display: flex; align-items: center; gap: 8px; padding: 4px 2px 0; }
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
.me-name { flex: 1; min-width: 0; font-size: 12.5px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; }
.me-btn {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  color: var(--ink-3);
  display: grid;
  place-items: center;
  transition: 0.14s;
}
.me-btn:hover { background: var(--canvas-soft); color: var(--ink); }

.main-col { min-width: 0; display: flex; flex-direction: column; }
.main-inner { width: 100%; max-width: 1180px; margin: 0 auto; padding: 28px 36px 76px; }

.mobile-bar { display: none; }

@media (max-width: 860px) {
  .shell { grid-template-columns: 1fr; }
  .sidebar { display: none; }
  .mobile-bar {
    position: sticky;
    top: 0;
    z-index: 30;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 14px;
    background: var(--surface-glass);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid var(--line);
  }
  .mb-top { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .mb-brand { display: flex; align-items: center; gap: 8px; font-size: 14px; }
  .mb-actions { display: flex; align-items: center; gap: 3px; }
  .mb-tabs { display: flex; gap: 4px; overflow-x: auto; scrollbar-width: none; }
  .mb-tabs::-webkit-scrollbar { display: none; }
  .mb-tabs button {
    height: 28px;
    padding: 0 10px;
    border-radius: 7px;
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-2);
    white-space: nowrap;
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .mb-tabs button.active { background: var(--btn-primary-bg); color: var(--btn-primary-ink); }
  .mb-dot {
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 99px;
    background: var(--danger);
    color: #fff;
    font-size: 10px;
    display: inline-grid;
    place-items: center;
  }
  .main-inner { padding: 18px 16px 64px; }
}
</style>
