import { useEffect, useState, type ReactNode } from 'react'

/** Mount on first visit and retain each tab's DOM, state and independent scroll position. */
export function KeepAlivePage({ active, children }: { active: boolean; children: ReactNode }) {
  const [visited, setVisited] = useState(active)
  useEffect(() => { if (active) setVisited(true) }, [active])
  if (!active && !visited) return null
  return <div hidden={!active} className="app-tab-page">{children}</div>
}
