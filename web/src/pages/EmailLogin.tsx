import { useState } from 'react'
import { Link, useLocation } from 'wouter'
import { api, getToken } from '../api/client.ts'
import { CatWindow, Screen, QuietButton, FieldError } from '../components/ui.tsx'
import { TERMS_VERSION } from '../lib/consent.ts'
import { useProfile } from '../lib/session.tsx'
import { logout } from '../lib/logout.ts'
import { EmailBind } from './me/EmailBind.tsx'

export default function EmailLogin() {
  const profile = useProfile()
  const [, navigate] = useLocation()
  const [agreed, setAgreed] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  return <Screen className="justify-center">
    <CatWindow spec={{ eyes: 'open', mouth: 'smile', blush: true }} label="小伴等你回来" size={140} />
    <h1 className="mt-6 text-center font-display text-3xl text-lamp">找回你的小伴</h1>
    {profile.email ? <div className="mt-6 space-y-4 text-center">
      <p>当前已登录 {profile.email}</p>
      <Link href="/me" className="block text-lamp underline">回到我的账号</Link>
      <QuietButton disabled={busy} onClick={async () => {
        setBusy(true)
        try { await logout() } catch (e) { setError((e as Error).message); setBusy(false) }
      }}>退出并换一个账号</QuietButton>
      <FieldError message={error} />
    </div> : <div className="mt-6 space-y-5">
      <p className="text-sm leading-relaxed text-window/70">输入邮箱，用验证码登录。第一次来，也可以在这里注册。</p>
      <label className="flex gap-2.5 text-left text-[13px] leading-relaxed text-window/80">
        <input type="checkbox" className="mt-0.5 size-4 shrink-0 accent-[var(--color-lamp)]" checked={agreed}
          onChange={(e) => setAgreed(e.target.checked)} />
        <span>我同意<Link href="/terms" className="text-lamp underline">《用户协议》</Link>和
          <Link href="/privacy" className="text-lamp underline">《隐私政策》</Link>，继续登录即表示接受。</span>
      </label>
      <EmailBind mode="login" disabled={!agreed} termsVersion={TERMS_VERSION} beforeSend={async () => {
        if (getToken()) {
          await api('/api/consent', { method: 'PUT', body: { termsVersion: TERMS_VERSION } })
        }
      }} onDone={() => navigate('/', { replace: true })} />
    </div>}
    <Link href={profile.onboarded ? '/' : '/hello'} className="mt-6 text-center text-sm text-window/60 underline">返回网站，先体验一下</Link>
  </Screen>
}
