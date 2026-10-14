<script setup lang="ts">
import { ref } from 'vue'
import { api, attachmentUrl, downloadAttachment } from '@/api/forum'
import type { Attachment } from '@/api/types'
import { formatSize } from '@/lib/format'
import { useToast } from '@/stores/toast'
import { useSession } from '@/stores/session'

/** restricted：帖子有阅读等级限制，链接带不上登录令牌，改为带令牌取回后保存。 */
defineProps<{ items: Attachment[]; restricted: boolean }>()
const toast = useToast()
const session = useSession()
const busy = ref<string | null>(null)

async function save(att: Attachment) {
  if (busy.value) return
  if (att.price > 0 && !session.requireLogin()) return
  if (att.price > 0 && !att.purchased && !window.confirm(`购买「${att.name}」需要 ${att.price} 积分，重复下载免费。确认购买？`)) return
  busy.value = att.id
  try {
    if (att.price > 0 && !att.purchased) {
      const result = await api.purchaseAttachment(att.id, att.price)
      att.purchased = result.purchased
      if (session.info) session.info.user.points = result.balance
      if (result.charged) toast.show(`已购买，扣除 ${result.charged} 积分`)
    }
    await downloadAttachment(att)
  } catch (e) {
    toast.error(e)
  } finally {
    busy.value = null
  }
}
</script>

<template>
  <section class="attachments" aria-label="附件">
    <h2>附件 <span>{{ items.length }}</span></h2>
    <ul>
      <li v-for="a in items" :key="a.id">
        <button v-if="restricted || a.price > 0" type="button" class="file" :disabled="!!busy" @click="save(a)">
          <span class="name">{{ a.name }}</span>
          <span class="size">{{ formatSize(a.size) }}</span>
          <span class="price">{{ busy === a.id ? '下载中…' : a.price > 0 ? a.purchased ? '已解锁 · 下载' : `${a.price} 积分 · 购买` : '免费下载' }}</span>
        </button>
        <a v-else class="file" :href="attachmentUrl(a.id)" :download="a.name">
          <span class="name">{{ a.name }}</span>
          <span class="size">{{ formatSize(a.size) }}</span>
          <span class="price">免费下载</span>
        </a>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.attachments {
  margin: 18px 0 0;
}
.price { color: var(--brand); font-size: var(--fs-xs); white-space: nowrap; }
.file:disabled { opacity: .65; cursor: wait; }
@media(max-width:480px) { .file { flex-wrap: wrap; gap: 5px 8px; } .name { flex-basis: 65%; } }
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
