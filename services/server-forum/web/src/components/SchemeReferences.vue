<script setup lang="ts">
import type { SchemeReference } from '@/api/schemes'
import { kindLabel } from '@/lib/content'
defineProps<{ items: SchemeReference[] }>()
function readable(item: SchemeReference) { return item.available === true && Number.isSafeInteger(item.pack_id) && item.pack_id > 0 }
</script>
<template>
  <ol class="scheme-references">
    <li v-for="(item, index) in items" :key="`${item.pack_id}-${index}`">
      <RouterLink v-if="readable(item)" :to="`/packs/${item.pack_id}`">{{ item.title || item.slug || `作品 ${index + 1}` }} <span v-if="item.kind">· {{ kindLabel(item.kind) }}</span></RouterLink>
      <span v-else class="unavailable">第 {{ index + 1 }} 项作品已下架或暂不可用</span>
      <span v-if="item.version != null" class="version">v{{ item.version }}</span>
      <span v-else class="version">未指定版本</span>
    </li>
  </ol>
</template>
<style scoped>
.scheme-references { margin: 0; padding: 0; list-style: none; display: grid; gap: 8px; }
.scheme-references li { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; }
.version, .unavailable { color: var(--ink-3); font-size: var(--fs-sm); }
</style>
