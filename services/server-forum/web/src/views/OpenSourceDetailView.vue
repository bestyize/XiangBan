<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { categoryLabel, findProject } from '@/lib/openSource'
import { openSourceCatalog as catalog, loadOpenSourceCatalog } from '@/stores/openSource'
import { useRepoBookmarks } from '@/lib/repoBookmarks'
import { useHead } from '@/lib/head'
import CommunityBack from '@/components/community/CommunityBack.vue'
import DiscoverGlyph from '@/components/community/DiscoverGlyph.vue'
import RepoComments from '@/components/community/RepoComments.vue'
import OpenSourceProjectCard from '@/components/community/OpenSourceProjectCard.vue'
import '@/components/community/community.css'
import '@/components/community/open-source.css'
const route = useRoute()
void loadOpenSourceCatalog()
const project = computed(() => findProject(`${String(route.params.owner)}/${String(route.params.repo)}`, catalog))
const bookmarks = useRepoBookmarks()
const broken = ref(false)
watch(project, () => { broken.value = false })
const related = computed(() => project.value ? catalog.projects.filter(item => item.full_name !== project.value!.full_name && (item.category === project.value!.category || item.related_categories.includes(project.value!.category) || project.value!.related_categories.includes(item.category))).slice(0, 2) : [])
useHead(() => ({ title: project.value ? `${project.value.name} · 精选` : '项目未收录' }))
</script>
<template>
  <main class="community-page os-detail-page">
    <nav class="os-detail-nav"><CommunityBack fallback="/open-source" label="精选" /><span>项目导读</span></nav>
    <template v-if="project">
      <div class="os-detail-layout"><article class="os-detail-body">
        <header class="os-detail-hero"><div class="os-kicker">{{ categoryLabel(project.category) }}<span>·</span>{{ project.name }}</div><h1>{{ project.title }}</h1><a class="os-repo-name" :href="`https://github.com/${project.full_name}`" target="_blank" rel="noopener noreferrer"><DiscoverGlyph name="github" :size="16" />{{ project.full_name }}<DiscoverGlyph name="external" :size="14" /></a><p class="os-detail-description">{{ project.summary }}</p><div class="os-tags"><span v-for="tag in project.tags" :key="tag">{{ tag }}</span></div><p class="os-hero-effort"><DiscoverGlyph name="compass" :size="14" />{{ project.effort }}</p><div class="os-mobile-actions"><a class="os-button" :href="`https://github.com/${project.full_name}`" target="_blank" rel="noopener noreferrer" aria-label="前往 GitHub 查看项目">GitHub<DiscoverGlyph name="external" :size="16" /></a><button class="os-button secondary" type="button" :aria-pressed="bookmarks.has(project.full_name)" @click="bookmarks.toggle(project.full_name)"><DiscoverGlyph name="bookmark" :size="18" />{{ bookmarks.has(project.full_name) ? '已收藏' : '收藏项目' }}</button></div><p v-if="bookmarks.error.value" class="os-status" role="alert">{{ bookmarks.error.value }}</p></header>
        <figure v-if="project.cover_url && !broken" class="os-detail-cover"><img :src="project.cover_url" :alt="`${project.name} · ${project.cover_caption}`" @error="broken = true"><figcaption>{{ project.cover_caption }} · 来自<a :href="project.source_url" target="_blank" rel="noopener noreferrer">项目官方仓库 ↗</a></figcaption></figure>
        <section class="os-detail-section"><h2>为什么值得看看</h2><ol class="os-detail-highlights"><li v-for="item in project.highlights" :key="item">{{ item }}</li></ol></section>
        <section class="os-detail-section"><h2>适合谁</h2><p>{{ project.audience }}</p></section>
        <section class="os-detail-section"><h2>从这里开始</h2><p>{{ project.getting_started }}</p></section>
        <section class="os-detail-section"><h2>开始前了解一下</h2><ul><li v-for="note in project.notes" :key="note">{{ note }}</li></ul></section>
        <section class="os-detail-section"><h2>继续了解</h2><div class="os-reference-links"><a :href="project.homepage_url" target="_blank" rel="noopener noreferrer">项目官网与文档<DiscoverGlyph name="external" :size="16" /></a><a :href="project.source_url" target="_blank" rel="noopener noreferrer">README 与源代码<DiscoverGlyph name="external" :size="16" /></a><a :href="`https://github.com/${project.full_name}#license`" target="_blank" rel="noopener noreferrer">查看项目许可<DiscoverGlyph name="external" :size="16" /></a></div><p class="os-source-note">{{ catalog.notice }}<br>资料整理：{{ catalog.reviewed_at }}</p></section>
      </article><aside class="os-detail-sidebar"><section class="os-info-card"><h2>项目速览</h2><dl><div><dt>项目</dt><dd>{{ project.name }}</dd></div><div><dt>使用门槛</dt><dd>{{ project.effort }}</dd></div><div><dt>所属分类</dt><dd>{{ [project.category, ...project.related_categories].map(categoryLabel).join(' / ') }}</dd></div><div><dt>资料来源</dt><dd>项目官方 README</dd></div></dl><div class="os-detail-actions"><a class="os-button" :href="`https://github.com/${project.full_name}`" target="_blank" rel="noopener noreferrer"><DiscoverGlyph name="github" :size="18" />前往 GitHub<DiscoverGlyph name="external" :size="16" /></a><button class="os-button secondary" type="button" :aria-pressed="bookmarks.has(project.full_name)" @click="bookmarks.toggle(project.full_name)"><DiscoverGlyph name="bookmark" :size="18" />{{ bookmarks.has(project.full_name) ? '已收藏' : '收藏项目' }}</button><a class="os-button secondary" href="#comments"><DiscoverGlyph name="comment" :size="18" />交流与提问</a></div><p class="os-source-note">收藏保存在当前浏览器。</p><p v-if="bookmarks.error.value" class="os-status" role="alert">{{ bookmarks.error.value }}</p></section></aside></div>
      <section class="os-detail-comments"><RepoComments :key="project.full_name" :owner="project.full_name.split('/')[0]!" :repo="project.full_name.split('/')[1]!" /></section>
      <section v-if="related.length" class="os-related"><h2>还可以看看</h2><div class="os-project-grid"><OpenSourceProjectCard v-for="item in related" :key="item.full_name" :project="item" /></div></section>
    </template>
    <section v-else class="os-empty"><DiscoverGlyph name="search" :size="24" /><h1>这个项目尚未收录</h1><p>回到精选页，发现其他值得关注的项目。</p><RouterLink to="/open-source" class="os-button">浏览精选</RouterLink></section>
  </main>
</template>
