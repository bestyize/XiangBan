<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ApiError } from '@/api/forum'
import { openSourceBrowse } from '@/api/openSourceBrowse'
import { findProject, openSourceCatalog } from '@/lib/openSource'
import { openSourceProjectIntros } from '@/lib/openSourceProjectIntro'
import { fetchSoftwareSnapshot, initialSoftwareRelease } from '@/lib/openSourceRelease'
import { findSoftwareProject, softwareDirectoryPreview, softwareDownload, softwareOwnerIcon } from '@/lib/openSourceSoftware'
import { useCommunityResource } from '@/components/community/resource'
import { fetchProjectReadme, renderProjectReadme } from '@/lib/projectReadme'
import { useRepoBookmarks } from '@/lib/repoBookmarks'
import { useHead } from '@/lib/head'
import { curatedProjectAbout } from '@/lib/githubProjectAbout'
import RequestState from '@/components/community/RequestState.vue'
import DetailLayout from '@/components/community/DetailLayout.vue'
import { webLink } from '@/components/community/contract'
import DiscoverGlyph from '@/components/community/DiscoverGlyph.vue'
import RepoComments from '@/components/community/RepoComments.vue'
import DockButton from '@/components/comments/DockButton.vue'
import AppIcon from '@/components/AppIcon.vue'
import OpenSourceReleaseHistory from '@/components/community/OpenSourceReleaseHistory.vue'
import OpenSourceReleasePackages from '@/components/community/OpenSourceReleasePackages.vue'
import OpenSourceProjectCard from '@/components/community/OpenSourceProjectCard.vue'
import '@/components/community/community.css'
import '@/components/community/open-source.css'
import '@/components/community/project-readme.css'
const route = useRoute(), router = useRouter()
const { data: catalog, loading, error, load } = useCommunityResource(async () => {
  const owner = String(route.params.owner), repo = String(route.params.repo)
  const software = findSoftwareProject(`${owner}/${repo}`)
  if (!softwareDirectoryPreview) {
    try {
      return await openSourceBrowse.project(owner, repo)
    } catch (err) {
      if (!(err instanceof ApiError) || err.status !== 404 || !software) throw err
    }
  }
  const item = findProject(`${owner}/${repo}`) ?? software
  if (!item) throw new Error('项目暂不可用')
  const related = openSourceCatalog.projects.filter(other =>
    other !== item && (other.category === item.category || other.related_categories.includes(item.category)),
  ).slice(0, 2)
  return { categories: openSourceCatalog.categories, projects: [item], related }
})
const { data: readme, loading: readmeLoading, error: readmeError, load: loadReadme } = useCommunityResource(() => fetchProjectReadme(String(route.params.owner), String(route.params.repo)))
const language = ref<'zh' | 'en'>('zh'), article = ref<HTMLElement | null>(null), copyMessage = ref('')
const project = computed(() => catalog.value?.projects[0])
const intro = computed(() => project.value && openSourceProjectIntros[project.value.full_name.toLowerCase()])
const download = computed(() => project.value && softwareDownload(project.value.full_name))
const { data: snapshot, error: snapshotError, loading: snapshotLoading, load: loadSnapshot } = useCommunityResource(async () => {
  const name = project.value?.full_name
  const source = download.value
  if (!name || !source) return null
  try {
    return await fetchSoftwareSnapshot(name, source.releaseRepo)
  } catch (err) {
    if (err instanceof ApiError && err.status === 404 && findSoftwareProject(name)) return null
    throw err
  }
})
watch(() => project.value?.full_name, () => { void loadSnapshot() }, { immediate: true })
const { data: initialRelease, load: loadInitialRelease } = useCommunityResource(() => {
  const source = download.value
  return source ? initialSoftwareRelease(source.releaseRepo, source.version) : Promise.resolve(null)
})
watch(() => project.value?.full_name, () => { void loadInitialRelease() }, { immediate: true })
const version = computed(() => snapshot.value ? snapshot.value.version : download.value?.version)
const fromSoftware = computed(() => route.hash === '#official-download')
const hasSidebar = computed(() => !!(date(updated.value) || card.value?.license || card.value?.github_archived || card.value?.upstream_maintenance_notice || download.value))
const sections = computed(() => [
  ...(download.value ? [{ id: 'official-download', label: '最新版本' }] : []),
  { id: 'project-description', label: '项目说明' },
  ...(download.value ? [{ id: 'release-history', label: '历史版本' }] : []),
  ...(catalog.value?.related.length ? [{ id: 'related-projects', label: '相关项目' }] : []),
])
function back() {
  if (window.history.state?.back) router.back()
  else void router.push(fromSoftware.value ? '/resources?tab=software' : '/open-source')
}
function jumpToSection(event: MouseEvent, id: string) {
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
  event.preventDefault()
  const section = document.getElementById(id)
  section?.focus({ preventScroll: true })
  section?.scrollIntoView({ block: 'start' })
}
const downloadSection = ref<HTMLElement | null>(null)
const historySection = ref<HTMLElement | null>(null)
watch([downloadSection, historySection], ([download, history]) => {
  const section = [download, history].find(item => item && route.hash === '#' + item.id)
  section?.scrollIntoView({ block: 'start' })
}, { flush: 'post' })
const card = computed(() => readme.value?.card)
const summary = computed(() => {
  const software = fromSoftware.value && project.value && findSoftwareProject(project.value.full_name)
  return (software && software.summary) || (project.value && curatedProjectAbout(project.value.full_name, card.value?.summary_zh || project.value.summary)) || ''
})
const github = computed(() => card.value?.github_url || `https://github.com/${project.value?.full_name}`)
const icon = computed(() => project.value?.icon_url || card.value?.icon_source_url
  || (download.value && project.value ? softwareOwnerIcon(project.value.full_name) : undefined))
