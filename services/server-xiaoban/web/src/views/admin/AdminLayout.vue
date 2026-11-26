<script setup lang="ts">
import '@/styles/admin.css'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Icon, { type IconName } from '@/components/Icon.vue'
import AdminCommandPalette, { type CommandEntry } from '@/components/admin/AdminCommandPalette.vue'
import * as adminApi from '@/admin/api'
import {
  ADMIN_ACCENTS,
  adminAccent,
  adminTheme,
  applyAdminRootAttrs,
  setAdminAccent,
  toggleAdminTheme,
} from '@/admin/prefs'
import { useAdmin } from '@/stores/admin'

const route = useRoute()
const router = useRouter()
const admin = useAdmin()

interface NavItem {
  to: string
  label: string
  icon: IconName
  desc: string
  badge?: () => { text: string; warn?: boolean } | null
  dot?: () => boolean
}

const pendingWallpapers = ref(0)
const latestRelease = ref('')
const activeNotices = ref(0)
const keyCoolingDown = ref(false)

const mobileMenuOpen = ref(false)
const cmdOpen = ref(false)
const accentMenuOpen = ref(false)

const NAV_GROUPS: { label: string; items: NavItem[] }[] = [
  {
    label: '总览',
    items: [
      { to: '/admin/overview', label: '运营总览', icon: 'dashboard', desc: '全局掌握图库审核、发版、公告、图源与缓存状态' },
    ],
  },
  {
    label: '内容与壁纸',
    items: [
      {
        to: '/admin/wallpapers',
        label: '壁纸图库',
        icon: 'image',
        desc: '审阅、分类并上下架库内图片与动态视频',
        badge: () => (pendingWallpapers.value > 0 ? { text: String(pendingWallpapers.value), warn: true } : null),
      },
      { to: '/admin/collector', label: '采集工作台', icon: 'compass', desc: '聚合检索外部供应商并一键采入图库' },
      { to: '/admin/upload', label: '自定义上架', icon: 'upload', desc: '上传本地 4K 图片或循环动态视频' },
    ],
  },
  {
    label: '产品与分发',
    items: [
      {
        to: '/admin/apps',
        label: 'App 版本发布',
        icon: 'package',
        desc: '上传四端安装包、管理版本号与更新说明',
        badge: () => (latestRelease.value ? { text: `v${latestRelease.value}` } : null),
      },
      {
        to: '/admin/desktop',
        label: '官网与公告',
        icon: 'megaphone',
        desc: '配置官网介绍及网站/桌面/Android 公告',
        badge: () => (activeNotices.value > 0 ? { text: String(activeNotices.value) } : null),
      },
      { to: '/admin/ai-providers', label: 'AI 模型目录', icon: 'sparkles', desc: '管理桌面端“设置 → AI 模型”服务商与默认推荐' },
    ],
  },
  {
    label: '基础设施',
    items: [
      {
        to: '/admin/keys',
        label: '图源与密钥',
        icon: 'key',
        desc: '外部图源 API 密钥池、代理开关与直链/副本策略',
        dot: () => keyCoolingDown.value,
      },
      { to: '/admin/proxies', label: '代理池', icon: 'network', desc: '出网代理轮换策略与连通性测速' },
      { to: '/admin/cache', label: '搜索缓存', icon: 'database', desc: 'PostgreSQL 搜索缓存命中率与容量策略' },
    ],
  },
]

const currentTitle = computed(() => {
  if (route.path === '/admin/account') return '管理员账户'
  for (const g of NAV_GROUPS) {
    const f = g.items.find((i) => route.path.startsWith(i.to))
    if (f) return f.label
  }
  return '管理后台'
})

const commands = computed<CommandEntry[]>(() => [
  ...NAV_GROUPS.flatMap((g) =>
    g.items.map((i) => ({
      id: i.to,
      label: i.label,
      hint: g.label,
      icon: i.icon,
      run: () => void router.push(i.to),
    })),
  ),
  {
    id: '/admin/account',
    label: '管理员账户',
    hint: '系统',
    icon: 'shield',
    run: () => void router.push('/admin/account'),
  },
  {
    id: 'toggle-theme',
    label: adminTheme.value === 'light' ? '切换深色暗室模式' : '切换浅色工作台模式',
    hint: '外观',
    icon: adminTheme.value === 'light' ? 'moon' : 'sun',
    run: () => toggleAdminTheme(),
  },
])

