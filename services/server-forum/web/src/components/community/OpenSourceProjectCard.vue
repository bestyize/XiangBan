<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { OpenSourceProjectSummary } from '@/lib/openSource'
import { projectPath } from '@/lib/openSourcePaths'
import OpenSourceCover from './OpenSourceCover.vue'
import { curatedProjectAbout } from '@/lib/githubProjectAbout'
import ProjectIcon from './ProjectIcon.vue'
import cardMedia from '@/lib/openSourceCardMedia.json'
const props = defineProps<{ project: OpenSourceProjectSummary; coverPreview?: boolean }>()
// Published screenshot covers also include repository home pages. Only show
// the product screenshots and photos reviewed in public/open-source-media.
const media: Partial<Record<string, string>> = cardMedia
const brokenCover = ref(false)
const project = computed(() => {
  const source = props.project
  const summary = curatedProjectAbout(source.full_name, source.summary)
  return { ...source, summary }
})
const cover = computed(() => media[project.value.full_name.toLowerCase()])
const hasCover = computed(() => props.coverPreview || (!!cover.value && !brokenCover.value))
watch(() => [props.project.full_name, props.project.icon_url, cover.value], () => {
  brokenCover.value = false
})
</script>
<template>
  <article class="os-project-card os-project-card--curated" :class="{ 'os-project-card--media': hasCover }">
    <RouterLink v-if="hasCover" :to="projectPath(project.full_name)" class="os-card-cover" tabindex="-1" aria-hidden="true">
      <OpenSourceCover v-if="coverPreview" :project="project" preview />
      <img v-else :src="cover" alt="" loading="lazy" @error="brokenCover = true">
    </RouterLink>
    <div class="os-card-copy">
      <div class="os-card-identity">
        <ProjectIcon :name="project.name" :src="project.icon_url" :fit="project.icon_fit" />
        <div class="os-card-name">
          <h3><RouterLink :to="projectPath(project.full_name)">{{ project.name }}</RouterLink></h3>
          <span class="os-card-repository" :title="project.full_name">{{ project.full_name }}</span>
        </div>
      </div>
      <p v-if="project.summary" class="os-card-summary">{{ project.summary }}</p>
      <div class="os-tags"><span v-for="tag in project.tags.slice(0, 2)" :key="tag">{{ tag }}</span></div>
    </div>
  </article>
</template>
<style scoped>
.os-project-card--curated {
  align-self: start;
  border-radius: 12px;
  transition: border-color .15s;
}
.os-project-card--curated:hover,
.os-project-card--curated:focus-within {
  border-color: color-mix(in srgb, var(--brand) 45%, var(--line));
}
.os-card-cover img { object-fit: contain; }
.os-card-identity { display: flex; align-items: center; gap: 10px; min-width: 0; }
.os-card-name { min-width: 0; flex: 1; }
.os-card-name h3 { margin: 0; font-size: 14px; line-height: 22px; font-weight: 400; }
.os-card-name h3 a {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}
.os-card-repository {
  display: block;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ink-3);
  font: 11px/1.4 ui-monospace, monospace;
}
.os-card-summary {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
  min-height: 3.2em;
  margin: 12px 0 0;
  color: var(--ink-2);
  font-size: 13px;
  line-height: 1.6;
}
.os-tags { margin-top: 14px; gap: 6px; }
.os-tags > span { padding: 3px 8px; }
@media (prefers-reduced-motion: reduce) {
  .os-project-card--curated { transition: none; }
}
</style>
