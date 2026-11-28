<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import AuthSheet from './components/AuthSheet.vue';
import DeepLinkView from './components/DeepLinkView.vue';
import GameHall from './components/GameHall.vue';
import GamePlayer from './components/GamePlayer.vue';
import GamePreviewSheet from './components/GamePreviewSheet.vue';
import MineView from './components/MineView.vue';
import RecordsView from './components/RecordsView.vue';
import UiIcon from './components/UiIcon.vue';
import { ACCENTS, getGameById, type GameItem } from './data/games';
import { loadSavedSession, logoutSession, type AuthSession } from './lib/accountApi';
import {
  clearAllGameData,
  createBrowserStorage,
  defaultHallState,
  loadHallState,
  markGameHasSave,
  parseHashRoute,
  recordGameDuration,
  recordGameLaunch,
  recordGameScore,
  saveHallState,
  type HallStateData
} from './lib/storage';

type TabKey = 'home' | 'records' | 'mine';

const storage = createBrowserStorage();
const prefersDark =
  typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;

const state = ref<HallStateData>(loadHallState(storage, prefersDark));
const session = ref<AuthSession | null>(loadSavedSession());
const activeTab = ref<TabKey>('home');
const previewGame = ref<GameItem | null>(null);
const runningGame = ref<GameItem | null>(null);
const deeplinkGame = ref<GameItem | null>(null);
const authOpen = ref(false);
const toastText = ref('');

let toastTimer: number | undefined;
let deferredInstallPrompt: { prompt: () => Promise<void>; userChoice: Promise<{ outcome: string }> } | null = null;
const canInstallPwa = ref(false);

const isDark = computed(() => state.value.prefs.theme === 'dark');
const savedCount = computed(() =>
  Object.values(state.value.records).filter((r) => r.hasSave).length
);

function showToast(msg: string) {
  toastText.value = msg;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toastText.value = '';
  }, 2400);
}

function applyThemeTokens() {
  const root = document.documentElement;
  const dark = state.value.prefs.theme === 'dark';
  root.dataset.theme = dark ? 'dark' : 'light';
  const accent = ACCENTS.find((a) => a.id === state.value.prefs.accentId) || ACCENTS[0];
  root.style.setProperty('--accent', accent.color);
  root.style.setProperty('--accent-soft', dark ? 'rgba(255,255,255,0.12)' : accent.soft);
}

watch(
  state,
  (next) => {
    saveHallState(storage, next);
    applyThemeTokens();
  },
  { deep: true }
);

function syncFromHash() {
  const route = parseHashRoute(window.location.hash);
  if (route.kind === 'play') {
    const g = getGameById(route.gameId);
    if (g && runningGame.value?.id !== g.id) {
      deeplinkGame.value = g;
      return;
    }
  }
  deeplinkGame.value = null;
}

function toggleTheme() {
  state.value.prefs.theme = isDark.value ? 'light' : 'dark';
}

function cycleAccent() {
  const idx = (ACCENTS.findIndex((a) => a.id === state.value.prefs.accentId) + 1) % ACCENTS.length;
  const next = ACCENTS[idx];
  state.value.prefs.accentId = next.id;
  showToast(`主题强调色已切换为「${next.name}」`);
}

function setLayout(layout: 'grid' | 'list') {
  state.value.prefs.layout = layout;
}

function toggleFavorite(id: string) {
  const list = state.value.favorites;
  const idx = list.indexOf(id);
  if (idx >= 0) {
    list.splice(idx, 1);
    showToast('已取消收藏');
  } else {
    list.push(id);
    showToast('已加入「我的 → 收藏」');
  }
}

async function shareGame(game: GameItem) {
  const shareUrl = `${window.location.origin}${window.location.pathname}#/play/${game.id}`;
  const shareTitle = `小伴游戏厅 · ${game.name}`;
  if (navigator.share) {
    try {
      await navigator.share({ title: shareTitle, text: game.tagline, url: shareUrl });
      return;
    } catch {
      // Fall back to clipboard
    }
  }
  try {
    await navigator.clipboard.writeText(shareUrl);
    showToast(`已复制直达链接：${game.name}`);
  } catch {
    showToast(`链接：${shareUrl}`);
  }
}

function launchGame(game: GameItem) {
  previewGame.value = null;
  deeplinkGame.value = null;
  state.value = recordGameLaunch(state.value, game.id);
  runningGame.value = game;
}

function handleGameExit(playedSeconds: number) {
  if (runningGame.value) {
    state.value = recordGameDuration(state.value, runningGame.value.id, playedSeconds);
  }
  runningGame.value = null;
  if (window.location.hash.startsWith('#/play/')) {
    window.history.replaceState(null, '', window.location.pathname);
  }
}

function handleGameScore(gameId: string, score: number) {
  state.value = recordGameScore(state.value, gameId, score);
}

function handleSaveUpdated(gameId: string) {
  state.value = markGameHasSave(state.value, gameId, true);
}

function handleClearAll() {
  clearAllGameData(storage);
  state.value = defaultHallState(isDark.value);
  showToast('已清理当前浏览器的全部游戏存档与缓存');
}

function handleImported(count: number) {
  state.value = loadHallState(storage, isDark.value);
  showToast(`已成功恢复 ${count} 项存档与配置`);
}

function handleAuthSuccess(nextSession: AuthSession, msg: string) {
  session.value = nextSession;
  authOpen.value = false;
  showToast(msg);
}

