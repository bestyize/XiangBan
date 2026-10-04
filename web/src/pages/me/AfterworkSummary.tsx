import { useState } from 'react'
import {
  getAfterworkActivityLabel,
  getWeeklyAfterworkSummary,
} from '../../lib/afterwork.ts'
import { useAfterwork } from '../../lib/useAfterwork.ts'
import { FieldError } from '../../components/ui.tsx'

export function AfterworkSummary({ timeZone }: { timeZone: string }) {
  const { data, legacy, busy, loading, error, clear, remember, importLegacy } = useAfterwork()
  const [confirmClear, setConfirmClear] = useState(false)
  const summary = getWeeklyAfterworkSummary(data, timeZone)

  async function clearRecords() {
    if (await clear()) setConfirmClear(false)
  }

  function removePreferredActivity() {
    void remember(null)
  }

  return (
    <div>
      <FieldError message={error} />
      {loading && <p className="mb-3 text-xs text-window/55">正在读取账号记录…</p>}
      {summary.count > 0 ? (
        <div>
          <p className="text-window">本周，你为自己选了 {summary.count} 次下班安排。</p>
          {summary.topActivityId && (
            <p className="mt-1 text-sm text-window/70">
              最常选：{getAfterworkActivityLabel(summary.topActivityId)}
              {summary.topActivityCount > 1 && '，共 ' + summary.topActivityCount + ' 次'}
            </p>
          )}
        </div>
      ) : (
        <p className="text-sm text-window/65">选过的下班安排会在这里做一周小回顾，不打卡也不会影响统计。</p>
      )}

      {data.preferredActivityId && (
        <div className="mt-3 flex items-center justify-between gap-3 rounded-xl bg-night/20 px-3 py-2">
          <p className="text-sm text-window/85">
            常做安排：{getAfterworkActivityLabel(data.preferredActivityId)}
          </p>
          <button type="button" disabled={busy || loading} className="shrink-0 text-xs text-window/55 underline" onClick={removePreferredActivity}>
            取消记忆
          </button>
        </div>
      )}

      {(legacy.entries.length > 0 || legacy.preferredActivityId) && <div className="mt-3 rounded-xl border border-lamp/25 p-3"><p className="text-xs text-window/70">这台设备上还有旧版的安排记录，可以导入到当前账号，已有的同日记录会保留。</p>
        <button type="button" disabled={busy || loading} className="mt-2 text-sm text-lamp underline" onClick={() => void importLegacy()}>导入本机旧记录到当前账号</button></div>}

      {data.entries.length > 0 && (
        confirmClear ? (
          <div className="mt-3 rounded-xl border border-window/10 p-3">
            <p className="text-sm text-window/75">要清除这些下班安排和常做习惯吗？</p>
            <div className="mt-2 flex gap-4">
              <button type="button" disabled={busy || loading} className="text-sm text-lamp" onClick={clearRecords}>确认清除</button>
              <button type="button" className="text-sm text-window/55" onClick={() => setConfirmClear(false)}>取消</button>
            </div>
          </div>
        ) : (
          <button type="button" disabled={busy || loading} className="mt-3 text-xs text-window/50 underline" onClick={() => setConfirmClear(true)}>
            清除记录
          </button>
        )
      )}
    </div>
  )
}