const images = computed(() => {
  const cover = webLink(project.value?.cover_url)
  return cover ? [cover] : []
})
const stars = computed(() => snapshot.value?.stars ?? card.value?.stars ?? project.value?.stars)
const updated = computed(() => snapshot.value?.updatedAt || card.value?.code_updated_at || project.value?.last_pushed_at)
const rendered = computed(() => readme.value ? renderProjectReadme(readme.value[language.value]) : '')
const bookmarks = useRepoBookmarks()
const category = computed(() => catalog.value?.categories.find(item => item.key === project.value?.category)?.label ?? '开源项目')
const date = (value?: string) => {
  if (!value) return ''
  const parsed = new Date(value)
  return Number.isNaN(parsed.getTime()) ? '' : new Intl.DateTimeFormat('sv-SE', { timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit' }).format(parsed)
}
const count = (value: number) => Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(value)
watch(() => [route.params.owner, route.params.repo], () => {
  language.value = 'zh'; copyMessage.value = ''
  void load(); void loadReadme()
}, { immediate: true })
useHead(() => ({ title: project.value ? `${project.value.name} · 开源精选` : '开源精选', description: summary.value }))
watch([rendered, project], async () => {
  await nextTick()
  article.value?.querySelectorAll('pre').forEach(pre => {
    if (pre.querySelector('.readme-copy')) return
    const button = document.createElement('button')
    button.type = 'button'; button.textContent = '复制'; button.className = 'readme-copy'
    button.setAttribute('aria-label', '复制代码'); pre.appendChild(button)
  })
})
async function articleClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  const button = target.closest<HTMLButtonElement>('.readme-copy')
  if (button) {
    try { await navigator.clipboard.writeText(button.parentElement?.querySelector('code')?.textContent ?? ''); button.textContent = '已复制' }
    catch { copyMessage.value = '无法自动复制，请选中代码复制。' }
  }
  const link = target.closest<HTMLAnchorElement>('a[href^="#"]')
  if (link) {
    event.preventDefault()
    let id = link.getAttribute('href')!.slice(1)
    try { id = decodeURIComponent(id) } catch { /* Keep the upstream anchor. */ }
    const element = Array.from(article.value?.querySelectorAll('[id],a[name]') ?? []).find(node => node.id === id || node.getAttribute('name') === id)
    element?.scrollIntoView({ block: 'start' })
  }
}
</script>
<template>
  <DetailLayout :key="route.path" :ready="!!project" :title="card?.name || project?.name"
    :author="project?.full_name" :author-image="icon" :images="images" :badge="category"
    shareable @back="back">
    <template #state>
      <RequestState v-if="loading || error" :loading="loading" :error="error" @retry="load" />
      <section v-else class="os-empty"><h1>这个项目尚未收录</h1><RouterLink to="/open-source">返回精选</RouterLink></section>
    </template>
    <template v-if="project" #default>
      <p v-if="summary" class="xhs-prose project-summary">{{ summary }}</p>
      <div class="xhs-hashtags">
        <span>#{{ category }}</span><span v-for="tag in project.tags.filter(tag => tag !== category)" :key="tag">#{{ tag }}</span>
      </div>
      <a v-if="download" href="#official-download" class="xhs-info-strip" @click="jumpToSection($event, 'official-download')">
        <span class="xhs-strip-tag"><AppIcon name="download" :size="12" /> 下载</span>
        <span class="xhs-strip-text">查看官方安装包{{ version ? ' · ' + version : '' }}</span><AppIcon name="arrow" :size="13" />
      </a>
      <a :href="github" class="xhs-info-strip" target="_blank" rel="noopener noreferrer">
        <span class="xhs-strip-tag"><DiscoverGlyph name="github" :size="13" /> GitHub</span>
        <span class="xhs-strip-text">{{ project.full_name }}</span><AppIcon name="arrow" :size="13" />
      </a>
      <a v-if="card?.primary_url && card.primary_url !== github" :href="card.primary_url"
        class="xhs-info-strip" target="_blank" rel="noopener noreferrer">
        <span class="xhs-strip-tag"><AppIcon name="external" :size="13" /> {{ card.primary_label || '项目官网' }}</span>
        <span class="xhs-strip-text">{{ card.primary_url }}</span><AppIcon name="arrow" :size="13" />
      </a>
      <div class="xhs-meta-row project-meta">
        <time v-if="date(updated)" :datetime="updated">更新于 {{ date(updated) }}</time>
        <span v-if="stars != null" :title="`${stars.toLocaleString()} Stars · 采集于 ${date(snapshot?.syncedAt || card?.stars_fetched_at) || '目录采集日'}`">{{ count(stars) }} GitHub Stars</span>
        <span v-if="card?.icon_attribution" class="readme-icon-credit">{{ card.icon_attribution }}</span>
      </div>
      <div class="project-layout" :class="{ 'with-sidebar': hasSidebar }">
        <div class="project-main">
          <nav class="project-section-nav" aria-label="项目详情章节">
            <a v-for="section in sections" :key="section.id" :href="`#${section.id}`" @click="jumpToSection($event, section.id)">{{ section.label }}</a>
          </nav>
          <section v-if="download" id="official-download" ref="downloadSection" tabindex="-1" class="project-panel" aria-labelledby="download-heading">
            <div class="project-panel-heading">
              <h2 id="download-heading">最新版本</h2>
              <span class="project-source"><DiscoverGlyph name="github" :size="15" />GitHub 官方</span>
            </div>
            <div class="project-panel-body">
              <OpenSourceReleasePackages :data="snapshot ? snapshot.latest : initialRelease" :synced="!!snapshot" :loading="snapshotLoading" :error="snapshotError" :latest="download.latest" :platforms="download.platforms" @retry="loadSnapshot" />
            </div>
          </section>
          <section id="project-description" tabindex="-1" class="project-panel readme-section" aria-labelledby="project-description-heading">
            <div class="project-panel-heading">
              <h2 id="project-description-heading">项目说明</h2>
              <div v-if="readme && !intro" class="readme-languages" aria-label="正文语言"><button :aria-pressed="language === 'zh'" @click="language = 'zh'">中文</button><span aria-hidden="true">|</span><button :aria-pressed="language === 'en'" @click="language = 'en'">English</button></div>
            </div>
            <div class="project-panel-body">
              <div v-if="intro" class="project-intro" lang="zh-CN">
                <p>{{ intro.purpose }}</p>
                <dl>
                  <div><dt>适合场景</dt><dd>{{ intro.scenarios }}</dd></div>
                  <div><dt>怎么上手</dt><dd>{{ intro.start }}</dd></div>
                  <div><dt>关键门槛</dt><dd>{{ intro.requirement }}</dd></div>
                </dl>
                <a :href="intro.link.url" target="_blank" rel="noopener noreferrer">{{ intro.link.label }} <DiscoverGlyph name="external" :size="13" /></a>
              </div>
              <div v-if="intro" class="readme-toolbar">
                <h3>README</h3>
                <div v-if="readme" class="readme-languages" aria-label="正文语言">
                  <button :aria-pressed="language === 'zh'" @click="language = 'zh'">中文</button>
                  <span aria-hidden="true">|</span>
                  <button :aria-pressed="language === 'en'" @click="language = 'en'">English</button>
                </div>
              </div>
        <RequestState v-if="readmeLoading || readmeError" :loading="readmeLoading" :error="readmeError" @retry="loadReadme" />
        <template v-else-if="readme"><article ref="article" class="project-markdown" :lang="language === 'zh' ? 'zh-CN' : 'en'" @click="articleClick" v-html="rendered" /><p v-if="copyMessage" role="status">{{ copyMessage }}</p><footer class="readme-provenance"><p>{{ typeof readme.translation === 'string' ? readme.translation : '依据项目官方 README 整理' }}</p><p><a :href="readme.source_url" target="_blank" rel="noopener noreferrer">查看上游原文 ↗</a><span> · </span><a :href="`${github}/commit/${readme.commit}`" target="_blank" rel="noopener noreferrer">版本 {{ readme.commit.slice(0, 7) }}</a><span> · 资料采集 {{ date(readme.fetched_at) }}</span></p><details><summary>许可与版权声明</summary><section v-for="license in readme.licenses" :key="license.name"><h3>{{ license.name }}</h3><pre>{{ license.text }}</pre></section></details></footer></template>
        <div v-else class="readme-unavailable"><p v-if="download && summary">{{ summary }}</p><p>中文 README 正在整理中，可以先阅读项目官方原文。</p><a :href="project.source_url" target="_blank" rel="noopener noreferrer">查看官方 README ↗</a><img v-if="project.cover_url" :src="project.cover_url" :alt="`${project.name} 官方项目展示`" loading="lazy"></div>
            </div>
          </section>
          <section v-if="download" id="release-history" ref="historySection" tabindex="-1" class="project-panel" aria-labelledby="release-history-heading">
            <div class="project-panel-heading"><h2 id="release-history-heading">历史版本</h2></div>
            <div class="project-panel-body">
              <OpenSourceReleaseHistory :releases="snapshot?.history" :loading="snapshotLoading" :error="snapshotError" :releases-url="download.releases" :platforms="download.platforms" @retry="loadSnapshot" />
            </div>
          </section>
          <section v-if="catalog?.related.length" id="related-projects" tabindex="-1" class="os-related">
            <h2>相关项目</h2>
            <div class="os-project-grid"><OpenSourceProjectCard v-for="item in catalog.related.slice(0, 4)" :key="item.full_name" :project="item" /></div>
          </section>
        </div>
        <aside v-if="hasSidebar" class="project-sidebar" aria-label="项目资料">
          <section v-if="date(updated) || card?.github_archived || card?.upstream_maintenance_notice" class="project-panel project-fact">
            <h2>项目维护</h2>
            <p v-if="date(updated)">仓库更新于 {{ date(updated) }}</p>
            <p v-if="card?.github_archived || card?.upstream_maintenance_notice" class="readme-maintenance">{{ card?.upstream_maintenance_notice || '上游仓库已归档。' }}</p>
          </section>
          <section v-if="card?.license" class="project-panel project-fact">
            <h2>开源协议</h2><p>{{ card.license }}</p>
          </section>
          <section v-if="download" class="project-panel project-fact">
            <h2>开发语言</h2><p>{{ download.language }}</p>
          </section>
        </aside>
      </div>
      <p v-if="bookmarks.error.value" class="readme-save-error" role="alert">{{ bookmarks.error.value }}</p>
    </template>
    <template v-if="project" #comments>
      <RepoComments detail :key="project.full_name" :owner="project.full_name.split('/')[0]!" :repo="project.full_name.split('/')[1]!">
        <template #actions>
          <DockButton detail :class="{ 'is-saved': bookmarks.has(project.full_name) }"
            :label="bookmarks.has(project.full_name) ? '取消收藏' : '收藏'"
            :text="bookmarks.has(project.full_name) ? '已收藏' : '收藏'"
            :active="bookmarks.has(project.full_name)" title="收藏保存在当前浏览器" @click="bookmarks.toggle(project.full_name)">
            <AppIcon name="star" :size="19" />
          </DockButton>
        </template>
      </RepoComments>
    </template>
  </DetailLayout>
</template>

<style scoped>
.project-summary { margin: 0; }
.project-meta { justify-content: flex-start; flex-wrap: wrap; }
.project-source { border: 1px solid var(--line); border-radius: 7px; padding: 3px 8px; background: var(--surface-2); color: var(--ink-2); font-size: 12px; }
.project-layout { display: grid; gap: 20px; margin-top: 18px; }
.project-main, .project-sidebar { min-width: 0; }
.project-main, .project-sidebar { display: grid; align-content: start; gap: 14px; }
.project-section-nav { display: flex; gap: 18px; overflow-x: auto; border-bottom: 1px solid var(--line); }
.project-section-nav a { flex: none; padding: 10px 0; color: var(--ink-2); font-size: 13px; }
.project-section-nav a:hover, .project-section-nav a:focus-visible { color: var(--brand); }
.project-panel { min-width: 0; background: var(--surface); border: 1px solid var(--line); border-radius: 14px; margin: 0; }
.project-panel-heading { display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap; padding: 12px 16px; border-bottom: 1px solid var(--line); }
.project-panel-heading h2, .project-fact h2 { margin: 0; font-size: 15px; }
.project-panel-body, .project-fact { padding: 12px 16px; }
.project-fact p { margin: 8px 0 0; font-size: 13px; color: var(--ink-2); }
.project-source { display: inline-flex; align-items: center; gap: 5px; }
.project-main section[id] { scroll-margin-top: 80px; }
@media (min-width: 880px) {
  .project-layout.with-sidebar { grid-template-columns: minmax(0, 1fr) minmax(200px, 25%); }
}
</style>
