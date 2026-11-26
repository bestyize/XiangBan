<script setup lang="ts">
// 工作台概览：KPI 卡片 + 跨模块待办行动中心 + 采集源健康灯 + 最近审计动态。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type AdminStats, type AuditRow } from '@/api/admin'
import { operationsApi, type OperationsSummary } from '@/api/communityOperations'
import { contentReportsApi } from '@/api/contentSocial'
import AdminIcon from './AdminIcon.vue'
import {
  AUDIT_ACTIONS,
  AUDIT_TARGETS,
  errorText,
  fmtTime,
  openContentReports,
  openModeration,
  openReports,
  totalReviewTodo,
} from './shared'

const stats = ref<AdminStats | null>(null)
const ops = ref<OperationsSummary | null>(null)
const audits = ref<AuditRow[]>([])
const loading = ref(true)
const error = ref('')

const pendingDrafts = computed(() => ops.value?.columns.reduce((sum, col) => sum + col.pending, 0) ?? 0)
const scheduledDrafts = computed(() => ops.value?.columns.reduce((sum, col) => sum + col.scheduled, 0) ?? 0)
const attentionLinks = computed(() =>
  Object.entries(ops.value?.links.statuses ?? {}).reduce(
    (sum, [key, count]) => sum + (['healthy', 'unchecked'].includes(key) ? 0 : count),
    0,
  ),
)
const staleSources = computed(
  () => ops.value?.sources.filter(s => s.freshness === 'stale' || s.freshness === 'never').length ?? 0,
)
const totalActionItems = computed(
  () => totalReviewTodo.value + pendingDrafts.value + attentionLinks.value,
)

const FRESH_LABEL: Record<string, string> = {
  fresh: '更新正常',
  stale: '更新过期',
  never: '尚未成功',
  disabled: '已停用',
}

function sourceDot(freshness: string, hasError: boolean): string {
  if (hasError || freshness === 'stale') return 'warn'
  if (freshness === 'fresh') return 'ok'
  return ''
}

function sourceTone(freshness: string, hasError: boolean): string {
  if (hasError || freshness === 'stale') return 'suspect'
  if (freshness === 'fresh') return 'valid'
  return ''
}

