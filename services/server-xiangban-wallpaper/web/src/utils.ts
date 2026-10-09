import type { WallpaperItem } from './types'

const PROVIDER_NAMES: Record<string, string> = {
  unsplash: 'Unsplash',
  pixabay: 'Pixabay',
  pexels: 'Pexels',
  okslop: 'okslop',
  demo: '精选',
}

export const providerName = (id: string) => PROVIDER_NAMES[id] ?? id

/** Tile ratio (height / width), clamped so extreme panoramas and tall strips stay browsable. */
export function tileRatio(item: WallpaperItem): number {
  const { width, height } = item.resolution
  if (!width || !height) return 0.66
  return Math.min(Math.max(height / width, 0.5), 2)
}

export function cardImage(item: WallpaperItem): string {
  // 瀑布流卡片优先加载轻量缩略图（small / thumb / preview_url），大幅降低网络带宽消耗并提升首屏渲染速度
  return item.downloads.small ?? item.downloads.thumb ?? item.preview_url ?? item.downloads.medium ?? item.regular_url
}

/**
 * Image for full-size viewing. The medium rendition is about 1000 to 1300 px wide, which is soft once it
 * fills a high-density screen, so anything wider than that gets the large one (capped near 2560 px by the server).
 * Read at call time so rotating a phone or resizing the window is taken into account.
 */
export function viewImage(item: WallpaperItem): string {
  const deviceWidth = window.innerWidth * (window.devicePixelRatio || 1)
  return deviceWidth > 1400 ? (item.downloads.large ?? item.downloads.original) : (item.downloads.medium ?? item.regular_url)
}

/** Wallpaper's own color when the provider sends one, otherwise a stable muted hue derived from its id. */
export function colorOf(item: WallpaperItem): string {
  if (item.dominant_color) return item.dominant_color
  let hash = 0
  for (const ch of item.id) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0
  return `hsl(${hash % 360} 28% 34%)`
}

/** Short badge for a card: video length, otherwise a resolution tier worth advertising. */
export function badgeOf(item: WallpaperItem): string | null {
  if (item.media_type === 'video' && item.video) return formatDuration(item.video.duration_secs)
  const longSide = Math.max(item.resolution.width, item.resolution.height)
  if (longSide >= 7680) return '8K'
  if (longSide >= 3840) return '4K'
  return null
}

export function formatDuration(secs: number): string {
  const m = Math.floor(secs / 60)
  const s = Math.floor(secs % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}

export function orientationLabel(item: WallpaperItem): string {
  return { landscape: '横屏', portrait: '竖屏', squarish: '方形', all: '' }[item.orientation]
}

function extensionOf(url: string, fallback: string): string {
  try {
    const match = new URL(url).pathname.match(/\.([a-z0-9]{2,4})$/i)
    return match ? match[1].toLowerCase() : fallback
  } catch {
    return fallback
  }
}

export type DownloadOutcome = 'saved' | 'opened' | 'blocked'

/**
 * Cross-origin `<a download>` is ignored by browsers, so fetch the file as a blob first.
 * If that fails (the host refuses CORS, or the link is dead), open the link in a new tab so the user can
 * save it by hand. The outcome says which happened, because a blocked pop-up looks like "nothing happened".
 */
export async function downloadFile(url: string, item: WallpaperItem): Promise<DownloadOutcome> {
  const ext = extensionOf(url, item.media_type === 'video' ? 'mp4' : 'jpg')
  const filename = `${item.provider}-${item.source_id}.${ext}`
  try {
    const res = await fetch(url, { mode: 'cors' })
    if (!res.ok) throw new Error(String(res.status))
    const blobUrl = URL.createObjectURL(await res.blob())
    const a = document.createElement('a')
    a.href = blobUrl
    a.download = filename
    document.body.appendChild(a)
    a.click()
    a.remove()
    setTimeout(() => URL.revokeObjectURL(blobUrl), 10_000)
    return 'saved'
  } catch {
    return window.open(url, '_blank', 'noopener') ? 'opened' : 'blocked'
  }
}
