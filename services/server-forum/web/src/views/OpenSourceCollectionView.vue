<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { openSourceBrowse } from '@/api/openSourceBrowse'
import { useCommunityResource } from '@/components/community/resource'
import RequestState from '@/components/community/RequestState.vue'
import CommunityBack from '@/components/community/CommunityBack.vue'
import DiscoverGlyph from '@/components/community/DiscoverGlyph.vue'
import OpenSourceProjectCard from '@/components/community/OpenSourceProjectCard.vue'
import { useHead } from '@/lib/head'
import '@/components/community/community.css'
import '@/components/community/open-source.css'
const route = useRoute()
const { data: catalog, loading, error, load } = useCommunityResource(() => openSourceBrowse.collection(String(route.params.key)))
watch(() => route.params.key, load, { immediate: true })
const collection = computed(() => catalog.value?.collections[0])
const projects = computed(() => catalog.value?.projects ?? [])
useHead(() => ({ title: collection.value ? `${collection.value.title} · 精选` : '专题未收录' }))
</script>
<template><main class="community-page os-detail-page"><nav class="os-detail-nav"><CommunityBack fallback="/open-source" label="精选" /><span>专题</span></nav><template v-if="collection && catalog"><header class="os-collection-lead"><div class="os-kicker"><DiscoverGlyph name="layers" :size="16" />专题精选 · {{ projects.length }} 个项目</div><h1>{{ collection.title }}</h1><p>{{ collection.description }}</p></header><div class="os-collection-list"><section v-for="(project, index) in projects" :key="project.full_name" class="os-collection-entry"><header><span>0{{ index + 1 }}</span><p>{{ collection.notes[index] }}</p></header><OpenSourceProjectCard :project="project" /></section></div><p class="os-source-note">{{ catalog.notice }}</p></template><RequestState v-else-if="loading || error" :loading="loading" :error="error" @retry="load" /><section v-else class="os-empty"><h1>这个专题尚未收录</h1><RouterLink to="/open-source" class="os-button">浏览精选</RouterLink></section></main></template>
