<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { discoverApi } from '@/api/discover'
import CommunityBack from '@/components/community/CommunityBack.vue'
import ReadmeBlocks from '@/components/community/ReadmeBlocks.vue'
import RepoComments from '@/components/community/RepoComments.vue'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import { displayDate, repoGrowth } from '@/components/community/contract'
import { useHead } from '@/lib/head'
import { useRepoBookmarks } from '@/lib/repoBookmarks'
import { repoAbout, reviewedProjectIcon } from '@/lib/githubProjectAbout'
import ProjectIcon from '@/components/community/ProjectIcon.vue'
import '@/components/community/community.css'
const route = useRoute()
const owner = computed(() => String(route.params.owner || ''))
const repo = computed(() => String(route.params.repo || ''))
const detail = useCommunityResource(() => discoverApi.repo(owner.value, repo.value))
const lang = ref<'zh' | 'en'>('zh'), expanded = ref(false)
watch([owner, repo], () => { lang.value = 'zh'; expanded.value = false; void detail.load() }, { immediate:true })
const d = computed(() => detail.data.value)
const about = computed(() => repoAbout(d.value?.full_name || '', d.value?.description, d.value?.description_zh))
const icon = computed(() => reviewedProjectIcon(d.value?.full_name || ''))
const baseline = computed(() => d.value?.metric === 'star total baseline')
const hasZh = computed(() => !!d.value?.readme.zh_blocks?.length)
const showZh = computed(() => lang.value === 'zh' && hasZh.value)
const blocks = computed(() => (showZh.value ? d.value?.readme.zh_blocks : d.value?.readme.blocks) ?? [])
const long = computed(() => blocks.value.length > 14)
const color = computed(() => (/^#[\da-f]{6}$/i.test(d.value?.language_color || '') ? d.value!.language_color : 'var(--ink-3)'))
const github = computed(() => `https://github.com/${encodeURIComponent(owner.value)}/${encodeURIComponent(repo.value)}`)
const bookmarks = useRepoBookmarks()
const saved = computed(() => !!d.value && bookmarks.has(d.value.full_name))
const saveError = bookmarks.error
function toggleSave() { if (d.value) bookmarks.toggle(d.value.full_name) }
function jumpToComments() { document.getElementById('comments')?.scrollIntoView({ behavior:'smooth', block:'start' }) }
useHead(() => ({ title:d.value ? `${d.value.full_name} · 开源热榜` : '项目详情', description: d.value ? about.value.text : '' }))
</script>
<template><main class="community-page"><div class="repo-page">
  <CommunityBack fallback="/?tab=open-source&category=hot" label="开源热榜" />
  <RequestState :loading="detail.loading.value" :error="detail.error.value" @retry="detail.load" />
  <template v-if="d">
    <section class="hero">
      <div class="hero-top"><ProjectIcon class="tile" :name="d.repo" :src="icon.url" :fit="icon.fit" /><div class="names"><h1>{{ d.repo }}</h1><p class="owner">{{ d.full_name }}</p></div><span class="rank" :class="{ off: !d.is_latest }">{{ d.is_latest ? '本期' : '最近上榜' }}第 {{ d.rank }} 名</span></div>
      <p class="desc">{{ about.text }}</p>
      <details v-if="about.original && about.text !== about.original" class="about-original"><summary>GitHub About 原文</summary><p>{{ about.original }}</p></details>
      <div class="stats"><div><b>{{ d.star_total.toLocaleString('zh-CN') }}</b><span>GitHub Star</span></div><div v-if="!baseline && d.star_gain != null"><b class="gain">{{ repoGrowth(d.star_gain) }}</b><span>本期新增 Star</span></div><div v-if="d.language"><b><i :style="{ background:color }" />{{ d.language }}</b><span>主要语言</span></div></div>
      <div class="actions"><button type="button" class="primary" :class="{ on:saved }" :aria-pressed="saved" @click="toggleSave">{{ saved ? '已收藏' : '收藏' }}</button><a class="secondary" :href="github" target="_blank" rel="noopener noreferrer">在 GitHub 查看 ↗</a></div>
      <p class="meta">{{ d.source || 'GitHub' }} · {{ baseline ? 'Star 总量快照' : '本期榜单数据' }}<template v-if="d.as_of"> · 整理于 {{ displayDate(d.as_of) }}</template> · <button type="button" class="link" @click="jumpToComments">跳到评论</button></p>
      <p v-if="saveError" class="meta" role="status">{{ saveError }}</p>
    </section>
    <section class="intro">
      <div class="head"><h2>项目介绍</h2><div v-if="d.readme.status === 'ok'" class="seg" role="group" aria-label="语言"><button type="button" :class="{ on:showZh }" :disabled="!hasZh" :aria-pressed="showZh" @click="lang = 'zh'">中文</button><button type="button" :class="{ on:!showZh }" :aria-pressed="!showZh" @click="lang = 'en'">原文</button></div></div>
      <template v-if="d.readme.status === 'ok' && blocks.length">
        <p v-if="showZh" class="note">AI 翻译，可能有误，以原文为准。<template v-if="d.readme.zh_truncated"> 译文只覆盖前半部分，完整内容请看原文。</template></p>
        <p v-else-if="lang === 'zh' && !hasZh" class="note">中文翻译暂未提供，已显示英文原文。</p>
        <div class="body" :class="{ col:long && !expanded }"><ReadmeBlocks :blocks="blocks" /><div v-if="long && !expanded" class="fade" /></div>
        <button v-if="long" type="button" class="more" :aria-expanded="expanded" @click="expanded = !expanded">{{ expanded ? '收起' : '展开全文' }}</button>
      </template>
      <div v-else class="empty"><b>{{ d.readme.status === 'none' ? '这个仓库没有 README' : '暂时无法获取 README' }}</b><span>{{ d.readme.status === 'none' ? '作者还没有写项目介绍，可以去 GitHub 查看代码和文档。' : '稍后再来看看，或直接去 GitHub 查看。' }}</span><a class="secondary" :href="github" target="_blank" rel="noopener noreferrer">在 GitHub 查看 ↗</a></div>
    </section>
    <RepoComments style="--comments-bleed: 16px" :owner="owner" :repo="repo" />
  </template>
</div></main></template>
<style scoped>
.repo-page { max-width:800px; margin:0 auto; padding:16px 16px 40px; }
.hero,.intro { padding:24px; margin-top:16px; border:1px solid var(--line); border-radius:12px; background:var(--surface); }
.hero-top { display:flex; align-items:center; gap:12px; flex-wrap:wrap; } .tile { width:48px; height:48px; border-radius:12px; }
.names { flex:1; min-width:120px; } .owner { margin:5px 0 0; font:12px/1.5 ui-monospace,monospace; color:var(--ink-3); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; } h1 { margin:0; font-size:22px; line-height:1.3; overflow-wrap:anywhere; }
.rank { flex:none; font-size:12px; font-weight:600; color:var(--brand); background:color-mix(in srgb,var(--brand) 10%,var(--surface)); padding:4px 10px; border-radius:999px; } .rank.off { color:var(--ink-3); background:var(--surface-2); }
.desc { margin:18px 0 0; font-size:15px; line-height:1.7; overflow-wrap:anywhere; }
.about-original { margin-top:8px;font-size:12px;line-height:1.7;color:var(--ink-3); }
.about-original summary { cursor:pointer;padding:6px 0;width:fit-content; }
.about-original p { margin:4px 0 0;overflow-wrap:anywhere; }
.stats { display:flex; align-items:flex-start; flex-wrap:wrap; gap:16px 32px; margin-top:18px; padding-top:16px; border-top:1px solid var(--line); }
.stats b { display:block;font-size:17px;font-weight:500;line-height:1.5;font-variant-numeric:tabular-nums; }
.stats span { display:block;margin-top:3px;font-size:12px;color:var(--ink-3); }
.stats i { display:inline-block;width:6px;height:6px;border-radius:50%;margin-right:5px; }
.stats .gain { color:var(--brand); }
.actions { display:flex; flex-wrap:wrap; gap:8px; margin-top:16px; }
.primary,.secondary { display:inline-flex; align-items:center; justify-content:center; gap:6px; min-height:44px; padding:0 18px; border:0; border-radius:12px; font-size:14px; font-weight:600; cursor:pointer; }
.actions { flex-wrap:nowrap; } .actions > * { white-space:nowrap; } .primary { flex:1; min-width:96px; background:var(--brand-fill,var(--brand)); color:#fff; } .primary.on { background:color-mix(in srgb,var(--brand) 10%,var(--surface)); color:var(--brand); }
.secondary { background:var(--surface-2); color:var(--ink); } .meta { margin:12px 0 0; font-size:12px; color:var(--ink-3); } .link { border:0; background:none; padding:0; font:inherit; color:var(--brand); cursor:pointer; text-decoration:underline; }
.head { display:flex; align-items:center; justify-content:space-between; gap:12px; margin-bottom:12px; } h2 { margin:0; font-size:17px; font-weight:700; }
.seg { display:flex; padding:2px; border-radius:10px; background:var(--surface-2); } .seg button { min-height:32px; padding:0 14px; border:0; border-radius:8px; background:none; color:var(--ink-2); font-size:13px; cursor:pointer; } .seg .on { background:var(--surface); color:var(--ink); font-weight:600; box-shadow:0 1px 2px rgba(0,0,0,.1); } .seg button:disabled { opacity:.4; cursor:default; }
.note { margin:0 0 12px; padding:8px 10px; border-radius:10px; background:color-mix(in srgb,#e6a700 14%,var(--surface)); color:var(--ink-2); font-size:12px; line-height:1.5; }
.body { position:relative; } .body.col { max-height:300px; overflow:hidden; } .fade { position:absolute; left:0; right:0; bottom:0; height:90px; background:linear-gradient(to bottom,transparent,var(--surface)); pointer-events:none; }
.more { display:block; width:100%; min-height:44px; margin-top:8px; border:0; border-radius:12px; background:var(--surface-2); color:var(--ink); font-size:14px; font-weight:600; cursor:pointer; }
.empty { display:flex; flex-direction:column; align-items:center; gap:6px; padding:20px 8px; text-align:center; color:var(--ink-2); font-size:13px; line-height:1.6; } .empty b { font-size:15px; color:var(--ink); } .empty .secondary { margin-top:10px; }
@media (max-width:480px) { .repo-page { padding-inline:12px; } .hero,.intro { padding:16px; } .stats { gap:14px 20px; } }
</style>
