<script setup lang="ts">
// 数据分析：匿名埋点汇总 + 每日活跃设备/次日留存可视化图表 + 事件排行进度条。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type Metrics } from '@/api/admin'
import AdminIcon from './AdminIcon.vue'
import { errorText } from './shared'

const NAMES: Record<string, string> = {
  view_home: '看首页',
  view_weekly: '看周刊列表',
  view_weekly_issue: '看周刊',
  view_project: '看项目',
  view_pack: '看内容包',
  view_discuss: '看讨论',
  search: '搜索',
  apply_click: '点「在小伴中打开」',
  download_click: '点下载',
  share_copy_link: '复制链接',
  share_poster: '生成海报',
  publish_start: '打开发布页',
  publish_done: '发布完成',
  desktop_community_open: '桌面打开社区页',
  desktop_apply_open: '桌面打开应用确认',
  desktop_apply_confirm: '桌面确认应用',
  desktop_apply_success: '桌面应用成功',
  desktop_apply_cancel: '桌面取消应用',
  desktop_scheme_replicate: '桌面复刻方案',
}

const days = ref<7 | 14 | 30>(14)
const data = ref<Metrics | null>(null)
const loading = ref(false)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    data.value = await adminApi.metrics(days.value)
  } catch (e) {
    error.value = errorText(e)
  } finally {
    loading.value = false
  }
}

function pickDays(next: 7 | 14 | 30) {
  days.value = next
  void load()
}

const retentionByDay = computed(() => new Map((data.value?.retention ?? []).map(r => [r.day, r])))

const series = computed(() => {
  const rows = [...(data.value?.daily_devices ?? [])].sort((a, b) => a.day.localeCompare(b.day))
  return rows.map(d => {
    const r = retentionByDay.value.get(d.day)
    const rate = r && r.devices > 0 ? Math.round((r.returned / r.devices) * 100) : null
    return { day: d.day, devices: d.devices, returned: r?.returned ?? null, rate }
  })
})

const avgRetention = computed(() => {
  const valid = series.value.filter(s => s.rate !== null)
  if (!valid.length) return null
  return Math.round(valid.reduce((acc, s) => acc + (s.rate ?? 0), 0) / valid.length)
})

const maxDevices = computed(() => Math.max(1, ...series.value.map(s => s.devices)))

const retentionPolyline = computed(() => {
  const list = series.value
  if (!list.length) return ''
  const step = 520 / Math.max(list.length, 1)
  return list
    .map((item, idx) => {
      if (item.rate === null) return null
      const x = 30 + idx * step + step / 2
      const y = 148 - Math.round((item.rate / 100) * 124)
      return `${x.toFixed(1)},${y}`
    })
    .filter((pt): pt is string => !!pt)
    .join(' ')
})

const totals = computed(() => {
  const acc = new Map<string, { name: string; dim: string; count: number; devices: number }>()
  for (const e of data.value?.events ?? []) {
    const key = `${e.name}|${e.dim}`
    const row = acc.get(key) ?? { name: e.name, dim: e.dim, count: 0, devices: 0 }
    row.count += e.count
    row.devices += e.devices
    acc.set(key, row)
  }
  return [...acc.values()].sort((a, b) => b.count - a.count)
})

const maxEventCount = computed(() => Math.max(1, totals.value[0]?.count ?? 1))

onMounted(load)
</script>

