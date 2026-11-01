<script setup lang="ts">
import { ref, watch } from 'vue'
import type { PackItem } from '@/api/content'
import { kindLabel } from '@/lib/content'
import UserAvatar from '@/components/UserAvatar.vue'
import { webLink } from './contract'
const props = defineProps<{ pack:PackItem }>()
const imageFailed=ref(false)
watch(() => props.pack.cover_url, () => { imageFailed.value=false })
</script>
<template><RouterLink :to="`/packs/${pack.id}`" class="pack-card" :class="{visual: !!webLink(pack.cover_url) && !imageFailed}"><div v-if="webLink(pack.cover_url) && !imageFailed" class="cover"><img :src="webLink(pack.cover_url)" alt="" loading="lazy" :class="{contain:pack.kind === 'game' || pack.kind === 'widget'}" @error="imageFailed = true"><span class="cover-kind">{{ kindLabel(pack.kind) }}</span></div><div class="copy"><span v-if="!webLink(pack.cover_url) || imageFailed" class="kind">{{ kindLabel(pack.kind) }}</span><h3>{{ pack.title }}</h3><p v-if="pack.summary && !webLink(pack.cover_url)" class="summary">{{ pack.summary }}</p><div v-if="pack.author?.nickname || pack.apply_count > 0" class="author"><UserAvatar v-if="pack.author?.nickname" :name="pack.author.nickname" :size="20" /><span class="author-name">{{ pack.author?.nickname || '社区作品' }}</span><small v-if="pack.apply_count > 0">{{ pack.apply_count }} 次使用</small></div></div></RouterLink></template>
<style scoped>
.pack-card{display:block;min-width:0;background:var(--surface);border-radius:12px;overflow:hidden;color:var(--ink)}.cover{position:relative;background:var(--surface-2);aspect-ratio:1.5;overflow:hidden}.cover img{display:block;width:100%;height:100%;object-fit:cover}.cover img.contain{object-fit:contain}.cover-kind{position:absolute;left:8px;top:8px;background:#0009;color:#fff;border-radius:4px;font-size:12px;padding:2px 6px}.copy{padding:10px 12px 12px}.kind{font-size:12px;color:var(--ink-3)}h3{font-size:15px;font-weight:600;line-height:1.4;margin:4px 0 0;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere}.summary{font-size:12px;color:var(--ink-3);margin:6px 0 0;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}.author{display:flex;align-items:center;gap:6px;margin-top:10px;font-size:12px;color:var(--ink-3);min-width:0}.author>.author-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1}.author small{white-space:nowrap;font-size:12px}
</style>
