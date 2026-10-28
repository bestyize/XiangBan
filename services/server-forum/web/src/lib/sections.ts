import type { RouteLocationNormalizedLoaded } from 'vue-router'

export type Section = 'home' | 'resources' | 'discuss'

/** 顶栏三个一级栏目：首页（内容流）、资源（论坛里的资源分享板块）、讨论（其余板块和帖子）。 */
export function sectionOf(route: Pick<RouteLocationNormalizedLoaded, 'name' | 'params'>): Section {
  switch (route.name) {
    case 'home':
    case 'pack':
    case 'weekly-issue':
    case 'project':
    case 'publish':
    case 'works':
      return 'home'
    case 'category': {
      const slug = String(route.params.slug ?? '')
      return slug === 'share' || slug.startsWith('share-') ? 'resources' : 'discuss'
    }
    default:
      return 'discuss'
  }
}
