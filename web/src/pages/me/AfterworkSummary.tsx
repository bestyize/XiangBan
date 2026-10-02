import { useEffect, useState } from 'react'
import {
  clearAfterworkData,
  getAfterworkActivityLabel,
  getWeeklyAfterworkSummary,
  loadAfterworkData,
  setPreferredAfterworkActivity,
  type AfterworkData,
} from '../../lib/afterwork.ts'

export function AfterworkSummary({ timeZone }: { timeZone: string }) {
  const [data, setData] = useState<AfterworkData>(() => loadAfterworkData())
  const [confirmClear, setConfirmClear] = useState(false)
  const summary = getWeeklyAfterworkSummary(data, timeZone)

  useEffect(() => {
    setData(loadAfterworkData())
  }, [])

  function clearRecords() {
    setData(clearAfterworkData())
    setConfirmClear(false)
  }

  function removePreferredActivity() {
    setData(setPreferredAfterworkActivity(null))
  }

  return (
    <div>
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
          <button type="button" className="shrink-0 text-xs text-window/55 underline" onClick={removePreferredActivity}>
            取消记忆
          </button>
        </div>
      )}

      <p className="mt-3 text-xs text-window/45">这些记录只在当前浏览器保存，不会上传；清除本站浏览器数据也会删除。</p>

      {data.entries.length > 0 && (
        confirmClear ? (
          <div className="mt-3 rounded-xl border border-window/10 p-3">
            <p className="text-sm text-window/75">要清除这台设备上的下班安排和常做习惯吗？</p>
            <div className="mt-2 flex gap-4">
              <button type="button" className="text-sm text-lamp" onClick={clearRecords}>确认清除</button>
              <button type="button" className="text-sm text-window/55" onClick={() => setConfirmClear(false)}>取消</button>
            </div>
          </div>
        ) : (
          <button type="button" className="mt-3 text-xs text-window/50 underline" onClick={() => setConfirmClear(true)}>
            清除本机记录
          </button>
        )
      )}
    </div>
  )
}
