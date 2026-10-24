<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { admin, session, setSession } from '@/lib/api';
import { toasts } from '@/lib/toast';
import Icon from '@/components/Icon.vue';
import '@/styles/admin.css';

const route = useRoute();
const router = useRouter();
const pending = ref(0);

const NAV = [
  { to: 'admin-dashboard', label: '总览', icon: 'home' },
  { to: 'admin-sources', label: '采集来源', icon: 'database' },
  { to: 'admin-jobs', label: '采集任务', icon: 'tasks' },
  { to: 'admin-staging', label: '暂存区', icon: 'inbox' },
  { to: 'admin-library', label: '内容库', icon: 'grid' },
  { to: 'admin-categories', label: '分类', icon: 'tag' },
  { to: 'admin-settings', label: '设置', icon: 'sliders' },
];

async function refresh() {
  try {
    pending.value = (await admin.stats()).pendingStaged;
  } catch {
    /* 401 由全局钩子处理；其他错误不影响侧栏 */
  }
}
async function who() {
  if (session.username) return;
  try {
    session.username = (await admin.me()).username;
  } catch {
    /* 同上 */
  }
}
onMounted(() => { refresh(); who(); });
watch(() => route.name, refresh);

function logout() {
  setSession('');
  router.push({ name: 'admin-login' });
}
</script>

<template>
  <div class="admin shell">
    <aside class="side">
      <div class="brand">
        <svg width="28" height="28" viewBox="0 0 30 30" aria-hidden="true"><path d="M4 25V7l6 4.5h10L26 7v18a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3z" fill="#fff" /><circle cx="11" cy="18" r="1.7" fill="#141414" /><circle cx="19" cy="18" r="1.7" fill="#141414" /></svg>
        小伴市场后台
      </div>
      <nav aria-label="后台导航">
        <RouterLink v-for="n in NAV" :key="n.to" :to="{ name: n.to }" class="nav" active-class="" :class="{ on: route.name === n.to }">
          <Icon :name="n.icon" :size="18" :stroke="2" />
          <span>{{ n.label }}</span>
          <b v-if="n.to === 'admin-staging' && pending" class="count">{{ pending }}</b>
        </RouterLink>
      </nav>
      <div class="me">
        <span>{{ session.username || '管理员' }}</span>
        <button type="button" @click="logout">退出</button>
      </div>
    </aside>
    <main class="content"><RouterView /></main>

    <div class="toasts" aria-live="polite">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="t.tone" role="status">{{ t.text }}</div>
    </div>
  </div>
</template>

<style scoped>
.shell { display: flex; flex-wrap: wrap; min-height: 100vh; background: var(--bg); font-size: 14px; line-height: 1.55; }
.side { flex: 1 1 232px; max-width: 232px; min-width: 200px; background: #141414; color: #c8c8c6; padding: 20px 14px; display: flex; flex-direction: column; gap: 22px; }
.brand { display: flex; align-items: center; gap: 10px; padding: 0 8px; color: #fff; font-weight: 700; font-size: 16px; }
nav { display: flex; flex-direction: column; gap: 2px; }
.nav { display: flex; align-items: center; gap: 10px; padding: 10px 12px; min-height: 44px; border-radius: 9px; color: #c8c8c6; }
.nav:hover { background: #1f1f1f; }
.nav.on { background: #2a2a2a; color: #fff; font-weight: 700; }
.nav span { flex: 1; }
.count { padding: 0 7px; border-radius: 999px; background: #fff; color: #141414; font-size: 12px; }
.me { margin-top: auto; padding: 10px 12px; border-top: 1px solid #2e2e2e; display: flex; align-items: center; justify-content: space-between; font-size: 13px; }
.me button { border: 0; background: transparent; color: #c8c8c6; text-decoration: underline; cursor: pointer; min-height: 32px; }
.content { flex: 999 1 640px; min-width: 0; padding: 28px 32px 56px; }
.toasts { position: fixed; right: 20px; bottom: 20px; z-index: 80; display: flex; flex-direction: column; gap: 8px; max-width: min(420px, calc(100vw - 40px)); }
.toast { padding: 12px 16px; border-radius: 12px; background: #141414; color: #fff; box-shadow: 0 8px 24px rgba(20, 20, 20, 0.25); font-size: 14px; }
.toast.bad { background: #8f2a19; }
@media (max-width: 760px) { .content { padding: 20px 16px 40px; } .side { max-width: none; flex-basis: 100%; } }
</style>
