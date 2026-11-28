<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  FILTERS,
  GAMES,
  filterGames,
  formatScore,
  getFeaturedGames,
  getGameById,
  type FilterKey,
  type GameItem
} from '../data/games';
import { formatRelativeTime, type GameRecord } from '../lib/storage';
import UiIcon from './UiIcon.vue';

const props = defineProps<{
  dark: boolean;
  layout: 'grid' | 'list';
  lastGameId: string | null;
  records: Record<string, GameRecord>;
  canInstallPwa: boolean;
}>();

const emit = defineEmits<{
  toggleTheme: [];
  setLayout: [layout: 'grid' | 'list'];
  preview: [game: GameItem];
  launch: [game: GameItem];
  goRecords: [];
  installPwa: [];
}>();

const activeFilter = ref<FilterKey>('all');
const heroIdx = ref(0);
const rollingId = ref<string | null>(null);

const heroes = computed(() => getFeaturedGames());
const currentHero = computed(() => heroes.value[heroIdx.value % heroes.value.length]);
const visibleGames = computed(() => filterGames(GAMES, activeFilter.value));
const lastGame = computed(() => getGameById(props.lastGameId));
const lastRecord = computed(() => (lastGame.value ? props.records[lastGame.value.id] : undefined));

function countFor(key: FilterKey): number {
  return filterGames(GAMES, key).length;
}

function recordBadge(g: GameItem): string {
  const rec = props.records[g.id];
  if (!rec) return '';
  if (rec.bestScore > 0) return formatScore(g, rec.bestScore);
  if (rec.hasSave) return '有存档';
  return '';
}

function lastSubtitle(): string {
  if (!lastGame.value || !lastRecord.value) return '上次局内进度已自动保存在本机';
  const parts: string[] = [];
  if (lastRecord.value.bestScore > 0) parts.push(`最高 ${formatScore(lastGame.value, lastRecord.value.bestScore)}`);
  else if (lastRecord.value.hasSave) parts.push('本机存档就绪');
  if (lastRecord.value.lastPlayedAt) parts.push(formatRelativeTime(lastRecord.value.lastPlayedAt));
  return parts.join(' · ') || '随时续玩';
}

// 轮播触摸左右滑动
let touchX = 0;
let touchY = 0;
function onHeroTouchStart(e: TouchEvent) {
  if (!e.touches[0]) return;
  touchX = e.touches[0].clientX;
  touchY = e.touches[0].clientY;
}
function onHeroTouchEnd(e: TouchEvent) {
  if (!touchX || !e.changedTouches[0]) return;
  const dx = e.changedTouches[0].clientX - touchX;
  const dy = e.changedTouches[0].clientY - touchY;
  touchX = 0;
  touchY = 0;
  if (Math.abs(dx) > 36 && Math.abs(dx) > Math.abs(dy) * 1.2) {
    const len = heroes.value.length;
    heroIdx.value = (heroIdx.value + (dx < 0 ? 1 : -1) + len) % len;
  }
}

function rollRandom() {
  const pool = visibleGames.value;
  if (!pool.length || rollingId.value) return;
  let steps = 6;
  const tick = () => {
    const pick = pool[Math.floor(Math.random() * pool.length)];
    rollingId.value = pick.id;
    if (--steps > 0) {
      window.setTimeout(tick, 85);
    } else {
      window.setTimeout(() => {
        rollingId.value = null;
        emit('preview', pick);
      }, 170);
    }
  };
  tick();
}
</script>

