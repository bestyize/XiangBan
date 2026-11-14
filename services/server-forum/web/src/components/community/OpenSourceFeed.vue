<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { openSourceBrowse, type ProjectPage } from '@/api/openSourceBrowse'
import { previewOpenSourceCover } from '@/lib/openSourceCoverPreview'
import RequestState from './RequestState.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import RepoBoard from './RepoBoard.vue'
import MasonryGrid from './MasonryGrid.vue'
import OpenSourceProjectCard from './OpenSourceProjectCard.vue'
import OpenSourceCollectionCard from './OpenSourceCollectionCard.vue'
import './open-source.css'
const route = useRoute(), router = useRouter()
const category = computed(() => typeof route.query.category === 'string' ? route.query.category : 'recommend')
const tag = computed(() => typeof route.query.tag === 'string' ? route.query.tag : '')
const filterKey = computed(() => JSON.stringify([category.value, tag.value]))
const pages = reactive(new Map<string, ProjectPage>())
const pending = reactive(new Set<string>())
const errors = reactive(new Map<string, string>())
const categories = ref<Pick<ProjectPage['categories'][number], 'key' | 'label'>[]>([
  { key: 'ai', label: 'AI 与智能体' },
  { key: 'skills', label: 'Skill 与 MCP' },
  { key: 'office', label: '办公效率' },
  { key: 'design', label: '设计创作' },
  { key: 'media', label: '影音媒体' },
  { key: 'dev', label: '开发工具' },
  { key: 'learn', label: '教程资源' },
  { key: 'hardware', label: '开源硬件' },
  { key: 'security', label: '安全逆向' },
  { key: 'fun', label: '趣味探索' },
  { key: 'web', label: '建站应用' },
])
const tabs = computed(() => [{ key: 'recommend', label: '推荐' }, { key: 'hot', label: '热榜' }, ...categories.value])
const catalog = computed(() => pages.get(filterKey.value))
const coverPreview = computed(() => import.meta.env.DEV && route.query.coverPreview === '1')
const projects = computed(() => {
  const items = catalog.value?.projects ?? []
  return coverPreview.value ? items.map(previewOpenSourceCover) : items
})
const tags = computed(() => catalog.value?.tags ?? [])
const loading = computed(() => pending.has(filterKey.value))
const error = computed(() => errors.get(filterKey.value) ?? '')
const showCollections = computed(() => category.value === 'recommend' && !tag.value)
const sentinel = ref<HTMLElement>()

async function load() {
  const key = filterKey.value
  const selectedCategory = category.value, selectedTag = tag.value
  const previous = pages.get(key)
  if (selectedCategory === 'hot' || pending.has(key) || previous?.next_page === null) return
  pending.add(key)
  errors.delete(key)
  try {
    let next = await openSourceBrowse.page(selectedCategory, selectedTag, previous?.next_page ?? 1)
    if (previous && next.revision !== previous.revision) {
      next = await openSourceBrowse.page(selectedCategory, selectedTag, 1)
    } else if (previous) {
      const seen = new Set(previous.projects.map(item => item.full_name.toLowerCase()))
      next = { ...next, projects: [...previous.projects, ...next.projects.filter(item => !seen.has(item.full_name.toLowerCase()))], collections: previous.collections }
    }
    pages.set(key, next)
    if (next.categories.length !== categories.value.length || next.categories.some((item, index) =>
      item.key !== categories.value[index]?.key || item.label !== categories.value[index]?.label,
    )) {
      categories.value = next.categories
    }
  } catch (e) {
    errors.set(key, e instanceof Error ? e.message : '加载失败，请重试')
  } finally {
    pending.delete(key)
  }
}

watch(filterKey, () => { if (!catalog.value) void load() }, { immediate: true })
watch([sentinel, loading, error], ([element, busy, failure], _, cleanup) => {
  if (!element || busy || failure || typeof IntersectionObserver === 'undefined') return
  const observer = new IntersectionObserver(entries => {
    if (entries.some(entry => entry.isIntersecting)) void load()
  })
  observer.observe(element)
  cleanup(() => observer.disconnect())
}, { flush: 'post' })
const categoriesElement = ref<HTMLElement>()
watch([category, tabs], async () => {
  await nextTick()
  categoriesElement.value?.querySelector('[aria-current="page"]')?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
}, { immediate: true })
function update(values: Record<string, string | undefined>) { void router.replace({ query: { ...route.query, ...values, page: undefined } }) }
function selectCategory(value: string) { update({ category: value === 'recommend' ? undefined : value, tag: undefined, q: undefined, saved: undefined }) }
function reset() { update({ tag: undefined, q: undefined, saved: undefined }) }
</script>
<template>
  <section class="os-feed" :class="{ 'os-feed--cover-preview': coverPreview }" aria-label="精选">
    <h2 class="sr-only">精选</h2>
    <nav ref="categoriesElement" class="os-category-tabs" aria-label="开源项目分类"><button v-for="item in tabs" :key="item.key" type="button" :aria-current="category === item.key ? 'page' : undefined" @click="selectCategory(item.key)">{{ item.label }}</button></nav>

    <RepoBoard v-if="category === 'hot'" />
    <template v-else>
    <div v-if="category !== 'recommend' && tags.length" class="os-tag-filters" aria-label="细分标签"><button type="button" :aria-pressed="!tag" @click="update({tag: undefined})">全部</button><button v-for="item in tags" :key="item" type="button" :aria-pressed="tag === item" @click="update({tag: tag === item ? undefined : item})">{{ item }}</button></div>
    <div v-if="catalog" class="os-feed-heading"><h3 class="sr-only">{{ tabs.find(item => item.key === category)?.label ?? "精选" }}</h3><span>{{ catalog.total }} 个项目</span></div>
    <MasonryGrid v-if="projects.length" class="os-project-grid">
      <template v-for="(project, index) in projects" :key="project.full_name"><OpenSourceProjectCard :project="project" :cover-preview="coverPreview" /><OpenSourceCollectionCard v-if="showCollections && index === 3 && catalog?.collections[0]" :collection="catalog?.collections[0]" /><OpenSourceCollectionCard v-if="showCollections && index === 7 && catalog?.collections[1]" :collection="catalog?.collections[1]" /></template>
    </MasonryGrid>
    <div v-else-if="catalog && !loading" class="os-empty" role="status"><DiscoverGlyph name="code" :size="24" /><h3>这个分类的项目正在整理中</h3><p>可以先看看其他分类。</p><button v-if="tag" type="button" class="os-button" @click="reset">清除标签筛选</button></div>
    <RequestState v-if="loading || error" :loading="loading" :error="error" @retry="load" />
    <div v-if="catalog?.next_page && !error" ref="sentinel" class="os-feed-note">
      <button type="button" class="os-button" :disabled="loading" @click="load">加载更多</button>
    </div>
    <footer v-if="catalog" class="os-feed-note"><span>根据项目公开资料整理 · {{ catalog.reviewed_at }}</span></footer>
    </template>
  </section>
</template>
