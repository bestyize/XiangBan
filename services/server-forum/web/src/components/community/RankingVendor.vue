<script setup lang="ts">
import { computed } from 'vue'
import manifest from '../../../public/vendor-logos/manifest.json'
import legacy from '../../../public/vendor-logos/legacy.json'
const props = defineProps<{ name: string }>()
const normalize = (s: string) => s.toLowerCase().split('/')[0]!.replace(/[^\p{L}\p{N}]/gu, '')
const fallbackColors: [string[], string][] = [
  [['openai'], '#10A37F'], [['z-ai'], '#5366D3'], [['meta'], '#00A3D9'], [['tencent'], '#2F6BFF'],
  [['spacex'], '#8B5CF6'], [['alibaba'], '#FF7A00'], [['xiaomi'], '#FF6900'], [['inclusion'], '#49B3F1'],
  [['stepfun'], '#00B8A9'], [['reflection'], '#155C2C'], [['nvidia'], '#76B900'], [['thinking'], '#6B7280'], [['mistral'], '#EF9538'],
]
const logo = computed(() => {
  const normalized = normalize(props.name)
  const item = manifest.vendors.find(v => v.aliases.some(a => normalize(a) === normalized))
  if (item) return {file:item.file,color:`#${item.color}`}
  const lower = props.name.toLowerCase()
  const old = legacy.find(v => v.aliases.some(a => lower.includes(a)))
  return old ? {file:old.file,color:`#${old.color}`} : null
})
const color = computed(() => logo.value?.color || fallbackColors.find(([aliases]) => aliases.some(a => props.name.toLowerCase().includes(a)))?.[1] || '#2F6BFF')
function luminance(hex: string) {
  const rgb = [1,3,5].map(i => parseInt(hex.slice(i,i+2), 16) / 255).map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4)
  return .2126 * rgb[0]! + .7152 * rgb[1]! + .0722 * rgb[2]!
}
const darkLogo = computed(() => !!logo.value && luminance(color.value) < .18)
const letter = computed(() => Array.from(props.name.trim() || '?')[0]!.toUpperCase())
const style = computed(() => ({ '--vendor-color':color.value, ...(logo.value ? { '--vendor-mask':`url("/vendor-logos/${logo.value.file}")` } : {}) }))
</script>
<template>
  <span class="ranking-vendor" :class="{ 'dark-logo': darkLogo, 'letter-icon': !logo }" :style="style" role="img" :aria-label="name.trim() || '未知厂商'">
    <span v-if="logo" class="brand-mark" aria-hidden="true" /><span v-else aria-hidden="true">{{ letter }}</span>
  </span>
</template>
<style scoped>
.ranking-vendor{width:18px;height:18px;flex:0 0 18px;display:inline-grid;place-items:center;color:var(--vendor-color);vertical-align:middle}.brand-mark{display:block;width:100%;height:100%;background:currentColor;mask:var(--vendor-mask) center/contain no-repeat;-webkit-mask:var(--vendor-mask) center/contain no-repeat}.letter-icon{border-radius:4.5px;background:color-mix(in srgb,var(--vendor-color) 12%,transparent);font-size:7.56px;line-height:1;font-weight:700}:global(:root[data-theme="dark"]) .dark-logo{color:var(--ink)}
</style>
