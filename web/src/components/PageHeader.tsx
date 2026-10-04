import type { ReactNode } from 'react'

export function PageHeader({ title, children }: { title: string; children?: ReactNode }) {
  return <header className="app-page-header">
    <h1>{title}</h1>
    {children && <div className="app-page-actions">{children}</div>}
  </header>
}
