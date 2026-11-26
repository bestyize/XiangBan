/** Only public work identities cross the app boundary, never a URL, local path or credential. */
export type ClientWork = { kind: 'builtin' | 'idea'; id: string } | { kind: 'pack'; id: number; version: number }

export function clientWorkLink(work?: ClientWork): string | undefined {
  if (!work) return
  const base = 'xiaoban://community/run/'
  if (work.kind === 'pack') {
    if (Number.isSafeInteger(work.id) && work.id > 0 && Number.isInteger(work.version) && work.version > 0 && work.version <= 2147483647) {
      return base + 'pack/' + work.id + '/' + work.version
    }
  } else if ((work.kind === 'builtin' || work.kind === 'idea') && typeof work.id === 'string' && /^[a-z0-9][a-z0-9-]{1,39}$/.test(work.id)) {
    return base + work.kind + '/' + work.id
  }
}

export function workClientPlatform(ua: string, platform: string, touchPoints: number) {
  if (/XiaobanWorkHost|iPhone|iPad|iPod/i.test(ua) || (platform === 'MacIntel' && touchPoints > 1)) return
  if (/Android/i.test(ua)) return 'android' as const
  if (/Windows|Macintosh/i.test(ua)) return 'desktop' as const
}
