<script setup lang="ts">
import { computed } from 'vue';
import { ALL_TESTS } from '../data';
import { findProfileByCode } from '../engine/scoring';
import type { EvaluatedResult, PeerPayload, TestDefinition, TestId } from '../engine/types';
import ResultPoster from '../components/ResultPoster.vue';

const props = defineProps<{
  test: TestDefinition;
  result: EvaluatedResult;
  allResults: Partial<Record<TestId, EvaluatedResult>>;
  peer?: PeerPayload;
}>();

const emit = defineEmits<{
  back: [];
  retest: [testId: TestId];
  openPoster: [];
  systemShare: [];
  openCompat: [testId: TestId, peer?: PeerPayload];
  startNext: [testId: TestId];
  gotoArchive: [];
}>();

const profile = computed(() => findProfileByCode(props.test, props.result.profileCode));

const unlockedCount = computed(
  () => ALL_TESTS.filter((t) => Boolean(props.allResults[t.id])).length
);

const nextUntested = computed(() =>
  ALL_TESTS.find((t) => t.id !== props.test.id && !props.allResults[t.id])
);
</script>

<template>
  <div class="result-page">
    <header class="topbar">
      <button class="icon-btn" aria-label="返回首页" @click="emit('back')">
        <svg viewBox="0 0 24 24"><path d="M15 18l-6-6 6-6" /></svg>
      </button>
      <span class="topbar-title">{{ test.title }}</span>
      <div class="res-top-right">
        <button class="sticker" style="background: var(--surface); color: var(--ink)" @click="emit('retest', test.id)">
          重测
        </button>
        <button class="icon-btn share-ico-btn" aria-label="系统分享" title="拉起系统分享" @click="emit('systemShare')">
          <svg viewBox="0 0 24 24">
            <path d="M12 3v12" />
            <path d="M7.5 7.5 12 3l4.5 4.5" />
            <path d="M5 13v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5" />
          </svg>
        </button>
      </div>
    </header>

    <div class="res-pad">
      <!-- 如果有好友邀请合盘，置顶高能入口 -->
      <button
        v-if="peer"
        class="peer-ready-bar"
        @click="emit('openCompat', test.id, peer)"
      >
        <span>✦ 已解锁你与 <b>{{ peer.name }}</b> 的双人合盘报告</span>
        <b>立即查看 →</b>
      </button>

      <!-- 6 大专属视觉皮肤主卡 -->
      <ResultPoster :test="test" :result="result" :profile="profile" />

      <!-- 抹茶绿天选搭子 × 樱花粉相处宿敌 -->
      <div class="match-pair">
        <div class="mp-box buddy">
          <small>★ 天选搭子</small>
          <strong>{{ profile.buddy.name }}</strong>
          <p>{{ profile.buddy.desc }}</p>
        </div>
        <div class="mp-box rival">
          <small>⚡ 相处宿敌</small>
          <strong>{{ profile.rival.name }}</strong>
          <p>{{ profile.rival.desc }}</p>
        </div>
      </div>

      <!-- 连测集卡引导 -->
      <div class="next-quest">
        <div>
          <b class="num">已集齐 {{ unlockedCount }}/{{ ALL_TESTS.length }} 枚灵魂贴纸</b>
          <small v-if="nextUntested">推荐下一个盲盒：《{{ nextUntested.title }}》</small>
          <small v-else>全部 6 枚盲盒已集齐！快去晒你的黑金总卡</small>
        </div>
        <button
          v-if="nextUntested"
          class="nq-go"
          @click="emit('startNext', nextUntested.id)"
        >
          拆下一个 ›
        </button>
        <button v-else class="nq-go" @click="emit('gotoArchive')">
          看总卡 ›
        </button>
      </div>

      <p v-if="test.disclaimer" class="res-disclaimer">{{ test.disclaimer }}</p>
    </div>

    <!-- 吸底双操作条 -->
    <div class="sticky-actions">
      <button class="btn-pop" @click="emit('openPoster')">存高清海报 ✦</button>
      <button class="btn-pop-light" @click="emit('openCompat', test.id, peer)">
        {{ peer ? '看双人合盘' : '拉 TA 合盘' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.result-page {
  min-height: 100%;
  position: relative;
}
.res-top-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.share-ico-btn {
  background: var(--pop-lime);
  color: #18181b;
  border-color: #18181b;
}
.res-pad {
  padding: 6px 16px calc(92px + env(safe-area-inset-bottom, 0px));
}
.peer-ready-bar {
  width: 100%;
  margin-bottom: 12px;
  padding: 11px 13px;
  border-radius: 14px;
  background: var(--pop-lime);
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 3px 3px 0 var(--pop-red);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 12.5px;
  font-weight: 750;
  text-align: left;
}
.peer-ready-bar b {
  font-weight: 950;
  white-space: nowrap;
}
.match-pair {
  margin-top: 12px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.mp-box {
  padding: 12px;
  border-radius: 16px;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  color: #18181b;
}
.mp-box.buddy {
  background: var(--pop-mint);
}
.mp-box.rival {
  background: var(--pop-pink);
}
.mp-box small {
  display: inline-block;
  padding: 2px 7px;
  border-radius: 99px;
  background: #fffdf8;
  border: 1.2px solid #18181b;
  font-size: 10px;
  font-weight: 850;
}
.mp-box strong {
  display: block;
  font-size: 14px;
  font-weight: 900;
  margin-top: 7px;
}
.mp-box p {
  font-size: 11.5px;
  margin-top: 3px;
  line-height: 1.45;
  font-weight: 600;
  opacity: 0.88;
}

.next-quest {
  margin-top: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  background: #18181b;
  color: #fffdf8;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 var(--pop-lime);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.next-quest b {
  font-size: 13px;
  font-weight: 850;
  color: var(--pop-lime);
  display: block;
}
.next-quest small {
  font-size: 11px;
  opacity: 0.82;
  display: block;
  margin-top: 2px;
}
.nq-go {
  flex-shrink: 0;
  height: 32px;
  padding: 0 12px;
  border-radius: 99px;
  background: var(--pop-lime);
  color: #18181b;
  font-size: 11.5px;
  font-weight: 900;
}

.res-disclaimer {
  margin-top: 12px;
  font-size: 11px;
  color: var(--muted);
  text-align: center;
  line-height: 1.5;
}

.sticky-actions {
  position: fixed;
  left: 50%;
  transform: translateX(-50%);
  width: 100%;
  max-width: 430px;
  bottom: 0;
  padding: 10px 16px calc(14px + env(safe-area-inset-bottom, 0px));
  background: linear-gradient(180deg, transparent 0%, var(--bg) 32%);
  display: grid;
  grid-template-columns: 1.15fr 1fr;
  gap: 10px;
  z-index: 20;
}
@media (min-width: 560px) {
  .sticky-actions {
    position: sticky;
    left: auto;
    transform: none;
    max-width: none;
  }
}
</style>
