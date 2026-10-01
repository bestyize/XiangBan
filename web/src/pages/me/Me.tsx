import { useState, type ReactNode } from 'react'
import { Link } from 'wouter'
import { api } from '../../api/client.ts'
import type { Profile } from '../../api/types.ts'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { FieldError, Screen } from '../../components/ui.tsx'
import { useProfile, useSession } from '../../lib/session.tsx'
import { AccountActions } from './AccountActions.tsx'
import { EmailBind } from './EmailBind.tsx'
import { MeStats } from './MeStats.tsx'

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
  const { setProfile } = useSession()
  const [merged, setMerged] = useState(false)
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

        <Section title="统计">
          <MeStats companionName={profile.companionName} />
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
              {merged && <span className="mt-1 block text-sm text-lamp">找回了你的账号，这台设备上的记录也合并进去了。</span>}
            </p>
          ) : (
            <EmailBind onDone={setMerged} />
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

        <Section title="隐私">
          <RowLink href="/privacy" label="隐私政策" detail="查看" />
          <RowLink href="/terms" label="用户协议" detail="查看" />
          <div className="mt-2 border-t border-window/10 pt-2">
            <AccountActions />
          </div>
        </Section>
      </div>
    </Screen>
  )
}
