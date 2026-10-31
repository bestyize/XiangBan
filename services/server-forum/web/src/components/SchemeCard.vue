<script setup lang="ts">
import type { CommunityScheme } from '@/api/schemes'
import SchemeReferences from './SchemeReferences.vue'
import { timeAgo } from '@/lib/format'
defineProps<{ scheme: CommunityScheme }>()
</script>
<template>
  <article class="scheme-card">
    <h3><RouterLink :to="`/schemes/${scheme.id}`">{{ scheme.title }}</RouterLink></h3>
    <p v-if="scheme.summary" class="summary">{{ scheme.summary }}</p>
    <p class="meta"><RouterLink :to="`/u/${encodeURIComponent(scheme.author.uid)}`">{{ scheme.author.nickname || '社区作者' }}</RouterLink><span>{{ scheme.items.length }} 项作品</span><span>{{ timeAgo(scheme.created_at) }}</span><span v-if="scheme.apply_count > 0">{{ scheme.apply_count }} 次复刻</span></p>
    <SchemeReferences :items="scheme.items" />
  </article>
</template>
<style scoped>
.scheme-card { padding: 16px; border-bottom: 1px solid var(--line); }
h3 { margin: 0; font-size: var(--fs-md); font-weight: 600; }
.summary { margin: 8px 0; color: var(--ink-2); font-size: var(--fs-sm); }
.meta { display: flex; flex-wrap: wrap; gap: 4px 12px; margin: 8px 0 12px; color: var(--ink-3); font-size: var(--fs-xs); }
</style>
