<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { getTestById } from './data';
import {
  AccountApiError,
  getMe,
  logoutSession,
  refreshSession,
  type AuthSessionResponse
} from './engine/accountApi';
import { findProfileByCode } from './engine/scoring';
import {
  buildHashPath,
  encodePeerShareUrl,
  parseHashRoute,
  type RouteView
} from './engine/shareCodec';
import {
  localStorageAdapter,
  type QuizDraft,
  type StoredAuthSession,
  type UserPrefs
} from './engine/storage';
import { triggerSystemShare } from './engine/systemShare';
import type { EvaluatedResult, PeerPayload, TestId } from './engine/types';
import HomeView from './views/HomeView.vue';
import QuizView from './views/QuizView.vue';
import ResultView from './views/ResultView.vue';
import CompatView from './views/CompatView.vue';
import ArchiveView from './views/ArchiveView.vue';
import MineView from './views/MineView.vue';
import AuthView from './views/AuthView.vue';
import PosterSheet from './components/PosterSheet.vue';

const route = ref<RouteView>({ name: 'home' });
const authSession = ref<StoredAuthSession | null>(localStorageAdapter.getAuthSession());
const currentUid = computed(() => authSession.value?.user.uid);
const results = ref<Partial<Record<TestId, EvaluatedResult>>>(
  localStorageAdapter.getResults(authSession.value?.user.uid)
);
const drafts = ref<QuizDraft[]>(localStorageAdapter.listDrafts());
const prefs = ref<UserPrefs>(localStorageAdapter.getPrefs());
const compatCount = ref<number>(localStorageAdapter.getCompatCount());
const guestNickname = ref<string>(localStorageAdapter.getNickname());
const nickname = computed(() => authSession.value?.user.username || guestNickname.value);
const dark = ref<boolean>(localStorageAdapter.getTheme() === 'dark');
const posterOpen = ref(false);
const toastMsg = ref('');
const scrollEl = ref<HTMLElement | null>(null);
let toastTimer: number | null = null;

function applyThemeClass() {
  document.documentElement.classList.toggle('dark', dark.value);
}

function showToast(msg: string) {
  toastMsg.value = msg;
  if (toastTimer) window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toastMsg.value = '';
  }, 2600);
}

function refreshDrafts() {
  drafts.value = localStorageAdapter.listDrafts();
}

function navigate(next: RouteView) {
  route.value = next;
  if (next.name === 'mine') {
    refreshDrafts();
  }
  const targetHash = buildHashPath(next);
  if (window.location.hash !== targetHash) {
    window.location.hash = targetHash;
  }
  window.scrollTo({ top: 0 });
  if (scrollEl.value) {
    scrollEl.value.scrollTo({ top: 0 });
  }
}

function syncFromHash() {
  const parsed = parseHashRoute(window.location.hash, (id) => getTestById(id));
  // 如果进入 result 但本地还没测过，自动重定向到该测试开测页
  if (parsed.name === 'result' && !results.value[parsed.testId]) {
    route.value = { name: 'quiz', testId: parsed.testId, peer: parsed.peer };
    return;
  }
  if (parsed.name === 'mine') {
    refreshDrafts();
  }
  route.value = parsed;
}

async function hydrateAuthSession() {
  const stored = authSession.value;
  if (!stored) return;

  let accessToken = stored.accessToken;
  let refreshToken = stored.refreshToken;
  let expiresAt = stored.expiresAt;

  try {
    if (expiresAt <= Date.now() + 60_000) {
      const tokens = await refreshSession(refreshToken);
      accessToken = tokens.access_token;
      refreshToken = tokens.refresh_token;
      expiresAt = Date.now() + tokens.expires_in * 1000;
    }
    const freshUser = await getMe(accessToken);
    const updated: StoredAuthSession = {
      accessToken,
      refreshToken,
      expiresAt,
      user: freshUser
    };
    authSession.value = updated;
    localStorageAdapter.saveAuthSession(updated);
    localStorageAdapter.mergeGuestResultsToAccount(freshUser.uid);
    results.value = localStorageAdapter.getResults(freshUser.uid);
  } catch (e) {
    if (e instanceof AccountApiError && (e.status === 401 || e.status === 403)) {
      authSession.value = null;
      localStorageAdapter.clearAuthSession();
      results.value = localStorageAdapter.getResults();
    }
  }
}

onMounted(() => {
  applyThemeClass();
  syncFromHash();
  window.addEventListener('hashchange', syncFromHash);
  void hydrateAuthSession();
});

onBeforeUnmount(() => {
  window.removeEventListener('hashchange', syncFromHash);
  if (toastTimer) window.clearTimeout(toastTimer);
});

function toggleTheme() {
  dark.value = !dark.value;
  localStorageAdapter.saveTheme(dark.value ? 'dark' : 'light');
  applyThemeClass();
}

