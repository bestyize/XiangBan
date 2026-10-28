import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import DiscussView from '@/views/DiscussView.vue'
import { useSession } from '@/stores/session'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'home', component: HomeView },
  { path: '/discuss', name: 'discuss', component: DiscussView },
  // 旧入口：资源、周刊、项目列表都并进了首页的二级 Tab。
  { path: '/resources', redirect: '/' },
  { path: '/packs/:id(\\d+)', name: 'pack', component: () => import('@/views/PackView.vue') },
  { path: '/weekly', redirect: { path: '/', query: { tab: 'weekly' } } },
  { path: '/publish', name: 'publish', component: () => import('@/views/PublishView.vue'), meta: { auth: true } },
  { path: '/me/works', name: 'works', component: () => import('@/views/MyWorksView.vue'), meta: { auth: true } },
  { path: '/weekly/:week', name: 'weekly-issue', component: () => import('@/views/WeeklyView.vue') },
  { path: '/projects', redirect: { path: '/', query: { tab: 'projects' } } },
  { path: '/projects/:slug', name: 'project', component: () => import('@/views/ProjectView.vue') },
  { path: '/c/:slug', name: 'category', component: () => import('@/views/CategoryView.vue') },
  { path: '/t/:id(\\d+)', name: 'topic', component: () => import('@/views/TopicView.vue') },
  { path: '/t/:id(\\d+)/edit', name: 'edit', component: () => import('@/views/ComposeView.vue'), meta: { auth: true } },
  { path: '/new', name: 'new', component: () => import('@/views/ComposeView.vue'), meta: { auth: true } },
  { path: '/tag/:name', name: 'tag', component: () => import('@/views/TagView.vue') },
  { path: '/search', name: 'search', component: () => import('@/views/SearchView.vue') },
  { path: '/u/:uid', name: 'user', component: () => import('@/views/UserView.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/views/SettingsView.vue'), meta: { auth: true } },
  { path: '/points', name: 'points', component: () => import('@/views/PointsView.vue'), meta: { auth: true } },
  { path: '/rules', name: 'rules', component: () => import('@/views/StaticView.vue'), props: { page: 'rules' } },
  { path: '/terms', name: 'terms', component: () => import('@/views/StaticView.vue'), props: { page: 'terms' } },
  { path: '/privacy', name: 'privacy', component: () => import('@/views/StaticView.vue'), props: { page: 'privacy' } },
  { path: '/admin/:section?', name: 'admin', component: () => import('@/views/admin/AdminView.vue'), meta: { bare: true } },
  { path: '/:pathMatch(.*)*', name: 'notfound', component: () => import('@/views/NotFoundView.vue') },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, saved) {
    if (saved) return saved
    if (to.hash) return { el: to.hash, top: 80 }
    if (to.path === from.path) return undefined
    return { top: 0 }
  },
})

// 需要登录的页面：没登录就弹登录框，停在原页面（直接打开时回首页）。
router.beforeEach((to, from) => {
  if (!to.meta.auth) return
  const session = useSession()
  if (session.loggedIn) return
  session.loginOpen = true
  return from.name ? false : { name: 'home' }
})
