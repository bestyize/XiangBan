<script setup lang="ts">
// 待审队列：主从双栏检视 + 快捷理由芯片，彻底淘汰原生 window.prompt。
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { adminApi, type ModerationRow } from '@/api/admin'
import { kindLabel } from '@/lib/content'
import AdminIcon from './AdminIcon.vue'
import { fmtTime, openModeration, perform, refreshReviewBadges } from './shared'

const TYPE: Record<string, string> = { pack: '内容包', topic: '帖子', post: '回复', scheme: '方案' }
const QUICK_NOTES = [
  '清单与内容校验正常，审核通过',
  '命中复核词为正常讨论，确认放行',
  '缺少必要说明或存在合规风险，拒绝上架',
]

const rows = ref<ModerationRow[]>([])
const loading = ref(true)
const selectedId = ref<number | null>(null)
const notes = reactive<Record<number, string>>({})
const busy = ref(false)
const noteError = ref('')

const activeRow = computed<ModerationRow | null>(() => {
  if (!rows.value.length) return null
  return rows.value.find(r => r.id === selectedId.value) ?? rows.value[0] ?? null
})

watch(rows, list => {
  if (!list.length) selectedId.value = null
  else if (!list.some(r => r.id === selectedId.value)) selectedId.value = list[0]!.id
})

async function load() {
  loading.value = true
  try {
    rows.value = (await adminApi.moderationQueue()).items
    openModeration.value = rows.value.length
  } finally {
    loading.value = false
  }
}

async function resolve(r: ModerationRow, action: 'approve' | 'reject') {
  noteError.value = ''
  const note = (notes[r.id] ?? '').trim()
  if (action === 'reject' && !note) {
    noteError.value = '拒绝时请填写原因（会记录在操作日志中）'
    return
  }
  busy.value = true
  const ok = await perform(
    () => adminApi.resolveModeration(r.id, action, note, r.version),
    action === 'approve' ? '审核已通过' : '已拒绝并记录日志',
  )
  busy.value = false
  if (ok) {
    delete notes[r.id]
    await load()
    void refreshReviewBadges()
  }
}

function link(r: ModerationRow): string {
  if (r.target_type === 'pack') return `/packs/${r.target_id}`
  if (r.target_type === 'topic') return `/t/${r.target_id}`
  if (r.target_type === 'post' && r.topic_id) return `/t/${r.topic_id}`
  if (r.target_type === 'scheme') return `/schemes/${r.target_id}`
  return ''
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar is-spread">
    <span class="muted">用户新发布内容包，以及命中「复核词」的帖子、回复与桌面方案将进入此队列。</span>
    <button class="btn btn-sm" type="button" :disabled="loading" @click="load">
      <AdminIcon name="refresh" :size="13" /> 刷新
    </button>
  </div>

  <div v-if="loading && !rows.length" class="panel adm-card">
    <p class="adm-state muted">正在读取待审队列…</p>
  </div>
  <div v-else-if="!rows.length" class="panel adm-card">
    <p class="adm-empty">当前没有待审内容</p>
  </div>
  <div v-else class="adm-split">
    <section class="panel adm-card">
      <div class="adm-card-head">
        <h3>待审队列（{{ rows.length }}）</h3>
      </div>
      <ul class="adm-queue">
        <li
          v-for="r in rows"
          :key="r.id"
          :class="{ 'is-selected': activeRow?.id === r.id }"
          @click="selectedId = r.id; noteError = ''"
        >
          <div style="display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-bottom: 5px">
            <span class="adm-tags">
              <span class="adm-tag suspect">{{ TYPE[r.target_type] ?? r.target_type }}</span>
              <span v-if="r.kind" class="adm-tag line">{{ kindLabel(r.kind) }}</span>
            </span>
            <span class="adm-mono">{{ fmtTime(r.created_at).slice(5) }}</span>
          </div>
          <div class="adm-title" style="font-size: 13px">{{ r.title ?? `#${r.target_id}` }}</div>
          <div class="adm-sub">
            <span>作者 {{ r.author ?? '-' }}</span>
            <span>{{ r.note || '新发布待审核' }}</span>
          </div>
        </li>
      </ul>
    </section>

    <section v-if="activeRow" class="panel adm-card">
      <div class="adm-card-head">
        <div class="adm-tags">
          <span class="adm-tag suspect">{{ TYPE[activeRow.target_type] ?? activeRow.target_type }}</span>
          <span v-if="activeRow.kind" class="adm-tag line">{{ kindLabel(activeRow.kind) }}</span>
          <span v-if="activeRow.version" class="adm-mono">v{{ activeRow.version }}</span>
        </div>
        <div class="adm-actions">
          <a v-if="link(activeRow)" class="btn btn-sm" :href="link(activeRow)" target="_blank" rel="noopener">
            <AdminIcon name="external" :size="13" /> 预览前台内容
          </a>
          <RouterLink v-if="activeRow.target_type === 'topic' || activeRow.target_type === 'post'" class="btn btn-sm" to="/admin/topics">
            去帖子管理
          </RouterLink>
        </div>
      </div>

      <div class="adm-card-body">
        <h2 style="margin: 0 0 8px; font-size: 17px">{{ activeRow.title ?? `#${activeRow.target_id}` }}</h2>
        <div class="adm-sub" style="margin-bottom: 14px">
          <span>目标编号 #{{ activeRow.target_id }}</span>
          <span>作者 {{ activeRow.author ?? '-' }}</span>
          <span>提交时间 {{ fmtTime(activeRow.created_at) }}</span>
        </div>

        <div class="adm-eyebrow" style="margin-bottom: 6px">入队原因</div>
        <p class="adm-quote" style="margin-top: 0">{{ activeRow.note || '用户新发布内容，等待人工审核确认。' }}</p>

        <div style="border-top: 1px solid var(--adm-line); padding-top: 16px; margin-top: 16px">
          <div class="adm-eyebrow" style="margin-bottom: 8px">快捷审核备注</div>
          <div class="adm-chips">
            <button v-for="txt in QUICK_NOTES" :key="txt" type="button" class="adm-chip-btn" @click="notes[activeRow.id] = txt; noteError = ''">
              {{ txt }}
            </button>
          </div>
          <label class="field" style="margin-bottom: 12px">
            <span>审核备注 / 拒绝原因（拒绝时必填，记入审计日志）</span>
            <textarea
              v-model="notes[activeRow.id]"
              class="textarea"
              rows="2"
              placeholder="输入通过备注或拒绝原因…"
              @input="noteError = ''"
            />
          </label>
          <p v-if="noteError" class="error-text" role="alert" style="margin: 0 0 10px">{{ noteError }}</p>
          <div class="adm-actions">
            <button class="btn btn-primary" type="button" :disabled="busy" @click="resolve(activeRow, 'approve')">
              <AdminIcon name="check" :size="14" />
              {{ activeRow.target_type === 'topic' || activeRow.target_type === 'post' ? '标记已处理' : '通过审核' }}
            </button>
            <button
              v-if="activeRow.target_type === 'pack' || activeRow.target_type === 'scheme'"
              class="btn btn-danger"
              type="button"
              :disabled="busy"
              @click="resolve(activeRow, 'reject')"
            >
              拒绝上架
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
