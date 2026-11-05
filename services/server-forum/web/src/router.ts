import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import DiscussView from '@/views/DiscussView.vue'
import { useSession } from '@/stores/session'
import { trackRoute } from '@/lib/analytics'

const routes: RouteRecordRaw[] = [
  { path: '/me/subscriptions', name: 'subscriptions', component: () => import('@/views/SubscriptionsView.vue'), meta: { auth: true } },
  { path: '/', name: 'home', component: HomeView },
  { path: '/ai', name: 'ai', component: () => import('@/views/AiChatView.vue'), meta: { focus: true } },
  { path: '/create', name: 'create', component: () => import('@/views/CreateView.vue') },
  { path: '/me', name: 'me', component: () => import('@/views/MeView.vue') },
  { path: '/frontier', name: 'frontier', component: () => import('@/views/FrontierView.vue') },
  { path: '/frontier/:key', name: 'frontier-detail', component: () => import('@/views/FrontierView.vue') },
  { path: '/daily', name: 'daily', component: () => import('@/views/DailyView.vue') },
  { path: '/daily/:key', name: 'daily-detail', component: () => import('@/views/DailyView.vue') },
  { path: '/tools/:key', name: 'tool-detail', component: () => import('@/views/ToolView.vue') },
  { path: '/repo/:owner/:repo', name: 'repo-detail', component: () => import('@/views/RepoView.vue') },
  { path: '/open-source', name: 'open-source', redirect: to => ({path:'/', query:{...to.query, tab:'open-source'}}) },
  { path: '/open-source/collections/:key', name: 'open-source-collection', component: () => import('@/views/OpenSourceCollectionView.vue') },
  { path: '/open-source/:owner/:repo', name: 'open-source-project', component: () => import('@/views/OpenSourceDetailView.vue') },
  { path: '/discuss', name: 'discuss', component: DiscussView },
  { path: '/ideas', name: 'ideas', component: DiscussView },
  { path: '/models', name: 'model-rankings', component: () => import('@/views/ModelRankingsView.vue') },
  { path: '/resources', name: 'resources', component: () => import('@/views/ResourcesView.vue') },
  { path: '/challenges', name: 'challenges', component: () => import('@/views/ChallengesView.vue') },
  { path: '/challenges/:key', name: 'challenge', component: () => import('@/views/ChallengeView.vue') },
  { path: '/schemes', name: 'schemes', component: () => import('@/views/SchemesView.vue') },
  { path: '/schemes/new', name: 'scheme-publish', component: () => import('@/views/SchemePublishView.vue'), meta: { auth: true } },
  { path: '/schemes/:id(\\d+)', name: 'scheme', component: () => import('@/views/SchemeView.vue') },
  { path: '/packs/:id(\\d+)', name: 'pack', component: () => import('@/views/PackView.vue') },
  { path: '/weekly', redirect: { path: '/', query: { tab: 'weekly' } } },
  { path: '/publish', name: 'publish', component: () => import('@/views/PublishView.vue'), meta: { auth: true } },
  { path: '/me/content-bookmarks', name: 'content-bookmarks', component: () => import('@/views/ContentBookmarksView.vue'), meta: { auth: true } },
  { path: '/me/scheme-backups', name:'scheme-backups', component:()=>import('@/views/SchemeBackupsView.vue'), meta:{auth:true} },
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
  { path: '/notifications', name: 'notifications', component: () => import('@/views/NotificationsView.vue'), meta: { auth: true } },
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
    if (to.path === from.path) {
      if (['/', '/discuss', '/ideas', '/resources', '/models', '/daily', '/open-source'].includes(to.path) && to.fullPath !== from.fullPath) return { top:0 }
      return undefined
    }
    return { top: 0 }
  },
})

router.afterEach(to => trackRoute(to.name, to.query))

// 需要登录的页面：没登录就弹登录框，停在原页面（直接打开时回我的页）。
router.beforeEach((to, from) => {
  if (to.path === '/resources' && to.query.tab === 'docs') return { path:'/resources', replace:true }
  if (to.path === '/discuss' && to.query.tab === 'create') {
    const { tab, filter, page, ...query } = to.query
    return { path:'/ideas', query, hash:to.hash, replace:true }
  }
  if (to.path === '/discuss' && to.query.tab === 'casual-media') {
    if (to.query.filter === 'movies') {
      return { path:'/resources', query:{ tab:'media', page:to.query.page }, hash:to.hash, replace:true }
    }
    return { path:'/discuss', query:{ ...to.query, tab:undefined, filter:undefined, page:undefined }, hash:to.hash, replace:true }
  }
  if (to.path === '/resources' && to.query.tab === 'open') return {path:'/', query:{...to.query, tab:'open-source'}}
  if (to.path === '/') {
    if (to.query.tab === 'repos') return {path:'/', query:{...to.query, tab:'open-source', category:'hot'}}
    const kind = String(to.query.kind || to.query.tab || '')
    if (kind === 'schemes') return { path: '/schemes', query: {...to.query, tab: undefined, kind: undefined} }
    if (['game','widget','wallpaper','effect','pet','skill','expert','workflow','reminder','routine','task','goal'].includes(kind)) {
      return {path:'/ideas', query:{...to.query, filter:undefined, kind:undefined, tab:undefined, page:undefined}}
    }
  }
  if (!to.meta.auth) return
  const session = useSession()
  if (session.loggedIn) return
  session.loginReturnTo = to.fullPath
  session.loginOpen = true
  return from.name ? false : { name: 'me' }
})
