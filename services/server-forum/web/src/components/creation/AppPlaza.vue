<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { contentApi } from '@/api/content'
import AppIcon from '@/components/AppIcon.vue'
import ModalDialog from '@/components/ModalDialog.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import FilterChips from '@/components/community/FilterChips.vue'
import RequestState from '@/components/community/RequestState.vue'
import { webLink } from '@/components/community/contract'
import { useCommunityResource } from '@/components/community/resource'
import { ANDROID_DOWNLOAD_URL, BUILTIN_GAMES, builtinGame, MOBILE_ONLY_APPS, WANCISOU_ICON } from '@/lib/builtinCreation'
import { appRunLocation } from '@/lib/appRuntime'

const emit = defineEmits<{ play: [id: string] }>()
const route = useRoute(), router = useRouter()
const playableGames = BUILTIN_GAMES.filter(game => game.id !== 'mbti')
const games = [
  ...playableGames.slice(0, 3).map(game => ({ ...game, playable: true })),
  { ...MOBILE_ONLY_APPS.pocketSprites, playable: false },
  ...playableGames.slice(3).map(game => ({ ...game, playable: true })),
]
const mbti = builtinGame('mbti')!
const mobileTools = [
  { ...MOBILE_ONLY_APPS.remoteControl, note: '在支持红外的 Android 手机中，打开「应用 → 广场 → 精选工具 → 遥控器」。' },
  { ...MOBILE_ONLY_APPS.quickLaunch, note: '在 Android 小伴 App 中打开「应用 → 广场 → 精选工具 → 轻启」，为已安装的应用创建桌面快捷方式。' },
]
const mobileTool = ref<(typeof mobileTools)[number] | null>(null)
const showAllGames = computed(() => route.query.kind === 'game')
const showAllTools = computed(() => route.query.kind === 'app')
const filters = [
  { key: 'featured', label: '精选' }, { key: 'latest', label: '最新' },
  { key: 'game', label: '游戏' }, { key: 'app', label: '工具' },
]
const filter = computed(() => filters.find(item => item.key === route.query.feed)?.key || 'featured')
const page = computed(() => Math.min(1000, Math.max(1, Math.floor(Number(route.query.page)) || 1)))
const tools = useCommunityResource(() => contentApi.packs({ kind: 'app', featured: true }))
watch(showAllGames, gamesOnly => { if (!gamesOnly) void tools.load() }, { immediate: true })
const featuredTools = computed(() => (tools.data.value?.items ?? [])
  .slice(0, showAllTools.value ? undefined : 1)
  .map(tool => ({ ...tool, icon: tool.slug === 'wancisou' ? WANCISOU_ICON : webLink(tool.cover_url) })))
const failedIcons = reactive(new Set<string>())
const linkedTool = computed(() => {
  const slug = route.query.tool
  if (showAllGames.value || typeof slug !== 'string' || !slug) return null
  return tools.data.value?.items.find(tool => tool.slug === slug) ?? null
})
// Keep links shared before tools moved to the independent window.
watch(linkedTool, tool => {
  if (tool) void router.replace(appRunLocation(tool.id, tool.version))
}, { immediate: true })

const feed = useCommunityResource(async () => {
  if (showAllGames.value || showAllTools.value) return null
  const selected = filter.value
  const result = await contentApi.packs({
    kind: selected === 'featured' ? undefined : selected === 'app' ? 'app' : 'game',
    featured: selected === 'featured' || undefined,
    sort: selected === 'latest' ? 'new' : selected === 'featured' ? undefined : 'hot',
    page: page.value,
  })
  const items = selected === 'featured' ? result.items.filter(pack => pack.featured && (pack.kind === 'game' || pack.kind === 'app')) : result.items
  return { ...result, items }
})
watch([filter, page, showAllGames, showAllTools], feed.load, { immediate: true })

function select(key: string) {
  void router.replace({ query: { ...route.query, feed: key === 'featured' ? undefined : key, page: undefined } })
}
function go(page: number) {
  void router.replace({ query: { ...route.query, page: page === 1 ? undefined : page } })
}
</script>

