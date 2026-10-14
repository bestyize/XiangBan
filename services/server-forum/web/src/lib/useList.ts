import { ref, watch, type WatchSource } from 'vue'
import type { Page, TopicItem } from '@/api/types'
import { useToast } from '@/stores/toast'

/** 带分页和加载状态的帖子列表；sources 变化时回到第一页重新加载。 */
export function useTopicList(fetcher: (page: number) => Promise<Page<TopicItem>>, sources: WatchSource[]) {
  const toast = useToast()
  const items = ref<TopicItem[]>([])
  const page = ref(1)
  const total = ref(0)
  const pageSize = ref(30)
  const loading = ref(false)
  let seq = 0

  async function load(p = 1) {
    const mine = ++seq
    loading.value = true
    try {
      const res = await fetcher(p)
      if (mine !== seq) return
      items.value = res.items
      page.value = res.page
      total.value = res.total
      pageSize.value = res.page_size
    } catch (e) {
      if (mine === seq) toast.error(e)
    } finally {
      if (mine === seq) loading.value = false
    }
  }

  function go(p: number) {
    load(p)
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  watch(sources, () => {
    // 切换标签、用户或登录身份时，先清掉上一份结果，避免短暂展示旧身份可见的帖子。
    items.value = []
    total.value = 0
    load(1)
  }, { immediate: true })

  return { items, page, total, pageSize, loading, load, go }
}

