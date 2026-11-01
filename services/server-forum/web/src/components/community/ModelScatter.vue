<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import type { Board } from '@/lib/modelRankings'
import ModelScatterPlot from './ModelScatterPlot.vue'
defineProps<{board:Board}>()
const full=ref(false),dialog=ref<HTMLDialogElement|null>(null),expand=ref<HTMLButtonElement|null>(null)
watch(full,async value=>{await nextTick();if(value&&!dialog.value?.open)dialog.value?.showModal();else if(!value&&dialog.value?.open)dialog.value.close()})
function close(){full.value=false;void nextTick(()=>expand.value?.focus())}
onBeforeUnmount(()=>dialog.value?.close())
</script>
<template><section class="model-scatter"><header><div><h2>智能 vs 价格</h2><p>左上角：较高智能、较低价格</p></div><button ref="expand" aria-label="全屏查看智能与价格散点图" @click="full=true">全屏 ↗</button></header><ModelScatterPlot :board="board"/><p class="formula">加权价格：输入 × 75% + 输出 × 25%，美元/百万 Token · 横轴为对数轴</p><dialog ref="dialog" aria-label="智能与价格散点图，全屏" @cancel.prevent="close" @close="full=false"><ModelScatterPlot v-if="full" :board="board" full @close="close"/></dialog></section></template>
<style scoped>
.model-scatter { padding:14px; background:var(--surface); border-radius:16px; } header { display:flex;align-items:center;justify-content:space-between;gap:8px; } h2 { font-size:16px;margin:0; } header p,.formula { font-size:12px;line-height:1.6;color:var(--ink-3);margin:4px 0 0; } header button { flex:none;min-height:44px;border:0;border-radius:8px;background:var(--surface-2);color:var(--ink-2);font-size:12px;padding:8px 12px;cursor:pointer; } dialog { position:fixed;inset:0;width:100%;height:100dvh;max-width:100%;max-height:100dvh;margin:0;padding:max(4px,env(safe-area-inset-top)) max(4px,env(safe-area-inset-right)) max(4px,env(safe-area-inset-bottom)) max(4px,env(safe-area-inset-left));border:0;border-radius:0;background:var(--surface);color:var(--ink);box-sizing:border-box; } dialog::backdrop { background:var(--surface); }
</style>
