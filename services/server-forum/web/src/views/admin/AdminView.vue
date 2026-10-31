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
import MetricsPanel from './MetricsPanel.vue'
import ModerationPanel from './ModerationPanel.vue'
import WordsPanel from './WordsPanel.vue'
import PacksPanel from './PacksPanel.vue'
import ProjectsPanel from './ProjectsPanel.vue'
import WeeklyPanel from './WeeklyPanel.vue'
import DiscoverPanel from './DiscoverPanel.vue'
import CollectionPanel from './CollectionPanel.vue'
import ToolMirrorsPanel from './ToolMirrorsPanel.vue'
import CommunityEditorialPanel from './CommunityEditorialPanel.vue'
import AuditPanel from './AuditPanel.vue'
import AccountPanel from './AccountPanel.vue'
import { openReports, toasts } from './shared'
import './admin.css'
import { useTheme, THEME_LABEL } from '@/lib/theme'
const { preference, cycleTheme } = useTheme()

const SECTIONS = [
  { key: '', label: '概览', comp: OverviewPanel },
  { key: 'metrics', label: '数据', comp: MetricsPanel },
  { key: 'reports', label: '举报', comp: ReportsPanel },
  { key: 'topics', label: '帖子', comp: TopicsPanel },
  { key: 'users', label: '用户', comp: UsersPanel },
  { key: 'editor-accounts', label: '编辑账号', comp: EditorAccounts },
  { key: 'editor', label: '批量发帖', comp: EditorPanel },
  { key: 'categories', label: '分类', comp: CategoriesPanel },
  { key: 'moderation', label: '待审', comp: ModerationPanel },
  { key: 'words', label: '敏感词', comp: WordsPanel },
  { key: 'packs', label: '内容包', comp: PacksPanel },
  { key: 'projects', label: '项目', comp: ProjectsPanel },
  { key: 'weekly', label: '周刊', comp: WeeklyPanel },
  { key: 'discover', label: '发现页', comp: DiscoverPanel },
  { key: 'collection', label: '内容采集', comp: CollectionPanel },
  { key: 'tool-mirrors', label: '工具镜像', comp: ToolMirrorsPanel },
  { key: 'community-editorial', label: '精选与挑战', comp: CommunityEditorialPanel },
  { key: 'audit', label: '操作日志', comp: AuditPanel },
  { key: 'account', label: '账号', comp: AccountPanel },
] as const

const GROUPS = [
  { label: '工作台', keys: ['', 'metrics'] },
  { label: '内容运营', keys: ['discover', 'collection', 'tool-mirrors', 'community-editorial', 'packs', 'projects', 'weekly', 'editor', 'editor-accounts'] },
  { label: '社区治理', keys: ['reports', 'moderation', 'topics', 'users', 'categories', 'words'] },
  { label: '系统', keys: ['audit', 'account'] },
].map(group => ({...group, sections: group.keys.map(key => SECTIONS.find(section => section.key === key)!)}))
const DESCRIPTIONS: Record<string,string> = {
  'tool-mirrors': '核验官方安装包，管理资源镜像和下载回退',
  collection: '管理官方数据源，检查采集记录与待审内容', discover: '编排社区栏目，保存草稿并审核发布',
  'community-editorial': '选择首页作品，发起挑战并管理投稿', packs: '审核作品、版本和资源来源',
  projects: '维护项目介绍与讨论入口', weekly: '组织本期内容，发布社区周刊',
}
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
        <div v-for="group in GROUPS" :key="group.label" class="adm-nav-group">
          <span class="adm-nav-label">{{ group.label }}</span>
          <RouterLink v-for="s in group.sections" :key="s.key" :to="to(s.key)" :class="{ 'is-active': s.key === current.key }" :aria-current="s.key === current.key ? 'page' : undefined">
            {{ s.label }}<span v-if="s.key === 'reports' && openReports > 0" class="adm-badge">{{ openReports }}</span>
          </RouterLink>
        </div>
      </nav>
      <div class="adm-side-foot">
        <span class="adm-who">登录为 <b>{{ adminAuth.username || '管理员' }}</b></span>
        <div class="adm-side-links">
          <RouterLink class="btn btn-ghost btn-sm" to="/">打开论坛</RouterLink>
          <button class="btn btn-ghost btn-sm" type="button" aria-label="切换后台主题" @click="cycleTheme">{{ THEME_LABEL[preference] }}外观</button>
          <button class="btn btn-ghost btn-sm" @click="logout">退出登录</button>
        </div>
      </div>
    </aside>

    <main class="adm-main">
      <div class="adm-head">
        <div><p class="adm-breadcrumb">超级小伴 <span>/</span> 管理后台</p><h1>{{ current.label }}</h1><p v-if="DESCRIPTIONS[current.key]" class="adm-page-description">{{ DESCRIPTIONS[current.key] }}</p></div>
      </div>
      <div class="adm-content"><component :is="current.comp" :key="current.key" /></div>
    </main>

    <div class="adm-toasts" aria-live="polite">
      <div v-for="t in toasts" :key="t.id" class="adm-toast" :class="t.kind">{{ t.text }}</div>
    </div>
  </div>
</template>
