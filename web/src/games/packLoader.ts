export interface GamePackManifest {
  id: string
  version: string
  name: string
  description: string
  entry: string
  cover?: string
  runtimeProtocol: number
}

export const GAME_RUNTIME_PROTOCOL = 1
const PACK_ID = /^[a-z0-9-]+$/
const configuredPackBase = import.meta.env.VITE_GAME_PACK_BASE_URL as string | undefined
const PACK_BASE = configuredPackBase?.trim()
  ? configuredPackBase.replace(/\/+$/, '')
  : '/game-packs'

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function safeEntryPath(value: unknown): value is string {
  if (typeof value !== 'string' || value.startsWith('/')) return false
  return value.split('/').every((part) =>
    part.length > 0 && part !== '.' && part !== '..' && /^[a-zA-Z0-9._-]+$/.test(part),
  )
}

function packAssetUrl(id: string, file: string): string {
  const safeId = encodeURIComponent(id)
  const safePath = file.split('/').map(encodeURIComponent).join('/')
  return PACK_BASE + '/' + safeId + '/' + safePath
}

async function fetchJson(url: string, signal?: AbortSignal): Promise<unknown> {
  const response = await fetch(url, { signal })
  if (!response.ok) throw new Error('游戏资源加载失败（' + response.status + '）')
  return response.json() as Promise<unknown>
}

export async function loadGameManifest(id: string, signal?: AbortSignal): Promise<GamePackManifest> {
  if (!PACK_ID.test(id)) throw new Error('游戏编号无效')
  const value = await fetchJson(packAssetUrl(id, 'manifest.json'), signal)
  if (
    !isRecord(value) ||
    value.id !== id ||
    typeof value.version !== 'string' ||
    typeof value.name !== 'string' ||
    typeof value.description !== 'string' ||
    !safeEntryPath(value.entry) ||
    (value.cover !== undefined && !safeEntryPath(value.cover)) ||
    value.runtimeProtocol !== GAME_RUNTIME_PROTOCOL
  ) {
    throw new Error('游戏包清单格式不支持')
  }

  return {
    id,
    version: value.version,
    name: value.name,
    description: value.description,
    entry: value.entry,
    cover: typeof value.cover === 'string' ? value.cover : undefined,
    runtimeProtocol: GAME_RUNTIME_PROTOCOL,
  }
}

export async function loadGameCatalog(signal?: AbortSignal): Promise<GamePackManifest[]> {
  const value = await fetchJson(PACK_BASE + '/catalog.json', signal)
  if (!isRecord(value) || value.version !== 1 || !Array.isArray(value.games)) {
    throw new Error('游戏清单格式不支持')
  }

  const ids = value.games as unknown[]
  if (!ids.every((id) => typeof id === 'string' && PACK_ID.test(id))) {
    throw new Error('游戏清单包含无效编号')
  }

  return Promise.all([...new Set(ids as string[])].map((id) => loadGameManifest(id, signal)))
}

export function gameEntryUrl(pack: GamePackManifest, nonce: string): string {
  return packAssetUrl(pack.id, pack.entry) + '?hostNonce=' + encodeURIComponent(nonce)
}

export function gamePackThumbnailUrl(pack: GamePackManifest): string {
  return pack.cover ? packAssetUrl(pack.id, pack.cover) : ''
}
