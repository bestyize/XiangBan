<script setup lang="ts">
import type { OpenSourceProjectSummary } from '@/lib/openSource'
import { projectPath } from '@/lib/openSourcePaths'
import OpenSourceCover from './OpenSourceCover.vue'
defineProps<{ project: OpenSourceProjectSummary }>()
</script>
<template>
  <article class="os-project-card">
    <RouterLink :to="projectPath(project.full_name)" class="os-card-cover os-composed-cover" tabindex="-1" aria-hidden="true">
      <OpenSourceCover :project="project" />
    </RouterLink>
    <div class="os-card-copy">
      <h3><RouterLink :to="projectPath(project.full_name)" :aria-label="`${project.name}：${project.title}`">{{ project.title }}</RouterLink></h3>
      <div class="os-tags"><span v-for="tag in project.tags.slice(0, 2)" :key="tag">{{ tag }}</span></div>
    </div>
  </article>
</template>