async function refreshBadges() {
  const [wpRes, relRes, deskRes, provRes] = await Promise.allSettled([
    adminApi.listAdminWallpapers({ status: 'pending', page: 1, per_page: 1 }),
    adminApi.listAppReleases(),
    adminApi.getDesktopConfig(),
    adminApi.listProviders(),
  ])
  if (wpRes.status === 'fulfilled') pendingWallpapers.value = wpRes.value.total
  if (relRes.status === 'fulfilled') {
    const pub = relRes.value.find((r) => r.published)
    latestRelease.value = pub?.version ?? ''
  }
  if (deskRes.status === 'fulfilled') {
    activeNotices.value = deskRes.value.announcements.filter((a) => a.published).length
  }
  if (provRes.status === 'fulfilled') {
    keyCoolingDown.value = provRes.value.some((p) => p.keys.some((k) => !!k.cooldown_secs))
  }
}

watch(
  () => route.path,
  () => {
    mobileMenuOpen.value = false
    accentMenuOpen.value = false
  },
)

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    cmdOpen.value = !cmdOpen.value
  }
}

function onDocClick() {
  accentMenuOpen.value = false
}

const previousTitle = document.title

onMounted(() => {
  applyAdminRootAttrs(true)
  document.title = '小伴控制台 · 管理后台'
  adminApi.setUnauthorizedHandler(() => {
    admin.clear()
    router.replace({ path: '/admin/login', query: { redirect: route.fullPath } })
  })
  document.addEventListener('keydown', onKeydown)
  document.addEventListener('click', onDocClick)
  void refreshBadges()
})

onBeforeUnmount(() => {
  applyAdminRootAttrs(false)
  document.title = previousTitle
  adminApi.setUnauthorizedHandler(null)
  document.removeEventListener('keydown', onKeydown)
  document.removeEventListener('click', onDocClick)
})

async function signOut() {
  await admin.logout()
  router.replace('/admin/login')
}
</script>

