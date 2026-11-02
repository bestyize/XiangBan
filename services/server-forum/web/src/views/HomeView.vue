<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { discoverApi } from '@/api/discover'
import CommunityTabs from '@/components/community/CommunityTabs.vue'
import NewsRow from '@/components/community/NewsRow.vue'
import RequestState from '@/components/community/RequestState.vue'
import RepoBoard from '@/components/community/RepoBoard.vue'
import ModelPreview from '@/components/community/ModelPreview.vue'
import TrackerPanel from '@/components/community/TrackerPanel.vue'
import NavDirectory from '@/components/community/NavDirectory.vue'
import { useCommunityResource } from '@/components/community/resource'
import { repoMetric, webLink } from '@/components/community/contract'
import DiscoverGlyph from '@/components/community/DiscoverGlyph.vue'
import DiscoverRepoRow from '@/components/community/DiscoverRepoRow.vue'
import CommunityPackCard from '@/components/community/CommunityPackCard.vue'
import WeeklyFeed from '@/components/WeeklyFeed.vue'
import ProjectFeed from '@/components/ProjectFeed.vue'
import { useHead } from '@/lib/head'
import '@/components/community/community.css'
const route = useRoute(), router = useRouter()
const tabs = [{ key: 'recommend', label: '推荐' }, { key: 'rankings', label: '排行榜' }, { key: 'repos', label: '开源热榜' }, { key: 'tracker', label: '追踪' }, { key: 'nav', label: '导航' }]
const tab = computed(() => {
  const raw = String(route.query.tab || 'recommend')
  return tabs.some(t => t.key === raw) || raw === 'weekly' || raw === 'projects' ? raw : 'recommend'
})
const { data, loading, error, load } = useCommunityResource(discoverApi.home)
const frontier = useCommunityResource(() => discoverApi.frontier(1))
void frontier.load()
watch(tab, current => { if (current === 'recommend') void load() }, { immediate: true })
function setTab(key: string) { void router.replace({ query: { ...(key === 'recommend' ? {} : { tab: key }) } }) }
const banners = computed(() => (data.value?.banners || []).map(banner => {
  const link = banner.link || ''
  const match = /^(topic|pack|tool|weekly):(.+)$/.exec(link)
  const paths:Record<string,string> = {topic:'/t/',pack:'/packs/',tool:'/tools/',weekly:'/weekly/'}
  const validReference = match && (!['topic','pack'].includes(match[1]!) || /^\d+$/.test(match[2]!))
  const href = validReference ? paths[match[1]!] + encodeURIComponent(match[2]!) : webLink(link)
  return {...banner,href}
}).filter(banner => banner.href))
useHead(() => ({ title: '社区' }))
</script>
<template><main class="community-page home-page">
  <h1 class="sr-only">小伴社区</h1>
  <CommunityTabs :items="tabs" :active="tab" label="首页栏目" @change="setTab" />
  <template v-if="tab === 'weekly' || tab === 'projects'"><RouterLink to="/" class="community-back">← 返回推荐</RouterLink><WeeklyFeed v-if="tab === 'weekly'" /><ProjectFeed v-else /></template>
  <template v-else-if="tab === 'recommend'">
    <RequestState :loading="loading" :error="error" @retry="load" />
    <div v-if="data" class="recommend-feed"><header class="desktop-discovery-head"><div><span>小伴社区</span><h2>发现值得关注的新内容</h2></div><RouterLink to="/daily">浏览全部日报<DiscoverGlyph name="arrow" :size="16"/></RouterLink></header>
      <div v-if="banners.length" class="banner-strip" aria-label="社区推荐"><a v-for="banner in banners" :key="banner.key" :href="banner.href" class="banner-card"><div class="banner-top"><span v-if="banner.tag" class="banner-tag">{{ banner.tag }}</span><DiscoverGlyph name="sparkle" :size="20"/></div><h2>{{ banner.title }}</h2><div class="banner-bottom"><p v-if="banner.byline">{{ banner.byline }}</p><span v-if="banner.sample" class="sample">示例</span><DiscoverGlyph name="arrow" :size="18"/></div></a></div>
      <section v-if="frontier.data.value?.items.length" class="home-card"><div class="section-heading"><div><DiscoverGlyph name="news" :size="18"/><h2>小伴前沿</h2></div><RouterLink to="/frontier">查看全部<DiscoverGlyph name="arrow" :size="14"/></RouterLink></div><div class="news-grid"><NewsRow v-for="item in frontier.data.value.items.slice(0,3)" :key="item.key" :item="item" frontier /></div></section>
      <section class="daily-section"><div class="section-heading"><div><DiscoverGlyph name="news" :size="18"/><h2>AI 日报</h2><span v-if="data.news.date&&data.news.total>0" class="update-date">{{ data.news.date.slice(5).replace('-',' / ') }} 更新</span></div><RouterLink to="/daily">往期<DiscoverGlyph name="arrow" :size="14"/></RouterLink></div><div class="news-grid"><NewsRow v-for="item in data.news.items" :key="item.key" :item="item"/></div><RequestState :empty="!data.news.items.length" message="暂无已发布日报。"/></section>
      <section v-if="data.featured.length" class="home-card featured-card"><div class="section-heading"><div><DiscoverGlyph name="sparkle" :size="18"/><h2>精选创作</h2></div><RouterLink to="/discuss?tab=create">更多<DiscoverGlyph name="arrow" :size="14"/></RouterLink></div><div class="featured-strip"><CommunityPackCard v-for="pack in data.featured" :key="pack.id" :pack="pack"/></div></section>
      <section v-if="data.repos&&data.repos.ranks.length" class="home-card repo-preview"><div class="section-heading"><div><DiscoverGlyph name="star" :size="18"/><h2>{{ data.repos.metric==='star total baseline'?'开源项目快照':'开源热榜' }}</h2></div><button type="button" @click="setTab('repos')">完整榜单<DiscoverGlyph name="arrow" :size="14"/></button></div><p class="repo-note">{{ repoMetric(data.repos) }} · {{ data.repos.source }}<span v-if="data.repos.sample"> · 示例数据</span></p><DiscoverRepoRow v-for="repo in data.repos.ranks.slice(0,3)" :key="repo.full_name" :repo="repo" :baseline="data.repos.metric==='star total baseline'" compact/></section>
      <p v-if="data.sample" class="sample-note"><DiscoverGlyph name="info" :size="14"/>带「示例」字样的条目为示例数据。</p>
    </div>
  </template>
  <RepoBoard v-else-if="tab === 'repos'" />
  <ModelPreview v-else-if="tab === 'rankings'" />
  <TrackerPanel v-else-if="tab === 'tracker'" />
  <NavDirectory v-else />
