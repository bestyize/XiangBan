<script setup lang="ts">
import { computed } from 'vue'
import type { RepoRank } from '@/api/discover'
import { repoAbout, reviewedProjectIcon } from '@/lib/githubProjectAbout'
import { repoGrowth, repoPage } from './contract'
import DiscoverGlyph from './DiscoverGlyph.vue'
import ProjectIcon from './ProjectIcon.vue'
type Repo = RepoRank & { language_color?: string; spark?: number[] }
const props = withDefaults(defineProps<{ repo: Repo; compact?: boolean; baseline?: boolean }>(), { compact: false, baseline: false })
const name = computed(() => props.repo.full_name.split('/').at(-1) || props.repo.full_name)
const about = computed(() => repoAbout(props.repo.full_name, props.repo.description, props.repo.description_zh))
const icon = computed(() => reviewedProjectIcon(props.repo.full_name))
const color = computed(() => /^#[\da-f]{6}$/i.test(props.repo.language_color || '') ? props.repo.language_color : 'var(--ink-3)')
const spark = computed(() => props.repo.star_gain != null ? (props.repo.spark || []).filter(Number.isFinite) : [])
const bars = computed(() => { const max = Math.max(1, ...spark.value); return spark.value.map(v => Math.max(2, v / max * 24)) })
const page = computed(() => repoPage(props.repo.full_name))
const href = computed(() => `https://github.com/${props.repo.full_name.split('/').map(encodeURIComponent).join('/')}`)
</script>
<template>
  <article class="repo-card" :class="{ compact }">
    <div class="repo-main">
      <span class="rank" :class="{ top: repo.rank <= 3 }" :aria-label="`第 ${repo.rank} 名`">{{ String(repo.rank).padStart(2, '0') }}</span>
      <component :is="page ? 'RouterLink' : 'a'" v-bind="page ? { to: page } : { href, target: '_blank', rel: 'noopener noreferrer' }" class="repo-link">
        <ProjectIcon :name="name" :src="icon.url" :fit="icon.fit" />
        <div class="repo-identity"><h3>{{ name }}</h3><span :title="repo.full_name">{{ repo.full_name }}</span></div>
        <DiscoverGlyph v-if="!page" class="external" name="external" :size="14" />
      </component>
    </div>
    <p class="desc">{{ about.text }}</p>
    <div class="repo-meta">
      <span class="stars" :aria-label="`${repo.star_total} GitHub 星标`"><DiscoverGlyph name="star" :size="13" />{{ repo.star_total.toLocaleString('zh-CN') }}</span>
      <span v-if="repo.language" class="language"><i :style="{ background: color }" />{{ repo.language }}</span>
      <span v-if="!baseline && repo.star_gain != null" class="growth">本期 {{ repoGrowth(repo.star_gain) }}</span>
      <span v-if="bars.length >= 2" class="spark" aria-hidden="true"><i v-for="(bar, index) in bars" :key="index" :style="{ height: `${bar}px` }" /></span>
    </div>
    <p v-if="repo.suspected_growth" class="note">增长异常 · 待复核</p>
    <p v-else-if="!compact && !baseline && repo.star_gain != null && repo.observed_days != null && !repo.complete" class="note">实际观测 {{ repo.observed_days }} 天</p>
  </article>
</template>
<style scoped>
.repo-card { position:relative;min-width:0;display:flex;flex-direction:column;background:var(--surface);padding:16px;border:1px solid var(--line);border-radius:12px; }
.repo-card:hover,.repo-card:focus-within { border-color:color-mix(in srgb,var(--brand) 45%,var(--line)); }
.repo-main { display:flex;align-items:center;gap:10px; }
.rank { width:20px;flex:none;font-size:12px;font-variant-numeric:tabular-nums;color:var(--ink-3); }
.rank.top { color:var(--brand); }
.repo-link { display:flex;align-items:center;gap:10px;flex:1;min-width:0;min-height:44px; }
.repo-link::after { content:'';position:absolute;inset:0;border-radius:inherit; }
.repo-link:focus-visible { outline:none; }
.repo-link:focus-visible::after { outline:2px solid var(--focus);outline-offset:3px; }
.repo-identity { flex:1;min-width:0; }
h3 { margin:0;min-width:0;font-size:14px;font-weight:400;line-height:22px;overflow-wrap:anywhere;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden; }
.repo-identity > span { display:block;margin-top:2px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--ink-3);font:11px/1.4 ui-monospace,monospace; }
.external { color:var(--ink-3);flex:none; }
.desc { margin:12px 0 14px;font-size:13px;line-height:1.6;color:var(--ink-2);display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;min-height:3.2em; }
.repo-meta { display:flex;align-items:center;flex-wrap:wrap;gap:6px 12px;margin-top:auto;padding-top:10px;border-top:1px solid var(--line);color:var(--ink-3);font-size:12px; }
.language,.stars { display:inline-flex;align-items:center;gap:5px; }
.language i { width:6px;height:6px;border-radius:50%; }
.growth { margin-left:auto;color:var(--ink-2);font-variant-numeric:tabular-nums; }
.spark { height:24px;display:inline-flex;align-items:flex-end;gap:2px; }
.spark i { width:3px;border-radius:2px;background:var(--ink-3);opacity:.35; }
.note { font-size:12px;line-height:1.6;margin:8px 0 0;color:var(--ink-3); }
.compact { padding:14px 0;border:0;border-radius:0; }
.compact + .compact { border-top:1px solid var(--line); }
.compact .desc { margin-left:30px;min-height:0; }
.compact .repo-meta { margin-left:30px;border-top:0;padding-top:0; }
</style>
