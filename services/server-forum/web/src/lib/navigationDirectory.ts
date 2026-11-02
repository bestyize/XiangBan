import catalog from '../../../editorial/navigation.json' with { type: 'json' }
import type { NavItem, Group } from '../api/discover.ts'

export const navigationCategories = catalog.categories
export const navigationCatalogVersion = catalog.version
export interface NavigationDirectory { groups: Group<NavItem>[]; catalog_version?: number }
const categories = new Set(navigationCategories.map(category => category.key))
const defaults = new Map(catalog.items.map(item => [item.key, item]))
const legacyDefaults = catalog.legacy_items as Record<string, {name:string;vendor:string;url:string}>
const legacyKeys = new Set(Object.keys(legacyDefaults))
function knownItem(item: NavItem) { return defaults.get(item.key) || catalog.items.find(entry => entry.url.replace(/\/$/, '') === item.url.replace(/\/$/, '')) }

export function navigationCategory(item: NavItem, group = ''): string {
  if (item.category && categories.has(item.category)) return item.category
  return knownItem(item)?.category || navigationCategories.find(category => category.label === group)?.key || 'models'
}

/** New servers own the published directory; older snapshots receive the requested taxonomy upgrade. */
export function navigationGroups(data: NavigationDirectory, isAllowed: (url: string) => boolean): Group<NavItem>[] {
  const upgrading = (data.catalog_version || 0) < navigationCatalogVersion
  const entries: NavItem[] = data.groups.flatMap(group => group.items.filter(item => isAllowed(item.url)).map(item => {
    const category = navigationCategory(item, group.name)
    const known = upgrading ? knownItem(item) : undefined
    const merged: NavItem = {...known, ...item}
    if (known && upgrading && !item.category) {
      const legacy = legacyDefaults[item.key]
      for (const field of ['name','vendor','url'] as const) if (legacy && item[field] === legacy[field]) merged[field] = known[field]
      if (!item.logo && merged.vendor !== known.vendor) delete merged.logo
    }
    return { ...merged, category,
      ...(upgrading && !item.category ? { summary: item.summary || known?.summary || '', subtype: known?.subtype || '', region: known?.region || (group.name === '国产' ? 'cn' : group.name === '海外' ? 'global' : '') } : {}) }
  }))
  if (upgrading) {
    const keys = new Set(entries.map(item => item.key))
    const identities = new Set(entries.map(item => `${item.category}:${item.url.replace(/\/$/, '')}`))
    for (const item of catalog.items) {
      // v1 editors may already have removed a legacy entry. Do not restore it during the v2 upgrade.
      if (data.catalog_version === 1 && legacyKeys.has(item.key) && !keys.has(item.key)) continue
      const identity = `${item.category}:${item.url.replace(/\/$/, '')}`
      if (keys.has(item.key) || identities.has(identity) || !isAllowed(item.url)) continue
      entries.push(item); keys.add(item.key); identities.add(identity)
    }
    const order = new Map(catalog.items.map((item, index) => [item.key, index]))
    entries.sort((a,b) => (order.get(a.key) ?? 10000) - (order.get(b.key) ?? 10000))
  }
  return navigationCategories.map(category => ({ name: category.label, items: entries.filter(item => item.category === category.key) })).filter(group => group.items.length)
}

export function navigationMatches(item: NavItem, category: string, filter = ''): boolean {
  if (item.category !== category) return false
  if (!filter) return true
  if (category === 'models') return item.region === filter
  if (category === 'api' && filter === 'thirdparty') return item.subtype === 'router' || item.subtype === 'inference'
  return item.subtype === filter
}
export function navigationType(item: NavItem): string {
  if (item.category === 'models') return item.region === 'cn' ? '国产原厂' : item.region === 'global' ? '海外原厂' : '模型原厂'
  if (item.subtype === 'router') return '聚合路由'
  if (item.subtype === 'inference') return '推理服务'
  return navigationCategories.find(category => category.key === item.category)?.filters.find(filter => filter.key === item.subtype)?.label || ''
}
export function navigationLogo(item: NavItem): string {
  const known = defaults.get(item.key)
  return item.logo || (known && known.vendor === item.vendor ? known.logo : '') || item.name || item.vendor || ''
}
export function navigationIcon(item: NavItem): string | undefined {
  return item.key === 'hermes-agent' && item.vendor === 'Nous Research' ? '/vendor-logos/hermes-agent.png' : undefined
}
export function navigationDomain(url: string): string {
  try { return new URL(url).hostname.replace(/^www\./, '') } catch { return '' }
}