</main></template>
<style scoped>
.recommend-feed{display:grid;gap:12px;padding-top:8px;min-width:0}.banner-strip{display:flex;gap:8px;overflow:auto;scroll-snap-type:x mandatory;scrollbar-width:none}.banner-card{flex:0 0 88%;display:flex;flex-direction:column;min-height:128px;padding:16px;border-radius:16px;background:var(--surface);scroll-snap-align:start;box-sizing:border-box}.banner-card:only-child{flex-basis:100%}.banner-top,.banner-bottom{display:flex;align-items:center;justify-content:space-between;gap:8px}.banner-top{color:var(--brand)}.banner-tag{font-size:12px;font-weight:500;padding:3px 8px;border-radius:6px;background:color-mix(in srgb,var(--brand) 8%,var(--surface))}.banner-card h2{font-size:16px;line-height:1.6;margin:12px 0;overflow-wrap:anywhere}.banner-bottom{margin-top:auto;color:var(--ink-3);font-size:12px}.banner-bottom p{margin:0;flex:1}.sample{color:var(--brand)}.section-heading{display:flex;align-items:center;justify-content:space-between;gap:8px;min-height:48px}.section-heading>div{display:flex;align-items:center;flex-wrap:wrap;gap:6px;min-width:0}.section-heading h2{margin:0;font-size:16px;font-weight:600}.section-heading>a,.section-heading>button{display:flex;align-items:center;justify-content:flex-end;gap:3px;flex:none;font-size:12px;color:var(--ink-3);min-height:44px;background:none;border:0;padding:0;cursor:pointer}.daily-section>.section-heading{padding:0 4px 4px}.update-date{font-size:12px;color:var(--ink-3);font-weight:400;margin-left:4px}.news-grid{display:grid;gap:8px;min-width:0}.home-card{background:var(--surface);border-radius:16px;padding:8px 14px 12px;min-width:0}.featured-strip{display:flex;gap:8px;overflow:auto;scroll-snap-type:x proximity;padding:8px 0 2px}.featured-strip :deep(.pack-card){flex:0 0 140px;scroll-snap-align:start;background:var(--surface-2)}.featured-strip :deep(.copy){padding:10px}.featured-strip :deep(h3){font-size:14px}.repo-note{font-size:12px;line-height:1.6;margin:0 0 4px;color:var(--ink-3)}.sample-note{display:flex;gap:6px;align-items:center;color:var(--ink-3);font-size:12px;margin:0;padding:4px}.sample-note svg{flex:none}@media(min-width:768px){.recommend-feed{gap:16px}.banner-card{flex-basis:calc(50% - 4px);min-height:144px}.banner-card:only-child{flex-basis:100%}.news-grid{grid-template-columns:repeat(2,minmax(0,1fr))}.news-grid>:last-child:nth-child(odd){grid-column:1/-1}.home-card{padding:12px 20px 16px}.featured-strip{gap:12px}.featured-strip :deep(.pack-card){flex-basis:180px}}