async function handleLogout() {
  await logoutSession(session.value);
  session.value = null;
  showToast('已退出登录，本机游戏存档仍保留');
}

async function installPwa() {
  if (!deferredInstallPrompt) return;
  try {
    await deferredInstallPrompt.prompt();
    const choice = await deferredInstallPrompt.userChoice;
    if (choice.outcome === 'accepted') {
      canInstallPwa.value = false;
      showToast('已添加「小伴游戏厅」到桌面');
    }
  } catch {
    /* ignore */
  }
}

function onBeforeInstallPrompt(e: Event) {
  e.preventDefault();
  deferredInstallPrompt = e as unknown as typeof deferredInstallPrompt;
  canInstallPwa.value = true;
}

function clearDeepLink() {
  deeplinkGame.value = null;
  window.history.replaceState(null, '', window.location.pathname);
}

function switchTab(tab: TabKey) {
  activeTab.value = tab;
  window.scrollTo({ top: 0, behavior: 'instant' as ScrollBehavior });
}

onMounted(() => {
  applyThemeTokens();
  syncFromHash();
  window.addEventListener('hashchange', syncFromHash);
  window.addEventListener('beforeinstallprompt', onBeforeInstallPrompt);
});

onBeforeUnmount(() => {
  window.removeEventListener('hashchange', syncFromHash);
  window.removeEventListener('beforeinstallprompt', onBeforeInstallPrompt);
  window.clearTimeout(toastTimer);
});
</script>

<template>
  <div class="app-shell">
    <!-- 分享直达落地页 -->
    <DeepLinkView
      v-if="deeplinkGame && !runningGame"
      :game="deeplinkGame"
      @back-hall="clearDeepLink"
      @launch="launchGame"
    />

    <!-- 主站三 Tab（原生文档滚动，不锁死手机触控） -->
    <template v-else-if="!runningGame">
      <GameHall
        v-if="activeTab === 'home'"
        :dark="isDark"
        :layout="state.prefs.layout"
        :last-game-id="state.lastGameId"
        :records="state.records"
        :can-install-pwa="canInstallPwa"
        @toggle-theme="toggleTheme"
        @set-layout="setLayout"
        @preview="previewGame = $event"
        @launch="launchGame"
        @go-records="switchTab('records')"
        @install-pwa="installPwa"
      />

      <RecordsView
        v-else-if="activeTab === 'records'"
        :records="state.records"
        :user-name="session?.user.username || null"
        @launch="launchGame"
        @go-hall="switchTab('home')"
      />

      <MineView
        v-else
        :dark="isDark"
        :accent-id="state.prefs.accentId"
        :muted="state.prefs.muted"
        :rotate-guide="state.prefs.rotateGuide"
        :favorites="state.favorites"
        :records="state.records"
        :session="session"
        :storage="storage"
        @toggle-theme="toggleTheme"
        @cycle-accent="cycleAccent"
        @toggle-mute="state.prefs.muted = !state.prefs.muted"
        @toggle-rotate-guide="state.prefs.rotateGuide = !state.prefs.rotateGuide"
        @open-auth="authOpen = true"
        @logout="handleLogout"
        @go-records="switchTab('records')"
        @launch="launchGame"
        @clear-all="handleClearAll"
        @imported="handleImported"
        @toast="showToast"
      />

      <!-- 底部三项导航：大厅 / 战绩 / 我的 -->
      <nav class="bottom-nav" aria-label="主导航">
        <button
          type="button"
          class="nav-item"
          :class="{ active: activeTab === 'home' }"
          @click="switchTab('home')"
        >
          <UiIcon name="gamepad" :size="21" />
          <span>大厅</span>
        </button>
        <button
          type="button"
          class="nav-item"
          :class="{ active: activeTab === 'records' }"
          @click="switchTab('records')"
        >
          <UiIcon name="trophy" :size="21" />
          <span>战绩</span>
        </button>
        <button
          type="button"
          class="nav-item"
          :class="{ active: activeTab === 'mine' }"
          @click="switchTab('mine')"
        >
          <UiIcon name="user" :size="21" />
          <span>我的</span>
        </button>
      </nav>
    </template>

    <!-- 卡带预览抽屉 -->
    <GamePreviewSheet
      v-if="previewGame && !runningGame"
      :game="previewGame"
      :record="state.records[previewGame.id]"
      :is-favorite="state.favorites.includes(previewGame.id)"
      @close="previewGame = null"
      @toggle-fav="toggleFavorite"
      @share="shareGame"
      @launch="launchGame"
    />

    <!-- 登录注册弹窗 -->
    <AuthSheet
      v-if="authOpen && !runningGame"
      :saved-count="savedCount"
      @close="authOpen = false"
      @success="handleAuthSuccess"
      @toast="showToast"
    />

    <!-- 全屏沉浸游戏运行宿主 -->
    <GamePlayer
      v-if="runningGame"
      :game="runningGame"
      :dark="isDark"
      :muted="state.prefs.muted"
      :rotate-guide-enabled="state.prefs.rotateGuide"
      :storage="storage"
      @exit="handleGameExit"
      @score="handleGameScore"
      @save-updated="handleSaveUpdated"
      @toggle-mute="state.prefs.muted = !state.prefs.muted"
      @share="shareGame"
    />

    <!-- 轻提示 Toast -->
    <div v-if="toastText" class="toast-pill" role="status">{{ toastText }}</div>
  </div>
</template>