<template>
  <header class="app-topbar">
    <div class="app-brand">
      <div class="xb-mark"><UiIcon name="brand" :size="15" /></div>
      <span class="app-brand-name">小伴游戏厅</span>
    </div>
    <div class="top-actions">
      <button type="button" class="icon-btn" title="切换深浅色" aria-label="切换深浅色" @click="emit('toggleTheme')">
        <UiIcon :name="dark ? 'sun' : 'moon'" :size="19" />
      </button>
      <button type="button" class="icon-btn" title="本机战绩" aria-label="本机战绩" @click="emit('goRecords')">
        <UiIcon name="trophy" :size="19" />
      </button>
    </div>
  </header>

  <main class="hall-main">
    <!-- 继续上次浮条（仅玩过后展示） -->
    <button
      v-if="lastGame"
      type="button"
      class="resume-strip"
      @click="emit('launch', lastGame)"
    >
      <span class="resume-icon"><img :src="lastGame.icon" :alt="lastGame.name" /></span>
      <span class="resume-meta">
        <span class="resume-top"><i class="resume-dot" />继续上次 · {{ lastGame.name }}</span>
        <span class="resume-sub">{{ lastSubtitle() }}</span>
      </span>
      <span class="resume-btn"><UiIcon name="play" :size="12" />续玩</span>
    </button>

    <!-- 今日主打沉浸大片轮播 -->
    <section
      v-if="currentHero"
      class="hero-card"
      @touchstart.passive="onHeroTouchStart"
      @touchend.passive="onHeroTouchEnd"
    >
      <img class="hero-bg" :src="currentHero.cover" :alt="currentHero.name" />
      <div class="hero-scrim" />
      <div class="hero-top-row">
        <span class="hero-badge">{{ currentHero.heroBadge }}</span>
        <div class="hero-dots" role="tablist" aria-label="今日主打切换">
          <button
            v-for="(h, i) in heroes"
            :key="h.id"
            type="button"
            :class="{ active: i === heroIdx }"
            :aria-label="h.name"
            @click="heroIdx = i"
          />
        </div>
      </div>
      <div class="hero-bottom">
        <div class="hero-copy" @click="emit('preview', currentHero)">
          <h1 class="hero-title">{{ currentHero.name }}</h1>
          <p class="hero-desc">{{ currentHero.tagline }}</p>
        </div>
        <button type="button" class="hero-play" @click="emit('launch', currentHero)">
          <UiIcon name="play" :size="12" />开玩
        </button>
      </div>
    </section>

    <!-- 筛选胶囊 + 随便玩 -->
    <div class="filter-row">
      <div class="filter-rail">
        <button
          v-for="f in FILTERS"
          :key="f.key"
          type="button"
          class="chip"
          :class="{ active: activeFilter === f.key }"
          @click="activeFilter = f.key"
        >
          {{ f.label }}<span class="mini-count num">{{ countFor(f.key) }}</span>
        </button>
      </div>
      <button type="button" class="dice-btn" @click="rollRandom">
        <UiIcon name="dice" :size="14" />随便玩
      </button>
    </div>

    <!-- 标题 + 视图切换 -->
    <div class="sec-head">
      <h2 class="sec-title">
        精选游戏 <span class="sec-count num">{{ visibleGames.length }} 款 · 免登录直玩</span>
      </h2>
      <div class="layout-toggle" aria-label="切换卡片样式">
        <button
          type="button"
          :class="{ active: layout === 'grid' }"
          title="卡带双列"
          aria-label="卡带双列"
          @click="emit('setLayout', 'grid')"
        >
          <UiIcon name="grid" :size="14" />
        </button>
        <button
          type="button"
          :class="{ active: layout === 'list' }"
          title="精装列表"
          aria-label="精装列表"
          @click="emit('setLayout', 'list')"
        >
          <UiIcon name="list" :size="14" />
        </button>
      </div>
    </div>

    <!-- 双列卡带画廊 -->
    <div v-if="layout === 'grid'" class="game-grid">
      <button
        v-for="g in visibleGames"
        :key="g.id"
        type="button"
        class="cart-card"
        :class="{ 'highlight-roll': rollingId === g.id }"
        @click="emit('preview', g)"
      >
        <div class="cart-cover">
          <img :src="g.cover" :alt="g.name" loading="lazy" />
          <span class="orient-pill">
            <UiIcon :name="g.orient === 'landscape' ? 'landMini' : 'phoneMini'" :size="10" />
            {{ g.orientLabel }}
          </span>
          <span v-if="recordBadge(g)" class="record-pill num">{{ recordBadge(g) }}</span>
        </div>
        <div class="cart-body">
          <span class="cart-icon"><img :src="g.icon" alt="" loading="lazy" /></span>
          <span class="cart-text">
            <span class="cart-name">{{ g.name }}</span>
            <span class="cart-sub">{{ g.genre }} · {{ g.tagline.split('·')[0].trim() }}</span>
          </span>
        </div>
      </button>
    </div>

    <!-- 精装列表视图 -->
    <div v-else class="game-list">
      <div
        v-for="g in visibleGames"
        :key="g.id"
        class="list-row"
        role="button"
        tabindex="0"
        @click="emit('preview', g)"
        @keydown.enter="emit('preview', g)"
      >
        <span class="list-icon"><img :src="g.icon" :alt="g.name" loading="lazy" /></span>
        <span class="list-info">
          <span class="list-title-row">
            <span class="list-name">{{ g.name }}</span>
            <span class="mini-tag">{{ g.orientLabel }}</span>
          </span>
          <span class="list-desc">{{ g.tagline }}</span>
        </span>
        <button type="button" class="list-play" @click.stop="emit('launch', g)">开玩</button>
      </div>
    </div>

    <!-- PWA 添加到主屏幕轻引导（浏览器支持时出现） -->
    <div v-if="canInstallPwa" class="pwa-banner">
      <div class="pwa-copy">
        <b>添加到桌面，断网也能玩</b>
        <small>全站 8 款游戏仅 3.8 MB，全屏无地址栏启动</small>
      </div>
      <button type="button" class="pwa-btn" @click="emit('installPwa')">安装</button>
    </div>
  </main>
