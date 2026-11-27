<script setup lang="ts">
import { computed, ref } from 'vue';
import { ALL_TESTS, buildSoulInsight } from '../data';
import { findProfileByCode } from '../engine/scoring';
import { triggerSystemShare } from '../engine/systemShare';
import type { EvaluatedResult, TestId } from '../engine/types';

const props = defineProps<{
  results: Partial<Record<TestId, EvaluatedResult>>;
  nickname: string;
}>();

const emit = defineEmits<{
  startTest: [testId: TestId];
  openResult: [testId: TestId];
  updateNickname: [name: string];
  toast: [msg: string];
}>();

const editingName = ref(false);
const draftName = ref(props.nickname);

const completedItems = computed(() =>
  ALL_TESTS.filter((t) => Boolean(props.results[t.id])).map((t) => {
    const res = props.results[t.id]!;
    const profile = findProfileByCode(t, res.profileCode);
    return { test: t, res, profile };
  })
);

const insightText = computed(() => buildSoulInsight(props.results));

const heroEmoji = computed(
  () => completedItems.value[0]?.profile.avatarEmoji || '✨'
);

function saveName() {
  emit('updateNickname', draftName.value.trim() || '你');
  editingName.value = false;
  emit('toast', '合盘署名昵称已更新');
}

function handleStampClick(testId: TestId) {
  if (props.results[testId]) {
    emit('openResult', testId);
  } else {
    emit('startTest', testId);
  }
}

function buildTotalCardDataUrl(): string {
  const canvas = document.createElement('canvas');
  canvas.width = 1080;
  canvas.height = 1440;
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';

  ctx.fillStyle = '#18181B';
  ctx.fillRect(0, 0, 1080, 1440);
  ctx.strokeStyle = '#E4F857';
  ctx.lineWidth = 10;
  ctx.strokeRect(48, 48, 984, 1344);

  ctx.fillStyle = '#E4F857';
  ctx.font = '900 36px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('人格贩卖机 VEND·ME · 黑金灵魂收藏卡', 96, 136);

  ctx.fillStyle = '#FFFDF8';
  ctx.font = '900 72px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(`${heroEmoji.value} ${props.nickname} 的灵魂档案`, 96, 250);

  ctx.fillStyle = '#B8B4C2';
  ctx.font = '700 32px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(`已点亮 ${completedItems.value.length} / ${ALL_TESTS.length} 枚潮玩盲盒贴纸`, 96, 314);

  let y = 390;
  completedItems.value.forEach((item) => {
    ctx.fillStyle = item.test.posterBg;
    ctx.fillRect(96, y, 888, 118);
    ctx.fillStyle = '#18181B';
    ctx.font = '900 36px -apple-system,"PingFang SC",sans-serif';
    ctx.fillText(`${item.profile.avatarEmoji} ${item.test.shortTitle}：${item.profile.title}`, 132, y + 60);
    ctx.font = '700 26px ui-monospace,Menlo,monospace';
    ctx.fillText(`${item.profile.code} · ${item.profile.rarity}`, 132, y + 98);
    y += 140;
  });

  ctx.fillStyle = '#E4F857';
  ctx.font = '800 30px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('people.xiaban.xyz · 人格贩卖机 · 0元全免费', 96, 1330);

  return canvas.toDataURL('image/png');
}

function exportTotalCard() {
  if (completedItems.value.length === 0) {
    emit('toast', '先测 1 个盲盒再来晒你的灵魂收藏卡吧');
    return;
  }
  const url = buildTotalCardDataUrl();
  const a = document.createElement('a');
  a.download = `vendme-soul-card-${completedItems.value.length}.png`;
  a.href = url;
  a.click();
  emit('toast', '已保存 1080×1440 黑金灵魂总卡 PNG');
}

