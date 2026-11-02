<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import { useCommunityResource } from './resource'
import RequestState from './RequestState.vue'
import CommunityTabs from './CommunityTabs.vue'
import RankingVendor from './RankingVendor.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import { webLink } from './contract'
import { navigationCategories, navigationDomain, navigationGroups, navigationIcon, navigationLogo, navigationMatches, navigationType } from '@/lib/navigationDirectory'
const route = useRoute(), router = useRouter()
const { data, loading, error, load } = useCommunityResource(discoverApi.nav)
const categories = navigationCategories
const category = computed(() => categories.find(item => item.key === route.query.nav_category) || categories[0]!)
const selected = computed(() => category.value.key)
const filter = computed(() => category.value.filters.some(item => item.key === route.query.nav_filter) ? String(route.query.nav_filter) : '')
const groups = computed(() => data.value ? navigationGroups(data.value, url => !!webLink(url)) : [])
const allEntries = computed(() => groups.value.flatMap(group => group.items))
const entries = computed(() => allEntries.value.filter(item => navigationMatches(item, selected.value, filter.value)))
function selectCategory(key: string) { void router.replace({query:{...route.query, nav_category:key === 'models' ? undefined : key, nav_filter:undefined}}) }
function selectFilter(key: string) { void router.replace({query:{...route.query, nav_filter:key || undefined}}) }
onMounted(load)
</script>
<template>
  <div class="directory">
    <CommunityTabs :items="categories" :active="selected" label="AI 导航分类" @change="selectCategory" />
    <div class="directory-filters" role="group" :aria-label="`${category.label}筛选`">
      <button type="button" :aria-pressed="!filter" :class="{on:!filter}" @click="selectFilter('')">全部</button>
      <button v-for="item in category.filters" :key="item.key" type="button" :aria-pressed="filter===item.key" :class="{on:filter===item.key}" @click="selectFilter(item.key)">{{ item.label }}</button>
    </div>
    <RequestState :loading="loading" :error="error" :empty="!!data && !entries.length" message="这个分类还没有已发布入口。" @retry="load" />
    <div v-if="data" class="nav-grid">
      <a v-for="item in entries" :key="item.key" :href="webLink(item.url)" target="_blank" rel="noopener noreferrer" class="nav-entry" :title="`${item.name} · ${navigationDomain(item.url)}`">
        <span class="brand-tile" aria-hidden="true"><img v-if="navigationIcon(item)" :src="navigationIcon(item)" alt="" loading="lazy"><RankingVendor v-else :name="navigationLogo(item)" /></span><DiscoverGlyph name="external" :size="13" class="external" />
        <span class="entry-name">{{ item.name }}<small v-if="item.sample" class="sample">示例</small></span>
        <span class="entry-vendor">{{ item.vendor || navigationDomain(item.url) }}</span>
        <span v-if="item.summary" class="entry-summary">{{ item.summary }}</span>
        <span class="entry-foot"><span class="entry-domain">{{ navigationDomain(item.url) }}</span><span v-if="navigationType(item)" class="entry-type">{{ navigationType(item) }}</span></span>
      </a>
    </div>
    <p v-if="data && entries.length" class="nav-note"><DiscoverGlyph name="external" :size="13" />点击卡片，在新标签页打开官方入口</p>
  </div>
</template>
<style scoped>
.directory{min-width:0;padding:0 12px 8px;container-type:inline-size}
.directory :deep(.community-tabs){background:none;padding-inline:0;justify-content:flex-start;gap:28px;margin-bottom:20px}
.directory-filters{display:flex;gap:8px;flex-wrap:wrap;margin-bottom:20px}
.directory-filters button{min-height:34px;padding:5px 13px;background:var(--surface);color:var(--ink-2);border:1px solid var(--line);border-radius:24px;cursor:pointer;font-size:12px;line-height:1.5}
.directory-filters button.on{color:var(--brand);border-color:color-mix(in srgb,var(--brand) 24%,var(--line));background:color-mix(in srgb,var(--brand) 6%,var(--surface))}
.nav-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
.nav-entry{position:relative;min-width:0;display:grid;grid-template-columns:36px minmax(0,1fr);align-content:start;column-gap:8px;padding:14px;box-sizing:border-box;background:var(--surface);border:1px solid var(--line);border-radius:12px;color:var(--ink);transition:border-color .15s,background .15s}
.nav-entry:hover{background:color-mix(in srgb,var(--surface-2) 35%,var(--surface));border-color:var(--line-strong)}
.nav-entry:focus-visible,.directory-filters button:focus-visible{outline:2px solid var(--brand);outline-offset:3px}
.brand-tile{grid-column:1;grid-row:1/span 2;align-self:center;display:grid;place-items:center;width:36px;height:36px;border-radius:9px;background:var(--surface-2)}
.brand-tile>img{width:26px;height:26px;object-fit:contain}.brand-tile :deep(.ranking-vendor){width:24px;height:24px;flex-basis:24px}.brand-tile :deep(.letter-icon){font-size:16px;background:none}
.external{position:absolute;top:12px;right:10px;color:var(--ink-3);opacity:.5}.nav-entry:hover .external{color:var(--brand);opacity:1}
.entry-name{grid-column:2;padding-right:10px;font-size:14px;line-height:1.5;font-weight:400;min-width:0;overflow-wrap:anywhere}
.entry-vendor{grid-column:2;font-size:12px;line-height:1.5;color:var(--ink-3);margin-top:2px;overflow-wrap:anywhere}
.entry-summary{grid-column:1/-1;margin-top:12px;font-size:12px;line-height:1.7;min-height:3.4em;color:var(--ink-2);display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:2;overflow:hidden}
.entry-foot{grid-column:1/-1;display:flex;justify-content:space-between;align-items:center;gap:8px;margin-top:12px;min-width:0}
.entry-domain{display:none;min-width:0;font-size:12px;line-height:1.5;color:var(--ink-3);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.entry-type{flex:none;font-size:11px;line-height:1.5;color:var(--ink-3);background:var(--surface-2);border-radius:5px;padding:2px 6px}
.sample{font-size:11px;font-weight:400;color:var(--brand);margin-left:4px}
.nav-note{display:flex;align-items:center;gap:6px;color:var(--ink-3);font-size:12px;line-height:1.6;margin:24px 0 0}.nav-note svg{flex:none}
@container (min-width:600px){.nav-grid{grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}.entry-domain{display:block}}
@container (min-width:920px){.nav-grid{grid-template-columns:repeat(4,minmax(0,1fr));gap:16px}.nav-entry{padding:18px}}
@media(min-width:1024px){.directory{padding:0 0 8px}}
@media(max-width:370px){.nav-entry{padding:12px}.directory :deep(.community-tabs){gap:22px}}
@media(prefers-reduced-motion:reduce){.nav-entry{transition:none}}
</style>
