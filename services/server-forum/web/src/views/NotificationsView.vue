<script setup lang="ts">
import { onMounted, ref } from 'vue'
import PagerNav from '@/components/PagerNav.vue'
import ContentShell from '@/components/ContentShell.vue'
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

function text(n: NotificationItem): string {
  const who = n.actor?.nickname ?? '有人'
  switch (n.kind) {
    case 'reply': return `${who} 回复了你的帖子「${n.title}」`
    case 'reply_to': return `${who} 回复了你在「${n.title}」里的回复`
    case 'mention': return `${who} 在「${n.title}」里提到了你`
    case 'accepted': return `你在「${n.title}」里的回答被采纳了`
    case 'pack_approved': return `你的内容包「${n.title}」通过了审核`
    case 'pack_rejected': return `你的内容包「${n.title}」没有通过审核${n.excerpt ? `：${n.excerpt}` : ''}`
  }
}

function target(n: NotificationItem): string {
  if (n.pack_id) return `/packs/${n.pack_id}`
  return n.topic_id ? `/t/${n.topic_id}` : '/'
}

async function load(p = 1) {
  loading.value = true
  try {
    const res = await api.notifications(p)
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

onMounted(() => load())
</script>

<template>
  <ContentShell>
    <h1 class="title">通知</h1>
    <p v-if="loading && !items.length" class="state">加载中</p>
    <p v-else-if="!items.length" class="state">还没有通知。</p>
    <ul v-else class="list">
      <li v-for="n in items" :key="n.id" :class="{ unread: !n.read }">
        <RouterLink :to="target(n)">
          <span>{{ text(n) }}</span>
          <small v-if="n.excerpt && n.kind !== 'pack_rejected'">{{ n.excerpt }}</small>
          <time>{{ timeAgo(n.created_at) }}</time>
        </RouterLink>
      </li>
    </ul>
    <PagerNav :page="page" :page-size="30" :total="total" @go="load" />
  </ContentShell>
</template>

<style scoped>
.title {
  margin: 0;
  padding: 16px 16px 8px;
  font-size: var(--fs-xl);
  font-weight: 600;
}
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.list {
  list-style: none;
  margin: 0;
  padding: 0 16px;
}
.list li {
  border-bottom: 1px solid var(--line);
}
.list a {
  display: grid;
  gap: 2px;
  padding: 12px 0 12px 14px;
  position: relative;
}
.list li.unread a::before {
  content: '';
  position: absolute;
  left: 0;
  top: 19px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--brand);
}
small {
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
time {
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
</style>
