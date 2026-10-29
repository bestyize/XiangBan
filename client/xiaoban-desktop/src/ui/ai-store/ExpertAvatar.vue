<script setup lang="ts">
import { computed } from 'vue';
const props = defineProps<{ id: string }>();
const palettes: Record<string, string[]> = {
  research: ['#f3eded','#f2c5ae','#48302f','#f4b3b2'], code: ['#e9f1f5','#eac0a7','#383639','#638fba'],
  design: ['#f7e9de','#f1c5ad','#6c3b32','#f2b58c'], writer: ['#f1ecf8','#f4c9ae','#5b443a','#a79ad4'],
  coach: ['#e8f3e8','#e8bd9f','#3b3431','#99b8a2'], learning: ['#eeeaf6','#f2c9b6','#473939','#9b94ca'],
};
const p = computed(() => palettes[props.id] ?? palettes.research);
</script>
<template>
  <svg class="expert-avatar" viewBox="0 0 96 104" aria-hidden="true">
    <rect width="96" height="104" :fill="p[0]" rx="13" /><ellipse cx="48" cy="104" rx="47" ry="35" :fill="p[3]" />
    <path d="M34 71h28v22H34z" :fill="p[1]" /><path d="M17 91q4-21 19-22l12 9 12-9q18 2 21 22v15H17z" :fill="p[3]" />
    <path v-if="!['code','coach'].includes(id)" d="M20 52q-5-34 17-42 24-10 38 11 7 12 3 46l-12 12H28z" :fill="p[2]" />
    <path v-else d="M23 47Q18 18 37 12q24-9 38 13l-1 28-9-15-36 0z" :fill="p[2]" />
    <ellipse cx="48" cy="49" rx="22" ry="27" :fill="p[1]" /><path d="M26 42q1-28 24-29 19 0 23 26-11-3-20-14-8 13-27 17z" :fill="p[2]" />
    <path d="M36 49q3-2 6 0m12 0q3-2 6 0" stroke="#443837" stroke-width="2.2" fill="none" stroke-linecap="round" />
    <circle cx="39" cy="51" r="1.7" fill="#25272d" /><circle cx="57" cy="51" r="1.7" fill="#25272d" />
    <path d="M45 62q3 2 6 0" fill="none" stroke="#a36d66" stroke-width="1.7" stroke-linecap="round" />
    <g v-if="id === 'code'" fill="none" stroke="#534b49" stroke-width="2"><rect x="30" y="43" width="16" height="14" rx="5" /><rect x="50" y="43" width="16" height="14" rx="5" /><path d="M46 49h4" /></g>
  </svg>
</template>
<style scoped>.expert-avatar{width:70px;height:75px;flex:none;border-radius:12px;overflow:hidden}</style>
