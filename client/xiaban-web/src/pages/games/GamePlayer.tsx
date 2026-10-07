import { useLoginReminder } from '../../components/LoginReminder.tsx'
import { clearLoginReturn, readLoginReturn } from '../../lib/loginReturn.ts'
import { useEffect, useRef, useState } from 'react'
import { useLocation, useRoute } from 'wouter'
import { GAME_RUNTIME_PROTOCOL, gameEntryUrl, loadGameManifest, type GamePackManifest } from '../../games/packLoader.ts'
import { api, getToken } from '../../api/client.ts'
import { storage } from '../../lib/storage.ts'
import { track } from '../../lib/track.ts'
import { useProfile } from '../../lib/session.tsx'
import { canShareFinishedRound, localResultPath, parseSnakeResult } from '../../games/resultShare.ts'
import { canShareOtherResult, otherResultPath, parseOtherResult } from '../../games/otherResultShare.ts'

interface Round { id: string | null; score: number; lines: number; maxTile: number; finished: boolean; timer: ReturnType<typeof setTimeout> | null }

const HOST_CHANNEL = 'xiaban.game-host'
const PACK_CHANNEL = 'xiaban.game-pack'

function createNonce(): string {
  try {
    return crypto.randomUUID()
  } catch {
    return Date.now().toString(36) + Math.random().toString(36).slice(2)
  }
}

function gameSafeArea() {
  const style = getComputedStyle(document.documentElement)
  return Object.fromEntries(['top', 'right', 'bottom', 'left'].map((side) => [
    side, Math.max(0, Math.min(160, parseFloat(style.getPropertyValue('--safe-' + side)) || 0)),
  ]))
}

