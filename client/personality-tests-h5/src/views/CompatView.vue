<script setup lang="ts">
import { computed, ref } from 'vue';
import { ALL_TESTS, getTestById } from '../data';
import { computeCompatibility } from '../engine/scoring';
import { encodePeerShareUrl } from '../engine/shareCodec';
import { renderCompatPosterDataUrl } from '../engine/posterCanvas';
import { triggerSystemShare } from '../engine/systemShare';
import type { EvaluatedResult, PeerPayload, TestId } from '../engine/types';

const props = defineProps<{
  testId: TestId;
  results: Partial<Record<TestId, EvaluatedResult>>;
  peer?: PeerPayload;
  nickname: string;
}>();

const emit = defineEmits<{
  back: [];
  startTest: [testId: TestId];
  toast: [msg: string];
}>();

const selectedTestId = ref<TestId>(props.testId);

const currentTest = computed(() => getTestById(selectedTestId.value));
const myResult = computed(() => props.results[selectedTestId.value]);

// 如果没有真实 peer 传入，提供一个可切换的“示例好友”方便用户直观预览合盘效果
const demoPeerIdx = ref(0);
const demoNames = ['林小满', '阿野', '橘子汽水'];

const effectivePeer = computed<PeerPayload | null>(() => {
  if (props.peer && props.peer.testId === selectedTestId.value) {
    return props.peer;
  }
  const test = currentTest.value;
  const normals = test.profiles.filter((p) => !p.isHiddenEgg);
  const pick = normals[(demoPeerIdx.value + 1) % normals.length] ?? normals[0];
  const scores: Record<string, number> = {};
  test.dimensions.slice(0, 4).forEach((d, idx) => {
    scores[d.id] = pick.targetVector?.[d.id] ?? [76, 62, 38, 82][idx % 4];
  });
  return {
    testId: test.id,
    profileCode: pick.code,
    name: `${demoNames[demoPeerIdx.value % demoNames.length]}（示例）`,
    scores
  };
});

const report = computed(() => {
  if (!myResult.value || !effectivePeer.value) return null;
  return computeCompatibility(currentTest.value, myResult.value, effectivePeer.value);
});

async function copyInviteLink() {
  if (!myResult.value) {
    emit('toast', '先花 2 分钟测完这套题，就能生成你的专属合盘链接');
    return;
  }
  const url = encodePeerShareUrl(
    window.location.href,
    currentTest.value,
    myResult.value,
    props.nickname
  );
  try {
    await navigator.clipboard.writeText(url);
    emit('toast', '合盘邀请链接已复制！发给微信好友测完即出报告');
  } catch {
    emit('toast', `请复制链接发给好友：${url}`);
  }
}

function downloadCompatPoster() {
  if (!report.value) return;
  const url = renderCompatPosterDataUrl(currentTest.value, report.value, props.nickname);
  const a = document.createElement('a');
  a.download = `vendme-compat-${currentTest.value.id}-${report.value.score}.png`;
  a.href = url;
  a.click();
  emit('toast', '已保存 1080×1440 高清双人合照海报');
}

async function systemShareCompat() {
  if (!myResult.value || !report.value) {
    emit('toast', '先测完当前盲盒即可系统分享双人合盘');
    return;
  }
  const shareUrl = encodePeerShareUrl(
    window.location.href,
    currentTest.value,
    myResult.value,
    props.nickname
  );
  const imgUrl = renderCompatPosterDataUrl(currentTest.value, report.value, props.nickname);
  const outcome = await triggerSystemShare({
    title: `${props.nickname} × ${report.value.peerName.replace('（示例）', '')} · 合拍度 ${report.value.score}%`,
    text: `我们在「${currentTest.value.title}」测出 ${report.value.score}%（${report.value.badgeLabel}）！点链接测同款看咱俩合不合拍：`,
    url: shareUrl,
    imageDataUrl: imgUrl,
    filename: `vendme-compat-${currentTest.value.id}-${report.value.score}.png`
  });
  if (outcome.message) {
    emit('toast', outcome.message);
  }
}

function switchDemoPeer() {
  demoPeerIdx.value = (demoPeerIdx.value + 1) % 3;
}
</script>

