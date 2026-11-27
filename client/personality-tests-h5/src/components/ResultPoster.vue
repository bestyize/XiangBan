<script setup lang="ts">
import type { EvaluatedResult, ResultProfile, TestDefinition } from '../engine/types';
import ArtIcons from './ArtIcons.vue';

defineProps<{
  test: TestDefinition;
  result: EvaluatedResult;
  profile: ResultProfile;
}>();
</script>

<template>
  <!-- 皮肤 1：人类真实面具（SBTI 荧光黄大字报） -->
  <div v-if="test.id === 'sbti'" class="poster-sbti">
    <div class="sbti-top">
      <span>VEND·ME · REAL MASK</span>
      <span>{{ profile.isHiddenEgg ? '★ 1% 隐藏彩蛋' : 'SBTI 毒舌解构版' }}</span>
    </div>
    <div class="sbti-stamp">{{ profile.rarity }}</div>
    <div class="sbti-code mono">{{ profile.code }}</div>
    <div class="sbti-name">
      {{ profile.avatarEmoji }} {{ profile.title }}
      <span v-if="profile.extra?.stampText" class="sticker" style="background: #fffdf8">
        {{ profile.extra.stampText }}
      </span>
    </div>
    <div class="sbti-quote">“{{ profile.quote }}”</div>
    <div class="sbti-bars">
      <div v-for="dim in test.dimensions" :key="dim.id" class="sbti-bar-item">
        <div class="sbti-bar-top">
          <span>{{ dim.label }}</span>
          <span class="num">{{ result.scores[dim.id] ?? 50 }}%</span>
        </div>
        <div class="sbti-track">
          <i
            :class="{ hot: dim.hotWhenHigh && (result.scores[dim.id] ?? 50) >= 68 }"
            :style="{ width: `${result.scores[dim.id] ?? 50}%` }"
          />
        </div>
      </div>
    </div>
    <div v-if="profile.extra?.eggHint" class="sbti-egg">
      <span>🎁 {{ profile.extra.eggHint }}</span>
    </div>
  </div>

  <!-- 皮肤 2：人格双色主导色（Pantone 潮玩艺术色卡） -->
  <div v-else-if="test.id === 'pantone'" class="poster-pantone">
    <div
      class="pantone-swatch"
      :style="{
        background:
          profile.extra?.swatchGradient ||
          'linear-gradient(145deg,#FFB085 0%,#F78FA7 48%,#2B3A67 100%)'
      }"
    >
      <div class="pantone-orb" />
      <div class="pantone-top">
        <span>VEND·ME COLOR SYSTEM</span>
        <span class="mono">{{ profile.code }}</span>
      </div>
      <div class="pantone-center">
        <h3>{{ profile.title }}</h3>
        <p>{{ profile.subtitle }} · {{ profile.rarity }}</p>
      </div>
    </div>
    <div class="pantone-meta">
      <div v-if="profile.extra?.surfaceColor && profile.extra?.soulColor" class="pantone-codes">
        <div class="p-chip">
          <span class="p-dot" :style="{ background: profile.extra.surfaceColor.hex }" />
          <div>
            <b>表层色 {{ profile.extra.surfaceColor.pct }}%</b>
            <small>{{ profile.extra.surfaceColor.name }} {{ profile.extra.surfaceColor.hex }}</small>
          </div>
        </div>
        <div class="p-chip">
          <span class="p-dot" :style="{ background: profile.extra.soulColor.hex }" />
          <div>
            <b>灵魂色 {{ profile.extra.soulColor.pct }}%</b>
            <small>{{ profile.extra.soulColor.name }} {{ profile.extra.soulColor.hex }}</small>
          </div>
        </div>
      </div>
      <p class="pantone-poem">“{{ profile.quote }}”</p>
    </div>
  </div>

  <!-- 皮肤 3：动物塑深度版（波普暖橘画框） -->
  <div v-else-if="test.id === 'animal'" class="poster-animal">
    <div class="animal-top">
      <span>ANIMAL BLIND BOX</span>
      <span class="sticker" style="background: var(--pop-lime)">{{ profile.rarity }}</span>
    </div>
    <div class="animal-frame">
      <ArtIcons kind="animal-hero" :animal-key="profile.extra?.animalKey" />
      <div class="animal-name">{{ profile.title }}</div>
      <div class="animal-en mono">{{ profile.extra?.animalEn || profile.code }}</div>
    </div>
    <div class="animal-pills">
      <span v-for="pill in profile.extra?.animalPills || profile.tags" :key="pill">{{ pill }}</span>
    </div>
    <div class="animal-guide">
      <b>{{ profile.avatarEmoji }} 饲养须知：</b>{{ profile.quote }}
      <template v-if="profile.extra?.feedingGuide"> {{ profile.extra.feedingGuide }}</template>
    </div>
  </div>

  <!-- 皮肤 4：避雷说明书（草莓粉 × 警示带） -->
  <div v-else-if="test.id === 'manual'" class="poster-manual">
    <div class="manual-stripe" />
    <div class="manual-inner">
      <div class="manual-head">
        <div>
          <h3>{{ profile.title }} · 说明书</h3>
          <span class="mono" style="font-size: 10.5px; font-weight: 750">
            {{ profile.extra?.modelNo || profile.code }} · {{ profile.rarity }}
          </span>
        </div>
        <span class="sticker" style="background: var(--pop-lime)">必读</span>
      </div>
      <div v-if="profile.extra?.specs" class="manual-grid">
        <div v-for="spec in profile.extra.specs" :key="spec.label" class="m-cell">
          <small>{{ spec.label }}</small>
          <strong class="num">{{ spec.value }}</strong>
        </div>
      </div>
      <div class="manual-list">
        <div v-if="profile.extra?.ingredients"><b>主要成分：</b>{{ profile.extra.ingredients }}</div>
        <div v-if="profile.extra?.chargeMethod" style="margin-top: 6px">
          <b>充电方式：</b>{{ profile.extra.chargeMethod }}
        </div>
      </div>
      <div v-if="profile.extra?.warning" class="manual-warn">⚠ {{ profile.extra.warning }}</div>
    </div>
  </div>

  <!-- 皮肤 5：精神电量体检单（冰蓝 × 荧光黄化验单） -->
  <div v-else-if="test.id === 'checkup'" class="poster-checkup">
    <div class="chk-head">
      <b>精神状态与电量体检单</b>
      <span class="sticker" style="background: var(--pop-lime)">{{ profile.rarity }}</span>
    </div>
    <div class="chk-ages">
      <div class="chk-box">
        <small>诊断结论</small>
        <strong style="font-size: 18px; margin-top: 6px">{{ profile.title }}</strong>
      </div>
      <div class="chk-box" style="background: var(--pop-lime)">
        <small>实测精神年龄</small>
        <strong class="num" style="color: #e0243c">{{ profile.extra?.mentalAge ?? 58 }} 岁</strong>
      </div>
    </div>
    <div class="chk-bat">
      <div class="chk-bat-top">
        <span>⚡ 当前心理电量</span>
        <span class="num" style="color: var(--pop-lime)">
          {{ profile.extra?.batteryLabel || '45% · 建议充电' }}
        </span>
      </div>
      <div class="chk-bat-bar">
        <i
          :style="{
            width: `${profile.extra?.batteryPct ?? 45}%`,
            background: (profile.extra?.batteryPct ?? 45) >= 65 ? '#6BE5A8' : '#FF4D5E'
          }"
        />
      </div>
    </div>
    <div v-if="profile.extra?.rows" class="chk-table">
      <div v-for="row in profile.extra.rows" :key="row.name" class="chk-row">
        <span>{{ row.name }}</span>
        <span :class="row.status === 'flag' ? 'flag' : 'ok'">{{ row.value }}</span>
      </div>
    </div>
    <div v-if="profile.extra?.prescription" class="chk-rx">{{ profile.extra.prescription }}</div>
  </div>

  <!-- 皮肤 6：16 型人格速测（电光紫新波普） -->
  <div v-else class="poster-classic16">
    <div class="c16-top">
      <span>16 型人格速测 · 荣格八维</span>
      <span class="sticker" style="background: var(--pop-lime)">{{ profile.rarity }}</span>
    </div>
    <div class="c16-code mono">{{ profile.code }}</div>
    <div class="c16-role">{{ profile.avatarEmoji }} {{ profile.title }}</div>
    <div class="c16-axes">
      <div v-for="dim in test.dimensions" :key="dim.id" class="axis-row">
        <div class="axis-lbl">
          <span>{{ dim.highLabel }} {{ result.scores[dim.id] ?? 50 }}%</span>
          <span>{{ dim.lowLabel }} {{ 100 - (result.scores[dim.id] ?? 50) }}%</span>
        </div>
        <div class="axis-bar">
          <i :style="{ width: `${result.scores[dim.id] ?? 50}%` }" />
        </div>
      </div>
    </div>
    <div class="c16-note">
      <b v-if="profile.extra?.jungFunctions">荣格主导功能：{{ profile.extra.jungFunctions }}。</b>
      {{ profile.quote }}
    </div>
  </div>
