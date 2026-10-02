import { useEffect, useState } from 'react'
import { Link } from 'wouter'
import { TAB_BAR_SPACE } from '../../components/TabBar.tsx'
import { Screen } from '../../components/ui.tsx'
import { loadGameCatalog, type GamePackManifest } from '../../games/packLoader.ts'

export default function GameHub() {
  const [games, setGames] = useState<GamePackManifest[] | null>(null)
  const [error, setError] = useState<string | null>(null)

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
          <span className="text-xs text-window/45">单人 · 随时开始</span>
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
              <span className="flex size-14 shrink-0 items-center justify-center rounded-2xl bg-lamp/15 text-3xl" aria-hidden="true">🧩</span>
              <span className="min-w-0 flex-1">
                <span className="block font-bold text-window">{game.name}</span>
                <span className="mt-1 block text-sm text-window/60">{game.description}</span>
              </span>
              <span className="shrink-0 text-sm text-lamp">开一局 ›</span>
            </Link>
          ))}
        </div>

        <p className="mt-6 text-xs text-window/40">每款游戏独立加载，游戏记录不会发送到服务器。</p>
      </div>
    </Screen>
  )
}
