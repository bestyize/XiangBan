<script setup lang="ts">
import { computed } from 'vue'
import RankingVendor from './RankingVendor.vue'
const props = defineProps<{ name:string; fallback:string }>()
const special = computed(() => { const n=props.name.toLowerCase(); return n.includes('opencode') ? 'opencode' : n.includes('claude') ? 'claude' : '' })
</script>
<template><svg v-if="special" class="tool-mark" :class="special" viewBox="0 0 24 24" aria-hidden="true"><path v-if="special === 'claude'" d="M21 10.5h3v3h-3v3h-1.5v3H18v-3h-1.5v3H15v-3H9v3H7.5v-3H6v3H4.5v-3H3v-3H0v-3h3v-6h18Zm-15 0h1.5v-3H6Zm10.5 0H18v-3h-1.5z"/><path v-else d="M22 24H2V0h20zM17 4.8H7v14.4h10z"/></svg><RankingVendor v-else :name="fallback" /></template>
<style scoped>.tool-mark{width:27px;height:27px;fill:currentColor}.claude{color:#D97757}.opencode{color:#111}:global(:root[data-theme="dark"]) .opencode{color:var(--ink)}</style>
