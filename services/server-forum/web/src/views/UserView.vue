<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import AvatarEditor from '@/components/AvatarEditor.vue'
import ForumShell from '@/components/ForumShell.vue'
import PagerNav from '@/components/PagerNav.vue'
import TopicList from '@/components/TopicList.vue'
import { confirmDialog } from '@/ui'
import UserAvatar from '@/components/UserAvatar.vue'
import UserConnections from '@/components/UserConnections.vue'
import { api, ApiError } from '@/api/forum'
import type { Profile, UserReply } from '@/api/types'
import { TRUST_LABEL, formatDate, timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { useTopicList } from '@/lib/useList'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'

const route = useRoute()
const router = useRouter()
const session = useSession()
const toast = useToast()
const uid = computed(() => String(route.params.uid))

type Tab = 'topics' | 'replies' | 'following' | 'followers'
const tab = computed<Tab>(() => {
  const value = route.query.tab
  return value === 'replies' || value === 'following' || value === 'followers' ? value : 'topics'
})
function setTab(value: Tab) {
  router.replace({ query: { ...route.query, tab: value === 'topics' ? undefined : value } })
}

const profile = ref<Profile | null>(null)
const missing = ref(false)
const replies = ref<UserReply[]>([])
const followBusy = ref(false)
const blockBusy = ref(false)
const avatarOpen = ref(false)
const moreOpen = ref(false)
let profileSeq = 0

useHead(() => ({ title: profile.value?.nickname ?? '用户' }))

const isMe = computed(() => session.user?.uid === uid.value)
const avatarSrc = computed(() => (isMe.value ? session.user?.avatar_url || profile.value?.avatar_url : profile.value?.avatar_url))

function closeMoreOnOutside(event: MouseEvent) {
  const el = event.target as HTMLElement | null
  if (moreOpen.value && !el?.closest('.u-more-wrap')) moreOpen.value = false
}
onMounted(() => document.addEventListener('click', closeMoreOnOutside))
onBeforeUnmount(() => document.removeEventListener('click', closeMoreOnOutside))

watch(
  [uid, () => session.user?.uid],
  async ([u]) => {
    const mine = ++profileSeq
    profile.value = null
    missing.value = false
    moreOpen.value = false
    try {
      const res = await api.profile(u!)
      if (mine === profileSeq) profile.value = res
    } catch (e) {
      if (mine !== profileSeq) return
      if (e instanceof ApiError && e.status === 404) missing.value = true
      else toast.error(e)
    }
  },
  { immediate: true },
)

async function copyId() {
  if (!session.accountId) return
  try {
    await navigator.clipboard?.writeText(session.accountId)
    toast.show(`已复制账号 ID ${session.accountId}`)
  } catch {
    toast.show(`账号 ID ${session.accountId}`)
  }
}

async function shareProfile() {
  moreOpen.value = false
  const url = `${window.location.origin}/u/${encodeURIComponent(uid.value)}`
  try {
    await navigator.clipboard?.writeText(url)
    toast.show('已复制主页链接')
  } catch {
    toast.show(url)
  }
}

async function toggleBlock() {
  moreOpen.value = false
  if (!session.requireLogin() || blockBusy.value || !profile.value) return
  const current = profile.value
  if (!current.blocked && !await confirmDialog({ title: `屏蔽 ${current.nickname}？`, description: '他的帖子不会再出现在你的列表和搜索里，回复会被折叠，你们会互相取消关注。对方不会收到提示。', confirmLabel: '确认屏蔽', danger: true })) return
  blockBusy.value = true
  try {
    const res = await api.block(current.uid, !current.blocked)
    if (profile.value === current) {
      current.blocked = res.blocked
      if (res.blocked) current.following = false
    }
    toast.show(res.blocked ? '已屏蔽' : '已取消屏蔽')
  } catch (e) {
    toast.error(e)
  } finally {
    blockBusy.value = false
  }
}

async function toggleFollow() {
  if (!session.requireLogin() || followBusy.value || !profile.value) return
  const current = profile.value
  followBusy.value = true
  try {
    const res = await api.follow(current.uid, !current.following)
    if (profile.value === current) Object.assign(current, res)
  } catch (e) {
    toast.error(e)
  } finally {
    followBusy.value = false
  }
}

const topics = useTopicList(p => api.userTopics(uid.value, p), [uid, () => session.user?.uid])

watch(
  [tab, uid, () => session.user?.uid],
  async ([t, u, viewer]) => {
    if (t === 'replies' && !replies.value.length) {
      try {
        const res = await api.userPosts(u)
        if (uid.value === u && session.user?.uid === viewer && tab.value === 'replies') replies.value = res.items
      } catch (e) {
        toast.error(e)
      }
    }
  },
  { immediate: true },
)
watch([uid, () => session.user?.uid], () => (replies.value = []), { flush: 'sync' })
</script>

<template>
  <ForumShell :rail="false">
    <div class="u-page">
      <div v-if="missing" class="u-card u-empty">
        <span class="u-empty-ico"><AppIcon name="me" :size="22" /></span>
        <strong>找不到这个用户</strong>
        <p>登录过超级小伴的用户才有主页。</p>
      </div>

      <template v-else-if="profile">
        <header class="u-card u-hero-card">
          <div class="u-hero-top">
            <button
              v-if="isMe"
              type="button"
              class="u-av-btn"
              aria-label="更换头像"
              @click="avatarOpen = true"
            >
              <UserAvatar :name="profile.nickname" :src="avatarSrc" :size="72" :initial-size="24" />
            </button>
            <UserAvatar v-else :name="profile.nickname" :src="avatarSrc" :size="72" :initial-size="24" />

            <div class="u-who">
              <div class="u-name-row">
                <h1>{{ profile.nickname }}</h1>
                <span v-if="TRUST_LABEL[profile.trust_level]" class="u-chip">{{ TRUST_LABEL[profile.trust_level] }}</span>
                <span v-if="profile.banned" class="u-chip danger">已封禁</span>
              </div>
              <div class="u-meta">
                <button v-if="isMe && session.accountId" type="button" class="u-id-copy" @click="copyId">
                  <span>ID {{ session.accountId }}</span>
                  <AppIcon name="copy" :size="12" />
                </button>
                <span v-if="isMe && session.accountId" class="dot" aria-hidden="true">·</span>
                <span>{{ formatDate(profile.created_at) }} 加入 · {{ timeAgo(profile.last_seen_at) }}活跃</span>
              </div>
            </div>

            <div class="u-actions">
              <template v-if="isMe">
                <button type="button" class="u-pill secondary" @click="avatarOpen = true">换头像</button>
                <RouterLink to="/settings" class="u-pill secondary">设置</RouterLink>
                <button type="button" class="u-icon-btn" aria-label="分享主页" @click="shareProfile">
                  <AppIcon name="share-up" :size="18" />
                </button>
              </template>
              <template v-else>
                <button
                  v-if="profile.blocked"
                  type="button"
                  class="u-pill secondary"
                  :disabled="blockBusy"
                  @click="toggleBlock"
                >已屏蔽</button>
                <button
                  v-else-if="!profile.banned || profile.following"
                  type="button"
                  class="u-pill"
                  :class="profile.following ? 'secondary' : 'primary'"
                  :disabled="followBusy || (session.loggedIn && !session.user)"
                  :aria-pressed="profile.following"
                  @click="toggleFollow"
                >
                  <AppIcon v-if="!profile.following && !followBusy" name="plus" :size="13" />
                  <span>{{ followBusy ? '处理中…' : profile.following ? '已关注' : '关注' }}</span>
                </button>
                <div class="u-more-wrap">
                  <button type="button" class="u-icon-btn" aria-label="更多操作" @click.stop="moreOpen = !moreOpen">
                    <AppIcon name="more" :size="18" />
                  </button>
                  <div v-if="moreOpen" class="u-pop-menu" role="menu">
                    <button type="button" class="u-pop-item" role="menuitem" @click="shareProfile">
                      <AppIcon name="share-up" :size="15" />
                      <span>分享主页</span>
                    </button>
                    <button
                      type="button"
                      class="u-pop-item"
                      :class="{ danger: !profile.blocked }"
                      role="menuitem"
                      :disabled="blockBusy"
                      @click="toggleBlock"
                    >
                      <AppIcon name="block" :size="15" />
                      <span>{{ profile.blocked ? '取消屏蔽' : '屏蔽该用户' }}</span>
                    </button>
                  </div>
                </div>
              </template>
            </div>
          </div>

          <div v-if="!isMe && profile.blocked" class="u-blocked-bar">
            <span>已屏蔽对方，互相取消关注</span>
            <button type="button" :disabled="blockBusy" @click="toggleBlock">取消屏蔽</button>
          </div>

          <dl class="u-stats">
            <button type="button" class="u-stat" @click="setTab('topics')">
              <dd>{{ profile.topic_count }}</dd>
              <dt>帖子</dt>
            </button>
            <button type="button" class="u-stat" @click="setTab('replies')">
              <dd>{{ profile.post_count }}</dd>
              <dt>回复</dt>
            </button>
            <button type="button" class="u-stat" @click="setTab('following')">
              <dd>{{ profile.following_count }}</dd>
              <dt>关注</dt>
            </button>
            <button type="button" class="u-stat" @click="setTab('followers')">
              <dd>{{ profile.follower_count }}</dd>
              <dt>粉丝</dt>
            </button>
          </dl>
        </header>

        <section class="u-card u-body-card">
          <div class="u-tabs" role="tablist">
            <button type="button" role="tab" :aria-selected="tab === 'topics'" :class="{ on: tab === 'topics' }" @click="setTab('topics')">
              <span>帖子</span><small>{{ profile.topic_count }}</small>
            </button>
            <button type="button" role="tab" :aria-selected="tab === 'replies'" :class="{ on: tab === 'replies' }" @click="setTab('replies')">
              <span>回复</span><small>{{ profile.post_count }}</small>
            </button>
            <button type="button" role="tab" :aria-selected="tab === 'following'" :class="{ on: tab === 'following' }" @click="setTab('following')">
              <span>关注</span><small>{{ profile.following_count }}</small>
            </button>
            <button type="button" role="tab" :aria-selected="tab === 'followers'" :class="{ on: tab === 'followers' }" @click="setTab('followers')">
              <span>粉丝</span><small>{{ profile.follower_count }}</small>
            </button>
          </div>

          <div v-if="tab === 'topics' && !isMe && profile.blocked" class="u-empty">
            <span class="u-empty-ico"><AppIcon name="block" :size="22" /></span>
            <strong>你已屏蔽该用户</strong>
            <p>对方的帖子与回复已对你隐藏，取消屏蔽后即可恢复查看。</p>
            <button type="button" class="u-pill secondary" :disabled="blockBusy" @click="toggleBlock">取消屏蔽</button>
          </div>
          <template v-else-if="tab === 'topics'">
            <TopicList :items="topics.items.value" :loading="topics.loading.value" empty="还没有发布过帖子。" />
            <PagerNav :page="topics.page.value" :page-size="topics.pageSize.value" :total="topics.total.value" @go="topics.go" />
          </template>
          <UserConnections v-else-if="tab === 'following' || tab === 'followers'" :uid="uid" :mode="tab" />
          <div v-else class="u-replies">
            <RouterLink v-for="r in replies" :key="r.id" :to="`/t/${r.topic_id}#floor-${r.floor}`" class="u-reply">
              <span class="rt">回复了「{{ r.topic_title }}」</span>
              <span class="rx">{{ r.excerpt }}</span>
              <span class="rd">{{ timeAgo(r.created_at) }}</span>
            </RouterLink>
            <div v-if="!replies.length" class="u-empty">
              <span class="u-empty-ico"><AppIcon name="message" :size="22" /></span>
              <strong>还没有回复记录</strong>
              <p>参与社区讨论后，你的回复会展示在这里。</p>
            </div>
          </div>
        </section>
      </template>
    </div>

    <AvatarEditor
      v-if="isMe"
      v-model:open="avatarOpen"
      :name="profile?.nickname || session.user?.nickname || '?'"
      :current="avatarSrc ?? null"
    />
  </ForumShell>
</template>

<style scoped>
.u-page {
  max-width: 820px;
  margin: 0 auto;
  display: grid;
  gap: 14px;
}

.u-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: var(--shadow-card);
  overflow: visible;
}
.u-body-card { overflow: hidden; }

