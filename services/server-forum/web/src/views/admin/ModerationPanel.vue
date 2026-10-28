<script setup lang="ts">
// 待审队列：用户新发布的内容包，以及命中“复核词”的帖子、回复、方案。
import { onMounted, ref } from 'vue'
import { adminApi, type ModerationRow } from '@/api/admin'
import { kindLabel } from '@/lib/content'
import { fmtTime, perform } from './shared'

const TYPE: Record<string, string> = { pack: '内容包', topic: '帖子', post: '回复', scheme: '方案' }
const rows = ref<ModerationRow[]>([])
const loading = ref(true)

async function load() {
  loading.value = true
  try {
    rows.value = (await adminApi.moderationQueue()).items
  } finally {
    loading.value = false
  }
}

async function resolve(r: ModerationRow, action: 'approve' | 'reject') {
  let note = ''
  if (action === 'reject') {
    note = window.prompt('拒绝原因（会记录在操作日志里）') ?? ''
    if (!note.trim()) return
  }
  if (await perform(() => adminApi.resolveModeration(r.id, action, note.trim()), '已处理')) await load()
}

/** 帖子和回复要在帖子管理里隐藏；这里只标记“已处理”。 */
function link(r: ModerationRow): string {
  if (r.target_type === 'pack') return `/packs/${r.target_id}`
  if (r.target_type === 'topic') return `/t/${r.target_id}`
  if (r.target_type === 'post' && r.topic_id) return `/t/${r.topic_id}`
  return ''
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <button class="btn" :disabled="loading" @click="load">刷新</button>
  </div>
  <div class="panel adm-card">
    <p v-if="loading && !rows.length" class="adm-state muted">加载中</p>
    <p v-else-if="!rows.length" class="adm-empty">没有待审内容</p>
    <table v-else class="adm-table">
      <thead>
        <tr><th>内容</th><th>类型</th><th>作者</th><th>原因</th><th>提交时间</th><th>操作</th></tr>
      </thead>
      <tbody>
        <tr v-for="r in rows" :key="r.id">
          <td data-label="内容" class="adm-wrap">
            <RouterLink v-if="link(r)" :to="link(r)" target="_blank">{{ r.title ?? `#${r.target_id}` }}</RouterLink>
            <template v-else>{{ r.title ?? `#${r.target_id}` }}</template>
          </td>
          <td data-label="类型">{{ TYPE[r.target_type] ?? r.target_type }}<template v-if="r.kind">（{{ kindLabel(r.kind) }}）</template></td>
          <td data-label="作者">{{ r.author ?? '-' }}</td>
          <td data-label="原因" class="adm-wrap adm-note">{{ r.note || '新发布，等待审核' }}</td>
          <td data-label="提交时间" class="shrink">{{ fmtTime(r.created_at) }}</td>
          <td data-label="操作" class="adm-actions">
            <button class="btn btn-sm btn-primary" @click="resolve(r, 'approve')">{{ r.target_type === 'topic' || r.target_type === 'post' ? '已处理' : '通过' }}</button>
            <button v-if="r.target_type === 'pack' || r.target_type === 'scheme'" class="btn btn-sm" @click="resolve(r, 'reject')">拒绝</button>
            <RouterLink v-else-if="link(r)" class="btn btn-sm" to="/admin/topics">去帖子管理</RouterLink>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
