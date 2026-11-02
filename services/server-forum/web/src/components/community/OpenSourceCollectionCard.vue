<script setup lang="ts">
import { computed } from 'vue'
import { collectionPath, collectionProjects, type OpenSourceCollection } from '@/lib/openSource'
import DiscoverGlyph from './DiscoverGlyph.vue'
import { openSourceCatalog } from '@/stores/openSource'
const props = defineProps<{ collection: OpenSourceCollection }>()
const projects = computed(() => collectionProjects(props.collection, openSourceCatalog))
</script>
<template>
  <RouterLink :to="collectionPath(collection.key)" class="os-collection-card">
    <div class="os-collection-heading"><span><DiscoverGlyph name="layers" :size="16" />专题精选</span><span>{{ projects.length }} 个项目<DiscoverGlyph name="arrow" :size="16" /></span></div>
    <h3>{{ collection.title }}</h3><p>{{ collection.summary }}</p>
    <div class="os-collection-projects"><span v-for="project in projects" :key="project.full_name">{{ project.name }}</span></div>
  </RouterLink>
</template>
