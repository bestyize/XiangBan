<script setup lang="ts">
// 举报队列：待处理按时间从早到晚，已处理按最近处理在前。
import { onMounted, reactive, ref } from 'vue'
import { adminApi, type AdminReport } from '@/api/admin'
import AdminPager from './AdminPager.vue'
import {
  fmtTime,
  openReports,
  perform,
  REPORT_REASONS,
  REPORT_STATUS,
  TOPIC_STATUS,
  topicTone,
  usePagedList,
} from './shared'

const status = ref<'open' | 'resolved' | 'dismissed' | 'all'>('open')
const STATUS_TABS = [
  { key: 'open', label: '待处理' },
  { key: 'resolved', label: '已处理' },
  { key: 'dismissed', label: '已驳回' },
  { key: 'all', label: '全部' },
] as const

const list = usePagedList<AdminReport>((page, size) => adminApi.reports({ status: status.value, page, page_size: size }), 20)

/** 每条举报各自的处理说明输入。 */
const notes = reactive<Record<number, string>>({})
const busyId = ref(0)

function pick(s: typeof status.value) {
  status.value = s
  list.search()
}

/** 被举报内容在前台的地址：帖子 /t/{id}，回复 /t/{帖子id}#floor-{楼层}。 */
function targetHref(r: AdminReport): string | null {
  const t = r.target
  if (!t.exists) return null
  if (t.type === 'topic') return `/t/${t.id}`
  if (!t.topic_id) return null
  return t.floor ? `/t/${t.topic_id}#floor-${t.floor}` : `/t/${t.topic_id}`
}

function canHide(r: AdminReport) {
  return r.target.exists && r.target.status === 'normal'
}

async function handle(r: AdminReport, action: 'resolve' | 'dismiss', hide: boolean) {
  busyId.value = r.id
  const ok = await perform(
    () =>
      adminApi.handleReport(r.id, {
        action,
        result: notes[r.id]?.trim() || undefined,
        hide_target: hide || undefined,
      }),
    action === 'dismiss' ? '已驳回举报' : hide ? '已处理，内容已隐藏' : '已处理举报',
  )
  busyId.value = 0
  if (ok) {
    delete notes[r.id]
    await list.load()
    // 顺手刷新侧栏角标
    adminApi.stats().then(s => (openReports.value = s.reports_open)).catch(() => {})
  }
}

onMounted(async () => {
  await list.load()
  if (status.value === 'open') openReports.value = list.total.value
})
</script>

<template>
  <div class="adm-toolbar">
    <div class="adm-seg" role="group" aria-label="举报状态">
      <button v-for="t in STATUS_TABS" :key="t.key" :class="{ 'is-on': status === t.key }" @click="pick(t.key)">
        {{ t.label }}
      </button>
    </div>
  </div>

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">加载中</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有{{ status === 'open' ? '待处理的' : '' }}举报</p>
    <ul v-else class="adm-list">
      <li v-for="r in list.items.value" :key="r.id">
        <div class="adm-report-head">
          <span class="adm-tag danger">{{ REPORT_REASONS[r.reason] ?? r.reason }}</span>
          <span v-if="r.report_count > 1" class="adm-tag line">该内容被举报 {{ r.report_count }} 次</span>
          <span v-if="r.status !== 'open'" class="adm-tag" :class="r.status === 'resolved' ? 'valid' : 'expired'">
            {{ REPORT_STATUS[r.status] }}
          </span>
          <span class="muted">{{ fmtTime(r.created_at) }}</span>
        </div>

        <div class="adm-target">
          <template v-if="r.target.type === 'topic'">
            <span class="muted">帖子 #{{ r.target.id }}</span>
            <a v-if="targetHref(r)" class="adm-link adm-title" :href="targetHref(r)!" target="_blank" rel="noopener">
              {{ r.target.title }}
            </a>
          </template>
          <template v-else>
            <span class="muted">回复 #{{ r.target.id }}</span>
            <template v-if="r.target.exists">
              <a v-if="targetHref(r)" class="adm-link adm-title" :href="targetHref(r)!" target="_blank" rel="noopener">
                {{ r.target.topic_title }} 第 {{ r.target.floor }} 楼
              </a>
            </template>
          </template>
          <span v-if="!r.target.exists" class="adm-tag expired">内容已不存在</span>
          <span v-else-if="r.target.status" class="adm-tag" :class="topicTone(r.target.status)">
            {{ TOPIC_STATUS[r.target.status] ?? r.target.status }}
          </span>
          <blockquote v-if="r.target.type === 'post' && r.target.excerpt" class="adm-quote">{{ r.target.excerpt }}</blockquote>
        </div>

        <div class="adm-sub" style="margin: 0">
          <span>
            举报人
            <a class="adm-link" :href="`/u/${r.reporter.uid}`" target="_blank" rel="noopener">
              {{ r.reporter.nickname || r.reporter.uid.slice(0, 8) }}
            </a>
          </span>
        </div>
        <p v-if="r.detail" class="adm-note" style="margin: 0">{{ r.detail }}</p>

        <div v-if="r.status === 'open'" class="adm-report-actions">
          <input v-model="notes[r.id]" class="input" maxlength="500" placeholder="处理说明（可选）" aria-label="处理说明" />
          <button
            class="btn btn-sm btn-primary"
            :disabled="busyId === r.id || !canHide(r)"
            :title="canHide(r) ? '' : '内容已不存在或已不是正常状态'"
            @click="handle(r, 'resolve', true)"
          >
            处理并隐藏内容
          </button>
          <button class="btn btn-sm" :disabled="busyId === r.id" @click="handle(r, 'resolve', false)">处理不隐藏</button>
          <button class="btn btn-sm" :disabled="busyId === r.id" @click="handle(r, 'dismiss', false)">驳回</button>
        </div>
        <div v-else class="adm-sub" style="margin: 0">
          <span>处理人 {{ r.handled_by || '-' }}</span>
          <span>处理时间 {{ fmtTime(r.handled_at) }}</span>
          <span v-if="r.result">说明 {{ r.result }}</span>
        </div>
      </li>
    </ul>
  </div>

  <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value" @go="list.goto" />
</template>
