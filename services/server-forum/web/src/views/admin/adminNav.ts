// 管理后台信息架构定义：4 大域 × 12 项导航，同时 100% 向后兼容 23 个历史 /admin/:section 路由。

export type AdminIconName =
  | 'home'
  | 'chart'
  | 'shield'
  | 'doc'
  | 'users'
  | 'folder'
  | 'compass'
  | 'star'
  | 'radar'
  | 'cube'
  | 'mail'
  | 'lock'
  | 'search'
  | 'x'
  | 'plus'
  | 'refresh'
  | 'external'
  | 'check'
  | 'warn'
  | 'menu'
  | 'sun'
  | 'logout'
  | 'copy'

export interface AdminSubTab {
  section: string
  label: string
  badgeKey?: 'reports' | 'contentReports' | 'moderation'
}

export interface AdminNavItem {
  id: string
  /** 点击左侧菜单默认跳转的 section（空串表示 /admin） */
  defaultSection: string
  label: string
  icon: AdminIconName
  description: string
  badgeType?: 'danger' | 'soft'
  subtabs?: readonly AdminSubTab[]
}

export interface AdminNavGroup {
  label: string
  items: readonly AdminNavItem[]
}

export const ADMIN_NAV_GROUPS: readonly AdminNavGroup[] = [
  {
    label: '工作台',
    items: [
      {
        id: 'overview',
        defaultSection: '',
        label: '概览与待办',
        icon: 'home',
        description: '实时掌握社区核心指标、跨模块待办行动中心与自动采集健康态势',
      },
      {
        id: 'metrics',
        defaultSection: 'metrics',
        label: '数据分析',
        icon: 'chart',
        description: '匿名活跃设备、次日留存趋势与站内核心交互排行',
      },
    ],
  },
  {
    label: '审核与治理',
    items: [
      {
        id: 'review',
        defaultSection: 'reports',
        label: '审核中心',
        icon: 'shield',
        badgeType: 'danger',
        description: '聚合处理帖子与回复举报、作品与方案举报及待审内容队列',
        subtabs: [
          { section: 'reports', label: '帖子与回复举报', badgeKey: 'reports' },
          { section: 'content-reports', label: '作品与方案举报', badgeKey: 'contentReports' },
          { section: 'moderation', label: '待审内容队列', badgeKey: 'moderation' },
        ],
      },
      {
        id: 'topics',
        defaultSection: 'topics',
        label: '帖子管理',
        icon: 'doc',
        description: '多维检索社区帖子，支持侧滑治理抽屉、分类调度与批量操作',
      },
      {
        id: 'users',
        defaultSection: 'users',
        label: '用户与编辑',
        icon: 'users',
        description: '维护成员信任等级、限时禁言/封禁以及站方编辑账号与创作令牌',
        subtabs: [
          { section: 'users', label: '社区用户' },
          { section: 'editor-accounts', label: '站方编辑账号' },
        ],
      },
      {
        id: 'taxonomy',
        defaultSection: 'categories',
        label: '板块与安全',
        icon: 'folder',
        description: '树形维护社区一二级板块权限及拦截词 / 复核词库',
        subtabs: [
          { section: 'categories', label: '板块分类树' },
          { section: 'words', label: '敏感词与复核词' },
        ],
      },
    ],
  },
  {
    label: '内容运营',
    items: [
      {
        id: 'discover',
        defaultSection: 'discover',
        label: '发现页编排',
        icon: 'compass',
        description: '编排首页头图、小伴前沿、AI 日报、排行榜与导航等 12 类发现页草稿并审核发布',
      },
      {
        id: 'open-source',
        defaultSection: 'open-source',
        label: '精选项目',
        icon: 'star',
        description: '仅管理精选板块的 GitHub 项目、素材、1280×720 封面模板与发布版本',
      },
      {
        id: 'collection',
        defaultSection: 'collection',
        label: '采编与健康',
        icon: 'radar',
        description: '一站管理官方数据源采集、栏目新鲜度与外链巡检及官方工具安装包镜像校验',
        subtabs: [
          { section: 'collection', label: '采集来源与调度' },
          { section: 'community-operations', label: '栏目健康与外链巡检' },
          { section: 'tool-mirrors', label: '工具镜像校验' },
        ],
      },
      {
        id: 'works',
        defaultSection: 'packs',
        label: '作品与活动',
        icon: 'cube',
        description: '管理内容包上下架与官方登记、首页精选与挑战、专题项目、社区周刊及批量发帖',
        subtabs: [
          { section: 'packs', label: '内容包管理' },
          { section: 'community-editorial', label: '首页精选与挑战' },
          { section: 'projects', label: '专题项目' },
          { section: 'weekly', label: '社区周刊' },
          { section: 'editor', label: '站方批量发帖' },
        ],
      },
    ],
  },
  {
    label: '系统',
    items: [
      {
        id: 'ops-sys',
        defaultSection: 'contact',
        label: '站务与日志',
        icon: 'mail',
        description: '维护站务交流群二维码与失效策略，检索全站管理员审计日志',
        subtabs: [
          { section: 'contact', label: '官方联系方式' },
          { section: 'audit', label: '操作审计日志' },
        ],
      },
      {
        id: 'account',
        defaultSection: 'account',
        label: '账号安全',
        icon: 'lock',
        description: '修改管理员登录密码并管理会话安全',
      },
    ],
  },
] as const

export interface ResolvedAdminRoute {
  groupLabel: string
  nav: AdminNavItem
  section: string
  subtabLabel?: string
}

/** 根据路由参数 section 解析所属导航域与当前子 Tab；未知值安全回退到概览。 */
export function resolveAdminRoute(rawSection: string): ResolvedAdminRoute {
  const key = rawSection.trim()
  for (const group of ADMIN_NAV_GROUPS) {
    for (const nav of group.items) {
      if (nav.subtabs) {
        const hit = nav.subtabs.find(t => t.section === key)
        if (hit) return { groupLabel: group.label, nav, section: hit.section, subtabLabel: hit.label }
        if (nav.id === key) {
          const first = nav.subtabs[0]!
          return { groupLabel: group.label, nav, section: first.section, subtabLabel: first.label }
        }
      } else if (nav.defaultSection === key || nav.id === key) {
        return { groupLabel: group.label, nav, section: nav.defaultSection }
      }
    }
  }
  const fallbackGroup = ADMIN_NAV_GROUPS[0]!
  const fallbackNav = fallbackGroup.items[0]!
  return { groupLabel: fallbackGroup.label, nav: fallbackNav, section: '' }
}

export interface CommandEntry {
  section: string
  title: string
  group: string
  keywords: string
}

/** 供 ⌘K 命令面板检索的全部 23 个细分模块入口。 */
export const ADMIN_COMMANDS: readonly CommandEntry[] = ADMIN_NAV_GROUPS.flatMap(g =>
  g.items.flatMap(nav =>
    nav.subtabs
      ? nav.subtabs.map(t => ({
          section: t.section,
          title: `${nav.label} · ${t.label}`,
          group: g.label,
          keywords: `${nav.label} ${t.label} ${t.section} ${nav.id} ${g.label}`.toLowerCase(),
        }))
      : [
          {
            section: nav.defaultSection,
            title: nav.label,
            group: g.label,
            keywords: `${nav.label} ${nav.defaultSection || 'overview'} ${nav.id} ${g.label}`.toLowerCase(),
          },
        ],
  ),
)

export function adminSectionPath(section: string): string {
  return section ? `/admin/${section}` : '/admin'
}
