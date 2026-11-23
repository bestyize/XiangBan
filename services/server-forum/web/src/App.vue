<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppHeader from '@/components/AppHeader.vue'
import TabBar from '@/components/TabBar.vue'
import LoginDialog from '@/components/LoginDialog.vue'
import AppearanceDialog from '@/components/AppearanceDialog.vue'
import ModalDialog from '@/components/ModalDialog.vue'
import ToastHost from '@/components/ToastHost.vue'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { useTopicComposer } from '@/stores/topicComposer'
import { usePackPublisher } from '@/stores/packPublisher'
import { captureInvite } from '@/lib/invite'
import { areaOf, communitySections, isCommunityRoot, sectionOf, type CommunityLink, type Section } from '@/lib/sections'
const route = useRoute(), router = useRouter(), session = useSession(), meta = useMeta()
const composer = useTopicComposer()
const publisher = usePackPublisher()
const TopicComposer = defineAsyncComponent(() => import('@/components/TopicComposer.vue'))
const PackPublishDialog = defineAsyncComponent(() => import('@/components/PackPublishDialog.vue'))
// 这些详情页在手机上有底部互动栏，页脚链接会被它挤在评论和栏之间，所以隐藏。
const DOCK_PAGES = new Set(['topic', 'daily-detail', 'frontier-detail', 'open-source-project', 'repo-detail', 'pack', 'scheme', 'idea-work'])
captureInvite()
let visitDay = ''
watch([() => route.fullPath, () => session.user?.uid], () => {
  const day = new Intl.DateTimeFormat('en-CA', {timeZone:'Asia/Shanghai'}).format(new Date())
  if (session.user && visitDay && day !== visitDay) session.loadSession()
  if (session.user) visitDay = day
})
const bare = computed(() => route.meta.bare === true)
const detailLayout = computed(() => ['topic', 'idea-work', 'open-source-project', 'repo-detail', 'pack', 'scheme', 'daily-detail', 'frontier-detail'].includes(String(route.name)))
const hasOwnHeader = computed(() => detailLayout.value || ['ai', 'create', 'app-works', 'me'].includes(String(route.name)))
const hasCommentDock = computed(() => DOCK_PAGES.has(String(route.name)))
const section = computed(() => sectionOf(route, meta.categories))
const destinations = ref<Record<Section, string>>({ home: '/', discuss: '/discuss', resources: '/resources' })
const communityLinks = computed<CommunityLink[]>(() => communitySections.map(({ key, label }) => ({ key, label, to: destinations.value[key] })))
const searchOpen = ref(false)
const searchQuery = ref('')

