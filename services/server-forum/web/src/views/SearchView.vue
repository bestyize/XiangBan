<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import type { TopicItem } from '@/api/types'
import { useToast } from '@/stores/toast'
import { useHead } from '@/lib/head'

const route = useRoute()
const router = useRouter()
const toast = useToast()

const q = computed(() => (typeof route.query.q === 'string' ? route.query.q.trim() : ''))
const input = ref(q.value)
useHead(() => ({ title: q.value ? `搜索 ${q.value}` : '搜索' }))

const items = ref<TopicItem[]>([])
const total = ref(0)
const page = ref(1)
const loading = ref(false)

async function load(p = 1) {
  if (!q.value) {
    items.value = []
    total.value = 0
    return
  }
  loading.value = true
  try {
    const res = await api.search({ q: q.value, page: p })
    items.value = res.items
    total.value = res.total
    page.value = res.page
  } catch (e) {
    toast.error(e)
  } finally {
    loading.value = false
  }
}

watch(
  q,
  v => {
    input.value = v
    load(1)
  },
  { immediate: true },
)

function submit() {
  const v = input.value.trim()
  if (v) router.replace({ query: { q: v } })
}
</script>

<template>
  <ForumShell :rail="false">
    <form class="search" role="search" @submit.prevent="submit">
      <label for="search-page" class="sr-only">搜索</label>
      <input id="search-page" v-model="input" class="input" type="search" maxlength="50" placeholder="搜帖子或内容" />
      <button class="btn btn-primary" type="submit">搜索</button>
    </form>

    <template v-if="q">
      <p class="summary muted">找到 {{ total }} 个和"{{ q }}"有关的帖子</p>
      <TopicList :items="items" :loading="loading" empty="没有找到，换个关键词试试，比如「免费额度」。" />
      <PagerNav :page="page" :page-size="30" :total="total" @go="load" />
    </template>
    <p v-else class="muted hint">输入任何关键词，搜帖子的标题和内容。</p>
  </ForumShell>
</template>

<style scoped>
.search {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}
.search .input {
  font-size: var(--fs-lg);
}
.search .btn {
  height: auto;
  padding: 0 20px;
}
.summary {
  font-size: var(--fs-sm);
  margin: 0 0 10px;
}
.hint {
  text-align: center;
  padding: 40px 0;
}
</style>
