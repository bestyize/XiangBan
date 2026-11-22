<script setup lang="ts">
import { ref, watch } from 'vue'
import { api } from '@/api/forum'
import type { PublicUser } from '@/api/types'
import { TRUST_LABEL } from '@/lib/format'
import { useToast } from '@/stores/toast'
import AppIcon from './AppIcon.vue'
import PagerNav from './PagerNav.vue'
import UserAvatar from './UserAvatar.vue'

const props = defineProps<{ uid: string; mode: 'following' | 'followers' }>()
const toast = useToast()
const items = ref<PublicUser[]>([])
const page = ref(1)
const pageSize = ref(30)
const total = ref(0)
const loading = ref(false)
const failed = ref(false)
let seq = 0

async function load(p = 1) {
  const mine = ++seq
  loading.value = true
  failed.value = false
  try {
    const res = await api[props.mode](props.uid, p)
    if (mine !== seq) return
    items.value = res.items
    page.value = res.page
    pageSize.value = res.page_size
    total.value = res.total
  } catch (e) {
    if (mine === seq) {
      failed.value = true
      toast.error(e)
    }
  } finally {
    if (mine === seq) loading.value = false
  }
}

watch(
  () => [props.uid, props.mode],
  () => {
    items.value = []
    total.value = 0
    page.value = 1
    load()
  },
  { immediate: true },
)
</script>

<template>
  <div class="connections" :aria-busy="loading">
    <p v-if="loading" class="muted empty">加载中…</p>
    <div v-else-if="failed" class="empty">
      <button class="btn btn-sm" type="button" @click="load(page)">重新加载</button>
    </div>
    <template v-else-if="items.length">
      <div class="people-grid">
        <RouterLink v-for="user in items" :key="user.uid" :to="`/u/${user.uid}`" class="person">
          <UserAvatar :name="user.nickname" :src="user.avatar_url" :size="40" />
          <div class="who">
            <strong>{{ user.nickname }}</strong>
            <span class="muted">{{ TRUST_LABEL[user.trust_level] }}</span>
          </div>
          <AppIcon name="arrow" :size="14" class="chev" />
        </RouterLink>
      </div>
    </template>
    <div v-else class="empty-box">
      <span class="empty-ico"><AppIcon name="users" :size="22" /></span>
      <strong>{{ mode === 'following' ? '还没有关注用户' : '还没有粉丝' }}</strong>
      <p>去社区发现有趣的创作者和讨论吧。</p>
    </div>
  </div>
  <PagerNav :page="page" :page-size="pageSize" :total="total" @go="load" />
</template>

<style scoped>
.people-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  padding: 14px 16px 18px;
}
.person {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid var(--line);
  background: var(--bg, var(--surface));
  transition: background-color 0.14s, border-color 0.14s;
}
.person:hover {
  background: var(--surface-2);
}
.who {
  display: grid;
  gap: 2px;
  min-width: 0;
  flex: 1;
}
.who strong {
  overflow-wrap: anywhere;
  font-size: 14.5px;
  font-weight: 600;
  color: var(--ink);
}
.who span { font-size: 12px; }
.chev { color: var(--ink-3); opacity: 0.7; flex: none; }
.empty { padding: 36px 16px; text-align: center; }
.empty-box {
  padding: 48px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 8px;
}
.empty-ico {
  width: 48px;
  height: 48px;
  border-radius: 15px;
  background: var(--surface-2);
  color: var(--ink-3);
  display: grid;
  place-items: center;
}
.empty-box strong { font-size: 15px; font-weight: 600; color: var(--ink); }
.empty-box p { margin: 0; font-size: 13px; color: var(--ink-3); }

@media (max-width: 767px) {
  .people-grid {
    grid-template-columns: 1fr;
    gap: 0;
    padding: 0;
  }
  .person {
    border: 0;
    border-radius: 0;
    background: var(--surface);
    padding: 12px 16px;
  }
  .person + .person {
    border-top: 0.5px solid var(--line);
  }
}
</style>