function handleUpdatePrefs(partial: Partial<UserPrefs>) {
  prefs.value = localStorageAdapter.savePrefs(partial);
  if (partial.autoAdvance !== undefined) {
    showToast(partial.autoAdvance ? '已开启选完自动切下一题' : '已切换为手动点击下一题');
  } else if (partial.defaultPosterRatio) {
    showToast(
      `默认海报比例已切换为 ${partial.defaultPosterRatio === '916' ? '9:16 朋友圈' : '3:4 小红书'}`
    );
  }
}

function handleAuthenticated(session: AuthSessionResponse) {
  const stored: StoredAuthSession = {
    accessToken: session.access_token,
    refreshToken: session.refresh_token,
    expiresAt: Date.now() + session.expires_in * 1000,
    user: session.user
  };
  authSession.value = stored;
  localStorageAdapter.saveAuthSession(stored);
  const merged = localStorageAdapter.mergeGuestResultsToAccount(session.user.uid);
  results.value = localStorageAdapter.getResults(session.user.uid);

  const welcome = session.is_new_user
    ? `欢迎加入相伴，${session.user.username} (#${session.user.account_id})！`
    : `欢迎回来，${session.user.username}！`;
  showToast(merged > 0 ? `${welcome} 已关联 ${merged} 枚贴纸` : welcome);
  navigate({ name: 'mine' });
}

async function handleLogout() {
  const token = authSession.value?.refreshToken;
  authSession.value = null;
  localStorageAdapter.clearAuthSession();
  results.value = localStorageAdapter.getResults();
  if (token) {
    void logoutSession(token);
  }
  showToast('已退出相伴账号，本机已测贴纸仍保留');
}

function startTest(testId: TestId, clearOldDraft = false) {
  if (clearOldDraft) {
    localStorageAdapter.clearDraft(testId);
    refreshDrafts();
  }
  navigate({ name: 'quiz', testId });
}

function openResult(testId: TestId, peer?: PeerPayload) {
  if (!results.value[testId]) {
    navigate({ name: 'quiz', testId, peer });
    return;
  }
  navigate({ name: 'result', testId, peer });
}

function openCompat(testId?: TestId, peer?: PeerPayload) {
  const picked: TestId =
    testId ||
    (Object.keys(results.value)[0] as TestId | undefined) ||
    'animal';
  if (peer) {
    compatCount.value = localStorageAdapter.incrementCompatCount();
  }
  navigate({ name: 'compat', testId: picked, peer });
}

function handleQuizCompleted(result: EvaluatedResult, peer?: PeerPayload) {
  localStorageAdapter.saveResult(result, currentUid.value);
  results.value = { ...results.value, [result.testId]: result };
  refreshDrafts();
  if (result.isHiddenEgg) {
    showToast(`🎉 触发 1% 隐藏彩蛋：${result.profileTitle}！`);
  }
  navigate({ name: 'result', testId: result.testId, peer });
}

function handleUpdateNickname(name: string) {
  guestNickname.value = name;
  localStorageAdapter.saveNickname(name);
}

const activeTest = computed(() => {
  if (route.value.name === 'quiz' || route.value.name === 'result' || route.value.name === 'compat') {
    return getTestById(route.value.testId);
  }
  return getTestById('sbti');
});

const activeResult = computed(() => results.value[activeTest.value.id]);
const activeProfile = computed(() =>
  activeResult.value ? findProfileByCode(activeTest.value, activeResult.value.profileCode) : null
);

const showBottomTabs = computed(
  () =>
    route.value.name === 'home' ||
    route.value.name === 'compat' ||
    route.value.name === 'archive' ||
    route.value.name === 'mine'
);

const localStickerCount = computed(() => Object.keys(results.value).length);

async function copyCurrentShareLink() {
  if (!activeResult.value) return;
  const url = encodePeerShareUrl(
    window.location.href,
    activeTest.value,
    activeResult.value,
    nickname.value
  );
  compatCount.value = localStorageAdapter.incrementCompatCount();
  try {
    await navigator.clipboard.writeText(url);
    showToast('合盘链接已复制！发给微信好友测完即看合拍度');
  } catch {
    showToast('链接已生成，可从浏览器地址栏复制分享');
  }
}

async function handleResultSystemShare(imageDataUrl?: string, filename?: string) {
  if (!activeResult.value || !activeProfile.value) return;
  const url = encodePeerShareUrl(
    window.location.href,
    activeTest.value,
    activeResult.value,
    nickname.value
  );
  const outcome = await triggerSystemShare({
    title: `我在「${activeTest.value.title}」测出：${activeProfile.value.title}`,
    text: `${activeProfile.value.avatarEmoji} 我是「${activeProfile.value.title}」(${activeProfile.value.rarity})：“${activeProfile.value.quote}” 快来测同款看咱俩合拍度！`,
    url,
    imageDataUrl,
    filename
  });
  if (outcome.message) {
    showToast(outcome.message);
  }
}
</script>