<template>
  <section v-if="!showAllTools" class="plaza-section" aria-labelledby="plaza-games">
    <header>
      <h2 id="plaza-games">精选游戏</h2>
      <RouterLink v-if="!showAllGames" to="/create?kind=game">全部 <span aria-hidden="true">›</span></RouterLink>
      <RouterLink v-else to="/create">返回广场</RouterLink>
    </header>
    <div class="game-strip" :class="{ 'game-grid': showAllGames }">
      <template v-for="game in games" :key="game.id">
        <button v-if="game.playable" type="button" class="game-tile" :aria-label="`试玩${game.name}`" @click="emit('play', game.id)">
          <img class="shortcut-icon" :src="game.icon" alt="" decoding="async" />
          <strong>{{ game.name }}</strong><small>{{ game.genre }}</small>
        </button>
        <RouterLink v-else class="game-tile" to="/ai">
          <img class="shortcut-icon" :src="game.icon" alt="" decoding="async" />
          <strong>{{ game.name }}</strong><small>在手机 App 游玩</small>
        </RouterLink>
      </template>
    </div>
  </section>

  <section v-if="!showAllGames" class="plaza-section" aria-labelledby="plaza-tools">
    <header>
      <h2 id="plaza-tools">精选工具</h2>
      <RouterLink v-if="!showAllTools" to="/create?kind=app">全部 <span aria-hidden="true">›</span></RouterLink>
      <RouterLink v-else to="/create">返回广场</RouterLink>
    </header>
    <div class="tool-grid">
      <RouterLink class="tool-tile" :to="{ name: 'mbti-run' }">
        <img class="shortcut-icon" :src="mbti.icon" alt="" decoding="async" />
        <strong>{{ mbti.name }}</strong>
      </RouterLink>
      <button v-for="tool in mobileTools" :key="tool.id" class="tool-tile" type="button" @click="mobileTool = tool">
        <img class="shortcut-icon" :src="tool.icon" alt="" decoding="async" />
        <strong>{{ tool.name }}</strong><small>在手机 App 使用</small>
      </button>
      <RouterLink v-for="tool in featuredTools" :key="tool.id" class="tool-tile" :to="appRunLocation(tool.id, tool.version)">
        <img v-if="tool.icon && !failedIcons.has(tool.icon)" class="shortcut-icon" :src="tool.icon" alt="" decoding="async" @error="failedIcons.add(tool.icon)" />
        <span v-else class="shortcut-icon icon-fallback" aria-hidden="true"><AppIcon name="grid" /></span>
        <strong>{{ tool.title }}</strong>
      </RouterLink>
    </div>
    <p v-if="tools.loading.value" class="tools-status" role="status">正在加载社区工具…</p>
    <p v-else-if="tools.error.value" class="tools-status" role="alert">{{ tools.error.value }} <button class="btn btn-sm" type="button" @click="tools.load">重试</button></p>
  </section>

  <ModalDialog :open="Boolean(mobileTool)" :title="mobileTool?.name ?? '在手机 App 使用'" @close="mobileTool = null">
    <div v-if="mobileTool" class="tool-note">
      <p>{{ mobileTool.note }}</p>
      <a class="btn btn-primary" :href="ANDROID_DOWNLOAD_URL">下载 Android 版</a>
    </div>
  </ModalDialog>

  <section v-if="!showAllGames && !showAllTools" class="plaza-section" aria-labelledby="plaza-community">
    <header><h2 id="plaza-community">社区作品</h2></header>
    <FilterChips class="plaza-filters" :items="filters" :active="filter" label="社区作品筛选" @change="select" />
    <RequestState :loading="feed.loading.value" :error="feed.error.value"
      :empty="Boolean(feed.data.value && !feed.data.value.items.length)"
      :message="filter === 'featured' ? '暂无精选作品' : filter === 'app' ? '还没有社区工具' : '还没有社区作品'" @retry="feed.load" />
    <div v-if="feed.data.value?.items.length" class="plaza-packs">
      <CommunityPackCard v-for="pack in feed.data.value.items" :key="pack.id" :pack="pack" layout="visual" />
    </div>
    <nav v-if="feed.data.value && (page > 1 || page * feed.data.value.page_size < feed.data.value.total)" class="plaza-pager" aria-label="社区作品分页">
      <button class="btn" type="button" :disabled="page <= 1" @click="go(page - 1)">上一页</button>
      <span>第 {{ page }} 页</span>
      <button class="btn" type="button" :disabled="page >= 1000 || page * feed.data.value.page_size >= feed.data.value.total" @click="go(page + 1)">下一页</button>
    </nav>
  </section>
</template>

<style scoped>
.plaza-section { margin-bottom: 24px; min-width: 0; }
.plaza-section > header { display: flex; align-items: center; justify-content: space-between; gap: 12px; min-height: 44px; margin-bottom: 8px; }
h2 { margin: 0; font-size: 16px; font-weight: 650; }
header a { display: inline-flex; align-items: center; gap: 4px; min-height: 44px; color: var(--ink-2); font-size: 13px; }
header a:hover { color: var(--brand); }
.game-strip { display: flex; gap: 8px; overflow-x: auto; scrollbar-width: thin; padding: 4px 2px 6px; }
.game-tile { flex: 0 0 min(100px, calc((100% - 24px) / 4)); }
.game-grid, .tool-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 100px)); gap: 8px; }
.game-tile, .tool-tile { display: flex; flex-direction: column; align-items: center; gap: 4px; min-width: 0; padding: 4px 0; border: 0; border-radius: 8px; background: none; color: var(--ink); text-align: center; }
.game-tile:hover, .tool-tile:hover { color: var(--brand); }
.shortcut-icon { display: block; width: min(56px, 100%); aspect-ratio: 1; border-radius: 16px; object-fit: cover; background: var(--surface-2); }
.icon-fallback { display: grid; place-items: center; color: var(--brand); }
.game-tile strong, .tool-tile strong { max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 500; }
small { color: var(--ink-3); font-size: 12px; }
.tools-status { margin: 8px 0 0; color: var(--ink-3); font-size: 12px; }
.tool-note { display: grid; justify-items: start; gap: 14px; }
.tool-note p { margin: 0; color: var(--ink-2); }
.plaza-filters { padding-inline: 0; }
.plaza-packs { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.plaza-pager { display: flex; align-items: center; justify-content: center; gap: 12px; margin-top: 12px; font-size: 13px; }
@media (min-width: 768px) {
  .plaza-packs { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
</style>
