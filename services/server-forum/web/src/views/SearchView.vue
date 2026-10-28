<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import PackCard from '@/components/PackCard.vue'
import { api } from '@/api/forum'
import { contentApi, type PackItem } from '@/api/content'
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
const packs = ref<PackItem[]>([])
const packTotal = ref(0)
const total = ref(0)
const page = ref(1)
const loading = ref(false)

async function load(p = 1) {
  if (!q.value) {
    items.value = []
    packs.value = []
    total.value = 0
    packTotal.value = 0
    return
  }
  loading.value = true
  try {
    // 内容包只在第一页搜，翻页只翻帖子；内容包搜索失败不影响帖子结果。
    const [res, found] = await Promise.all([
      api.search({ q: q.value, page: p }),
      p === 1 ? contentApi.packs({ q: q.value, page_size: 8 }).catch(() => null) : Promise.resolve(null),
    ])
    if (p === 1) {
      packs.value = found?.items ?? []
      packTotal.value = found?.total ?? 0
    }
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
      <section v-if="packs.length" class="packs">
        <p class="summary muted">内容 {{ packTotal }} 个</p>
        <div class="pack-grid">
          <PackCard v-for="p in packs" :key="p.id" :pack="p" />
        </div>
      </section>
      <p class="summary muted">找到 {{ total }} 个和"{{ q }}"有关的帖子</p>
      <TopicList :items="items" :loading="loading" empty="没有找到，换个关键词试试。" />
      <PagerNav :page="page" :page-size="30" :total="total" @go="load" />
    </template>
    <p v-else class="muted hint">输入任何关键词，搜帖子和内容包。</p>
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
.packs {
  margin-bottom: 16px;
}
.pack-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px 12px;
}
@media (min-width: 768px) {
  .pack-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
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
