import type { Tip } from '../../api/types.ts'
import {
  AFTERWORK_ACTIVITIES,
  getAfterworkActivityLabel,
  getRecentAfterworkActivityCount,
  type AfterworkActivityId,
} from '../../lib/afterwork.ts'
import { useAfterwork } from '../../lib/useAfterwork.ts'
import { FieldError } from '../../components/ui.tsx'

interface AfterworkChoiceProps {
  workDay: string
  localTime: string
  tips: Tip[] | null
}

function tonightSuggestion(localTime: string, tips: Tip[] | null): string {
  if (tips?.some((tip) => tip.kind === 'rain')) return '今晚可能有雨，散步也可以换成在家放松。'
  if (tips?.some((tip) => tip.kind === 'cold')) return '天气有点冷，窝在家里休息也很好。'
  if (localTime >= '21:00') return '辛苦啦，忙到这么晚，休息也是今晚的安排。'
  return '不用把晚上排满，选个让自己舒服的安排就好。'
}

export function AfterworkChoice({ workDay, localTime, tips }: AfterworkChoiceProps) {
  const { data, choose: saveChoice, remember: savePreference, dismiss: saveDismissal, busy, loading, error } = useAfterwork()

  const selected = data.entries.find((entry) => entry.workDay === workDay)?.activityId ?? null
  const selectedCount = selected ? getRecentAfterworkActivityCount(data, selected, workDay) : 0
  const dismissedCount = selected ? data.promptDismissedAt[selected] ?? 0 : 0
  const showHabitPrompt = Boolean(
    selected &&
    selected !== 'no_plan' &&
    selectedCount >= 3 &&
    data.preferredActivityId !== selected &&
    selectedCount > dismissedCount,
  )

  function choose(activityId: AfterworkActivityId) {
    void saveChoice(workDay, activityId)
  }

  function remember(activityId: AfterworkActivityId) {
    void savePreference(activityId)
  }

  function dismiss(activityId: AfterworkActivityId) {
    void saveDismissal(activityId, selectedCount)
  }

  return (
    <section className="mt-5 rounded-2xl border border-lamp/15 bg-dusk-soft/30 p-4" aria-label="今晚留给自己的安排">
      <div className="flex items-baseline justify-between gap-3">
        <h2 className="font-display text-xl text-lamp">今晚留给自己</h2>
        {data.preferredActivityId && (
          <span className="text-xs text-window/55">
            常做：{getAfterworkActivityLabel(data.preferredActivityId)}
          </span>
        )}
      </div>
      <p className="mt-1 text-sm text-window/70">{tonightSuggestion(localTime, tips)}</p>
      <p className="mt-2 text-xs text-window/45">{loading ? '正在读取当前账号的安排…' : busy ? '正在保存…' : '安排会保存在当前用户 ID 下，登录后可跨设备找回。'}</p><FieldError message={error} />

      {data.preferredActivityId && !selected && (
        <button
          type="button"
          disabled={busy || loading}
          className="mt-3 rounded-full border border-lamp/40 px-3 py-1.5 text-sm text-lamp"
          onClick={() => choose(data.preferredActivityId!)}
        >
          今晚也选「{getAfterworkActivityLabel(data.preferredActivityId)}」
        </button>
      )}

      <div className="mt-3 grid grid-cols-2 gap-2">
        {AFTERWORK_ACTIVITIES.map((activity) => {
          const active = selected === activity.id
          return (
            <button
              key={activity.id}
              type="button"
              disabled={busy || loading}
              aria-pressed={active}
              onClick={() => choose(activity.id)}
              className={[
                'flex min-h-12 items-center gap-2 rounded-xl border px-3 py-2 text-left text-sm transition-colors',
                active ? 'border-lamp/70 bg-lamp/10 text-lamp' : 'border-window/10 bg-night/15 text-window/85',
              ].join(' ')}
            >
              <span aria-hidden="true">{activity.emoji}</span>
              <span>{activity.label}</span>
              {active && <span className="ml-auto text-xs" aria-hidden="true">✓</span>}
            </button>
          )
        })}
      </div>

      {selected && (
        <div className="mt-3 rounded-xl bg-night/20 px-3 py-2.5">
          <p className="text-sm text-window">
            {selected === 'no_plan'
              ? '记下啦：今天不安排。'
              : '记下啦：今晚「' + getAfterworkActivityLabel(selected) + '」。'}
          </p>
          {selected !== 'no_plan' && data.preferredActivityId === selected && (
            <p className="mt-1 text-xs text-lamp">这是你记下的常做安排。</p>
          )}
        </div>
      )}

      {showHabitPrompt && selected && selected !== 'no_plan' && (
        <div className="mt-3 rounded-xl border border-lamp/20 p-3">
          <p className="text-sm text-window/85">
            近两周你有 {selectedCount} 天下班后选了「{getAfterworkActivityLabel(selected)}」。要把它记成常做安排吗？
          </p>
          <div className="mt-2 flex gap-3">
            <button type="button" disabled={busy || loading} className="text-sm text-lamp" onClick={() => remember(selected)}>记下来</button>
            <button type="button" disabled={busy || loading} className="text-sm text-window/55" onClick={() => dismiss(selected)}>下次再说</button>
          </div>
        </div>
      )}
    </section>
  )
}