<template>
  <div class="xb-admin control-app">
    <div v-if="mobileMenuOpen" class="mobile-scrim" @click="mobileMenuOpen = false" />

    <aside class="xb-side" :class="{ open: mobileMenuOpen }">
      <RouterLink to="/admin/overview" class="side-brand" title="wallpaper.xiangban.xyz · 返回运营总览">
        <span class="xb-mark">
          <img src="/app-icon.png" alt="小伴" @error="($event.target as HTMLImageElement).style.display = 'none'" />
        </span>
        <span class="side-brand-name">小伴控制台</span>
      </RouterLink>

      <button type="button" class="xb-search-trigger" title="快捷跳转与操作 (⌘K)" @click="cmdOpen = true">
        <Icon name="search" :size="14" />
        <span>快速跳转…</span>
        <span class="xb-kbd-mini">⌘K</span>
      </button>

      <nav class="side-nav" aria-label="管理后台导航">
        <div v-for="group in NAV_GROUPS" :key="group.label">
          <div class="side-group-title">{{ group.label }}</div>
          <div class="side-list">
            <RouterLink
              v-for="item in group.items"
              :key="item.to"
              :to="item.to"
              class="side-item"
              active-class="is-active"
              :title="item.desc"
              :aria-description="item.desc"
            >
              <Icon :name="item.icon" :size="15" />
              <span>{{ item.label }}</span>
              <span
                v-if="item.badge?.()"
                class="side-count"
                :class="{ 'is-warn': item.badge()?.warn }"
              >
                {{ item.badge()?.text }}
              </span>
              <span v-else-if="item.dot?.()" class="side-dot" title="有图源密钥正在冷却" />
            </RouterLink>
          </div>
        </div>
      </nav>

      <div class="side-foot">
        <div class="foot-bar">
          <RouterLink to="/" class="xb-btn-ghost is-sm" title="返回小伴官网前台">
            <Icon name="external" :size="13" />
            <span>访问前台</span>
          </RouterLink>
          <div class="foot-tools">
            <div class="accent-wrap" @click.stop>
              <button
                type="button"
                class="xb-icon-btn is-sm"
                title="切换主题色"
                aria-label="切换主题色"
                @click="accentMenuOpen = !accentMenuOpen"
              >
                <Icon name="palette" :size="15" />
              </button>
              <div v-if="accentMenuOpen" class="accent-pop" role="menu" aria-label="主题色">
                <button
                  v-for="a in ADMIN_ACCENTS"
                  :key="a.id"
                  type="button"
                  class="accent-dot"
                  :class="{ 'is-active': adminAccent === a.id }"
                  :style="{ background: a.color }"
                  :title="a.label"
                  :aria-label="a.label"
                  @click="setAdminAccent(a.id); accentMenuOpen = false"
                />
              </div>
            </div>
            <button
              type="button"
              class="xb-icon-btn is-sm"
              :title="adminTheme === 'light' ? '切换深色暗室模式' : '切换浅色工作台'"
              :aria-label="adminTheme === 'light' ? '切换深色暗室模式' : '切换浅色工作台'"
              @click="toggleAdminTheme"
            >
              <Icon :name="adminTheme === 'light' ? 'moon' : 'sun'" :size="15" />
            </button>
          </div>
        </div>

        <div class="side-acc">
          <RouterLink to="/admin/account" class="side-acc-btn" title="修改管理员用户名与密码">
            <span class="side-avatar">{{ (admin.user?.username || 'A').slice(0, 1).toUpperCase() }}</span>
            <span class="side-acc-name">{{ admin.user?.username || 'admin' }}</span>
          </RouterLink>
          <button type="button" class="xb-icon-btn is-sm" title="退出登录" aria-label="退出登录" @click="signOut">
            <Icon name="logout" :size="14" />
          </button>
        </div>
      </div>
    </aside>

    <section class="control-main">
      <div class="mobile-bar">
        <button type="button" class="xb-icon-btn" aria-label="打开菜单" @click="mobileMenuOpen = true">
          <Icon name="menu" :size="18" />
        </button>
        <strong class="mobile-title">{{ currentTitle }}</strong>
        <button type="button" class="xb-icon-btn" aria-label="快捷跳转" @click="cmdOpen = true">
          <Icon name="search" :size="16" />
        </button>
      </div>

      <main class="control-workspace">
        <div v-if="admin.user?.is_default_password && route.path !== '/admin/account'" class="alert-wrap">
          <div class="xb-alert" role="alert">
            <div class="xb-alert-l">
              <Icon name="alert" :size="15" />
              <span>当前仍在使用默认管理员密码，存在未授权登录风险。</span>
            </div>
            <RouterLink to="/admin/account" class="xb-btn-ghost is-sm" style="color: inherit; font-weight: 600">
              修改密码
            </RouterLink>
          </div>
        </div>
        <RouterView />
      </main>
    </section>

    <AdminCommandPalette :open="cmdOpen" :commands="commands" @close="cmdOpen = false" />
  </div>
</template>

<style scoped>
.control-app {
  height: 100vh;
  height: 100dvh;
  display: grid;
  grid-template-columns: 232px minmax(0, 1fr);
  background: var(--ui-tint);
  overflow: hidden;
  position: relative;
}

.xb-side {
  display: flex;
  flex-direction: column;
  padding: 14px 10px 12px;
  overflow-y: auto;
  scrollbar-width: none;
  background: var(--ui-tint);
}
.xb-side::-webkit-scrollbar { display: none; }

.side-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 44px;
  padding: 0 8px;
  border-radius: 10px;
  transition: background 0.14s;
}
.side-brand:hover { background: var(--ui-surface-hover); }

