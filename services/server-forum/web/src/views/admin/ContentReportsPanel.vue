<script setup lang="ts">
// 作品与方案举报：主从双栏检视 + 快捷内部处理说明 + 居中下架确认弹层。
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { AdminApiError } from '@/api/admin'
import { contentReportsApi, type ContentReport } from '@/api/contentSocial'
import AdminIcon from './AdminIcon.vue'
import AdminModal from './AdminModal.vue'
import AdminPager from './AdminPager.vue'
import {
  fmtTime,
  notify,
  openContentReports,
  refreshReviewBadges,
  REPORT_REASONS,
  REPORT_STATUS,
  usePagedList,
} from './shared'

const status = ref('open')
const notes = reactive<Record<number, string>>({})
const busy = ref(0)
const handlingError = ref('')
const selectedId = ref<number | null>(null)
const removeConfirm = ref<ContentReport | null>(null)

const tabs = [
  { key: 'open', label: '待处理' },
  { key: 'resolved', label: '已下架处理' },
  { key: 'dismissed', label: '已驳回' },
  { key: 'all', label: '全部' },
]

const QUICK_NOTES = [
  '核实存在侵权或违规内容，已下架该作品并关闭关联举报',
  '核实内容符合社区规范，驳回本次举报并保留公开状态',
]

const list = usePagedList<ContentReport>(page => contentReportsApi.list(status.value, page), 20)

const activeRow = computed<ContentReport | null>(() => {
  const rows = list.items.value
  if (!rows.length) return null
  return rows.find(r => r.id === selectedId.value) ?? rows[0] ?? null
})

watch(
  () => list.items.value,
  rows => {
    if (!rows.length) selectedId.value = null
    else if (!rows.some(r => r.id === selectedId.value)) selectedId.value = rows[0]!.id
  },
)

function pick(value: string) {
  if (busy.value) return
  status.value = value
  handlingError.value = ''
  void list.search()
}

function requestRemove(row: ContentReport) {
  if (!notes[row.id]?.trim() || busy.value) return
  removeConfirm.value = row
}

async function handle(row: ContentReport, action: 'remove' | 'dismiss') {
  const note = notes[row.id]?.trim()
  if (!note || busy.value) return
  removeConfirm.value = null
  busy.value = row.id
  handlingError.value = ''
  try {
    const result = await contentReportsApi.handle(row, action, note)
    delete notes[row.id]
    notify(action === 'dismiss' ? '已驳回举报，内容状态保留' : `已下架处理，共处理 ${result.reports_handled} 条举报`)
    await list.load()
    void refreshReviewBadges()
  } catch (e) {
    handlingError.value = e instanceof Error ? e.message : '处理失败，请重试'
    if (e instanceof AdminApiError && e.status === 409) await list.load()
  } finally {
    busy.value = 0
  }
}

onMounted(async () => {
  await list.load()
  if (status.value === 'open') openContentReports.value = list.total.value
})
</script>

