import { useEffect, useRef, useState } from 'react'
import type { Unlock } from '../api/types.ts'
import { dexArt, fallbackArt } from '../robot/moods.ts'
import { Robot } from './Robot.tsx'
import { PrimaryButton } from './ui.tsx'

/** Plays once per newly unlocked expression after a check-in (PRD 5.8). */
export function UnlockOverlay({ unlocks, onDone }: { unlocks: Unlock[]; onDone: () => void }) {
  const [index, setIndex] = useState(0)
  const button = useRef<HTMLButtonElement>(null)
  const current = unlocks[index]

  useEffect(() => {
    button.current?.focus()
  }, [index])

  if (!current) return null
  const art = dexArt[current.id] ?? fallbackArt
  const last = index === unlocks.length - 1

  return (
    <div role="dialog" aria-modal="true" aria-labelledby="unlock-title" className="fixed inset-0 z-50 grid place-items-center bg-dusk-deep/85 px-8 backdrop-blur-sm">
      <div className="w-full max-w-xs text-center" key={current.id}>
        <p className="text-sm text-window/70">{unlocks.length > 1 ? `解锁了 ${unlocks.length} 个新表情（${index + 1}/${unlocks.length}）` : '解锁了新表情'}</p>
        <div className="relative mx-auto mt-5 size-52">
          <span className="absolute inset-0 animate-burst rounded-full border-4 border-lamp" aria-hidden="true" />
          <div className="grid size-full animate-pop place-items-center rounded-full" style={{ background: art.tint }}>
            <Robot spec={art.spec} label={current.name} size={176} />
          </div>
        </div>
        <h2 id="unlock-title" className="mt-6 text-2xl font-bold text-window">{current.name}</h2>
        <p className="mt-1 text-sm text-window/65">{current.hint}</p>
        <PrimaryButton ref={button} className="mt-8" onClick={() => (last ? onDone() : setIndex(index + 1))}>
          {last ? '收下了' : '下一个'}
        </PrimaryButton>
      </div>
    </div>
  )
}
