<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useMeta } from '@/stores/meta'
import { CONDITIONS, DEAL_TYPE_LABEL } from '@/lib/format'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'

useHead(() => ({ title: 'AI 羊毛库' }))

const route = useRoute()
const router = useRouter()
const meta = useMeta()

const SORTS = [
  { key: 'new', label: '最新' },
  { key: 'verified', label: '最近验证' },
  { key: 'hot', label: '热门' },
  { key: 'expiring', label: '快过期' },
] as const

const str = (k: string) => (typeof route.query[k] === 'string' ? (route.query[k] as string) : '')
const category = computed(() => str('category'))
const product = computed(() => str('product'))
const dealType = computed(() => str('type'))
const region = computed(() => str('region'))
const sort = computed(() => str('sort') || 'new')
const without = computed(() => str('without').split(',').filter(Boolean))
const showExpired = computed(() => str('expired') === '1')

const dealRoot = computed(() => meta.tree.find(n => n.kind === 'deal'))

function update(patch: Record<string, string | undefined>) {
  const next: Record<string, string> = {}
  for (const [k, v] of Object.entries({ ...route.query, ...patch })) {
    if (typeof v === 'string' && v) next[k] = v
  }
  router.replace({ query: next })
}

function toggleWithout(key: string) {
  const set = new Set(without.value)
  if (set.has(key)) set.delete(key)
  else set.add(key)
  update({ without: [...set].join(',') })
}

const filtersOn = computed(
  () => !!(category.value || product.value || dealType.value || region.value || without.value.length || showExpired.value),
)

const list = useTopicList(
  p =>
    api.deals({
      category: category.value || undefined,
      product: product.value || undefined,
      deal_type: dealType.value || undefined,
      region: region.value || undefined,
      without: without.value.join(',') || undefined,
      deal_status: showExpired.value ? 'all' : 'active',
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
        <h1>AI 羊毛库</h1>
        <p class="muted">按条件筛出你能领的。失效的会自动沉底，也欢迎你帮忙投票。</p>
      </div>
      <RouterLink to="/new?category=deals-llm-api" class="btn btn-primary">分享羊毛</RouterLink>
    </header>

    <section class="filters panel" aria-label="筛选">
      <div class="frow">
        <span class="flabel">分类</span>
        <div class="opts">
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
        </div>
      </div>
      <div class="frow">
        <span class="flabel">我没有</span>
        <div class="opts">
          <button
            v-for="c in CONDITIONS"
            :key="c.key"
            type="button"
            :class="{ on: without.includes(c.key) }"
            :aria-pressed="without.includes(c.key)"
            @click="toggleWithout(c.key)"
          >
            {{ c.label.replace('需要', '') }}
          </button>
        </div>
      </div>
      <div class="frow selects">
        <label>
          <span class="flabel">类型</span>
          <select class="select" :value="dealType" @change="update({ type: ($event.target as HTMLSelectElement).value })">
            <option value="">全部类型</option>
            <option v-for="(label, key) in DEAL_TYPE_LABEL" :key="key" :value="key">{{ label }}</option>
          </select>
        </label>
        <label>
          <span class="flabel">产品</span>
          <select class="select" :value="product" @change="update({ product: ($event.target as HTMLSelectElement).value })">
            <option value="">全部产品</option>
            <option v-for="p in meta.products" :key="p.id" :value="p.slug">{{ p.name }}</option>
          </select>
        </label>
        <label>
          <span class="flabel">地区</span>
          <select class="select" :value="region" @change="update({ region: ($event.target as HTMLSelectElement).value })">
            <option value="">不限</option>
            <option value="cn">国内能用</option>
            <option value="intl">海外能用</option>
          </select>
        </label>
        <label class="check">
          <input type="checkbox" :checked="showExpired" @change="update({ expired: showExpired ? undefined : '1' })" />
          <span>显示已失效</span>
        </label>
      </div>
    </section>

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
      <button v-if="filtersOn" type="button" class="btn btn-ghost btn-sm" @click="router.replace({ query: {} })">清除筛选</button>
    </div>

    <TopicList :items="list.items.value" :loading="list.loading.value">
      <template #empty>
        <p>{{ filtersOn ? '没有符合这些条件的羊毛，放宽一点试试。' : '羊毛库还是空的。' }}</p>
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
.filters {
  padding: 12px 14px;
  display: grid;
  gap: 10px;
  margin-bottom: 14px;
}
.frow {
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr);
  gap: 10px;
  align-items: start;
}
.flabel {
  font-size: var(--fs-xs);
  font-weight: 700;
  color: var(--ink-3);
  padding-top: 4px;
}
.opts {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.opts button {
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 14px;
  padding: 3px 11px;
  font-size: var(--fs-xs);
  font-weight: 600;
  color: var(--ink-2);
}
.opts button:hover {
  border-color: var(--line-strong);
}
.opts button.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.selects {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 16px;
  align-items: center;
  padding-top: 10px;
  border-top: 1px solid var(--line);
}
.selects label {
  display: flex;
  align-items: center;
  gap: 8px;
}
.selects .flabel {
  padding: 0;
}
.selects .select {
  width: auto;
  min-width: 120px;
  padding: 4px 8px;
  font-size: var(--fs-sm);
}
.check {
  font-size: var(--fs-sm);
  color: var(--ink-2);
  margin-left: auto;
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
  .frow {
    grid-template-columns: minmax(0, 1fr);
    gap: 6px;
  }
  .flabel {
    padding: 0;
  }
  .opts {
    flex-wrap: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
    margin: 0 -14px;
    padding: 0 14px;
  }
  .opts button {
    flex: none;
  }
  .selects label:not(.check) {
    flex: 1 1 40%;
  }
  .selects .select {
    min-width: 0;
    flex: 1;
  }
  .check {
    margin-left: 0;
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
