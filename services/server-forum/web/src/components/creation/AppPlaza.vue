<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { contentApi } from '@/api/content'
import AppIcon from '@/components/AppIcon.vue'
import ModalDialog from '@/components/ModalDialog.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import RequestState from '@/components/community/RequestState.vue'
import { webLink } from '@/components/community/contract'
import { useCommunityResource } from '@/components/community/resource'
import { ANDROID_DOWNLOAD_URL, BUILTIN_GAMES, builtinGame, MOBILE_ONLY_APPS, WANCISOU_ICON, type BuiltinGame } from '@/lib/builtinCreation'
import { appRunLocation, builtinRunLocation } from '@/lib/appRuntime'

const route = useRoute(), router = useRouter()
const games = BUILTIN_GAMES.filter(game => game.id !== 'mbti')
const mbti = builtinGame('mbti')!
const wallpaperIcon = `${import.meta.env.BASE_URL}creation-previews/live-wallpapers.webp`
const mobileTools = [
  { ...MOBILE_ONLY_APPS.remoteControl, genre: '工具', desc: '电视空调红外遥控', note: '在支持红外的 Android 手机中，打开「应用 → 广场 → 精选工具 → 遥控器」。' },
  { ...MOBILE_ONLY_APPS.quickLaunch, genre: '效率', desc: '桌面快捷启动', note: '在 Android 小伴 App 中打开「应用 → 广场 → 精选工具 → 轻启」，为已安装的应用创建桌面快捷方式。' },
]
const mobileTool = ref<(typeof mobileTools)[number] | null>(null)
const showAllGames = computed(() => route.query.kind === 'game')
const showAllTools = computed(() => route.query.kind === 'app')

const GAME_TAGLINES: Record<string, string> = {
  'block-puzzle': '8×8 经典消除连击',
  'dusk-glider': '乘日落气流收集微光',
  'ink-blade': '极简水墨一击必杀',
  'pro-cube': '3D 真实手感计时',
  'snake-arena': '无尽成长冲分挑战',
  'jade-gomoku': '水墨玉石人机对弈',
  'merge-2048': '顺滑数字合成挑战',
  'water-sort': '45 关调饮分类手帐',
}
function gameTagline(game: BuiltinGame) {
  return GAME_TAGLINES[game.id] || game.description
}

