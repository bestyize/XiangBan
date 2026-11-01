<script setup lang="ts">
import type { NewsItem } from '@/api/discover'
import { displayDate, webLink } from './contract'
import RankingVendor from './RankingVendor.vue'
defineProps<{ item: NewsItem }>()
</script>
<template><RouterLink :to="`/daily/${encodeURIComponent(item.key)}`" class="news-row"><div class="news-copy"><h3>{{ item.title }}</h3><div class="news-meta"><RankingVendor v-if="item.source" :name="item.source" aria-hidden="true"/><span v-if="item.source">{{ item.source }}</span><span v-if="item.source&&(item.source_published_at||item.published_at)" aria-hidden="true">·</span><time>{{ item.time_basis==='collected'?'采集于 ':'' }}{{ displayDate(item.source_published_at||item.published_at) }}</time><span v-if="item.sample" class="sample">示例</span></div></div><img v-if="webLink(item.cover_url)" :src="webLink(item.cover_url)" alt="" loading="lazy" @error="($event.target as HTMLImageElement).style.display='none'"></RouterLink></template>
<style scoped>
.news-row{display:flex;align-items:center;gap:12px;min-height:88px;padding:16px;background:var(--surface);border-radius:16px;color:var(--ink);transition:background .15s}.news-row:hover{background:color-mix(in srgb,var(--surface-2) 35%,var(--surface))}.news-copy{min-width:0;flex:1}h3{font-size:15px;font-weight:600;line-height:1.65;margin:0 0 10px;overflow-wrap:anywhere}.news-meta{display:flex;align-items:center;flex-wrap:wrap;gap:4px 6px;color:var(--ink-3);font-size:12px;line-height:1.5}.news-meta :deep(.ranking-vendor){width:14px;height:14px;flex-basis:14px}img{flex:none;width:88px;height:68px;object-fit:cover;border-radius:10px}.sample{font-size:12px;color:var(--brand);padding:1px 5px;background:color-mix(in srgb,var(--brand) 7%,var(--surface));border-radius:4px}@media(min-width:768px){.news-row{padding:20px}h3{font-size:16px}img{width:120px;height:80px}}
</style>
