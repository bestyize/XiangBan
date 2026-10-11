<script setup lang="ts">
// 用户管理：搜索昵称或完整 uid，调整信任等级，禁言、封禁与恢复。
import { onMounted, reactive, ref } from 'vue'
import { adminApi, type AdminUser, type UserStatus } from '@/api/admin'
import AdminPager from './AdminPager.vue'
import { fmtDate, fmtTime, fromNow, perform, TRUST_LEVELS, USER_STATUS, userTone, usePagedList } from './shared'

const q = ref('')
const status = ref('')

const list = usePagedList<AdminUser>(
  (page, size) => adminApi.users({ q: q.value.trim(), status: status.value, page, page_size: size }),
  30,
)

/** 每个用户各自的禁言时长输入（小时），留空表示无限期。 */
const hours = reactive<Record<string, string>>({})

async function setStatus(u: AdminUser, next: UserStatus, okText: string, confirmText?: string) {
  if (confirmText && !window.confirm(confirmText)) return
  let muteHours: number | undefined
  if (next === 'muted') {
    const raw = (hours[u.uid] ?? '').trim()
    if (raw) {
      muteHours = Number(raw)
      if (!Number.isInteger(muteHours) || muteHours < 1) {
        window.alert('禁言时长需要是不小于 1 的整数小时，留空表示无限期。')
        return
      }
    }
  }
  const out: { r?: Awaited<ReturnType<typeof adminApi.setUserStatus>> } = {}
  const ok = await perform(async () => {
    out.r = await adminApi.setUserStatus(u.uid, next, muteHours)
  }, okText)
  if (ok && out.r) {
    u.status = out.r.status
    u.muted_until = out.r.muted_until
    delete hours[u.uid]
  }
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

onMounted(list.load)
</script>

<template>
  <div class="adm-toolbar">
    <form @submit.prevent="list.search()">
      <input v-model="q" class="input" placeholder="搜索昵称或完整 uid" aria-label="搜索用户" />
      <button class="btn" type="submit">搜索</button>
    </form>
    <select v-model="status" class="select" aria-label="状态" @change="list.search()">
      <option value="">全部状态</option>
      <option value="active">正常</option>
      <option value="muted">禁言中</option>
      <option value="banned">已封禁</option>
    </select>
  </div>

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">加载中</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有符合条件的用户</p>
    <table v-else class="adm-table">
      <thead>
        <tr>
          <th>用户</th>
          <th>信任等级</th>
          <th>状态</th>
          <th>数据</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in list.items.value" :key="u.uid">
          <td class="full">
            <div class="adm-title">
              <a :href="`/u/${u.uid}`" target="_blank" rel="noopener">{{ u.nickname }}</a>
            </div>
            <div class="adm-mono">{{ u.uid }}</div>
            <div class="adm-sub">
              <span>注册 {{ fmtDate(u.created_at) }}</span>
              <span>最近活跃 {{ fromNow(u.last_seen_at) }}</span>
            </div>
          </td>
          <td data-label="信任等级">
            <select class="select is-sm" style="min-width: 104px" aria-label="信任等级" :value="u.trust_level" @change="setTrust(u, $event)">
              <option v-for="t in TRUST_LEVELS" :key="t.level" :value="t.level">{{ t.level }} {{ t.name }}</option>
            </select>
          </td>
          <td data-label="状态">
            <span class="adm-tag" :class="userTone(u.status)">{{ USER_STATUS[u.status] }}</span>
            <div v-if="u.status === 'muted'" class="adm-sub">
              <span>{{ u.muted_until ? `至 ${fmtTime(u.muted_until)}` : '无限期' }}</span>
            </div>
          </td>
          <td data-label="数据">
            <div class="adm-sub" style="margin: 0">
              <span>帖子 {{ u.topic_count }}</span>
              <span>回复 {{ u.post_count }}</span>
              <span>积分 {{ u.points }}</span>
            </div>
          </td>
          <td class="full">
            <div class="adm-actions">
              <template v-if="u.status !== 'banned'">
                <input
                  v-model="hours[u.uid]"
                  class="input is-sm"
                  style="width: 148px"
                  inputmode="numeric"
                  placeholder="小时，空为无限期"
                  aria-label="禁言时长（小时）"
                />
                <button class="btn btn-sm" @click="setStatus(u, 'muted', `已禁言 ${u.nickname}`)">禁言</button>
              </template>
              <button v-if="u.status !== 'active'" class="btn btn-sm" @click="setStatus(u, 'active', `已恢复 ${u.nickname}`)">
                恢复
              </button>
              <button
                v-if="u.status !== 'banned'"
                class="btn btn-sm btn-danger"
                @click="setStatus(u, 'banned', `已封禁 ${u.nickname}`, `确定封禁「${u.nickname}」？封禁后该用户将被限制使用论坛，可随时点「恢复」解除。`)"
              >
                封禁
              </button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value" @go="list.goto" />
</template>
