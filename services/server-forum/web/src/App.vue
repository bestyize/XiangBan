<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppHeader from '@/components/AppHeader.vue'
import TabBar from '@/components/TabBar.vue'
import LoginDialog from '@/components/LoginDialog.vue'
import ToastHost from '@/components/ToastHost.vue'
import { useSession } from '@/stores/session'
import { useMeta } from '@/stores/meta'
import { captureInvite } from '@/lib/invite'

const route = useRoute()
const router = useRouter()
const session = useSession()
const meta = useMeta()
captureInvite()
let visitDay = ''
watch([() => route.fullPath, () => session.user?.uid], () => {
  const day = new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Shanghai' }).format(new Date())
  if (session.user && visitDay && day !== visitDay) session.loadSession()
  if (session.user) visitDay = day
})

/** 管理后台自带布局，不显示前台的顶栏和底部导航。 */
const bare = computed(() => route.meta.bare === true)

onMounted(() => {
  meta.load()
  session.restore()
})

// 在需要登录的页面里退出登录，回到首页。
watch(
  () => session.loggedIn,
  v => {
    if (!v && route.meta.auth) router.replace({ name: 'home' })
  },
)
</script>

<template>
  <template v-if="bare">
    <RouterView />
  </template>
  <template v-else>
    <AppHeader />
    <main class="app-main">
      <RouterView />
    </main>
    <footer class="app-footer">
      <nav aria-label="站点信息">
        <RouterLink to="/rules">社区规则</RouterLink>
        <RouterLink to="/terms">用户协议</RouterLink>
        <RouterLink to="/privacy">隐私政策</RouterLink>
        <a href="mailto:support@xiangban.xyz">联系我们</a>
      </nav>
      <p>超级小伴是开发者和 AI 用户的社区。福利羊毛由用户分享，领取前请确认官方条款。</p>
    </footer>
    <TabBar />
  </template>
  <LoginDialog />
  <ToastHost />
</template>

<style>
.app-main {
  max-width: var(--page-w);
  margin: 0 auto;
  padding: 20px 20px 40px;
  min-height: calc(100vh - var(--header-h) - 120px);
}
.app-footer {
  max-width: var(--page-w);
  margin: 0 auto;
  padding: 20px 20px 36px;
  color: var(--ink-3);
  font-size: var(--fs-xs);
  border-top: 1px solid var(--line);
}
.app-footer nav {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 18px;
  margin-bottom: 6px;
}
.app-footer a:hover {
  color: var(--ink);
}
.app-footer p {
  margin: 0;
}
@media (max-width: 767px) {
  .app-main {
    padding: 12px 12px 24px;
  }
  .app-footer {
    padding: 16px 14px calc(var(--tabbar-h) + 24px + env(safe-area-inset-bottom));
  }
}
</style>