</template>

<style scoped>
.hall-main { padding: 6px 16px 20px; }

.resume-strip {
  width: 100%; text-align: left;
  display: flex; align-items: center; gap: 10px;
  padding: 9px 11px; border-radius: 14px;
  background: var(--surface); border: 1px solid var(--hairline);
  margin-bottom: 12px; touch-action: pan-y;
  transition: transform 0.12s;
}
.resume-strip:active { transform: scale(0.99); }
.resume-icon {
  width: 40px; height: 40px; border-radius: 10px; overflow: hidden; flex: none;
  border: 1px solid var(--hairline); background: var(--fill);
}
.resume-icon img { width: 100%; height: 100%; object-fit: cover; }
.resume-meta { flex: 1; min-width: 0; }
.resume-top { display: flex; align-items: center; gap: 6px; font-size: 13.5px; font-weight: 650; }
.resume-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); flex: none; }
.resume-sub {
  display: block; font-size: 11.5px; color: var(--ink2);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 2px;
}
.resume-btn {
  height: 30px; padding: 0 12px; border-radius: 999px; flex: none;
  background: var(--accent); color: var(--accent-ink);
  font-size: 12px; font-weight: 650; display: inline-flex; align-items: center; gap: 4px;
}

.hero-card {
  position: relative; border-radius: 20px; overflow: hidden;
  aspect-ratio: 16 / 9.6; background: #18181b;
  margin-bottom: 14px; border: 1px solid var(--hairline);
}
.hero-bg { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
.hero-scrim {
  position: absolute; inset: 0;
  background: linear-gradient(180deg, rgba(12, 12, 16, 0.12) 0%, rgba(12, 12, 16, 0.28) 42%, rgba(12, 12, 16, 0.86) 100%);
}
.hero-top-row {
  position: absolute; top: 11px; left: 12px; right: 12px; z-index: 2;
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
}
.hero-badge {
  padding: 4px 9px; border-radius: 999px;
  background: rgba(16, 16, 20, 0.56); backdrop-filter: blur(10px); -webkit-backdrop-filter: blur(10px);
  color: #fff; font-size: 11px; font-weight: 600; border: 1px solid rgba(255, 255, 255, 0.14);
}
.hero-dots { display: flex; align-items: center; gap: 5px; }
.hero-dots button {
  width: 6px; height: 6px; border-radius: 999px; background: rgba(255, 255, 255, 0.44);
  transition: width 0.2s, background 0.2s;
}
.hero-dots button.active { width: 18px; background: #fff; }
.hero-bottom {
  position: absolute; left: 14px; right: 14px; bottom: 13px; z-index: 2;
  display: flex; align-items: flex-end; justify-content: space-between; gap: 10px;
}
.hero-copy { min-width: 0; color: #fff; cursor: pointer; }
.hero-title { font-size: 20px; font-weight: 750; letter-spacing: -0.02em; line-height: 1.2; }
.hero-desc {
  font-size: 12px; color: rgba(255, 255, 255, 0.84); margin-top: 3px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.hero-play {
  height: 36px; padding: 0 14px; border-radius: 999px; flex: none;
  background: #ffffff; color: #18181b; font-size: 12.5px; font-weight: 700;
  display: inline-flex; align-items: center; gap: 5px;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.26);
}
.hero-play:active { transform: scale(0.95); }

.filter-row { display: flex; align-items: center; gap: 7px; margin-bottom: 13px; }
.filter-rail {
  flex: 1; min-width: 0; display: flex; align-items: center; gap: 6px;
  overflow-x: auto; -webkit-overflow-scrolling: touch; touch-action: pan-x;
  scrollbar-width: none; padding: 1px 0;
}
.filter-rail::-webkit-scrollbar { display: none; }
.chip {
  height: 31px; padding: 0 11px; border-radius: 999px; flex: none;
  font-size: 12.5px; font-weight: 550; color: var(--ink2); background: var(--fill);
  display: inline-flex; align-items: center; gap: 4px;
  transition: background 0.15s, color 0.15s;
}
.chip.active { background: var(--accent); color: var(--accent-ink); font-weight: 650; }
.chip .mini-count { font-size: 10.5px; opacity: 0.78; }
.dice-btn {
  height: 31px; padding: 0 11px; border-radius: 999px; flex: none;
  background: var(--surface); border: 1px solid var(--hairline);
  font-size: 12.5px; font-weight: 600; color: var(--ink);
  display: inline-flex; align-items: center; gap: 4px;
}
.dice-btn:active { transform: scale(0.95); }

.sec-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
.sec-title { font-size: 15.5px; font-weight: 700; display: flex; align-items: center; gap: 6px; }
.sec-count { font-size: 12px; color: var(--ink3); font-weight: 500; }
.layout-toggle { display: inline-flex; padding: 2px; border-radius: 8px; background: var(--fill); gap: 2px; }
.layout-toggle button {
  width: 28px; height: 26px; border-radius: 6px; display: grid; place-items: center; color: var(--ink3);
}
.layout-toggle button.active { background: var(--surface); color: var(--ink); box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08); }

.game-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.cart-card {
  background: var(--surface); border: 1px solid var(--hairline);
  border-radius: 16px; overflow: hidden; display: flex; flex-direction: column;
  text-align: left; touch-action: pan-y; transition: transform 0.14s, border-color 0.15s;
}
.cart-card:active { transform: scale(0.98); }
.cart-card.highlight-roll { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-soft); }
.cart-cover { position: relative; aspect-ratio: 16 / 10; background: var(--fill); overflow: hidden; }
.cart-cover img { width: 100%; height: 100%; object-fit: cover; }
.orient-pill {
  position: absolute; top: 7px; right: 7px; padding: 2px 6px; border-radius: 6px;
  background: rgba(16, 16, 20, 0.62); backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px);
  color: #fff; font-size: 10px; font-weight: 600; display: inline-flex; align-items: center; gap: 3px;
}
.record-pill {
  position: absolute; left: 7px; bottom: 7px; padding: 2px 6px; border-radius: 6px;
  background: rgba(16, 16, 20, 0.7); color: #fff; font-size: 10px; font-weight: 600;
}
.cart-body { padding: 9px 10px 10px; display: flex; align-items: center; gap: 8px; }
.cart-icon {
  width: 34px; height: 34px; border-radius: 9px; overflow: hidden; flex: none;
  border: 1px solid var(--hairline); background: var(--fill);
}
.cart-icon img { width: 100%; height: 100%; object-fit: cover; }
.cart-text { flex: 1; min-width: 0; }
.cart-name { display: block; font-size: 14px; font-weight: 650; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.cart-sub { display: block; font-size: 11px; color: var(--ink2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 1px; }

.game-list {
  background: var(--surface); border: 1px solid var(--hairline);
  border-radius: 16px; overflow: hidden;
}
.list-row {
  display: flex; align-items: center; gap: 11px; padding: 11px 12px;
  width: 100%; text-align: left; position: relative; cursor: pointer;
}
.list-row + .list-row::before {
  content: ""; position: absolute; top: 0; left: 73px; right: 12px; height: 0.5px; background: var(--hairline);
}
.list-icon {
  width: 50px; height: 50px; border-radius: 13px; overflow: hidden; flex: none;
  border: 1px solid var(--hairline); background: var(--fill);
}
.list-icon img { width: 100%; height: 100%; object-fit: cover; }
.list-info { flex: 1; min-width: 0; }
.list-title-row { display: flex; align-items: center; gap: 6px; }
.list-name { font-size: 14.5px; font-weight: 650; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.mini-tag {
  font-size: 10px; font-weight: 600; padding: 1px 5px; border-radius: 5px;
  background: var(--fill); color: var(--ink2); flex: none;
}
.list-desc { display: block; font-size: 12px; color: var(--ink2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 2px; }
.list-play {
  height: 30px; padding: 0 13px; border-radius: 999px; flex: none;
  background: var(--fill); color: var(--ink); font-size: 12px; font-weight: 650;
}
.list-play:hover { background: var(--accent); color: var(--accent-ink); }

.pwa-banner {
  margin-top: 14px; padding: 12px 14px; border-radius: 16px;
  background: var(--surface); border: 1px solid var(--hairline);
  display: flex; align-items: center; justify-content: space-between; gap: 10px;
}
.pwa-copy b { display: block; font-size: 13.5px; font-weight: 700; }
.pwa-copy small { display: block; font-size: 11.5px; color: var(--ink2); margin-top: 2px; }
.pwa-btn {
  height: 32px; padding: 0 14px; border-radius: 999px; flex: none;
  background: var(--accent); color: var(--accent-ink); font-size: 12.5px; font-weight: 650;
}
</style>