.u-hero-top {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 22px 16px;
}
.u-av-btn {
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: transparent;
  flex: none;
  cursor: pointer;
}
.u-who {
  flex: 1;
  min-width: 0;
}
.u-name-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}
.u-name-row h1 {
  margin: 0;
  font-size: 22px;
  line-height: 28px;
  font-weight: 700;
  color: var(--ink);
  overflow-wrap: anywhere;
}
.u-chip {
  display: inline-flex;
  align-items: center;
  height: 20px;
  padding: 0 8px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--ink-2);
  font-size: 11px;
  font-weight: 550;
}
.u-chip.danger {
  background: var(--brand-soft);
  color: var(--danger, #e3203c);
}

.u-meta {
  margin-top: 6px;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px 8px;
  font-size: 12.5px;
  color: var(--ink-3);
}
.u-id-copy {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--ink-3);
  font-size: 12.5px;
}
.u-id-copy:hover { color: var(--ink-2); }

.u-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
}

.u-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 32px;
  padding: 0 14px;
  border: 0;
  border-radius: 999px;
  font-size: 12.5px;
  font-weight: 600;
  white-space: nowrap;
  transition: opacity 0.14s, background-color 0.14s;
}
.u-pill:hover:not(:disabled) { opacity: 0.88; }
.u-pill:disabled { opacity: 0.45; cursor: default; }
.u-pill.primary { background: var(--brand-soft); color: var(--brand); }
.u-pill.secondary { background: var(--surface-2); color: var(--ink-2); font-weight: 500; }

