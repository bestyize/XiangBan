import { useEffect, useRef, type ReactNode } from 'react'

interface SheetProps {
  open: boolean
  onClose: () => void
  title: string
  /** Right side of the title row, e.g. a progress count */
  aside?: ReactNode
  children: ReactNode
}

/** Bottom sheet on a native <dialog>: focus trap, Esc and backdrop tap close it. */
export function Sheet({ open, onClose, title, aside, children }: SheetProps) {
  const dialog = useRef<HTMLDialogElement>(null)

  useEffect(() => {
    const el = dialog.current
    if (!el) return
    if (open && !el.open) el.showModal()
    if (!open && el.open) el.close()
  }, [open])

  return (
    <dialog
      ref={dialog}
      className="app-sheet"
      aria-label={title}
      onCancel={(e) => { e.preventDefault(); onClose() }}
      // Any other way the dialog closes still has to reach React state, or it can never reopen.
      onClose={() => { if (open) onClose() }}
      onClick={(e) => { if (e.target === e.currentTarget) onClose() }}
    >
      <div className="app-sheet-handle" aria-hidden="true" />
      <header className="app-sheet-header">
        <h2>{title}</h2>
        {aside}
        <button type="button" className="app-sheet-close" onClick={onClose} aria-label="关闭">
          <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </header>
      <div className="app-sheet-body">{open && children}</div>
    </dialog>
  )
}
