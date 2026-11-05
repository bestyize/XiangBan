import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/api/forum'
import type { Category } from '@/api/types'
import { CATEGORY_ALIASES, activeCategories } from '@/lib/communityStructure'

export interface CategoryNode extends Category {
  children: Category[]
}

/** 帖子数达到这个值的二级分类才在导航里单独露出，其余合并进一级分类。 */
export const SHOW_CHILD_THRESHOLD = 20

export const useMeta = defineStore('meta', () => {
  const categories = ref<Category[]>([])
  let loading: Promise<void> | null = null

  const tree = computed<CategoryNode[]>(() => {
    const current = activeCategories(categories.value)
    const roots = current.filter(c => c.parent_id === null)
    return roots.map(r => ({ ...r, children: current.filter(c => c.parent_id === r.id) }))
  })

  const allTree = computed<CategoryNode[]>(() => categories.value.filter(c=>c.parent_id===null).map(r=>({...r,children:categories.value.filter(c=>c.parent_id===r.id)})))

  const bySlug = computed(() => {
    const map = new Map(categories.value.map(c => [c.slug, c]))
    for (const [old, name] of Object.entries(CATEGORY_ALIASES)) { const category=map.get(name); if(category) map.set(old,category) }
    return map
  })
  const byId = computed(() => new Map(categories.value.map(c => [c.id, c])))

  function load(force = false) {
    if (loading && !force) return loading
    loading = api
      .categories()
      .then(c => {
        categories.value = c
      })
      .catch(() => {
        loading = null
      })
    return loading
  }

  function parentOf(c: Category | undefined) {
    if (!c || c.parent_id === null) return c
    return byId.value.get(c.parent_id)
  }

  /** 一级分类下帖子总数（含子分类）。 */
  function totalCount(node: CategoryNode) {
    return node.topic_count + node.children.reduce((n, c) => n + c.topic_count, 0)
  }

  return { categories, tree, allTree, bySlug, byId, load, parentOf, totalCount }
})
