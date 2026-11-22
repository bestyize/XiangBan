<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { contentApi } from '@/api/content'
import { api } from '@/api/forum'
import AppIcon from '@/components/AppIcon.vue'
import AvatarEditor from '@/components/AvatarEditor.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { useCommunityResource } from '@/components/community/resource'
import { useHead } from '@/lib/head'
import { useTheme } from '@/lib/theme'
import { useNotifications } from '@/stores/notifications'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '我的' }))
const session = useSession()
const toast = useToast()
const inbox = useNotifications()
const { openAppearance, resolved } = useTheme()

const avatarOpen = ref(false)
const profile = computed(() => (session.user ? '/u/' + encodeURIComponent(session.user.uid) : ''))
const dailyCheckIn = useCommunityResource(async () => (session.user ? (await api.points()).checked_in_today : null))
const summary = useCommunityResource(async () => {
  if (!session.user) return null
  const [packs, bookmarks] = await Promise.allSettled([contentApi.myPacks(), api.bookmarks()])
  return {
    works: packs.status === 'fulfilled' ? packs.value.items.length : null,
    bookmarked: bookmarks.status === 'fulfilled' ? bookmarks.value.total : null,
  }
})

watch(
  () => session.user?.uid,
  () => {
    void dailyCheckIn.load()
    void summary.load()
  },
  { immediate: true },
)

const checkingIn = ref(false)
const checkInDisabled = computed(
  () => !session.user || session.restoring || dailyCheckIn.loading.value || checkingIn.value || dailyCheckIn.data.value === true,
)

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

async function copyId() {
  if (!session.accountId) return
  try {
    await navigator.clipboard?.writeText(session.accountId)
    toast.show(`已复制账号 ID ${session.accountId}`)
  } catch {
    toast.show(`账号 ID ${session.accountId}`)
  }
}

const unreadBadge = computed(() => (inbox.unread > 99 ? '99+' : inbox.unread > 0 ? String(inbox.unread) : ''))
</script>

