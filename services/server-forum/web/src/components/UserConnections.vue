<script setup lang="ts">
import { ref, watch } from 'vue'
import { api } from '@/api/forum'
import type { PublicUser } from '@/api/types'
import { useToast } from '@/stores/toast'
import { TRUST_LABEL } from '@/lib/format'
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

watch(() => [props.uid, props.mode], () => {
  items.value = []
  total.value = 0
  page.value = 1
  load()
}, { immediate: true })
</script>

<template>
  <div class="panel connections" :aria-busy="loading">
    <p v-if="loading" class="muted empty">加载中…</p>
    <div v-else-if="failed" class="empty">
      <button class="btn btn-sm" type="button" @click="load(page)">重新加载</button>
    </div>
    <template v-else>
      <RouterLink v-for="user in items" :key="user.uid" :to="`/u/${user.uid}`" class="person">
        <UserAvatar :name="user.nickname" :src="user.avatar_url" :size="40" />
        <div class="who">
          <strong>{{ user.nickname }}</strong>
          <span class="muted">{{ TRUST_LABEL[user.trust_level] }}</span>
        </div>
        <span class="muted visit">查看主页</span>
      </RouterLink>
      <p v-if="!items.length" class="muted empty">{{ mode === 'following' ? '还没有关注任何人。' : '还没有粉丝。' }}</p>
    </template>
  </div>
  <PagerNav :page="page" :page-size="pageSize" :total="total" @go="load" />
</template>

<style scoped>
.person { display: flex; align-items: center; gap: 12px; padding: 14px 16px; }
.person + .person { border-top: 1px solid var(--line); }
.person:hover { background: var(--surface-2); }
.who { display: grid; gap: 2px; min-width: 0; }
.who strong { overflow-wrap: anywhere; font-size: var(--fs-sm); }
.who span, .visit { font-size: var(--fs-xs); }
.visit { margin-left: auto; white-space: nowrap; }
.empty { padding: 32px 16px; text-align: center; }
</style>
