<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { collectionProjects } from '@/lib/openSource'
import { openSourceCatalog as catalog, loadOpenSourceCatalog } from '@/stores/openSource'
import CommunityBack from '@/components/community/CommunityBack.vue'
import DiscoverGlyph from '@/components/community/DiscoverGlyph.vue'
import OpenSourceProjectCard from '@/components/community/OpenSourceProjectCard.vue'
import { useHead } from '@/lib/head'
import '@/components/community/community.css'
import '@/components/community/open-source.css'
const route = useRoute()
void loadOpenSourceCatalog()
const collection = computed(() => catalog.collections.find(item => item.key === route.params.key))
const projects = computed(() => collection.value ? collectionProjects(collection.value, catalog) : [])
useHead(() => ({ title: collection.value ? `${collection.value.title} · 精选` : '专题未收录' }))
</script>
<template><main class="community-page os-detail-page"><nav class="os-detail-nav"><CommunityBack fallback="/open-source" label="精选" /><span>专题</span></nav><template v-if="collection"><header class="os-collection-lead"><div class="os-kicker"><DiscoverGlyph name="layers" :size="16" />专题精选 · {{ projects.length }} 个项目</div><h1>{{ collection.title }}</h1><p>{{ collection.description }}</p></header><div class="os-collection-list"><section v-for="(project, index) in projects" :key="project.full_name" class="os-collection-entry"><header><span>0{{ index + 1 }}</span><p>{{ collection.notes[index] }}</p></header><OpenSourceProjectCard :project="project" /></section></div><p class="os-source-note">{{ catalog.notice }}</p></template><section v-else class="os-empty"><h1>这个专题尚未收录</h1><RouterLink to="/open-source" class="os-button">浏览精选</RouterLink></section></main></template>
