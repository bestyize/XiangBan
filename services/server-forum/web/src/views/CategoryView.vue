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
import { requiredPostLevel } from '@/lib/format'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'
import { tagLabel } from '@/lib/format'

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

// 板块的内置标签可以当筛选用：?tag=xxx。换了板块或者标签不在这个板块里，就当没选。
const presetTags = computed(() => category.value?.preset_tags ?? [])
const tag = computed(() => {
  const t = typeof route.query.tag === 'string' ? route.query.tag : ''
  return presetTags.value.includes(t) ? t : ''
})

const list = useTopicList(
  p => api.topics({ category: slug.value, tag: tag.value || undefined, sort: sort.value, page: p }),
  [slug, sort, tag],
)

/** 切排序时保留标签，切标签时保留排序。 */
function withQuery(patch: { sort?: string; tag?: string }) {
  const q: Record<string, string> = {}
  const nextSort = 'sort' in patch ? patch.sort : sort.value
  const nextTag = 'tag' in patch ? patch.tag : tag.value
  if (nextSort && nextSort !== 'latest') q.sort = nextSort
  if (nextTag) q.tag = nextTag
  return { query: q }
}

/** 能发帖的分类：叶子分类且不是站务专用。一级分类发帖时让用户再选子分类。 */
const composeTarget = computed(() => {
  const c = category.value
  if (!c) return null
  // 已登录但等级不够发帖的板块（比如只有守护者能发的「小伴产品」和站务公告）不显示按钮；没登录的先让他登录。
  const level = session.user?.trust_level
  if (level !== undefined && level < requiredPostLevel(c)) return null
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

    <nav v-if="presetTags.length" class="tag-filter" aria-label="按标签筛选">
      <RouterLink :to="withQuery({ tag: '' })" class="chip" :class="{ on: !tag }">全部</RouterLink>
      <RouterLink v-for="t in presetTags" :key="t" :to="withQuery({ tag: t })" class="chip" :class="{ on: tag === t }">{{ tagLabel(t) }}</RouterLink>
    </nav>

    <div class="tabs" role="tablist" aria-label="排序">
      <RouterLink :to="withQuery({ sort: 'latest' })" role="tab" :aria-selected="sort === 'latest'" :class="{ on: sort === 'latest' }">最新回复</RouterLink>
      <RouterLink :to="withQuery({ sort: 'new' })" role="tab" :aria-selected="sort === 'new'" :class="{ on: sort === 'new' }">新帖</RouterLink>
      <RouterLink :to="withQuery({ sort: 'hot' })" role="tab" :aria-selected="sort === 'hot'" :class="{ on: sort === 'hot' }">热门</RouterLink>
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
.tag-filter {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}
.tag-filter .chip {
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 14px;
  padding: 3px 12px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-2);
}
.tag-filter .chip:hover {
  border-color: var(--line-strong);
}
.tag-filter .chip.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
@media (max-width: 767px) {
  /* 手机上标签单行横向滑动。 */
  .tag-filter {
    flex-wrap: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
    margin-right: -12px;
    padding-right: 12px;
  }
  .tag-filter::-webkit-scrollbar {
    display: none;
  }
  .tag-filter .chip {
    flex: none;
  }
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
