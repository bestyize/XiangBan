import { TAB_BAR_SPACE } from '../components/TabBar.tsx'
import { UnlockOverlay } from '../components/UnlockOverlay.tsx'
import { PrimaryButton, Screen } from '../components/ui.tsx'
import { Link } from 'wouter'
import { IdleView } from './home/IdleView.tsx'
import { ResultView } from './home/ResultView.tsx'
import { useToday } from './home/useToday.ts'
import { PageHeader } from '../components/PageHeader.tsx'
import { AccountBadge } from '../components/AccountBadge.tsx'

export default function Home({ active = true }: { active?: boolean }) {
  const { status, today, tips, unlocked, dismissUnlocked, error, busy, fresh, checkIn, undo, reload } = useToday(active)

  return (
    <Screen>
      <div className="flex flex-1 flex-col" style={{ paddingBottom: TAB_BAR_SPACE }}>
        <PageHeader title="下班了么">
            <Link href="/dex" className="flex min-h-11 shrink-0 items-center gap-1.5 rounded-full border border-lamp/35 bg-lamp/10 px-2 text-[11px] text-lamp min-[400px]:px-3 min-[400px]:text-xs">
              <svg aria-hidden="true" viewBox="0 0 24 24" className="size-4" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M12 5v15M12 5C9 3 5 3 3 4v15c2-1 6-1 9 1 3-2 7-2 9-1V4c-2-1-6-1-9 1Z" /></svg>
              表情图鉴
            </Link>
            <AccountBadge compact />
        </PageHeader>

        {status === 'loading' && <p className="mt-24 text-center text-window/60">小伴在翻今天的记录…</p>}
        {status === 'error' && (
          <div className="mt-24 text-center">
            <p className="text-window/80">{error}</p>
            <PrimaryButton className="mt-6 max-w-60" onClick={reload}>再试一次</PrimaryButton>
          </div>
        )}
        {status === 'ready' && today && (today.checkin ? (
          <ResultView checkin={today.checkin} tips={tips} fresh={fresh} busy={busy} error={error} onUndo={undo} />
        ) : (
          <IdleView greeting={today.greeting} busy={busy} error={error} onCheckIn={checkIn} />
        ))}
      </div>
      {unlocked.length > 0 && <UnlockOverlay unlocks={unlocked} onDone={dismissUnlocked} />}
    </Screen>
  )
}
