<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { openSourceCategories, filterProjects } from '@/lib/openSource'
import { openSourceCatalog as catalog, loadOpenSourceCatalog } from '@/stores/openSource'
import DiscoverGlyph from './DiscoverGlyph.vue'
import RepoBoard from './RepoBoard.vue'
import OpenSourceProjectCard from './OpenSourceProjectCard.vue'
import OpenSourceCollectionCard from './OpenSourceCollectionCard.vue'
import './open-source.css'
const route = useRoute(), router = useRouter()
void loadOpenSourceCatalog()
const category = computed(() => openSourceCategories.some(item => item.key === route.query.category) ? String(route.query.category) : 'recommend')
const currentCategory = computed(() => openSourceCategories.find(item => item.key === category.value)!)
const tag = computed(() => String(route.query.tag || ''))
const categoryProjects = computed(() => filterProjects(catalog.projects, category.value))
const tags = computed(() => [...new Set(categoryProjects.value.flatMap(item => item.tags))].slice(0, 8))
const projects = computed(() => filterProjects(catalog.projects, category.value, '', tag.value))
const showCollections = computed(() => category.value === 'recommend' && !tag.value)
const categoriesElement = ref<HTMLElement>()
watch(category, async () => {
  await nextTick()
  categoriesElement.value?.querySelector('[aria-current="page"]')?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
}, { immediate: true })
function update(values: Record<string, string | undefined>) { void router.replace({ query: { ...route.query, ...values, page: undefined } }) }
function selectCategory(value: string) { update({ category: value === 'recommend' ? undefined : value, tag: undefined, q: undefined, saved: undefined }) }
function reset() { update({ tag: undefined, q: undefined, saved: undefined }) }
</script>
<template>
  <section class="os-feed" aria-label="精选">
    <h2 class="sr-only">精选</h2>
    <nav ref="categoriesElement" class="os-category-tabs" aria-label="开源项目分类"><button v-for="item in openSourceCategories" :key="item.key" type="button" :aria-current="category === item.key ? 'page' : undefined" @click="selectCategory(item.key)">{{ item.label }}</button></nav>

    <RepoBoard v-if="category === 'hot'" />
    <template v-else>
    <div v-if="category !== 'recommend' && tags.length" class="os-tag-filters" aria-label="细分标签"><button type="button" :aria-pressed="!tag" @click="update({tag: undefined})">全部</button><button v-for="item in tags" :key="item" type="button" :aria-pressed="tag === item" @click="update({tag: tag === item ? undefined : item})">{{ item }}</button></div>
    <div class="os-feed-heading"><h3 class="sr-only">{{ currentCategory.label }}</h3><span>{{ projects.length }} 个项目</span></div>
    <div v-if="projects.length" class="os-project-grid">
      <template v-for="(project, index) in projects" :key="project.full_name"><OpenSourceProjectCard :project="project" /><OpenSourceCollectionCard v-if="showCollections && index === 3 && catalog.collections[0]" :collection="catalog.collections[0]" /><OpenSourceCollectionCard v-if="showCollections && index === 7 && catalog.collections[1]" :collection="catalog.collections[1]" /></template>
    </div>
    <div v-else class="os-empty" role="status"><DiscoverGlyph name="code" :size="24" /><h3>这个分类的项目正在整理中</h3><p>可以先看看其他分类。</p><button v-if="tag" type="button" class="os-button" @click="reset">清除标签筛选</button></div>
    <footer class="os-feed-note"><span>根据项目公开资料整理 · {{ catalog.reviewed_at }}</span><RouterLink to="/c/resource-open">社区分享 →</RouterLink></footer>
    </template>
  </section>
</template>