async function systemShareTotalCard() {
  if (completedItems.value.length === 0) {
    emit('toast', '先测 1 个盲盒再来分享你的灵魂收藏卡吧');
    return;
  }
  const tags = completedItems.value.map((i) => i.profile.title).join(' · ');
  const imgUrl = buildTotalCardDataUrl();
  const outcome = await triggerSystemShare({
    title: `${props.nickname} 的黑金灵魂总卡（${completedItems.value.length}/${ALL_TESTS.length}）`,
    text: `我的多维灵魂标签：${tags}。来人格贩卖机集齐你的 6 枚贴纸！`,
    url: window.location.href.split('#')[0],
    imageDataUrl: imgUrl,
    filename: `vendme-soul-card-${completedItems.value.length}.png`
  });
  if (outcome.message) {
    emit('toast', outcome.message);
  }
}
</script>

<template>
  <div class="archive-page">
    <header class="topbar">
      <span class="topbar-title">我的灵魂集卡册</span>
      <div class="arch-top-actions">
        <button class="sticker" style="background: var(--surface); color: var(--ink)" @click="exportTotalCard">
          存总卡
        </button>
        <button class="icon-btn arch-share-btn" aria-label="系统分享总卡" title="系统分享黑金总卡" @click="systemShareTotalCard">
          <svg viewBox="0 0 24 24">
            <path d="M12 3v12" />
            <path d="M7.5 7.5 12 3l4.5 4.5" />
            <path d="M5 13v5a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-5" />
          </svg>
        </button>
      </div>
    </header>

    <div class="arch-pad">
      <!-- 黑金潮玩收藏总卡 -->
      <section class="soul-holo">
        <div class="soul-top">
          <div class="soul-user">
            <div class="soul-ava">{{ heroEmoji }}</div>
            <div>
              <div v-if="!editingName" class="name-line">
                <b>{{ nickname }} 的灵魂卡</b>
                <button class="edit-mini" @click="draftName = nickname; editingName = true">改昵称</button>
              </div>
              <div v-else class="name-edit">
                <input v-model="draftName" maxlength="10" placeholder="合盘昵称" @keyup.enter="saveName" />
                <button class="save-mini" @click="saveName">保存</button>
              </div>
              <small>本地免费建档 · 集齐 3 枚解锁跨测评洞察</small>
            </div>
          </div>
          <span class="soul-prog num">{{ completedItems.length }} / {{ ALL_TESTS.length }}</span>
        </div>

        <div v-if="completedItems.length > 0" class="soul-tags">
          <span
            v-for="item in completedItems"
            :key="item.test.id"
            class="soul-tag"
            :style="{ background: item.test.tagColor }"
          >
            {{ item.profile.title }}
          </span>
        </div>

        <div class="soul-insight">
          <b style="color: var(--pop-lime)">✦ 跨盲盒洞察：</b>{{ insightText }}
        </div>
      </section>

      <!-- 6 格微倾潮玩贴纸墙 -->
      <div class="stamp-grid">
        <button
          v-for="(test, idx) in ALL_TESTS"
          :key="test.id"
          class="stamp-item"
          :class="{ locked: !results[test.id] }"
          :style="
            results[test.id]
              ? {
                  background: test.posterBg,
                  transform: idx % 2 === 0 ? 'rotate(-1deg)' : 'rotate(1deg)'
                }
              : {}
          "
          @click="handleStampClick(test.id)"
        >
          <div class="stamp-top">
            <span>{{ test.title }}</span>
            <span>{{ results[test.id] ? '★' : '?' }}</span>
          </div>
          <strong>{{ results[test.id]?.profileTitle || '待拆盲盒' }}</strong>
          <p>
            {{
              results[test.id]
                ? `${results[test.id]?.profileCode} · 点看海报`
                : `${test.questions.length} 题 · 去解锁 ›`
            }}
          </p>
        </button>
      </div>

      <div class="local-note">
        ✦ 所有记录免费存于本机浏览器，无需注册，收藏网址下次继续拆盲盒
      </div>
    </div>
  </div>
</template>