</template>

<style scoped>
/* 皮肤 1 */
.poster-sbti {
  border-radius: 22px;
  padding: 18px;
  background: var(--pop-lime);
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
  position: relative;
  overflow: hidden;
}
.sbti-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 10px;
  font-weight: 900;
  letter-spacing: 0.8px;
  border-bottom: 2px solid #18181b;
  padding-bottom: 8px;
}
.sbti-stamp {
  position: absolute;
  right: 14px;
  top: 46px;
  transform: rotate(8deg);
  background: var(--pop-red);
  color: #fff;
  border: 2px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  border-radius: 99px;
  padding: 4px 10px;
  font-size: 10.5px;
  font-weight: 900;
}
.sbti-code {
  font-size: 52px;
  font-weight: 950;
  letter-spacing: -2.5px;
  line-height: 1;
  margin-top: 12px;
}
.sbti-name {
  font-size: 20px;
  font-weight: 900;
  margin-top: 5px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.sbti-quote {
  margin-top: 11px;
  padding: 11px 12px;
  background: #18181b;
  color: #fffdf8;
  border-radius: 14px;
  font-size: 12.5px;
  line-height: 1.55;
  font-weight: 650;
}
.sbti-bars {
  margin-top: 12px;
  padding: 12px;
  border-radius: 14px;
  background: #fffdf8;
  border: 1.8px solid #18181b;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px 12px;
}
.sbti-bar-item {
  font-size: 11px;
  font-weight: 800;
}
.sbti-bar-top {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
}
.sbti-track {
  height: 8px;
  background: #ece7da;
  border: 1.2px solid #18181b;
  border-radius: 99px;
  overflow: hidden;
}
.sbti-track i {
  display: block;
  height: 100%;
  background: #18181b;
  border-radius: 99px;
}
.sbti-track i.hot {
  background: var(--pop-red);
}
.sbti-egg {
  margin-top: 10px;
  padding: 8px 11px;
  border-radius: 12px;
  background: #fff7df;
  border: 1.5px dashed #18181b;
  font-size: 11px;
  font-weight: 750;
  line-height: 1.45;
}

/* 皮肤 2 */
.poster-pantone {
  border-radius: 22px;
  overflow: hidden;
  background: #fffdf8;
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
}
.pantone-swatch {
  height: 214px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  color: #ffffff;
  border-bottom: 2px solid #18181b;
  position: relative;
  overflow: hidden;
}
.pantone-orb {
  position: absolute;
  right: -18px;
  top: -18px;
  width: 132px;
  height: 132px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 244, 180, 0.52), transparent 68%);
}
.pantone-top {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 850;
  position: relative;
  z-index: 1;
}
.pantone-center {
  position: relative;
  z-index: 1;
}
.pantone-center h3 {
  font-size: 26px;
  font-weight: 900;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
}
.pantone-center p {
  font-size: 12px;
  font-weight: 700;
  margin-top: 4px;
  opacity: 0.95;
}
.pantone-meta {
  padding: 15px;
}
.pantone-codes {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
}
.p-chip {
  padding: 9px;
  border-radius: 12px;
  background: #fbf6ee;
  border: 1.5px solid #18181b;
  display: flex;
  align-items: center;
  gap: 8px;
}
.p-dot {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  border: 1.4px solid #18181b;
  flex-shrink: 0;
}
.p-chip b {
  display: block;
  font-size: 11.5px;
  font-weight: 850;
}
.p-chip small {
  display: block;
  font-size: 10px;
  color: #6e6960;
  font-weight: 700;
}
.pantone-poem {
  margin-top: 11px;
  font-size: 12.5px;
  line-height: 1.6;
  font-weight: 650;
  color: #333;
}

