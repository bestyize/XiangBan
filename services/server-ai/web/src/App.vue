<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ACCENT_PRESETS } from './accents';
import { api } from './api';
import { units } from './format';
import { applyPrefs, prefs, setAccent, toggleTheme, toggleUnitMode } from './prefs';
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

const currentPage = computed(() => pages.find((p) => p.key === current.value)!);
const view = computed(() => currentPage.value.component);
const changingPassword = ref(false);
const cmdOpen = ref(false);
const openIssues = ref(0);

type GatewayState = 'checking' | 'ok' | 'bad' | 'unset';
const gateway = ref<GatewayState>('checking');
const gatewayLatency = ref<number | null>(null);
const gatewayLabel = computed(() => {
  if (gateway.value === 'ok') return gatewayLatency.value !== null ? `New API · ${gatewayLatency.value}ms` : 'New API 已连接';
  if (gateway.value === 'bad') return '网关连接异常';
  if (gateway.value === 'unset') return '待配置令牌';
  return '正在探测网关…';
});
const gatewayDot = computed(() => (gateway.value === 'ok' ? 'ok' : gateway.value === 'bad' ? 'danger' : 'warn'));

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
  notify(prefs.unitMode === 'both' ? '已开启「$ + 小伴点」双单位对照' : '已切换为仅美元显示');
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
      <div class="brand-box">
        <BrandMark :size="30" />
        <div class="brand-txt">
          <strong>小伴 AI 网关</strong>
          <span>管理控制台</span>
        </div>
      </div>

      <button type="button" class="cmd-trigger" title="打开全局快速检索（⌘K）" @click="cmdOpen = true">
        <span class="cmd-l"><Icon name="search" :size="14" /><span>快速检索…</span></span>
        <kbd>⌘K</kbd>
      </button>

      <nav class="nav-groups" aria-label="主导航">
        <div v-for="group in navGroups" :key="group.label" class="nav-group">
          <div class="nav-group-label">{{ group.label }}</div>
          <button
            v-for="page in group.items"
            :key="page.key"
            type="button"
            class="nav-item"
            :class="{ active: current === page.key }"
            :aria-current="current === page.key ? 'page' : undefined"
            :title="page.desc"
            @click="goPage(page.key)"
          >
            <Icon :name="page.icon" :size="16" />
            <span class="truncate">{{ page.label }}</span>
            <span v-if="page.key === 'issues' && openIssues > 0" class="nav-badge num" :aria-label="`${openIssues} 条未处理`">{{ openIssues }}</span>
          </button>
        </div>
      </nav>

      <div class="sb-foot">
        <div class="me-row">
          <span class="me-av" aria-hidden="true">{{ (session.username || 'A').slice(0, 1).toUpperCase() }}</span>
          <div class="me-meta">
            <strong class="truncate">{{ session.username }}</strong>
            <span>管理员</span>
          </div>
          <button type="button" class="btn icon sm ghost" aria-label="修改密码" title="修改管理员密码" @click="changingPassword = true">
            <Icon name="key" :size="14" />
          </button>
          <button type="button" class="btn icon sm ghost" aria-label="退出登录" title="退出登录" @click="signOut">
            <Icon name="log-out" :size="14" />
          </button>
        </div>

        <div class="sb-foot-btns">
          <button
            type="button"
            class="sb-foot-btn"
            title="切换金额单位：美元 + 小伴点对照 / 仅美元"
            @click="onToggleUnit"
          >
            <Icon name="coins" :size="13" />
            <span>{{ prefs.unitMode === 'both' ? '$ + 点' : '仅 $' }}</span>
          </button>
          <button
            type="button"
            class="sb-foot-btn"
            :title="prefs.theme === 'dark' ? '切换浅色模式' : '切换深色模式'"
            @click="toggleTheme"
          >
            <Icon :name="prefs.theme === 'dark' ? 'sun' : 'moon'" :size="13" />
            <span>{{ prefs.theme === 'dark' ? '浅色' : '深色' }}</span>
          </button>
        </div>
      </div>
    </aside>

    <div class="main-col">
      <header class="topbar">
        <div class="crumb">
          <span>{{ currentPage.group }}</span>
          <span>/</span>
          <strong>{{ currentPage.label }}</strong>
        </div>

        <div class="topbar-right">
          <div class="swatches" role="radiogroup" aria-label="切换主题色">
            <button
              v-for="a in ACCENT_PRESETS"
              :key="a.id"
              type="button"
              role="radio"
              class="swatch"
              :class="{ active: prefs.accent === a.id }"
              :aria-checked="prefs.accent === a.id"
              :aria-label="`主题色：${a.name}`"
              :title="a.name"
              :style="{ background: prefs.theme === 'dark' ? a.dark.btn : a.light.btn }"
              @click="setAccent(a.id)"
            ></button>
          </div>
          <button type="button" class="status-chip" :title="`网关状态：${gatewayLabel}（点击进入配置）`" @click="goPage('gateway')">
            <i class="dot" :class="[gatewayDot, { blink: gateway === 'checking' }]"></i>
            <span class="num">{{ gatewayLabel }}</span>
          </button>
        </div>
      </header>

      <header class="mobile-bar">
        <div class="mb-top">
          <div class="mb-brand">
            <BrandMark :size="26" />
            <strong>小伴 AI 网关</strong>
          </div>
          <div class="mb-actions">
            <button type="button" class="btn icon sm ghost" aria-label="快速检索" @click="cmdOpen = true"><Icon name="search" :size="14" /></button>
            <button type="button" class="btn sm ghost" @click="onToggleUnit">{{ prefs.unitMode === 'both' ? '$+点' : '$' }}</button>
            <button type="button" class="btn icon sm ghost" aria-label="切换主题" @click="toggleTheme"><Icon :name="prefs.theme === 'dark' ? 'sun' : 'moon'" :size="14" /></button>
            <button type="button" class="btn icon sm ghost" aria-label="修改密码" @click="changingPassword = true"><Icon name="key" :size="14" /></button>
            <button type="button" class="btn icon sm ghost" aria-label="退出登录" @click="signOut"><Icon name="log-out" :size="14" /></button>
          </div>
        </div>
        <div class="mb-tabs" role="tablist">
          <button
            v-for="page in pages"
            :key="page.key"
            type="button"
            class="mb-tab"
            :class="{ active: current === page.key }"
            @click="goPage(page.key)"
          >
            <span>{{ page.label }}</span>
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
  grid-template-columns: 236px minmax(0, 1fr);
  min-height: 100vh;
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  background: var(--surface);
  border-right: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  padding: 14px 12px 12px;
  overflow-y: auto;
  z-index: 40;
}

