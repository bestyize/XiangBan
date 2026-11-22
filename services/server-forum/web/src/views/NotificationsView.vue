<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppIcon from '@/components/AppIcon.vue'
import ContentShell from '@/components/ContentShell.vue'
import PagerNav from '@/components/PagerNav.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { api } from '@/api/forum'
import type { NotificationItem } from '@/api/types'
import { timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { useNotifications } from '@/stores/notifications'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '通知' }))
const toast = useToast()
const inbox = useNotifications()
const items = ref<NotificationItem[]>([])
const page = ref(1)
const total = ref(0)
const loading = ref(true)
const tab = ref<'all' | 'unread'>('all')
const markingAll = ref(false)

const hasUnread = computed(() => inbox.unread > 0 || items.value.some(n => !n.read))

function text(n: NotificationItem): string {
  const who = n.actor?.nickname ?? '有人'
  switch (n.kind) {
    case 'reply': return n.pack_id ? `${who} 评论了你的作品「${n.title}」` : `${who} 回复了你的帖子「${n.title}」`
    case 'reply_to': return `${who} 回复了你在「${n.title}」里的回复`
    case 'mention': return `${who} 在「${n.title}」里提到了你`
    case 'accepted': return `你在「${n.title}」里的回答被采纳了`
    case 'content_removed': return '你的社区内容已由管理员下架'
    case 'pack_approved': return `你的内容包「${n.title}」通过了审核`
    case 'pack_rejected': return `你的内容包「${n.title}」没有通过审核${n.excerpt ? `：${n.excerpt}` : ''}`
    default: return n.title || '社区消息'
  }
}

function tagOf(kind: NotificationItem['kind']): string {
  switch (kind) {
    case 'reply':
    case 'reply_to': return '回复'
    case 'mention': return '提及'
    case 'accepted': return '采纳'
    case 'pack_approved':
    case 'pack_rejected':
    case 'content_removed': return '审核通知'
    default: return '通知'
  }
}

function target(n: NotificationItem): string {
  if (n.kind === 'content_removed' && !n.pack_id) return '/me/works?tab=schemes'
  if (n.pack_id) return `/packs/${n.pack_id}`
  return n.topic_id ? `/t/${n.topic_id}` : '/'
}

async function load(p = 1) {
  loading.value = true
  try {
    const res = await api.notifications(p, tab.value === 'unread')
    items.value = res.items
    page.value = res.page
    total.value = res.total
    // 打开列表就算看过了：未读的小圆点在这一页里还保留，刷新或翻页后消失。
    if (res.items.some(n => !n.read)) {
      await api.markRead(res.items.filter(n => !n.read).map(n => n.id))
      await inbox.refresh()
    }
  } catch (e) {
    toast.error(e)
  } finally {
    loading.value = false
  }
}

function switchTab(next: 'all' | 'unread') {
  if (tab.value === next) return
  tab.value = next
  void load(1)
}

async function markAllRead() {
  if (markingAll.value || !hasUnread.value) return
  markingAll.value = true
  try {
    await api.markRead()
    items.value = items.value.map(item => ({ ...item, read: true }))
    if (tab.value === 'unread') items.value = []
    await inbox.refresh()
    toast.show('已全部标记为已读')
  } catch (e) {
    toast.error(e)
  } finally {
    markingAll.value = false
  }
}

onMounted(() => void load())
</script>

