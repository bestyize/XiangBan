<script setup lang="ts">
import { onMounted, ref } from 'vue'
import ContentShell from '@/components/ContentShell.vue'
import { contentApi, type ProjectItem } from '@/api/content'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '项目' }))
const toast = useToast()
const items = ref<ProjectItem[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    items.value = (await contentApi.projects()).items
  } catch (e) {
    toast.error(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <ContentShell>
    <h1 class="sr-only">项目</h1>
    <p v-if="loading" class="state">加载中</p>
    <p v-else-if="!items.length" class="state">还没有项目。</p>
    <ul v-else class="projects">
      <li v-for="p in items" :key="p.slug">
        <RouterLink :to="`/projects/${p.slug}`">
          <h2>{{ p.name }}</h2>
          <p>{{ p.summary }}</p>
        </RouterLink>
      </li>
    </ul>
  </ContentShell>
</template>

<style scoped>
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.projects {
  list-style: none;
  margin: 0;
  padding: 0 16px;
}
.projects li {
  border-bottom: 1px solid var(--line);
}
.projects a {
  display: block;
  padding: 14px 0;
}
h2 {
  margin: 0;
  font-size: var(--fs-lg);
  font-weight: 500;
}
p {
  margin: 4px 0 0;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
</style>
