<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import type { EvaluatedResult, PeerPayload, TestDefinition } from '../engine/types';
import { evaluateTest, findProfileByCode } from '../engine/scoring';
import { localStorageAdapter } from '../engine/storage';
import ArtIcons from '../components/ArtIcons.vue';

const props = defineProps<{
  test: TestDefinition;
  peer?: PeerPayload;
}>();

const emit = defineEmits<{
  back: [];
  completed: [result: EvaluatedResult, peer?: PeerPayload];
}>();

const showLanding = ref(false);
const step = ref(0);
const answers = ref<Record<string, string>>({});
const analyzing = ref(false);
let advanceTimer: number | null = null;
let analyzeTimer: number | null = null;

function initSession() {
  if (advanceTimer) window.clearTimeout(advanceTimer);
  if (analyzeTimer) window.clearTimeout(analyzeTimer);
  analyzing.value = false;

  const draft = localStorageAdapter.getDraft(props.test.id);
  if (draft && Object.keys(draft.answers).length > 0 && draft.step < props.test.questions.length) {
    step.value = draft.step;
    answers.value = { ...draft.answers };
  } else {
    step.value = 0;
    answers.value = {};
  }

  // 如果是从好友合盘邀请链接点进来，先展示 0 门槛合盘落地挑战卡
  showLanding.value = Boolean(props.peer) && Object.keys(answers.value).length === 0;
}

watch(() => props.test.id, initSession, { immediate: true });

onBeforeUnmount(() => {
  if (advanceTimer) window.clearTimeout(advanceTimer);
  if (analyzeTimer) window.clearTimeout(analyzeTimer);
});

const total = computed(() => props.test.questions.length);
const currentQ = computed(() => props.test.questions[step.value]);
const progressPct = computed(() => Math.round(((step.value + 1) / total.value) * 100));

const microHint = computed(() => {
  const hints = props.test.microHints;
  const idx = Math.min(
    hints.length - 1,
    Math.floor((step.value / Math.max(1, total.value)) * hints.length)
  );
  return hints[idx] || '✦ 按第一直觉选择即可';
});

const peerProfile = computed(() =>
  props.peer ? findProfileByCode(props.test, props.peer.profileCode) : null
);

function pickOption(optKey: string) {
  if (analyzing.value || !currentQ.value) return;
  answers.value = { ...answers.value, [currentQ.value.id]: optKey };
  localStorageAdapter.saveDraft({
    testId: props.test.id,
    step: step.value,
    answers: answers.value,
    updatedAt: Date.now()
  });

  if (advanceTimer) window.clearTimeout(advanceTimer);
  advanceTimer = window.setTimeout(() => {
    if (step.value < total.value - 1) {
      step.value += 1;
      localStorageAdapter.saveDraft({
        testId: props.test.id,
        step: step.value,
        answers: answers.value,
        updatedAt: Date.now()
      });
    } else {
      finishTest();
    }
  }, 230);
}

function goPrev() {
  if (step.value > 0 && !analyzing.value) {
    step.value -= 1;
  }
}

function resetQuiz() {
  step.value = 0;
  answers.value = {};
  localStorageAdapter.clearDraft(props.test.id);
}

function finishTest() {
  analyzing.value = true;
  analyzeTimer = window.setTimeout(() => {
    const result = evaluateTest(props.test, answers.value);
    localStorageAdapter.clearDraft(props.test.id);
    analyzing.value = false;
    emit('completed', result, props.peer);
  }, 850);
}
</script>

