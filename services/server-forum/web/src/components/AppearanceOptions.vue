<script setup lang="ts">
import { useId } from 'vue'
import { useTheme, THEME_OPTIONS, accents } from '@/lib/theme'
const groupId = useId()
const { preference, resolved, setPreference, accent, setAccent } = useTheme()
</script>
<template>
  <div class="appearance-options">
    <fieldset><legend>外观模式</legend><div class="modes">
      <label v-for="item in THEME_OPTIONS" :key="item.value" :class="{ selected:preference === item.value }">
        <input type="radio" :name="`${groupId}-mode`" :value="item.value" :checked="preference === item.value" @change="setPreference(item.value)">
        <span>{{ item.label }}</span>
      </label>
    </div></fieldset>
    <fieldset><legend>主题色</legend><div class="accents">
      <label v-for="item in accents" :key="item.id" :class="{ selected:accent === item.id }">
        <input type="radio" :name="`${groupId}-accent`" :value="item.id" :checked="accent === item.id" @change="setAccent(item.id)">
        <span class="swatch" :style="{ background:(resolved === 'dark' ? item.dark : item.light).btn }" aria-hidden="true"><span v-if="accent === item.id" class="check">✓</span></span>
        <span>{{ item.name }}</span>
      </label>
    </div></fieldset>
    <p>选择后立即生效，并保存在当前浏览器。</p>
  </div>
</template>
<style scoped>
fieldset{min-width:0;margin:0 0 22px;padding:0;border:0}legend{padding:0;margin-bottom:12px;font-size:14px;font-weight:600}.modes,.accents{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px}label{position:relative;display:flex;align-items:center;justify-content:center;gap:10px;min-height:48px;padding:10px 6px;border:1px solid var(--line);border-radius:12px;background:var(--surface-2);cursor:pointer;font-size:13px}label.selected{border-color:var(--brand);background:var(--brand-soft);color:var(--brand)}label:focus-within{outline:2px solid var(--focus);outline-offset:3px}input{position:absolute;width:1px;height:1px;opacity:0}.accents label{flex-direction:column;min-height:90px}.swatch{width:30px;height:30px;display:grid;place-items:center;border-radius:50%;box-shadow:inset 0 0 0 1px #ffffff25}.check{display:grid;place-items:center;width:18px;height:18px;border-radius:50%;background:#fff;color:#222;font-size:12px;font-weight:700}p{margin:0;color:var(--ink-3);font-size:12px}
@media(max-width:380px){.modes{grid-template-columns:1fr}.modes label{min-height:44px}}
</style>
