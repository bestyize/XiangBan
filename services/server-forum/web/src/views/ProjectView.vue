<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import { ApiError } from '@/api/forum'
import { contentApi, type ProjectDetail } from '@/api/content'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

const route = useRoute()
const toast = useToast()
const project = ref<ProjectDetail | null>(null)
const missing = ref(false)

useHead(() => ({ title: project.value?.name ?? '项目' }))

watch(
  () => route.params.slug,
  async slug => {
    project.value = null
    missing.value = false
    try {
      project.value = await contentApi.project(String(slug))
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) missing.value = true
      else toast.error(e)
    }
  },
  { immediate: true },
)
</script>

<template>
  <ContentShell>
    <p v-if="missing" class="state">这个项目不存在。<RouterLink to="/projects">看全部项目</RouterLink></p>
    <p v-else-if="!project" class="state">加载中</p>
    <article v-else class="project">
      <h1>{{ project.name }}</h1>
      <p v-if="project.summary" class="lead">{{ project.summary }}</p>
      <p v-if="project.links.length" class="links">
        <a v-for="l in project.links" :key="l.url" :href="l.url" target="_blank" rel="noopener noreferrer">{{ l.title }}</a>
      </p>
      <!-- 正文由后端渲染并经过净化（ammonia）。 -->
      <div class="prose" v-html="project.body_html" />
    </article>
  </ContentShell>
</template>

<style scoped>
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.project {
  padding: 16px;
}
h1 {
  margin: 0;
  font-size: var(--fs-xl);
  font-weight: 600;
}
.lead {
  margin: 8px 0 0;
  color: var(--ink-2);
}
.links {
  margin: 10px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 16px;
  font-size: var(--fs-sm);
}
.links a {
  color: var(--brand);
}
.prose {
  margin-top: 14px;
}
</style>
