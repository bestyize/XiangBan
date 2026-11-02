export type ArticleLayout = 'reading' | 'visual' | 'plain' | 'masonry'
export type PackLayout = 'visual' | 'directory' | 'masonry'

export function cardTitleLines(category = ''): 2 | 3 {
  return category === 'resource-movies' || category === 'share-movies' ? 3 : 2
}

/** An illustration does not establish that a post is a visual work. */
export function articleLayout(category: string): ArticleLayout {
  if (category === 'site' || category.startsWith('site-')) return 'plain'
  if (['resource-movies', 'resource-music', 'casual-media'].includes(category)) return 'masonry'
  return ['resource-tools', 'resource-docs', 'resource-open', 'casual-gear'].includes(category) ? 'visual' : 'reading'
}
export function packLayout(kind: string, wallpaperFeed = false): PackLayout {
  if (wallpaperFeed && kind === 'wallpaper') return 'masonry'
  return ['wallpaper', 'effect', 'widget', 'game'].includes(kind) ? 'visual' : 'directory'
}
/** Bound thumbnails; detail pages retain the original image. */
export function wallpaperRatio(width: number, height: number, minimum = 4 / 5): number {
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) return 16 / 9
  return Math.min(16 / 9, Math.max(minimum, width / height))
}
export function masonrySpan(height: number, row = 4, gap = 12): number {
  return Math.max(1, Math.ceil((height + gap) / (row + gap)))
}
