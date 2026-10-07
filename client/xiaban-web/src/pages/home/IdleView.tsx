import { useEffect, useState } from 'react'
import type { Greeting } from '../../api/types.ts'
import { PunchButton } from '../../components/PunchButton.tsx'
import { Bubble, RobotStage, FieldError } from '../../components/ui.tsx'
import { moodFace } from '../../robot/moods.ts'
import { useProfile } from '../../lib/session.tsx'
import { formatDuration, minutesPastWorkEnd } from '../../lib/time.ts'

interface IdleViewProps {
  greeting: Greeting | null
  busy: boolean
  error: string | null
  onCheckIn: () => void
}

function useNow(intervalMs: number) {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), intervalMs)
    return () => clearInterval(id)
  }, [intervalMs])
  return now
}

export function IdleView({ greeting, busy, error, onCheckIn }: IdleViewProps) {
  const profile = useProfile()
  const past = minutesPastWorkEnd(useNow(30_000), profile.workEnd)
  const status =
    past < 0
      ? `离 ${profile.workEnd} 下班还有 ${formatDuration(-past)}`
      : past < 1
        ? `现在就是 ${profile.workEnd}，正点`
        : `${profile.workEnd} 就该下班了，已经过了 ${formatDuration(past)}`

  return (
    <div className="flex flex-1 flex-col">
      <RobotStage className="mt-6" spec={moodFace[greeting?.mood ?? 'idle']} label={`${profile.companionName}在家等你`} size={200} interactive autoSleep />
      <Bubble className="mt-5">{greeting?.text ?? '今天也辛苦了。下班了就按一下。'}</Bubble>
      <div className="mt-auto pt-10 pb-4">
        <PunchButton onPress={onCheckIn} busy={busy} />
        <p className="mt-8 text-center text-sm text-window/65">{status}</p>
        <FieldError message={error} />
      </div>
    </div>
  )
}
