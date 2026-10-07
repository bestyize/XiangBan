import { useEffect, useState } from 'react'
import { api, getToken } from '../../api/client.ts'
import type { Unlock, UnlocksResponse } from '../../api/types.ts'
import { dexArt, fallbackArt } from '../../robot/moods.ts'
import { Robot } from '../../components/Robot.tsx'
import { Sheet } from '../../components/Sheet.tsx'
import { PrimaryButton } from '../../components/ui.tsx'
import { useLoginModal } from '../../lib/loginModal.ts'

function Tile({ item }: { item: Unlock }) {
  const art = dexArt[item.id] ?? fallbackArt
  const unlocked = item.unlockedAt !== null
  return (
    <li className="flex flex-col items-center text-center">
      <div className="grid aspect-square w-full place-items-center rounded-3xl"
        style={{ background: unlocked ? art.tint : 'rgba(61, 80, 105, 0.45)' }}>
        <Robot spec={art.spec} alive={false} label={unlocked ? item.name : '还没解锁的表情'} size={84}
          className={unlocked ? '' : 'opacity-25 brightness-0 invert'} />
      </div>
      <p className={`mt-2 text-sm ${unlocked ? 'font-bold text-window' : 'text-window/60'}`}>{unlocked ? item.name : '？？？'}</p>
      <p className="mt-0.5 text-xs leading-snug text-window/50">{item.hint}</p>
    </li>
  )
}

/** 表情图鉴 (PRD 5.8) as a sheet over the home screen; reloads each time it opens. */
export function DexSheet({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { openLogin } = useLoginModal()
  const [data, setData] = useState<UnlocksResponse | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Guests have no server account, so there is no collection to load yet.
  const guest = !getToken()
  const load = () => {
    setError(null)
    api<UnlocksResponse>('/api/unlocks').then(setData).catch((e: Error) => setError(e.message))
  }
  useEffect(() => {
    if (open && !guest) load()
  }, [open, guest])

  const progress = data && (
    <p className="text-window/80" aria-label={`已收集 ${data.unlockedCount} 个，共 ${data.appTotal} 个`}>
      <span className="font-display text-2xl text-lamp">{data.unlockedCount}</span>
      <span className="text-sm"> / {data.appTotal}</span>
    </p>
  )

  return (
    <Sheet open={open} onClose={onClose} title="表情图鉴" aside={progress}>
      <p className="text-sm text-window/60">网页版能集齐 30 个，全套 75 个要在相伴 App 里慢慢攒。</p>
      {error && (
        <div className="mt-10 text-center">
          <p className="text-window/80">{error}</p>
          <PrimaryButton className="mt-4 max-w-60" onClick={load}>再试一次</PrimaryButton>
        </div>
      )}
      {guest && (
        <div className="mt-8 flex flex-col items-center text-center">
          <Robot spec="greeting" label="小伴" size={110} />
          <p className="mt-4 text-[15px] text-window/85">登录后，打卡解锁的表情会收进这里。</p>
          <button type="button" onClick={() => { onClose(); openLogin() }} className="mt-6 w-full rounded-full bg-lamp px-6 py-3.5 text-center font-bold text-ink transition active:scale-[.98]">登录 / 注册</button>
        </div>
      )}
      {!guest && !data && !error && <p className="mt-10 text-center text-sm text-window/50">正在翻图鉴…</p>}
      {data && (
        <ul className="mt-5 grid grid-cols-3 gap-x-3 gap-y-5">
          {data.items.map((item) => <Tile key={item.id} item={item} />)}
        </ul>
      )}
    </Sheet>
  )
}
