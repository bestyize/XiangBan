<script setup lang="ts">
import { computed, ref } from 'vue';
import { ALL_TESTS } from '../data';
import type { EvaluatedResult, TestCategory, TestId } from '../engine/types';
import ArtIcons from '../components/ArtIcons.vue';

const props = defineProps<{
  results: Partial<Record<TestId, EvaluatedResult>>;
  dark: boolean;
}>();

const emit = defineEmits<{
  startTest: [testId: TestId];
  openResult: [testId: TestId];
  gotoArchive: [];
  toggleTheme: [];
}>();

const activeCat = ref<'all' | TestCategory>('all');

const filters: Array<{ key: 'all' | TestCategory; label: string }> = [
  { key: 'all', label: '全部 6 款' },
  { key: 'viral', label: '爆款梗' },
  { key: 'aesthetic', label: '小红书出片' },
  { key: 'relation', label: '情侣合盘' },
  { key: 'classic', label: '性格深度' }
];

const filteredTests = computed(() =>
  activeCat.value === 'all'
    ? ALL_TESTS
    : ALL_TESTS.filter((t) => t.category === activeCat.value)
);

const unlockedCount = computed(
  () => ALL_TESTS.filter((t) => Boolean(props.results[t.id])).length
);

function handleCardClick(testId: TestId) {
  if (props.results[testId]) {
    emit('openResult', testId);
  } else {
    emit('startTest', testId);
  }
}
</script>

<template>
  <div class="home-page">
    <header class="topbar">
      <div class="home-brand">
        <div class="home-logo">贩</div>
        <h2>人格贩卖机</h2>
        <small>VEND</small>
      </div>
      <div class="home-right">
        <button class="theme-mini" :aria-label="dark ? '切暖纸亮色' : '切暗夜模式'" @click="emit('toggleTheme')">
          {{ dark ? '☀️' : '🌙' }}
        </button>
        <button class="archive-pill" @click="emit('gotoArchive')">
          <span>★</span>
          <span class="num">集卡 {{ unlockedCount }}/{{ ALL_TESTS.length }}</span>
        </button>
      </div>
    </header>

    <div class="home-pad">
      <!-- 当期主推爆款盲盒大卡 -->
      <section class="hero-box">
        <svg class="hero-art" viewBox="0 0 100 100" aria-hidden="true">
          <rect x="14" y="16" width="72" height="68" rx="22" fill="#E4F857" stroke="#18181B" stroke-width="3.5" />
          <circle cx="37" cy="45" r="5" fill="#18181B" />
          <circle cx="63" cy="45" r="5" fill="#18181B" />
          <path d="M34 61 Q50 72 66 61" stroke="#18181B" stroke-width="3.8" fill="none" stroke-linecap="round" />
          <path d="M16 26 L8 18 M84 26 L92 18" stroke="#FFFDF8" stroke-width="3" stroke-linecap="round" />
          <circle cx="80" cy="76" r="11" fill="#FFB5C8" stroke="#18181B" stroke-width="3" />
          <text x="80" y="80" text-anchor="middle" font-size="11" font-weight="900" fill="#18181B">?</text>
        </svg>

        <div class="hero-stickers">
          <span class="sticker" style="background: var(--pop-lime); transform: rotate(-2deg)">
            HOT · 本周刷屏
          </span>
          <span class="sticker" style="background: #fffdf8; transform: rotate(1deg)">
            0元全免费
          </span>
        </div>

        <h1 class="hero-title">人类真实面具测试</h1>
        <p class="hero-desc">
          撕掉正经人格滤镜！测出你是「体面伪人」「摸鱼圣体」还是 1% 隐藏彩蛋款？
        </p>

        <div class="hero-foot">
          <span class="hero-meta num">20 题 · 2 分钟出图</span>
          <button class="hero-cta" @click="emit('startTest', 'sbti')">
            {{ results.sbti ? '再抽一次 →' : '0元投币开测 →' }}
          </button>
        </div>
      </section>

      <!-- 场景筛选胶囊 -->
      <div class="filter-row" role="tablist" aria-label="测评分类">
        <button
          v-for="item in filters"
          :key="item.key"
          class="f-chip"
          :class="{ active: activeCat === item.key }"
          @click="activeCat = item.key"
        >
          {{ item.label }}
        </button>
      </div>

      <!-- 双列彩色盲盒封面墙 -->
      <div class="grid-2">
        <button
          v-for="test in filteredTests"
          :key="test.id"
          class="blind-card"
          @click="handleCardClick(test.id)"
        >
          <div class="bc-cover" :style="{ background: test.cardBg }">
            <span class="bc-badge">{{ test.badge }}</span>
            <span v-if="results[test.id]" class="bc-done" title="已完成">✓</span>
            <ArtIcons :kind="test.id" />
          </div>
          <div class="bc-body">
            <div class="bc-title">{{ test.title }}</div>
            <div class="bc-sub">{{ test.subtitle }}</div>
            <div class="bc-foot">
              <span class="num">{{ test.questions.length }}题·{{ test.estimatedMinutes }}min</span>
              <span class="bc-go">{{ results[test.id] ? '看海报' : '开测' }}</span>
            </div>
          </div>
        </button>
      </div>

      <div class="free-marquee">
        <span>✦ 零付费墙 · 免登录直接出高清无水印海报</span>
        <span class="mono">FREE</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-pad {
  padding: 4px 16px 26px;
}
.home-brand {
  display: flex;
  align-items: center;
  gap: 8px;
}
.home-logo {
  width: 30px;
  height: 30px;
  border-radius: 10px;
  background: var(--pop-red);
  color: #fff;
  border: 1.8px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  transform: rotate(-5deg);
  display: grid;
  place-items: center;
  font-size: 15px;
  font-weight: 900;
}
.home-brand h2 {
  font-size: 18px;
  font-weight: 900;
  letter-spacing: -0.4px;
}
.home-brand small {
  font-size: 9px;
  font-weight: 900;
  padding: 2px 5px;
  border-radius: 5px;
  background: #18181b;
  color: var(--pop-lime);
  letter-spacing: 0.6px;
}
.home-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.theme-mini {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  background: var(--surface);
  border: 1.5px solid var(--border);
  box-shadow: var(--shadow-sm);
  display: grid;
  place-items: center;
  font-size: 14px;
}
.theme-mini:active {
  transform: translate(1.5px, 1.5px);
  box-shadow: none;
}
.archive-pill {
  height: 32px;
  padding: 0 11px;
  border-radius: 99px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.6px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  font-size: 11.5px;
  font-weight: 850;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.archive-pill:active {
  transform: translate(1.5px, 1.5px);
  box-shadow: none;
}

.hero-box {
  margin-top: 4px;
  padding: 16px;
  border-radius: 20px;
  background: linear-gradient(135deg, #ff5864 0%, #ff7a45 100%);
  color: #ffffff;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
  position: relative;
  overflow: hidden;
}
.hero-box::before {
  content: '';
  position: absolute;
  inset: 0;
  background-image: radial-gradient(rgba(255, 255, 255, 0.24) 1.2px, transparent 1.2px);
  background-size: 12px 12px;
  opacity: 0.55;
  pointer-events: none;
}
.hero-art {
  position: absolute;
  right: 10px;
  top: 14px;
  width: 92px;
  height: 92px;
  transform: rotate(6deg);
  pointer-events: none;
}
.hero-stickers {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  position: relative;
  z-index: 1;
}
.hero-title {
  font-size: 24px;
  font-weight: 950;
  line-height: 1.2;
  margin-top: 11px;
  letter-spacing: -0.5px;
  text-shadow: 2px 2px 0 rgba(24, 24, 27, 0.28);
  position: relative;
  z-index: 1;
  max-width: 74%;
}
.hero-desc {
  font-size: 12.5px;
  margin-top: 7px;
  line-height: 1.5;
  opacity: 0.96;
  font-weight: 600;
  position: relative;
  z-index: 1;
  max-width: 78%;
}
.hero-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 14px;
  position: relative;
  z-index: 1;
}
.hero-meta {
  font-size: 11px;
  font-weight: 750;
  padding: 5px 10px;
  border-radius: 99px;
  background: rgba(24, 24, 27, 0.24);
}
.hero-cta {
  height: 40px;
  padding: 0 16px;
  border-radius: 99px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.8px solid #18181b;
  box-shadow: 2.5px 2.5px 0 #18181b;
  font-size: 13.5px;
  font-weight: 900;
  display: inline-flex;
  align-items: center;
}
.hero-cta:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}

