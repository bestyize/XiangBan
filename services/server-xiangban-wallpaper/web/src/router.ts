import { nextTick } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { useAdmin } from './stores/admin'

export const router = createRouter({
  history: createWebHistory(),
  // Scroll is restored per page in the guards below, because pages are kept alive between tabs.
  scrollBehavior: () => false,
  routes: [
    { path: '/desktop', redirect: to => ({ path: '/', query: to.query, hash: to.hash }) },
    { path: '/', name: 'desktop', component: () => import('./views/DesktopView.vue'), meta: { tab: 'desktop' } },
    { path: '/images', name: 'home', component: () => import('./views/BrowseView.vue'), meta: { tab: 'home' } },
    { path: '/images/c/:id', alias: '/c/:id', name: 'category', component: () => import('./views/BrowseView.vue'), meta: { tab: 'home' } },
    { path: '/videos', name: 'videos', component: () => import('./views/BrowseView.vue'), meta: { tab: 'videos' } },
    {
      path: '/videos/c/:id',
      name: 'video-category',
      component: () => import('./views/BrowseView.vue'),
      meta: { tab: 'videos' },
    },
    { path: '/search', name: 'search', component: () => import('./views/BrowseView.vue'), meta: { tab: 'search' } },
    { path: '/categories', name: 'categories', component: () => import('./views/CategoriesView.vue'), meta: { tab: 'categories' } },
    { path: '/favorites', name: 'favorites', component: () => import('./views/FavoritesView.vue'), meta: { tab: 'favorites' } },
    { path: '/me', name: 'me', component: () => import('./views/SettingsView.vue'), meta: { tab: 'me' } },
    {
      path: '/admin/login',
      name: 'admin-login',
      component: () => import('./views/admin/LoginView.vue'),
      meta: { admin: true, public: true },
    },
    {
      path: '/admin',
      component: () => import('./views/admin/AdminLayout.vue'),
      meta: { admin: true },
      redirect: '/admin/collector',
      children: [
        { path: 'desktop', name: 'admin-desktop', component: () => import('./views/admin/DesktopAdminView.vue') },
        { path: 'collector', name: 'admin-collector', component: () => import('./views/admin/CollectorView.vue') },
        { path: 'wallpapers', name: 'admin-wallpapers', component: () => import('./views/admin/WallpapersView.vue') },
        { path: 'upload', name: 'admin-upload', component: () => import('./views/admin/UploadView.vue') },
        { path: 'keys', name: 'admin-keys', component: () => import('./views/admin/ProvidersView.vue') },
        { path: 'proxies', name: 'admin-proxies', component: () => import('./views/admin/ProxiesView.vue') },
        { path: 'cache', name: 'admin-cache', component: () => import('./views/admin/CacheView.vue') },
        { path: 'account', name: 'admin-account', component: () => import('./views/admin/AccountView.vue') },
      ],
    },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})

// Everything under /admin except the login page needs a signed-in administrator.
router.beforeEach(async (to) => {
  if (!to.matched.some((r) => r.meta.admin)) return
  const admin = useAdmin()
  if (!admin.checked) await admin.refresh()

  const isLogin = to.matched.some((r) => r.meta.public)
  if (isLogin) return admin.user ? '/admin/keys' : undefined
  if (!admin.user) return { path: '/admin/login', query: { redirect: to.fullPath } }
})

const scrollByPath = new Map<string, number>()

router.beforeEach((to, from) => {
  // Opening or stepping the viewer only changes the query; the page underneath must not jump.
  if (to.path !== from.path) scrollByPath.set(from.path, window.scrollY)
})

router.afterEach(async (to, from) => {
  if (to.path === from.path) return
  await nextTick()
  window.scrollTo({ top: scrollByPath.get(to.path) ?? 0, behavior: 'instant' })
})
