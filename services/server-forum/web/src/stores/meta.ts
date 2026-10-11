import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '@/api/forum'
import type { Category, Product } from '@/api/types'

export interface CategoryNode extends Category {
  children: Category[]
}

/** 帖子数达到这个值的二级分类才在导航里单独露出，其余合并进一级分类。 */
export const SHOW_CHILD_THRESHOLD = 20

export const useMeta = defineStore('meta', () => {
  const categories = ref<Category[]>([])
  const products = ref<Product[]>([])
  let loading: Promise<void> | null = null

  const tree = computed<CategoryNode[]>(() => {
    const roots = categories.value.filter(c => c.parent_id === null)
    return roots.map(r => ({ ...r, children: categories.value.filter(c => c.parent_id === r.id) }))
  })

  const bySlug = computed(() => new Map(categories.value.map(c => [c.slug, c])))
  const byId = computed(() => new Map(categories.value.map(c => [c.id, c])))

  function load(force = false) {
    if (loading && !force) return loading
    loading = Promise.all([api.categories(), api.products()])
      .then(([c, p]) => {
        categories.value = c
        products.value = p
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

  return { categories, products, tree, bySlug, byId, load, parentOf, totalCount }
})
