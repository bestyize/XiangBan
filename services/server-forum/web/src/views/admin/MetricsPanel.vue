<script setup lang="ts">
// 数据：匿名埋点的汇总（每日活跃设备、事件次数与独立设备数、次日留存）。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type Metrics } from '@/api/admin'

const NAMES: Record<string, string> = {
  view_home: '看首页', view_weekly: '看周刊列表', view_weekly_issue: '看周刊', view_project: '看项目', view_pack: '看内容包',
  view_discuss: '看讨论', search: '搜索', apply_click: '点“在小伴中打开”', download_click: '点下载', share_copy_link: '复制链接',
  share_poster: '生成海报', publish_start: '打开发布页', publish_done: '发布完成',
  desktop_community_open: '桌面打开社区页', desktop_apply_open: '桌面打开应用确认', desktop_apply_confirm: '桌面确认应用',
  desktop_apply_success: '桌面应用成功', desktop_apply_cancel: '桌面取消应用', desktop_scheme_replicate: '桌面复刻方案',
}
const days = ref(14)
const data = ref<Metrics | null>(null)

async function load() {
  data.value = await adminApi.metrics(days.value)
}

/** 汇总期内每个事件的总次数和各天独立设备数之和（近似“人次”）。 */
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

onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <select v-model.number="days" class="select" aria-label="统计天数" @change="load">
      <option :value="7">最近 7 天</option>
      <option :value="14">最近 14 天</option>
      <option :value="30">最近 30 天</option>
    </select>
    <button class="btn" @click="load">刷新</button>
  </div>

  <p class="adm-sub">匿名设备标识加盐哈希后存储，只记每天每个设备做过某类操作几次。“设备·天”是各天独立设备数相加。</p>

  <div v-if="data" class="panel adm-card">
    <h2 class="adm-section-title">每日活跃设备与次日留存</h2>
    <table class="adm-table">
      <thead><tr><th>日期</th><th>活跃设备</th><th>次日回来</th><th>次日留存</th></tr></thead>
      <tbody>
        <tr v-for="d in data.daily_devices" :key="d.day">
          <td data-label="日期">{{ d.day }}</td>
          <td data-label="活跃设备">{{ d.devices }}</td>
          <template v-for="r in data.retention.filter(x => x.day === d.day)" :key="r.day">
            <td data-label="次日回来">{{ r.returned }}</td>
            <td data-label="次日留存">{{ r.devices ? Math.round((r.returned / r.devices) * 100) : 0 }}%</td>
          </template>
          <template v-if="!data.retention.some(x => x.day === d.day)"><td data-label="次日回来">-</td><td data-label="次日留存">-</td></template>
        </tr>
      </tbody>
    </table>
  </div>

  <div v-if="data" class="panel adm-card">
    <h2 class="adm-section-title">事件</h2>
    <p v-if="!totals.length" class="adm-empty">还没有数据</p>
    <table v-else class="adm-table">
      <thead><tr><th>事件</th><th>类型/来源</th><th>次数</th><th>设备·天</th></tr></thead>
      <tbody>
        <tr v-for="t in totals" :key="t.name + t.dim">
          <td data-label="事件">{{ NAMES[t.name] ?? t.name }}</td>
          <td data-label="类型/来源">{{ t.dim === '-' ? '' : t.dim }}</td>
          <td data-label="次数">{{ t.count }}</td>
          <td data-label="设备·天">{{ t.devices }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
