<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { contentReportsApi, type ContentReport } from '@/api/contentSocial'
import { AdminApiError } from '@/api/admin'
import { fmtTime, REPORT_REASONS, REPORT_STATUS, usePagedList } from './shared'
import AdminPager from './AdminPager.vue'
const status = ref('open'), notes = reactive<Record<number, string>>({}), busy = ref(0), message = ref(''), handlingError = ref('')
const tabs = [{ key:'open', label:'待处理' },{ key:'resolved', label:'已下架处理' },{ key:'dismissed', label:'已驳回' },{ key:'all', label:'全部' }]
const list = usePagedList<ContentReport>(page => contentReportsApi.list(status.value, page), 20)
function pick(value: string) { if (busy.value) return; status.value = value; message.value = ''; handlingError.value = ''; list.search() }
async function handle(row: ContentReport, action: 'remove' | 'dismiss') {
  const note = notes[row.id]?.trim()
  if (!note || busy.value) return
  if (action === 'remove' && !window.confirm(`确定下架「${row.target.title || '该内容'}」并处理此内容的全部待审举报？`)) return
  busy.value = row.id; handlingError.value = ''; message.value = ''
  try { const result = await contentReportsApi.handle(row, action, note); delete notes[row.id]; message.value = action === 'dismiss' ? '已驳回举报，内容状态保留' : `已下架处理，共处理 ${result.reports_handled} 条举报`; await list.load() }
  catch (e) { handlingError.value = e instanceof Error ? e.message : '处理失败，请重试'; if (e instanceof AdminApiError && e.status === 409) await list.load() }
  finally { busy.value = 0 }
}
onMounted(() => list.load())
</script>
<template>
  <div class="adm-toolbar"><div class="adm-seg" role="group" aria-label="内容举报状态"><button v-for="tab in tabs" :key="tab.key" :disabled="!!busy" :class="{ 'is-on':status === tab.key }" @click="pick(tab.key)">{{ tab.label }}</button></div><button class="btn btn-sm" :disabled="list.loading.value || !!busy" @click="list.load()">刷新</button></div>
  <p class="muted">举报理由与处理说明仅供管理员审核。下架将同步隐藏收藏、公开详情与引用卡片。</p>
  <p v-if="message" role="status">{{ message }}</p><p v-if="handlingError" class="error-text" role="alert">{{ handlingError }}</p>
  <div class="adm-card"><p v-if="list.error.value" class="adm-state error-text" role="alert">{{ list.error.value }} <button class="btn" @click="list.load()">重试</button></p><p v-else-if="list.loading.value" class="adm-state">正在读取举报…</p><p v-else-if="!list.items.value.length" class="adm-empty">暂无对应状态的举报。</p>
    <ul v-else class="adm-list"><li v-for="row in list.items.value" :key="row.id" class="content-report">
      <div class="report-meta"><strong>{{ row.kind === 'pack' ? '内容包' : '方案' }} #{{ row.target_id }}</strong><span class="adm-tag">{{ REPORT_REASONS[row.reason] || row.reason }}</span><span>{{ REPORT_STATUS[row.status] }}</span><time>{{ fmtTime(row.created_at) }}</time></div>
      <a v-if="row.target.exists && row.target.status === 'published'" :href="`/${row.kind === 'pack' ? 'packs' : 'schemes'}/${row.target_id}`" target="_blank" rel="noopener" class="adm-link report-title">{{ row.target.title || '查看内容' }}</a><p v-else>{{ row.target.title || '内容已不存在' }}（已不可公开访问）</p>
      <p class="muted">举报人 {{ row.reporter.nickname || row.reporter.uid }}</p><p v-if="row.detail" class="report-note">{{ row.detail }}</p>
      <form v-if="row.status === 'open'" class="report-handling" @submit.prevent="handle(row, 'dismiss')"><label class="field"><span>内部处理说明（必填，不向作者公开）</span><textarea v-model="notes[row.id]" class="input" maxlength="500" rows="3" required :disabled="!!busy" /></label><div class="report-meta"><button type="button" class="btn btn-primary" :disabled="!!busy || !notes[row.id]?.trim()" @click="handle(row, 'remove')">下架并处理该内容全部举报</button><button class="btn" :disabled="!!busy || !notes[row.id]?.trim()">驳回此条举报</button></div></form>
      <div v-else class="report-result"><span>{{ row.handled_by }} · {{ fmtTime(row.handled_at) }}</span><p>{{ row.result }}</p></div>
    </li></ul>
  </div>
  <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value || !!busy" @go="list.goto" />
</template>
<style scoped>
.content-report { min-width:0; }.report-meta { display:flex; align-items:center; gap:8px 12px; flex-wrap:wrap; }.report-title,.report-note,.report-result { overflow-wrap:anywhere; white-space:pre-wrap; }.report-handling { display:grid; gap:12px; margin-top:12px; }.report-result { padding-top:12px; color:var(--ink-2); }.report-handling textarea { resize:vertical; }
</style>