/* 皮肤 3 */
.poster-animal {
  border-radius: 22px;
  padding: 16px;
  background: var(--pop-peach);
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
  text-align: center;
}
.animal-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  font-weight: 900;
}
.animal-frame {
  margin: 10px auto 8px;
  padding: 10px;
  border-radius: 18px;
  background: #fffdf8;
  border: 2px solid #18181b;
  box-shadow: 3px 3px 0 #18181b;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.animal-name {
  font-size: 24px;
  font-weight: 950;
  letter-spacing: -0.4px;
}
.animal-en {
  font-size: 10.5px;
  letter-spacing: 2px;
  font-weight: 850;
  color: #9a5b25;
  margin-top: 2px;
}
.animal-pills {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 6px;
  margin-top: 10px;
}
.animal-pills span {
  padding: 4px 10px;
  border-radius: 99px;
  background: var(--pop-lime);
  color: #18181b;
  border: 1.4px solid #18181b;
  font-size: 11px;
  font-weight: 850;
}
.animal-guide {
  margin-top: 12px;
  padding: 11px 12px;
  border-radius: 14px;
  background: #fffdf8;
  border: 1.6px solid #18181b;
  text-align: left;
  font-size: 12px;
  line-height: 1.6;
  font-weight: 600;
}

/* 皮肤 4 */
.poster-manual {
  border-radius: 20px;
  overflow: hidden;
  background: #fffdf8;
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
}
.manual-stripe {
  height: 14px;
  background: repeating-linear-gradient(
    -45deg,
    #18181b,
    #18181b 10px,
    var(--pop-lime) 10px,
    var(--pop-lime) 20px
  );
  border-bottom: 2px solid #18181b;
}
.manual-inner {
  padding: 15px;
}
.manual-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  border-bottom: 2px solid #18181b;
  padding-bottom: 10px;
}
.manual-head h3 {
  font-size: 19px;
  font-weight: 950;
}
.manual-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 11px;
}
.m-cell {
  padding: 9px 10px;
  border-radius: 12px;
  background: var(--pop-pink);
  border: 1.5px solid #18181b;
}
.m-cell:nth-child(2) {
  background: var(--pop-lime);
}
.m-cell:nth-child(3) {
  background: var(--pop-blue);
}
.m-cell:nth-child(4) {
  background: var(--pop-purple);
}
.m-cell small {
  display: block;
  font-size: 10px;
  font-weight: 800;
  opacity: 0.8;
}
.m-cell strong {
  display: block;
  font-size: 16px;
  font-weight: 950;
  margin-top: 2px;
}
.manual-list {
  margin-top: 11px;
  font-size: 12px;
  line-height: 1.6;
  font-weight: 650;
}
.manual-warn {
  margin-top: 10px;
  padding: 9px 11px;
  border-radius: 12px;
  background: var(--pop-red);
  color: #fff;
  border: 1.6px solid #18181b;
  font-size: 11.5px;
  font-weight: 800;
}

