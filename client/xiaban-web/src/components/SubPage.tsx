import type { ReactNode } from 'react'
import { useLocation } from 'wouter'
import { Screen } from './ui.tsx'

interface NavBarProps {
  title: string
  /** Where the back arrow goes; without it the browser history is used */
  back?: string
  /** Right side of the navigation bar */
  action?: ReactNode
}

/** Sticky bar of a pushed screen: back arrow, centered title, optional action. */
export function NavBar({ title, back, action }: NavBarProps) {
  const [, navigate] = useLocation()
  const goBack = () => {
    if (back) navigate(back)
    else if (history.length > 1) history.back()
    else navigate('/')
  }
  return (
    <header className="app-nav-bar">
      <button type="button" className="app-nav-back" onClick={goBack} aria-label="返回">
        <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M15 5l-7 7 7 7" /></svg>
      </button>
      <h1>{title}</h1>
      <div className="app-nav-action">{action}</div>
    </header>
  )
}

/** A pushed screen sliding in like a native page. */
export function SubPage({ className = '', children, ...bar }: NavBarProps & { className?: string; children: ReactNode }) {
  return (
    <Screen className={`app-subpage ${className}`}>
      <NavBar {...bar} />
      {children}
    </Screen>
  )
}
