import { useEffect, useRef, useState } from 'react'
import { Link, useLocation } from 'wouter'
import { api, getToken } from '../api/client.ts'
import { TERMS_VERSION } from '../lib/consent.ts'
import { clearLoginReturn, readLoginReturn } from '../lib/loginReturn.ts'
import { useSession } from '../lib/session.tsx'
import { EmailBind } from '../pages/me/EmailBind.tsx'

/** Centered login dialog over a dimmed page, opened from anywhere via useLoginModal(). */
export function LoginModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null)
  const { status } = useSession()
  const [location, navigate] = useLocation()
  const [agreed, setAgreed] = useState(true)

  useEffect(() => {
    const el = dialog.current
    if (!el) return
    if (open && !el.open) el.showModal()
    if (!open && el.open) el.close()
  }, [open])

  // Dismissed without logging in: forget where a reminder wanted to resume, so a later login doesn't jump there.
  function dismiss() { clearLoginReturn(); onClose() }

  function done() {
    const back = readLoginReturn()?.path
    onClose()
    if (back && back !== location) navigate(back, { replace: true })
    else if (location === '/hello') navigate('/', { replace: true })
  }

  return (
    <dialog
      ref={dialog}
      className="app-modal"
      aria-labelledby="login-modal-title"
      onCancel={(e) => { e.preventDefault(); dismiss() }}
      onClose={() => { if (open) dismiss() }}
      onClick={(e) => { if (e.target === e.currentTarget) dismiss() }}
    >
      <div className="app-modal-card">
        <button type="button" className="app-modal-close" onClick={dismiss} aria-label="关闭">
          <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
        <h2 id="login-modal-title" className="text-center text-xl font-bold text-window">登录 / 注册</h2>
        <p className="mt-1.5 text-center text-[13px] text-window/55">邮箱验证码登录，新邮箱自动注册</p>
        <div className="mt-6">
          {open && (status === 'ready'
            ? <EmailBind mode="login" disabled={!agreed} termsVersion={TERMS_VERSION} onDone={done} beforeSend={async () => {
              if (getToken()) await api('/api/consent', { method: 'PUT', body: { termsVersion: TERMS_VERSION } })
            }} />
            : <p className="py-6 text-center text-sm text-window/60">{status === 'error' ? '小伴没接到你，关掉再试一次吧。' : '小伴正在开门…'}</p>)}
        </div>
        <label className="mt-5 flex justify-center gap-2 text-[12px] leading-relaxed text-window/55">
          <input type="checkbox" className="mt-0.5 size-3.5 shrink-0 accent-[var(--color-lamp)]" checked={agreed} onChange={(e) => setAgreed(e.target.checked)} />
          <span>已阅读并同意<Link href="/terms" onClick={dismiss} className="text-lamp">《用户协议》</Link>和<Link href="/privacy" onClick={dismiss} className="text-lamp">《隐私政策》</Link></span>
        </label>
      </div>
    </dialog>
  )
}
