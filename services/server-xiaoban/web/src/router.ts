import { nextTick } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { useAdmin } from './stores/admin'

function isMobileUa(): boolean {
  if (typeof navigator === 'undefined') return false
  return /Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent || '')
}

function redirectExternal(url: string) {
  if (typeof window !== 'undefined') {
    window.location.replace(url)
  }
  return false
}

export const router = createRouter({
  history: createWebHistory(),
  // Scroll is restored per page in the guards below, because pages are kept alive between tabs.
  scrollBehavior: () => false,
  routes: [
    { path: '/', alias: '/images', name: 'home', component: () => import('./views/BrowseView.vue'), meta: { tab: 'home' } },
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
    { path: '/favorites', redirect: '/' },
    { path: '/me', name: 'me', component: () => import('./views/SettingsView.vue'), meta: { tab: 'me' } },
    { path: '/guide', name: 'guide', component: () => import('./views/GuideView.vue'), meta: { tab: 'guide' } },

    // Legacy product & download paths now live under xiaban.xyz / market.xiangban.xyz
    { path: '/desktop', alias: ['/pc', '/features', '/desktop-experience'], beforeEnter: () => redirectExternal('https://xiaban.xyz/desktop'), component: () => import('./views/BrowseView.vue') },
    { path: '/app', alias: '/mobile', beforeEnter: () => redirectExternal('https://xiaban.xyz/app'), component: () => import('./views/BrowseView.vue') },
    { path: '/download', beforeEnter: () => redirectExternal(isMobileUa() ? 'https://xiaban.xyz/app' : 'https://xiaban.xyz/desktop'), component: () => import('./views/BrowseView.vue') },
    { path: '/plugins', beforeEnter: () => redirectExternal('https://market.xiangban.xyz/'), component: () => import('./views/BrowseView.vue') },

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
      redirect: '/admin/overview',
      children: [
        { path: 'overview', name: 'admin-overview', component: () => import('./views/admin/OverviewView.vue') },
        { path: 'ai-providers', name: 'admin-ai-providers', component: () => import('./views/admin/AiProvidersView.vue') },
        { path: 'apps', name: 'admin-apps', component: () => import('./views/admin/AppReleasesView.vue') },
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

// Legacy anchor /#desktop-downloads from older notices redirects to xiaban.xyz/desktop.
router.beforeEach((to) => {
  if ((to.path === '/' || to.path === '/images') && to.hash === '#desktop-downloads') {
    return redirectExternal(isMobileUa() ? 'https://xiaban.xyz/app' : 'https://xiaban.xyz/desktop')
  }
})

// Everything under /admin except the login page needs a signed-in administrator.
router.beforeEach(async (to) => {
  if (!to.matched.some((r) => r.meta.admin)) return
  const admin = useAdmin()
  if (!admin.checked) await admin.refresh()

  const isLogin = to.matched.some((r) => r.meta.public)
  if (isLogin) return admin.user ? '/admin/overview' : undefined
  if (!admin.user) return { path: '/admin/login', query: { redirect: to.fullPath } }
})

const scrollByPath = new Map<string, number>()

router.beforeEach((to, from) => {
  // Opening or stepping the viewer only changes the query; the page underneath must not jump.
  if (to.path !== from.path) scrollByPath.set(from.path, window.scrollY)
})

router.afterEach(async (to, from) => {
  if (to.path === from.path && to.hash === from.hash) return
  await nextTick()
  if (to.hash) {
    let id = to.hash.slice(1)
    try { id = decodeURIComponent(id) } catch { /* Keep the original hash. */ }
    const target = document.getElementById(id)
    if (target) { target.scrollIntoView(); return }
  }
  if (to.path !== from.path) window.scrollTo({ top: scrollByPath.get(to.path) ?? 0, behavior: 'instant' })
})
