<script setup lang="ts">
import { computed,onMounted } from 'vue'
import { discoverApi } from '@/api/discover'
import { useCommunityResource } from './resource'
import RequestState from './RequestState.vue'
import RankingVendor from './RankingVendor.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import { webLink } from './contract'
const {data,loading,error,load}=useCommunityResource(discoverApi.nav)
const groups=computed(()=>(data.value?.groups||[]).map(group=>({...group,items:group.items.filter(item=>webLink(item.url))})).filter(group=>group.items.length))
onMounted(load)
</script>
<template><div class="directory"><RequestState :loading="loading" :error="error" :empty="!!data&&!groups.length" message="还没有已发布导航。" @retry="load"/><template v-if="data"><div class="nav-groups"><section v-for="group in groups" :key="group.name" class="nav-card"><header><div><h2>{{ group.name||'常用入口' }}</h2><p>{{ group.items.length }} 个官方入口</p></div><DiscoverGlyph name="globe" :size="20"/></header><div class="nav-grid"><a v-for="item in group.items" :key="item.key" :href="webLink(item.url)" target="_blank" rel="noopener noreferrer"><span class="brand-tile"><RankingVendor :name="item.vendor||item.name"/></span><div><span class="name">{{ item.name }}</span><small v-if="item.sample" class="sample">示例</small></div><DiscoverGlyph name="external" :size="13" class="external"/></a></div></section></div><p v-if="groups.length" class="nav-note"><DiscoverGlyph name="info" :size="14"/>官方入口会在新标签页打开。</p></template></div></template>
<style scoped>
.directory{padding-top:8px;min-width:0}.nav-groups{display:grid;gap:8px;align-items:start}.nav-card{min-width:0;background:var(--surface);border-radius:16px;padding:16px}header{display:flex;align-items:center;justify-content:space-between;gap:12px;color:var(--ink-3);padding-bottom:12px}h2{margin:0;font-size:16px;line-height:1.5;color:var(--ink);font-weight:600}header p{margin:4px 0 0;font-size:12px;line-height:1.5}.nav-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:4px 12px}a{display:flex;align-items:center;min-height:64px;gap:8px;padding:8px 0;color:var(--ink);border-radius:10px}a:hover{background:var(--surface-2)}a>div{min-width:0;flex:1}.brand-tile{display:grid;place-items:center;flex:none;width:36px;height:36px;border-radius:10px;background:var(--surface-2)}.brand-tile :deep(.ranking-vendor){width:22px;height:22px;flex-basis:22px}.brand-tile :deep(.letter-icon){font-size:14px;background:none}.name{font-size:14px;font-weight:500;line-height:1.5;overflow-wrap:anywhere}.external{flex:none;color:var(--ink-3);opacity:.6}.sample{display:block;font-size:12px;color:var(--brand);line-height:1.5}.nav-note{display:flex;align-items:center;gap:6px;color:var(--ink-3);font-size:12px;margin:12px 4px}.nav-note svg{flex:none}@media(min-width:768px){.nav-groups{grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}.nav-card{padding:20px}.nav-grid{gap:8px 12px}}@media(max-width:370px){.nav-grid{column-gap:8px}.brand-tile{width:32px;height:32px}.external{display:none}}
</style>
