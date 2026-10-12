<script setup lang="ts">
import { attachmentUrl, downloadAttachment } from '@/api/forum'
import type { Attachment } from '@/api/types'
import { formatSize } from '@/lib/format'
import { useToast } from '@/stores/toast'

/** restricted：帖子有阅读等级限制，链接带不上登录令牌，改为带令牌取回后保存。 */
defineProps<{ items: Attachment[]; restricted: boolean }>()
const toast = useToast()

async function save(att: Attachment) {
  try {
    await downloadAttachment(att)
  } catch (e) {
    toast.error(e)
  }
}
</script>

<template>
  <section class="attachments" aria-label="附件">
    <h2>附件 <span>{{ items.length }}</span></h2>
    <ul>
      <li v-for="a in items" :key="a.id">
        <button v-if="restricted" type="button" class="file" @click="save(a)">
          <span class="name">{{ a.name }}</span>
          <span class="size">{{ formatSize(a.size) }}</span>
        </button>
        <a v-else class="file" :href="attachmentUrl(a.id)" :download="a.name">
          <span class="name">{{ a.name }}</span>
          <span class="size">{{ formatSize(a.size) }}</span>
        </a>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.attachments {
  margin: 18px 0 0;
}
h2 {
  margin: 0 0 8px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
h2 span {
  color: var(--ink-3);
  font-weight: 400;
}
ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}
.file {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 12px;
  border: 1px solid var(--line);
  border-radius: var(--r-sm);
  background: var(--surface-2);
  color: var(--ink);
  font-size: var(--fs-sm);
  text-align: left;
  text-decoration: none;
  cursor: pointer;
}
.file:hover {
  border-color: var(--ink-3);
}
.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.size {
  color: var(--ink-3);
  font-size: var(--fs-xs);
  font-variant-numeric: tabular-nums;
}
</style>
