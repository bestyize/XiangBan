<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { api } from './api';
import { units } from './format';
import { session, signOut } from './session';
import BrandMark from './components/BrandMark.vue';
import Icon, { type IconName } from './components/Icon.vue';
import PasswordDialog from './components/PasswordDialog.vue';
import ToastHost from './components/ToastHost.vue';
import LoginView from './views/LoginView.vue';
import OverviewView from './views/OverviewView.vue';
import ProductsView from './views/ProductsView.vue';
import UsersView from './views/UsersView.vue';
import CodesView from './views/CodesView.vue';
import IssuesView from './views/IssuesView.vue';
import GatewayView from './views/GatewayView.vue';
import AuditView from './views/AuditView.vue';

const pages = [
  { key: 'overview', label: '概览', icon: 'dashboard', component: OverviewView },
  { key: 'products', label: '产品与档位', icon: 'layers', component: ProductsView },
  { key: 'users', label: '用户额度', icon: 'users', component: UsersView },
  { key: 'codes', label: '兑换码', icon: 'ticket', component: CodesView },
  { key: 'issues', label: '对账异常', icon: 'alert', component: IssuesView },
  { key: 'gateway', label: '网关连接', icon: 'plug', component: GatewayView },
  { key: 'audit', label: '操作日志', icon: 'scroll', component: AuditView },
] as const satisfies readonly { key: string; label: string; icon: IconName; component: unknown }[];

type PageKey = (typeof pages)[number]['key'];

const current = ref<PageKey>('overview');
const view = computed(() => pages.find((p) => p.key === current.value)!.component);
const changingPassword = ref(false);
const openIssues = ref(0);

type Gateway = 'checking' | 'ok' | 'bad' | 'unset';
const gateway = ref<Gateway>('checking');
const gatewayText = computed(() => ({ checking: '正在检查网关…', ok: 'AI 网关已连接', bad: 'AI 网关连不上', unset: '还没设置网关令牌' })[gateway.value]);

async function refreshBadge() {
  try {
    const overview = await api.overview();
    units.perDollar = overview.quota_per_unit;
    openIssues.value = overview.open_anomalies;
  } catch {
    /* 角标拿不到不影响使用 */
  }
}

async function refreshGateway() {
  gateway.value = 'checking';
  try {
    const { newapi } = await api.settings();
    if (!newapi.configured) return void (gateway.value = 'unset');
    gateway.value = (await api.testGateway()).ok ? 'ok' : 'bad';
  } catch {
    gateway.value = 'bad';
  }
}

function refreshAll() {
  refreshBadge();
  refreshGateway();
}

onMounted(() => session.token && refreshAll());
watch(() => session.token, (token) => token && refreshAll());
watch(current, () => session.token && refreshBadge());

// 登录成功后幕布从中间向两边拉开。只在刚登录时出现，刷新页面不重演；系统要求减少动效时直接跳过。
const curtain = ref<'off' | 'closed' | 'open'>('off');
async function entered() {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  curtain.value = 'closed';
  await nextTick();
  setTimeout(() => (curtain.value = 'open'), 140);
  setTimeout(() => (curtain.value = 'off'), 1250);
}
</script>

<template>
  <LoginView v-if="!session.token" @entered="entered" />
  <div v-else class="shell">
    <aside class="rail">
      <div class="brand">
        <BrandMark :size="34" />
        <div class="name"><strong>相伴 AI</strong><span>控制台</span></div>
      </div>

      <nav aria-label="主导航">
        <button
          v-for="page in pages"
          :key="page.key"
          type="button"
          class="nav-item"
          :class="{ active: current === page.key }"
          :aria-current="current === page.key ? 'page' : undefined"
          @click="current = page.key"
        >
          <Icon :name="page.icon" :size="18" />
          <span class="label">{{ page.label }}</span>
          <span v-if="page.key === 'issues' && openIssues > 0" class="badge" :aria-label="`${openIssues} 条未处理`">{{ openIssues }}</span>
        </button>
      </nav>

      <div class="foot">
        <button type="button" class="gw" :class="gateway" :title="gatewayText" @click="current = 'gateway'">
          <i class="dot" aria-hidden="true"></i>
          <span>{{ gatewayText }}</span>
        </button>
        <div class="me">
          <span class="who" aria-hidden="true">{{ (session.username || 'A').slice(0, 1).toUpperCase() }}</span>
          <span class="uname">{{ session.username }}</span>
          <button type="button" class="ico" aria-label="修改密码" title="修改密码" @click="changingPassword = true"><Icon name="key" :size="16" /></button>
          <button type="button" class="ico" aria-label="退出登录" title="退出登录" @click="signOut"><Icon name="log-out" :size="16" /></button>
        </div>
      </div>
    </aside>

    <main class="main">
      <component :is="view" @go="(key: PageKey) => (current = key)" @changed="refreshAll" />
    </main>
    <PasswordDialog v-if="changingPassword" @close="changingPassword = false" />
  </div>

  <div v-if="curtain !== 'off'" class="curtain" :class="{ open: curtain === 'open' }" aria-hidden="true">
    <div class="half l"></div>
    <div class="half r"></div>
  </div>
  <ToastHost />
