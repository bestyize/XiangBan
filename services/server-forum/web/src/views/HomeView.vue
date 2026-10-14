<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import RightRail from '@/components/RightRail.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'
import { useSession } from '@/stores/session'

useHead(() => ({ title: '' }))

const route = useRoute()
const router = useRouter()
const session = useSession()

const TABS = [
  { key: 'following', label: '关注' },
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

const list = useTopicList(
  p =>
    tab.value === 'following'
      ? session.loggedIn
        ? api.followingTopics(p)
        : Promise.resolve({ items: [], page: p, page_size: 30, total: 0 })
      : api.topics({
      home: true,
      sort: tab.value === 'featured' ? 'new' : tab.value,
      featured: tab.value === 'featured',
      page: p,
    }),
  [tab, () => session.user?.uid, () => session.loggedIn],
)

function setTab(t: Tab) {
  if (t === 'following' && !session.requireLogin()) return
  router.replace({ query: t === 'latest' ? {} : { tab: t } })
}
</script>

<template>
  <ForumShell active="home">
    <h1 class="sr-only">超级小伴</h1>
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
    <div v-if="tab === 'following'" class="following-head">
      <span class="muted">关注用户的最新帖子</span>
      <RouterLink v-if="session.user" :to="`/u/${session.user.uid}?tab=following`">我关注的人</RouterLink>
    </div>
    <div v-if="tab === 'following' && !session.loggedIn" class="panel login-prompt">
      <p class="muted">登录后查看关注动态。</p>
      <button class="btn btn-primary" type="button" @click="session.loginOpen = true">登录</button>
    </div>
    <TopicList
      v-else
      :items="list.items.value"
      :loading="list.loading.value"
      :empty="tab === 'following' ? '暂时没有关注动态，去用户主页关注感兴趣的人吧。' : '还没有帖子，来发第一帖吧。'"
    />
    <PagerNav :page="list.page.value" :page-size="list.pageSize.value" :total="list.total.value" @go="list.go" />

    <template #rail>
      <RightRail />
    </template>
  </ForumShell>
</template>

<style scoped>
.following-head { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 8px; padding: 4px 0 12px; font-size: var(--fs-xs); }
.following-head a { color: var(--brand); }
.login-prompt { padding: 32px 16px; text-align: center; }
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
</style>

