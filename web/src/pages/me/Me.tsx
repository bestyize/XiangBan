import { useEffect, useState } from 'react'
import { Link, useLocation } from 'wouter'
import { api } from '../../api/client.ts'
import type { Guardian, Profile, Stats } from '../../api/types.ts'
import { ACTIVITY_PATH } from '../../lib/activity.ts'
import { Robot } from '../../components/Robot.tsx'
import { LinkRow, ListGroup, SwitchRow } from '../../components/List.tsx'
import { PageHeader } from '../../components/PageHeader.tsx'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { FieldError, Screen } from '../../components/ui.tsx'
import { getWeeklyAfterworkSummary } from '../../lib/afterwork.ts'
import { useLoginModal } from '../../lib/loginModal.ts'
import { useProfile, useSession } from '../../lib/session.tsx'
import { useAfterwork } from '../../lib/useAfterwork.ts'
import { MeStats } from './MeStats.tsx'

const personalityName = { toxic: '毒舌心软', gentle: '温柔治愈', chatty: '元气话痨', tsundere: '高冷傲娇' } as const

/** null: not loaded yet; undefined: not set up. */
function useGuardian(enabled: boolean, active: boolean) {
  const [guardian, setGuardian] = useState<Guardian | undefined | null>(null)
  useEffect(() => {
    if (!enabled || !active) return
    api<Guardian | undefined>('/api/guardian').then(setGuardian).catch(() => setGuardian(null))
  }, [enabled, active])
  if (!enabled) return '未开启'
  if (guardian === null) return ''
  if (!guardian) return '未开启'
  return guardian.paused ? '已暂停' : '已开启'
}

/** The 下班啦 row's detail: current streak, otherwise how many days in total. */
function useCheckinDetail(active: boolean) {
  const [stats, setStats] = useState<Stats | null>(null)
  useEffect(() => {
    if (active) api<Stats>('/api/stats').then(setStats).catch(() => undefined)
  }, [active])
  if (!stats) return undefined
  if (stats.currentStreak > 0) return `连续 ${stats.currentStreak} 天`
  return stats.totalDays > 0 ? `打过 ${stats.totalDays} 天` : '去打卡'
}

/** 我的: who I am, a few numbers, and a short list of things to set. Account and data controls live in 设置. */
export default function Me() {
  const profile = useProfile()
  const { setProfile, isLoggedIn, refresh } = useSession()
  const { openLogin } = useLoginModal()
  const [location] = useLocation()
  // The tab stays mounted, so refresh whenever it is shown again.
  const active = location === '/me'
  const [error, setError] = useState<string | null>(null)
  const guardian = useGuardian(Boolean(profile.email), active)
  const checkinDetail = useCheckinDetail(active)
  const { data: habits } = useAfterwork()
  const weekHabits = getWeeklyAfterworkSummary(habits, profile.timezone).count

  useEffect(() => {
    if (active) void refresh().catch(() => undefined)
  }, [active, refresh])

  async function toggleRecall(enabled: boolean) {
    setError(null)
    try {
      setProfile(await api<Profile>('/api/subscription', { method: 'PUT', body: { enabled } }))
    } catch (e) {
      setError((e as Error).message)
    }
  }

  const name = profile.nickname ?? (isLoggedIn ? '相伴用户' : '游客')

  return (
    <Screen>
      <div style={{ paddingBottom: TAB_BAR_SPACE }}>
        <PageHeader title="我的">
          <Link href="/me/settings" aria-label="设置" className="grid size-11 place-items-center rounded-full text-window/80">
            <svg viewBox="0 0 24 24" className="size-6" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z" />
              <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" />
            </svg>
          </Link>
        </PageHeader>

        <section aria-label="账号" className="mt-2 rounded-3xl bg-dusk-soft/35 p-4">
          <Link href="/me/account" className="flex items-center gap-4 rounded-2xl transition active:opacity-70">
            <span className="flex size-16 shrink-0 items-center justify-center overflow-hidden rounded-full bg-lamp/15 text-2xl text-lamp">
              {profile.avatarUrl
                ? <img src={profile.avatarUrl} alt="" className="size-full object-cover" />
                : <Robot spec="idle" alive={false} label="默认头像" size={60} className="mt-3" />}
            </span>
            <span className="min-w-0 flex-1">
              <span className="block truncate text-xl font-bold text-window">{name}</span>
              <span className="mt-0.5 block truncate text-sm text-window/55">{isLoggedIn ? profile.email : '未登录'}</span>
            </span>
            <svg viewBox="0 0 24 24" className="size-5 shrink-0 text-window/40" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 5l7 7-7 7" /></svg>
          </Link>
          {!isLoggedIn && (
            <button type="button" onClick={openLogin} className="mt-4 block w-full rounded-full bg-lamp px-5 py-3 text-center font-bold text-ink transition active:scale-[.98]">登录 / 注册</button>
          )}
        </section>

        <MeStats companionName={profile.companionName} createdAt={profile.createdAt} active={active} />

        <ListGroup>
          <LinkRow href="/me/profile" label="小伴档案" detail={`${profile.companionName}，${personalityName[profile.personality]}`} />
          <LinkRow href={ACTIVITY_PATH} label="下班打卡" detail={checkinDetail} />
          <LinkRow href="/me/habits" label="下班习惯" detail={weekHabits > 0 ? `本周 ${weekHabits} 次` : undefined} />
          <LinkRow href="/me/guardian" label="平安守护" detail={guardian} />
        </ListGroup>

        <ListGroup>
          <SwitchRow
            label="回归提醒邮件"
            hint={profile.email ? '好几天没来时，发邮件叫你回来' : '绑定邮箱后可用'}
            checked={profile.recallOptIn}
            disabled={!profile.email}
            onChange={(next) => void toggleRecall(next)}
          />
        </ListGroup>
        <div className="mt-3"><FieldError message={error} /></div>
      </div>
    </Screen>
  )
}