export default function GamePlayer() {
  const [, params] = useRoute('/games/:gameId')
  const [, navigate] = useLocation()
  const gameId = params?.gameId ?? ''
  const profile = useProfile()
  const [nonce] = useState(createNonce)
  const { ask, modal } = useLoginReminder('game')
  const [allowed, setAllowed] = useState(false)
  const [pack, setPack] = useState<GamePackManifest | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [ready, setReady] = useState(false)
  const iframeRef = useRef<HTMLIFrameElement>(null)
  const roundRef = useRef<Round | null>(null)
  const [recordError, setRecordError] = useState<string | null>(null)

  useEffect(() => { if (pack) document.title = `${pack.name}｜相伴` }, [pack])

  useEffect(() => {
    if (pack?.orientation === 'landscape') document.documentElement.dataset.gameLayout = 'landscape'
    return () => { delete document.documentElement.dataset.gameLayout }
  }, [pack?.orientation])

  useEffect(() => {
    if (pack?.orientation !== 'landscape') return
    const sendLayout = () => iframeRef.current?.contentWindow?.postMessage({
      channel: HOST_CHANNEL, protocol: GAME_RUNTIME_PROTOCOL, type: 'host.layout', nonce, safeArea: gameSafeArea(),
    }, '*')
    window.addEventListener('resize', sendLayout)
    document.addEventListener('fullscreenchange', sendLayout)
    return () => {
      window.removeEventListener('resize', sendLayout)
      document.removeEventListener('fullscreenchange', sendLayout)
    }
  }, [pack?.orientation, nonce])

  useEffect(() => {
    if (readLoginReturn()?.path === `/games/${gameId}`) clearLoginReturn()
    if (['game_share', 'game-share'].includes(new URLSearchParams(location.search).get('from') || '')) setAllowed(true)
    else ask(() => setAllowed(true))
  }, [gameId])

  useEffect(() => {
    setPack(null)
    setError(null)
    setReady(false)
    if (!gameId) {
      setError('没有找到这款游戏')
      return
    }
    const controller = new AbortController()
    loadGameManifest(gameId, controller.signal)
      .then(setPack)
      .catch((cause: unknown) => {
        if (cause instanceof Error && cause.name === 'AbortError') return
        setError(cause instanceof Error ? cause.message : '游戏包暂时无法加载')
      })
    return () => controller.abort()
  }, [gameId])

  useEffect(() => {
    async function persist(round: Round, keepalive = false) {
      if (!round.id) return
      if (!getToken()) {
        try {
          const raw = storage.get('xb_guest_games') || '{}'
          const records = JSON.parse(raw) as Record<string, { sessions: number; bestScore: number; bestTile: number }>
          const current = records[gameId] || { sessions: 0, bestScore: 0, bestTile: 0 }
          if (round.finished) current.sessions += 1
          if (round.score > current.bestScore) current.bestScore = round.score
          if (round.maxTile > current.bestTile) current.bestTile = round.maxTile
          records[gameId] = current
          storage.set('xb_guest_games', JSON.stringify(records))
        } catch {}
        return
      }
      try { await api(`/api/games/sessions/${round.id}`, { method: 'PUT', keepalive, body: { score: round.score, lines: round.lines, maxTile: round.maxTile, finished: round.finished } }) }
      catch { if (round === roundRef.current) setRecordError('游戏记录暂未保存，网络恢复后可继续重试。') }
    }
    function finish(round: Round) {
      if (round.finished) return
      if (round.timer) clearTimeout(round.timer)
      round.timer = null; round.finished = true
      track('game_finish', { game: gameId, score: String(round.score) })
      void persist(round, true)
    }
    function receiveMessage(event: MessageEvent) {
      if (event.source !== iframeRef.current?.contentWindow) return
      const message: unknown = event.data
      if (!message || typeof message !== 'object') return
      const data = message as Record<string, unknown>
      if (
        data.channel !== PACK_CHANNEL ||
        data.protocol !== GAME_RUNTIME_PROTOCOL ||
        data.nonce !== nonce
      ) return

      if (data.type === 'game.ready') setReady(true)
      if (data.type === 'game.share') {
        const result = gameId === 'snake-arena' ? parseSnakeResult(data.result) : null
        const other = parseOtherResult(data.result)
        const round = roundRef.current
        const path = canShareFinishedRound(result, round) ? localResultPath(result, profile.inviteCode)
          : canShareOtherResult(other, gameId, round) ? otherResultPath(other, profile.inviteCode) : null
        if (path) {
          const open = async () => {
            try { if (document.fullscreenElement) await document.exitFullscreen() } catch { /* Sharing still works outside fullscreen. */ }
            navigate(path)
          }
          void open()
        }
        return
      }
      if (data.type === 'game.start') {
        if (roundRef.current) finish(roundRef.current)
        const round: Round = { id: null, score: 0, lines: 0, maxTile: 0, finished: false, timer: null }
        roundRef.current = round; setRecordError(null)
        track('game_start', { game: gameId })
        if (!getToken()) {
          round.id = `guest_${Date.now()}`
        } else {
          void api<{ id: string }>('/api/games/sessions', { method: 'POST', body: { gameId } }).then((session) => {
            round.id = session.id; void persist(round)
          }).catch(() => { if (round === roundRef.current) setRecordError('对局记录暂未建立，请检查网络。') })
        }
      }
      if (data.type === 'game.progress' || data.type === 'game.result') {
        const round = roundRef.current
        if (!round || round.finished) return
        const score = Number(data.score); const lines = Number(data.lines ?? 0); const tile = Number(data.maxTile ?? 0)
        if (!Number.isInteger(score) || score < 0 || score > 1e9 || !Number.isInteger(lines) || lines < 0 || lines > 1e6 || !Number.isInteger(tile) || tile < 0 || tile > 1073741824) return
        round.score = score; round.lines = lines; round.maxTile = tile
        if (data.type === 'game.result') finish(round)
        else if (!round.timer) round.timer = setTimeout(() => { round.timer = null; void persist(round) }, 1200)
      }
      if (data.type === 'game.exit') {
        if (roundRef.current) finish(roundRef.current)
        navigate('/games')
      }
    }

    window.addEventListener('message', receiveMessage)
    const hide = () => { if (document.visibilityState === 'hidden' && roundRef.current) void persist(roundRef.current, true) }
    document.addEventListener('visibilitychange', hide)
    return () => { window.removeEventListener('message', receiveMessage); document.removeEventListener('visibilitychange', hide); if (roundRef.current) finish(roundRef.current) }
  }, [navigate, nonce, gameId, profile.inviteCode])

  function initializeGame() {
    iframeRef.current?.contentWindow?.postMessage({
      channel: HOST_CHANNEL,
      protocol: GAME_RUNTIME_PROTOCOL,
      type: 'host.init',
      nonce,
      safeArea: gameSafeArea(),
    }, '*')
  }

  return (
    <div className={'game-stage' + (pack?.orientation === 'landscape' ? ' game-stage-landscape' : '')}>
      {error ? (
        <main className="game-stage-message">
          <p role="alert" className="text-sm text-blush">{error}</p>
          <button type="button" className="mt-4 text-sm text-lamp" onClick={() => navigate('/games')}>
            返回游戏广场
          </button>
        </main>
      ) : pack && allowed ? (
        <>
          {!ready && <div className="game-stage-loading" role="status">正在加载游戏…</div>}
          <iframe
            ref={iframeRef}
            title={pack.name}
            src={gameEntryUrl(pack, nonce, { roundType: new URLSearchParams(location.search).get('round') || undefined, difficulty: new URLSearchParams(location.search).get('difficulty') || undefined })}
            sandbox="allow-scripts allow-orientation-lock"
            allow="fullscreen"
            allowFullScreen
            onLoad={initializeGame}
            className="game-stage-frame"
          />
        </>
      ) : (
        <p className="game-stage-message text-sm text-window/55">正在读取游戏包…</p>
      )}
      {recordError && <p role="status" className="game-stage-notice">{recordError}</p>}
      {modal}
    </div>
  )
}