.u-icon-btn {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--ink-2);
  display: grid;
  place-items: center;
}
.u-icon-btn:hover { background: var(--surface-2); color: var(--ink); }

.u-more-wrap { position: relative; }
.u-pop-menu {
  position: absolute;
  right: 0;
  top: 40px;
  min-width: 146px;
  padding: 5px;
  border-radius: 12px;
  border: 1px solid var(--line);
  background: var(--surface);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.14);
  z-index: 30;
}
.u-pop-item {
  width: 100%;
  height: 36px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  text-align: left;
}
.u-pop-item:hover { background: var(--surface-2); }
.u-pop-item.danger { color: var(--danger, #e3203c); }

.u-blocked-bar {
  margin: 0 20px 14px;
  padding: 10px 12px;
  border-radius: 12px;
  background: var(--brand-soft);
  color: var(--danger, #e3203c);
  font-size: 12.5px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.u-blocked-bar button {
  height: 28px;
  padding: 0 11px;
  border: 0;
  border-radius: 999px;
  background: var(--surface);
  color: var(--danger, #e3203c);
  font-size: 12px;
  font-weight: 600;
}

.u-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin: 0;
  padding: 10px 12px;
  border-top: 1px solid var(--line);
}
.u-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 5px 4px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  transition: background-color 0.13s;
}
.u-stat:hover { background: var(--surface-2); }
.u-stat dd {
  margin: 0;
  font-size: 17px;
  line-height: 22px;
  font-weight: 700;
  color: var(--ink);
  font-variant-numeric: tabular-nums;
}
.u-stat dt {
  font-size: 12px;
  color: var(--ink-3);
}

.u-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 12px;
  border-bottom: 1px solid var(--line);
}
.u-tabs button {
  position: relative;
  height: 44px;
  padding: 0 12px;
  border: 0;
  background: transparent;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 14.5px;
  font-weight: 500;
  color: var(--ink-2);
}
.u-tabs button small {
  font-size: 12px;
  color: var(--ink-3);
  font-variant-numeric: tabular-nums;
}
.u-tabs button.on {
  color: var(--ink);
  font-weight: 650;
}
.u-tabs button.on::after {
  content: '';
  position: absolute;
  left: 50%;
  bottom: 0;
  transform: translateX(-50%);
  width: 20px;
  height: 2.5px;
  border-radius: 999px;
  background: var(--brand);
}

.u-replies > * + * {
  border-top: 1px solid var(--line);
}
.u-reply {
  display: grid;
  gap: 6px;
  padding: 14px 18px;
  transition: background-color 0.12s;
}
.u-reply:hover { background: var(--surface-2); }
.rt { font-size: 13.5px; font-weight: 550; color: var(--ink-2); }
.rx {
  padding: 7px 11px;
  border-radius: 8px;
  background: var(--surface-2);
  font-size: 13.5px;
  line-height: 19px;
  color: var(--ink);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.rd { font-size: 12px; color: var(--ink-3); }

.u-empty {
  padding: 52px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 9px;
}
.u-empty-ico {
  width: 48px;
  height: 48px;
  border-radius: 15px;
  background: var(--surface-2);
  color: var(--ink-3);
  display: grid;
  place-items: center;
}
.u-empty strong { font-size: 15px; font-weight: 600; color: var(--ink); }
.u-empty p { margin: 0; font-size: 13px; color: var(--ink-3); max-width: 260px; }

@media (max-width: 767px) {
  .u-page { gap: 10px; }
  .u-card { border-radius: 0; border-left: 0; border-right: 0; box-shadow: none; }
  .u-hero-top { flex-wrap: wrap; padding: 14px 16px; gap: 12px; }
  .u-actions { width: 100%; justify-content: flex-end; }
  .u-tabs { padding: 0 8px; }
  .u-tabs button { padding: 0 10px; font-size: 14px; }
}
</style>
