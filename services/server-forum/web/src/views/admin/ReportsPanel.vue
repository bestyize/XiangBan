<script setup lang="ts">
// 举报队列：主从双栏工作流（左侧排队列表 + 右侧检视与快捷处置面板）。
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { adminApi, type AdminReport } from '@/api/admin'
import AdminIcon from './AdminIcon.vue'
import AdminPager from './AdminPager.vue'
import {
  fmtTime,
  openReports,
  perform,
  refreshReviewBadges,
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

const QUICK_NOTES = [
  '确认违反社区规范，已隐藏相关内容',
  '广告引流或违规交易属实，已处理并隐藏',
  '核实内容未违反社区规范，驳回本次举报',
]

const list = usePagedList<AdminReport>(
  (page, size) => adminApi.reports({ status: status.value, page, page_size: size }),
  20,
)

const selectedId = ref<number | null>(null)
const notes = reactive<Record<number, string>>({})
const busyId = ref(0)

const activeReport = computed<AdminReport | null>(() => {
  const rows = list.items.value
  if (!rows.length) return null
  return rows.find(r => r.id === selectedId.value) ?? rows[0] ?? null
})

watch(
  () => list.items.value,
  rows => {
    if (!rows.length) {
      selectedId.value = null
    } else if (!rows.some(r => r.id === selectedId.value)) {
      selectedId.value = rows[0]!.id
    }
  },
)

function pick(s: typeof status.value) {
  status.value = s
  void list.search()
}

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

function targetTitle(r: AdminReport): string {
  if (r.target.type === 'topic') return r.target.title || `帖子 #${r.target.id}`
  if (r.target.exists && r.target.topic_title) {
    return `${r.target.topic_title}${r.target.floor ? `（第 ${r.target.floor} 楼）` : ''}`
  }
  return `回复 #${r.target.id}`
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
    void refreshReviewBadges()
  }
}

onMounted(async () => {
  await list.load()
  if (status.value === 'open') openReports.value = list.total.value
})
</script>

