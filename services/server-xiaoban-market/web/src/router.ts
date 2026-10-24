import { createRouter, createWebHistory } from 'vue-router';
import { hooks, session } from './lib/api';

export const router = createRouter({
  history: createWebHistory(),
  scrollBehavior: (to, from) => (to.name === 'store-detail' || from.name === 'store-detail' ? false : { top: 0 }),
  routes: [
    { path: '/', redirect: '/experts' },
    { path: '/:kind(skills|experts|connectors)', name: 'store', component: () => import('./views/StoreView.vue'), props: true },
    { path: '/:kind(skills|experts|connectors)/:ns/:slug', name: 'store-detail', component: () => import('./views/StoreView.vue'), props: true },
    { path: '/admin/login', name: 'admin-login', component: () => import('./views/admin/LoginView.vue'), meta: { title: '登录' } },
    {
      path: '/admin',
      component: () => import('./views/admin/AdminLayout.vue'),
      meta: { admin: true },
      children: [
        { path: '', name: 'admin-dashboard', component: () => import('./views/admin/DashboardView.vue'), meta: { title: '总览' } },
        { path: 'sources', name: 'admin-sources', component: () => import('./views/admin/SourcesView.vue'), meta: { title: '采集来源' } },
        { path: 'jobs', name: 'admin-jobs', component: () => import('./views/admin/JobsView.vue'), meta: { title: '采集任务' } },
        { path: 'staging', name: 'admin-staging', component: () => import('./views/admin/StagingView.vue'), meta: { title: '暂存区' } },
        { path: 'library', name: 'admin-library', component: () => import('./views/admin/LibraryView.vue'), meta: { title: '内容库' } },
        { path: 'categories', name: 'admin-categories', component: () => import('./views/admin/CategoriesView.vue'), meta: { title: '分类' } },
        { path: 'settings', name: 'admin-settings', component: () => import('./views/admin/SettingsView.vue'), meta: { title: '设置' } },
      ],
    },
    { path: '/:pathMatch(.*)*', component: () => import('./views/NotFound.vue') },
  ],
});

router.beforeEach((to) => {
  if (to.matched.some((r) => r.meta.admin) && !session.token) {
    return { name: 'admin-login', query: { redirect: to.fullPath } };
  }
});

router.afterEach((to) => {
  const t = to.meta.title as string | undefined;
  document.title = t ? `${t} · 小伴市场后台` : '小伴市场';
});

hooks.onUnauthorized = () => {
  if (router.currentRoute.value.name !== 'admin-login') {
    router.push({ name: 'admin-login', query: { redirect: router.currentRoute.value.fullPath } });
  }
};
