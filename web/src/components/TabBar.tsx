import { Link, useLocation } from 'wouter'

const TABS = [
  { href: '/', label: '下班' },
  { href: '/dex', label: '图鉴' },
  { href: '/me', label: '我的' },
] as const

export function TabBar() {
  const [location] = useLocation()
  return (
    <nav
      aria-label="主导航"
      className="fixed inset-x-0 bottom-0 z-40 mx-auto flex max-w-[480px] justify-around border-t border-window/10 bg-dusk-deep/95 backdrop-blur"
      style={{ paddingBottom: 'var(--safe-bottom)' }}
    >
      {TABS.map((tab) => {
        const active = tab.href === '/' ? location === '/' : location.startsWith(tab.href)
        return (
          <Link
            key={tab.href}
            href={tab.href}
            aria-current={active ? 'page' : undefined}
            className={`flex-1 py-3 text-center text-[15px] ${active ? 'font-bold text-lamp' : 'text-window/60'}`}
          >
            {tab.label}
          </Link>
        )
      })}
    </nav>
  )
}

/** Bottom padding so content clears the fixed tab bar. */
export const TAB_BAR_SPACE = 'calc(var(--safe-bottom) + 72px)'
