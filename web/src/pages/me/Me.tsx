import { useEffect, useState, type ReactNode } from 'react'
import { Link } from 'wouter'
import { api } from '../../api/client.ts'
import type { Profile } from '../../api/types.ts'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { FieldError, Screen } from '../../components/ui.tsx'
import { useProfile, useSession } from '../../lib/session.tsx'
import { AccountActions } from './AccountActions.tsx'
import { MeStats } from './MeStats.tsx'
import { AfterworkSummary } from './AfterworkSummary.tsx'
import { logout } from '../../lib/logout.ts'
import { GuestNotice } from '../../components/GuestNotice.tsx'

const personalityName = { toxic: '毒舌心软', gentle: '温柔治愈', chatty: '元气话痨', tsundere: '高冷傲娇' } as const

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mt-6">
      <h2 className="mb-2 text-sm font-bold text-window/60">{title}</h2>
      <div className="rounded-2xl bg-dusk-soft/35 p-4">{children}</div>
    </section>
  )
}

export function RowLink({ href, label, detail }: { href: string; label: string; detail?: string }) {
  return (
    <Link href={href} className="flex items-center justify-between py-1 text-window">
      <span>{label}</span>
      <span className="text-sm text-window/60">{detail ?? '去设置'}</span>
    </Link>
  )
}

export default function Me() {
  const profile = useProfile()
  const { setProfile, isLoggedIn, refresh } = useSession()
  const [loggingOut, setLoggingOut] = useState(false)
  useEffect(() => { void refresh().catch(() => undefined) }, [refresh])
  const [error, setError] = useState<string | null>(null)

  async function toggleRecall() {
    setError(null)
    try {
      setProfile(await api<Profile>('/api/subscription', { method: 'PUT', body: { enabled: !profile.recallOptIn } }))
    } catch (e) {
      setError((e as Error).message)
    }
  }

  return (
    <Screen>
      <div style={{ paddingBottom: TAB_BAR_SPACE }}>
        <h1 className="pt-2 font-display text-4xl text-lamp">我的</h1>

        <section className="mt-5 rounded-3xl border border-lamp/25 bg-dusk-soft/35 p-5" aria-label="账号和登录状态">
          <div className="flex items-center gap-4"><Link href="/me/account" aria-label="编辑头像" className="flex size-16 shrink-0 items-center justify-center overflow-hidden rounded-full bg-lamp/15 text-2xl text-lamp">
            {profile.avatarUrl ? <img src={profile.avatarUrl} alt="我的头像" className="size-full object-cover" /> : (isLoggedIn ? '☀' : '🐾')}
          </Link><div className="min-w-0"><h2 className="truncate text-xl font-bold">{profile.nickname ?? (profile.userId ? `${isLoggedIn ? '用户' : '游客'} #${profile.userId}` : '游客')}</h2>
            <p className="mt-1 text-sm text-lamp">{isLoggedIn ? '已登录 · 邮箱账号' : '未登录 · 游客模式'}</p><p className="mt-1 text-xs text-window/50">{profile.userId ? `用户 ID：${profile.userId}` : '浏览状态 · 尚未创建账号'}</p></div></div>
          {isLoggedIn ? <div className="mt-4 flex flex-wrap items-center justify-between gap-3 text-sm"><span className="text-window/60">{profile.email}</span><button disabled={loggingOut} className="text-window/70 underline" onClick={async () => {
            setLoggingOut(true); setError(null)
            try { await logout('/me') } catch (e) { setError((e as Error).message); setLoggingOut(false) }
          }}>{loggingOut ? '正在退出…' : '退出登录'}</button></div> : <div className="mt-4"><p className="mb-3 text-xs leading-relaxed text-window/60">登录后，换设备也能找回你的打卡、游戏和下班记录。</p>
            <Link href="/login" className="block w-full rounded-full bg-lamp px-5 py-3 text-center font-bold text-ink">登录 / 注册</Link></div>}
          <Link href="/me/account" className="mt-4 block text-center text-sm text-lamp underline">编辑昵称和头像</Link>
        </section>

        {!isLoggedIn && <GuestNotice className="mt-4" />}

        <Section title="统计">
          <MeStats companionName={profile.companionName} />
        </Section>

        <Section title="下班后的习惯">
          <AfterworkSummary timeZone={profile.timezone} />
        </Section>

        <Section title="小伴档案">
          <RowLink href="/me/profile" label={`${profile.companionName}，${personalityName[profile.personality]}`} detail="修改" />
          <p className="mt-1 text-sm text-window/60">
            {profile.workStart}–{profile.workEnd}{profile.cityName ? `，${profile.cityName}` : ''}{profile.payday ? `，每月 ${profile.payday} 号发薪` : ''}
          </p>
        </Section>

        <Section title="账号">
          {profile.email ? (
            <p className="text-window">
              已绑定邮箱 {profile.email}
            </p>
          ) : (
            <RowLink href="/login" label="邮箱登录 / 注册" detail="保存或找回账号" />
          )}
        </Section>

        <Section title="平安守护">
          <RowLink href="/me/guardian" label="没打卡时通知联系人" />
        </Section>

        <Section title="提醒邮件">
          <label className="flex items-center justify-between gap-4 text-window">
            <span>
              好几天没来的时候，发邮件叫我回来
              <span className="mt-0.5 block text-xs text-window/50">每次最多两封，邮件里随时可以退订</span>
            </span>
            <input type="checkbox" role="switch" className="size-5 shrink-0 accent-[var(--color-lamp)]"
              checked={profile.recallOptIn} disabled={!profile.email} onChange={toggleRecall} aria-label="回归提醒邮件" />
          </label>
          {!profile.email && <p className="mt-2 text-xs text-window/50">绑定邮箱后才能打开。</p>}
          <FieldError message={error} />
        </Section>

        {isLoggedIn && <Section title="数据管理"><AccountActions /></Section>}
      </div>
    </Screen>
  )
}
