<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import RightRail from '@/components/RightRail.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'

const route = useRoute()
const router = useRouter()
const meta = useMeta()
const session = useSession()

const slug = computed(() => String(route.params.slug))
const category = computed(() => meta.bySlug.get(slug.value))
const parent = computed(() => {
  const c = category.value
  return c?.parent_id ? meta.byId.get(c.parent_id) : undefined
})
const sort = computed(() => (route.query.sort === 'new' || route.query.sort === 'hot' ? route.query.sort : 'latest'))

useHead(() => ({ title: category.value?.name ?? '分类' }))

const list = useTopicList(p => api.topics({ category: slug.value, sort: sort.value, page: p }), [slug, sort])

/** 能发帖的分类：叶子分类且不是站务专用。一级分类发帖时让用户再选子分类。 */
const composeTarget = computed(() => {
  const c = category.value
  if (!c || c.admin_only) return null
  return `/new?category=${c.slug}`
})

function compose() {
  if (!composeTarget.value || !session.requireLogin()) return
  router.push(composeTarget.value)
}
</script>

<template>
  <ForumShell :active="slug">
    <header class="head">
      <div>
        <p v-if="parent" class="crumb">
          <RouterLink :to="`/c/${parent.slug}`">{{ parent.name }}</RouterLink>
        </p>
        <h1>{{ category?.name ?? '' }}</h1>
        <p v-if="category?.description" class="muted desc">{{ category.description }}</p>
      </div>
      <button v-if="composeTarget" type="button" class="btn btn-primary" @click="compose">在这里发帖</button>
    </header>

    <div class="tabs" role="tablist" aria-label="排序">
      <RouterLink :to="{ query: {} }" role="tab" :aria-selected="sort === 'latest'" :class="{ on: sort === 'latest' }">最新回复</RouterLink>
      <RouterLink :to="{ query: { sort: 'new' } }" role="tab" :aria-selected="sort === 'new'" :class="{ on: sort === 'new' }">新帖</RouterLink>
      <RouterLink :to="{ query: { sort: 'hot' } }" role="tab" :aria-selected="sort === 'hot'" :class="{ on: sort === 'hot' }">热门</RouterLink>
    </div>

    <p v-if="meta.categories.length && !category" class="panel missing">这个分类不存在，可能已经被合并或改名。</p>
    <template v-else>
      <TopicList :items="list.items.value" :loading="list.loading.value" :show-category="!parent">
        <template #empty>
          <p>这个分类还没有帖子。</p>
          <button v-if="composeTarget" type="button" class="btn btn-sm" @click="compose">发第一帖</button>
        </template>
      </TopicList>
      <PagerNav :page="list.page.value" :page-size="list.pageSize.value" :total="list.total.value" @go="list.go" />
    </template>

    <template #rail>
      <RightRail />
    </template>
  </ForumShell>
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 16px;
  margin-bottom: 14px;
}
.crumb {
  margin: 0;
  font-size: var(--fs-sm);
  color: var(--ink-3);
}
.crumb a:hover {
  color: var(--ink);
}
h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.desc {
  margin: 2px 0 0;
  font-size: var(--fs-sm);
}
.tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 10px;
  border-bottom: 1px solid var(--line);
}
.tabs a {
  padding: 8px 12px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-3);
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
}
.tabs a.on {
  color: var(--ink);
  border-bottom-color: var(--ink);
}
.missing {
  padding: 40px 20px;
  text-align: center;
  color: var(--ink-3);
}
@media (max-width: 767px) {
  h1 {
    font-size: var(--fs-xl);
  }
  .head .btn {
    height: 32px;
    font-size: var(--fs-xs);
  }
}
</style>
