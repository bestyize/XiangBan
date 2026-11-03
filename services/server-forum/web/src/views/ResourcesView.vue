<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '@/api/forum'
import type { Page, TopicItem } from '@/api/types'
import CommunityTabs from '@/components/community/CommunityTabs.vue'
import HelpStats from '@/components/community/HelpStats.vue'
import MasonryGrid from '@/components/community/MasonryGrid.vue'
import OpenSourceSoftwareDirectory from '@/components/community/OpenSourceSoftwareDirectory.vue'
import RequestState from '@/components/community/RequestState.vue'
import ResourceTopicRow from '@/components/community/ResourceTopicRow.vue'
import PagerNav from '@/components/PagerNav.vue'
import { useCommunityResource } from '@/components/community/resource'
import { useHead } from '@/lib/head'
import { RESOURCE_TABS, resourceTopicCategory } from '@/lib/communityStructure'
import '@/components/community/community.css'
const route = useRoute(), router = useRouter()
const tabs = RESOURCE_TABS
const tab = computed(() => tabs.some(t => t.key === route.query.tab) ? String(route.query.tab) : 'software')
const sectionTitle = computed(() => tabs.find(item => item.key === tab.value)?.label || '资源')
const page = computed(() => Math.max(1, Math.floor(Number(route.query.page) || 1)))
type ResourceData = { kind:'topics'; result:Page<TopicItem> }
const { data, loading, error, load } = useCommunityResource<ResourceData | null>(async () => {
  if (tab.value === 'software') return null
  const category = resourceTopicCategory(tab.value)
  return { kind:'topics', result: await api.topics({ category, page:page.value, sort:'new' }) }
})
watch([tab,page], load, { immediate:true })
function setTab(tab:string) { void router.replace({ query: { tab } }) }
function go(page:number) { void router.replace({ query:{ ...route.query, page } }) }
useHead(() => ({ title:'资源' }))
</script>
<template>
  <main class="community-page resource-page" :class="{ 'resource-page--media': tab === 'media' }">
    <h1 class="sr-only">社区资源</h1>
    <CommunityTabs :class="{ 'resource-tabs--software': tab === 'software' }" :items="tabs" :active="tab" label="资源栏目" @change="setTab" />
    <OpenSourceSoftwareDirectory v-if="tab === 'software'" />
    <template v-else>
      <HelpStats v-if="tab === 'help'" />
      <RequestState :loading="loading" :error="error" @retry="load" />
      <template v-if="data?.kind === 'topics'">
        <div class="desktop-feed-heading meta-only">
          <h2 class="sr-only">{{ sectionTitle }}</h2><span>共 {{ data.result.total.toLocaleString('zh-CN') }} 条</span>
        </div>
        <MasonryGrid v-if="tab === 'media' && data.result.items.length" class="resource-media-grid" :columns="2">
          <ResourceTopicRow v-for="topic in data.result.items" :key="topic.id" :topic="topic" layout="masonry" mode="media" note-cover />
        </MasonryGrid>
        <div v-else-if="tab === 'help'" class="resource-feed topic-grid feed-help">
          <ResourceTopicRow v-for="topic in data.result.items" :key="topic.id" :topic="topic" mode="help" />
        </div>
        <RequestState :empty="!data.result.items.length" />
        <PagerNav :page="data.result.page" :page-size="data.result.page_size" :total="data.result.total" @go="go" />
      </template>
    </template>
  </main>
</template>
<style scoped>
.resource-page .resource-tabs--software { margin-bottom: 4px; }
.resource-media-grid { padding: 8px 12px; }
.resource-page--media { container-type: inline-size; }
@container (min-width: 880px) {
  .resource-page .resource-media-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
@container (min-width: 1160px) {
  .resource-page .resource-media-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); }
}
.desktop-feed-heading{display:none}.resource-feed{display:grid;gap:8px;padding:8px 12px}.tool-card{display:flex;align-items:center;gap:12px;min-width:0;background:var(--surface);border-radius:12px;padding:12px 14px}.tool-icon{width:48px;height:48px;flex:none;display:grid;place-items:center;border-radius:12px;background:var(--surface-2)}.tool-icon img{width:100%;height:100%;object-fit:contain;border-radius:12px}.tool-body{display:grid;gap:4px;min-width:0;flex:1}.tool-body strong{font-size:15px;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.tool-meta{display:flex;gap:6px;align-items:center;flex-wrap:wrap;color:var(--ink-3);font-size:12px}.license{padding:1px 6px;border-radius:5px;background:var(--surface-2)}.sample{font-size:12px;color:var(--brand);font-weight:400}.arrow{font-size:20px;color:var(--ink-3)}@media(min-width:768px){.resource-feed{gap:10px}.tool-card{padding:14px 16px}}
@media(min-width:1100px){
 .resource-page{max-width:1120px}
 .resource-page--media{max-width:1440px}
 .desktop-feed-heading{display:flex;align-items:baseline;justify-content:space-between;padding:20px 12px 2px}.desktop-feed-heading h2{font-size:17px;margin:0}.desktop-feed-heading span{font-size:12px;color:var(--ink-3)}
 .resource-feed{gap:12px}.tool-grid,.tool-card{min-height:96px;padding:16px 18px}.topic-grid :deep(.topic-card){height:100%;padding:18px}
}


.topic-grid{grid-template-columns:repeat(var(--card-columns,2),minmax(0,1fr))}.topic-grid.feed-help{grid-template-columns:minmax(0,1fr)}.topic-grid :deep(.topic-card.visual-card){padding:0}
@media(min-width:640px){.tool-grid{grid-template-columns:repeat(var(--card-columns,2),minmax(0,1fr))}}
</style>
