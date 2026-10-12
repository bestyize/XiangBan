<script setup lang="ts">
import { computed } from 'vue';
import { countdownValue, emptyCountdown } from './content.ts';
import type { CountdownData } from './model.ts';
const props=defineProps<{data?:CountdownData;now:Date}>();
const data=computed(()=>props.data??emptyCountdown());
const value=computed(()=>countdownValue(data.value,props.now));
</script>
<template>
  <div class="countdown-content"><h2>{{ data.title }}</h2><span class="countdown-label">{{ value.label }}</span><div class="countdown-days"><strong>{{ value.days }}</strong><span>天</span></div><p>{{ value.date }}<span v-if="data.repeat==='yearly'"> · 每年纪念</span></p></div>
</template>
<style scoped>
.countdown-content{text-align:center;padding-top:17px}.countdown-content h2{font-size:17px;font-weight:500;line-height:1.5;margin:0 0 8px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.countdown-label{font-size:11px;opacity:.65}.countdown-days{display:flex;align-items:baseline;justify-content:center;gap:8px;margin:4px 0}.countdown-days strong{font-size:53px;font-weight:300;line-height:1.1;font-variant-numeric:tabular-nums}.countdown-days>span{font-size:12px;opacity:.6}.countdown-content p{font-size:10px;opacity:.6;margin:10px 0 0}
</style>