<template>
  <div class="quiz-page">
    <header class="topbar">
      <button class="icon-btn" aria-label="返回" @click="emit('back')">
        <svg viewBox="0 0 24 24"><path d="M15 18l-6-6 6-6" /></svg>
      </button>
      <span class="topbar-title">{{ test.title }}</span>
      <button
        v-if="!showLanding && Object.keys(answers).length > 0"
        class="sticker"
        style="background: var(--pop-cream)"
        @click="resetQuiz"
      >
        重来
      </button>
      <span v-else style="width: 36px" />
    </header>

    <!-- 分享落地挑战卡（仅当带有好友合盘参数且未开答时出现） -->
    <div v-if="showLanding && peer && peerProfile" class="landing-wrap">
      <div class="peer-challenge-bar">
        <span>✦ <b>{{ peer.name }}</b> 邀请你测同款 · 测完秒出双人合盘</span>
      </div>
      <div class="landing-card" :style="{ background: test.cardBg }">
        <span class="sticker" style="background: #fffdf8">0元全免费 · 免登录</span>
        <div class="landing-art">
          <ArtIcons :kind="test.id" />
        </div>
        <h1>{{ test.title }}</h1>
        <p>{{ test.heroDesc }}</p>
        <div class="landing-peer-box">
          <span>{{ peerProfile.avatarEmoji }} {{ peer.name }} 的结果已锁进盲盒，等你测完同步揭晓合拍度！</span>
        </div>
      </div>
      <button class="btn-pop landing-go" @click="showLanding = false">
        立即开测（{{ total }} 题 · 约 {{ test.estimatedMinutes }} 分钟）→
      </button>
    </div>

    <!-- 沉浸答题流 -->
    <div v-else-if="currentQ" class="quiz-wrap">
      <div v-if="peer" class="peer-mini-strip">
        <span>✦ 正在和 <b>{{ peer.name }}</b> 测同款，答完直出双人合盘</span>
      </div>

      <div class="q-prog-box">
        <span class="q-num-badge num mono">
          {{ String(step + 1).padStart(2, '0') }}/{{ String(total).padStart(2, '0') }}
        </span>
        <div class="q-track">
          <div class="q-fill" :style="{ width: `${progressPct}%` }" />
        </div>
      </div>

      <div class="q-bubble">{{ microHint }}</div>

      <div class="q-card">
        <span class="q-card-tag">{{ currentQ.tag }}</span>
        <h2 class="q-title">{{ currentQ.title }}</h2>
        <p class="q-sub">{{ currentQ.sub }}</p>
      </div>

      <div class="q-opts">
        <button
          v-for="opt in currentQ.options"
          :key="opt.key"
          class="q-opt"
          :class="{ selected: answers[currentQ.id] === opt.key }"
          @click="pickOption(opt.key)"
        >
          <span class="q-opt-letter">{{ opt.key }}</span>
          <span class="q-opt-body">
            <strong>{{ opt.text }}</strong>
            <small>{{ opt.sub }}</small>
          </span>
        </button>
      </div>

      <div class="q-foot">
        <button
          class="q-prev"
          :style="{ visibility: step === 0 ? 'hidden' : 'visible' }"
          @click="goPrev"
        >
          ‹ 上一题
        </button>
        <span class="q-safe">纯免费 · 自动保存进度</span>
      </div>
    </div>

    <!-- 解析过渡层 -->
    <div v-if="analyzing" class="analyzing-layer">
      <svg class="star-spin" viewBox="0 0 80 80" aria-hidden="true">
        <path
          d="M40 4 L49 28 L76 40 L49 52 L40 76 L31 52 L4 40 L31 28 Z"
          fill="#FF4D5E"
          stroke="#18181B"
          stroke-width="3.5"
          stroke-linejoin="round"
        />
        <circle cx="40" cy="40" r="10" fill="#FFFDF8" stroke="#18181B" stroke-width="3" />
      </svg>
      <h3>正在撕开你的表层滤镜…</h3>
      <p>正在比对 {{ test.dimensions.length }} 个维度并校验稀有彩蛋</p>
    </div>
  </div>
</template>