async function load() {
  loading.value = true
  error.value = ''
  const [statsRes, contentRes, modRes, opsRes, auditRes] = await Promise.allSettled([
    adminApi.stats(),
    contentReportsApi.list('open', 1),
    adminApi.moderationQueue(),
    operationsApi.summary(),
    adminApi.audit(6),
  ])

  if (statsRes.status === 'fulfilled') {
    stats.value = statsRes.value
    openReports.value = statsRes.value.reports_open
  } else {
    error.value = errorText(statsRes.reason)
  }
  if (contentRes.status === 'fulfilled') openContentReports.value = contentRes.value.total
  if (modRes.status === 'fulfilled') openModeration.value = modRes.value.items.length
  if (opsRes.status === 'fulfilled') ops.value = opsRes.value
  if (auditRes.status === 'fulfilled') audits.value = auditRes.value
  loading.value = false
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar is-spread">
    <span class="muted">
      <template v-if="ops">运营快照更新于 {{ fmtTime(ops.as_of) }}</template>
      <template v-else>展示社区实时统计与跨模块待办</template>
    </span>
    <div class="adm-actions">
      <button class="btn btn-sm" type="button" :disabled="loading" @click="load">
        <AdminIcon name="refresh" :size="13" /> {{ loading ? '刷新中…' : '刷新数据' }}
      </button>
      <RouterLink class="btn btn-sm btn-primary" to="/admin/reports">
        进入审核中心（{{ totalReviewTodo }}）
      </RouterLink>
    </div>
  </div>

  <p v-if="error && !stats" class="error-text" role="alert">{{ error }}</p>
  <p v-else-if="loading && !stats" class="muted">正在加载工作台数据…</p>

  <template v-if="stats">
    <section class="adm-kpi-grid">
      <RouterLink class="panel adm-kpi" to="/admin/metrics">
        <div class="adm-kpi-top">
          <span>今日活跃用户 (DAU)</span>
          <span class="adm-tag valid">近 7 天 +{{ stats.users_new_7d }}</span>
        </div>
        <div class="adm-kpi-val">{{ stats.dau.toLocaleString() }}</div>
        <div class="adm-kpi-sub">用户总数 {{ stats.users_total.toLocaleString() }} · 24h 新增 +{{ stats.users_new_24h }}</div>
      </RouterLink>

      <RouterLink class="panel adm-kpi" to="/admin/topics">
        <div class="adm-kpi-top">
          <span>24 小时新增内容</span>
          <span class="adm-tag info">社区互动</span>
        </div>
        <div class="adm-kpi-val">{{ (stats.topics_24h + stats.posts_24h).toLocaleString() }}</div>
        <div class="adm-kpi-sub">
          新帖 {{ stats.topics_24h }}（共 {{ stats.topics_total.toLocaleString() }}）· 新回复 {{ stats.posts_24h }}（共 {{ stats.posts_total.toLocaleString() }}）
        </div>
      </RouterLink>

      <RouterLink class="panel adm-kpi" to="/admin/reports">
        <div class="adm-kpi-top">
          <span>待处理治理与审核</span>
          <span class="adm-tag" :class="totalReviewTodo > 0 ? 'danger' : 'valid'">
            {{ totalReviewTodo > 0 ? '需优先处理' : '全部清空' }}
          </span>
        </div>
        <div class="adm-kpi-val" :class="{ 'is-alert': totalReviewTodo > 0 }">{{ totalReviewTodo }}</div>
        <div class="adm-kpi-sub">
          帖子举报 {{ openReports }} · 作品举报 {{ openContentReports }} · 待审 {{ openModeration }}
        </div>
      </RouterLink>

      <RouterLink class="panel adm-kpi" to="/admin/discover">
        <div class="adm-kpi-top">
          <span>待发布发现页草稿</span>
          <span class="adm-tag" :class="scheduledDrafts > 0 ? 'info' : ''">
            {{ scheduledDrafts > 0 ? `含 ${scheduledDrafts} 条定时` : '运营编排' }}
          </span>
        </div>
        <div class="adm-kpi-val">{{ pendingDrafts }}</div>
        <div class="adm-kpi-sub">
          外链待核对 {{ attentionLinks }} · 待更新来源 {{ staleSources }}
        </div>
      </RouterLink>
    </section>

    <div class="adm-dash-grid">
      <section class="panel adm-card">
        <div class="adm-card-head">
          <h2>
            待办行动中心
            <span class="adm-tag" :class="totalActionItems > 0 ? 'danger' : 'valid'">
              {{ totalActionItems > 0 ? `${totalActionItems} 项待关注` : '暂无积压' }}
            </span>
          </h2>
          <RouterLink class="btn btn-ghost btn-sm" to="/admin/reports">前往审核中心 →</RouterLink>
        </div>

        <div>
          <RouterLink class="adm-todo-row" to="/admin/reports">
            <div class="adm-todo-left">
              <span class="adm-todo-ic" :class="openReports > 0 ? 'danger' : 'ok'">
                <AdminIcon name="shield" :size="16" />
              </span>
              <div>
                <div class="adm-title">帖子与回复举报（{{ openReports }} 条）</div>
                <div class="adm-sub">按时间从早到晚排队，支持处理并隐藏或驳回</div>
              </div>
            </div>
            <span class="btn btn-sm" :class="openReports > 0 ? 'btn-soft' : ''">{{ openReports > 0 ? '立即处理' : '查看' }}</span>
          </RouterLink>

          <RouterLink class="adm-todo-row" to="/admin/content-reports">
            <div class="adm-todo-left">
              <span class="adm-todo-ic" :class="openContentReports > 0 ? 'danger' : 'ok'">
                <AdminIcon name="cube" :size="16" />
              </span>
              <div>
                <div class="adm-title">作品与方案举报（{{ openContentReports }} 条）</div>
                <div class="adm-sub">核实违规内容包或桌面方案，支持一键下架并关闭关联举报</div>
              </div>
            </div>
            <span class="btn btn-sm">{{ openContentReports > 0 ? '去核对' : '查看' }}</span>
          </RouterLink>

          <RouterLink class="adm-todo-row" to="/admin/moderation">
            <div class="adm-todo-left">
              <span class="adm-todo-ic" :class="openModeration > 0 ? 'warn' : 'ok'">
                <AdminIcon name="check" :size="16" />
              </span>
              <div>
                <div class="adm-title">待审内容队列（{{ openModeration }} 条）</div>
                <div class="adm-sub">用户新发布内容包（待升 T1）及命中复核词的帖子与回复</div>
              </div>
            </div>
            <span class="btn btn-sm">{{ openModeration > 0 ? '进入队列' : '查看' }}</span>
          </RouterLink>

          <RouterLink class="adm-todo-row" to="/admin/discover">
            <div class="adm-todo-left">
              <span class="adm-todo-ic info">
                <AdminIcon name="compass" :size="16" />
              </span>
              <div>
                <div class="adm-title">发现页待审草稿（{{ pendingDrafts }} 条）</div>
                <div class="adm-sub">小伴前沿、AI 日报、导航与榜单等栏目草稿预览与定时发布</div>
              </div>
            </div>
            <span class="btn btn-sm">去编排</span>
          </RouterLink>

          <RouterLink class="adm-todo-row" to="/admin/community-operations">
            <div class="adm-todo-left">
              <span class="adm-todo-ic" :class="attentionLinks > 0 || staleSources > 0 ? 'warn' : 'ok'">
                <AdminIcon name="warn" :size="16" />
              </span>
              <div>
                <div class="adm-title">外链巡检与来源新鲜度（{{ attentionLinks }} 条待核对链接）</div>
                <div class="adm-sub">
                  {{ staleSources > 0 ? `有 ${staleSources} 个启用来源待更新，请及时核对` : '已启用来源更新状态正常' }}
                </div>
              </div>
            </div>
            <span class="btn btn-sm">巡检详情</span>
          </RouterLink>
        </div>
      </section>

      <div style="display: grid; gap: 16px; align-content: start">
        <section class="panel adm-card">
          <div class="adm-card-head">
            <h2>自动采集源健康</h2>
            <RouterLink class="btn btn-ghost btn-sm" to="/admin/collection">管理来源 →</RouterLink>
          </div>
          <div v-if="ops?.sources.length">
            <div v-for="s in ops.sources" :key="s.id" class="adm-health-row">
              <div style="min-width: 0">
                <div class="adm-title" style="display: flex; align-items: center; gap: 7px">
                  <span class="adm-dot" :class="sourceDot(s.freshness, !!s.last_error)" />
                  <span>{{ s.title }}</span>
                </div>
                <div class="adm-sub">
                  <span>最近成功 {{ fmtTime(s.last_success_at) }}</span>
                  <span v-if="s.pending > 0">待审 {{ s.pending }}</span>
                </div>
              </div>
              <span class="adm-tag" :class="sourceTone(s.freshness, !!s.last_error)">
                {{ s.last_error ? '有异常' : FRESH_LABEL[s.freshness] ?? s.freshness }}
              </span>
            </div>
          </div>
          <p v-else class="adm-empty" style="padding: 24px">暂无采集来源数据</p>
        </section>

        <section class="panel adm-card">
          <div class="adm-card-head">
            <h2>最近操作动态</h2>
            <RouterLink class="btn btn-ghost btn-sm" to="/admin/audit">全部日志 →</RouterLink>
          </div>
          <div v-if="audits.length">
            <div v-for="a in audits" :key="a.id" class="adm-health-row" style="align-items: flex-start">
              <div style="min-width: 0">
                <div style="font-size: 13px">
                  <strong>{{ a.admin }}</strong> · {{ AUDIT_ACTIONS[a.action] ?? a.action }}
                </div>
                <div class="adm-sub">
                  <span>{{ AUDIT_TARGETS[a.target_type] ?? a.target_type }} {{ a.target_id }}</span>
                  <span v-if="a.detail">{{ a.detail }}</span>
                </div>
              </div>
              <span class="adm-mono shrink">{{ fmtTime(a.created_at).slice(5) }}</span>
            </div>
          </div>
          <p v-else class="adm-empty" style="padding: 24px">暂无操作记录</p>
        </section>
      </div>
    </div>
  </template>
</template>
