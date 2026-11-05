<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { OpenSourceRelease } from '@/lib/openSourceRelease'
import type { SoftwarePlatform } from '@/lib/openSourceSoftware'
import { isMainlandChina, softwareDownloadUrl } from '@/lib/softwareDownload'
import DiscoverGlyph from './DiscoverGlyph.vue'

const props = defineProps<{ packages: OpenSourceRelease['packages']; platforms: readonly SoftwarePlatform[] }>()
const platform = ref<SoftwarePlatform>()
const mainland = ref<boolean | null>(null)
onMounted(async () => { mainland.value = await isMainlandChina() })
const order: readonly SoftwarePlatform[] = ['Windows', 'macOS', 'Linux', 'Android']
const groups = computed(() => order.flatMap(key => {
  const packages = props.packages.filter(item => item.platform === key)
  return packages.length ? [{ key, packages }] : []
}))
const activeGroup = computed(() => groups.value.find(group => group.key === platform.value)
  ?? groups.value.find(group => props.platforms.includes(group.key)) ?? groups.value[0])
const sizeFormat = new Intl.NumberFormat('zh-CN', { maximumFractionDigits: 1 })
function packageSize(bytes: number) {
  return bytes >= 1024 * 1024 ? `${sizeFormat.format(bytes / (1024 * 1024))} MB` : `${sizeFormat.format(bytes / 1024)} KB`
}
</script>

<template>
  <div class="release-package-list">
    <div v-if="groups.length" class="release-platforms" role="group" aria-label="安装包平台">
      <button v-for="group in groups" :key="group.key" type="button"
        :aria-pressed="group.key === activeGroup?.key" @click="platform = group.key">
        {{ group.key }} <span>{{ group.packages.length }}</span>
      </button>
    </div>
    <ul v-if="activeGroup" class="release-assets" :aria-label="`${activeGroup.key} 安装包`">
      <li v-for="asset in activeGroup.packages" :key="asset.url">
        <div class="release-asset-info">
          <strong>{{ asset.name }}</strong>
          <div class="release-meta">
            <span>{{ asset.format }}</span>
            <span v-if="asset.architecture">{{ asset.architecture }}</span>
            <span v-if="asset.size !== undefined">{{ packageSize(asset.size) }}</span>
          </div>
        </div>
        <div class="release-download-links">
          <span v-if="mainland === null" class="release-meta">正在选择下载线路…</span>
          <a v-else :href="softwareDownloadUrl(asset.url, mainland)" :aria-label="`下载 ${asset.name}`" class="release-download">
            {{ mainland ? '加速下载' : '下载' }}<DiscoverGlyph name="external" :size="14" />
          </a>
          <a v-if="mainland" :href="asset.url" class="release-meta">GitHub 官方下载</a>
        </div>
      </li>
    </ul>
    <p v-else class="release-meta">此版本暂无可识别的安装包。</p>
  </div>
</template>

<style scoped>
.release-package-list { display: grid; gap: 12px; }
.release-meta { display: flex; flex-wrap: wrap; gap: 8px; color: var(--ink-3); font-size: 12px; }
.release-platforms { display: flex; flex-wrap: wrap; gap: 4px; width: fit-content; padding: 4px; border-radius: 9px; background: var(--surface-2); }
.release-platforms button { border: 1px solid transparent; border-radius: 6px; background: transparent; color: var(--ink-2); padding: 5px 8px; min-height: 34px; font: inherit; font-size: 13px; cursor: pointer; }
.release-platforms button[aria-pressed="true"] { background: var(--surface); color: var(--ink); border-color: var(--line); }
.release-platforms span { margin-left: 3px; color: var(--ink-3); font-size: 11px; }
.release-assets { display: grid; gap: 6px; padding: 0; margin: 0; list-style: none; }
.release-assets li { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 9px 12px; border-radius: 9px; background: var(--surface-2); }
.release-asset-info { min-width: 0; overflow-wrap: anywhere; font-size: 13px; }
.release-asset-info .release-meta { margin-top: 3px; }
.release-download-links { display: grid; justify-items: end; gap: 5px; flex-shrink: 0; }
.release-download { display: inline-flex; align-items: center; gap: 5px; flex: none; min-height: 36px; padding: 5px 11px; border-radius: 7px; background: var(--brand); color: white; font-size: 13px; }
</style>