<template>
  <div class="app-stage">
    <div class="app-shell">
      <main ref="scrollEl" class="main-scroll" :class="{ 'has-tabbar': showBottomTabs }">
        <HomeView
          v-if="route.name === 'home'"
          :results="results"
          :dark="dark"
          @start-test="(id) => startTest(id, false)"
          @open-result="(id) => openResult(id)"
          @goto-archive="navigate({ name: 'archive' })"
          @toggle-theme="toggleTheme"
        />

        <QuizView
          v-else-if="route.name === 'quiz'"
          :test="activeTest"
          :peer="route.peer"
          :auto-advance="prefs.autoAdvance"
          @back="refreshDrafts(); navigate({ name: 'home' })"
          @completed="handleQuizCompleted"
        />

        <ResultView
          v-else-if="route.name === 'result' && activeResult"
          :test="activeTest"
          :result="activeResult"
          :all-results="results"
          :peer="route.peer"
          @back="navigate({ name: 'home' })"
          @retest="(id) => startTest(id, true)"
          @open-poster="posterOpen = true"
          @system-share="handleResultSystemShare()"
          @open-compat="(id, peer) => openCompat(id, peer)"
          @start-next="(id) => startTest(id, false)"
          @goto-archive="navigate({ name: 'archive' })"
        />

        <CompatView
          v-else-if="route.name === 'compat'"
          :test-id="route.testId"
          :results="results"
          :peer="route.peer"
          :nickname="nickname"
          @back="navigate({ name: 'home' })"
          @start-test="(id) => startTest(id, false)"
          @toast="showToast"
        />

        <ArchiveView
          v-else-if="route.name === 'archive'"
          :results="results"
          :nickname="nickname"
          @start-test="(id) => startTest(id, false)"
          @open-result="(id) => openResult(id)"
          @update-nickname="handleUpdateNickname"
          @toast="showToast"
        />

        <MineView
          v-else-if="route.name === 'mine'"
          :user="authSession?.user || null"
          :access-token="authSession?.accessToken || null"
          :results="results"
          :drafts="drafts"
          :prefs="prefs"
          :dark="dark"
          :compat-count="compatCount"
          @goto-auth="(m) => navigate({ name: 'auth', mode: m })"
          @resume-draft="(id) => startTest(id, false)"
          @toggle-theme="toggleTheme"
          @update-prefs="handleUpdatePrefs"
          @logout="handleLogout"
          @toast="showToast"
        />

        <AuthView
          v-else-if="route.name === 'auth'"
          :initial-mode="route.mode"
          :local-sticker-count="localStickerCount"
          @back="navigate({ name: 'mine' })"
          @authenticated="handleAuthenticated"
          @toast="showToast"
        />
      </main>

      <!-- 底部四栏潮玩通栏导航 -->
      <nav v-if="showBottomTabs" class="tabbar" aria-label="主导航">
        <button
          class="tab-item"
          :class="{ active: route.name === 'home' }"
          @click="navigate({ name: 'home' })"
        >
          <svg viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="9" />
            <polygon points="10 10 16 8 14 14 8 16 10 10" />
          </svg>
          <span>发现</span>
        </button>

        <button
          class="tab-item"
          :class="{ active: route.name === 'compat' }"
          @click="openCompat()"
        >
          <svg viewBox="0 0 24 24">
            <path d="M16 11a4 4 0 1 0-8 0" />
            <path d="M4 20c0-3.3 3.6-5 8-5s8 1.7 8 5" />
            <circle cx="18" cy="7" r="2.5" />
          </svg>
          <span>合盘</span>
        </button>

        <button
          class="tab-item"
          :class="{ active: route.name === 'archive' }"
          @click="navigate({ name: 'archive' })"
        >
          <svg viewBox="0 0 24 24">
            <rect x="4" y="4" width="16" height="16" rx="4" />
            <path d="M8 10h8M8 14h5" />
          </svg>
          <span>集卡</span>
        </button>

        <button
          class="tab-item"
          :class="{ active: route.name === 'mine' }"
          @click="navigate({ name: 'mine' })"
        >
          <svg viewBox="0 0 24 24">
            <circle cx="12" cy="8" r="4" />
            <path d="M5 20c0-3.6 3.1-6 7-6s7 2.4 7 6" />
          </svg>
          <span>我的</span>
        </button>
      </nav>

      <!-- 高清海报保存 BottomSheet -->
      <PosterSheet
        v-if="activeResult && activeProfile"
        :open="posterOpen"
        :test="activeTest"
        :result="activeResult"
        :profile="activeProfile"
        :default-ratio="prefs.defaultPosterRatio"
        @close="posterOpen = false"
        @copy-link="copyCurrentShareLink"
        @system-share="(img, file) => handleResultSystemShare(img, file)"
        @downloaded="(msg) => showToast(msg)"
      />
    </div>

    <Transition name="fade-up">
      <div v-if="toastMsg" class="pop-toast">{{ toastMsg }}</div>
    </Transition>
  </div>
</template>