<template>
  <div class="compat-page">
    <header class="topbar">
      <button class="icon-btn" aria-label="返回" @click="emit('back')">
        <svg viewBox="0 0 24 24"><path d="M15 18l-6-6 6-6" /></svg>
      </button>
      <span class="topbar-title">双人灵魂合盘</span>
      <button class="sticker" style="background: var(--pop-lime)" @click="copyInviteLink">
        复制链接
      </button>
    </header>

    <div class="compat-pad">
      <!-- 切换要合盘的测试 -->
      <div class="test-tabs">
        <button
          v-for="t in ALL_TESTS"
          :key="t.id"
          class="t-tab"
          :class="{ active: selectedTestId === t.id }"
          @click="selectedTestId = t.id"
        >
          {{ t.shortTitle }}
          <span v-if="results[t.id]" class="done-dot" />
        </button>
      </div>

      <!-- 尚未测过当前测试时的空态引导 -->
      <div v-if="!myResult" class="empty-box">
        <div class="empty-emoji">🧲</div>
        <h3>先解锁你的《{{ currentTest.title }}》</h3>
        <p>只需 {{ currentTest.estimatedMinutes }} 分钟测完，就能生成免登录合盘链接，发给对象、闺蜜或搭子查看你们的灵魂合拍度！</p>
        <button class="btn-pop" style="margin-top: 16px; width: 100%" @click="emit('startTest', currentTest.id)">
          立即开测（{{ currentTest.questions.length }} 题）→
        </button>
      </div>

      <!-- 已有结果：完整双人撞击报告 -->
      <template v-else-if="report">
        <div class="invite-banner">
          <span>
            ✦ <b>{{ report.peerName }}</b>
            {{ peer && peer.testId === selectedTestId ? '发来同款挑战 · 真实匹配' : '预览合盘效果 · 点右上角拉真人测' }}
          </span>
          <button
            v-if="!peer || peer.testId !== selectedTestId"
            class="switch-demo"
            @click="switchDemoPeer"
          >
            换个示例
          </button>
        </div>

        <div class="compat-stage">
          <div class="duo-row">
            <div class="duo-unit left">
              <div class="duo-ava">{{ report.myProfile.avatarEmoji }}</div>
              <b>{{ nickname }}</b>
              <small>{{ report.myProfile.title }}</small>
            </div>
            <div class="duo-badge">
              <strong class="num">{{ report.score }}%</strong>
              <span>{{ report.badgeLabel }}</span>
            </div>
            <div class="duo-unit right">
              <div class="duo-ava">{{ report.peerProfile.avatarEmoji }}</div>
              <b>{{ report.peerName.replace('（示例）', '') }}</b>
              <small>{{ report.peerProfile.title }}</small>
            </div>
          </div>
          <h3 class="compat-title">{{ report.headline }}</h3>
          <p class="compat-sub">{{ report.summary }}</p>
        </div>

        <div class="dual-axes">
          <div class="da-head">
            <span>四维碰撞坐标</span>
            <div class="da-legend">
              <span><i style="background: var(--pop-red)" />{{ nickname }}</span>
              <span><i style="background: var(--pop-blue)" />TA</span>
            </div>
          </div>
          <div v-for="ax in report.axes" :key="ax.id" class="da-item">
            <div class="da-lbl">
              <span>{{ ax.label }}</span>
              <span class="num">{{ ax.me }}% vs {{ ax.ta }}%</span>
            </div>
            <div class="da-rail">
              <span class="da-dot me" :style="{ left: `${ax.me}%` }" />
              <span class="da-dot ta" :style="{ left: `${ax.ta}%` }" />
            </div>
          </div>
        </div>

        <div class="insight-list">
          <div class="ins-card good">
            <h4>★ 3 个高频默契点</h4>
            <ul>
              <li v-for="(s, idx) in report.synergies" :key="idx">{{ s }}</li>
            </ul>
          </div>
          <div class="ins-card warn">
            <h4>⚡ 2 条相处避雷提醒</h4>
            <ul>
              <li v-for="(c, idx) in report.cautions" :key="idx">{{ c }}</li>
            </ul>
          </div>
        </div>

        <div class="compat-actions-wrap">
          <button class="btn-pop" style="width: 100%" @click="systemShareCompat">
            <svg viewBox="0 0 24 24" class="share-svg" aria-hidden="true">
              <path d="M12 3v12" />
              <path d="M7.5 7.5 12 3l4.5 4.5" />
              <path d="M5 13v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5" />
            </svg>
            <span>系统分享合照 & 邀请链接</span>
          </button>
          <div class="compat-actions">
            <button class="btn-pop-light" style="height: 42px; font-size: 13px" @click="downloadCompatPoster">存合照 PNG</button>
            <button class="btn-pop-light" style="height: 42px; font-size: 13px" @click="copyInviteLink">复制邀请链接</button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.compat-pad {
  padding: 4px 16px 28px;
}
.test-tabs {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  scrollbar-width: none;
  padding-bottom: 12px;
}
.test-tabs::-webkit-scrollbar {
  display: none;
}
.t-tab {
  flex-shrink: 0;
  height: 30px;
  padding: 0 11px;
  border-radius: 99px;
  background: var(--surface);
  color: var(--ink-2);
  border: 1.5px solid var(--border);
  font-size: 11.5px;
  font-weight: 800;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.t-tab.active {
  background: #18181b;
  color: var(--pop-lime);
  box-shadow: 2px 2px 0 var(--pop-red);
}
.done-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--pop-mint);
}