.xb-mark {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  flex: none;
  overflow: hidden;
  background: linear-gradient(150deg, var(--ui-btn-hi), var(--ui-btn));
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.xb-mark img { width: 100%; height: 100%; object-fit: cover; }

.side-brand-name {
  font-size: 14.5px;
  font-weight: 650;
  line-height: 1;
  letter-spacing: -0.01em;
}

.xb-search-trigger {
  margin: 8px 2px 12px;
  height: 32px;
  padding: 0 10px;
  border-radius: 8px;
  background: var(--ui-surface-hover);
  color: var(--ui-text-secondary);
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12.5px;
  line-height: 1;
  transition: background 0.14s, color 0.14s;
}
.xb-search-trigger:hover {
  background: var(--ui-control);
  color: var(--ui-text-primary);
}
.xb-kbd-mini {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 19px;
  padding: 0 5px;
  border-radius: 5px;
  line-height: 1;
  font-family: var(--ui-mono);
  font-size: 10.5px;
  color: var(--ui-text-tertiary);
  background: var(--ui-surface);
}

.side-nav {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}
.side-group-title {
  padding: 0 10px 4px;
  font-size: 11px;
  font-weight: 600;
  color: var(--ui-text-tertiary);
  line-height: 1;
}
.side-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.side-item {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 34px;
  padding: 0 10px;
  border-radius: 8px;
  color: var(--ui-text-secondary);
  font-size: 13px;
  font-weight: 450;
  line-height: 1;
  transition: background 0.14s, color 0.14s;
}
.side-item:hover {
  background: var(--ui-surface-hover);
  color: var(--ui-text-primary);
}
.side-item.is-active {
  background: var(--ui-soft);
  color: var(--ui-ink);
  font-weight: 600;
}
.side-count {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 18px;
  padding: 0 6px;
  border-radius: 999px;
  line-height: 1;
  font-size: 11px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  background: var(--ui-surface-hover);
  color: var(--ui-text-secondary);
}
.side-count.is-warn {
  background: var(--ui-warning-soft);
  color: var(--ui-warning-text);
}
.side-dot {
  margin-left: auto;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--ui-warning);
}

.side-foot {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--ui-line);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.foot-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 2px;
}
.foot-tools {
  display: flex;
  align-items: center;
  gap: 2px;
}
.accent-wrap {
  position: relative;
}
.accent-pop {
  position: absolute;
  bottom: 34px;
  right: 0;
  padding: 7px 9px;
  border-radius: 10px;
  background: var(--ui-surface-raised);
  box-shadow: var(--ui-shadow-menu);
  display: flex;
  align-items: center;
  gap: 7px;
  z-index: 40;
}
.accent-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid transparent;
  transition: transform 0.14s;
}
.accent-dot:hover { transform: scale(1.15); }
.accent-dot.is-active {
  box-shadow: 0 0 0 2px var(--ui-surface), 0 0 0 3.5px var(--ui-text-primary);
}

.side-acc {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 40px;
  padding: 0 6px;
  border-radius: 10px;
  transition: background 0.14s;
}
.side-acc:hover { background: var(--ui-surface-hover); }
.side-acc-btn {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 9px;
}
.side-avatar {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  flex: none;
  background: var(--ui-soft);
  color: var(--ui-ink);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 650;
  line-height: 1;
}
.side-acc-name {
  font-size: 13px;
  font-weight: 550;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.control-main {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  margin: 8px 8px 8px 0;
  background: var(--ui-surface);
  border-radius: 12px;
  box-shadow: var(--ui-shadow-panel);
  overflow: hidden;
}
.control-workspace {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.alert-wrap {
  max-width: 1240px;
  margin: 16px auto 0;
  padding: 0 28px;
}

.mobile-bar {
  display: none;
  height: 48px;
  padding: 0 12px;
  flex: none;
  border-bottom: 1px solid var(--ui-line);
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.mobile-title { font-size: 14px; font-weight: 600; }
.mobile-scrim {
  display: none;
  position: fixed;
  inset: 0;
  z-index: 75;
  background: var(--ui-scrim);
}

@media (max-width: 860px) {
  .control-app { grid-template-columns: minmax(0, 1fr); }
  .mobile-scrim { display: block; }
  .xb-side {
    position: fixed;
    inset: 0 auto 0 0;
    width: 256px;
    z-index: 80;
    transform: translateX(-105%);
    transition: transform 0.24s var(--ui-ease);
    box-shadow: var(--ui-shadow-dialog);
  }
  .xb-side.open { transform: translateX(0); }
  .control-main { margin: 0; border-radius: 0; box-shadow: none; }
  .mobile-bar { display: flex; }
  .alert-wrap { padding: 0 16px; }
}
</style>
