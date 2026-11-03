<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { fetchSoftwareSummaries } from '@/lib/openSourceRelease'
import { openSourceBrowse } from '@/api/openSourceBrowse'
import { openSourceCatalog, type OpenSourceProjectSummary } from '@/lib/openSource'
import { projectPath } from '@/lib/openSourcePaths'
import { softwareCategories, softwareDirectoryPreview, findSoftwareProject, softwareDownload, softwareOwnerIcon, softwareProjects, type SoftwarePlatform } from '@/lib/openSourceSoftware'
import { useCommunityResource } from './resource'
import CommunityTabs from './CommunityTabs.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import RequestState from './RequestState.vue'

const route = useRoute()
const router = useRouter()
const { data, loading, error, load } = useCommunityResource(async () => {
  if (softwareDirectoryPreview) return openSourceCatalog.projects
  const first = await openSourceBrowse.page('all', '', 1)
  const projects = [...first.projects]
  let next = first.next_page
  while (next !== null) {
    const page = await openSourceBrowse.page('all', '', next)
    projects.push(...page.projects)
    next = page.next_page
  }
  return projects
})
void load()

const { data: summaries, loading: hotLoading, error: hotError, load: loadSummaries } =
  useCommunityResource(fetchSoftwareSummaries)
void loadSummaries()

const entries = computed(() => {
  const projects = new Map(softwareProjects.map(project => [project.full_name.toLowerCase(), project as OpenSourceProjectSummary]))
  for (const project of data.value ?? []) projects.set(project.full_name.toLowerCase(), project)
  return [...projects.values()].flatMap((project: OpenSourceProjectSummary) => {
    const download = softwareDownload(project.full_name)
    if (!download) return []
    const snapshot = summaries.value?.[project.full_name.toLowerCase()]
    return [{
      project: {
        ...project,
        summary: findSoftwareProject(project.full_name)?.summary || project.summary,
        stars: snapshot?.stars ?? project.stars,
        last_pushed_at: snapshot?.updatedAt ?? project.last_pushed_at,
      },
      download,
      snapshot,
      version: snapshot ? snapshot.version : download.version,
    }]
  })
})
const activeCategory = computed(() =>
  softwareCategories.find(item => item.key === route.query.category)?.key ?? 'all',
)
const sortOptions = [
  { key: 'stars', label: '收藏数', icon: 'star' },
  { key: 'updated', label: '最近更新', icon: 'refresh' },
  { key: 'hot', label: '近期热门', icon: 'clock' },
] as const
const sort = computed(() => sortOptions.find(item => item.key === route.query.sort)?.key ?? 'stars')
const pageSize = 48
const visibleLimit = ref(pageSize)
watch([activeCategory, sort], () => { visibleLimit.value = pageSize })
const byUpdated = (a: OpenSourceProjectSummary, b: OpenSourceProjectSummary) =>
  (Date.parse(b.last_pushed_at ?? '') || 0) - (Date.parse(a.last_pushed_at ?? '') || 0)
const byStars = (a: OpenSourceProjectSummary, b: OpenSourceProjectSummary) =>
  (b.stars ?? -1) - (a.stars ?? -1)
const categoryEntries = computed(() => activeCategory.value === 'all'
  ? entries.value
  : entries.value.filter(({ download }) => download.category === activeCategory.value),
)
const visibleEntries = computed(() => [...categoryEntries.value].sort(({ project: a, snapshot: aSnapshot }, { project: b, snapshot: bSnapshot }) => {
  if (sort.value === 'hot') {
    const aGain = aSnapshot?.starGain ?? undefined
    const bGain = bSnapshot?.starGain ?? undefined
    if (aGain === undefined && bGain !== undefined) return 1
    if (bGain === undefined && aGain !== undefined) return -1
    return (bGain ?? 0) - (aGain ?? 0) || byUpdated(a, b) || byStars(a, b)
  }
  if (sort.value === 'updated') return byUpdated(a, b) || byStars(a, b)
  return byStars(a, b) || byUpdated(a, b)
}))
const hasHotData = computed(() => categoryEntries.value.some(({ snapshot }) =>
  snapshot?.starGain != null,
))

function selectSort(sort: typeof sortOptions[number]['key']) {
  void router.replace({ query: { ...route.query, sort: sort === 'stars' ? undefined : sort } })
}

function selectCategory(category: string) {
  const query = { ...route.query }
  if (category === 'all') delete query.category
  else query.category = category
  delete query.use
  delete query.platform
  void router.replace({ query })
}

