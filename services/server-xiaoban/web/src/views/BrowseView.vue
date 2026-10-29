<script setup lang="ts">
import { computed, nextTick, onActivated, onDeactivated, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import FeedBar from '@/components/FeedBar.vue'
import FeedFooter from '@/components/FeedFooter.vue'
import Icon from '@/components/Icon.vue'
import StateBlock from '@/components/StateBlock.vue'
import WallpaperGrid from '@/components/WallpaperGrid.vue'
import { setAmbient } from '@/composables/ambient'
import { useFeed } from '@/composables/feed'
import { usePullToRefresh } from '@/composables/pullToRefresh'
import { openViewer } from '@/composables/viewer'
import { useCatalog } from '@/stores/catalog'
import type { FeedParams, Orientation, WallpaperItem } from '@/types'

type Kind = 'home' | 'category' | 'videos' | 'video-category' | 'search'

const route = useRoute()
const router = useRouter()
const catalog = useCatalog()

// Pages are kept alive per path, so what this instance shows is fixed when it is created.
const kind = route.name as Kind
const categoryId = kind === 'category' || kind === 'video-category' ? String(route.params.id) : undefined

const active = ref(true)
const q = ref(kind === 'search' ? String(route.query.q ?? '') : '')
const draft = ref(q.value)
const input = ref<HTMLInputElement>()
const orientation = ref<Orientation>('all')
const provider = ref('all')

const isVideo = kind === 'videos' || kind === 'video-category'
const media = isVideo ? 'video' : 'image'

const params = computed<FeedParams>(() => ({
  q: kind === 'search' ? q.value : undefined,
  category: categoryId,
  media,
  orientation: orientation.value,
  provider: provider.value,
}))

const { items, loading, loadingMore, error, hasMore, reload, loadMore } = useFeed(
  () => params.value,
  24,
  () => kind !== 'search' || !!q.value,
)

const { pull, refreshing, ready } = usePullToRefresh(reload)

/* ---- search ---- */

const RECENT_KEY = 'wc:recent-searches'

function readRecent(): string[] {
  try {
    const parsed = JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')
    return Array.isArray(parsed) ? parsed.filter((t): t is string => typeof t === 'string') : []
  } catch {
    return []
  }
}

const recent = ref<string[]>(readRecent())

function runSearch(term: string) {
  const value = term.trim()
  if (!value) return
  recent.value = [value, ...recent.value.filter((t) => t !== value)].slice(0, 8)
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent.value))
  } catch {
    // history is a convenience; ignore storage failures
  }
  draft.value = value
  q.value = value
  input.value?.blur()
  router.replace({ path: '/search', query: { q: value } })
}

function clearSearch() {
  draft.value = ''
  q.value = ''
  router.replace({ path: '/search' })
  nextTick(() => input.value?.focus())
}

watch(
  () => route.query.q,
  (value) => {
    if (kind !== 'search' || !active.value) return
    q.value = String(value ?? '')
    draft.value = q.value
  },
)

onActivated(() => {
  active.value = true
  if (kind === 'search') {
    q.value = String(route.query.q ?? '')
    draft.value = q.value
    if (!q.value) nextTick(() => input.value?.focus())
  }
  setAmbient(null)
})

onDeactivated(() => {
  active.value = false
})

function open(item: WallpaperItem) {
  openViewer(router, { items, loadMore }, item)
}

const showSearchHome = computed(() => kind === 'search' && !q.value)
</script>

<template>
  <main class="page browse">
    <div class="ptr" :style="{ height: `${pull}px` }" aria-hidden="true">
      <Icon name="refresh" :size="22" :class="{ spin: refreshing, ready }" />
    </div>

    <form v-if="kind === 'search'" class="searchbar" role="search" @submit.prevent="runSearch(draft)">
      <Icon name="search" :size="20" />
      <input
        ref="input"
        v-model="draft"
        type="search"
        enterkeyhint="search"
        placeholder="搜索壁纸，比如 极光、赛博朋克、雨夜"
        autocomplete="off"
        autocapitalize="off"
        aria-label="搜索壁纸"
      />
      <button v-if="draft" class="clear" type="button" aria-label="清除" @click="clearSearch">
        <Icon name="close" :size="18" />
      </button>
    </form>

    <FeedBar
      v-if="kind !== 'search' || q"
      v-model:orientation="orientation"
      v-model:provider="provider"
      :rail="kind !== 'search'"
      :active-category="categoryId"
      :media="media"
    />

    <section v-if="showSearchHome" class="suggest">
      <div v-if="recent.length" class="block">
        <h2 class="display">最近搜索</h2>
        <div class="chips">
          <button v-for="t in recent" :key="t" class="chip" type="button" @click="runSearch(t)">{{ t }}</button>
        </div>
      </div>
      <div class="block">
        <h2 class="display">按分类找</h2>
        <div class="chips">
          <RouterLink v-for="c in catalog.categories" :key="c.id" :to="`/c/${c.id}`" class="chip">{{ c.name }}</RouterLink>
        </div>
      </div>
    </section>

    <template v-else>
      <WallpaperGrid v-if="loading && items.length === 0" skeleton />

      <StateBlock
        v-else-if="error && items.length === 0"
        kind="error"
        title="没能加载壁纸"
        :hint="error"
        action="重试"
        @action="reload"
      />

      <StateBlock
        v-else-if="items.length === 0 && kind === 'search'"
        kind="empty"
        :title="`没有找到「${q}」`"
        hint="换个关键词，或者在筛选里放宽画幅和来源。"
      />

      <StateBlock
        v-else-if="items.length === 0"
        kind="empty"
        title="这里还没有壁纸"
        hint="可能是还没有配置任何数据源，或者当前筛选条件太严。"
        action="检查数据源"
        @action="router.push('/me')"
      />

      <template v-else>
        <WallpaperGrid :items="items" @open="open" />
        <FeedFooter
          :loading-more="loadingMore"
          :has-more="hasMore"
          :error="error"
          :count="items.length"
          @more="loadMore"
          @retry="loadMore"
        />
      </template>
    </template>
  </main>
</template>

<style scoped>
.browse {
  padding-top: 8px;
}

.ptr {
  display: grid;
  place-items: center;
  overflow: hidden;
  color: var(--text-3);
  transition: height 0.25s var(--ease);
}

.ptr .ready {
  color: var(--text);
}

.ptr .spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.heading {
  padding: 28px 0 12px;
}

.heading h1 {
  font-size: clamp(44px, 7vw, 76px);
}

.heading p {
  margin-top: 6px;
  max-width: 44ch;
  color: var(--text-2);
}

.searchbar {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 52px;
  margin: 12px 0 16px;
  padding: 0 18px;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--text-3);
}

.searchbar:focus-within {
  border-color: var(--line-strong);
  color: var(--text);
}

.searchbar input {
  flex: 1;
  min-width: 0;
  height: 100%;
  border: 0;
  outline: 0;
  background: none;
  color: var(--text);
  font-size: 16px;
}

.searchbar input::placeholder {
  color: var(--text-3);
}

.searchbar input::-webkit-search-cancel-button {
  display: none;
}

.clear {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--surface-2);
  color: var(--text-2);
}

.suggest {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding-top: 12px;
}

.block h2 {
  margin-bottom: 12px;
  font-size: 28px;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
