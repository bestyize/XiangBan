<script setup lang="ts">
// 管理后台外壳：4 大域 × 12 项导航 + 二级聚合 Tab，100% 兼容 23 个历史 /admin/:section 路由。
import { computed, onMounted, onUnmounted, ref, watch, type Component } from 'vue'
import { useRoute } from 'vue-router'
import { adminApi, adminAuth, clearAdminToken } from '@/api/admin'
import BrandMark from '@/components/BrandMark.vue'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import AdminCommandPalette from './AdminCommandPalette.vue'
import AdminIcon from './AdminIcon.vue'
import AdminLogin from './AdminLogin.vue'
import { ADMIN_NAV_GROUPS, adminSectionPath, resolveAdminRoute, type AdminSubTab } from './adminNav'
import AccountPanel from './AccountPanel.vue'
import AuditPanel from './AuditPanel.vue'
import CategoriesPanel from './CategoriesPanel.vue'
import CollectionPanel from './CollectionPanel.vue'
import CommunityEditorialPanel from './CommunityEditorialPanel.vue'
import CommunityOperationsPanel from './CommunityOperationsPanel.vue'
import ContactPanel from './ContactPanel.vue'
import ContentReportsPanel from './ContentReportsPanel.vue'
import DiscoverPanel from './DiscoverPanel.vue'
import EditorAccounts from './EditorAccounts.vue'
import EditorPanel from './EditorPanel.vue'
import MetricsPanel from './MetricsPanel.vue'
import ModerationPanel from './ModerationPanel.vue'
import OpenSourcePanel from './OpenSourcePanel.vue'
import OverviewPanel from './OverviewPanel.vue'
import PacksPanel from './PacksPanel.vue'
import ProjectsPanel from './ProjectsPanel.vue'
import ReportsPanel from './ReportsPanel.vue'
import ToolMirrorsPanel from './ToolMirrorsPanel.vue'
import TopicsPanel from './TopicsPanel.vue'
import UsersPanel from './UsersPanel.vue'
import WeeklyPanel from './WeeklyPanel.vue'
import WordsPanel from './WordsPanel.vue'
import {
  openContentReports,
  openModeration,
  openReports,
  refreshReviewBadges,
  toasts,
  totalReviewTodo,
} from './shared'
import './admin.css'

const { preference, cycleTheme } = useTheme()
const route = useRoute()
const sideOpen = ref(false)
const cmdOpen = ref(false)

const PANEL_MAP: Record<string, Component> = {
  '': OverviewPanel,
  metrics: MetricsPanel,
  reports: ReportsPanel,
  'content-reports': ContentReportsPanel,
  moderation: ModerationPanel,
  topics: TopicsPanel,
  users: UsersPanel,
  'editor-accounts': EditorAccounts,
  categories: CategoriesPanel,
  words: WordsPanel,
  discover: DiscoverPanel,
  'open-source': OpenSourcePanel,
  collection: CollectionPanel,
  'community-operations': CommunityOperationsPanel,
  'tool-mirrors': ToolMirrorsPanel,
  packs: PacksPanel,
  'community-editorial': CommunityEditorialPanel,
  projects: ProjectsPanel,
  weekly: WeeklyPanel,
  editor: EditorPanel,
  contact: ContactPanel,
  audit: AuditPanel,
  account: AccountPanel,
}

const resolved = computed(() => resolveAdminRoute(String(route.params.section ?? '')))
const currentComp = computed(() => PANEL_MAP[resolved.value.section] ?? OverviewPanel)
const loggedIn = computed(() => !!adminAuth.token)
const adminInitials = computed(() => (adminAuth.username || 'AD').slice(0, 2).toUpperCase())

function navBadge(navId: string): number {
  if (navId === 'review') return totalReviewTodo.value
  return 0
}

function subtabBadge(tab: AdminSubTab): number {
  if (tab.badgeKey === 'reports') return openReports.value
  if (tab.badgeKey === 'contentReports') return openContentReports.value
  if (tab.badgeKey === 'moderation') return openModeration.value
  return 0
}

watch(
  loggedIn,
  async ok => {
    if (!ok) return
    try {
      const me = await adminApi.me()
      adminAuth.username = me.username
      await refreshReviewBadges()
    } catch {
      // 401 已由请求层处理
    }
  },
  { immediate: true },
)

watch(
  () => route.fullPath,
  () => {
    sideOpen.value = false
  },
)

function onGlobalKey(e: KeyboardEvent) {
  if (!loggedIn.value) return
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    cmdOpen.value = !cmdOpen.value
  }
}

onMounted(() => window.addEventListener('keydown', onGlobalKey))
onUnmounted(() => window.removeEventListener('keydown', onGlobalKey))

function logout() {
  clearAdminToken()
}
</script>

