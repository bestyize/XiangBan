import { createRouter, createWebHistory } from 'vue-router';
import PortalHome from '../views/portal/PortalHome.vue';
import AdminLayout from '../views/admin/AdminLayout.vue';
import AdminDashboard from '../views/admin/AdminDashboard.vue';
import AdminApps from '../views/admin/AdminApps.vue';
import AdminCategories from '../views/admin/AdminCategories.vue';
import AdminSettings from '../views/admin/AdminSettings.vue';
import AdminLogin from '../views/admin/AdminLogin.vue';

const routes = [
  {
    path: '/',
    name: 'PortalHome',
    component: PortalHome,
  },
  {
    path: '/admin/login',
    name: 'AdminLogin',
    component: AdminLogin,
  },
  {
    path: '/admin',
    component: AdminLayout,
    children: [
      {
        path: '',
        redirect: '/admin/dashboard',
      },
      {
        path: 'dashboard',
        name: 'AdminDashboard',
        component: AdminDashboard,
      },
      {
        path: 'apps',
        name: 'AdminApps',
        component: AdminApps,
      },
      {
        path: 'categories',
        name: 'AdminCategories',
        component: AdminCategories,
      },
      {
        path: 'settings',
        name: 'AdminSettings',
        component: AdminSettings,
      },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 };
  },
});

router.beforeEach((to, _from, next) => {
  const token = localStorage.getItem('xiaoban_appstore_admin_token');
  if (to.path.startsWith('/admin') && to.path !== '/admin/login') {
    if (!token) {
      next('/admin/login');
      return;
    }
  }
  next();
});

export default router;
