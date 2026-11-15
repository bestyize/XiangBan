<script setup lang="ts">
import { computed } from 'vue'
import { useSession } from '@/stores/session'
import { useNotifications } from '@/stores/notifications'
import { useTheme } from '@/lib/theme'
import { useHead } from '@/lib/head'
import AppIcon from '@/components/AppIcon.vue'
import UserAvatar from '@/components/UserAvatar.vue'

useHead(() => ({ title: '我的' }))
const session = useSession(), inbox = useNotifications(), { openAppearance } = useTheme()
const profile = computed(() => session.user ? '/u/' + encodeURIComponent(session.user.uid) : '')
</script>

<template>
  <div class="me-page">
    <header class="me-header">
      <h1>我的</h1>
      <RouterLink v-if="session.loggedIn" to="/settings" class="me-setting" aria-label="设置"><AppIcon name="settings" /></RouterLink>
      <button v-else type="button" class="me-setting" aria-label="外观设置" @click="openAppearance"><AppIcon name="settings" /></button>
    </header>

    <section class="profile-block" aria-label="账号信息">
      <UserAvatar :name="session.user?.nickname || '?'" :src="session.user?.avatar_url" :size="64" />
      <div class="profile-copy">
        <RouterLink v-if="profile" :to="profile" class="profile-name">{{ session.user?.nickname }}</RouterLink>
        <strong v-else class="profile-name">{{ session.loggedIn ? '已登录' : '未登录' }}</strong>
        <p v-if="session.accountId" data-testid="account-id">账号 ID {{ session.accountId }}</p>
        <RouterLink v-if="session.user" to="/points" class="points">{{ session.user.points }} 积分</RouterLink>
        <p v-if="!session.loggedIn">登录后同步你在社区的身份</p>
      </div>
      <button v-if="!session.loggedIn" class="btn btn-primary" type="button" @click="session.requireLogin()">登录</button>
    </section>

    <nav class="me-quick" aria-label="常用入口">
      <RouterLink to="/me/works"><span><AppIcon name="folder" /></span>我的作品</RouterLink>
      <RouterLink to="/notifications"><span><AppIcon name="chat" /></span>通知{{ inbox.unread ? ' · ' + inbox.unread : '' }}</RouterLink>
      <RouterLink to="/ai"><span><AppIcon name="monitor" /></span>创作</RouterLink>
    </nav>

    <nav class="me-group" aria-label="我的账号与社区">
      <RouterLink class="me-row" to="/settings#bm-title">我的收藏<AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink v-if="profile" class="me-row" :to="profile + '?tab=following'">关注与粉丝<AppIcon name="arrow" :size="16" /></RouterLink>
      <button v-else class="me-row" type="button" @click="session.requireLogin()">关注与粉丝<AppIcon name="arrow" :size="16" /></button>
      <RouterLink class="me-row" to="/settings#blocks-title">屏蔽的用户<AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/ai">AI 额度<small>在客户端查看</small><AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/ai">我的设备<small>在客户端管理</small><AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/discuss?tab=site">帮助与反馈<AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/contact">交流群<AppIcon name="arrow" :size="16" /></RouterLink>
    </nav>

    <nav class="me-group" aria-label="更多">
      <RouterLink class="me-row" to="/me/content-bookmarks">作品收藏<AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/points">社区积分<AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/ai">下载小伴客户端<AppIcon name="arrow" :size="16" /></RouterLink>
      <RouterLink class="me-row" to="/privacy">隐私政策<AppIcon name="arrow" :size="16" /></RouterLink>
    </nav>
  </div>
</template>

<style scoped>
.me-page { max-width: 680px; margin: 0 auto; }
.me-header { display: grid; grid-template-columns: 44px 1fr 44px; align-items: center; min-height: 48px; background: var(--surface); border-radius: 14px 14px 0 0; padding-inline: 8px; }
h1 { grid-column: 2; margin: 0; font-size: 17px; font-weight: 650; text-align: center; }
.me-setting { display: grid; place-items: center; min-height: 44px; border: 0; background: none; color: var(--ink-2); }
.profile-block { display: flex; align-items: center; gap: 14px; padding: 10px 16px 18px; background: var(--surface); border-radius: 0 0 14px 14px; }
.profile-copy { flex: 1; min-width: 0; }
.profile-name { font-size: 22px; font-weight: 650; overflow-wrap: anywhere; }
.profile-copy p, .points { display: block; margin: 4px 0 0; color: var(--ink-2); font-size: 12px; }
.profile-block .btn { flex: none; border-radius: 99px; min-height: 36px; }
.me-quick { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); margin-block: 8px; padding: 14px 6px; border-radius: 14px; background: var(--surface); }
.me-quick a { display: flex; flex-direction: column; align-items: center; gap: 6px; font-size: 12px; }
.me-quick a > span { display: grid; place-items: center; width: 44px; height: 44px; border-radius: 14px; background: var(--surface-2); color: var(--ink); }
.me-group { margin-block: 8px; border-radius: 14px; overflow: hidden; background: var(--surface); }
.me-row { display: flex; align-items: center; gap: 10px; width: 100%; min-height: 48px; padding: 10px 16px; border: 0; border-bottom: 1px solid var(--line); background: none; color: var(--ink); text-align: start; font-size: 15px; }
.me-row:last-child { border-bottom: 0; }
.me-row small, .me-row > svg { margin-inline-start: auto; color: var(--ink-3); }
.me-row small + svg { margin-inline-start: 0; }
.me-row:hover, .me-quick a:hover, .me-setting:hover { color: var(--brand); }
@media (max-width: 767px) {
  .me-page { margin-inline: -12px; }
  .me-header, .profile-block, .me-quick, .me-group { border-radius: 0; }
}
</style>
