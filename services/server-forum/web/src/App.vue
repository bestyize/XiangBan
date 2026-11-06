<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppHeader from '@/components/AppHeader.vue'
import TabBar from '@/components/TabBar.vue'
import LoginDialog from '@/components/LoginDialog.vue'
import AppearanceDialog from '@/components/AppearanceDialog.vue'
import ToastHost from '@/components/ToastHost.vue'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { captureInvite } from '@/lib/invite'
import { areaOf } from '@/lib/sections'
const route = useRoute(), session = useSession(), meta = useMeta()
captureInvite()
let visitDay = ''
watch([() => route.fullPath, () => session.user?.uid], () => {
  const day = new Intl.DateTimeFormat('en-CA', {timeZone:'Asia/Shanghai'}).format(new Date())
  if (session.user && visitDay && day !== visitDay) session.loadSession()
  if (session.user) visitDay = day
})
const bare = computed(() => route.meta.bare === true)
onMounted(() => {meta.load();session.restore()})
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
  <div v-else class="site-layout" :class="{'focus-layout':route.meta.focus,'creation-layout':route.name === 'create','community-layout':areaOf(route) === 'community','community-root':['home','discuss','resources'].includes(String(route.name))}">
    <a class="skip-link" href="#main-content">跳到正文</a>
    <AppHeader />
    <main id="main-content" class="app-main" :class="{'focus-main':route.meta.focus}">
      <section v-if="route.meta.auth && !protectedReady" class="session-locked"><template v-if="session.loggedIn"><p>正在确认账号信息…</p><button class="btn" @click="session.loadSession()">重新读取</button></template><template v-else><p>登录已过期，重新登录后可以继续。</p><button class="btn btn-primary" @click="session.loginOpen = true">重新登录</button></template></section>
      <RouterView v-slot="{ Component }"><component :is="Component" v-if="!route.meta.auth || viewIdentity !== 'guest'" :key="viewIdentity" v-show="!route.meta.auth || protectedReady" /></RouterView>
    </main>
    <footer v-if="!route.meta.focus" class="app-footer" :class="{'footer-hide-narrow':route.name === 'topic'}"><nav aria-label="站点信息"><RouterLink to="/rules">社区规则</RouterLink><RouterLink to="/terms">用户协议</RouterLink><RouterLink to="/privacy">隐私政策</RouterLink><a href="mailto:support@xiangban.xyz">联系我们</a><a href="https://wallpaper.xiangban.xyz/#desktop-downloads" target="_blank" rel="noopener noreferrer">下载客户端</a></nav><p>发现好内容，体验新想法。</p></footer>
    <TabBar />
  </div>
  <LoginDialog /><ToastHost /><AppearanceDialog />
</template>
<style>
:root{--mobile-nav-offset:calc(56px + env(safe-area-inset-bottom));--brand-soft:color-mix(in srgb,var(--brand) 8%,var(--surface))}
.site-layout{max-width:1023px;margin:0 auto;padding-bottom:var(--mobile-nav-offset);min-height:100dvh;--page-w:1023px;background:color-mix(in srgb,var(--surface) 55%,var(--paper))}.app-main{container:page / inline-size;max-width:var(--page-w);margin:0 auto;padding:8px 24px 40px;min-height:calc(100dvh - 180px)}
.app-footer{max-width:var(--page-w);margin:0 auto;padding:16px 24px 28px;color:var(--ink-3);font-size:12px}.app-footer nav{display:flex;flex-wrap:wrap;gap:8px 20px}.app-footer p{margin:8px 0 0}.app-footer a:hover{color:var(--ink)}
.skip-link{position:fixed;left:12px;top:-80px;z-index:100;padding:12px 20px;background:var(--surface);border-radius:12px}.skip-link:focus{top:8px}
@media(max-width:1023px){.app-main{padding:4px 20px 24px}.app-footer{padding:16px 20px 20px}}
@media(max-width:767px){.footer-hide-narrow{display:none}.creation-layout .site-header{display:none}.app-main{padding:0 12px 20px}.app-main.focus-main{padding-bottom:0}.community-layout .app-main{padding-left:8px;padding-right:8px}.app-footer{padding:16px 12px 12px}}
@media(min-width:1100px){
  .app-main{padding:12px 32px 48px}
  .app-footer{padding:20px 32px 28px;border-top:1px solid var(--line);display:flex;justify-content:space-between;align-items:center;gap:20px;font-size:12px}
  .app-footer p{margin:0}.app-footer nav{gap:16px}
}
@media(min-width:1600px){.app-main{padding-left:40px;padding-right:40px}}
@media(prefers-reduced-motion:reduce){*,*::before,*::after{animation-duration:.01ms!important;transition-duration:.01ms!important;scroll-behavior:auto!important}}
</style>
