/** The 下班啦 check-in campaign lives here; the 相伴 home page is '/'. */
export const ACTIVITY_PATH = '/xiaban'

const AFTER_HELLO_KEY = 'xb_after_hello'

/** A newcomer from a shared card goes through onboarding first, then back to the campaign. */
export function rememberAfterOnboarding(path: string) {
  try {
    sessionStorage.setItem(AFTER_HELLO_KEY, path)
  } catch {
    // Private mode: they simply land on the home page.
  }
}

export function takeAfterOnboarding(): string | null {
  try {
    const path = sessionStorage.getItem(AFTER_HELLO_KEY)
    sessionStorage.removeItem(AFTER_HELLO_KEY)
    return path?.startsWith('/') && !path.startsWith('//') ? path : null
  } catch {
    return null
  }
}

interface HistoryLike {
  state: unknown
  replaceState(data: unknown, unused: string, url?: string): void
  pushState(data: unknown, unused: string, url?: string): void
}

/**
 * Opening the campaign straight from a link (or an old card QR pointing at '/?from=card') slips the home page
 * underneath it in the history, so "back" leads into 相伴 instead of out of the site. Runs before the app mounts.
 */
export function layHomeUnderActivity(loc: { pathname: string; search: string; hash: string }, history: HistoryLike) {
  // window.location is live: read it before rewriting the history.
  const { pathname, search, hash } = loc
  const legacyCard = pathname === '/' && new URLSearchParams(search).get('from') === 'card'
  if (pathname !== ACTIVITY_PATH && !legacyCard) return false
  // A reload of the campaign page already has home beneath it.
  if ((history.state as { xbActivity?: boolean } | null)?.xbActivity) return false
  history.replaceState(null, '', '/')
  history.pushState({ xbActivity: true }, '', `${ACTIVITY_PATH}${search}${hash}`)
  return true
}