function openSearch() { searchOpen.value = true }
function submitSearch() {
  const q = searchQuery.value.trim()
  if (!q) return
  searchOpen.value = false
  void router.push({ name: 'search', query: { q } })
}
function searchShortcut(event: KeyboardEvent) {
  if (bare.value || event.defaultPrevented || event.key.toLowerCase() !== 'k' || !(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey || document.querySelector('dialog[open]')) return
  event.preventDefault()
  openSearch()
}
onMounted(() => {
  meta.load()
  session.restore()
  window.addEventListener('keydown', searchShortcut)
})
onBeforeUnmount(() => window.removeEventListener('keydown', searchShortcut))
watch(() => route.fullPath, path => {
  if (isCommunityRoot(route)) destinations.value[section.value] = path
  searchOpen.value = false
}, { immediate: true })
// Old posting and publishing links open the editor over their destination page.
watch(() => route.fullPath, async () => {
  composer.close()
  publisher.close()
  const { postDialog, postSection, postBoard, packDialog, ...query } = route.query
  const editing = postDialog === 'edit' && route.name === 'topic'
  const publishing = route.name === 'works' && typeof packDialog === 'string'
  if (!publishing && !editing && (postDialog !== 'new' || !isCommunityRoot(route))) return
  const page = router.resolve({ path: route.path, query, hash: route.hash })
  if (await router.replace(page) || route.fullPath !== page.fullPath) return
  if (publishing) {
    const id = Number(packDialog)
    publisher.open(Number.isSafeInteger(id) && id > 0 ? id : undefined)
    return
  }
  const editId = editing ? Number(route.params.id) : undefined
  const section = postSection === 'ideas' || postSection === 'resources' ? postSection : 'discuss'
  const category = typeof postBoard === 'string' ? postBoard : undefined
  composer.open({ editId, section, category })
}, { immediate: true })
// Keep the current editor mounted while reauthenticating, but conceal protected content.
const viewIdentity = ref('guest')
const protectedReady = computed(() => session.loggedIn && Boolean(session.user?.uid) && session.user?.uid === viewIdentity.value)
watch(() => session.user?.uid, uid => { if (uid) viewIdentity.value = uid }, { immediate:true })
watch(() => session.loggedIn, v => {
  if (!v && route.meta.auth) { session.loginReturnTo = route.fullPath; session.loginOpen = true }
})
</script>
<template>
  <RouterView v-if="bare" />
  <div v-else class="site-layout" :class="{'community-layout':areaOf(route) === 'community','community-root':isCommunityRoot(route),'detail-layout':detailLayout}">
    <a class="skip-link" href="#main-content">跳到正文</a>
    <AppHeader v-show="!hasOwnHeader" :section="section" :community-links="communityLinks" :search-open="searchOpen" @search="openSearch" />
    <main id="main-content" class="app-main">
      <section v-if="route.meta.auth && !protectedReady" class="session-locked"><template v-if="session.loggedIn"><p>正在确认账号信息…</p><button class="btn" @click="session.loadSession()">重新读取</button></template><template v-else><p>登录已过期，重新登录后可以继续。</p><button class="btn btn-primary" @click="session.loginOpen = true">重新登录</button></template></section>
      <RouterView v-slot="{ Component }"><component :is="Component" v-if="!route.meta.auth || viewIdentity !== 'guest'" :key="viewIdentity" v-show="!route.meta.auth || protectedReady" /></RouterView>
    </main>
    <footer v-if="route.name !== 'ai' && !detailLayout" class="app-footer" :class="{'footer-hide-narrow':hasOwnHeader || hasCommentDock || route.name === 'discuss'}"><nav aria-label="站点信息"><RouterLink to="/rules">社区规则</RouterLink><RouterLink to="/terms">用户协议</RouterLink><RouterLink to="/privacy">隐私政策</RouterLink><RouterLink to="/contact">联系我们</RouterLink><RouterLink to="/ai">下载客户端</RouterLink></nav></footer>
    <TabBar :section="section" :community-links="communityLinks" :search-open="searchOpen" :comment-dock="hasCommentDock" @search="openSearch" />
    <ModalDialog :open="searchOpen" title="搜索社区内容" :width="540" @close="searchOpen = false">
      <form class="site-search-form" role="search" @submit.prevent="submitSearch">
        <label class="sr-only" for="site-search">搜索帖子、作品和内容</label>
        <input id="site-search" v-model="searchQuery" type="search" placeholder="搜索帖子、作品和内容" maxlength="100" enterkeyhint="search" autofocus />
        <button class="btn btn-primary" type="submit">搜索</button>
      </form>
    </ModalDialog>
    <TopicComposer v-if="composer.target" :key="composer.target.editId ?? 'new'" />
    <PackPublishDialog v-if="publisher.target" />
  </div>
  <LoginDialog /><ToastHost /><AppearanceDialog />
</template>
<style>
:root{--mobile-nav-offset:calc(56px + env(safe-area-inset-bottom));--brand-soft:color-mix(in srgb,var(--brand) 8%,var(--surface))}
body:has(.site-layout) {
  --community-font: 'PingFang SC', 'HarmonyOS Sans SC', 'Hiragino Sans GB', 'Microsoft YaHei UI',
    'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --paper: #f2f5fa;
  --surface: #fff;
  --surface-2: #ebeff7;
  --ink: #111827;
  --ink-2: #475569;
  --ink-3: #627187;
  --community-muted: #838e9f;
  --community-divider: rgb(15 23 42 / 5.5%);
  --line: #0f172a12;
  --line-strong: #0f172a24;
  --surface-glass: color-mix(in srgb, var(--surface) 80%, transparent);
  --brand-soft: color-mix(in srgb, var(--brand-fill) 12%, var(--surface));
  --shadow-card: 0 6px 24px -8px #0f172a15, 0 1px 3px #0f172a0a;
}
html[data-theme="dark"] body:has(.site-layout) {
  --paper: #0f1219;
  --surface: #191f2c;
  --surface-2: #232a3b;
  --ink: #f8fafc;
  --ink-2: #cbd5e1;
  --ink-3: #9aa8bc;
  --community-muted: #8895aa;
  --community-divider: rgb(255 255 255 / 7.5%);
  --line: #ffffff15;
  --line-strong: #ffffff28;
  --shadow-card: 0 8px 26px -8px #0007;
}
.site-layout {
  --page-w: 960px;
  --header-h: 48px;
  /* The phone draft blends navigation with the canvas outside the scroll area. */
  --mobile-nav-surface: color-mix(in srgb, var(--surface) 80%, var(--paper));
  --site-canvas-gradient:
    radial-gradient(64% 38% at 18% 0%, color-mix(in srgb, var(--brand-fill) 10%, transparent), transparent 72%),
    radial-gradient(52% 32% at 88% 4%, color-mix(in srgb, var(--brand-hover) 8%, transparent), transparent 70%);
  max-width: var(--page-w);
  min-height: 100dvh;
  margin: 0 auto;
  padding-bottom: var(--mobile-nav-offset);
}
@media (width < 960px) {
  html:has(.site-layout) { scrollbar-gutter: stable; }
  body:has(.site-layout) { isolation: isolate; }
  .site-layout.community-root {
    --header-h: 46px;
    --mobile-header-image: linear-gradient(var(--surface-glass), var(--surface-glass)), var(--site-canvas-gradient);
  }

  /* Draw the draft's glass and canvas together, with content below both. */
  .site-layout.community-root :is(.community-header, .page-section-tabs) {
    font-family: var(--community-font);
    background-color: var(--paper);
    background-image: var(--mobile-header-image);
    background-size: 100% 100dvh;
    background-repeat: no-repeat;
  }
  .site-layout.community-root .page-section-tabs {
    background-position: center calc(-1 * (var(--header-h) + env(safe-area-inset-top)));
  }

  /* The phone comp scrolls its content inside a stationary canvas. */
  .site-layout::before {
    content: '';
    position: fixed;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    background-image: var(--site-canvas-gradient);
  }
}
.app-main{container:page / inline-size;max-width:var(--page-w);margin:0 auto;padding:8px 24px 40px;min-height:calc(100dvh - 180px)}
.site-search-form { display: flex; align-items: center; gap: 8px; }
.site-search-form input { flex: 1; min-width: 0; height: 44px; padding: 0 14px; border: 1px solid var(--line); border-radius: 12px; background: var(--surface-2); font-size: 16px; }
.site-search-form .btn { min-height: 44px; border-radius: 999px; }
.app-footer{max-width:var(--page-w);margin:0 auto;padding:16px 24px 28px;color:var(--ink-3);font-size:12px}.app-footer nav{display:flex;flex-wrap:wrap;gap:8px 20px}.app-footer p{margin:8px 0 0}.app-footer a:hover{color:var(--ink)}
.skip-link{position:fixed;left:12px;top:-80px;z-index:100;padding:12px 20px;background:var(--surface);border-radius:12px}.skip-link:focus{top:8px}
@media (width < 960px) {
  .app-main { padding: 4px 20px 24px; }
  .app-footer { padding: 16px 20px 20px; }
  .footer-hide-narrow { display: none; }
}
@media(max-width:767px){.app-main{padding:8px 12px 20px}.community-root .app-main{padding-inline:6px}.app-footer{padding:16px 12px 12px}}
@media (width >= 960px) {
  .site-layout {
    max-width: none;
    --page-w: 1440px;
    --desktop-rail-width: 236px;
    --mobile-nav-offset: 0px;
    padding-inline-start: var(--desktop-rail-width);
    background-color: var(--paper);
    background-image: var(--site-canvas-gradient);
  }
}
@media(min-width:1100px){
  .app-main{padding:12px 32px 48px}
  .app-footer{padding:20px 32px 28px;border-top:1px solid var(--line);display:flex;justify-content:space-between;align-items:center;gap:20px;font-size:12px}
  .app-footer p{margin:0}.app-footer nav{gap:16px}
}
@media(min-width:1600px){.app-main{padding-left:40px;padding-right:40px}}
.site-layout.detail-layout .app-main { max-width: 1200px; padding: 22px 28px 48px; }
@media (width < 960px) {
  .site-layout.detail-layout {
    --header-h: calc(49px + env(safe-area-inset-top));
    --mobile-nav-offset: calc(52px + env(safe-area-inset-bottom));
    background: var(--surface);
  }
  .site-layout.detail-layout::before { display: none; }
  .site-layout.detail-layout .app-main { padding: 0 0 24px; }
}
@media(prefers-reduced-motion:reduce){*,*::before,*::after{animation-duration:.01ms!important;transition-duration:.01ms!important;scroll-behavior:auto!important}}
</style>
