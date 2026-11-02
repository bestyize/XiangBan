<script setup lang="ts">
withDefaults(defineProps<{items:readonly {key:string;label:string}[];active:string;label?:string}>(),{label:'分类筛选'})
defineEmits<{change:[key:string]}>()
</script>
<template><div class="filter-row" role="group" :aria-label="label"><button v-for="item in items" :key="item.key" type="button" :class="{on:active===item.key}" :aria-pressed="active===item.key" @click="$emit('change',item.key)">{{ item.label }}</button></div></template>
<style scoped>
.filter-row{display:flex;gap:8px;overflow-x:auto;padding:4px 12px 12px;scrollbar-width:none}.filter-row::-webkit-scrollbar{display:none}.filter-row button{flex:none;border:0;background:var(--surface);color:var(--ink-2);border-radius:24px;min-height:36px;padding:4px 14px;font-size:12px;cursor:pointer}.filter-row button.on{color:var(--brand);background:color-mix(in srgb,var(--brand) 9%,var(--surface));font-weight:600}.filter-row button:focus-visible{outline:2px solid var(--brand);outline-offset:-2px}
@media(min-width:1100px){.filter-row{padding-bottom:4px}}
</style>
