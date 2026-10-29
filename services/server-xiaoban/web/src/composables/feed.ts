import { ref, shallowRef, watch } from 'vue'
import { searchWallpapers } from '@/api'
import type { FeedParams, WallpaperItem } from '@/types'

/** Paginated, filter-aware wallpaper list. Changing the params resets it; stale responses are dropped. */
export function useFeed(params: () => FeedParams, perPage = 24, enabled: () => boolean = () => true) {
  const items = shallowRef<WallpaperItem[]>([])
  const loading = ref(false)
  const loadingMore = ref(false)
  const error = ref<string | null>(null)
  const hasMore = ref(true)

  let page = 0
  let generation = 0
  let controller: AbortController | null = null

  async function fetchPage(nextPage: number, append: boolean) {
    const mine = generation
    controller = new AbortController()
    try {
      const res = await searchWallpapers(params(), nextPage, perPage, controller.signal)
      if (mine !== generation) return
      const seen = new Set(items.value.map((i) => i.id))
      const fresh = res.items.filter((i) => !seen.has(i.id))
      items.value = append ? [...items.value, ...fresh] : fresh
      page = nextPage
      hasMore.value = res.has_more && res.items.length > 0
      error.value = null
    } catch (e) {
      if (mine !== generation || (e instanceof DOMException && e.name === 'AbortError')) return
      error.value = e instanceof Error ? e.message : '加载失败'
    } finally {
      if (mine === generation) {
        loading.value = false
        loadingMore.value = false
      }
    }
  }

  async function reload() {
    generation++
    controller?.abort()
    items.value = []
    page = 0
    hasMore.value = true
    error.value = null
    loadingMore.value = false
    if (!enabled()) {
      loading.value = false
      hasMore.value = false
      return
    }
    loading.value = true
    await fetchPage(1, false)
  }

  async function loadMore() {
    if (loading.value || loadingMore.value || !hasMore.value) return
    loadingMore.value = true
    await fetchPage(page + 1, true)
  }

  watch(() => JSON.stringify([params(), enabled()]), reload, { immediate: true })

  return { items, loading, loadingMore, error, hasMore, reload, loadMore }
}
