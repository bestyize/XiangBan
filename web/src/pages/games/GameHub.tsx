import { useEffect, useState } from 'react'
import { Link } from 'wouter'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { Screen } from '../../components/ui.tsx'
import { gamePackThumbnailUrl, loadGameCatalog, type GamePackManifest } from '../../games/packLoader.ts'
import { api, getToken } from '../../api/client.ts'
import { useProfile, useSession } from '../../lib/session.tsx'
import { AccountBadge } from '../../components/AccountBadge.tsx'
import { storage } from '../../lib/storage.ts'
import { GuestNotice } from '../../components/GuestNotice.tsx'

export default function GameHub() {
  const [games, setGames] = useState<GamePackManifest[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [summaries, setSummaries] = useState<{ gameId: string; sessions: number; bestScore: number; bestTile: number }[]>([])
  const [recordError, setRecordError] = useState<string | null>(null)
  const profile = useProfile()
  const { isLoggedIn } = useSession()
  async function refreshRecords() {
    if (!getToken()) {
      try {
        const raw = storage.get('xb_guest_games') || '{}'
        const records = JSON.parse(raw) as Record<string, { sessions: number; bestScore: number; bestTile: number }>
        const list = Object.entries(records).map(([gameId, val]) => ({ gameId, sessions: val.sessions, bestScore: val.bestScore, bestTile: val.bestTile }))
        setSummaries(list)
        setRecordError(null)
      } catch {
        setSummaries([])
      }
      return
    }
    try { setSummaries(await api('/api/games/summary')); setRecordError(null) } catch (e) { setRecordError((e as Error).message) }
  }
  useEffect(() => { void refreshRecords() }, [profile.userId])

  useEffect(() => {
    const controller = new AbortController()
    loadGameCatalog(controller.signal)
      .then(setGames)
      .catch((cause: unknown) => {
        if (cause instanceof Error && cause.name === 'AbortError') return
        setError(cause instanceof Error ? cause.message : '游戏广场暂时打不开')
      })
    return () => controller.abort()
  }, [])

  return (
    <Screen>
      <div className="flex-1" style={{ paddingBottom: TAB_BAR_SPACE }}>
        <header className="flex items-center justify-between pt-2">
          <Link href="/" className="text-sm text-window/60">‹ 回到首页</Link>
          <AccountBadge />
        </header>
        <h1 className="mt-5 font-display text-4xl text-lamp">小游戏广场</h1>
        <p className="mt-2 text-sm text-window/65">下班后，轻松玩一局。</p>

        {games === null && !error && (
          <p className="mt-8 text-sm text-window/55">小伴正在摆好游戏…</p>
        )}
        {error && <p role="alert" className="mt-8 text-sm text-blush">{error}</p>}

        {games && games.length === 0 && (
          <p className="mt-8 text-sm text-window/55">新游戏还在路上。</p>
        )}

        <div className="mt-6 space-y-3">
          {games?.map((game) => (
            <Link
              key={game.id}
              href={'/games/' + game.id}
              className="flex items-center gap-4 rounded-2xl border border-window/10 bg-dusk-soft/35 p-4 transition hover:bg-dusk-soft/60"
            >
              {game.cover ? (
                <img src={gamePackThumbnailUrl(game)} alt="" className="size-14 shrink-0 rounded-2xl object-cover" />
              ) : (
                <span className="flex size-14 shrink-0 items-center justify-center rounded-2xl bg-lamp/15 text-3xl" aria-hidden="true">🧩</span>
              )}
              <span className="min-w-0 flex-1">
                <span className="block font-bold text-window">{game.name}</span>
                <span className="mt-1 block text-sm text-window/60">{game.description}</span>
              </span>
              <span className="shrink-0 text-sm text-lamp">开一局 ›</span>
            </Link>
          ))}
        </div>

        <section className="mt-6 rounded-2xl bg-dusk-soft/35 p-4"><div className="flex items-center justify-between"><h2 className="font-bold">我的游戏记录</h2><button className="text-xs text-lamp underline" onClick={() => void refreshRecords()}>刷新</button></div>
          <p className="mt-2 text-xs text-window/50">{isLoggedIn ? `用户 ID ${profile.userId} · 已登录，可跨设备找回` : '游客模式 · 数据保存在当前浏览器'}</p>
          {recordError && <p role="alert" className="mt-2 text-xs text-blush">{recordError}</p>}
          {summaries.length ? summaries.map((record) => <div key={record.gameId} className="mt-3 flex justify-between gap-3 text-sm"><span>{games?.find((game) => game.id === record.gameId)?.name ?? record.gameId}</span><span className="text-window/65">{record.sessions} 局 · 最高 {record.bestScore} 分</span></div>) : <p className="mt-3 text-sm text-window/50">还没有游戏记录，轻松开一局吧。</p>}
        </section>
        {!isLoggedIn && <GuestNotice className="mt-4" />}
        <p className="mt-6 text-xs text-window/40">每款游戏独立加载，成绩与对局记录按数字用户 ID 保存在独立游戏表中。</p>
      </div>
    </Screen>
  )
}
