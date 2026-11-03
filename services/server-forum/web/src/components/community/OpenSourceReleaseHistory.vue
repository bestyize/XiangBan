<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { OpenSourceRelease } from '@/lib/openSourceRelease'
import type { SoftwarePlatform } from '@/lib/openSourceSoftware'
import DiscoverGlyph from './DiscoverGlyph.vue'
import OpenSourcePackageList from './OpenSourcePackageList.vue'
import RequestState from './RequestState.vue'

const props = defineProps<{
  releases?: OpenSourceRelease[]
  releasesUrl: string
  platforms: readonly SoftwarePlatform[]
  loading: boolean
  error: string
}>()
defineEmits<{ retry: [] }>()

const limit = ref(8)
watch(() => props.releasesUrl, () => { limit.value = 8 })
const visible = computed(() => props.releases?.slice(0, limit.value) ?? [])
const dateFormat = new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeZone: 'Asia/Shanghai' })
</script>

<template>
  <div class="release-history">
    <details v-for="release in visible" :key="release.id">
      <summary>
        <span class="history-version">{{ release.version }}</span>
        <span v-if="release.prerelease" class="history-preview">预发布</span>
        <time v-if="release.publishedAt" :datetime="release.publishedAt">{{ dateFormat.format(new Date(release.publishedAt)) }}</time>
        <span class="history-count">{{ release.packages.length }} 个安装包</span>
        <DiscoverGlyph name="down" :size="14" />
      </summary>
      <div class="history-packages">
        <OpenSourcePackageList :packages="release.packages" :platforms="platforms" />
        <a :href="release.url" target="_blank" rel="noopener noreferrer">查看此版本的 GitHub 发布页 ↗</a>
      </div>
    </details>
    <RequestState v-if="!visible.length" :loading="loading" :error="error" :empty="!loading && !error"
      message="暂无已同步的历史版本。" @retry="$emit('retry')" />
    <div class="history-actions">
      <button v-if="(releases?.length ?? 0) > limit" type="button" @click="limit += 8">加载更多版本</button>
      <a :href="releasesUrl" target="_blank" rel="noopener noreferrer">在 GitHub 查看全部版本 ↗</a>
    </div>
  </div>
</template>

<style scoped>
.release-history, .history-packages { display: grid; gap: 10px; }
.release-history details { border: 1px solid var(--line); border-radius: 9px; }
.release-history summary { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; min-height: 44px; padding: 8px 12px; list-style: none; cursor: pointer; color: var(--ink-3); font-size: 12px; }
.release-history summary::-webkit-details-marker { display: none; }
.release-history details[open] > summary { border-bottom: 1px solid var(--line); }
.release-history details[open] > summary svg { transform: rotate(180deg); }
.history-version { font-weight: 650; color: var(--ink); overflow-wrap: anywhere; }
.history-preview { padding: 2px 6px; border-radius: 5px; background: var(--surface-2); }
.history-count { margin-left: auto; }
.history-packages { padding: 12px; }
.history-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; }
.history-actions button { padding: 6px 10px; border: 1px solid var(--line); border-radius: 7px; background: var(--surface); color: var(--ink); font: inherit; font-size: 13px; cursor: pointer; }
.release-history a, .release-history p { color: var(--ink-3); font-size: 12px; }
</style>
