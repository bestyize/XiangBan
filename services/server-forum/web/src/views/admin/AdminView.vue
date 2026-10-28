<script setup lang="ts">
// 管理后台外壳：未登录显示登录页；已登录显示侧栏（窄屏为顶部标签行）和当前分区。
// 分区由路由参数 section 决定：/admin、/admin/reports、/admin/topics ……
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { adminApi, adminAuth, clearAdminToken } from '@/api/admin'
import AdminLogin from './AdminLogin.vue'
import OverviewPanel from './OverviewPanel.vue'
import ReportsPanel from './ReportsPanel.vue'
import TopicsPanel from './TopicsPanel.vue'
import UsersPanel from './UsersPanel.vue'
import EditorPanel from './EditorPanel.vue'
import EditorAccounts from './EditorAccounts.vue'
import CategoriesPanel from './CategoriesPanel.vue'
import PacksPanel from './PacksPanel.vue'
import ProjectsPanel from './ProjectsPanel.vue'
import WeeklyPanel from './WeeklyPanel.vue'
import AuditPanel from './AuditPanel.vue'
import AccountPanel from './AccountPanel.vue'
import { openReports, toasts } from './shared'
import './admin.css'

const SECTIONS = [
  { key: '', label: '概览', comp: OverviewPanel },
  { key: 'reports', label: '举报', comp: ReportsPanel },
  { key: 'topics', label: '帖子', comp: TopicsPanel },
  { key: 'users', label: '用户', comp: UsersPanel },
  { key: 'editor-accounts', label: '编辑账号', comp: EditorAccounts },
  { key: 'editor', label: '批量发帖', comp: EditorPanel },
  { key: 'categories', label: '分类', comp: CategoriesPanel },
  { key: 'packs', label: '内容包', comp: PacksPanel },
  { key: 'projects', label: '项目', comp: ProjectsPanel },
  { key: 'weekly', label: '周刊', comp: WeeklyPanel },
  { key: 'audit', label: '操作日志', comp: AuditPanel },
  { key: 'account', label: '账号', comp: AccountPanel },
] as const

const route = useRoute()

const current = computed(() => {
  const key = String(route.params.section ?? '')
  return SECTIONS.find(s => s.key === key) ?? SECTIONS[0]
})

const loggedIn = computed(() => !!adminAuth.token)

// 登录后（含刷新页面时带着旧令牌）确认身份并取待处理举报数；令牌失效时接口会自动清空登录态。
watch(
  loggedIn,
  async ok => {
    if (!ok) return
    try {
      const [me, stats] = await Promise.all([adminApi.me(), adminApi.stats()])
      adminAuth.username = me.username
      openReports.value = stats.reports_open
    } catch {
      // 401 已由请求层处理；其他错误不影响进入后台
    }
  },
  { immediate: true },
)

function logout() {
  clearAdminToken()
}

function to(key: string) {
  return key ? `/admin/${key}` : '/admin'
}
</script>

<template>
  <AdminLogin v-if="!loggedIn" />
  <div v-else class="adm">
    <aside class="adm-side">
      <div class="adm-brand">
        <span class="adm-brand-mark" aria-hidden="true" />
        <span>超级小伴 <small>管理后台</small></span>
      </div>
      <nav class="adm-nav" aria-label="后台导航">
        <RouterLink v-for="s in SECTIONS" :key="s.key" :to="to(s.key)" :class="{ 'is-active': s.key === current.key }">
          {{ s.label }}
          <span v-if="s.key === 'reports' && openReports > 0" class="adm-badge">{{ openReports }}</span>
        </RouterLink>
      </nav>
      <div class="adm-side-foot">
        <span class="adm-who">登录为 <b>{{ adminAuth.username || '管理员' }}</b></span>
        <div class="adm-side-links">
          <RouterLink class="btn btn-ghost btn-sm" to="/">打开论坛</RouterLink>
          <button class="btn btn-ghost btn-sm" @click="logout">退出登录</button>
        </div>
      </div>
    </aside>

    <main class="adm-main">
      <div class="adm-head">
        <h1>{{ current.label }}</h1>
      </div>
      <component :is="current.comp" :key="current.key" />
    </main>

    <div class="adm-toasts" aria-live="polite">
      <div v-for="t in toasts" :key="t.id" class="adm-toast" :class="t.kind">{{ t.text }}</div>
    </div>
  </div>
</template>
