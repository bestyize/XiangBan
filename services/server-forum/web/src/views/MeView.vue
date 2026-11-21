<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { contentApi } from '@/api/content'
import { api } from '@/api/forum'
import AppIcon from '@/components/AppIcon.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { useCommunityResource } from '@/components/community/resource'
import { useHead } from '@/lib/head'
import { useTheme } from '@/lib/theme'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '我的' }))
const session = useSession(), toast = useToast(), { openAppearance, resolved } = useTheme()
const profile = computed(() => session.user ? '/u/' + encodeURIComponent(session.user.uid) : '')
const dailyCheckIn = useCommunityResource(async () => session.user ? (await api.points()).checked_in_today : null)
const summary = useCommunityResource(async () => {
  if (!session.user) return null
  const [packs, bookmarks] = await Promise.allSettled([contentApi.myPacks(), api.bookmarks()])
  return {
    published: packs.status === 'fulfilled' ? packs.value.items.filter(pack => pack.status === 'published').length : null,
    bookmarked: bookmarks.status === 'fulfilled' ? bookmarks.value.total : null,
  }
})
watch(() => session.user?.uid, () => {
  void dailyCheckIn.load()
  void summary.load()
}, { immediate: true })

const checkingIn = ref(false)
const checkInDisabled = computed(() => !session.user || session.restoring || dailyCheckIn.loading.value || checkingIn.value || dailyCheckIn.data.value === true)
async function checkIn() {
  const uid = session.user?.uid
  if (!uid || checkInDisabled.value) return
  checkingIn.value = true
  try {
    const result = await api.checkIn()
    if (session.user?.uid !== uid) return
    if (session.info) session.info.user.points = result.balance
    dailyCheckIn.data.value = true
    toast.show(result.earned ? `签到成功，获得 ${result.earned} 积分` : '今天已经签到')
  } catch (error) {
    if (session.user?.uid === uid) toast.error(error)
  } finally {
    checkingIn.value = false
  }
}

const links = computed(() => {
  const count = summary.data.value
  return [
    { to: '/me/works', label: '我的作品', detail: count?.published == null ? '' : `${count.published} 个已发布` },
    { to: '/settings#bm-title', label: '我的收藏', detail: count?.bookmarked == null ? '' : `${count.bookmarked} 条帖子` },
    { to: '/ai', label: 'AI 额度与兑换', detail: '在桌面/手机查看' },
    { to: '/contact', label: '社区交流群', detail: 'QQ 群 / 微信群' },
  ]
})
</script>

<template>
  <div class="me-page">
    <header class="me-header">
      <h1>我的</h1>
      <button type="button" class="me-appearance" aria-label="外观与配色" aria-haspopup="dialog" @click="openAppearance">
        <AppIcon :name="resolved === 'dark' ? 'sun' : 'moon'" :size="17" />
      </button>
    </header>

    <div class="me-card">
      <section class="me-surface profile-block" aria-label="账号信息">
        <UserAvatar class="profile-avatar" :name="session.user?.nickname || '?'" :src="session.user?.avatar_url"
          :size="52" :initial-size="19" />
        <div class="profile-copy">
          <RouterLink v-if="profile" :to="profile" class="profile-name">{{ session.user?.nickname }}</RouterLink>
          <strong v-else class="profile-name">{{ session.loggedIn ? '已登录' : '未登录' }}</strong>
          <div v-if="session.user || session.accountId" class="profile-meta section-meta">
            <span v-if="session.accountId" data-testid="account-id">账号 ID {{ session.accountId }}</span>
            <RouterLink v-if="session.user" class="profile-points" to="/points">
              <span v-if="session.accountId" aria-hidden="true">· </span>{{ session.user.points }} 积分
            </RouterLink>
          </div>
        </div>
        <button v-if="session.loggedIn" class="me-profile-action" type="button" :disabled="checkInDisabled" @click="checkIn">
          {{ checkingIn ? '签到中…' : dailyCheckIn.data.value ? '今日已签到' : '每日签到' }}
        </button>
        <button v-else class="me-profile-action" type="button" @click="session.requireLogin()">登录</button>
      </section>

      <nav class="me-surface me-group" aria-label="我的入口">
        <RouterLink v-for="item in links" :key="item.to" class="me-row" :to="item.to">
          <span>{{ item.label }}</span>
          <span class="row-detail section-meta">{{ item.detail }} <AppIcon name="arrow" :size="14" /></span>
        </RouterLink>
      </nav>
    </div>
  </div>
</template>

<style scoped>
:global(.site-layout .app-main:has(.me-page)) {
  max-width: 1200px;
  padding: 22px 28px 48px;
}
.me-page { font-family: var(--community-font); font-size: 14px; line-height: 1.5; }
.me-header { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
.me-appearance { display: none; }
.me-card { max-width: 680px; margin: 0 auto; display: grid; gap: 12px; }
.me-surface {
  background: var(--surface);
  border: 1px solid transparent;
  border-radius: 18px;
  box-shadow: var(--shadow-card);
}
.profile-block { display: flex; align-items: center; gap: 14px; padding: 16px 18px; }
.profile-avatar { border: 0; background: var(--brand-soft); color: var(--brand); }
.profile-copy { flex: 1; min-width: 0; }
.profile-name { font-size: 18px; font-weight: 700; overflow-wrap: anywhere; }
.section-meta { color: var(--community-muted); font-size: 12px; }
.profile-meta { display: flex; flex-wrap: wrap; gap: 0 4px; overflow-wrap: anywhere; }
.profile-points { white-space: nowrap; }
.profile-name:hover, .profile-points:hover { color: var(--brand); }
.me-profile-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  height: 32px;
  padding: 0 14px;
  border: 1px solid var(--line-strong);
  border-radius: 999px;
  background: var(--surface);
  color: var(--ink);
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}
.me-profile-action:enabled:hover { border-color: var(--brand); color: var(--brand); }
.me-profile-action:disabled { color: var(--community-muted); cursor: default; }
.me-group { overflow: hidden; }
.me-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 13px 16px;
  border-bottom: 1px solid var(--community-divider);
  transition: background-color 0.14s;
}
.me-row:hover { background: var(--surface-2); }
.me-row:last-child { border-bottom: 0; }
.me-row:focus-visible { outline-offset: -2px; }
.row-detail { display: inline-flex; align-items: center; gap: 4px; text-align: end; }
.row-detail svg { flex: none; }
@media (width < 960px) {
  :global(.site-layout .app-main:has(.me-page)) { padding: 0 6px 20px; }
  :global(.site-layout:has(.me-page) .app-footer) { display: none; }
  .me-header {
    position: sticky;
    top: 0;
    z-index: 20;
    display: grid;
    grid-template-columns: 34px minmax(0, 1fr) 34px;
    align-items: center;
    width: auto;
    height: calc(47px + env(safe-area-inset-top));
    margin: 0 -6px 8px;
    padding: env(safe-area-inset-top) 10px 0;
    overflow: visible;
    clip-path: none;
    border-bottom: 1px solid var(--community-divider);
    background: var(--mobile-nav-surface);
    backdrop-filter: blur(16px) saturate(150%);
  }
  .me-header h1 { grid-column: 2; margin: 0; text-align: center; font-size: 16px; font-weight: 700; }
  .me-appearance {
    grid-column: 3;
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--ink);
  }
  .me-appearance:active { background: var(--surface-2); }
}
</style>
