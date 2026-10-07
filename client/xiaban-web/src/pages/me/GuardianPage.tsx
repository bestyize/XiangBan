import { useEffect, useState } from 'react'
import { api } from '../../api/client.ts'
import type { Guardian, GuardianUpdate } from '../../api/types.ts'
import { SubPage } from '../../components/SubPage.tsx'
import { FieldError, QuietButton } from '../../components/ui.tsx'
import { useProfile } from '../../lib/session.tsx'
import { GuardianForm } from './GuardianForm.tsx'

function when(ms: number) {
  const d = new Date(ms)
  return `${d.getMonth() + 1}月${d.getDate()}日 ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}

export default function GuardianPage() {
  const profile = useProfile()
  const [guardian, setGuardian] = useState<Guardian | null | undefined>(undefined)
  const [editing, setEditing] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api<Guardian | undefined>('/api/guardian').then((g) => setGuardian(g ?? null)).catch((e: Error) => setError(e.message))
  }, [])

  async function run(action: () => Promise<Guardian | null>) {
    setBusy(true)
    setError(null)
    try {
      setGuardian(await action())
      setEditing(false)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusy(false)
    }
  }

  const save = (value: GuardianUpdate) => run(() => api<Guardian>('/api/guardian', { method: 'PUT', body: value }))
  const pause = (paused: boolean) => run(() => api<Guardian>('/api/guardian/pause', { method: 'PUT', body: { paused } }))
  const remove = () => run(async () => (await api('/api/guardian', { method: 'DELETE' }), null))

  return (
    <SubPage title="平安守护" back="/me">
      <p className="mt-2 text-sm leading-relaxed text-window/70">一个人住也有人惦记：连续几个工作日没打卡，小伴会先提醒你，再发邮件告诉你指定的联系人。</p>
      <p role="note" className="mt-3 rounded-2xl border border-blush/60 px-4 py-3 text-sm text-window">
        平安守护不是急救服务。紧急情况请拨打 120 或 110。
      </p>

      <section className="mt-6 flex-1">
        {guardian === undefined && !error && <p className="text-window/60">加载中…</p>}
        {!profile.email && guardian !== undefined && (
          <p className="text-window/80">先在「我的」里绑定你自己的邮箱，快到时间时小伴要先提醒你。</p>
        )}
        {profile.email && (guardian === null || editing) && (
          <GuardianForm initial={guardian ?? {}} busy={busy} error={error} onSubmit={save} />
        )}
        {guardian && !editing && (
          <div className="space-y-4">
            <div className="rounded-2xl bg-dusk-soft/40 p-4 text-[15px] leading-relaxed text-window/90">
              <p>联系人：{guardian.contactName}（{guardian.contactEmailMasked}）</p>
              <p>连续 {guardian.thresholdDays} 个工作日没打卡就通知</p>
              {guardian.paused ? (
                <p className="mt-2 text-lamp">已暂停。恢复后从那天重新开始算。</p>
              ) : guardian.nextNotifyAt ? (
                <p className="mt-2 text-lamp">
                  {when(guardian.nextNotifyAt)} 前还没打卡，就发邮件告诉{guardian.contactName}
                  {guardian.nextRemindAt ? `（${when(guardian.nextRemindAt).slice(-5)} 先提醒你）` : ''}
                </p>
              ) : (
                <p className="mt-2 text-lamp">已经通知过{guardian.contactName}。下次打卡后重新开始算。</p>
              )}
            </div>
            <FieldError message={error} />
            <div className="flex flex-wrap gap-2">
              <QuietButton disabled={busy} onClick={() => pause(!guardian.paused)}>{guardian.paused ? '恢复守护' : '休假中，先暂停'}</QuietButton>
              <QuietButton disabled={busy} onClick={() => setEditing(true)}>修改设置</QuietButton>
              <QuietButton disabled={busy} className="text-blush!" onClick={remove}>关闭平安守护</QuietButton>
            </div>
          </div>
        )}
      </section>
    </SubPage>
  )
}
