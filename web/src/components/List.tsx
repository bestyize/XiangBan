import type { ReactNode } from 'react'
import { Link } from 'wouter'

/** Grouped rows, like a native settings screen. */
export function ListGroup({ title, footer, children }: { title?: string; footer?: ReactNode; children: ReactNode }) {
  return (
    <section className="app-list-group">
      {title && <h2 className="app-list-title">{title}</h2>}
      <div className="app-list">{children}</div>
      {footer && <p className="app-list-footer">{footer}</p>}
    </section>
  )
}

const chevron = (
  <svg viewBox="0 0 24 24" className="app-row-chevron" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 5l7 7-7 7" /></svg>
)

interface RowProps {
  label: ReactNode
  detail?: ReactNode
  tone?: 'default' | 'danger' | 'accent'
}

function RowContent({ label, detail, tone = 'default', arrow }: RowProps & { arrow: boolean }) {
  return (
    <>
      <span className={`app-row-label${tone !== 'default' ? ` is-${tone}` : ''}`}>{label}</span>
      {detail !== undefined && <span className="app-row-detail">{detail}</span>}
      {arrow && chevron}
    </>
  )
}

export function LinkRow({ href, ...rest }: RowProps & { href: string }) {
  return <Link href={href} className="app-row is-pressable"><RowContent {...rest} arrow /></Link>
}

export function ButtonRow({ onClick, disabled, ...rest }: RowProps & { onClick: () => void; disabled?: boolean }) {
  return (
    <button type="button" className="app-row is-pressable" onClick={onClick} disabled={disabled}>
      <RowContent {...rest} arrow={false} />
    </button>
  )
}

export function InfoRow(props: RowProps) {
  return <div className="app-row"><RowContent {...props} arrow={false} /></div>
}

/** An on/off row with a native-style switch. */
export function SwitchRow({ label, hint, checked, disabled, onChange }: {
  label: ReactNode
  hint?: ReactNode
  checked: boolean
  disabled?: boolean
  onChange: (next: boolean) => void
}) {
  return (
    <label className={`app-row${disabled ? ' is-disabled' : ''}`}>
      <span className="app-row-label">
        {label}
        {hint && <span className="app-row-hint">{hint}</span>}
      </span>
      <input type="checkbox" role="switch" className="app-switch" checked={checked} disabled={disabled}
        onChange={(e) => onChange(e.target.checked)} />
    </label>
  )
}