<style scoped>
.quiz-page {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
}
.landing-wrap {
  flex: 1;
  padding: 8px 16px 28px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.peer-challenge-bar {
  padding: 10px 13px;
  border-radius: 14px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.8px solid #18181b;
  box-shadow: 2.5px 2.5px 0 #18181b;
  font-size: 12.5px;
  font-weight: 750;
}
.landing-card {
  padding: 20px 18px;
  border-radius: 22px;
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
  text-align: center;
}
.landing-art {
  margin: 18px auto 12px;
  display: grid;
  place-items: center;
}
.landing-card h1 {
  font-size: 25px;
  font-weight: 950;
}
.landing-card p {
  font-size: 13px;
  font-weight: 650;
  line-height: 1.6;
  margin-top: 8px;
  opacity: 0.9;
}
.landing-peer-box {
  margin-top: 16px;
  padding: 11px 12px;
  border-radius: 14px;
  background: #fffdf8;
  border: 1.6px solid #18181b;
  font-size: 12px;
  font-weight: 750;
  line-height: 1.5;
}
.landing-go {
  margin-top: auto;
  width: 100%;
  height: 52px;
  font-size: 15.5px;
}

.quiz-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 4px 16px calc(22px + env(safe-area-inset-bottom, 0px));
}
.peer-mini-strip {
  margin-bottom: 10px;
  padding: 7px 11px;
  border-radius: 12px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.5px solid #18181b;
  font-size: 11.5px;
  font-weight: 750;
}
.q-prog-box {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}
.q-num-badge {
  padding: 4px 10px;
  border-radius: 10px;
  background: #18181b;
  color: var(--pop-lime);
  font-size: 12px;
  font-weight: 900;
  flex-shrink: 0;
}
.q-track {
  flex: 1;
  height: 12px;
  border-radius: 99px;
  background: var(--surface);
  border: 1.8px solid var(--border);
  overflow: hidden;
  padding: 1.5px;
}
.q-fill {
  height: 100%;
  border-radius: 99px;
  background: repeating-linear-gradient(
    -45deg,
    var(--pop-red),
    var(--pop-red) 7px,
    #ff785a 7px,
    #ff785a 14px
  );
  transition: width 0.28s cubic-bezier(0.22, 1, 0.36, 1);
}
.q-bubble {
  align-self: flex-start;
  padding: 5px 11px;
  border-radius: 12px 12px 12px 2px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.5px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  font-size: 11.5px;
  font-weight: 800;
  transform: rotate(-1deg);
}
.q-card {
  margin-top: 14px;
  padding: 17px 16px;
  border-radius: 20px;
  background: var(--surface);
  border: 2px solid var(--border);
  box-shadow: var(--shadow-lg);
}
.q-card-tag {
  display: inline-block;
  padding: 3px 9px;
  border-radius: 99px;
  background: var(--pop-purple);
  color: #18181b;
  border: 1.4px solid #18181b;
  font-size: 10.5px;
  font-weight: 850;
  margin-bottom: 10px;
}
.q-title {
  font-size: 19.5px;
  font-weight: 900;
  line-height: 1.42;
  letter-spacing: -0.25px;
}
.q-sub {
  font-size: 12px;
  color: var(--muted);
  margin-top: 7px;
  font-weight: 550;
}

.q-opts {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: auto;
  padding-top: 18px;
}
.q-opt {
  width: 100%;
  min-height: 58px;
  padding: 12px 13px;
  border-radius: 16px;
  background: var(--surface);
  border: 1.8px solid var(--border);
  box-shadow: var(--shadow);
  display: flex;
  align-items: center;
  gap: 11px;
  text-align: left;
}
.q-opt:active {
  transform: translate(2px, 2px);
  box-shadow: none;
}
.q-opt-letter {
  width: 28px;
  height: 28px;
  border-radius: 10px;
  background: var(--sub);
  color: var(--ink);
  border: 1.5px solid var(--border);
  font-size: 12.5px;
  font-weight: 900;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}
.q-opt-body {
  flex: 1;
}
.q-opt-body strong {
  display: block;
  font-size: 13.5px;
  font-weight: 750;
  line-height: 1.4;
}
.q-opt-body small {
  display: block;
  font-size: 11.5px;
  color: var(--muted);
  margin-top: 2px;
  line-height: 1.35;
}
.q-opt.selected {
  background: var(--pop-lime);
  color: #18181b;
  border-color: #18181b;
  box-shadow: 3px 3px 0 var(--pop-red);
}
.q-opt.selected .q-opt-letter {
  background: #18181b;
  color: var(--pop-lime);
  border-color: #18181b;
}
.q-opt.selected small {
  color: #3f3d34;
}

.q-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 14px;
}
.q-prev {
  padding: 6px 12px;
  border-radius: 99px;
  background: var(--surface);
  border: 1.5px solid var(--border);
  font-size: 12px;
  font-weight: 750;
}
.q-safe {
  font-size: 11px;
  color: var(--muted);
  font-weight: 650;
}

.analyzing-layer {
  position: fixed;
  inset: 0;
  background: var(--pop-lime);
  color: #18181b;
  z-index: 35;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 28px;
  text-align: center;
}
@media (min-width: 560px) {
  .analyzing-layer {
    position: absolute;
  }
}
.star-spin {
  width: 78px;
  height: 78px;
  animation: spin 2s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.analyzing-layer h3 {
  font-size: 22px;
  font-weight: 950;
  margin-top: 18px;
}
.analyzing-layer p {
  font-size: 13px;
  font-weight: 700;
  margin-top: 8px;
  opacity: 0.82;
}
</style>
