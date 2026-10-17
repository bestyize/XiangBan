<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { api } from './api';
import { units } from './format';
import { session, signOut } from './session';
import LoginView from './views/LoginView.vue';
import OverviewView from './views/OverviewView.vue';
import ProductsView from './views/ProductsView.vue';
import UsersView from './views/UsersView.vue';
import CodesView from './views/CodesView.vue';
import IssuesView from './views/IssuesView.vue';
import AuditView from './views/AuditView.vue';
import PasswordDialog from './components/PasswordDialog.vue';

const pages = [
  { key: 'overview', label: '概览', component: OverviewView },
  { key: 'products', label: '产品与档位', component: ProductsView },
  { key: 'users', label: '用户额度', component: UsersView },
  { key: 'codes', label: '兑换码', component: CodesView },
  { key: 'issues', label: '对账异常', component: IssuesView },
  { key: 'audit', label: '操作日志', component: AuditView },
] as const;

type PageKey = (typeof pages)[number]['key'];

const current = ref<PageKey>('overview');
const view = computed(() => pages.find((p) => p.key === current.value)!.component);
const changingPassword = ref(false);
const openIssues = ref(0);

async function refreshBadge() {
  try {
    const overview = await api.overview();
    units.perDollar = overview.quota_per_unit;
    openIssues.value = overview.open_anomalies;
  } catch {
    /* 角标拿不到不影响使用 */
  }
}

onMounted(() => session.token && refreshBadge());
watch(() => session.token, (token) => token && refreshBadge());
watch(current, () => session.token && refreshBadge());
</script>

<template>
  <LoginView v-if="!session.token" />
  <div v-else class="shell">
    <aside class="side">
      <div class="brand">
        <span class="mark" aria-hidden="true"></span>
        <span>相伴 AI 控制台</span>
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
          <span>{{ page.label }}</span>
          <span v-if="page.key === 'issues' && openIssues > 0" class="badge" :aria-label="`${openIssues} 条未处理`">{{ openIssues }}</span>
        </button>
      </nav>
      <div class="side-foot">
        <div class="who">{{ session.username }}</div>
        <button type="button" class="link" @click="changingPassword = true">修改密码</button>
        <button type="button" class="link" @click="signOut">退出登录</button>
      </div>
    </aside>
    <main class="main">
      <component :is="view" @go="(key: PageKey) => (current = key)" @changed="refreshBadge" />
    </main>
    <PasswordDialog v-if="changingPassword" @close="changingPassword = false" />
  </div>
</template>

<style scoped>
.shell { display: grid; grid-template-columns: 216px 1fr; min-height: 100vh; }
.side { background: var(--surface); border-right: 1px solid var(--line); display: flex; flex-direction: column; padding: 20px 12px; position: sticky; top: 0; height: 100vh; }
.brand { display: flex; align-items: center; gap: 10px; font-weight: 650; font-size: 15px; padding: 2px 10px 22px; }
.mark { width: 18px; height: 18px; border-radius: 5px; background: var(--curtain); border: 1.5px solid var(--primary); position: relative; flex: none; }
.mark::after { content: ''; position: absolute; left: 0; right: 0; bottom: 0; height: 55%; background: var(--primary); border-radius: 0 0 3px 3px; }
nav { display: flex; flex-direction: column; gap: 2px; }
.nav-item { display: flex; align-items: center; justify-content: space-between; text-align: left; border: none; background: none; padding: 9px 12px; border-radius: var(--radius); cursor: pointer; color: var(--muted); }
.nav-item:hover { background: var(--bg); color: var(--ink); }
.nav-item.active { background: var(--primary-soft); color: var(--primary); font-weight: 600; }
.badge { min-width: 20px; height: 20px; padding: 0 6px; border-radius: 10px; background: var(--danger); color: #fff; font-size: 12px; line-height: 20px; text-align: center; font-weight: 500; }
.side-foot { margin-top: auto; padding: 12px 10px 0; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; border-top: 1px solid var(--line); }
.who { font-weight: 600; }
.link { border: none; background: none; padding: 0; color: var(--muted); cursor: pointer; }
.link:hover { color: var(--primary); text-decoration: underline; }
.main { padding: 28px 36px 56px; min-width: 0; max-width: 1180px; }

@media (max-width: 760px) {
  .shell { grid-template-columns: 1fr; grid-template-rows: auto 1fr; }
  .side { position: static; height: auto; flex-direction: row; flex-wrap: wrap; align-items: center; gap: 8px; padding: 10px 12px; }
  .brand { padding: 4px 8px; }
  nav { flex-direction: row; flex-wrap: wrap; }
  .side-foot { margin: 0 0 0 auto; border: none; flex-direction: row; align-items: center; gap: 12px; padding: 0; }
  .main { padding: 20px 14px 40px; }
}
</style>