<template>
  <div class="adm-toolbar is-spread">
    <div class="adm-seg" role="group" aria-label="内容举报状态">
      <button v-for="tab in tabs" :key="tab.key" type="button" :disabled="!!busy" :class="{ 'is-on': status === tab.key }" @click="pick(tab.key)">
        {{ tab.label }}
      </button>
    </div>
    <button class="btn btn-sm" type="button" :disabled="list.loading.value || !!busy" @click="list.load()">
      <AdminIcon name="refresh" :size="13" /> 刷新
    </button>
  </div>

  <p class="adm-sub" style="margin: 0">
    举报理由与处理说明仅供管理员内部留档。下架将同步隐藏收藏、公开详情与引用卡片。
  </p>
  <p v-if="handlingError" class="error-text" role="alert">{{ handlingError }}</p>

  <div v-if="list.error.value" class="panel adm-card">
    <p class="adm-state error-text" role="alert">
      {{ list.error.value }}
      <button class="btn btn-sm" type="button" @click="list.load()">重试</button>
    </p>
  </div>
  <div v-else-if="list.loading.value && !list.items.value.length" class="panel adm-card">
    <p class="adm-state muted">正在读取作品举报…</p>
  </div>
  <div v-else-if="!list.items.value.length" class="panel adm-card">
    <p class="adm-empty">暂无对应状态的作品与方案举报</p>
  </div>
  <div v-else class="adm-split">
    <section class="panel adm-card">
      <div class="adm-card-head">
        <h3>作品举报队列（{{ list.total.value }}）</h3>
      </div>
      <ul class="adm-queue">
        <li
          v-for="row in list.items.value"
          :key="row.id"
          :class="{ 'is-selected': activeRow?.id === row.id }"
          @click="selectedId = row.id"
        >
          <div style="display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-bottom: 5px">
            <span class="adm-tags">
              <span class="adm-tag danger">{{ REPORT_REASONS[row.reason] || row.reason }}</span>
              <span class="adm-tag line">{{ row.kind === 'pack' ? '内容包' : '方案' }} #{{ row.target_id }}</span>
            </span>
            <span class="adm-mono">{{ fmtTime(row.created_at).slice(5) }}</span>
          </div>
          <div class="adm-title" style="font-size: 13px">{{ row.target.title || '内容已不存在' }}</div>
          <div class="adm-sub">
            <span>{{ REPORT_STATUS[row.status] }}</span>
            <span>举报人 {{ row.reporter.nickname || row.reporter.uid }}</span>
          </div>
        </li>
      </ul>
      <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value || !!busy" @go="list.goto" />
    </section>

    <section v-if="activeRow" class="panel adm-card">
      <div class="adm-card-head">
        <div class="adm-tags">
          <span class="adm-tag danger">{{ REPORT_REASONS[activeRow.reason] || activeRow.reason }}</span>
          <span class="adm-tag line">{{ activeRow.kind === 'pack' ? '内容包' : '方案' }} #{{ activeRow.target_id }}</span>
          <span class="adm-tag" :class="activeRow.status === 'resolved' ? 'valid' : activeRow.status === 'open' ? 'suspect' : 'expired'">
            {{ REPORT_STATUS[activeRow.status] }}
          </span>
        </div>
        <a
          v-if="activeRow.target.exists && activeRow.target.status === 'published'"
          class="btn btn-sm"
          :href="`/${activeRow.kind === 'pack' ? 'packs' : 'schemes'}/${activeRow.target_id}`"
          target="_blank"
          rel="noopener"
        >
          <AdminIcon name="external" :size="13" /> 查看公开详情
        </a>
      </div>

      <div class="adm-card-body">
        <h2 style="margin: 0 0 8px; font-size: 17px">{{ activeRow.target.title || '内容已不存在' }}</h2>
        <div class="adm-sub" style="margin-bottom: 14px">
          <span>举报人 {{ activeRow.reporter.nickname || activeRow.reporter.uid }}</span>
          <span>提交时间 {{ fmtTime(activeRow.created_at) }}</span>
          <span v-if="!activeRow.target.exists || activeRow.target.status !== 'published'">当前已不可公开访问</span>
        </div>

        <template v-if="activeRow.detail">
          <div class="adm-eyebrow" style="margin-bottom: 6px">举报详情</div>
          <p class="adm-quote" style="margin-top: 0">{{ activeRow.detail }}</p>
        </template>

        <form v-if="activeRow.status === 'open'" style="border-top: 1px solid var(--adm-line); padding-top: 16px; margin-top: 16px" @submit.prevent="handle(activeRow, 'dismiss')">
          <div class="adm-eyebrow" style="margin-bottom: 8px">快捷处理说明</div>
          <div class="adm-chips">
            <button v-for="txt in QUICK_NOTES" :key="txt" type="button" class="adm-chip-btn" @click="notes[activeRow.id] = txt">
              {{ txt }}
            </button>
          </div>
          <label class="field" style="margin-bottom: 14px">
            <span>内部处理说明（必填，不向作者公开）</span>
            <textarea v-model="notes[activeRow.id]" class="textarea" maxlength="500" rows="3" required :disabled="!!busy" placeholder="记录核实依据与处理决定…" />
          </label>
          <div class="adm-actions">
            <button type="button" class="btn btn-danger-fill" :disabled="!!busy || !notes[activeRow.id]?.trim()" @click="requestRemove(activeRow)">
              下架并处理该内容全部举报
            </button>
            <button type="submit" class="btn" :disabled="!!busy || !notes[activeRow.id]?.trim()">
              驳回此条举报
            </button>
          </div>
        </form>

        <div v-else class="adm-callout" style="margin-top: 16px">
          处理人：<strong>{{ activeRow.handled_by || '-' }}</strong> · {{ fmtTime(activeRow.handled_at) }}
          <div v-if="activeRow.result" style="margin-top: 4px">{{ activeRow.result }}</div>
        </div>
      </div>
    </section>
  </div>

  <AdminModal :open="!!removeConfirm" title="确认下架被举报内容？" @close="removeConfirm = null">
    <p v-if="removeConfirm" style="margin: 0; color: var(--adm-ink-2)">
      确定下架「<strong>{{ removeConfirm.target.title || '该内容' }}</strong>」并一并处理此内容的全部待审举报？下架后公开详情、收藏与引用卡片将同步隐藏。
    </p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="removeConfirm = null">取消</button>
      <button v-if="removeConfirm" class="btn btn-danger-fill" type="button" :disabled="!!busy" @click="handle(removeConfirm, 'remove')">
        确认下架
      </button>
    </template>
  </AdminModal>
</template>
