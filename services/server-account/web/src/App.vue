<script setup lang="ts">
import { computed, ref } from 'vue';
import { session, signOut } from './session';
import LoginView from './views/LoginView.vue';
import DashboardView from './views/DashboardView.vue';
import UsersView from './views/UsersView.vue';
import ClientsView from './views/ClientsView.vue';
import AuditView from './views/AuditView.vue';
import PasswordDialog from './components/PasswordDialog.vue';

const pages = [
  { key: 'dashboard', label: '概览', component: DashboardView },
  { key: 'users', label: '用户', component: UsersView },
  { key: 'clients', label: '应用', component: ClientsView },
  { key: 'audit', label: '操作日志', component: AuditView },
] as const;

const current = ref<(typeof pages)[number]['key']>('dashboard');
const view = computed(() => pages.find((p) => p.key === current.value)!.component);
const changingPassword = ref(false);
</script>

<template>
  <LoginView v-if="!session.token" />
  <div v-else class="shell">
    <aside class="side">
      <div class="brand">相伴账号中心</div>
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
          {{ page.label }}
        </button>
      </nav>
      <div class="side-foot">
        <div class="who">{{ session.username }}</div>
        <button type="button" class="link" @click="changingPassword = true">修改密码</button>
        <button type="button" class="link" @click="signOut">退出登录</button>
      </div>
    </aside>
    <main class="main">
      <component :is="view" />
    </main>
    <PasswordDialog v-if="changingPassword" @close="changingPassword = false" />
  </div>
</template>

<style scoped>
.shell { display: grid; grid-template-columns: 208px 1fr; min-height: 100vh; }
.side { background: var(--surface); border-right: 1px solid var(--border); display: flex; flex-direction: column; padding: 18px 12px; position: sticky; top: 0; height: 100vh; }
.brand { font-weight: 600; font-size: 16px; padding: 4px 10px 18px; }
nav { display: flex; flex-direction: column; gap: 2px; }
.nav-item { text-align: left; border: none; background: none; padding: 9px 12px; border-radius: 8px; cursor: pointer; color: var(--muted); }
.nav-item:hover { background: var(--bg); color: var(--text); }
.nav-item.active { background: #eceafd; color: var(--primary); font-weight: 500; }
.side-foot { margin-top: auto; padding: 10px; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; border-top: 1px solid var(--border); }
.who { font-weight: 500; }
.link { border: none; background: none; padding: 0; color: var(--muted); cursor: pointer; }
.link:hover { color: var(--primary); }
.main { padding: 26px 32px; min-width: 0; max-width: 1180px; }

@media (max-width: 760px) {
  .shell { grid-template-columns: 1fr; grid-template-rows: auto 1fr; }
  .side { position: static; height: auto; flex-direction: row; flex-wrap: wrap; align-items: center; gap: 8px; }
  .brand { padding: 4px 8px; }
  nav { flex-direction: row; }
  .side-foot { margin: 0 0 0 auto; border: none; flex-direction: row; align-items: center; gap: 12px; padding: 0; }
  .main { padding: 18px 14px; }
}
</style>