<template>
  <div class="me-page">
    <div class="me-shell">
      <header class="me-pc-head">
        <h1>我的</h1>
        <RouterLink to="/settings" class="me-pill secondary">
          <AppIcon name="gear" :size="15" />
          <span>设置</span>
        </RouterLink>
      </header>

      <div class="me-top-slab">
        <header class="me-h5-bar">
          <h1 class="me-h5-title">我的</h1>
          <div class="me-h5-actions">
            <button type="button" class="me-icon-btn" aria-label="外观与配色" aria-haspopup="dialog" @click="openAppearance">
              <AppIcon :name="resolved === 'dark' ? 'sun' : 'moon'" :size="19" />
            </button>
            <RouterLink to="/settings" class="me-icon-btn" aria-label="设置">
              <AppIcon name="gear" :size="19" />
            </RouterLink>
          </div>
        </header>

        <section class="me-hero" aria-label="账号信息">
          <button
            v-if="session.loggedIn"
            type="button"
            class="me-avatar-btn"
            aria-label="更换头像"
            @click="avatarOpen = true"
          >
            <UserAvatar class="me-avatar" :name="session.user?.nickname || '?'" :src="session.user?.avatar_url" :size="64" :initial-size="22" />
          </button>
          <button v-else type="button" class="me-avatar-guest" aria-label="登录" @click="session.requireLogin()">
            <AppIcon name="me" :size="26" />
          </button>

          <div class="me-copy">
            <template v-if="session.loggedIn">
              <RouterLink v-if="profile" :to="profile" class="me-name">{{ session.user?.nickname || '已登录' }}</RouterLink>
              <strong v-else class="me-name">已登录</strong>
              <div class="me-sub-row">
                <button v-if="session.accountId" type="button" class="me-id-btn" data-testid="account-id" @click="copyId">
                  <span>ID {{ session.accountId }}</span>
                  <AppIcon name="copy" :size="12" />
                </button>
                <button
                  type="button"
                  class="me-checkin-inline"
                  :class="{ done: dailyCheckIn.data.value === true }"
                  :disabled="checkInDisabled"
                  @click="checkIn"
                >
                  {{ checkingIn ? '签到中…' : dailyCheckIn.data.value ? '已签到' : '签到' }}
                </button>
              </div>
            </template>
            <template v-else>
              <strong class="me-name">登录超级小伴</strong>
              <p class="me-guest-desc">同步作品、收藏与通知</p>
            </template>
          </div>

          <div class="me-hero-ops">
            <template v-if="session.loggedIn">
              <button
                type="button"
                class="me-pill me-checkin-pc"
                :class="dailyCheckIn.data.value ? 'done' : 'secondary'"
                :disabled="checkInDisabled"
                @click="checkIn"
              >
                <AppIcon v-if="dailyCheckIn.data.value" name="check" :size="14" />
                <span>{{ checkingIn ? '签到中…' : dailyCheckIn.data.value ? '今日已签到' : '每日签到' }}</span>
              </button>
              <RouterLink v-if="profile" :to="profile" class="me-home-link">
                <span>主页</span>
                <AppIcon name="arrow" :size="14" />
              </RouterLink>
            </template>
            <button v-else type="button" class="me-pill primary" @click="session.requireLogin()">登录</button>
          </div>
        </section>

        <div class="me-slab-divider" />

        <nav class="me-quick-grid" aria-label="快捷入口">
          <RouterLink to="/settings#bm-title" class="me-quick-item">
            <span class="me-quick-ico"><AppIcon name="bookmark" :size="19" /></span>
            <span class="me-quick-copy">
              <strong>我的收藏</strong>
              <small>{{ session.loggedIn ? (summary.data.value?.bookmarked != null ? `${summary.data.value.bookmarked} 条帖子` : '查看收藏') : '登录后查看' }}</small>
            </span>
          </RouterLink>
          <RouterLink to="/me/works" class="me-quick-item">
            <span class="me-quick-ico"><AppIcon name="folder" :size="19" /></span>
            <span class="me-quick-copy">
              <strong>我的作品</strong>
              <small>{{ session.loggedIn ? (summary.data.value?.works != null ? `${summary.data.value.works} 个作品` : '管理版本') : '登录后管理' }}</small>
            </span>
          </RouterLink>
          <RouterLink to="/notifications" class="me-quick-item">
            <span class="me-quick-ico">
              <AppIcon name="bell" :size="19" />
              <span v-if="session.loggedIn && unreadBadge" class="me-badge">{{ unreadBadge }}</span>
            </span>
            <span class="me-quick-copy">
              <strong>通知</strong>
              <small>{{ session.loggedIn ? (inbox.unread > 0 ? `${inbox.unread} 条未读` : '暂无未读') : '登录后查看' }}</small>
            </span>
          </RouterLink>
        </nav>
      </div>

      <p class="me-sec-title">服务与社群</p>
      <div class="me-groups-wrap">
        <nav class="me-group-card" aria-label="服务与设置">
          <RouterLink to="/ai?panel=account" class="me-menu-row">
            <AppIcon name="cpu" :size="19" class="lead" />
            <span class="me-menu-copy">
              <strong>模型配置</strong>
              <small>AI 额度、兑换与模型切换</small>
            </span>
            <AppIcon name="arrow" :size="14" class="chev" />
          </RouterLink>
          <RouterLink to="/settings" class="me-menu-row">
            <AppIcon name="gear" :size="19" class="lead" />
            <span class="me-menu-copy">
              <strong>账号与设置</strong>
              <small>头像、绑定手机邮箱、深浅色</small>
            </span>
            <AppIcon name="arrow" :size="14" class="chev" />
          </RouterLink>
        </nav>

        <nav class="me-group-card" aria-label="帮助与社群">
          <RouterLink to="/discuss?tab=site" class="me-menu-row">
            <AppIcon name="help" :size="19" class="lead" />
            <span class="me-menu-copy">
              <strong>帮助与反馈</strong>
              <small>站方讨论与问题反馈</small>
            </span>
            <AppIcon name="arrow" :size="14" class="chev" />
          </RouterLink>
          <RouterLink to="/contact" class="me-menu-row">
            <AppIcon name="community" :size="19" class="lead" />
            <span class="me-menu-copy">
              <strong>交流群</strong>
              <small>QQ 群 / 微信群</small>
            </span>
            <AppIcon name="arrow" :size="14" class="chev" />
          </RouterLink>
        </nav>
      </div>
    </div>

    <AvatarEditor
      v-if="session.loggedIn"
      v-model:open="avatarOpen"
      :name="session.user?.nickname || '?'"
      :current="session.user?.avatar_url ?? null"
    />
  </div>
