<script setup lang="ts">
import type { OpenSourceRelease } from '@/lib/openSourceRelease'
import type { SoftwarePlatform } from '@/lib/openSourceSoftware'
import DiscoverGlyph from './DiscoverGlyph.vue'
import OpenSourcePackageList from './OpenSourcePackageList.vue'
import RequestState from './RequestState.vue'

defineProps<{
  data?: OpenSourceRelease | null
  latest: string
  platforms: readonly SoftwarePlatform[]
  loading: boolean
  error: string
  synced: boolean
}>()
defineEmits<{ retry: [] }>()

const dateFormat = new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeZone: 'Asia/Shanghai' })
</script>

<template>
  <div class="release-packages">
    <RequestState :loading="loading" :error="error" :empty="!data && !loading && !error"
      :message="synced ? '暂无正式版本。' : '正在等待首次同步，可先查看 GitHub 官方发布页。'" @retry="$emit('retry')" />
    <template v-if="data">
      <div class="release-meta">
        <strong>{{ data.version }}</strong>
        <span v-if="!synced">收录时的正式版</span>
        <time v-if="data.publishedAt" :datetime="data.publishedAt">{{ dateFormat.format(new Date(data.publishedAt)) }}</time>
      </div>
      <OpenSourcePackageList :packages="data.packages" :platforms="platforms" />
    </template>
    <a class="release-upstream" :href="latest" target="_blank" rel="noopener noreferrer">
      查看 GitHub 官方发布页<DiscoverGlyph name="external" :size="13" />
    </a>
  </div>
</template>

<style scoped>
.release-packages { display: grid; gap: 12px; }
.release-meta { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; color: var(--ink-3); font-size: 12px; }
.release-meta strong { color: var(--brand); background: color-mix(in srgb, var(--brand) 12%, var(--surface)); padding: 3px 8px; border-radius: 5px; }
.release-upstream { display: inline-flex; align-items: center; gap: 5px; width: fit-content; color: var(--ink-3); font-size: 12px; }
</style>
