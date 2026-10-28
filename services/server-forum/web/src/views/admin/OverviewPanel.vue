<script setup lang="ts">
// 概览：一组朴素的数字，不画图。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type AdminStats } from '@/api/admin'
import { errorText, openReports } from './shared'

const stats = ref<AdminStats | null>(null)
const error = ref('')

interface StatItem {
  label: string
  value: number
  alert?: boolean
}

const groups = computed<{ title: string; items: StatItem[] }[]>(() => {
  const s = stats.value
  if (!s) return []
  return [
    {
      title: '社区',
      items: [
        { label: '今日活跃用户', value: s.dau },
        { label: '用户总数', value: s.users_total },
        { label: '24 小时新增用户', value: s.users_new_24h },
        { label: '7 天新增用户', value: s.users_new_7d },
      ],
    },
    {
      title: '内容',
      items: [
        { label: '帖子总数', value: s.topics_total },
        { label: '24 小时新帖', value: s.topics_24h },
        { label: '回复总数', value: s.posts_total },
        { label: '24 小时新回复', value: s.posts_24h },
      ],
    },
    {
      title: '待办',
      items: [{ label: '待处理举报', value: s.reports_open, alert: s.reports_open > 0 }],
    },
  ]
})

onMounted(async () => {
  try {
    stats.value = await adminApi.stats()
    openReports.value = stats.value.reports_open
  } catch (e) {
    error.value = errorText(e)
  }
})
</script>

<template>
  <p v-if="error" class="error-text">{{ error }}</p>
  <p v-else-if="!stats" class="muted">加载中</p>
  <template v-else>
    <section v-for="(g, i) in groups" :key="g.title">
      <h2 class="adm-section-title" :style="i === 0 ? 'margin-top: 0' : ''">{{ g.title }}</h2>
      <div class="panel adm-card">
        <div class="adm-stats">
          <div v-for="it in g.items" :key="it.label" class="adm-stat" :class="{ 'is-alert': it.alert }">
            <b>{{ it.value }}</b>
            <span>{{ it.label }}</span>
          </div>
        </div>
      </div>
    </section>
  </template>
</template>