<style scoped>
.arch-pad {
  padding: 4px 16px 28px;
}
.arch-top-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.arch-share-btn {
  background: var(--pop-lime);
  color: #18181b;
  border-color: #18181b;
}
.soul-holo {
  padding: 18px;
  border-radius: 22px;
  background: #18181b;
  color: #fffdf8;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 var(--pop-purple);
  position: relative;
  overflow: hidden;
}
.soul-holo::before {
  content: '';
  position: absolute;
  right: -35px;
  top: -35px;
  width: 165px;
  height: 165px;
  border-radius: 50%;
  background: radial-gradient(
    circle,
    rgba(228, 248, 87, 0.32) 0%,
    rgba(255, 77, 94, 0.18) 45%,
    transparent 72%
  );
  pointer-events: none;
}
.soul-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  position: relative;
  z-index: 1;
}
.soul-user {
  display: flex;
  align-items: center;
  gap: 10px;
}
.soul-ava {
  width: 46px;
  height: 46px;
  border-radius: 14px;
  background: var(--pop-peach);
  border: 2px solid #fffdf8;
  display: grid;
  place-items: center;
  font-size: 22px;
  transform: rotate(-4deg);
  flex-shrink: 0;
}
.name-line {
  display: flex;
  align-items: center;
  gap: 6px;
}
.name-line b {
  font-size: 15.5px;
  font-weight: 900;
}
.edit-mini {
  font-size: 10px;
  padding: 2px 7px;
  border-radius: 99px;
  background: rgba(255, 255, 255, 0.16);
  color: var(--pop-lime);
  font-weight: 750;
}
.name-edit {
  display: flex;
  align-items: center;
  gap: 6px;
}
.name-edit input {
  width: 112px;
  height: 26px;
  border-radius: 8px;
  border: 1.5px solid var(--pop-lime);
  background: #282630;
  color: #fffdf8;
  padding: 0 8px;
  font-size: 12px;
}
.save-mini {
  height: 26px;
  padding: 0 9px;
  border-radius: 8px;
  background: var(--pop-lime);
  color: #18181b;
  font-size: 11px;
  font-weight: 900;
}
.soul-user small {
  display: block;
  font-size: 11px;
  color: #b8b4c2;
  margin-top: 3px;
}
.soul-prog {
  padding: 4px 10px;
  border-radius: 99px;
  background: var(--pop-lime);
  color: #18181b;
  font-size: 12px;
  font-weight: 950;
  flex-shrink: 0;
}
.soul-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 14px;
  position: relative;
  z-index: 1;
}
.soul-tag {
  padding: 5px 10px;
  border-radius: 99px;
  color: #18181b;
  font-size: 11.5px;
  font-weight: 850;
  border: 1.4px solid #18181b;
}
.soul-insight {
  margin-top: 12px;
  padding: 11px 12px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.09);
  border: 1px solid rgba(255, 255, 255, 0.14);
  font-size: 12px;
  color: #edeae3;
  line-height: 1.6;
  position: relative;
  z-index: 1;
}

.stamp-grid {
  margin-top: 14px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 11px;
}
.stamp-item {
  padding: 13px;
  border-radius: 16px;
  border: 1.8px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  text-align: left;
  color: #18181b;
}
.stamp-item:active {
  transform: translate(2px, 2px) !important;
  box-shadow: none;
}
.stamp-item.locked {
  background: var(--surface);
  color: var(--muted);
  border: 2px dashed var(--muted);
  box-shadow: none;
}
.stamp-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 10.5px;
  font-weight: 800;
  opacity: 0.85;
}
.stamp-item strong {
  display: block;
  font-size: 15px;
  font-weight: 900;
  margin-top: 8px;
}
.stamp-item p {
  font-size: 11px;
  margin-top: 3px;
  font-weight: 700;
  opacity: 0.8;
}
.local-note {
  margin-top: 14px;
  padding: 11px 13px;
  border-radius: 14px;
  background: var(--surface);
  border: 1.5px solid var(--border);
  font-size: 11.5px;
  color: var(--muted);
  font-weight: 650;
}
</style>
