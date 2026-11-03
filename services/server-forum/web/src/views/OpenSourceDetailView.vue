<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { openSourceBrowse } from '@/api/openSourceBrowse'
import { useCommunityResource } from '@/components/community/resource'
import { fetchProjectReadme, renderProjectReadme } from '@/lib/projectReadme'
import { useRepoBookmarks } from '@/lib/repoBookmarks'
import { useHead } from '@/lib/head'
import RequestState from '@/components/community/RequestState.vue'
import CommunityBack from '@/components/community/CommunityBack.vue'
import DiscoverGlyph from '@/components/community/DiscoverGlyph.vue'
import RepoComments from '@/components/community/RepoComments.vue'
import OpenSourceProjectCard from '@/components/community/OpenSourceProjectCard.vue'
import '@/components/community/community.css'
import '@/components/community/open-source.css'
import '@/components/community/project-readme.css'
const route = useRoute()
const { data: catalog, loading, error, load } = useCommunityResource(() => openSourceBrowse.project(String(route.params.owner), String(route.params.repo)))
const { data: readme, loading: readmeLoading, error: readmeError, load: loadReadme } = useCommunityResource(() => fetchProjectReadme(String(route.params.owner), String(route.params.repo)))
const language = ref<'zh' | 'en'>('zh'), brokenIcon = ref(false), article = ref<HTMLElement | null>(null), copyMessage = ref('')
const project = computed(() => catalog.value?.projects[0])
const card = computed(() => readme.value?.card)
const summary = computed(() => card.value?.summary_zh || project.value?.summary || '')
const github = computed(() => card.value?.github_url || `https://github.com/${project.value?.full_name}`)
const icon = computed(() => project.value?.icon_url || card.value?.icon_source_url)
const stars = computed(() => card.value?.stars ?? project.value?.stars)
const updated = computed(() => card.value?.code_updated_at || project.value?.last_pushed_at)
const rendered = computed(() => readme.value ? renderProjectReadme(readme.value[language.value]) : '')
const bookmarks = useRepoBookmarks()
const category = computed(() => catalog.value?.categories.find(item => item.key === project.value?.category)?.label ?? '开源项目')
const date = (value?: string) => value ? value.slice(0, 10) : ''
const count = (value: number) => Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(value)
watch(() => [route.params.owner, route.params.repo], () => {
  language.value = 'zh'; brokenIcon.value = false; copyMessage.value = ''
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
  <main class="community-page readme-page">
    <nav class="readme-nav"><CommunityBack fallback="/open-source" label="精选" /><span>开源精选</span><a v-if="project" :href="github" target="_blank" rel="noopener noreferrer" aria-label="在 GitHub 查看项目"><DiscoverGlyph name="github" :size="20" /></a></nav>
    <template v-if="project">
      <header class="readme-title"><p>{{ category }}<span> · </span>OPEN SOURCE</p><h1>{{ project.name }}<span> — {{ summary.replace(/[。.]$/, '') }}</span></h1></header>
      <section class="readme-project-card" aria-label="项目信息">
        <div class="readme-project-top"><div class="readme-project-icon"><img v-if="icon && !brokenIcon" :src="icon" :alt="`${project.name} 图标`" referrerpolicy="no-referrer" @error="brokenIcon = true"><span v-else>{{ project.name.slice(0, 1) }}</span></div><div><h2>{{ card?.name || project.name }}</h2><a :href="github" target="_blank" rel="noopener noreferrer">{{ project.full_name }}<DiscoverGlyph name="external" :size="12" /></a></div></div>
        <p class="readme-project-summary">{{ summary }}</p>
        <div class="readme-project-meta"><span v-if="stars != null" :title="`${stars.toLocaleString()} Stars · 采集于 ${date(card?.stars_fetched_at) || '目录采集日'}`"><DiscoverGlyph name="star" :size="15" />{{ count(stars) }} Stars</span><span v-if="updated"><DiscoverGlyph name="clock" :size="14" />更新于 {{ date(updated) }}</span><span v-if="card?.license">{{ card.license }}</span><span v-if="card?.platforms.length">{{ card.platforms.join(' / ') }}</span></div>
        <div class="readme-project-links"><a v-if="card?.primary_url && card.primary_url !== github" class="readme-primary" :href="card.primary_url" target="_blank" rel="noopener noreferrer">{{ card.primary_label || '项目官网' }}<DiscoverGlyph name="external" :size="15" /></a><a :href="github" target="_blank" rel="noopener noreferrer"><DiscoverGlyph name="github" :size="17" />GitHub<DiscoverGlyph name="external" :size="14" /></a></div>
        <p v-if="card?.github_archived || card?.upstream_maintenance_notice" class="readme-maintenance">{{ card?.upstream_maintenance_notice || '上游仓库已归档，使用前请留意维护状态。' }}</p>
        <p v-if="card?.icon_attribution" class="readme-icon-credit">{{ card.icon_attribution }}</p>
      </section>
      <section class="readme-section" aria-label="项目 README">
        <div class="readme-toolbar"><span><DiscoverGlyph name="doc" :size="16" />README</span><div v-if="readme" class="readme-languages" aria-label="正文语言"><button :aria-pressed="language === 'zh'" @click="language = 'zh'">中文</button><span aria-hidden="true">|</span><button :aria-pressed="language === 'en'" @click="language = 'en'">English</button></div></div>
        <RequestState v-if="readmeLoading || readmeError" :loading="readmeLoading" :error="readmeError" @retry="loadReadme" />
        <template v-else-if="readme"><article ref="article" class="project-markdown" :lang="language === 'zh' ? 'zh-CN' : 'en'" @click="articleClick" v-html="rendered" /><p v-if="copyMessage" role="status">{{ copyMessage }}</p><footer class="readme-provenance"><p>{{ typeof readme.translation === 'string' ? readme.translation : '依据项目官方 README 整理' }}</p><p><a :href="readme.source_url" target="_blank" rel="noopener noreferrer">查看上游原文 ↗</a><span> · </span><a :href="`${github}/commit/${readme.commit}`" target="_blank" rel="noopener noreferrer">版本 {{ readme.commit.slice(0, 7) }}</a><span> · 资料采集 {{ date(readme.fetched_at) }}</span></p><details><summary>许可与版权声明</summary><section v-for="license in readme.licenses" :key="license.name"><h3>{{ license.name }}</h3><pre>{{ license.text }}</pre></section></details></footer></template>
        <div v-else class="readme-unavailable"><p>中文 README 正在整理中，可以先阅读项目官方原文。</p><a :href="project.source_url" target="_blank" rel="noopener noreferrer">查看官方 README ↗</a><img v-if="project.cover_url" :src="project.cover_url" :alt="`${project.name} 官方项目展示`" loading="lazy"></div>
      </section>
      <RepoComments :key="project.full_name" :owner="project.full_name.split('/')[0]!" :repo="project.full_name.split('/')[1]!" editorial><template #actions><a class="readme-bar-action" :href="github" target="_blank" rel="noopener noreferrer" aria-label="在 GitHub 查看项目"><DiscoverGlyph name="github" :size="24" /></a><button class="readme-bar-action" :class="{ saved: bookmarks.has(project.full_name) }" :aria-pressed="bookmarks.has(project.full_name)" title="收藏保存在当前浏览器" @click="bookmarks.toggle(project.full_name)"><DiscoverGlyph name="star" :size="26"/><span>{{ bookmarks.has(project.full_name) ? '已收藏' : '收藏' }}</span></button></template></RepoComments>
      <p v-if="bookmarks.error.value" class="readme-save-error" role="alert">{{ bookmarks.error.value }}</p>
      <section v-if="catalog?.related?.length" class="os-related"><h2>还可以看看</h2><div class="os-project-grid"><OpenSourceProjectCard v-for="item in catalog.related.slice(0, 4)" :key="item.full_name" :project="item" /></div></section>
    </template>
    <RequestState v-else-if="loading || error" :loading="loading" :error="error" @retry="load" />
    <section v-else class="os-empty"><h1>这个项目尚未收录</h1><RouterLink to="/open-source">返回精选</RouterLink></section>
  </main>
</template>
