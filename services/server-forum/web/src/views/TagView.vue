<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import RightRail from '@/components/RightRail.vue'
import TopicList from '@/components/TopicList.vue'
import PagerNav from '@/components/PagerNav.vue'
import { api } from '@/api/forum'
import { useTopicList } from '@/lib/useList'
import { useHead } from '@/lib/head'

const route = useRoute()
const name = computed(() => String(route.params.name))
useHead(() => ({ title: `#${name.value}` }))

const list = useTopicList(p => api.topics({ tag: name.value, sort: 'new', include_expired: true, page: p }), [name])
</script>

<template>
  <ForumShell :rail="true">
    <header class="head">
      <h1>#{{ name }}</h1>
      <p class="muted">{{ list.total.value }} 个帖子</p>
    </header>
    <TopicList :items="list.items.value" :loading="list.loading.value" empty="这个标签下还没有帖子。" />
    <PagerNav :page="list.page.value" :page-size="list.pageSize.value" :total="list.total.value" @go="list.go" />
    <template #rail>
      <RightRail />
    </template>
  </ForumShell>
</template>

<style scoped>
.head {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 14px;
}
h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
  overflow-wrap: anywhere;
}
.head p {
  margin: 0;
  font-size: var(--fs-sm);
}
</style>
