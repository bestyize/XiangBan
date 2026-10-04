import type { Meta, Personality } from '../../api/types.ts'
import type { CatSpec } from '../../cat/catSvg.ts'
import { CityPicker } from '../../components/CityPicker.tsx'

export const STEPS = ['hello', 'name', 'personality', 'nickname', 'city', 'hours'] as const
export type Step = (typeof STEPS)[number]

export interface Draft {
  companionName: string
  personality: Personality
  nickname: string
  cityCode: string | null
  workStart: string
  workEnd: string
}

export const personalityFace: Record<Personality, CatSpec> = {
  toxic: { eyes: 'smug', mouth: 'smirk', blush: true },
  gentle: { eyes: 'closed', mouth: 'smile', blush: true },
  chatty: { eyes: 'happy', mouth: 'open', blush: true, props: ['sparkles'] },
  tsundere: { eyes: 'closed', mouth: 'flat', blush: true },
}

export function stepFace(step: Step, draft: Draft): CatSpec {
  switch (step) {
    case 'hello':
      return { eyes: 'smug', mouth: 'smirk' }
    case 'personality':
      return personalityFace[draft.personality]
    case 'nickname':
      return { eyes: 'open', mouth: 'smile', blush: true }
    case 'hours':
      return { eyes: 'worried', mouth: 'wavy' }
    default:
      return { eyes: 'open', mouth: 'cat', blush: true }
  }
}

export function stepLine(step: Step, draft: Draft): string {
  switch (step) {
    case 'hello':
      return '你就是那个总是很晚下班的人吧。'
    case 'name':
      return '先给我起个名字吧。不起的话，我就叫小伴。'
    case 'personality':
      return `${draft.companionName || '小伴'}该用什么性格陪你？`
    case 'nickname':
      return '我该怎么称呼你？'
    case 'city':
      return '你在哪个城市上班？我帮你盯着天气。'
    case 'hours':
      return '最后一个问题：你几点上班、几点下班？'
  }
}

const inputClass =
  'w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-lg text-window placeholder:text-window/40 focus:border-lamp focus:outline-none'

interface BodyProps {
  step: Step
  draft: Draft
  meta: Meta | null
  update: (patch: Partial<Draft>) => void
}

export function StepBody({ step, draft, meta, update }: BodyProps) {
  switch (step) {
    case 'hello':
      return null
    case 'name':
      return (
        <label className="block">
          <span className="mb-2 block text-sm text-window/70">名字，2–8 个字</span>
          <input className={inputClass} value={draft.companionName} maxLength={8} autoFocus
            onChange={(e) => update({ companionName: e.target.value })} />
        </label>
      )
    case 'nickname':
      return (
        <label className="block">
          <span className="mb-2 block text-sm text-window/70">至少 2 字；英文或英文加数字至少 4 字，不能是纯数字</span>
          <input className={inputClass} value={draft.nickname} maxLength={24} placeholder="比如：阿明、打工人（可跳过）" autoFocus
            onChange={(e) => update({ nickname: e.target.value })} />
        </label>
      )
    case 'personality':
      return (
        <div className="grid grid-cols-2 gap-3" role="radiogroup" aria-label="小伴的性格">
          {(meta?.personalities ?? []).map((p) => {
            const selected = draft.personality === p.code
            return (
              <button key={p.code} type="button" role="radio" aria-checked={selected}
                onClick={() => update({ personality: p.code })}
                className={`flex flex-col items-start rounded-2xl border-2 p-3 text-left transition ${selected ? 'border-lamp bg-lamp/15' : 'border-dusk-soft hover:border-window/50'}`}>
                <span className="block font-bold text-window">{p.name}</span>
                <span className="mt-0.5 block text-xs text-window/60">{p.desc}</span>
                <span className="mt-2 block text-[13px] leading-snug text-window/85">“{p.sample}”</span>
              </button>
            )
          })}
        </div>
      )
    case 'city':
      return meta ? (
        <div>
          <CityPicker cities={meta.cities} value={draft.cityCode} onChange={(cityCode) => update({ cityCode })} />
          <p className="mt-2 text-xs text-window/50">只用来查天气和限行，不会申请定位权限。</p>
        </div>
      ) : null
    case 'hours':
      return (
        <div>
          <div className="grid grid-cols-2 gap-3">
            {(['workStart', 'workEnd'] as const).map((key) => (
              <label key={key} className="block">
                <span className="mb-2 block text-sm text-window/70">{key === 'workStart' ? '上班' : '正常下班'}</span>
                <input type="time" className={inputClass} value={draft[key]} onChange={(e) => update({ [key]: e.target.value })} />
              </label>
            ))}
          </div>
          <p className="mt-2 text-xs text-window/50">用来算加班和在岗时长。不填就按 9 点上班、6 点下班算。</p>
        </div>
      )
  }
}
