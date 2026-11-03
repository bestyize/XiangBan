<script setup lang="ts">
import { collectionPath } from '@/lib/openSourcePaths'
import type { OpenSourceCollection } from '@/lib/openSource'
import DiscoverGlyph from './DiscoverGlyph.vue'
defineProps<{ collection: Pick<OpenSourceCollection, 'key' | 'title' | 'summary' | 'projects'> & { project_names: string[] } }>()
</script>
<template>
  <RouterLink :to="collectionPath(collection.key)" class="os-collection-card">
    <div class="os-collection-heading"><span><DiscoverGlyph name="layers" :size="16" />专题精选</span><span>{{ collection.projects.length }} 个项目<DiscoverGlyph name="arrow" :size="16" /></span></div>
    <h3>{{ collection.title }}</h3><p>{{ collection.summary }}</p>
    <div class="os-collection-projects"><span v-for="name in collection.project_names" :key="name">{{ name }}</span></div>
  </RouterLink>
</template>
