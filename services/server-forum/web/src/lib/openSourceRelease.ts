import { request } from '@/api/forum'
import type { SoftwarePlatform } from './openSourceSoftware'

const packageExtension = /\.(exe|msi|msix|appx|dmg|pkg|appimage|deb|rpm|flatpak|apk|zip|7z|txz|tar\.(?:gz|bz2|xz|zst))$/i
const auxiliaryAsset = /debug|dsym|dbgsym|pdbs|symbols|(?:^|[-_.])(?:src|source|sources|sha256|checksums?)(?:[-_.]|$)|export_templates|dont-use-this-file|(?:^|[-_.])(?:libhb|plugin)(?:[-_.]|$)/i

function platformFor(name: string, format: string): SoftwarePlatform | undefined {
  if (format === 'apk') return 'Android'
  if (['exe', 'msi', 'msix', 'appx'].includes(format) || /(?:^|[-_.])win(?:dows)?(?:32|64)?(?:[-_.]|$)/.test(name)) return 'Windows'
  if (['dmg', 'pkg'].includes(format) || /(?:^|[-_.])(?:mac(?:os)?|darwin)(?:[-_.]|$)/.test(name)) return 'macOS'
  if (['appimage', 'deb', 'rpm', 'flatpak', 'tar.zst'].includes(format) || /(?:^|[-_.])(?:linux|ubuntu)(?:[-_.]|$)/.test(name)) return 'Linux'
}

function architectureFor(name: string) {
  if (/universal/.test(name)) return '通用'
  if (/aarch64|arm64|macos-apple/.test(name)) return 'ARM64'
  if (/armv7|arm32/.test(name)) return 'ARM 32 位'
  if (/x86[-_]64|x64|amd64|win64|macos-intel/.test(name)) return 'x64'
  if (/(?:^|[-_.])(?:x86(?:_32)?|i[3-6]86|win32)(?:[-_.]|$)/.test(name)) return 'x86'
}

interface GitHubAsset {
  name: string
  browser_download_url: string
  size?: number
}

interface GitHubRelease {
  id: number | string
  prerelease: boolean
  tag_name: string
  published_at: string | null
  assets: GitHubAsset[]
}

export interface SoftwareSyncSummary {
  stars: number
  updatedAt: string | null
  version: string | null
  syncedAt: string
  starGain: number | null
}

export function fetchSoftwareSummaries() {
  return request<Record<string, SoftwareSyncSummary>>('GET', '/open-source/software', undefined, false)
}

export async function fetchSoftwareSnapshot(project: string, releaseRepo: string) {
  const path = project.split('/').map(encodeURIComponent).join('/')
  const snapshot = await request<Omit<SoftwareSyncSummary, 'starGain'> & {
    latest: GitHubRelease | null
    history: GitHubRelease[]
  } | null>('GET', `/open-source/software/${path}`, undefined, false)
  if (!snapshot) return null
  return {
    ...snapshot,
    latest: snapshot.latest && normalizeRelease(releaseRepo, snapshot.latest),
    history: snapshot.history.map(release => normalizeRelease(releaseRepo, release)),
  }
}

function normalizeRelease(repo: string, release: GitHubRelease) {
  const releasePath = `/${repo.toLowerCase()}/releases/download/`
  const packages = release.assets.flatMap(asset => {
    const name = asset.name.toLowerCase()
    const format = name.match(packageExtension)?.[1]
    if (!format || auxiliaryAsset.test(name)) return []
    const platform = platformFor(name, format)
    if (!platform) return []
    const url = URL.parse(asset.browser_download_url)
    if (url?.origin !== 'https://github.com' || !url.pathname.toLowerCase().startsWith(releasePath)) return []
    return [{ name: asset.name, url: url.href, size: asset.size, format, platform, architecture: architectureFor(name) }]
  })
  return {
    id: release.id,
    version: release.tag_name,
    publishedAt: release.published_at,
    prerelease: release.prerelease,
    url: `https://github.com/${repo}/releases/tag/${encodeURIComponent(release.tag_name)}`,
    packages,
  }
}

export type OpenSourceRelease = ReturnType<typeof normalizeRelease>


export async function initialSoftwareRelease(repo: string, version: string) {
  const assets: Partial<Record<string, GitHubAsset[]>> = (await import('./softwareInitialAssets.json')).default
  const files = assets[repo.toLowerCase()]
  if (!files?.length || !version) return null
  const release = normalizeRelease(repo, {
    id: version, tag_name: version, published_at: null, prerelease: false, assets: files,
  })
  return release.packages.length ? release : null
}
