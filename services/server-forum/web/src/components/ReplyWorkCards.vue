<script setup lang="ts">
import type { ReplyWorkCard } from '@/api/replyContent'
defineProps<{ items: ReplyWorkCard[] }>()
</script>
<template>
  <ul v-if="items.length" class="work-cards" aria-label="回复引用的作品与方案"><li v-for="item in items" :key="`${item.kind}:${item.id}`">
    <RouterLink v-if="item.available" :to="`/${item.kind === 'pack' ? 'packs' : 'schemes'}/${item.id}`"><span class="type">{{ item.kind === 'pack' ? '内容包' : '方案' }}<template v-if="item.version"> · v{{ item.version }}</template></span><strong>{{ item.content.title }}</strong><span v-if="item.content.summary" class="summary">{{ item.content.summary }}</span></RouterLink>
    <span v-else class="unavailable">引用的{{ item.kind === 'pack' ? '作品或版本' : '方案' }}当前不可用</span>
  </li></ul>
</template>
<style scoped>
.work-cards { padding:0; margin:12px 0; list-style:none; display:grid; gap:8px; }.work-cards li { min-width:0; border:1px solid var(--line); border-radius:var(--r-sm); background:var(--surface-2); }.work-cards a { padding:10px 12px; display:grid; gap:4px; }.type,.summary,.unavailable { font-size:var(--fs-sm); color:var(--ink-3); }.unavailable { display:block; padding:12px; }.work-cards strong,.summary { overflow-wrap:anywhere; }.summary { overflow:hidden; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; }
</style>
