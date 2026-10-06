import { Link, useLocation } from 'wouter'

const TABS = [
  { href: '/', label: '相伴', icon: 'M3 10.5 12 3l9 7.5M5 9v11h5v-6h4v6h5V9' },
  { href: '/games', label: '玩一局', icon: 'M7 7h10c2 0 3 2 3.5 4l1 6c.4 2-1.6 3-3 1.5L16 16H8l-2.5 2.5C4 20 2 19 2.5 17l1-6C4 9 5 7 7 7ZM8 10v5m-2.5-2.5h5M15.5 11.5h.01m2 2h.01' },
  { href: '/tests', label: '趣味测试', icon: 'M12 3 14.8 9.2 21.5 10l-5 4.5 1.4 6.5-5.9-3.4-5.9 3.4 1.4-6.5-5-4.5 6.7-.8L12 3Z' },
  { href: '/me', label: '我的', icon: 'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM4 21v-2a8 8 0 0 1 16 0v2' },
] as const

export function TabBar() {
  const [location] = useLocation()
  return <nav aria-label="主导航" className="app-tab-bar">
    {TABS.map((tab) => {
      const active = tab.href === '/'
        ? location === '/'
        : location === tab.href || location.startsWith(`${tab.href}/`)
      return <Link key={tab.href} href={tab.href} aria-current={active ? 'page' : undefined} className={`app-tab${active ? ' is-active' : ''}`}>
        <span className="app-tab-icon" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? '1.9' : '1.6'} strokeLinecap="round" strokeLinejoin="round"><path d={tab.icon} /></svg></span>
        <span>{tab.label}</span>
      </Link>
    })}
  </nav>
}

/** Includes breathing room above the fixed navigation and the device safe area. */
export const TAB_BAR_SPACE = 'calc(var(--safe-bottom) + 88px)'
