<script setup lang="ts">
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, onActivated, onMounted, ref } from "vue";
import { Compass, Gamepad2, LayoutGrid, Palette, Save, Search, Sparkles, Square, Trash2, Wrench } from "lucide-vue-next";
import { NAlert, NPopconfirm, NSkeleton } from "naive-ui";
import type { ControlStore } from "../control-types.ts";
import { coverUrl, fallbackCoverUrl, iconUrl } from "../../games/catalog.ts";
import type { GameManifest } from "../../games/protocol.ts";

defineOptions({ name: 'AppsPage' });

const props = defineProps<{ store: ControlStore }>();

const isGamesMode = computed(() => !props.store.page || props.store.page === 'games');

type PrimaryTab = 'plaza' | 'builtin' | 'custom';
type CategoryFilter = 'all' | 'game' | 'tool' | 'assessment';

const primaryTab = ref<PrimaryTab>(isGamesMode.value ? 'builtin' : 'plaza');
const showFeaturedApps = computed(() => !isGamesMode.value && primaryTab.value === 'plaza');
const activeFilter = ref<CategoryFilter>('all');
const searchQuery = ref('');
const games = ref<GameManifest[]>([]);
const loading = ref(true);
const catalogError = ref("");
const folderErrors = ref<string[]>([]);
const opening = ref(false);

const primaryTabs = computed(() => {
  if (isGamesMode.value) {
    return [
      { id: 'builtin' as PrimaryTab, label: '官方游戏' },
      { id: 'custom' as PrimaryTab, label: '我的游戏' },
    ];
  }
  return [
    { id: 'plaza' as PrimaryTab, label: '精选应用' },
    { id: 'custom' as PrimaryTab, label: '我的作品' },
  ];
});

const categoryFilters: { id: CategoryFilter; label: string; icon: any }[] = [
  { id: 'all', label: '全部', icon: LayoutGrid },
  { id: 'game', label: '精选游戏', icon: Gamepad2 },
  { id: 'tool', label: '实用工具', icon: Wrench },
  { id: 'assessment', label: '心理测评', icon: Compass },
];

function getAppKind(app: GameManifest): CategoryFilter {
  if (app.kind === 'app') return app.category === 'quiz' ? 'assessment' : 'tool';
  if (app.kind === 'game') return 'game';
  if (app.id === 'mbti') return 'assessment';
  if (app.id === 'pomodoro' || app.id.includes('tool') || app.id.includes('calc')) return 'tool';
  return 'game';
}

function kindBadge(app: GameManifest): string {
  if (app.source === 'user') {
    if (app.layer === 'desktop') return '桌面层';
    if (app.kind === 'app') return app.category === 'quiz' ? '自创测评' : '自创工具';
    return '自定义';
  }
  if (app.id === 'mbti') return '测评 · 协议2';
  const kind = getAppKind(app);
  if (kind === 'assessment') return '心理测评';
  if (kind === 'tool') return '轻量工具';
  return app.orientation === 'landscape' ? '横屏游戏' : app.orientation === 'portrait' ? '竖屏游戏' : '休闲游戏';
}

const builtinApps = computed(() => games.value.filter(g => g.source !== 'user'));
const userApps = computed(() => games.value.filter(g => g.source === 'user'));
const isPlazaOrBuiltin = computed(() => primaryTab.value === 'plaza' || primaryTab.value === 'builtin');

const filteredApps = computed(() => {
  const list = isPlazaOrBuiltin.value ? builtinApps.value : userApps.value;
  const query = searchQuery.value.trim().toLowerCase();

  return list.filter(app => {
    // Category filter (Plaza only)
    if (showFeaturedApps.value && activeFilter.value !== 'all') {
      const kind = getAppKind(app);
      if (kind !== activeFilter.value) return false;
    }
    // Search query filter
    if (query) {
      const matchName = app.name.toLowerCase().includes(query);
      const matchDesc = (app.description || '').toLowerCase().includes(query);
      const matchId = app.id.toLowerCase().includes(query);
      if (!matchName && !matchDesc && !matchId) return false;
    }
    return true;
  });
});

