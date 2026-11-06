<script setup lang="ts">
import { computed } from 'vue'
import { discoverApi } from '@/api/discover'
import CommentSection from '@/components/comments/CommentSection.vue'
import type { CommentAdapter } from '@/components/comments/types'

const props = defineProps<{ articleKey: string; frontier?: boolean }>()
const adapter = computed<CommentAdapter>(() => {
  const key = props.articleKey
  return props.frontier
    ? { list: page => discoverApi.frontierComments(key, page), post: (body, to) => discoverApi.frontierComment(key, body, to), like: (id, on) => discoverApi.likeFrontierComment(key, id, on), remove: id => discoverApi.deleteFrontierComment(key, id) }
    : { list: page => discoverApi.comments(key, page), post: (body, to) => discoverApi.comment(key, body, to), like: (id, on) => discoverApi.likeComment(key, id, on), remove: id => discoverApi.deleteComment(key, id) }
})
</script>

<template><CommentSection :scope="`${frontier ? 'frontier' : 'daily'}:${articleKey}`" :adapter="adapter" column /></template>