/* 皮肤 5 */
.poster-checkup {
  border-radius: 20px;
  padding: 16px;
  background: var(--pop-blue);
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
}
.chk-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid #18181b;
  padding-bottom: 8px;
}
.chk-head b {
  font-size: 15px;
  font-weight: 950;
}
.chk-ages {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
  margin-top: 11px;
}
.chk-box {
  background: #fffdf8;
  border: 1.8px solid #18181b;
  box-shadow: 2px 2px 0 #18181b;
  border-radius: 14px;
  padding: 10px 12px;
}
.chk-box small {
  font-size: 11px;
  font-weight: 750;
  color: #5e5a53;
  display: block;
}
.chk-box strong {
  font-size: 28px;
  font-weight: 950;
  display: block;
  margin-top: 1px;
}
.chk-bat {
  margin-top: 10px;
  background: #18181b;
  color: #fffdf8;
  border-radius: 14px;
  padding: 11px 12px;
}
.chk-bat-top {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 800;
  margin-bottom: 7px;
}
.chk-bat-bar {
  height: 12px;
  border-radius: 99px;
  background: #34323d;
  overflow: hidden;
  padding: 2px;
}
.chk-bat-bar i {
  display: block;
  height: 100%;
  border-radius: 99px;
}
.chk-table {
  margin-top: 10px;
  font-size: 12px;
  background: #fffdf8;
  border-radius: 14px;
  border: 1.8px solid #18181b;
  overflow: hidden;
  font-weight: 750;
}
.chk-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1.2px solid #e8e4da;
}
.chk-row:last-child {
  border-bottom: 0;
}
.chk-row .flag {
  color: #e0243c;
  font-weight: 900;
}
.chk-row .ok {
  color: #158f53;
  font-weight: 900;
}
.chk-rx {
  margin-top: 10px;
  padding: 9px 11px;
  border-radius: 12px;
  background: #fffdf8;
  border: 1.5px dashed #18181b;
  font-size: 11.5px;
  font-weight: 700;
  line-height: 1.55;
}

/* 皮肤 6 */
.poster-classic16 {
  border-radius: 22px;
  padding: 18px;
  background: var(--pop-purple);
  color: #18181b;
  border: 2px solid #18181b;
  box-shadow: 4px 4px 0 #18181b;
}
.c16-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  font-weight: 900;
}
.c16-code {
  font-size: 50px;
  font-weight: 950;
  letter-spacing: -2.5px;
  margin-top: 6px;
  line-height: 1;
}
.c16-role {
  font-size: 18px;
  font-weight: 900;
  margin-top: 4px;
}
.c16-axes {
  margin-top: 12px;
  padding: 12px;
  border-radius: 14px;
  background: #fffdf8;
  border: 1.8px solid #18181b;
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.axis-row {
  font-size: 11px;
  font-weight: 800;
}
.axis-lbl {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
}
.axis-bar {
  height: 8px;
  background: #eeeaf8;
  border: 1.2px solid #18181b;
  border-radius: 99px;
  overflow: hidden;
}
.axis-bar i {
  display: block;
  height: 100%;
  background: #7c4dff;
  border-radius: 99px;
}
.c16-note {
  margin-top: 10px;
  font-size: 12px;
  font-weight: 650;
  line-height: 1.55;
}
</style>