function descriptionFor(app: GameManifest) {
  if (app.id === 'snake-arena') return '无尽成长或三分钟冲分，吃糖果长成大蛇，反击 AI。';
  if (app.id === 'mbti') return '48 道原创情境题，观察四个维度的日常偏好。约 10 分钟，支持独立原生窗口运行。';
  return app.description;
}

function onCoverError(event: Event, app: GameManifest) {
  const image = event.currentTarget as HTMLImageElement;
  const fallback = fallbackCoverUrl(app);
  if (image.getAttribute('src') !== fallback) image.src = fallback;
}

function onIconError(event: Event, app: GameManifest) {
  const image = event.currentTarget as HTMLImageElement;
  const icon = iconUrl(app);
  const cover = coverUrl(app);
  if (icon !== cover && image.getAttribute('src') === icon) image.src = cover;
  else onCoverError(event, app);
}

const message = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : typeof error === 'string' ? error : fallback;

async function loadApps() {
  loading.value = games.value.length === 0;
  catalogError.value = "";
  try {
    const catalog = await props.store.listGames();
    games.value = catalog.games;
    folderErrors.value = catalog.errors;
  } catch (error) {
    catalogError.value = message(error, "应用资料暂时无法读取。");
  } finally {
    loading.value = false;
  }
}

async function launchApp(app: GameManifest) {
  if (!props.store.desktop || opening.value) return;
  opening.value = true;
  try {
    // Older user packs still need the protocol 1 host.
    if (app.source === 'user' && app.runtimeProtocol !== 2) {
      await props.store.openGames(app.id);
      return;
    }

    // In modern Apps Plaza mode, all apps launch directly in V2 independent window container!
    await props.store.openApp(app.id, {
      title: app.name,
      orientation: app.orientation === 'landscape' || app.orientation === 'any' ? app.orientation : 'portrait',
      entry: app.entry ?? 'index.html',
    });
  } catch (error) {
    props.store.notify(message(error, "应用暂时无法打开。"), "error");
  } finally {
    opening.value = false;
  }
}

async function stopDeskGame() {
  try {
    await props.store.stopDeskGame();
  } catch (error) {
    props.store.notify(message(error, "桌面应用暂时无法停止。"), "error");
  }
}

async function removeApp(app: GameManifest) {
  try {
    await props.store.deleteUserGame(app.id);
    props.store.notify(isGamesMode.value ? `已删除游戏「${app.name}」` : `已删除作品「${app.name}」`, "success");
  } catch (error) {
    props.store.notify(message(error, "作品暂时无法删除。"), "error");
  }
  await loadApps();
}

async function imported() {
  await loadApps();
  primaryTab.value = 'custom';
}

onMounted(loadApps);
let opened = false;
onActivated(() => {
  if (opened) void loadApps();
  else opened = true;
});
</script>

