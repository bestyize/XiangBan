const countryLookup = 'https://ipwho.is/?fields=success,country_code'
const githubProxy = 'https://gh-proxy.com/'

let countryPromise: Promise<boolean> | undefined

// One lookup per page session, shared by the latest and historical release lists.
export function isMainlandChina(): Promise<boolean> {
  return countryPromise ??= lookupMainlandChina()
}

export async function lookupMainlandChina(fetcher: typeof fetch = fetch): Promise<boolean> {
  try {
    const response = await fetcher(countryLookup, {
      credentials: 'omit',
      referrerPolicy: 'no-referrer',
      signal: AbortSignal.timeout(3000),
    })
    if (!response.ok) return false
    const result: unknown = await response.json()
    return typeof result === 'object' && result !== null
      && 'success' in result && result.success === true
      && 'country_code' in result && result.country_code === 'CN'
  } catch {
    return false
  }
}

export function softwareDownloadUrl(url: string, mainland: boolean): string {
  // Only proxy GitHub Release asset URLs, never arbitrary external links.
  try {
    const parsed = new URL(url)
    if (mainland && parsed.origin === 'https://github.com'
      && !parsed.username && !parsed.password
      && /^\/[^/]+\/[^/]+\/releases\/download\/[^/]+\/.+/i.test(parsed.pathname)) {
      return githubProxy + parsed.href
    }
  } catch { /* Keep the original link if parsing fails. */ }
  return url
}
