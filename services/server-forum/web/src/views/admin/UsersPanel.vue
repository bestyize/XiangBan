<script setup lang="ts">
// 用户管理：清爽数据表 + 快捷禁言时长弹层 + 封禁确认弹层（移除每行常驻 148px 输入框与原生 confirm/alert）。
import { onMounted, ref } from 'vue'
import { adminApi, type AdminUser, type UserStatus } from '@/api/admin'
import AdminIcon from './AdminIcon.vue'
import AdminModal from './AdminModal.vue'
import AdminPager from './AdminPager.vue'
import { fmtDate, fmtTime, fromNow, notify, perform, TRUST_LEVELS, USER_STATUS, userTone, usePagedList } from './shared'

const q = ref('')
const status = ref('')
const muteTarget = ref<AdminUser | null>(null)
const muteHoursInput = ref('24')
const muteError = ref('')
const banTarget = ref<AdminUser | null>(null)
const busy = ref(false)

const MUTE_PRESETS = [
  { hours: '24', label: '24 小时' },
  { hours: '72', label: '3 天' },
  { hours: '168', label: '7 天' },
  { hours: '720', label: '30 天' },
  { hours: '', label: '无限期' },
]

const list = usePagedList<AdminUser>(
  (page, size) => adminApi.users({ q: q.value.trim(), status: status.value, page, page_size: size }),
  30,
)

function openMute(u: AdminUser) {
  muteTarget.value = u
  muteHoursInput.value = '24'
  muteError.value = ''
}

async function setStatus(u: AdminUser, next: UserStatus, okText: string, muteHours?: number) {
  busy.value = true
  const out: { r?: Awaited<ReturnType<typeof adminApi.setUserStatus>> } = {}
  const ok = await perform(async () => {
    out.r = await adminApi.setUserStatus(u.uid, next, muteHours)
  }, okText)
  busy.value = false
  if (ok && out.r) {
    u.status = out.r.status
    u.muted_until = out.r.muted_until
  }
  return ok
}

async function submitMute() {
  const u = muteTarget.value
  if (!u) return
  muteError.value = ''
  const raw = muteHoursInput.value.trim()
  let hours: number | undefined
  if (raw) {
    hours = Number(raw)
    if (!Number.isInteger(hours) || hours < 1) {
      muteError.value = '禁言时长须为不小于 1 的整数小时，留空表示无限期'
      return
    }
  }
  const ok = await setStatus(u, 'muted', `已禁言 ${u.nickname}`, hours)
  if (ok) muteTarget.value = null
}

async function submitBan() {
  const u = banTarget.value
  if (!u) return
  const ok = await setStatus(u, 'banned', `已封禁 ${u.nickname}`)
  if (ok) banTarget.value = null
}

async function setTrust(u: AdminUser, ev: Event) {
  const el = ev.target as HTMLSelectElement
  const level = Number(el.value)
  const ok = await perform(
    () => adminApi.setTrust(u.uid, level),
    `已将 ${u.nickname} 设为${TRUST_LEVELS[level]?.name ?? level}`,
  )
  if (ok) u.trust_level = level
  else el.value = String(u.trust_level)
}

async function copyUid(uid: string) {
  try {
    await navigator.clipboard.writeText(uid)
    notify('已复制 UID')
  } catch {
    notify('复制失败，请手动选中', 'error')
  }
}

onMounted(list.load)
</script>