</template>

<style scoped>
:global(.site-layout .app-main:has(.me-page)) {
  max-width: 1200px;
  padding: 22px 28px 48px;
}

.me-page {
  font-family: var(--community-font);
  font-size: 14px;
  line-height: 1.5;
}

.me-shell {
  max-width: 760px;
  margin: 0 auto;
}

.me-pc-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
.me-pc-head h1 {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 700;
}

.me-h5-bar { display: none; }

.me-top-slab {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.me-hero {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 22px;
}

.me-avatar-btn {
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: transparent;
  flex: none;
  cursor: pointer;
}
.me-avatar {
  box-shadow: 0 0 0 1px var(--line);
}
.me-avatar-guest {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  border: 1px solid var(--line);
  background: var(--surface-2);
  color: var(--ink-3);
  display: grid;
  place-items: center;
  flex: none;
}

.me-copy {
  flex: 1;
  min-width: 0;
}
.me-name {
  display: block;
  font-size: 21px;
  line-height: 28px;
  font-weight: 700;
  color: var(--ink);
  overflow-wrap: anywhere;
}
.me-name:hover { color: var(--brand); }
.me-guest-desc {
  margin: 3px 0 0;
  font-size: 13px;
  color: var(--ink-3);
}

.me-sub-row {
  margin-top: 5px;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.me-id-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 0;
  border: 0;
  background: transparent;
  color: var(--ink-3);
  font-size: 12px;
}
.me-id-btn:hover { color: var(--ink-2); }

.me-checkin-inline { display: none; }

.me-hero-ops {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
}

.me-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 32px;
  padding: 0 14px;
  border: 0;
  border-radius: 999px;
  font-size: 12.5px;
  font-weight: 600;
  white-space: nowrap;
  transition: opacity 0.14s, background-color 0.14s;
}
.me-pill:hover:not(:disabled) { opacity: 0.88; }
.me-pill:disabled { cursor: default; }
.me-pill.primary { background: var(--brand-soft); color: var(--brand); }
.me-pill.secondary { background: var(--surface-2); color: var(--ink-2); font-weight: 500; }
.me-pill.done { background: rgba(47, 163, 107, 0.12); color: #2fa36b; font-weight: 500; }

.me-home-link {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  height: 32px;
  padding: 0 6px 0 10px;
  border-radius: 999px;
  color: var(--ink-3);
  font-size: 13px;
}
.me-home-link:hover { background: var(--surface-2); color: var(--ink); }

.me-slab-divider {
  height: 1px;
  background: var(--line);
}

.me-quick-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.me-quick-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 15px 18px;
  transition: background-color 0.14s;
}
.me-quick-item:hover { background: var(--surface-2); }
.me-quick-item + .me-quick-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 16px;
  bottom: 16px;
  width: 1px;
  background: var(--line);
}
.me-quick-ico {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: var(--surface-2);
  color: var(--ink);
  display: grid;
  place-items: center;
  flex: none;
}
.me-badge {
  position: absolute;
  top: -4px;
  right: -5px;
  min-width: 17px;
  height: 17px;
  padding: 0 4.5px;
  border-radius: 999px;
  background: var(--danger, #e3203c);
  color: #fff;
  font-size: 10.5px;
  line-height: 17px;
  font-weight: 600;
  text-align: center;
  box-shadow: 0 0 0 2px var(--surface);
}
.me-quick-copy strong {
  display: block;
  font-size: 14.5px;
  font-weight: 600;
  color: var(--ink);
}
.me-quick-copy small {
  display: block;
  font-size: 12px;
  color: var(--ink-3);
  margin-top: 1px;
}

.me-sec-title {
  margin: 18px 4px 8px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--ink-3);
}