<template>
  <main class="xb-page apps-page games-page" :data-testid="store.page === 'games' ? 'games-page' : 'apps-page'" :aria-label="store.page === 'games' ? '游玩' : '应用大厅'">
    <header class="xb-page-head apps-head" data-tauri-drag-region>
      <div class="xb-tabs-lg" role="tablist" :aria-label="store.page === 'games' ? '游戏分类' : '应用大厅分类'">
        <button
          v-for="tab in primaryTabs"
          :key="tab.id"
          type="button"
          role="tab"
          :aria-selected="primaryTab === tab.id"
          :data-testid="store.page === 'games' ? `games-tab-${tab.id === 'plaza' ? 'builtin' : 'custom'}` : `apps-tab-${tab.id}`"
          @click="primaryTab = tab.id"
        >
          {{ store.page === 'games' ? (tab.id === 'plaza' ? '官方游戏' : '我的游戏') : tab.label }}
        </button>
      </div>

      <div class="apps-head-actions">
        <div class="apps-search">
          <Search :size="14" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索应用或游戏..."
            class="apps-search-input"
            data-testid="apps-search-input"
          />
        </div>
        <ResourceShareActions
          :store="store"
          kind="game"
          variant="xb"
          import-label="导入作品"
          :disabled="opening"
          @imported="imported"
        />
        <button
          type="button"
          class="xb-btn-primary"
          :disabled="!store.desktop"
          data-testid="apps-create"
          @click="store.startCreation(isGamesMode || (primaryTab === 'plaza' && activeFilter === 'game') ? 'game' : 'app')"
        >
          <Sparkles :size="14" />让小伴创作
        </button>
      </div>
    </header>

    <!-- Secondary Category Chips Filter (Plaza Only) -->
    <nav v-if="showFeaturedApps" class="apps-filter-bar" aria-label="子分类筛选">
      <button
        v-for="filter in categoryFilters"
        :key="filter.id"
        type="button"
        class="filter-chip"
        :class="{ active: activeFilter === filter.id }"
        :data-testid="`apps-filter-${filter.id}`"
        @click="activeFilter = filter.id"
      >
        <component :is="filter.icon" :size="13" />
        <span>{{ filter.label }}</span>
      </button>
    </nav>

    <div class="apps-body games-body xb-scroll">
      <div v-if="store.game.desk?.active" class="app-running-actions">
        <button type="button" class="xb-btn" data-testid="apps-stop-desk" @click="stopDeskGame">
          <Square :size="13" />停止桌面游戏
        </button>
      </div>

      <section
        class="apps-section"
        :aria-label="isGamesMode ? (primaryTab === 'builtin' ? '官方游戏' : '我的游戏') : (primaryTab === 'plaza' ? '精选应用' : '我的作品')"
        :data-testid="isGamesMode ? (primaryTab === 'custom' ? 'user-games' : 'official-games') : (primaryTab === 'custom' ? 'user-apps' : 'official-apps')"
      >
        <NAlert v-if="catalogError" class="catalog-error" type="warning" :bordered="false">
          {{ catalogError }}
          <button type="button" class="xb-link" @click="loadApps">重新载入</button>
        </NAlert>
        <NAlert
          v-if="primaryTab === 'custom' && folderErrors.length"
          class="catalog-error"
          type="warning"
          :show-icon="false"
          :data-testid="isGamesMode ? 'user-game-errors' : 'user-app-errors'"
        >
          有 {{ folderErrors.length }} 个{{ isGamesMode ? '游戏' : '作品' }}文件夹无法使用：{{ folderErrors.join('；') }}
        </NAlert>

        <div v-if="loading && showFeaturedApps" class="featured-app-grid" aria-label="正在读取应用资料">
          <div v-for="item in 6" :key="item" class="featured-app-skeleton">
            <NSkeleton class="featured-app-icon" :sharp="false" />
            <NSkeleton width="64px" height="12px" :sharp="false" />
          </div>
        </div>

        <div v-else-if="loading" class="app-grid" aria-label="正在读取应用资料">
          <article v-for="item in 6" :key="item" class="app-card-skeleton">
            <NSkeleton height="142px" :sharp="false" />
            <NSkeleton text :repeat="2" />
          </article>
        </div>

        <div v-else-if="showFeaturedApps && filteredApps.length" class="featured-app-grid">
          <button
            v-for="app in filteredApps"
            :key="app.id"
            type="button"
            class="featured-app"
            :data-testid="`app-${app.id}`"
            :title="descriptionFor(app)"
            :aria-label="`启动${app.name}`"
            :aria-description="descriptionFor(app)"
            :aria-disabled="opening || !store.desktop"
            @click="launchApp(app)"
          >
            <img class="featured-app-icon" :src="iconUrl(app)" alt="" loading="lazy" @error="onIconError($event, app)" />
            <span class="featured-app-name">{{ app.name }}</span>
          </button>
        </div>

        <div v-else-if="filteredApps.length" class="app-grid">
          <div v-for="app in filteredApps" :key="app.id" class="app-card-wrapper">
            <article
              class="app-card game-card"
              role="button"
              tabindex="0"
              :data-testid="app.source === 'user' ? (isGamesMode ? `user-game-${app.id}` : `user-app-${app.id}`) : (isGamesMode ? undefined : `app-${app.id}`)"
              :aria-label="isGamesMode ? `开始${app.name}` : `启动${app.name}`"
              :aria-disabled="opening || !store.desktop"
              @click="launchApp(app)"
              @keydown.enter.prevent="launchApp(app)"
              @keydown.space.prevent="launchApp(app)"
            >
              <div class="app-cover">
                <img :src="coverUrl(app)" :alt="`${app.name}封面`" loading="lazy" @error="onCoverError($event, app)" />
                <span class="app-kind" :class="{ 'v2-badge': app.id === 'mbti' }">{{ kindBadge(app) }}</span>
              </div>
              <div class="app-content">
                <h3>{{ app.name }}</h3>
                <p>{{ descriptionFor(app) }}</p>
                <div v-if="app.source === 'user' && app.layer === 'desktop'" class="app-footnote">
                  浮在桌面上运行 · 托盘菜单可随时停止
                </div>
                <div v-if="app.save" class="app-footnote">
                  <Save :size="13" /> 进度保存在本机
                </div>
              </div>
            </article>

            <ResourceShareActions
              v-if="app.source === 'user'"
              class="app-share"
              :store="store"
              :kind="app.kind ?? 'game'"
              :id="app.id"
              :disabled="opening"
            />
            <NPopconfirm
              v-if="app.source === 'user'"
              positive-text="删除"
              negative-text="取消"
              @positive-click="removeApp(app)"
            >
              <template #trigger>
                <button
                  type="button"
                  class="app-delete-btn game-delete-btn"
                  :title="`删除这个自创${isGamesMode ? '游戏' : '作品'}`"
                  :aria-label="`删除${isGamesMode ? '游戏' : '作品'} ${app.name}`"
                  :data-testid="isGamesMode ? `delete-game-${app.id}` : `delete-app-${app.id}`"
                  @click.stop
                >
                  <Trash2 :size="13" />
                </button>
              </template>
              确定要删除{{ isGamesMode ? '自创游戏' : '作品' }}「{{ app.name }}」吗？{{ isGamesMode ? '游戏文件夹和它的存档都会被清除。' : '相关文件夹和存档都会被清除。' }}
            </NPopconfirm>
          </div>
        </div>

        <div v-else-if="!catalogError" class="xb-empty apps-empty games-empty">
          <span class="xb-empty-icon"><Sparkles :size="24" /></span>
          <p>{{ searchQuery ? '未找到符合条件的内容。' : (primaryTab === 'custom' ? (isGamesMode ? '还没有自定义游戏，可从右上角导入或让小伴创作。' : '还没有自创作品，可从右上角导入或让小伴创作。') : (isGamesMode ? '暂时没有可展示的游戏资料，请稍后重试。' : '暂时没有可展示的应用，请稍后重试。')) }}</p>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.apps-page {
  --app-edge: 0 0 0 1px var(--ui-border);
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 22px 28px 0;
}
.apps-head {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.apps-head-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.apps-search {
  position: relative;
  display: flex;
  align-items: center;
}
.apps-search .search-icon {
  position: absolute;
  left: 10px;
  color: var(--ui-text-tertiary);
  pointer-events: none;
}
.apps-search-input {
  width: 170px;
  height: 32px;
  padding: 0 10px 0 30px;
  border-radius: 8px;
  border: 1px solid var(--ui-border);
  background: var(--ui-surface);
  color: var(--ui-text-primary);
  font-size: 12.5px;
  transition: all 0.2s ease;
}
.apps-search-input:focus {
  outline: none;
  border-color: var(--ui-btn);
  width: 210px;
  box-shadow: 0 0 0 2px var(--ui-shadow-hover);
}
.apps-filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 12px;
  border-radius: 14px;
  border: 1px solid var(--ui-border);
  background: var(--ui-surface);
  color: var(--ui-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}
.filter-chip:hover {
  background: var(--ui-soft);
  color: var(--ui-text-primary);
}
.filter-chip.active {
  background: var(--ui-ink);
  color: var(--ui-bg);
  border-color: var(--ui-ink);
}
.apps-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin: 0 -4px;
  padding: 4px 4px 28px;
}
.app-running-actions {
  display: flex;
  justify-content: flex-end;
}
.apps-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.catalog-error {
  border-radius: 12px;
}
.featured-app-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(104px, 1fr));
  gap: 12px 14px;
}
.featured-app,
.featured-app-skeleton {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  min-width: 0;
  padding: 10px 6px 8px;
}
.featured-app {
  border: 0;
  border-radius: 16px;
  background: transparent;
  color: var(--ui-text-primary);
  cursor: pointer;
}
.featured-app[aria-disabled="true"] {
  cursor: default;
}
.featured-app:not([aria-disabled="true"]):hover,
.featured-app:focus-visible {
  background: var(--ui-surface-hover);
}
.featured-app-grid .featured-app-icon {
  width: 80px;
  height: 80px;
  border-radius: 18px;
}
img.featured-app-icon {
  background: var(--ui-control);
  object-fit: contain;
}
.featured-app-name {
  font-size: 13px;
  font-weight: 500;
  line-height: 1.4;
  text-align: center;
  overflow-wrap: anywhere;
}
.app-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 18px;
}
.app-card-wrapper {
  position: relative;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.app-card {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 8px 8px 4px;
  border-radius: 16px;
  background: var(--ui-surface);
  box-shadow: var(--app-edge), 0 1px 2px rgba(0, 0, 0, 0.04), 0 3px 10px rgba(0, 0, 0, 0.03);
  cursor: pointer;
  transition: transform 200ms cubic-bezier(0.2, 0.8, 0.2, 1), box-shadow 200ms;
}
.app-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 0 0 1px var(--ui-border-strong), 0 2px 4px rgba(0, 0, 0, 0.05), 0 14px 28px var(--ui-shadow-hover);
}
.app-card:focus-visible {
  outline: 2px solid var(--ui-btn);
  outline-offset: 2px;
}
.app-cover {
  position: relative;
  height: 140px;
  overflow: hidden;
  border-radius: 12px;
  background: linear-gradient(135deg, var(--ui-soft), var(--ui-control));
}
.app-cover img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
}
.app-kind {
  position: absolute;
  right: 8px;
  bottom: 8px;
  height: 20px;
  padding: 0 7px;
  display: inline-flex;
  align-items: center;
  border-radius: 6px;
  background: color-mix(in srgb, var(--ui-surface) 90%, transparent);
  color: var(--ui-text-secondary);
  font-size: 11px;
  font-weight: 500;
  backdrop-filter: blur(8px);
}
.app-kind.v2-badge {
  background: color-mix(in srgb, var(--ui-btn) 18%, var(--ui-surface));
  color: var(--ui-btn);
  font-weight: 600;
}
.app-content {
  padding: 10px 6px 8px;
}
.app-content h3 {
  margin: 0 0 4px;
  color: var(--ui-text-primary);
  font-size: 14px;
  font-weight: 600;
}
.app-content p {
  margin: 0;
  min-height: 40px;
  color: var(--ui-text-secondary);
  font-size: 12.5px;
  line-height: 1.6;
}
.app-footnote {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  color: var(--ui-ink);
  font-size: 12px;
}
.app-share {
  padding: 0 2px;
}
.app-delete-btn {
  position: absolute;
  top: 14px;
  right: 14px;
  z-index: 3;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 0;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  display: grid;
  place-items: center;
  cursor: pointer;
  opacity: 0.7;
  transition: all 0.2s;
}
.app-delete-btn:hover,
.app-delete-btn:focus-visible {
  opacity: 1;
  background: var(--ui-danger);
}
.app-card-skeleton {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px 8px 14px;
  border-radius: 16px;
  background: var(--ui-surface);
  box-shadow: var(--app-edge);
}
.app-card-skeleton :deep(.n-skeleton) {
  border-radius: 12px;
}
.apps-empty {
  padding: 56px 24px;
}
@media (max-width: 900px) {
  .app-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
@media (max-width: 580px) {
  .app-grid {
    grid-template-columns: 1fr;
  }
  .app-cover {
    height: 175px;
  }
}
</style>