const platformColors: Record<SoftwarePlatform, string> = {
  Windows: '#0078d4',
  macOS: 'var(--ink)',
  Linux: '#bf7600',
  Android: '#168651',
}
const platformIcons: Record<SoftwarePlatform, string> = {
  Windows: 'M2 4.8 10.5 3.6v7.9H2zm9.5-1.3L22 2v9.5H11.5zM2 12.5h8.5v7.9L2 19.2zm9.5 0H22V22l-10.5-1.5z',
  macOS: 'M17.1 2c.1 1.2-.4 2.3-1.1 3.1-.8.9-1.9 1.5-3 1.4-.2-1.1.4-2.3 1.1-3.1.8-.8 2-1.4 3-1.4zM20 17.8c-.7 1.6-2 4.2-3.7 4.2-1 0-1.5-.6-2.9-.6s-2 .6-3 .6c-1.7 0-3.1-2.6-3.8-4.1-1.6-3.6-1.1-7.3 1-8.8 1-.7 2.2-1 3.1-.8 1 .2 1.8.7 2.6.7.8 0 1.7-.6 2.9-.8 1.4-.2 2.8.4 3.7 1.5-3.3 1.9-3.1 6.4.1 8.1z',
  Linux: 'M12 2c-2.7 0-4.5 2.3-4.5 5.4 0 1.4-.3 2.6-1.1 4-1 1.8-2.1 3.6-2.1 5.7 0 .7.2 1.3.5 1.7L2.5 20a1 1 0 0 0 .5 1.9h5.8c.9 0 1.3-.5 1.6-1h3.2c.3.5.7 1 1.6 1H21a1 1 0 0 0 .5-1.9l-2.3-1.2c.3-.4.5-1 .5-1.7 0-2.1-1.1-3.9-2.1-5.7-.8-1.4-1.1-2.6-1.1-4C16.5 4.3 14.7 2 12 2zM10 6a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6zm4 0a.8.8 0 1 1 0 1.6.8.8 0 0 1 0-1.6zm-4.4 3.4L12 8.3l2.4 1.1-2.4 1.5zM12 12c2.4 0 3.9 2.5 3.9 4.8 0 1.8-1.5 2.6-3.9 2.6s-3.9-.8-3.9-2.6c0-2.3 1.5-4.8 3.9-4.8z',
  Android: 'm5.5 5.3 1.8 3.1A10.4 10.4 0 0 0 2 17h20a10.4 10.4 0 0 0-5.3-8.6l1.8-3.1-.9-.5-1.9 3.2a10.5 10.5 0 0 0-7.4 0L6.4 4.8zM7.6 12a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm8.8 0a1 1 0 1 1 0 2 1 1 0 0 1 0-2z',
}
const failedIcons = reactive(new Set<string>())
const starCount = new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 })
const updatedDate = new Intl.DateTimeFormat('zh-CN', { month: 'numeric', day: 'numeric', year: 'numeric', timeZone: 'Asia/Shanghai' })

function updateLabel(value?: string) {
  if (!value) return ''
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) return ''
  const days = Math.floor((Date.now() - date.getTime()) / 86_400_000)
  if (days === 0) return '今天更新'
  if (days > 0 && days < 30) return `更新于 ${days} 天前`
  return `更新于 ${updatedDate.format(date)}`
}

</script>

<template>
  <section class="software-directory" aria-label="开源软件">
    <div class="software-toolbar">
      <CommunityTabs class="software-category-tabs" :items="softwareCategories" :active="activeCategory" label="软件分类" @change="selectCategory" />
      <div class="software-sort-options" role="group" aria-label="软件排序">
        <button v-for="item in sortOptions" :key="item.key" type="button"
          :aria-pressed="sort === item.key" :title="item.key === 'stars' ? '按 GitHub Star 总数排序' : undefined"
          @click="selectSort(item.key)">
          <DiscoverGlyph :name="item.icon" :size="12" />{{ item.label }}
        </button>
      </div>
    </div>
    <p v-if="sort === 'hot' && categoryEntries.length" class="software-sort-status" role="status">
      {{ hotLoading ? '正在读取近期星标数据…' : hotError ? '近期星标数据暂不可用，按最近更新展示。' : hasHotData ? '按近一周 GitHub 星标净增排序，暂无数据的软件排在后面。' : '暂无近一周星标数据，按最近更新展示。' }}
    </p>
    <RequestState :loading="loading" :error="error" :empty="!!data && !visibleEntries.length" message="这个分类暂未收录软件。" @retry="load" />
    <ul v-if="visibleEntries.length" class="software-list">
      <li v-for="{ project, download, snapshot, version } in visibleEntries.slice(0, visibleLimit)" :key="project.full_name">
        <RouterLink :to="`${projectPath(project.full_name)}#official-download`" class="software-card">
          <div class="software-card-top">
            <span class="software-icon" aria-hidden="true">
              <img v-if="!failedIcons.has(project.full_name)"
                :src="project.icon_url || softwareOwnerIcon(project.full_name)"
                alt="" loading="lazy" referrerpolicy="no-referrer" @error="failedIcons.add(project.full_name)">
              <span v-else>{{ project.name.slice(0, 1).toUpperCase() }}</span>
            </span>
            <div class="software-identity">
              <h3 :title="project.name">{{ project.name }}</h3>
              <span :title="project.full_name">{{ project.full_name }}</span>
            </div>
            <span v-if="project.stars != null" class="software-stars" :aria-label="`${project.stars} GitHub 星标`">
              <DiscoverGlyph name="star" :size="13" />{{ starCount.format(project.stars) }}
            </span>
          </div>
          <p class="software-summary">{{ project.summary }}</p>
          <div class="software-badges">
            <span v-if="version" class="software-version" :title="snapshot ? '已同步的 GitHub 正式版本' : '收录时的 GitHub 正式版本'"><DiscoverGlyph name="tag" :size="12" />{{ version }}</span>
            <span class="software-language">{{ download.language }}</span>
          </div>
          <div class="software-card-footer">
            <div class="software-platforms">
              <span v-for="item in download.platforms" :key="item" :title="item">
                <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
                  <path :d="platformIcons[item]" :fill="platformColors[item]" fill-rule="evenodd" />
                </svg>
                <span class="sr-only">{{ item }}</span>
              </span>
            </div>
            <span v-if="updateLabel(project.last_pushed_at)" class="software-updated">
              <DiscoverGlyph name="clock" :size="12" />{{ updateLabel(project.last_pushed_at) }}
            </span>
          </div>
        </RouterLink>
      </li>
    </ul>
    <button v-if="visibleEntries.length > visibleLimit" type="button" class="software-more"
      @click="visibleLimit += pageSize">加载更多（{{ visibleLimit }} / {{ visibleEntries.length }}）</button>
  </section>