.me-groups-wrap {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: var(--shadow-card);
  overflow: hidden;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.me-group-card {
  display: contents;
}
.me-menu-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 15px 18px;
  transition: background-color 0.14s;
}
.me-menu-row:hover { background: var(--surface-2); }
.me-menu-row:nth-child(even)::before {
  content: '';
  position: absolute;
  left: 0;
  top: 14px;
  bottom: 14px;
  width: 1px;
  background: var(--line);
}
.me-group-card:nth-child(2) .me-menu-row::after {
  content: '';
  position: absolute;
  top: 0;
  left: 16px;
  right: 16px;
  height: 1px;
  background: var(--line);
}
.me-menu-row .lead { color: var(--ink-2); flex: none; }
.me-menu-row .chev { color: var(--ink-3); flex: none; opacity: 0.7; }
.me-menu-copy { flex: 1; min-width: 0; }
.me-menu-copy strong { display: block; font-size: 14.5px; font-weight: 550; color: var(--ink); }
.me-menu-copy small { display: block; font-size: 12px; color: var(--ink-3); margin-top: 1px; }

/* ─── Mobile H5: Scheme 1 Full-bleed Refined ─── */
@media (width < 960px) {
  :global(.site-layout .app-main:has(.me-page)) { padding: 0 0 28px; }
  :global(.site-layout:has(.me-page) .app-footer) { display: none; }

  .me-pc-head,
  .me-sec-title,
  .me-checkin-pc { display: none; }

  .me-top-slab {
    border: 0;
    border-radius: 0;
    box-shadow: none;
  }

  .me-h5-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: calc(env(safe-area-inset-top) + 6px) 10px 4px 16px;
    background: var(--surface);
  }
  .me-h5-title {
    margin: 0;
    font-size: 18px;
    line-height: 24px;
    font-weight: 650;
  }
  .me-h5-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .me-icon-btn {
    width: 38px;
    height: 38px;
    border: 0;
    border-radius: 12px;
    background: transparent;
    color: var(--ink);
    display: grid;
    place-items: center;
  }
  .me-icon-btn:active { background: var(--surface-2); }

  .me-hero {
    padding: 6px 14px 18px 16px;
    gap: 14px;
  }

  .me-checkin-inline {
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    border: 0;
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--ink-2);
    font-size: 11px;
    font-weight: 550;
  }
  .me-checkin-inline.done {
    background: rgba(47, 163, 107, 0.12);
    color: #2fa36b;
  }

  .me-slab-divider {
    margin: 0 16px;
    height: 0.5px;
  }

  .me-quick-grid {
    padding: 14px 10px 16px;
  }
  .me-quick-item {
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 7px;
    padding: 4px 2px;
    border-radius: 12px;
  }
  .me-quick-item + .me-quick-item::before { display: none; }
  .me-quick-ico { width: 42px; height: 42px; border-radius: 14px; }
  .me-quick-copy strong { font-size: 12px; font-weight: 500; color: var(--ink-2); }
  .me-quick-copy small { display: none; }

  .me-groups-wrap {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 10px;
    background: transparent;
    border: 0;
    border-radius: 0;
    box-shadow: none;
  }
  .me-group-card {
    display: block;
    background: var(--surface);
  }
  .me-menu-row {
    min-height: 52px;
    padding: 12px 14px 12px 16px;
  }
  .me-menu-row:nth-child(even)::before,
  .me-group-card:nth-child(2) .me-menu-row::after { display: none; }
  .me-menu-row + .me-menu-row::before {
    content: '';
    display: block;
    position: absolute;
    top: 0;
    left: 47px;
    right: 0;
    height: 0.5px;
    background: var(--line);
  }
  .me-menu-copy strong { font-size: 15px; font-weight: 450; }
  .me-menu-copy small { display: none; }
}
</style>
