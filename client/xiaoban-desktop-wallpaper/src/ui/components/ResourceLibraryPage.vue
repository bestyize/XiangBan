<script setup lang="ts">
import {ref,watch} from 'vue';
const props=defineProps<{title:string;activeTab:string}>();
const body=ref<HTMLElement>();
watch(()=>props.activeTab,()=>body.value?.scrollTo?.(0,0));
</script>
<template>
 <section class="ui-page resource-library-page">
  <header class="page-header resource-library-header"><h1>{{ title }}</h1><div class="resource-library-actions"><slot name="actions"/></div></header>
  <div class="resource-library-tabs"><slot name="tabs"/></div>
  <div ref="body" class="resource-library-body"><slot/></div>
 </section>
</template>
<style scoped>
.resource-library-page{display:flex;flex-direction:column;flex:1;min-height:0;width:100%;overflow:hidden;color:var(--ui-text-primary)}
.resource-library-header{flex:none;display:flex;align-items:center;justify-content:space-between;gap:24px;padding:var(--ui-page-top) var(--ui-page-inset) 4px;margin:0;background:var(--ui-app-bg)}
.resource-library-header h1{margin:0;white-space:nowrap;font-size:24px;line-height:1.3;font-weight:700}
.resource-library-actions{display:flex;align-items:center;gap:10px;flex:none}
.resource-library-tabs{flex:none;padding:0 var(--ui-page-inset);background:var(--ui-app-bg)}
.resource-library-tabs :deep(.n-tabs-pane-wrapper){display:none}
.resource-library-body{flex:1;min-height:0;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;padding:var(--ui-section-gap) var(--ui-page-inset) 32px;display:flex;flex-direction:column;gap:var(--ui-section-gap)}
.resource-library-body>*{flex-shrink:0}
@media(max-width:760px){.resource-library-header{gap:12px}.resource-library-header h1{font-size:21px}.resource-library-actions{gap:8px}}
</style>
