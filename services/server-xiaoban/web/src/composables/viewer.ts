import { shallowRef, type Ref } from 'vue'
import type { Router } from 'vue-router'
import type { WallpaperItem } from '@/types'

export interface ViewerSource {
  items: Readonly<Ref<readonly WallpaperItem[]>>
  loadMore?: () => Promise<void>
}

/** The list the viewer swipes through. It is the live feed ref, so pages loaded while viewing extend it. */
export const viewerSource = shallowRef<ViewerSource | null>(null)

let openedByPush = false

/** The viewer lives in the `?w=<id>` query so the page underneath keeps its state and scroll position. */
export function openViewer(router: Router, source: ViewerSource, item: WallpaperItem) {
  viewerSource.value = source
  openedByPush = true
  router.push({ query: { ...router.currentRoute.value.query, w: item.id } })
}

export function stepViewer(router: Router, id: string) {
  router.replace({ query: { ...router.currentRoute.value.query, w: id } })
}

export function closeViewer(router: Router) {
  if (openedByPush) {
    openedByPush = false
    router.back()
    return
  }
  const { w: _w, ...rest } = router.currentRoute.value.query
  router.replace({ query: rest })
}