<template>
  <div class="adm-toolbar is-spread">
    <div class="adm-toolbar" style="margin: 0">
      <form @submit.prevent="list.search()">
        <div class="adm-search-box">
          <AdminIcon name="search" :size="14" />
          <input v-model="q" class="input" placeholder="搜索昵称或完整 UID" aria-label="搜索用户" />
        </div>
        <button class="btn" type="submit">搜索</button>
      </form>
      <div class="adm-seg" role="group" aria-label="用户状态">
        <button
          v-for="[val, lbl] in [['', '全部'], ['active', '正常'], ['muted', '禁言中'], ['banned', '已封禁']]"
          :key="val"
          type="button"
          :class="{ 'is-on': status === val }"
          @click="status = val; list.search()"
        >
          {{ lbl }}
        </button>
      </div>
    </div>
    <span class="muted">共 {{ list.total.value }} 位成员</span>
  </div>

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">正在读取用户列表…</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有符合条件的用户</p>
    <div v-else class="adm-table-scroll">
      <table class="adm-table">
        <thead>
          <tr>
            <th>用户</th>
            <th>信任等级</th>
            <th>账号状态</th>
            <th>社区数据</th>
            <th class="num" style="width: 190px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="u in list.items.value" :key="u.uid">
            <td class="full">
              <div style="display: flex; align-items: center; gap: 10px">
                <span class="adm-avatar is-sm">{{ (u.nickname || '?').slice(0, 1) }}</span>
                <div style="min-width: 0">
                  <div class="adm-title">
                    <a :href="`/u/${u.uid}`" target="_blank" rel="noopener">{{ u.nickname }}</a>
                  </div>
                  <div class="adm-sub">
                    <button
                      type="button"
                      class="adm-mono"
                      style="border: 0; background: none; padding: 0; display: inline-flex; align-items: center; gap: 4px"
                      title="点击复制完整 UID"
                      @click="copyUid(u.uid)"
                    >
                      {{ u.uid }} <AdminIcon name="copy" :size="11" />
                    </button>
                    <span>注册 {{ fmtDate(u.created_at) }}</span>
                    <span>活跃 {{ fromNow(u.last_seen_at) }}</span>
                  </div>
                </div>
              </div>
            </td>
            <td data-label="信任等级">
              <select class="select is-sm" style="min-width: 104px" aria-label="信任等级" :value="u.trust_level" @change="setTrust(u, $event)">
                <option v-for="t in TRUST_LEVELS" :key="t.level" :value="t.level">{{ t.level }} {{ t.name }}</option>
              </select>
            </td>
            <td data-label="账号状态" class="shrink">
              <span class="adm-tag" :class="userTone(u.status)">
                <span class="adm-dot" :class="u.status === 'active' ? 'ok' : u.status === 'muted' ? 'warn' : 'danger'" />
                {{ USER_STATUS[u.status] }}
              </span>
              <div v-if="u.status === 'muted'" class="adm-sub">
                <span>{{ u.muted_until ? `至 ${fmtTime(u.muted_until)}` : '无限期' }}</span>
              </div>
            </td>
            <td data-label="社区数据" class="shrink">
              <div class="adm-sub" style="margin: 0">
                <span>帖 {{ u.topic_count }}</span>
                <span>回 {{ u.post_count }}</span>
                <span><strong style="color: var(--adm-ink)">{{ u.points }}</strong> 分</span>
              </div>
            </td>
            <td class="num shrink">
              <div class="adm-actions">
                <button v-if="u.status !== 'banned'" class="btn btn-sm" type="button" @click="openMute(u)">禁言</button>
                <button
                  v-if="u.status !== 'active'"
                  class="btn btn-sm btn-soft"
                  type="button"
                  :disabled="busy"
                  @click="setStatus(u, 'active', `已恢复 ${u.nickname}`)"
                >
                  恢复
                </button>
                <button v-if="u.status !== 'banned'" class="btn btn-sm btn-danger" type="button" @click="banTarget = u">
                  封禁
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value" @go="list.goto" />
  </div>

  <AdminModal :open="!!muteTarget" :title="muteTarget ? `禁言用户 · ${muteTarget.nickname}` : ''" @close="muteTarget = null">
    <p style="margin: 0 0 12px; color: var(--adm-ink-2); font-size: 13px">
      选择禁言时长，期间该用户将无法发帖、评论或发布作品，可随时在后台恢复正常。
    </p>
    <div class="adm-chips" style="margin-bottom: 14px">
      <button v-for="p in MUTE_PRESETS" :key="p.label" type="button" class="adm-chip-btn" @click="muteHoursInput = p.hours; muteError = ''">
        {{ p.label }}
      </button>
    </div>
    <label class="field">
      <span>禁言时长（小时，留空表示无限期）</span>
      <input v-model="muteHoursInput" class="input" inputmode="numeric" placeholder="例如 24，留空为无限期" @input="muteError = ''" />
    </label>
    <p v-if="muteError" class="error-text" role="alert" style="margin: 8px 0 0">{{ muteError }}</p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="muteTarget = null">取消</button>
      <button class="btn btn-primary" type="button" :disabled="busy" @click="submitMute">确认禁言</button>
    </template>
  </AdminModal>

  <AdminModal :open="!!banTarget" :title="banTarget ? `确认封禁「${banTarget.nickname}」？` : ''" @close="banTarget = null">
    <p style="margin: 0; color: var(--adm-ink-2); font-size: 13px">
      封禁后该用户将被限制使用论坛，其关联的首页精选作品也会退出公开展示；可随时点「恢复」解除。
    </p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="banTarget = null">取消</button>
      <button class="btn btn-danger-fill" type="button" :disabled="busy" @click="submitBan">确认封禁</button>
    </template>
  </AdminModal>
</template>