<template>
  <AdminLogin v-if="!loggedIn" />
  <div v-else class="adm">
    <div class="adm-side-scrim" :class="{ 'is-open': sideOpen }" @click="sideOpen = false" />

    <aside class="adm-side" :class="{ 'is-open': sideOpen }">
      <div class="adm-brand">
        <span class="adm-brand-logo"><BrandMark /></span>
        <span class="adm-brand-text">
          <strong>超级小伴</strong>
          <small>社区运营台 · chat</small>
        </span>
      </div>

      <button class="adm-cmd-trigger" type="button" @click="cmdOpen = true">
        <span class="adm-cmd-trigger-l">
          <AdminIcon name="search" :size="14" />
          快速跳转…
        </span>
        <span class="adm-kbd">⌘K</span>
      </button>

      <nav class="adm-nav" aria-label="后台导航">
        <div v-for="group in ADMIN_NAV_GROUPS" :key="group.label" class="adm-nav-group">
          <span class="adm-nav-label">{{ group.label }}</span>
          <RouterLink
            v-for="item in group.items"
            :key="item.id"
            :to="adminSectionPath(item.defaultSection)"
            :class="{ 'is-active': resolved.nav.id === item.id }"
            :aria-current="resolved.nav.id === item.id ? 'page' : undefined"
          >
            <AdminIcon :name="item.icon" :size="16" />
            <span class="adm-nav-text">{{ item.label }}</span>
            <span v-if="navBadge(item.id) > 0" class="adm-badge" :class="{ 'is-soft': item.badgeType === 'soft' }">
              {{ navBadge(item.id) }}
            </span>
          </RouterLink>
        </div>
      </nav>

      <div class="adm-side-foot">
        <div class="adm-who-card">
          <span class="adm-avatar">{{ adminInitials }}</span>
          <div class="adm-who-meta">
            <b>{{ adminAuth.username || '管理员' }}</b>
            <span>管理员会话有效</span>
          </div>
        </div>
        <div class="adm-side-links">
          <RouterLink class="adm-foot-btn" to="/" title="打开社区前台">
            <AdminIcon name="external" :size="12" /> 前台
          </RouterLink>
          <button class="adm-foot-btn" type="button" aria-label="切换后台主题" @click="cycleTheme">
            <AdminIcon name="sun" :size="12" /> {{ THEME_LABEL[preference] }}
          </button>
          <button class="adm-foot-btn" type="button" @click="logout">
            <AdminIcon name="logout" :size="12" /> 退出
          </button>
        </div>
      </div>
    </aside>

    <div class="adm-col">
      <header class="adm-topbar">
        <div class="adm-topbar-left">
          <button class="adm-menu-btn" type="button" aria-label="展开导航" @click="sideOpen = true">
            <AdminIcon name="menu" :size="18" />
          </button>
          <p class="adm-breadcrumb">
            <span>超级小伴</span>
            <span>/</span>
            <span>{{ resolved.groupLabel }}</span>
            <span>/</span>
            <strong>{{ resolved.nav.label }}</strong>
            <template v-if="resolved.subtabLabel">
              <span>/</span>
              <span>{{ resolved.subtabLabel }}</span>
            </template>
          </p>
        </div>
        <div class="adm-topbar-right">
          <RouterLink v-if="totalReviewTodo > 0" class="btn btn-sm btn-soft" to="/admin/reports">
            待审核 {{ totalReviewTodo }}
          </RouterLink>
          <button class="btn btn-sm btn-ghost" type="button" title="快捷键 ⌘K" @click="cmdOpen = true">
            <AdminIcon name="search" :size="14" /> 跳转
          </button>
        </div>
      </header>

      <main class="adm-main">
        <div class="adm-head">
          <div>
            <h1>{{ resolved.nav.label }}</h1>
            <p class="adm-page-description">{{ resolved.nav.description }}</p>
          </div>
        </div>

        <nav v-if="resolved.nav.subtabs" class="adm-subtabs" :aria-label="`${resolved.nav.label}子模块`">
          <RouterLink
            v-for="tab in resolved.nav.subtabs"
            :key="tab.section"
            :to="adminSectionPath(tab.section)"
            class="adm-subtab"
            :class="{ 'is-active': resolved.section === tab.section }"
          >
            <span>{{ tab.label }}</span>
            <span
              v-if="subtabBadge(tab) > 0"
              class="adm-subtab-cnt"
              :class="{ 'is-danger': tab.badgeKey !== undefined }"
            >
              {{ subtabBadge(tab) }}
            </span>
          </RouterLink>
        </nav>

        <div class="adm-content">
          <component :is="currentComp" :key="resolved.section" />
        </div>
      </main>
    </div>

    <AdminCommandPalette :open="cmdOpen" @close="cmdOpen = false" @toggle-theme="cycleTheme" />

    <div class="adm-toasts" aria-live="polite">
      <div v-for="t in toasts" :key="t.id" class="adm-toast" :class="t.kind">{{ t.text }}</div>
    </div>
  </div>
</template>
