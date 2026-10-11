<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useMeta } from '@/stores/meta'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'

useHead(() => ({ title: '福利羊毛' }))

const route = useRoute()
const router = useRouter()
const meta = useMeta()

const SORTS = [
  { key: 'new', label: '最新' },
  { key: 'hot', label: '热门' },
  { key: 'expiring', label: '快过期' },
] as const

const str = (k: string) => (typeof route.query[k] === 'string' ? (route.query[k] as string) : '')
const category = computed(() => str('category'))
const sort = computed(() => str('sort') || 'new')

const dealRoot = computed(() => meta.tree.find(n => n.kind === 'deal'))

function update(patch: Record<string, string | undefined>) {
  const next: Record<string, string> = {}
  for (const [k, v] of Object.entries({ ...route.query, ...patch })) {
    if (typeof v === 'string' && v) next[k] = v
  }
  router.replace({ query: next })
}

// 已到期的羊毛默认不显示；只按分类和排序取。
const list = useTopicList(
  p =>
    api.deals({
      category: category.value || undefined,
      sort: sort.value as 'new',
      page: p,
    }),
  [() => route.query],
)
</script>

<template>
  <ForumShell active="deals" :rail="false">
    <header class="head">
      <div>
        <h1>福利羊毛</h1>
        <p class="muted">大家分享的免费额度、试用和优惠，到期后自动下架。</p>
      </div>
      <RouterLink to="/new?category=deals-ai" class="btn btn-primary">分享羊毛</RouterLink>
    </header>

    <nav class="cats" aria-label="分类">
      <button type="button" :class="{ on: !category }" @click="update({ category: undefined })">全部</button>
      <button
        v-for="c in dealRoot?.children ?? []"
        :key="c.id"
        type="button"
        :class="{ on: category === c.slug }"
        @click="update({ category: c.slug })"
      >
        {{ c.name }}
      </button>
    </nav>

    <div class="bar">
      <div class="sorts" role="tablist" aria-label="排序">
        <button
          v-for="s in SORTS"
          :key="s.key"
          type="button"
          role="tab"
          :aria-selected="sort === s.key"
          :class="{ on: sort === s.key }"
          @click="update({ sort: s.key === 'new' ? undefined : s.key })"
        >
          {{ s.label }}
        </button>
      </div>
      <span class="muted count">{{ list.total.value }} 条</span>
    </div>

    <TopicList :items="list.items.value" :loading="list.loading.value">
      <template #empty>
        <p>{{ category ? '这个分类下还没有福利。' : '还没有人分享福利。' }}</p>
      </template>
    </TopicList>
    <PagerNav :page="list.page.value" :page-size="list.pageSize.value" :total="list.total.value" @go="list.go" />
  </ForumShell>
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 14px;
}
h1 {
  margin: 0 0 2px;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.head p {
  margin: 0;
  font-size: var(--fs-sm);
}
.cats {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}
.cats button {
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 14px;
  padding: 4px 12px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-2);
}
.cats button:hover {
  border-color: var(--line-strong);
}
.cats button.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.bar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}
.sorts {
  display: flex;
  gap: 2px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 2px;
}
.sorts button {
  border: 0;
  background: none;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-3);
}
.sorts button.on {
  background: var(--paper);
  color: var(--ink);
  box-shadow: 0 0 0 1px var(--line);
}
.count {
  font-size: var(--fs-sm);
}
@media (max-width: 767px) {
  .head .btn {
    display: none;
  }
  h1 {
    font-size: var(--fs-xl);
  }
  /* 手机上分类标签单行横向滑动。 */
  .cats {
    flex-wrap: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
    margin-right: -12px;
    padding-right: 12px;
  }
  .cats::-webkit-scrollbar {
    display: none;
  }
  .cats button {
    flex: none;
  }
  .bar {
    flex-wrap: wrap;
  }
  .sorts {
    overflow-x: auto;
    max-width: 100%;
  }
  .sorts button {
    flex: none;
  }
}
</style>