</template>

<style scoped>
.shell { display: grid; grid-template-columns: 248px minmax(0, 1fr); min-height: 100vh; }

.rail { position: sticky; top: 0; height: 100vh; display: flex; flex-direction: column; padding: 20px 14px 16px; background: var(--ink); color: #c4d6d3; }
.brand { display: flex; align-items: center; gap: 12px; padding: 4px 8px 22px; }
.name { display: flex; flex-direction: column; line-height: 1.25; }
.name strong { color: #fff; font-size: 15.5px; letter-spacing: -0.005em; }
.name span { color: #7f9a97; font-size: 12px; font-weight: 600; }

nav { display: flex; flex-direction: column; gap: 3px; }
.nav-item { position: relative; display: flex; align-items: center; gap: 12px; width: 100%; padding: 10px 12px; border: none; border-radius: 10px; background: none; color: #a9bdba; font-weight: 600; font-size: 14px; text-align: left; cursor: pointer; transition: background-color 0.14s, color 0.14s; }
.nav-item:hover { background: rgba(255, 255, 255, 0.06); color: #fff; }
.nav-item.active { background: rgba(111, 216, 194, 0.13); color: #fff; }
.nav-item.active svg { color: var(--mint); }
.nav-item.active::before { content: ''; position: absolute; left: -14px; top: 9px; bottom: 9px; width: 3px; border-radius: 0 3px 3px 0; background: var(--mint); }
.nav-item:focus-visible { outline: 2px solid var(--mint); outline-offset: -2px; }
.label { flex: 1; }
.badge { min-width: 21px; height: 21px; padding: 0 6px; border-radius: 11px; background: #c93a2f; color: #fff; font-size: 12px; line-height: 21px; text-align: center; font-weight: 700; }

.foot { margin-top: auto; display: flex; flex-direction: column; gap: 10px; }
.gw { display: flex; align-items: center; gap: 10px; width: 100%; padding: 10px 12px; border: 1px solid rgba(255, 255, 255, 0.09); border-radius: 10px; background: rgba(255, 255, 255, 0.03); color: #b6c9c6; font-size: 12.5px; font-weight: 600; text-align: left; cursor: pointer; transition: background-color 0.14s; }
.gw:hover { background: rgba(255, 255, 255, 0.08); }
.dot { width: 8px; height: 8px; border-radius: 50%; background: #6b8380; flex: none; }
.gw.ok .dot { background: var(--mint); box-shadow: 0 0 0 3px rgba(111, 216, 194, 0.22); }
.gw.bad .dot { background: #f0766b; box-shadow: 0 0 0 3px rgba(240, 118, 107, 0.2); }
.gw.unset .dot { background: #e5b44c; box-shadow: 0 0 0 3px rgba(229, 180, 76, 0.2); }
.gw.checking .dot { animation: blink 1.1s ease-in-out infinite; }
@keyframes blink { 50% { opacity: 0.3; } }

.me { display: flex; align-items: center; gap: 8px; padding: 4px 4px 0 6px; }
.who { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 50%; background: #25393f; color: var(--mint); font-weight: 700; font-size: 13px; flex: none; }
.uname { flex: 1; min-width: 0; color: #e3efed; font-weight: 600; font-size: 13.5px; overflow: hidden; text-overflow: ellipsis; }
.ico { display: grid; place-items: center; width: 30px; height: 30px; border: none; background: none; color: #8aa3a0; border-radius: 8px; cursor: pointer; }
.ico:hover { color: #fff; background: rgba(255, 255, 255, 0.09); }

.main { min-width: 0; padding: 34px 44px 80px; }
.main > :deep(*) { max-width: 1180px; margin-left: auto; margin-right: auto; }

.curtain { position: fixed; inset: 0; z-index: 100; display: grid; grid-template-columns: 1fr 1fr; pointer-events: none; }
.half { background: var(--curtain-dark); transition: transform 1s cubic-bezier(0.66, 0, 0.2, 1); will-change: transform; }
.half.l { box-shadow: 14px 0 50px rgba(0, 0, 0, 0.4); }
.half.r { box-shadow: -14px 0 50px rgba(0, 0, 0, 0.4); }
.curtain.open .l { transform: translateX(-101%); }
.curtain.open .r { transform: translateX(101%); }

@media (max-width: 920px) {
  .shell { grid-template-columns: 1fr; grid-template-rows: auto minmax(0, 1fr); }
  .rail { position: static; height: auto; flex-direction: row; flex-wrap: wrap; align-items: center; gap: 6px 10px; padding: 12px 14px 10px; }
  .brand { padding: 0; }
  .name span { display: none; }
  nav { order: 3; flex-direction: row; width: 100%; overflow-x: auto; padding-bottom: 4px; scrollbar-width: none; }
  nav::-webkit-scrollbar { display: none; }
  .nav-item { width: auto; flex: none; padding: 8px 12px; }
  .nav-item.active::before { display: none; }
  .foot { margin: 0 0 0 auto; flex-direction: row; align-items: center; }
  .gw span { display: none; }
  .gw { width: auto; padding: 9px; }
  .uname { display: none; }
  .main { padding: 22px 16px 64px; }
}
</style>