</template>

<style scoped>
.software-directory{padding:0 12px 24px}
.software-more {
  display: block;
  margin: 16px auto 0;
  padding: 8px 16px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface);
  color: var(--ink);
  cursor: pointer;
}
.software-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px 16px;
  margin-bottom: 12px;
}
.software-toolbar .software-category-tabs {
  flex: 1 1 max-content;
  min-width: 0;
  max-width: 100%;
  justify-content: flex-start;
  gap: 16px;
  margin: 0;
  padding-inline: 0;
  border-bottom: 0;
  background: none;
}
@media (max-width: 767px) {
  .software-toolbar { row-gap: 2px; }
}
.software-sort-options {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 2px;
  font-size: 12px;
  padding: 2px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--surface-2);
}
.software-sort-options button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-height: 28px;
  padding: 3px 7px;
  border: 1px solid transparent;
  border-radius: 7px;
  background: transparent;
  color: var(--ink-2);
  font: inherit;
  white-space: nowrap;
  cursor: pointer;
}
.software-sort-options button:hover { color: var(--ink); background: var(--surface); }
.software-sort-options button[aria-pressed="true"] {
  border-color: var(--line);
  background: var(--surface);
  color: var(--brand);
  box-shadow: 0 1px 2px rgb(0 0 0 / 8%);
}
.software-sort-options button:focus-visible { outline: 2px solid var(--brand); outline-offset: 1px; }
.software-sort-status { margin: 0 0 12px; color: var(--ink-3); font-size: 12px; }
.software-list{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(100%,270px),1fr));gap:14px;margin:0;padding:0;list-style:none}
.software-list li{min-width:0}
.software-card{position:relative;display:flex;flex-direction:column;height:100%;padding:16px;background:var(--surface);border:1px solid var(--line);border-radius:12px;overflow:hidden}
.software-card::before{position:absolute;inset:0 auto 0 0;width:2px;content:'';background:var(--brand);opacity:0;transition:opacity .2s}
.software-card:hover::before,.software-card:focus-visible::before{opacity:1}
.software-card-top{display:flex;align-items:flex-start;gap:10px}
.software-icon{display:grid;place-items:center;width:40px;height:40px;flex:none;border-radius:10px;background:var(--surface-2);overflow:hidden;font-weight:700}
.software-icon img{width:100%;height:100%;object-fit:contain}
.software-identity{flex:1;min-width:0}
.software-identity h3,.software-identity span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.software-identity h3{margin:0;font-size:15px;line-height:1.4}
.software-identity span{margin-top:2px;color:var(--ink-3);font:11px/1.4 ui-monospace,monospace}
.software-stars,.software-updated{display:inline-flex;align-items:center;gap:4px;flex:none;color:var(--ink-3);font-size:11px}
.software-stars{padding-top:3px;font-variant-numeric:tabular-nums}
.software-stars svg{color:#b57625;fill:currentColor}
.software-directory .software-summary{display:-webkit-box;min-height:calc(2 * 1.6em);margin:12px 0 0;overflow:hidden;-webkit-box-orient:vertical;-webkit-line-clamp:2;color:var(--ink-3);font-size:13px;line-height:1.6}
.software-badges{display:flex;flex-wrap:wrap;gap:6px;margin:10px 0 14px}
.software-badges span{display:inline-flex;align-items:center;gap:4px;padding:3px 6px;border-radius:4px;font:11px/1.4 ui-monospace,monospace}
.software-version{color:var(--brand);background:color-mix(in srgb,var(--brand) 12%,var(--surface))}
.software-language{color:var(--ink-3);background:var(--surface-2)}
.software-language::before{width:6px;height:6px;border-radius:50%;background:#3d83bd;content:''}
.software-card-footer{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:6px 10px;margin-top:auto;padding-top:10px;border-top:1px solid var(--line)}
.software-platforms{display:flex;align-items:center;flex-wrap:wrap;gap:4px 8px;color:var(--ink-3)}
.software-platforms svg{display:block}
</style>
