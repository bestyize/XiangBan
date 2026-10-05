import { useEffect, useState } from 'react'
import { Link } from 'wouter'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { Screen } from '../../components/ui.tsx'
import { gamePackThumbnailUrl, loadGameCatalog, type GamePackManifest } from '../../games/packLoader.ts'
import { api, getToken } from '../../api/client.ts'
import { useProfile } from '../../lib/session.tsx'
import { PageHeader } from '../../components/PageHeader.tsx'
import { AccountBadge } from '../../components/AccountBadge.tsx'
import { storage } from '../../lib/storage.ts'

export default function GameHub({ active = true }: { active?: boolean }) {
  const [games, setGames] = useState<GamePackManifest[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [summaries, setSummaries] = useState<{ gameId: string; sessions: number; bestScore: number; bestTile: number }[]>([])
  const [recordError, setRecordError] = useState<string | null>(null)
  const profile = useProfile()
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
  useEffect(() => { if (active) void refreshRecords() }, [profile.userId, active])

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
        <PageHeader title="玩一局"><AccountBadge compact /></PageHeader>
        <p className="mt-2 text-sm text-window/65">下班后，轻松玩一局。</p>

        {games === null && !error && (
          <p className="mt-8 text-sm text-window/55">小伴正在摆好游戏…</p>
        )}
        {error && <p role="alert" className="mt-8 text-sm text-blush">{error}</p>}

        {games && games.length === 0 && (
          <p className="mt-8 text-sm text-window/55">新游戏还在路上。</p>
        )}

        <div className="game-grid">
          {games?.map((game) => {
            const record = summaries.find((item) => item.gameId === game.id)
            return (
              <Link key={game.id} href={'/games/' + game.id} className="game-card">
                <span className="game-card-cover">
                  {game.cover
                    ? <img src={gamePackThumbnailUrl(game)} alt="" />
                    : <span aria-hidden="true">🧩</span>}
                </span>
                <span className="game-card-name">{game.name}</span>
                <span className="game-card-desc">{game.description}</span>
                <span className="game-card-foot">
                  {record && record.sessions > 0
                    ? <><b>{record.bestScore}</b><small>最高分 · {record.sessions} 局</small></>
                    : <small>还没玩过</small>}
                  <span className="game-card-play" aria-hidden="true">▶</span>
                </span>
              </Link>
            )
          })}
        </div>

        {recordError && <p role="alert" className="mt-4 text-xs text-blush">{recordError}</p>}
      </div>
    </Screen>
  )
}
