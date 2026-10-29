<script setup lang="ts">
import { accent, accents, isDark, appearanceMode, appearanceModes, setAccent, setAppearance } from '../../appearance.ts';
</script>
<template>
  <div class="settings-appearance" data-testid="settings-appearance">
    <div class="xb-group-title">外观模式</div>
    <div class="xb-group mode-group" role="group" aria-label="外观模式" data-testid="appearance-mode">
      <button v-for="mode in appearanceModes" :key="mode.value" type="button" class="mode-card" :class="{ on: appearanceMode === mode.value }" :aria-pressed="appearanceMode === mode.value" :data-testid="`appearance-${mode.value}`" @click="setAppearance(mode.value)">
        <span class="mode-preview" :class="`is-${mode.value}`"><i></i><i></i></span>{{ mode.label }}
      </button>
    </div>
    <div class="xb-group-title">主题色</div>
    <div class="xb-group">
      <div class="xb-row swatch-row" role="radiogroup" aria-label="主题色">
        <button v-for="item in accents" :key="item.id" type="button" role="radio" class="swatch" :class="{ on: accent === item.id }" :aria-checked="accent === item.id" :data-testid="`settings-accent-${item.id}`" @click="setAccent(item.id)">
          <span :style="{ background: `linear-gradient(160deg, ${(isDark ? item.dark : item.light).btnHi}, ${(isDark ? item.dark : item.light).btn})`, '--ring': (isDark ? item.dark : item.light).btn }"></span>{{ item.name }}
        </button>
      </div>
    </div>
  </div>
</template>
<style scoped>
.mode-group{display:flex;gap:22px;padding:18px 16px}
.mode-card{display:flex;flex-direction:column;align-items:center;gap:8px;border:0;background:transparent;padding:0;font-size:12.5px;color:var(--ui-text-primary);cursor:pointer}
.mode-preview{width:132px;height:84px;border-radius:10px;overflow:hidden;display:flex;gap:4px;padding:6px;box-shadow:0 0 0 1px var(--ui-border)}
.mode-preview i{border-radius:5px}
.mode-preview i:first-child{width:28px}
.mode-preview i:last-child{flex:1}
.mode-preview.is-system{background:linear-gradient(90deg,#F2F2F4 50%,#1E1E20 50%)}
.mode-preview.is-system i:first-child{background:rgba(255,255,255,.9)}
.mode-preview.is-system i:last-child{background:linear-gradient(90deg,#fff 35%,#2C2C2E 35%)}
.mode-preview.is-light{background:#ECECEF}
.mode-preview.is-light i:first-child{background:#F7F7F9}
.mode-preview.is-light i:last-child{background:#fff}
.mode-preview.is-dark{background:#151517}
.mode-preview.is-dark i:first-child{background:#232326}
.mode-preview.is-dark i:last-child{background:#2C2C2F}
.mode-card.on .mode-preview{box-shadow:0 0 0 2px var(--ui-surface),0 0 0 4px var(--ui-btn)}
.mode-card.on{font-weight:500}
.swatch-row{flex-wrap:wrap;gap:18px;padding:14px 16px}
.swatch{display:flex;flex-direction:column;align-items:center;gap:7px;border:0;background:transparent;padding:0;font-size:12px;color:var(--ui-text-secondary);cursor:pointer}
.swatch>span{width:30px;height:30px;border-radius:50%;box-shadow:inset 0 0 0 1px rgba(0,0,0,.1)}
.swatch.on{color:var(--ui-text-primary);font-weight:500}
:root[data-theme=dark] .swatch>span{box-shadow:inset 0 0 0 1px rgba(255,255,255,.18)}
.swatch.on>span,:root[data-theme=dark] .swatch.on>span{box-shadow:0 0 0 2px var(--ui-surface),0 0 0 4px var(--ring)}
</style>
