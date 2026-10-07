import { useLoginReminder } from '../../components/LoginReminder.tsx'
import { clearLoginReturn, readLoginReturn } from '../../lib/loginReturn.ts'
import { useEffect, useRef, useState } from 'react'
import { useLocation, useRoute } from 'wouter'
import { GAME_RUNTIME_PROTOCOL, gameEntryUrl, loadGameManifest, type GamePackManifest } from '../../games/common/packLoader.ts'
import { api, getToken } from '../../api/client.ts'
import { storage } from '../../lib/storage.ts'
import { track } from '../../lib/track.ts'
import { useProfile } from '../../lib/session.tsx'
import { loadGameSave, storeGameSave, validSave, type GameSave } from '../../games/common/saves.ts'
import { ownResultPath } from '../../games/common/share.ts'
import { resultGame } from '../../games/index.ts'
import { actSnakeWallet, readSnakeWallet, type SnakeWalletAction } from '../../games/snake-arena/economy.ts'
import { rememberSnakeShareRound } from '../../games/snake-arena/shareReward.ts'

interface Round { id: string | null; score: number; lines: number; maxTile: number; finished: boolean; timer: ReturnType<typeof setTimeout> | null; key: string; start: Promise<string | null>; finalSave: Promise<boolean> | null; guestCounted: boolean }

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
  const registered = profile.accountType === 'registered'
  // Packs that keep progress only start once their save is loaded, so an empty save never overwrites real progress.
  const [saveStatus, setSaveStatus] = useState<'idle' | 'loading' | 'ready' | 'error'>('idle')
  const [saveAttempt, setSaveAttempt] = useState(0)
  const saveRef = useRef<GameSave | null>(null)
  const pendingSaveRef = useRef<GameSave | null>(null)
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const packRef = useRef<GamePackManifest | null>(null)
  packRef.current = pack

  useEffect(() => {
    if (!pack?.save || !allowed) return
    let cancelled = false
    setSaveStatus('loading')
    loadGameSave(gameId, registered)
      .then((data) => { if (!cancelled) { saveRef.current = data; setSaveStatus('ready') } })
      .catch(() => { if (!cancelled) setSaveStatus('error') })
    return () => { cancelled = true }
  }, [pack?.save, allowed, gameId, registered, saveAttempt])

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
    function flushSave(keepalive = false) {
      if (saveTimerRef.current) { clearTimeout(saveTimerRef.current); saveTimerRef.current = null }
      const data = pendingSaveRef.current
      if (!data) return
      pendingSaveRef.current = null
      saveRef.current = data
      void storeGameSave(gameId, registered, data, keepalive).catch(() => {
        pendingSaveRef.current ??= data
        setRecordError('游戏存档暂未同步，网络恢复后会自动重试。')
      })
    }
    async function persist(round: Round, keepalive = false) {
      if (!round.id) return false
      if (!getToken()) {
        try {
          const raw = storage.get('xb_guest_games') || '{}'
          const records = JSON.parse(raw) as Record<string, { sessions: number; bestScore: number; bestTile: number }>
          const current = records[gameId] || { sessions: 0, bestScore: 0, bestTile: 0 }
          if (round.finished && !round.guestCounted) { current.sessions += 1; round.guestCounted = true }
          if (round.score > current.bestScore) current.bestScore = round.score
          if (round.maxTile > current.bestTile) current.bestTile = round.maxTile
          records[gameId] = current
          storage.set('xb_guest_games', JSON.stringify(records))
        } catch {}
        return true
      }
      try { await api(`/api/games/sessions/${round.id}`, { method: 'PUT', keepalive, body: { score: round.score, lines: round.lines, maxTile: round.maxTile, finished: round.finished } }); return true }
      catch { if (round === roundRef.current) setRecordError('游戏记录暂未保存，网络恢复后可继续重试。'); return false }
    }
    function finish(round: Round) {
      if (round.finished) return
      if (round.timer) clearTimeout(round.timer)
      round.timer = null; round.finished = true
      track('game_finish', { game: gameId, score: String(round.score) })
      round.finalSave = round.start.then(() => persist(round, true))
    }
    let walletQueue: Promise<unknown> = Promise.resolve()
    function walletReply(requestId: string, payload: Record<string, unknown>) {
      iframeRef.current?.contentWindow?.postMessage({ channel: HOST_CHANNEL, protocol: GAME_RUNTIME_PROTOCOL, type: 'host.wallet', nonce, requestId, ...payload }, '*')
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
      if (data.type === 'game.save') {
        if (!packRef.current?.save || !validSave(data.data)) return
        pendingSaveRef.current = data.data
        if (!saveTimerRef.current) saveTimerRef.current = setTimeout(() => flushSave(), 800)
        return
      }

      if (data.type === 'game.wallet' && gameId === 'snake-arena') {
        if (typeof data.requestId !== 'string' || !/^[a-zA-Z0-9_-]{1,80}$/.test(data.requestId)) return
        const requestId = data.requestId
        const round = roundRef.current
        const run = async () => {
          try {
            if (data.action === 'read') { walletReply(requestId, { wallet: await readSnakeWallet(), coinsEarned: 0, protectionSeconds: 0 }); return }
            const actions: SnakeWalletAction[] = ['buy_shield', 'use_shield', 'revive', 'consume_start_shield', 'settle', 'share_reward']
            if (!actions.includes(data.action as SnakeWalletAction)) throw new Error('没有找到这个道具操作')
            const action = data.action as SnakeWalletAction
            const terminal = action === 'settle' || action === 'share_reward'
            if (action !== 'buy_shield' && (!round || data.roundKey !== round.key || (!terminal && round !== roundRef.current))) throw new Error('这一局已经切换，请重试')
            if (round && action !== 'buy_shield') {
              await round.start
              if (!round.id) throw new Error('这一局记录暂未建立')
              const final = terminal
              if (round.finished !== final) throw new Error(final ? '请先结束这一局' : '这一局已经结束')
              if (final && !(await round.finalSave)) {
                if (!(await persist(round))) throw new Error('战绩暂未保存，请稍后重试领取')
              }
            }
            const response = await actSnakeWallet(action, { requestId, sessionId: round?.id ?? undefined, score: round?.score, finished: round?.finished })
            walletReply(requestId, response as unknown as Record<string, unknown>)
          } catch (cause) { walletReply(requestId, { error: cause instanceof Error ? cause.message : '糖果币暂未连接，请重试' }) }
        }
        walletQueue = walletQueue.then(run, run)
        return
      }
      if (data.type === 'game.share') {
        const game = resultGame(gameId)
        const result = game?.parse(data.result) ?? null
        const path = game && result && game.canShare(result, roundRef.current) ? ownResultPath(game, result, profile.inviteCode) : null
        if (path) {
          const open = async () => {
            const round = roundRef.current
            if (gameId === 'snake-arena' && round) {
              await round.start
              if (round.finalSave) await round.finalSave
              if (round.id) rememberSnakeShareRound(round.id, round.score)
            }
            try { if (document.fullscreenElement) await document.exitFullscreen() } catch { /* Sharing still works outside fullscreen. */ }
            navigate(path)
          }
          void open()
        }
        return
      }
      if (data.type === 'game.start') {
        if (roundRef.current) finish(roundRef.current)
        const round: Round = { id: null, score: 0, lines: 0, maxTile: 0, finished: false, timer: null, key: typeof data.roundKey === 'string' ? data.roundKey : '', start: Promise.resolve(null), finalSave: null, guestCounted: false }
        roundRef.current = round; setRecordError(null)
        track('game_start', { game: gameId })
        if (!getToken()) {
          round.id = `guest_${Date.now()}_${createNonce().slice(0, 8)}`
          round.start = Promise.resolve(round.id)
        } else {
          round.start = api<{ id: string }>('/api/games/sessions', { method: 'POST', body: { gameId } }).then((session) => {
            round.id = session.id; void persist(round); return session.id
          }).catch(() => { if (round === roundRef.current) setRecordError('对局记录暂未建立，请检查网络。'); return null })
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
    const hide = () => {
      if (document.visibilityState !== 'hidden') return
      if (roundRef.current) void persist(roundRef.current, true)
      flushSave(true)
    }
    document.addEventListener('visibilitychange', hide)
    return () => {
      window.removeEventListener('message', receiveMessage); document.removeEventListener('visibilitychange', hide)
      if (roundRef.current) finish(roundRef.current)
      flushSave(true)
    }
  }, [navigate, nonce, gameId, profile.inviteCode, registered])

  function initializeGame() {
    iframeRef.current?.contentWindow?.postMessage({
      channel: HOST_CHANNEL,
      protocol: GAME_RUNTIME_PROTOCOL,
      type: 'host.init',
      nonce,
      safeArea: gameSafeArea(),
      ...(pack?.save ? { save: saveRef.current } : {}),
    }, '*')
  }

  const waitingForSave = Boolean(pack?.save) && saveStatus !== 'ready'

  return (
    <div className={'game-stage' + (pack?.orientation === 'landscape' ? ' game-stage-landscape' : '')}>
      {error ? (
        <main className="game-stage-message">
          <p role="alert" className="text-sm text-blush">{error}</p>
          <button type="button" className="mt-4 text-sm text-lamp" onClick={() => navigate('/games')}>
            返回游戏广场
          </button>
        </main>
      ) : pack && allowed && saveStatus === 'error' ? (
        <main className="game-stage-message">
          <p role="alert" className="text-sm text-blush">游戏存档暂时读不到，为免覆盖进度，先不开局。</p>
          <button type="button" className="mt-4 text-sm text-lamp" onClick={() => setSaveAttempt((value) => value + 1)}>
            重新读取存档
          </button>
        </main>
      ) : pack && allowed && waitingForSave ? (
        <p className="game-stage-message text-sm text-window/55" role="status">正在读取存档…</p>
      ) : pack && allowed ? (
        <>
          {!ready && <div className="game-stage-loading" role="status">正在加载游戏…</div>}
          <iframe
            ref={iframeRef}
            title={pack.name}
            src={gameEntryUrl(pack, nonce, resultGame(gameId)?.launchParams?.(new URLSearchParams(location.search)))}
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
