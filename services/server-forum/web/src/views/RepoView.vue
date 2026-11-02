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
import '@/components/community/community.css'
const route = useRoute()
const owner = computed(() => String(route.params.owner || ''))
const repo = computed(() => String(route.params.repo || ''))
const detail = useCommunityResource(() => discoverApi.repo(owner.value, repo.value))
const lang = ref<'zh' | 'en'>('zh'), expanded = ref(false)
watch([owner, repo], () => { lang.value = 'zh'; expanded.value = false; void detail.load() }, { immediate:true })
const d = computed(() => detail.data.value)
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
useHead(() => ({ title:d.value ? `${d.value.full_name} · 开源热榜` : '项目详情' }))
</script>
<template><main class="community-page"><div class="repo-page">
  <CommunityBack fallback="/" label="返回" />
  <RequestState :loading="detail.loading.value" :error="detail.error.value" @retry="detail.load" />
  <template v-if="d">
    <section class="hero">
      <div class="hero-top"><span class="tile" aria-hidden="true">{{ d.repo.slice(0, 1).toUpperCase() }}</span><div class="names"><p class="owner">{{ d.owner }} /</p><h1>{{ d.repo }}</h1></div><span v-if="d.is_latest" class="rank">本周 No.{{ d.rank }}</span><span v-else class="rank off">已掉出本周榜</span></div>
      <p v-if="d.description_zh || d.description" class="desc">{{ d.description_zh || d.description }}</p>
      <p v-if="d.description_zh && d.description && d.description_zh !== d.description" class="orig">原文：{{ d.description }}</p>
      <div class="stats"><div><span class="gain">{{ repoGrowth(d.star_gain) }}</span><span class="gain-l">近 7 天新增 Star</span></div><div class="mini"><div><b>{{ d.star_total.toLocaleString('zh-CN') }}</b><span>总 Star</span></div><div v-if="d.language"><b><i :style="{ background:color }" />{{ d.language }}</b><span>主要语言</span></div></div></div>
      <div class="actions"><button type="button" class="primary" :class="{ on:saved }" :aria-pressed="saved" @click="toggleSave">{{ saved ? '已收藏' : '收藏' }}</button><a class="secondary" :href="github" target="_blank" rel="noopener noreferrer">在 GitHub 查看 ↗</a></div>
      <p class="meta">数据来自 GitHub Trending 周榜<template v-if="d.as_of"> · 整理于 {{ displayDate(d.as_of) }}</template> · <button type="button" class="link" @click="jumpToComments">跳到评论</button></p>
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
    <RepoComments :owner="owner" :repo="repo" />
  </template>
</div></main></template>
<style scoped>
.repo-page { max-width:760px; margin:0 auto; padding:16px 22px 40px; background:var(--surface); }
.hero,.intro { padding:20px 0; border-bottom:1px solid var(--line); }
.hero-top { display:flex; align-items:center; gap:12px; } .tile { flex:none; display:grid; place-items:center; width:44px; height:44px; border-radius:12px; background:color-mix(in srgb,var(--brand) 10%,var(--surface)); color:var(--brand); font-size:20px; font-weight:700; }
.names { flex:1; min-width:0; } .owner { margin:0; font-size:13px; color:var(--ink-3); } h1 { margin:0; font-size:22px; line-height:1.25; overflow-wrap:anywhere; }
.rank { flex:none; font-size:12px; font-weight:600; color:var(--brand); background:color-mix(in srgb,var(--brand) 10%,var(--surface)); padding:4px 10px; border-radius:999px; } .rank.off { color:var(--ink-3); background:var(--surface-2); }
.desc { margin:14px 0 0; font-size:15px; line-height:1.7; } .orig { margin:6px 0 0; font-size:12px; line-height:1.5; color:var(--ink-3); }
.stats { display:flex; align-items:flex-end; justify-content:space-between; gap:16px; margin-top:16px; padding-top:16px; border-top:1px solid var(--line); }
.gain { display:block; font-size:32px; font-weight:700; line-height:1; color:var(--brand); font-variant-numeric:tabular-nums; } .gain-l { display:block; margin-top:6px; font-size:12px; color:var(--ink-2); }
.mini { display:flex; gap:20px; } .mini div { text-align:right; } .mini b { display:block; font-size:15px; font-weight:600; font-variant-numeric:tabular-nums; } .mini span { font-size:12px; color:var(--ink-3); } .mini i { display:inline-block; width:8px; height:8px; border-radius:50%; margin-right:5px; }
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
</style>
