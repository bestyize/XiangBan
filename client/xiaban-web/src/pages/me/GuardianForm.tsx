import { useState } from 'react'
import type { GuardianUpdate } from '../../api/types.ts'
import { FieldError, PrimaryButton } from '../../components/ui.tsx'

const input =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'

interface GuardianFormProps {
  initial: Partial<GuardianUpdate>
  busy: boolean
  error: string | null
  onSubmit: (value: GuardianUpdate) => void
}

export function GuardianForm({ initial, busy, error, onSubmit }: GuardianFormProps) {
  const [value, setValue] = useState<GuardianUpdate>({
    contactName: initial.contactName ?? '',
    contactEmail: '',
    selfName: initial.selfName ?? '',
    thresholdDays: initial.thresholdDays ?? 2,
    consent: false,
    informedContact: false,
  })
  const set = (patch: Partial<GuardianUpdate>) => setValue((v) => ({ ...v, ...patch }))
  const ready = value.contactName.trim() && value.contactEmail.includes('@') && value.selfName.trim() && value.consent && value.informedContact

  return (
    <form className="space-y-4" onSubmit={(e) => { e.preventDefault(); if (ready) onSubmit(value) }}>
      <label className="block">
        <span className="mb-1.5 block text-sm text-window/70">联系人的称呼</span>
        <input className={input} value={value.contactName} maxLength={8} placeholder="比如：妈妈、室友阿杰" onChange={(e) => set({ contactName: e.target.value })} />
      </label>
      <label className="block">
        <span className="mb-1.5 block text-sm text-window/70">联系人的邮箱</span>
        <input className={input} type="email" inputMode="email" value={value.contactEmail} onChange={(e) => set({ contactEmail: e.target.value })} />
      </label>
      <label className="block">
        <span className="mb-1.5 block text-sm text-window/70">联系人平时怎么叫你</span>
        <input className={input} value={value.selfName} maxLength={8} placeholder="写在给联系人的邮件里" onChange={(e) => set({ selfName: e.target.value })} />
      </label>

      <fieldset>
        <legend className="mb-1.5 text-sm text-window/70">连续几个工作日没打卡就通知</legend>
        <div className="grid grid-cols-3 gap-2">
          {[1, 2, 3].map((n) => (
            <button key={n} type="button" aria-pressed={value.thresholdDays === n} onClick={() => set({ thresholdDays: n })}
              className={`rounded-2xl border-2 py-2.5 ${value.thresholdDays === n ? 'border-lamp bg-action font-bold text-white' : 'border-dusk-soft text-window/80'}`}>
              {n} 天
            </button>
          ))}
        </div>
        <p className="mt-1.5 text-xs text-window/50">周末和法定节假日不算。到期那天晚上先提醒你，2 小时后还没打卡才通知联系人。</p>
      </fieldset>

      <label className="flex gap-3 text-sm text-window/85">
        <input type="checkbox" className="mt-0.5 size-4 shrink-0 accent-[var(--color-lamp)]" checked={value.consent} onChange={(e) => set({ consent: e.target.checked })} />
        我知道这不是急救服务，邮件可能延迟或被拦截；同意下班啦在我没打卡时给联系人发邮件。
      </label>
      <label className="flex gap-3 text-sm text-window/85">
        <input type="checkbox" className="mt-0.5 size-4 shrink-0 accent-[var(--color-lamp)]" checked={value.informedContact} onChange={(e) => set({ informedContact: e.target.checked })} />
        我已经告诉过联系人本人，ta 同意收到这样的邮件。
      </label>

      <FieldError message={error} />
      <PrimaryButton type="submit" disabled={busy || !ready}>开启平安守护</PrimaryButton>
    </form>
  )
}