const gameColumns = computed(() => {
  const groups: BuiltinGame[][] = []
  for (let i = 0; i < games.length; i += 3) groups.push(games.slice(i, i + 3))
  return groups
})

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
  const visible = showAllTools.value ? items : items.slice(0, 4)
  return visible.map(tool => ({
    ...tool,
    icon: tool.slug === 'wancisou' ? WANCISOU_ICON : webLink(tool.cover_url),
    desc: tool.summary?.trim() || (tool.slug === 'wancisou' ? '公开资源快速检索' : '社区精选工具'),
  }))
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
  <!-- 1. 精选游戏：方案 A 三行精装榜单流（仅复用 1:1 图标） -->
  <section v-if="!showAllTools" class="plaza-section" aria-labelledby="plaza-games">
    <header class="plaza-head">
      <h2 id="plaza-games">精选游戏</h2>
      <RouterLink v-if="!showAllGames" class="plaza-more-btn" to="/create?kind=game">全部 <AppIcon name="arrow" :size="13" /></RouterLink>
      <RouterLink v-else class="plaza-more-btn" to="/create">返回广场</RouterLink>
    </header>
    <div v-if="!showAllGames" class="rank-rail">
      <div v-for="(column, idx) in gameColumns" :key="idx" class="rank-col-card">
        <RouterLink v-for="game in column" :key="game.id"
          class="rank-row" :aria-label="`试玩${game.name}`"
          :to="builtinRunLocation(game.id)">
          <span class="plaza-app-icon sz-52"><img :src="game.icon" alt="" decoding="async" /></span>
          <span class="rank-copy">
            <span class="rank-name">{{ game.name }}</span>
            <small class="rank-sub">{{ game.genre }} · {{ gameTagline(game) }}</small>
          </span>
          <span class="open-pill" aria-hidden="true">打开</span>
        </RouterLink>
      </div>
    </div>
    <div v-else class="all-list-grid">
      <RouterLink v-for="game in games" :key="game.id"
        class="all-list-card" :aria-label="`试玩${game.name}`"
        :to="builtinRunLocation(game.id)">
        <span class="plaza-app-icon sz-52"><img :src="game.icon" alt="" decoding="async" /></span>
        <span class="rank-copy">
          <span class="rank-name">{{ game.name }}</span>
          <small class="rank-sub">{{ game.genre }} · {{ gameTagline(game) }}</small>
        </span>
        <span class="open-pill" aria-hidden="true">打开</span>
      </RouterLink>
    </div>
  </section>

  <!-- 2. 精选工具：方案 A 2×2 精致功能卡（首页）/ 精修分组列表（全部） -->
  <section v-if="!showAllGames" class="plaza-section" aria-labelledby="plaza-tools">
    <header class="plaza-head">
      <h2 id="plaza-tools">精选工具</h2>
      <RouterLink v-if="!showAllTools" class="plaza-more-btn" to="/create?kind=app">全部 <AppIcon name="arrow" :size="13" /></RouterLink>
      <RouterLink v-else class="plaza-more-btn" to="/create">返回广场</RouterLink>
    </header>

    <div v-if="!showAllTools" class="tools-grid">
      <RouterLink class="tool-card" :to="builtinRunLocation(mbti.id)">
        <span class="plaza-app-icon sz-42"><img :src="mbti.icon" alt="" decoding="async" /></span>
        <span class="tool-info">
          <span class="tool-name">{{ mbti.name }}</span>
          <small class="tool-desc">48 题人格报告</small>
        </span>
      </RouterLink>
      <button v-for="tool in mobileTools" :key="tool.id" class="tool-card" type="button"
        :aria-label="`${tool.name}（在手机 App 使用）`" @click="mobileTool = tool">
        <span class="plaza-app-icon sz-42"><img :src="tool.icon" alt="" decoding="async" /></span>
        <span class="tool-info">
          <span class="tool-name">{{ tool.name }}</span>
          <small class="tool-desc">{{ tool.desc }}</small>
        </span>
      </button>
      <RouterLink class="tool-card" to="/create?kind=wallpaper">
        <span class="plaza-app-icon sz-42"><img :src="wallpaperIcon" alt="" decoding="async" /></span>
        <span class="tool-info">
          <span class="tool-name">动态壁纸</span>
          <small class="tool-desc">浏览动态壁纸作品</small>
        </span>
      </RouterLink>
      <RouterLink v-for="tool in featuredTools" :key="tool.id" class="tool-card tool-card--community" :to="appRunLocation(tool.id, tool.version)">
        <span class="plaza-app-icon sz-42">
          <img v-if="tool.icon && !failedIcons.has(tool.icon)" :src="tool.icon" alt="" decoding="async" @error="failedIcons.add(tool.icon)" />
          <AppIcon v-else name="grid" />
        </span>
        <span class="tool-info">
          <span class="tool-name">{{ tool.title }}</span>
          <small class="tool-desc">{{ tool.desc }}</small>
        </span>
      </RouterLink>
    </div>

    <template v-else>
      <div class="all-list-grid">
        <RouterLink class="all-list-card" :to="builtinRunLocation(mbti.id)">
          <span class="plaza-app-icon sz-44"><img :src="mbti.icon" alt="" decoding="async" /></span>
          <span class="rank-copy">
            <span class="rank-name">{{ mbti.name }}</span>
            <small class="rank-sub">测评 · 48 题人格报告</small>
          </span>
          <span class="open-pill" aria-hidden="true">打开</span>
        </RouterLink>
        <button v-for="tool in mobileTools" :key="tool.id" class="all-list-card" type="button"
          :aria-label="`${tool.name}（在手机 App 使用）`" @click="mobileTool = tool">
          <span class="plaza-app-icon sz-44"><img :src="tool.icon" alt="" decoding="async" /></span>
          <span class="rank-copy">
            <span class="rank-name">{{ tool.name }}</span>
            <small class="rank-sub">{{ tool.genre }} · {{ tool.desc }}</small>
          </span>
          <span class="open-pill" aria-hidden="true">查看</span>
        </button>
        <RouterLink class="all-list-card" to="/create?kind=wallpaper">
          <span class="plaza-app-icon sz-44"><img :src="wallpaperIcon" alt="" decoding="async" /></span>
          <span class="rank-copy">
            <span class="rank-name">动态壁纸</span>
            <small class="rank-sub">壁纸 · 浏览社区作品</small>
          </span>
          <span class="open-pill" aria-hidden="true">打开</span>
        </RouterLink>
        <RouterLink v-for="tool in featuredTools" :key="tool.id" class="all-list-card" :to="appRunLocation(tool.id, tool.version)">
          <span class="plaza-app-icon sz-44">
            <img v-if="tool.icon && !failedIcons.has(tool.icon)" :src="tool.icon" alt="" decoding="async" @error="failedIcons.add(tool.icon)" />
            <AppIcon v-else name="grid" />
          </span>
          <span class="rank-copy">
            <span class="rank-name">{{ tool.title }}</span>
            <small class="rank-sub">社区精选 · {{ tool.desc }}</small>
          </span>
          <span class="open-pill" aria-hidden="true">打开</span>
        </RouterLink>
      </div>
      <p v-if="tools.loading.value" class="tools-status" role="status">正在加载社区工具…</p>
      <p v-else-if="tools.error.value" class="tools-status" role="alert">{{ tools.error.value }} <button class="btn btn-sm" type="button" @click="tools.load">重试</button></p>
    </template>
  </section>

  <ModalDialog :open="Boolean(mobileTool)" :title="mobileTool?.name ?? '在手机 App 使用'" mobile-mode="center" @close="mobileTool = null">
    <div v-if="mobileTool" class="tool-note">
      <p>{{ mobileTool.note }}</p>
      <a class="btn btn-primary" :href="ANDROID_DOWNLOAD_URL">下载 Android 版</a>
    </div>
  </ModalDialog>

  <!-- 3. 社区作品：净化双列/四列卡 -->
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
.plaza-head h2 { margin: 0; font-size: 16.5px; font-weight: 700; color: var(--ink); letter-spacing: -0.01em; }
.plaza-more-btn {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 2px 4px;
  border-radius: 999px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--community-muted);
  transition: color 0.15s;
}
.plaza-more-btn:hover { color: var(--ink); }