<template>
  <div class="adm-toolbar is-spread">
    <div class="adm-seg" role="group" aria-label="举报状态">
      <button v-for="t in STATUS_TABS" :key="t.key" type="button" :class="{ 'is-on': status === t.key }" @click="pick(t.key)">
        {{ t.label }}
      </button>
    </div>
    <button class="btn btn-sm" type="button" :disabled="list.loading.value" @click="list.load()">
      <AdminIcon name="refresh" :size="13" /> 刷新
    </button>
  </div>

  <div v-if="list.error.value" class="panel adm-card">
    <p class="adm-state error-text">{{ list.error.value }}</p>
  </div>
  <div v-else-if="list.loading.value && !list.items.value.length" class="panel adm-card">
    <p class="adm-state muted">正在读取举报队列…</p>
  </div>
  <div v-else-if="!list.items.value.length" class="panel adm-card">
    <p class="adm-empty">没有{{ status === 'open' ? '待处理的' : '' }}举报</p>
  </div>
  <div v-else class="adm-split">
    <section class="panel adm-card">
      <div class="adm-card-head">
        <h3>举报队列（{{ list.total.value }}）</h3>
        <span class="adm-tag line">{{ status === 'open' ? '最早优先' : '最近处理' }}</span>
      </div>
      <ul class="adm-queue">
        <li
          v-for="r in list.items.value"
          :key="r.id"
          :class="{ 'is-selected': activeReport?.id === r.id }"
          @click="selectedId = r.id"
        >
          <div style="display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-bottom: 5px">
            <span class="adm-tags">
              <span class="adm-tag danger">{{ REPORT_REASONS[r.reason] ?? r.reason }}</span>
              <span v-if="r.report_count > 1" class="adm-tag line">{{ r.report_count }} 次举报</span>
              <span v-if="r.status !== 'open'" class="adm-tag" :class="r.status === 'resolved' ? 'valid' : 'expired'">
                {{ REPORT_STATUS[r.status] }}
              </span>
            </span>
            <span class="adm-mono">{{ fmtTime(r.created_at).slice(5) }}</span>
          </div>
          <div class="adm-title" style="font-size: 13px; line-height: 1.45">{{ targetTitle(r) }}</div>
          <div class="adm-sub">
            <span>{{ r.target.type === 'topic' ? '帖子' : '回复' }} #{{ r.target.id }}</span>
            <span>举报人 {{ r.reporter.nickname || r.reporter.uid.slice(0, 8) }}</span>
          </div>
        </li>
      </ul>
      <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value" @go="list.goto" />
    </section>

    <section v-if="activeReport" class="panel adm-card">
      <div class="adm-card-head">
        <div class="adm-tags">
          <span class="adm-tag danger">{{ REPORT_REASONS[activeReport.reason] ?? activeReport.reason }}</span>
          <span v-if="activeReport.report_count > 1" class="adm-tag line">该内容被举报 {{ activeReport.report_count }} 次</span>
          <span v-if="!activeReport.target.exists" class="adm-tag expired">内容已不存在</span>
          <span v-else-if="activeReport.target.status" class="adm-tag" :class="topicTone(activeReport.target.status)">
            {{ TOPIC_STATUS[activeReport.target.status] ?? activeReport.target.status }}
          </span>
        </div>
        <a v-if="targetHref(activeReport)" class="btn btn-sm" :href="targetHref(activeReport)!" target="_blank" rel="noopener">
          <AdminIcon name="external" :size="13" /> 查看前台原页
        </a>
      </div>

      <div class="adm-card-body">
        <h2 style="margin: 0 0 8px; font-size: 17px; line-height: 1.4">{{ targetTitle(activeReport) }}</h2>
        <div class="adm-sub" style="margin-bottom: 14px">
          <span>{{ activeReport.target.type === 'topic' ? '帖子' : '回复' }} #{{ activeReport.target.id }}</span>
          <span>
            举报人
            <a class="adm-link" :href="`/u/${activeReport.reporter.uid}`" target="_blank" rel="noopener">
              {{ activeReport.reporter.nickname || activeReport.reporter.uid.slice(0, 8) }}
            </a>
          </span>
          <span>举报时间 {{ fmtTime(activeReport.created_at) }}</span>
        </div>

        <template v-if="activeReport.target.type === 'post' && activeReport.target.excerpt">
          <div class="adm-eyebrow" style="margin-bottom: 6px">被举报回复摘录</div>
          <blockquote class="adm-quote">{{ activeReport.target.excerpt }}</blockquote>
        </template>

        <template v-if="activeReport.detail">
          <div class="adm-eyebrow" style="margin: 12px 0 6px">举报人补充说明</div>
          <p class="adm-quote" style="margin-top: 0">{{ activeReport.detail }}</p>
        </template>

        <div v-if="activeReport.status === 'open'" style="border-top: 1px solid var(--adm-line); padding-top: 16px; margin-top: 16px">
          <div class="adm-eyebrow" style="margin-bottom: 8px">快捷处理说明</div>
          <div class="adm-chips">
            <button v-for="txt in QUICK_NOTES" :key="txt" type="button" class="adm-chip-btn" @click="notes[activeReport.id] = txt">
              {{ txt }}
            </button>
          </div>
          <label class="field" style="margin-bottom: 14px">
            <span>处理说明（可选，记入审计日志）</span>
            <textarea
              v-model="notes[activeReport.id]"
              class="textarea"
              rows="2"
              maxlength="500"
              placeholder="输入处理说明…"
            />
          </label>
          <div class="adm-actions">
            <button
              class="btn btn-primary"
              type="button"
              :disabled="busyId === activeReport.id || !canHide(activeReport)"
              :title="canHide(activeReport) ? '' : '内容已不存在或已不是正常状态'"
              @click="handle(activeReport, 'resolve', true)"
            >
              处理并隐藏内容
            </button>
            <button class="btn" type="button" :disabled="busyId === activeReport.id" @click="handle(activeReport, 'resolve', false)">
              处理不隐藏
            </button>
            <button class="btn btn-ghost" type="button" :disabled="busyId === activeReport.id" @click="handle(activeReport, 'dismiss', false)">
              驳回举报
            </button>
          </div>
        </div>

        <div v-else class="adm-callout" style="margin-top: 16px">
          处理人：<strong>{{ activeReport.handled_by || '-' }}</strong> · 处理时间 {{ fmtTime(activeReport.handled_at) }}
          <div v-if="activeReport.result" style="margin-top: 4px">处理说明：{{ activeReport.result }}</div>
        </div>
      </div>
    </section>
  </div>
</template>
