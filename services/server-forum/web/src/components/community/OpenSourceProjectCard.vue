<script setup lang="ts">
import { computed } from 'vue'
import type { OpenSourceProjectSummary } from '@/lib/openSource'
import { projectPath } from '@/lib/openSourcePaths'
import OpenSourceCover from './OpenSourceCover.vue'
import coverSummaries from '@/lib/openSourceCoverSummaries.json'
const props = defineProps<{ project: OpenSourceProjectSummary; coverPreview?: boolean }>()
const summaries: Partial<Record<string, string>> = coverSummaries
const project = computed(() => {
  const source = props.project
  const summary = summaries[source.full_name.toLowerCase()] ?? source.summary
  return { ...source, summary }
})
</script>
<template>
  <article class="os-project-card">
    <RouterLink :to="projectPath(project.full_name)" class="os-card-cover os-composed-cover" tabindex="-1" aria-hidden="true">
      <OpenSourceCover :project="project" :preview="coverPreview" />
    </RouterLink>
    <div class="os-card-copy">
      <h3><RouterLink :to="projectPath(project.full_name)" :aria-label="`${project.name}：${project.title}`">{{ project.title }}</RouterLink></h3>
      <p v-if="project.summary" class="os-card-summary">{{ project.summary }}</p>
      <div class="os-tags"><span v-for="tag in project.tags.slice(0, 2)" :key="tag">{{ tag }}</span></div>
    </div>
  </article>
</template>
<style scoped>
.os-card-summary {
  margin: 0;
  color: var(--ink-2);
  font-size: 13px;
  line-height: 1.5;
}
</style>
