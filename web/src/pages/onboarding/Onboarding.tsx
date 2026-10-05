import { useState } from 'react'
import { useLocation } from 'wouter'
import type { ProfileUpdate } from '../../api/types.ts'
import { track } from '../../lib/track.ts'
import { Bubble, CatWindow, FieldError, PrimaryButton, QuietButton, Screen } from '../../components/ui.tsx'
import { useLoginModal } from '../../lib/loginModal.ts'
import { useMeta } from '../../lib/meta.ts'
import { useProfile, useSession } from '../../lib/session.tsx'
import { STEPS, StepBody, stepFace, stepLine, type Draft, type Step } from './steps.tsx'

function changesFor(step: Step, draft: Draft): ProfileUpdate {
  switch (step) {
    case 'name':
      return { companionName: draft.companionName.trim() }
    case 'personality':
      return { personality: draft.personality }
    case 'nickname':
      return { nickname: draft.nickname.trim() }
    case 'city':
      return draft.cityCode ? { cityCode: draft.cityCode } : {}
    case 'hours':
      return { workStart: draft.workStart, workEnd: draft.workEnd }
    case 'hello':
      return {}
  }
}

export default function Onboarding() {
  const profile = useProfile()
  const { updateProfile } = useSession()
  const { openLogin } = useLoginModal()
  const meta = useMeta()
  const [, navigate] = useLocation()
  const [index, setIndex] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [draft, setDraft] = useState<Draft>(() => ({
    companionName: profile.companionName,
    personality: profile.personality,
    nickname: profile.nickname ?? '',
    cityCode: profile.cityCode,
    workStart: profile.workStart,
    workEnd: profile.workEnd,
  }))

  const step = STEPS[index]
  const isLast = index === STEPS.length - 1

  async function advance(changes: ProfileUpdate, finish: boolean, skipped: boolean) {
    setSaving(true)
    setError(null)
    try {
      const patch = finish ? { ...changes, onboarded: true } : changes
      if (Object.keys(patch).length > 0) await updateProfile(patch)
      track('onboarding_step', { step, action: skipped ? 'skip' : 'next' })
      if (finish) navigate('/', { replace: true })
      else setIndex((i) => i + 1)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  const nameInvalid = step === 'name' && [...draft.companionName.trim()].length < 2
  const blocked = saving || nameInvalid

  return (
    <Screen>
      <div className="flex h-10 items-center justify-between">
        <ol className="flex gap-1.5" aria-label={`第 ${index + 1} 步，共 ${STEPS.length} 步`}>
          {STEPS.map((s, i) => (
            <li key={s} className={`h-1.5 rounded-full transition-all ${i === index ? 'w-6 bg-lamp' : i < index ? 'w-1.5 bg-lamp/70' : 'w-1.5 bg-window/25'}`} />
          ))}
        </ol>
        <QuietButton disabled={saving} onClick={() => advance({}, step === 'hello' || isLast, true)}>
          {step === 'hello' ? '全部跳过' : '跳过'}
        </QuietButton>
      </div>

      <section className="mt-6 flex flex-1 flex-col" aria-live="polite">
        <CatWindow spec={stepFace(step, draft)} label={`${draft.companionName || '小伴'}在窗台上看着你`} size={step === 'city' || step === 'personality' ? 150 : 200} />
        <Bubble className="mt-5">{stepLine(step, draft)}</Bubble>
        <div className="mt-6">
          <StepBody step={step} draft={draft} meta={meta} update={(patch) => setDraft((d) => ({ ...d, ...patch }))} />
          <FieldError message={error} />
        </div>
      </section>

      <div className="mt-6">
        <PrimaryButton disabled={blocked} onClick={() => advance(changesFor(step, draft), isLast, false)}>
          {step === 'hello' ? '是我' : isLast ? '开始陪我下班' : '下一步'}
        </PrimaryButton>
        {step === 'hello' && <button type="button" onClick={openLogin} className="mt-5 block w-full text-center text-sm text-window/70 underline">已有记录？邮箱登录 / 注册</button>}
      </div>
    </Screen>
  )
}
