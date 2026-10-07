import type { ResultGame } from './types.ts'

const PUBLIC_BASE = 'https://xiaban.xyz'

/** Local preview addresses must never become unusable invitation links on shared cards. */
export function gamePublicBase(base: string): string {
  try {
    const url = new URL(base)
    const host = url.hostname.toLowerCase()
    const local = /^(localhost|127\.|0\.|10\.|192\.168\.|172\.(1[6-9]|2\d|3[01])\.)/.test(host) || host === '[::1]' || host.endsWith('.local')
    return !local && ['http:', 'https:'].includes(url.protocol) && !url.username && !url.password ? url.origin : PUBLIC_BASE
  } catch { return PUBLIC_BASE }
}

/** Only public invitation codes travel on links; anything else (an email, say) is dropped. */
export function referral(inviteCode?: string): string { return inviteCode && /^[a-zA-Z0-9_-]{1,80}$/.test(inviteCode) ? inviteCode : '' }

function sharePath<R extends { score: number }>(game: ResultGame<R>, result: R, query: URLSearchParams, inviteCode?: string): string {
  if (referral(inviteCode)) query.set('inv', referral(inviteCode))
  return `/games/${game.id}/share?${query}#${game.fragment(result)}`
}

/** The sharer's own result page, opened right after a round. */
export function ownResultPath<R extends { score: number }>(game: ResultGame<R>, result: R, inviteCode?: string): string {
  return sharePath(game, result, new URLSearchParams({ own: '1' }), inviteCode)
}

/** The public link friends receive. */
export function resultShareLink<R extends { score: number }>(game: ResultGame<R>, result: R, base: string, inviteCode?: string): string {
  return new URL(sharePath(game, result, new URLSearchParams({ from: 'game_share' }), inviteCode), gamePublicBase(base)).toString()
}

/** Guest play of the same game, without onboarding or a login reminder. */
export function challengePath<R extends { score: number }>(game: ResultGame<R>, result: R | null, inviteCode?: string): string {
  const query = new URLSearchParams({ from: 'game_share' })
  if (referral(inviteCode)) query.set('inv', referral(inviteCode))
  for (const [key, value] of Object.entries(game.challengeQuery?.(result) ?? {})) query.set(key, value)
  return `/games/${game.id}?${query}`
}