<template>
  <ContentShell>
    <div class="n-page">
      <header class="n-head">
        <h1>通知</h1>
        <button type="button" class="n-pill" :disabled="markingAll || !hasUnread" @click="markAllRead">
          {{ markingAll ? '处理中…' : '全部已读' }}
        </button>
      </header>

      <section class="n-card">
        <div class="n-tabs" role="tablist">
          <button type="button" role="tab" :aria-selected="tab === 'all'" :class="{ active: tab === 'all' }" @click="switchTab('all')">
            全部
          </button>
          <button type="button" role="tab" :aria-selected="tab === 'unread'" :class="{ active: tab === 'unread' }" @click="switchTab('unread')">
            <span>未读</span>
            <small v-if="inbox.unread > 0">{{ inbox.unread }}</small>
          </button>
        </div>

        <p v-if="loading && !items.length" class="n-state">加载中…</p>
        <div v-else-if="!items.length" class="n-empty">
          <span class="n-empty-ico"><AppIcon name="bell" :size="22" /></span>
          <strong>{{ tab === 'unread' ? '已全部读完' : '还没有收到通知' }}</strong>
          <p>帖子的回复、作品点赞与审核结果都会出现在这里。</p>
        </div>
        <ul v-else class="n-list">
          <li v-for="n in items" :key="n.id" :class="{ unread: !n.read }">
            <RouterLink :to="target(n)" class="n-row">
              <span class="n-av-wrap">
                <UserAvatar v-if="n.actor" :name="n.actor.nickname || '用户'" :src="n.actor.avatar_url" :size="40" />
                <span v-else class="n-sys-ico" :class="n.kind === 'pack_approved' ? 'ok' : 'warn'">
                  <AppIcon :name="n.kind === 'pack_approved' ? 'check' : 'bell'" :size="18" />
                </span>
                <i v-if="!n.read" class="n-dot" aria-label="未读" />
              </span>
              <span class="n-body">
                <span class="n-headline">{{ text(n) }}</span>
                <span v-if="n.excerpt && n.kind !== 'pack_rejected' && n.kind !== 'content_removed'" class="n-quote">{{ n.excerpt }}</span>
                <span class="n-foot">
                  <time>{{ timeAgo(n.created_at) }}</time>
                  <span aria-hidden="true">·</span>
                  <span>{{ tagOf(n.kind) }}</span>
                </span>
              </span>
              <AppIcon name="arrow" :size="14" class="n-chev" />
            </RouterLink>
          </li>
        </ul>
      </section>

      <PagerNav :page="page" :page-size="30" :total="total" @go="load" />
    </div>
  </ContentShell>
</template>

<style scoped>
.n-page {
  max-width: 820px;
  margin: 0 auto;
  padding: 6px 0 28px;
}

.n-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}
.n-head h1 {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 700;
}

.n-pill {
  height: 32px;
  padding: 0 13px;
  border: 0;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--ink-2);
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
}
.n-pill:hover:not(:disabled) { opacity: 0.85; }
.n-pill:disabled { opacity: 0.42; cursor: default; }

.n-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.n-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 12px;
  border-bottom: 1px solid var(--line);
}
.n-tabs button {
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
.n-tabs button small { font-size: 12px; color: var(--ink-3); }
.n-tabs button.active { color: var(--ink); font-weight: 650; }
.n-tabs button.active::after {
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

.n-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.n-list li { position: relative; }
.n-list li + li::before {
  content: '';
  position: absolute;
  top: 0;
  left: 70px;
  right: 16px;
  height: 1px;
  background: var(--line);
}

.n-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px 14px 18px;
  transition: background-color 0.12s;
}
.n-row:hover { background: var(--surface-2); }

.n-av-wrap {
  position: relative;
  flex: none;
  margin-top: 1px;
}
.n-sys-ico {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: grid;
  place-items: center;
}
.n-sys-ico.ok { background: rgba(47, 163, 107, 0.12); color: #2fa36b; }
.n-sys-ico.warn { background: var(--brand-soft); color: var(--danger, #e3203c); }

.n-dot {
  position: absolute;
  top: -1px;
  right: -1px;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--danger, #e3203c);
  box-shadow: 0 0 0 2px var(--surface);
}

.n-body {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 6px;
}
.n-headline {
  font-size: 14.5px;
  line-height: 21px;
  color: var(--ink-2);
}
.unread .n-headline {
  color: var(--ink);
  font-weight: 600;
}
.n-quote {
  padding: 7px 11px;
  border-radius: 8px;
  background: var(--surface-2);
  font-size: 13px;
  line-height: 19px;
  color: var(--ink-2);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.n-foot {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--ink-3);
}
.n-chev {
  color: var(--ink-3);
  opacity: 0.65;
  margin-top: 12px;
  flex: none;
}

.n-state { padding: 48px 16px; text-align: center; color: var(--ink-3); }
.n-empty {
  padding: 58px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 9px;
}
.n-empty-ico {
  width: 48px;
  height: 48px;
  border-radius: 15px;
  background: var(--surface-2);
  color: var(--ink-3);
  display: grid;
  place-items: center;
}
.n-empty strong { font-size: 15px; font-weight: 600; color: var(--ink); }
.n-empty p { margin: 0; font-size: 13px; color: var(--ink-3); max-width: 260px; }

@media (max-width: 767px) {
  .n-head { padding: 4px 12px 0; margin-bottom: 10px; }
  .n-card { border-radius: 0; border-left: 0; border-right: 0; box-shadow: none; }
  .n-row { padding: 13px 12px 13px 16px; }
  .n-list li + li::before { left: 68px; right: 0; }
}
</style>
