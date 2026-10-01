import { useEffect, useState } from 'react'
import { api } from '../api/client.ts'
import type { Unlock, UnlocksResponse } from '../api/types.ts'
import { expressions, fallbackExpression } from '../cat/expressions.ts'
import { Cat } from '../components/Cat.tsx'
import { TAB_BAR_SPACE } from '../components/TabBar.tsx'
import { PrimaryButton, Screen } from '../components/ui.tsx'

function Tile({ item }: { item: Unlock }) {
  const art = expressions[item.id] ?? fallbackExpression
  const unlocked = item.unlockedAt !== null
  return (
    <li className="flex flex-col items-center text-center">
      <div
        className="grid aspect-square w-full place-items-center rounded-3xl"
        style={{ background: unlocked ? art.tint : 'rgba(61, 74, 148, 0.35)' }}
      >
        <Cat spec={art.spec} label={unlocked ? item.name : '还没解锁的表情'} size={84}
          className={unlocked ? '' : 'opacity-25 brightness-0 invert'} />
      </div>
      <p className={`mt-2 text-sm ${unlocked ? 'font-bold text-window' : 'text-window/60'}`}>{unlocked ? item.name : '？？？'}</p>
      <p className="mt-0.5 text-xs leading-snug text-window/50">{item.hint}</p>
    </li>
  )
}

export default function Dex() {
  const [data, setData] = useState<UnlocksResponse | null>(null)
  const [error, setError] = useState<string | null>(null)

  const load = () => {
    setError(null)
    api<UnlocksResponse>('/api/unlocks').then(setData).catch((e: Error) => setError(e.message))
  }
  useEffect(load, [])

  return (
    <Screen>
      <div style={{ paddingBottom: TAB_BAR_SPACE }}>
        <header className="flex items-end justify-between pt-2">
          <h1 className="font-display text-4xl text-lamp">表情图鉴</h1>
          {data && (
            <p className="text-window/80" aria-label={`已收集 ${data.unlockedCount} 个，共 ${data.appTotal} 个`}>
              <span className="font-display text-3xl text-window">{data.unlockedCount}</span>
              <span className="text-lg"> / {data.appTotal}</span>
            </p>
          )}
        </header>
        <p className="mt-2 text-sm text-window/60">网页版能集齐 30 个，全套 75 个要在相伴 App 里慢慢攒。</p>

        {error && (
          <div className="mt-16 text-center">
            <p className="text-window/80">{error}</p>
            <PrimaryButton className="mt-4 max-w-60" onClick={load}>再试一次</PrimaryButton>
          </div>
        )}
        {data && (
          <ul className="mt-6 grid grid-cols-3 gap-x-3 gap-y-5">
            {data.items.map((item) => <Tile key={item.id} item={item} />)}
          </ul>
        )}
      </div>
    </Screen>
  )
}