.desktop-discovery-head{display:none}
@container page (min-width:1100px){
  .home-page{max-width:1400px}
  .recommend-feed{grid-template-columns:minmax(0,1.65fr) minmax(300px,1fr);column-gap:24px;row-gap:20px;align-items:start;padding-top:24px}
  .desktop-discovery-head{display:flex;grid-column:1/-1;align-items:flex-end;justify-content:space-between;gap:24px;padding-bottom:4px}
  .desktop-discovery-head span{font-size:12px;color:var(--ink-3)}
  .desktop-discovery-head h2{font-size:24px;line-height:1.4;letter-spacing:-.02em;margin:6px 0 0;font-weight:650}
  .desktop-discovery-head>a{display:flex;align-items:center;gap:8px;font-size:14px;min-height:44px;color:var(--ink-2)}
  .banner-strip{grid-column:1/-1;gap:16px}
  .banner-card{flex-basis:calc(50% - 8px);min-height:128px;padding:20px 24px}
  .daily-section{grid-column:1;grid-row:span 3;min-width:0}
  .daily-section>.section-heading{padding:0 0 10px}
  .section-heading h2{font-size:18px}
  .section-heading>a,.section-heading>button{font-size:13px}
  .news-grid{grid-template-columns:minmax(0,1fr);gap:12px}
  .news-grid>:last-child:nth-child(odd){grid-column:auto}
  .news-grid :deep(.news-row){padding:20px 24px;min-height:132px;gap:20px}
  .news-grid :deep(.news-row h3){font-size:17px;line-height:1.6;margin-bottom:12px}
  .news-grid :deep(.news-row img){width:136px;height:92px;border-radius:10px}
  .news-grid>:first-child :deep(h3){font-size:20px;line-height:1.5}
  .news-grid>:first-child :deep(img){width:160px;height:108px}
  .featured-card,.repo-preview{grid-column:2;padding:12px 18px 16px}
  .featured-strip{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;overflow:visible}
  .featured-strip :deep(.pack-card){min-width:0;max-width:none}
  .featured-strip :deep(.pack-card:nth-child(n+5)){display:none}
  .featured-strip :deep(.pack-card img){height:100px;aspect-ratio:auto}
  .featured-strip :deep(.copy){padding:10px 12px}
  .featured-strip :deep(h3){font-size:14px;line-height:1.5}
  .repo-preview :deep(.repo-link){gap:8px}
  .repo-preview :deep(.external){display:none}
  .repo-preview :deep(.repo-meta){padding-left:30px}
  .sample-note{grid-column:2;font-size:12px;line-height:1.6}
}
</style>
