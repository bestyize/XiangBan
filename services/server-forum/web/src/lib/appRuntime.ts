export const appRuntimeChannel = 'xiaoban.app-runtime.v2'

export function appRuntimeBase(value: unknown, development: boolean): URL | null {
  if (typeof value !== 'string' || !value) return null
  try {
    const url = new URL(value)
    const local = development && url.protocol === 'http:' && url.hostname.endsWith('.localhost')
    if (!local && (url.protocol !== 'https:' || url.port)) return null
    if (!/^[a-z0-9-]+(?:\.[a-z0-9-]+)+$/.test(url.hostname) || url.username || url.password ||
        url.pathname !== '/' || url.search || url.hash) return null
    return url
  } catch {
    return null
  }
}

export function appRuntimeEntry(base: URL, id: number, version: number, manifest: Record<string, unknown>): URL {
  if (!Number.isSafeInteger(id) || id < 1 || !Number.isSafeInteger(version) || version < 1 ||
      manifest.schemaVersion !== 3 || manifest.runtimeProtocol !== 2) throw new Error('作品版本信息无效')
  const entry = manifest.entry ?? 'index.html'
  if (typeof entry !== 'string' || !entry.endsWith('.html') ||
      !entry.split('/').every(part => /^[A-Za-z0-9_@-][A-Za-z0-9._@-]*$/.test(part))) throw new Error('作品入口无效')
  const url = new URL(base)
  url.hostname = `${id}.${base.hostname}`
  url.pathname = `/v${version}/${entry}`
  return url
}