.filter-row {
  display: flex;
  gap: 7px;
  overflow-x: auto;
  scrollbar-width: none;
  padding: 15px 0 12px;
}
.filter-row::-webkit-scrollbar {
  display: none;
}
.f-chip {
  flex-shrink: 0;
  height: 32px;
  padding: 0 13px;
  border-radius: 99px;
  background: var(--surface);
  color: var(--ink);
  border: 1.5px solid var(--border);
  font-size: 12px;
  font-weight: 750;
}
.f-chip.active {
  background: #18181b;
  color: var(--pop-lime);
  box-shadow: 2px 2px 0 var(--pop-red);
}
.dark .f-chip.active {
  background: var(--pop-lime);
  color: #18181b;
  border-color: #18181b;
}

.grid-2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 11px;
}
.blind-card {
  border-radius: 18px;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  text-align: left;
  background: var(--surface);
  color: var(--ink);
}
.blind-card:active {
  transform: translate(2px, 2px);
  box-shadow: 1px 1px 0 #18181b;
}
.bc-cover {
  height: 92px;
  padding: 10px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1.6px solid #18181b;
  overflow: hidden;
}
.bc-badge {
  position: absolute;
  left: 8px;
  top: 8px;
  padding: 2px 7px;
  border-radius: 99px;
  background: #fffdf8;
  color: #18181b;
  border: 1.3px solid #18181b;
  font-size: 9.5px;
  font-weight: 850;
  z-index: 2;
  transform: rotate(-2deg);
}
.bc-done {
  position: absolute;
  right: 8px;
  top: 8px;
  width: 21px;
  height: 21px;
  border-radius: 50%;
  background: #18181b;
  color: var(--pop-lime);
  font-size: 11px;
  font-weight: 900;
  display: grid;
  place-items: center;
  z-index: 2;
}
.bc-body {
  padding: 10px 11px 11px;
  flex: 1;
  display: flex;
  flex-direction: column;
}
.bc-title {
  font-size: 14.5px;
  font-weight: 850;
  line-height: 1.25;
}
.bc-sub {
  font-size: 11px;
  color: var(--muted);
  margin-top: 4px;
  line-height: 1.4;
  flex: 1;
}
.bc-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 9px;
  font-size: 10.5px;
  font-weight: 750;
  color: var(--ink-2);
}
.bc-go {
  padding: 2px 8px;
  border-radius: 99px;
  background: var(--sub);
  border: 1px solid var(--border);
  font-size: 10px;
  font-weight: 800;
}

.free-marquee {
  margin-top: 14px;
  padding: 10px 12px;
  border-radius: 14px;
  background: #18181b;
  color: var(--pop-lime);
  border: 1.8px solid #18181b;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11.5px;
  font-weight: 750;
}
</style>
