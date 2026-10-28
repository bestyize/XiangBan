<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import PackCard from '@/components/PackCard.vue'
import PagerNav from '@/components/PagerNav.vue'
import { contentApi, type PackItem, type SchemeItem } from '@/api/content'
import { PACK_KINDS, VISUAL_KINDS } from '@/lib/content'
import { timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '资源' }))

const route = useRoute()
const router = useRouter()
const toast = useToast()

const TABS = [{ key: '', label: '全部' }, ...PACK_KINDS.filter(k => k.key !== 'scheme'), { key: 'schemes', label: '方案' }]
const tab = computed(() => {
  const k = String(route.query.kind ?? '')
  return TABS.some(t => t.key === k) ? k : ''
})
const sort = computed<'new' | 'hot'>(() => (route.query.sort === 'hot' ? 'hot' : 'new'))
const wide = computed(() => tab.value !== '' && !VISUAL_KINDS.includes(tab.value))

const packs = ref<PackItem[]>([])
const schemes = ref<SchemeItem[]>([])
const page = ref(1)
const total = ref(0)
const pageSize = ref(20)
const loading = ref(false)
let seq = 0

async function load(p = 1) {
  const mine = ++seq
  loading.value = true
  try {
    if (tab.value === 'schemes') {
      const res = await contentApi.schemes(p)
      if (mine !== seq) return
      schemes.value = res.items
      packs.value = []
      ;({ page: page.value, total: total.value, page_size: pageSize.value } = res)
    } else {
      const res = await contentApi.packs({ kind: tab.value || undefined, sort: sort.value, page: p })
      if (mine !== seq) return
      packs.value = res.items
      schemes.value = []
      ;({ page: page.value, total: total.value, page_size: pageSize.value } = res)
    }
  } catch (e) {
    if (mine === seq) toast.error(e)
  } finally {
    if (mine === seq) loading.value = false
  }
}

watch([tab, sort], () => load(1), { immediate: true })

function setTab(key: string) {
  router.replace({ query: { ...(key ? { kind: key } : {}), ...(sort.value === 'hot' ? { sort: 'hot' } : {}) } })
}
function setSort(s: 'new' | 'hot') {
  router.replace({ query: { ...(tab.value ? { kind: tab.value } : {}), ...(s === 'hot' ? { sort: 'hot' } : {}) } })
}
function go(p: number) {
  load(p)
  window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>

<template>
  <ContentShell>
    <h1 class="sr-only">资源</h1>
    <div class="subtabs" role="tablist" aria-label="内容类型">
      <button v-for="t in TABS" :key="t.key" type="button" role="tab" :aria-selected="tab === t.key" :class="{ on: tab === t.key }" @click="setTab(t.key)">
        {{ t.label }}
      </button>
    </div>
    <div v-if="tab !== 'schemes'" class="sortbar">
      <button type="button" :class="{ on: sort === 'new' }" @click="setSort('new')">最新</button>
      <button type="button" :class="{ on: sort === 'hot' }" @click="setSort('hot')">最多使用</button>
    </div>

    <p v-if="loading && !packs.length && !schemes.length" class="state">加载中</p>
    <p v-else-if="!loading && !packs.length && !schemes.length" class="state">这里还没有内容。</p>

    <div v-if="packs.length" :class="wide ? 'list' : 'grid'">
      <PackCard v-for="p in packs" :key="p.id" :pack="p" :wide="wide" />
    </div>

    <div v-if="schemes.length" class="list">
      <article v-for="s in schemes" :key="s.id" class="scheme">
        <h3>{{ s.title }}</h3>
        <p class="meta">
          <span>{{ s.author.nickname }}</span>
          <span>{{ s.items.length }} 项内容</span>
          <span>{{ timeAgo(s.created_at) }}</span>
          <span v-if="s.apply_count">{{ s.apply_count }} 次复刻</span>
        </p>
        <p class="chips">
          <RouterLink v-for="it in s.items" :key="it.pack_id" :to="`/packs/${it.pack_id}`">{{ it.slug }}</RouterLink>
        </p>
      </article>
    </div>

    <PagerNav :page="page" :page-size="pageSize" :total="total" @go="go" />
  </ContentShell>
</template>

<style scoped>
.subtabs {
  display: flex;
  gap: 18px;
  padding: 10px 16px 0;
  overflow-x: auto;
  scrollbar-width: none;
}
.subtabs::-webkit-scrollbar {
  display: none;
}
.subtabs button {
  flex: none;
  border: 0;
  background: none;
  padding: 4px 0 8px;
  font-size: var(--fs-sm);
  color: var(--ink-3);
  border-bottom: 2px solid transparent;
}
.subtabs button.on {
  color: var(--ink);
  font-weight: 600;
  border-bottom-color: var(--brand);
}
.sortbar {
  display: flex;
  gap: 14px;
  padding: 10px 16px 4px;
  font-size: var(--fs-xs);
}
.sortbar button {
  border: 0;
  background: none;
  padding: 0;
  color: var(--ink-3);
}
.sortbar button.on {
  color: var(--ink);
  font-weight: 600;
}
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px 12px;
  padding: 8px 16px;
}
.list {
  padding: 0 16px;
}
.scheme {
  padding: 12px 0;
  border-bottom: 1px solid var(--line);
}
.scheme h3 {
  margin: 0;
  font-size: var(--fs-md);
  font-weight: 500;
}
.meta {
  margin: 4px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 10px;
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
.chips {
  margin: 6px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
@media (min-width: 768px) {
  .grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}
</style>
