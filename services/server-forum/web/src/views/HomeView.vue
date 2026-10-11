<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import RightRail from '@/components/RightRail.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import type { TopicItem } from '@/api/types'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'

useHead(() => ({ title: '' }))

const route = useRoute()
const router = useRouter()

const TABS = [
  { key: 'latest', label: '最新回复' },
  { key: 'new', label: '新帖' },
  { key: 'hot', label: '热门' },
  { key: 'featured', label: '精选' },
] as const
type Tab = (typeof TABS)[number]['key']

const tab = computed<Tab>(() => {
  const t = route.query.tab
  return TABS.some(x => x.key === t) ? (t as Tab) : 'latest'
})

const fresh = ref<TopicItem[]>([])
const activeDeals = ref(0)

onMounted(async () => {
  try {
    const res = await api.deals({ sort: 'verified', deal_status: 'valid', page_size: 4 })
    fresh.value = res.items
    activeDeals.value = res.total
  } catch {
    // 「现在能领」加载失败不影响下面的列表
  }
})

const list = useTopicList(
  p =>
    api.topics({
      home: true,
      sort: tab.value === 'featured' ? 'new' : tab.value,
      featured: tab.value === 'featured',
      page: p,
    }),
  [tab],
)

function setTab(t: Tab) {
  router.replace({ query: t === 'latest' ? {} : { tab: t } })
}
</script>

<template>
  <ForumShell active="home">
    <section class="fresh" aria-labelledby="fresh-title">
      <div class="fresh-head">
        <h1 id="fresh-title">现在能领</h1>
        <p class="muted">
          {{ activeDeals ? `${activeDeals} 条羊毛在领，领不到的会被标出来` : '每条羊毛都有人复查，领不到的会被标出来' }}
        </p>
        <RouterLink to="/deals" class="btn btn-sm">去羊毛库</RouterLink>
      </div>
      <TopicList v-if="fresh.length" :items="fresh" />
      <div v-else class="fresh-empty panel">
        <p>还没有人分享羊毛。知道哪家在送额度？</p>
        <RouterLink to="/new?category=deals-llm-api" class="btn btn-primary btn-sm">分享第一条</RouterLink>
      </div>
    </section>

    <div class="tabs" role="tablist" aria-label="排序">
      <button
        v-for="t in TABS"
        :key="t.key"
        type="button"
        role="tab"
        :aria-selected="tab === t.key"
        :class="{ on: tab === t.key }"
        @click="setTab(t.key)"
      >
        {{ t.label }}
      </button>
    </div>
    <TopicList :items="list.items.value" :loading="list.loading.value" empty="还没有帖子，来发第一帖吧。" />
    <PagerNav :page="list.page.value" :page-size="list.pageSize.value" :total="list.total.value" @go="list.go" />

    <template #rail>
      <RightRail />
    </template>
  </ForumShell>
</template>

<style scoped>
.fresh {
  margin-bottom: 22px;
}
.fresh-head {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 12px;
}
.fresh-head h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
  letter-spacing: 0.02em;
}
.fresh-head p {
  margin: 0;
  font-size: var(--fs-sm);
  flex: 1;
}
.fresh-empty {
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
.fresh-empty p {
  margin: 0;
}
.tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 10px;
  border-bottom: 1px solid var(--line);
}
.tabs button {
  border: 0;
  background: none;
  padding: 8px 12px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-3);
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
}
.tabs button.on {
  color: var(--ink);
  border-bottom-color: var(--ink);
}
@media (max-width: 767px) {
  .fresh-head {
    flex-wrap: wrap;
    gap: 2px 10px;
  }
  .fresh-head h1 {
    font-size: var(--fs-xl);
  }
  .fresh-head p {
    order: 3;
    flex-basis: 100%;
  }
  .fresh-head .btn {
    margin-left: auto;
  }
  .fresh-empty {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