/* Unified 1:1 Icon Container */
.plaza-app-icon {
  position: relative;
  flex: none;
  overflow: hidden;
  display: grid;
  place-items: center;
  background: var(--surface-2);
}
.plaza-app-icon::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  box-shadow: inset 0 0 0 1px var(--community-divider);
  pointer-events: none;
}
.plaza-app-icon.sz-52 { width: 52px; height: 52px; border-radius: 14px; }
.plaza-app-icon.sz-44 { width: 44px; height: 44px; border-radius: 12px; }
.plaza-app-icon.sz-42 { width: 42px; height: 42px; border-radius: 11px; }
.plaza-app-icon img { width: 100%; height: 100%; object-fit: cover; display: block; }

/* Scheme A: 3-Row Curated Game Cards */
.rank-rail {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  align-items: start;
}
@media (width < 1180px) {
  .rank-rail { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
.rank-col-card {
  background: var(--surface);
  border: 1px solid var(--community-divider);
  border-radius: 16px;
  padding: 4px 12px;
  min-width: 0;
}
.rank-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 10px 0;
  color: var(--ink);
  text-decoration: none;
  min-width: 0;
  transition: opacity 0.14s;
}
.rank-row + .rank-row::before {
  content: '';
  position: absolute;
  top: 0;
  left: 63px;
  right: 0;
  height: 1px;
  background: var(--community-divider);
}
.rank-row:hover .rank-name { color: var(--brand); }
.rank-row:hover .open-pill { background: var(--brand-soft); color: var(--brand); }
.rank-copy { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.rank-name {
  font-size: 14.5px;
  line-height: 20px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.14s;
}
.rank-sub {
  font-size: 12px;
  line-height: 16px;
  color: var(--community-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.open-pill {
  flex: none;
  height: 30px;
  padding: 0 13px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--ink);
  font-size: 12.5px;
  font-weight: 600;
  display: inline-grid;
  place-items: center;
  transition: background 0.14s, color 0.14s;
}

/* All Games / All Tools List Grid */
.all-list-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.all-list-card {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  background: var(--surface);
  border: 1px solid var(--community-divider);
  color: var(--ink);
  text-align: left;
  text-decoration: none;
  min-width: 0;
  transition: border-color 0.15s;
}
.all-list-card:hover { border-color: var(--line-strong, var(--line)); }
.all-list-card:hover .rank-name { color: var(--brand); }
.all-list-card:hover .open-pill { background: var(--brand-soft); color: var(--brand); }

/* Scheme A: 2×2 / 4-col Compact Tool Matrix */
.tools-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}
@media (width < 1080px) {
  .tools-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
.tool-card {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 12px;
  border-radius: 16px;
  background: var(--surface);
  border: 1px solid var(--community-divider);
  color: var(--ink);
  text-align: left;
  text-decoration: none;
  min-width: 0;
  transition: border-color 0.15s, transform 0.15s;
}
.tool-card:hover { border-color: var(--line-strong, var(--line)); transform: translateY(-1px); }
.tool-card:hover .tool-name { color: var(--brand); }
.tool-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.tool-name {
  font-size: 13.5px;
  line-height: 18px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color 0.14s;
}
.tool-desc {
  font-size: 11.5px;
  line-height: 15px;
  color: var(--community-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tools-status { margin: 10px 0 0; color: var(--community-muted); font-size: 12px; }
.tool-note { display: grid; justify-items: start; gap: 14px; }
.tool-note p { margin: 0; color: var(--ink-2); }

/* Scheme A: Clean Filter Pills */
.plaza-filters { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; }
.filter-pill {
  flex: none;
  height: 32px;
  padding: 0 13px;
  border-radius: 999px;
  border: 1px solid var(--community-divider);
  background: var(--surface);
  color: var(--community-muted);
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.filter-pill:hover { color: var(--ink); }
.filter-pill[aria-pressed="true"] {
  background: var(--brand-soft);
  border-color: color-mix(in srgb, var(--brand) 28%, transparent);
  color: var(--brand);
  font-weight: 600;
}

.plaza-packs { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
.plaza-end { margin: 16px 0 6px; font-size: 12px; color: var(--community-muted); }
.plaza-pager { display: flex; align-items: center; justify-content: center; gap: 12px; margin-top: 12px; font-size: 13px; }

/* H5 Mobile Breakpoint (< 960px): Exact Scheme A Horizontal Rail + 2×2 Tool Grid */
@media (width < 960px) {
  .plaza-section { margin-bottom: 18px; }
  .plaza-head, .plaza-filters, .tools-grid, .plaza-packs, .all-list-grid, .plaza-end, .tools-status {
    padding-inline: 10px;
  }
  .plaza-head { margin-bottom: 10px; }
  .plaza-head h2 { font-size: 16px; }

  .rank-rail {
    display: flex;
    gap: 10px;
    overflow-x: auto;
    overscroll-behavior-x: contain;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
    padding-inline: 10px;
  }
  .rank-rail::-webkit-scrollbar { display: none; }
  .rank-col-card {
    flex: none;
    width: min(306px, calc(100% - 34px));
    scroll-snap-align: start;
  }

  .tools-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }
  .tool-card--community { display: none; }
  .tool-card { padding: 11px; }

  .all-list-grid { grid-template-columns: 1fr; gap: 8px; }

  .plaza-filters { flex-wrap: nowrap; overflow-x: auto; scrollbar-width: none; gap: 8px; margin-bottom: 10px; }
  .plaza-filters::-webkit-scrollbar { display: none; }
  .filter-pill { height: 30px; padding: 0 12px; font-size: 12.5px; }

  .plaza-packs { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
}
</style>