.empty-box {
  margin-top: 12px;
  padding: 26px 18px;
  border-radius: 22px;
  background: var(--surface);
  border: 2px solid var(--border);
  box-shadow: var(--shadow-lg);
  text-align: center;
}
.empty-emoji {
  font-size: 42px;
  margin-bottom: 10px;
}
.empty-box h3 {
  font-size: 18px;
  font-weight: 900;
}
.empty-box p {
  font-size: 12.5px;
  color: var(--muted);
  line-height: 1.6;
  margin-top: 8px;
}

.invite-banner {
  padding: 9px 12px;
  border-radius: 14px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.8px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 11.5px;
  font-weight: 750;
  margin-bottom: 12px;
}
.switch-demo {
  flex-shrink: 0;
  padding: 3px 8px;
  border-radius: 99px;
  background: #18181b;
  color: var(--pop-lime);
  font-size: 10.5px;
  font-weight: 800;
}

.compat-stage {
  padding: 16px 14px;
  border-radius: 22px;
  background: var(--surface);
  border: 2px solid var(--border);
  box-shadow: var(--shadow-lg);
  text-align: center;
}
.duo-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.duo-unit {
  flex: 1;
  padding: 12px 6px;
  border-radius: 16px;
  border: 1.8px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  color: #18181b;
  min-width: 0;
}
.duo-unit.left {
  background: var(--pop-peach);
  transform: rotate(-1.5deg);
}
.duo-unit.right {
  background: var(--pop-blue);
  transform: rotate(1.5deg);
}
.duo-ava {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  margin: 0 auto 6px;
  display: grid;
  place-items: center;
  font-size: 22px;
  background: #fffdf8;
  border: 1.8px solid #18181b;
}
.duo-unit b {
  display: block;
  font-size: 13px;
  font-weight: 900;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.duo-unit small {
  display: block;
  font-size: 10.5px;
  font-weight: 750;
  margin-top: 2px;
  opacity: 0.85;
}
.duo-badge {
  width: 76px;
  height: 76px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--pop-red);
  color: #fffdf8;
  border: 2px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transform: rotate(-6deg);
}
.duo-badge strong {
  font-size: 25px;
  font-weight: 950;
  line-height: 1;
}
.duo-badge span {
  font-size: 9.5px;
  font-weight: 850;
  margin-top: 2px;
}
.compat-title {
  font-size: 17.5px;
  font-weight: 900;
  margin-top: 15px;
}
.compat-sub {
  font-size: 12.5px;
  color: var(--muted);
  margin-top: 5px;
  line-height: 1.5;
  font-weight: 550;
}

.dual-axes {
  margin-top: 12px;
  padding: 14px;
  border-radius: 18px;
  background: var(--surface);
  border: 1.8px solid var(--border);
  box-shadow: var(--shadow);
}
.da-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  font-weight: 850;
  margin-bottom: 12px;
}
.da-legend {
  display: flex;
  gap: 10px;
  font-size: 11px;
  font-weight: 750;
}
.da-legend span {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.da-legend i {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  border: 1.2px solid #18181b;
  display: inline-block;
}
.da-item + .da-item {
  margin-top: 12px;
}
.da-lbl {
  display: flex;
  justify-content: space-between;
  font-size: 11.5px;
  font-weight: 750;
  margin-bottom: 6px;
}
.da-rail {
  height: 8px;
  border-radius: 99px;
  background: var(--sub);
  border: 1.3px solid var(--border);
  position: relative;
}
.da-dot {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 15px;
  height: 15px;
  border-radius: 50%;
  border: 2px solid #18181b;
}
.da-dot.me {
  background: var(--pop-red);
  z-index: 2;
}
.da-dot.ta {
  background: var(--pop-blue);
  z-index: 1;
}

.insight-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ins-card {
  padding: 13px 14px;
  border-radius: 16px;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  color: #18181b;
}
.ins-card.good {
  background: var(--pop-mint);
}
.ins-card.warn {
  background: var(--pop-pink);
}
.ins-card h4 {
  font-size: 13.5px;
  font-weight: 900;
  margin-bottom: 6px;
}
.ins-card ul {
  margin: 0;
  padding-left: 16px;
  font-size: 12px;
  line-height: 1.65;
  font-weight: 650;
}

.compat-actions-wrap {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.compat-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.share-svg {
  width: 17px;
  height: 17px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}
</style>
