<script setup lang="ts">
import { computed } from 'vue'
import { discoverApi } from '@/api/discover'
import CommentSection from '@/components/comments/CommentSection.vue'
import type { CommentAdapter } from '@/components/comments/types'

const props = defineProps<{ owner: string; repo: string }>()
const adapter = computed<CommentAdapter>(() => {
  const { owner, repo } = props
  return { list: page => discoverApi.repoComments(owner, repo, page), post: (body, to) => discoverApi.repoComment(owner, repo, body, to), like: (id, on) => discoverApi.likeRepoComment(owner, repo, id, on), remove: id => discoverApi.deleteRepoComment(owner, repo, id) }
})
</script>

<template>
  <CommentSection :scope="`${owner}/${repo}`" :adapter="adapter">
    <template #actions><slot name="actions" /></template>
  </CommentSection>
</template>
