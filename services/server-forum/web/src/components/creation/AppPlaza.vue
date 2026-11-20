<script setup lang="ts">
import { computed, reactive, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { contentApi } from '@/api/content'
import AppIcon from '@/components/AppIcon.vue'
import ModalDialog from '@/components/ModalDialog.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import RequestState from '@/components/community/RequestState.vue'
import { webLink } from '@/components/community/contract'
import { useCommunityResource } from '@/components/community/resource'
import { ANDROID_DOWNLOAD_URL, BUILTIN_GAMES, builtinGame, MOBILE_ONLY_APPS, WANCISOU_ICON } from '@/lib/builtinCreation'
import { appRunLocation, builtinRunLocation } from '@/lib/appRuntime'

const route = useRoute(), router = useRouter()
const playableGames = BUILTIN_GAMES.filter(game => game.id !== 'mbti')
const games = [
  ...playableGames.slice(0, 3).map(game => ({ ...game, playable: true })),
  { ...MOBILE_ONLY_APPS.pocketSprites, playable: false },
  ...playableGames.slice(3).map(game => ({ ...game, playable: true })),
]
const mbti = builtinGame('mbti')!
const wallpaperIcon = `${import.meta.env.BASE_URL}creation-previews/live-wallpapers.webp`
const mobileTools = [
  { ...MOBILE_ONLY_APPS.remoteControl, genre: '工具', note: '在支持红外的 Android 手机中，打开「应用 → 广场 → 精选工具 → 遥控器」。' },
  { ...MOBILE_ONLY_APPS.quickLaunch, genre: '效率', note: '在 Android 小伴 App 中打开「应用 → 广场 → 精选工具 → 轻启」，为已安装的应用创建桌面快捷方式。' },
]
const mobileTool = ref<(typeof mobileTools)[number] | null>(null)
const showAllGames = computed(() => route.query.kind === 'game')
const showAllTools = computed(() => route.query.kind === 'app')
const previewGrid = useTemplateRef<HTMLDivElement>('previewGrid')
const previewColumns = ref(4)
const visibleGames = computed(() => showAllGames.value ? games : games.slice(0, previewColumns.value))
// The two preview grids share the same column layout.
watch(previewGrid, (grid, _, onCleanup) => {
  if (!grid) return
  const measure = () => {
    const columns = getComputedStyle(grid).gridTemplateColumns
    if (columns !== 'none') previewColumns.value = columns.split(/\s+/).length
  }
  measure()
  if (typeof ResizeObserver === 'undefined') return
  const observer = new ResizeObserver(measure)
  observer.observe(grid)
  onCleanup(() => observer.disconnect())
}, { flush: 'post' })
const filters = [
  { key: 'featured', label: '精选' }, { key: 'latest', label: '最新' },
  { key: 'game', label: '游戏' }, { key: 'app', label: '工具' },
]
const filter = computed(() => filters.find(item => item.key === route.query.feed)?.key || 'featured')
const page = computed(() => Math.min(1000, Math.max(1, Math.floor(Number(route.query.page)) || 1)))
const tools = useCommunityResource(() => contentApi.packs({ kind: 'app', featured: true }))
watch(showAllGames, gamesOnly => { if (!gamesOnly) void tools.load() }, { immediate: true })
const featuredTools = computed(() => {
  const items = tools.data.value?.items ?? []
  const builtInToolCount = mobileTools.length + 2 // MBTI and dynamic wallpapers
  const visible = showAllTools.value ? items : items.slice(0, Math.max(0, previewColumns.value - builtInToolCount))
  return visible.map(tool => ({ ...tool, icon: tool.slug === 'wancisou' ? WANCISOU_ICON : webLink(tool.cover_url) }))
})
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
    <header class="plaza-head">
      <h2 id="plaza-games">精选游戏</h2>
      <RouterLink v-if="!showAllGames" class="plaza-more-btn" to="/create?kind=game">全部 <AppIcon name="arrow" :size="13" /></RouterLink>
      <RouterLink v-else class="plaza-more-btn" to="/create">返回广场</RouterLink>
    </header>
    <div ref="previewGrid" class="plaza-icon-grid">
      <RouterLink v-for="game in visibleGames" :key="game.id"
        class="plaza-icon-item" :aria-label="game.playable ? `试玩${game.name}` : `${game.name}（在手机 App 游玩）`"
        :to="game.playable ? builtinRunLocation(game.id) : '/ai'">
        <span class="plaza-app-icon"><img :src="game.icon" alt="" decoding="async" /></span>
        <span class="plaza-app-name">{{ game.name }}</span>
        <small class="plaza-app-sub">{{ game.genre }}</small>
      </RouterLink>
    </div>
  </section>

  <section v-if="!showAllGames" class="plaza-section" aria-labelledby="plaza-tools">
    <header class="plaza-head">
      <h2 id="plaza-tools">精选工具</h2>
      <RouterLink v-if="!showAllTools" class="plaza-more-btn" to="/create?kind=app">全部 <AppIcon name="arrow" :size="13" /></RouterLink>
      <RouterLink v-else class="plaza-more-btn" to="/create">返回广场</RouterLink>
    </header>
    <div class="plaza-icon-grid">
      <RouterLink class="plaza-icon-item" :to="builtinRunLocation(mbti.id)">
        <span class="plaza-app-icon"><img :src="mbti.icon" alt="" decoding="async" /></span>
        <span class="plaza-app-name">{{ mbti.name }}</span>
        <small v-if="showAllTools" class="plaza-app-sub">{{ mbti.genre }}</small>
      </RouterLink>
      <button v-for="tool in mobileTools" :key="tool.id" class="plaza-icon-item" type="button"
        :aria-label="`${tool.name}（在手机 App 使用）`" @click="mobileTool = tool">
        <span class="plaza-app-icon"><img :src="tool.icon" alt="" decoding="async" /></span>
        <span class="plaza-app-name">{{ tool.name }}</span>
        <small v-if="showAllTools" class="plaza-app-sub">{{ tool.genre }}</small>
      </button>
      <RouterLink class="plaza-icon-item" to="/create?kind=wallpaper">
        <span class="plaza-app-icon"><img :src="wallpaperIcon" alt="" decoding="async" /></span>
        <span class="plaza-app-name">动态壁纸</span>
        <small v-if="showAllTools" class="plaza-app-sub">桌面</small>
      </RouterLink>
      <RouterLink v-for="tool in featuredTools" :key="tool.id" class="plaza-icon-item" :to="appRunLocation(tool.id, tool.version)">
        <span class="plaza-app-icon">
          <img v-if="tool.icon && !failedIcons.has(tool.icon)" :src="tool.icon" alt="" decoding="async" @error="failedIcons.add(tool.icon)" />
          <AppIcon v-else name="grid" />
        </span>
        <span class="plaza-app-name">{{ tool.title }}</span>
        <small v-if="showAllTools && tool.slug === 'wancisou'" class="plaza-app-sub">检索</small>
      </RouterLink>
    </div>
    <template v-if="showAllTools">
      <p v-if="tools.loading.value" class="tools-status" role="status">正在加载社区工具…</p>
      <p v-else-if="tools.error.value" class="tools-status" role="alert">{{ tools.error.value }} <button class="btn btn-sm" type="button" @click="tools.load">重试</button></p>
    </template>
  </section>

  <ModalDialog :open="Boolean(mobileTool)" :title="mobileTool?.name ?? '在手机 App 使用'" @close="mobileTool = null">
    <div v-if="mobileTool" class="tool-note">
      <p>{{ mobileTool.note }}</p>
      <a class="btn btn-primary" :href="ANDROID_DOWNLOAD_URL">下载 Android 版</a>
    </div>
  </ModalDialog>

  <section v-if="!showAllGames && !showAllTools" class="plaza-section" aria-labelledby="plaza-community">
    <header class="plaza-head"><h2 id="plaza-community">社区作品</h2></header>
    <div class="plaza-filters" role="group" aria-label="社区作品筛选">
      <button v-for="item in filters" :key="item.key" class="filter-pill" type="button"
        :aria-pressed="filter === item.key" @click="select(item.key)">{{ item.label }}</button>
    </div>
    <RequestState :loading="feed.loading.value" :error="feed.error.value"
      :empty="Boolean(feed.data.value && !feed.data.value.items.length)"
      :message="filter === 'featured' ? '暂无精选作品' : filter === 'app' ? '还没有社区工具' : '还没有社区作品'" @retry="feed.load" />
    <div v-if="feed.data.value?.items.length" class="plaza-packs">
      <CommunityPackCard v-for="pack in feed.data.value.items" :key="pack.id" :pack="pack" layout="visual" plaza />
    </div>
    <nav v-if="feed.data.value && (page > 1 || page * feed.data.value.page_size < feed.data.value.total)" class="plaza-pager" aria-label="社区作品分页">
      <button class="btn" type="button" :disabled="page <= 1" @click="go(page - 1)">上一页</button>
      <span>第 {{ page }} 页</span>
      <button class="btn" type="button" :disabled="page >= 1000 || page * feed.data.value.page_size >= feed.data.value.total" @click="go(page + 1)">下一页</button>
    </nav>
    <p v-else-if="feed.data.value?.items.length" class="plaza-end">已看完所有作品</p>
  </section>
</template>

<style scoped>
.plaza-section { margin-bottom: 24px; }
.plaza-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 12px; }
.plaza-head h2 { margin: 0; font-size: 16.5px; font-weight: 700; color: var(--ink); }
.plaza-more-btn {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 2px 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--community-muted);
}
.plaza-more-btn:hover { color: var(--ink); }
.plaza-icon-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(98px, 1fr));
  gap: 16px 14px;
}
.plaza-icon-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 0;
  gap: 6px;
  border: 0;
  background: transparent;
  padding: 2px 0;
  text-align: center;
  color: var(--ink);
}
.plaza-app-icon {
  width: 68px;
  height: 68px;
  border-radius: 17px;
  overflow: hidden;
  display: grid;
  place-items: center;
  background: var(--surface);
  border: 1px solid var(--community-divider);
  box-shadow: 0 2px 10px rgba(15, 23, 42, 0.06);
  transition: transform 0.16s ease;
}
.plaza-icon-item:hover .plaza-app-icon { transform: translateY(-2px); }
.plaza-app-icon img { width: 100%; height: 100%; object-fit: cover; display: block; }
.plaza-app-name {
  font-size: 12.5px;
  font-weight: 500;
  line-height: 1.25;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
.plaza-app-sub { font-size: 11px; color: var(--community-muted); line-height: 1.15; margin-top: -3px; }
.tools-status { margin: 8px 0 0; color: var(--community-muted); font-size: 12px; }
.tool-note { display: grid; justify-items: start; gap: 14px; }
.tool-note p { margin: 0; color: var(--ink-2); }
.plaza-filters { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; margin-bottom: 14px; }
.filter-pill {
  flex: none;
  height: 28px;
  padding: 0 8px;
  border: 0;
  background: transparent;
  color: var(--community-muted);
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  transition: color 0.15s;
}
.filter-pill:hover, .filter-pill[aria-pressed="true"] { color: var(--ink); }
.filter-pill[aria-pressed="true"] { font-weight: 700; }
.plaza-packs { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
.plaza-end { margin: 16px 0 6px; font-size: 12px; color: var(--community-muted); }
.plaza-pager { display: flex; align-items: center; justify-content: center; gap: 12px; margin-top: 12px; font-size: 13px; }
@media (max-width: 1023px) {
  .plaza-section { margin-bottom: 18px; }
  .plaza-head, .plaza-icon-grid, .plaza-end { padding-inline: 6px; }
  .plaza-icon-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px 10px; }
  .plaza-app-icon { width: 60px; height: 60px; border-radius: 15px; }
  .plaza-filters { flex-wrap: nowrap; overflow-x: auto; scrollbar-width: none; gap: 6px; margin-bottom: 8px; padding-bottom: 2px; }
  .filter-pill { height: 26px; padding: 0 7px; font-size: 12px; }
  .plaza-packs { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; }
}
</style>