.brand-box {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 8px 14px;
}
.brand-txt { min-width: 0; }
.brand-txt strong { display: block; font-size: 14.5px; font-weight: 700; letter-spacing: -0.015em; line-height: 1.2; }
.brand-txt span { display: block; font-size: 11px; color: var(--ink-3); margin-top: 2px; }

.cmd-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  height: 34px;
  padding: 0 10px;
  margin: 0 2px 14px;
  border-radius: var(--r-inner);
  background: var(--subtle);
  color: var(--ink-3);
  font-size: 12.5px;
  white-space: nowrap;
  transition: background 140ms, color 140ms;
}
.cmd-trigger:hover { background: var(--hover); color: var(--ink-2); }
.cmd-l { display: inline-flex; align-items: center; gap: 7px; min-width: 0; }
.cmd-trigger kbd {
  font-family: var(--mono);
  font-size: 10.5px;
  padding: 1px 5px;
  border-radius: 5px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink-3);
}

.nav-groups { display: flex; flex-direction: column; gap: 14px; }
.nav-group-label {
  padding: 0 10px 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-4);
  letter-spacing: 0.04em;
}
.nav-item {
  width: 100%;
  height: 34px;
  padding: 0 10px;
  border-radius: var(--r-inner);
  display: flex;
  align-items: center;
  gap: 9px;
  text-align: left;
  color: var(--ink-2);
  font-size: 13px;
  font-weight: 500;
  position: relative;
  margin-bottom: 2px;
  transition: background 140ms, color 140ms;
}
.nav-item:hover { background: var(--subtle); color: var(--ink-1); }
.nav-item.active {
  background: var(--brand-soft);
  color: var(--brand-ink);
  font-weight: 600;
}
.nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: var(--brand);
}
.nav-badge {
  margin-left: auto;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: var(--r-pill);
  background: var(--danger);
  color: #ffffff;
  font-size: 11px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.sb-foot {
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.me-row { display: flex; align-items: center; gap: 8px; padding: 4px 4px; }
.me-av {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  flex: none;
  background: var(--brand-soft);
  color: var(--brand-ink);
  font-size: 12px;
  font-weight: 700;
  display: grid;
  place-items: center;
}
.me-meta { flex: 1; min-width: 0; }
.me-meta strong { display: block; font-size: 12.5px; line-height: 1.2; }
.me-meta span { display: block; font-size: 11px; color: var(--ink-3); }
.sb-foot-btns { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
.sb-foot-btn {
  height: 30px;
  border-radius: var(--r-inner);
  background: var(--subtle);
  color: var(--ink-2);
  font-size: 12px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  transition: background 140ms, color 140ms;
}
.sb-foot-btn:hover { background: var(--hover); color: var(--ink-1); }

/* 右侧主区 + 52px 吸顶顶栏 */
.main-col { min-width: 0; display: flex; flex-direction: column; }
.topbar {
  position: sticky;
  top: 0;
  z-index: 30;
  height: 52px;
  padding: 0 28px;
  flex: none;
  background: color-mix(in srgb, var(--canvas) 92%, transparent);
  backdrop-filter: blur(12px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.crumb { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--ink-3); }
.crumb strong { color: var(--ink-1); font-weight: 600; }
.topbar-right { display: flex; align-items: center; gap: 10px; }

.swatches {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: var(--r-pill);
  background: var(--surface);
  border: 1px solid var(--line);
}
.swatch {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  position: relative;
  transition: transform 140ms var(--ease);
}
.swatch:hover { transform: scale(1.18); }
.swatch.active::after {
  content: '';
  position: absolute;
  inset: -3px;
  border-radius: 50%;
  border: 1.8px solid var(--ink-1);
}

.status-chip {
  height: 30px;
  padding: 0 11px;
  border-radius: var(--r-pill);
  background: var(--surface);
  border: 1px solid var(--line);
  font-size: 12px;
  color: var(--ink-2);
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  transition: background 140ms, color 140ms;
}
.status-chip:hover { background: var(--subtle); color: var(--ink-1); }
.dot.blink { animation: blink 1.1s ease-in-out infinite; }
@keyframes blink { 50% { opacity: 0.3; } }

.main-inner { width: 100%; max-width: 1140px; margin: 0 auto; padding: 12px 28px 64px; }

.mobile-bar { display: none; }

@media (max-width: 860px) {
  .shell { grid-template-columns: 1fr; }
  .sidebar, .topbar { display: none; }
  .mobile-bar {
    position: sticky;
    top: 0;
    z-index: 30;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 14px;
    background: var(--surface);
    border-bottom: 1px solid var(--line);
  }
  .mb-top { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .mb-brand { display: flex; align-items: center; gap: 8px; font-size: 14px; }
  .mb-actions { display: flex; align-items: center; gap: 4px; }
  .mb-tabs { display: flex; gap: 6px; overflow-x: auto; scrollbar-width: none; }
  .mb-tabs::-webkit-scrollbar { display: none; }
  .mb-tab {
    height: 28px;
    padding: 0 11px;
    border-radius: var(--r-pill);
    background: var(--subtle);
    font-size: 12px;
    font-weight: 550;
    color: var(--ink-2);
    white-space: nowrap;
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .mb-tab.active { background: var(--brand); color: var(--brand-on); font-weight: 600; }
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
  .main-inner { padding: 14px 14px 56px; }
}
</style>