<template>
  <div class="adm-toolbar is-spread">
    <div class="adm-seg" role="group" aria-label="统计天数">
      <button v-for="d in ([7, 14, 30] as const)" :key="d" type="button" :class="{ 'is-on': days === d }" @click="pickDays(d)">
        最近 {{ d }} 天
      </button>
    </div>
    <button class="btn btn-sm" type="button" :disabled="loading" @click="load">
      <AdminIcon name="refresh" :size="13" /> {{ loading ? '刷新中…' : '刷新' }}
    </button>
  </div>

  <p class="adm-sub" style="margin: 0">
    匿名设备标识加盐哈希后存储，只记每天每个设备做过某类操作几次。「设备·天」为各天独立设备数之和。
  </p>

  <p v-if="error" class="error-text" role="alert">{{ error }}</p>
  <p v-else-if="loading && !data" class="muted">正在加载分析数据…</p>

  <template v-if="data">
    <div class="adm-dash-grid">
      <section class="panel adm-card">
        <div class="adm-card-head">
          <h2>每日活跃设备与次日留存趋势</h2>
          <span v-if="avgRetention !== null" class="adm-tag valid">平均次日留存 {{ avgRetention }}%</span>
        </div>
        <div class="adm-card-body">
          <p v-if="!series.length" class="adm-empty">该时间段暂无设备数据</p>
          <svg v-else viewBox="0 0 580 184" style="width: 100%; height: 190px; display: block">
            <g stroke="var(--adm-line)" stroke-width="1">
              <line x1="28" y1="24" x2="554" y2="24" />
              <line x1="28" y1="66" x2="554" y2="66" />
              <line x1="28" y1="108" x2="554" y2="108" />
              <line x1="28" y1="148" x2="554" y2="148" />
            </g>
            <g v-for="(item, idx) in series" :key="item.day">
              <rect
                :x="30 + idx * (520 / series.length) + 4"
                :y="148 - Math.round((item.devices / maxDevices) * 120)"
                :width="Math.max(6, 520 / series.length - 8)"
                :height="Math.max(2, Math.round((item.devices / maxDevices) * 120))"
                rx="4"
                fill="var(--brand-fill)"
                :opacity="idx === series.length - 1 ? 0.92 : 0.3"
              >
                <title>{{ item.day }} · 活跃设备 {{ item.devices }}{{ item.rate !== null ? ` · 次日留存 ${item.rate}%` : '' }}</title>
              </rect>
              <text
                v-if="series.length <= 14 || idx % 3 === 0 || idx === series.length - 1"
                :x="30 + idx * (520 / series.length) + 520 / series.length / 2"
                y="168"
                text-anchor="middle"
                font-size="10"
                fill="var(--adm-ink-3)"
              >
                {{ item.day.slice(5) }}
              </text>
            </g>
            <polyline
              v-if="retentionPolyline"
              fill="none"
              stroke="var(--adm-ok)"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
              :points="retentionPolyline"
            />
          </svg>
        </div>
      </section>

      <section class="panel adm-card">
        <div class="adm-card-head">
          <h2>核心事件触发排行</h2>
          <span class="adm-tag">{{ totals.length }} 项</span>
        </div>
        <div class="adm-card-body">
          <p v-if="!totals.length" class="adm-empty">还没有事件数据</p>
          <div v-else style="display: grid; gap: 13px">
            <div v-for="t in totals.slice(0, 8)" :key="t.name + t.dim">
              <div style="display: flex; justify-content: space-between; gap: 8px; font-size: 12.5px; margin-bottom: 5px">
                <span style="font-weight: 500">
                  {{ NAMES[t.name] ?? t.name }}
                  <span v-if="t.dim && t.dim !== '-'" class="adm-mono"> · {{ t.dim }}</span>
                </span>
                <span class="adm-sub" style="margin: 0">
                  <strong style="color: var(--adm-ink)">{{ t.count.toLocaleString() }}</strong> 次 · {{ t.devices.toLocaleString() }} 设备·天
                </span>
              </div>
              <div class="adm-bar-track">
                <div class="adm-bar-fill" :style="{ width: `${Math.max(3, Math.round((t.count / maxEventCount) * 100))}%` }" />
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>

    <section class="panel adm-card">
      <div class="adm-card-head">
        <h2>每日活跃设备与次日留存明细</h2>
      </div>
      <div class="adm-table-scroll">
        <table class="adm-table">
          <thead>
            <tr>
              <th>日期</th>
              <th class="num">活跃设备</th>
              <th class="num">次日回来</th>
              <th class="num">次日留存</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="d in data.daily_devices" :key="d.day">
              <td data-label="日期" class="adm-mono">{{ d.day }}</td>
              <td data-label="活跃设备" class="num">{{ d.devices }}</td>
              <td data-label="次日回来" class="num">{{ retentionByDay.get(d.day)?.returned ?? '-' }}</td>
              <td data-label="次日留存" class="num">
                <template v-if="retentionByDay.has(d.day)">
                  {{
                    retentionByDay.get(d.day)!.devices
                      ? `${Math.round((retentionByDay.get(d.day)!.returned / retentionByDay.get(d.day)!.devices) * 100)}%`
                      : '0%'
                  }}
                </template>
                <template v-else>-</template>
              </td>
            </tr>
            <tr v-if="!data.daily_devices.length">
              <td colspan="4" class="adm-empty">暂无明细数据</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section v-if="totals.length > 8" class="panel adm-card">
      <div class="adm-card-head">
        <h2>全部事件汇总</h2>
      </div>
      <div class="adm-table-scroll">
        <table class="adm-table">
          <thead>
            <tr>
              <th>事件</th>
              <th>类型 / 来源</th>
              <th class="num">次数</th>
              <th class="num">设备·天</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="t in totals" :key="t.name + t.dim">
              <td data-label="事件">{{ NAMES[t.name] ?? t.name }}</td>
              <td data-label="类型 / 来源" class="adm-mono">{{ t.dim === '-' ? '' : t.dim }}</td>
              <td data-label="次数" class="num">{{ t.count }}</td>
              <td data-label="设备·天" class="num">{{ t.devices }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </template>
</template>
