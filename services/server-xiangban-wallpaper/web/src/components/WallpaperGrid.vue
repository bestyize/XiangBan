<script setup lang="ts">
import { computed, ref } from 'vue'
import { useElementSize } from '@vueuse/core'
import WallpaperCard from './WallpaperCard.vue'
import type { WallpaperItem } from '@/types'
import { tileRatio } from '@/utils'

const props = withDefaults(
  defineProps<{
    items?: readonly WallpaperItem[]
    skeleton?: boolean
    collectorMode?: boolean
    collectedIds?: ReadonlySet<string>
  }>(),
  {
    skeleton: false,
    collectorMode: false,
  },
)
const emit = defineEmits<{
  open: [item: WallpaperItem]
  collect: [item: WallpaperItem]
}>()

const SKELETON_RATIOS = [0.66, 1.5, 0.9, 0.56, 1.2, 0.75, 1.0, 1.6, 0.66, 0.85, 1.3, 0.6]

const el = ref<HTMLElement>()
const { width } = useElementSize(el)

const gap = computed(() => (width.value < 600 ? 8 : 14))
const columnCount = computed(() => {
  const w = width.value
  if (w < 560) return 2
  if (w < 900) return 3
  if (w < 1300) return 4
  if (w < 1700) return 5
  return 6
})

interface Entry {
  key: string
  ratio: number
  item?: WallpaperItem
}

const entries = computed<Entry[]>(() =>
  props.skeleton
    ? SKELETON_RATIOS.map((ratio, i) => ({ key: `s${i}`, ratio }))
    : (props.items ?? []).map((item) => ({ key: item.id, ratio: tileRatio(item), item })),
)

/**
 * Masonry by shortest column. Heights come from the known aspect ratios, so no image has to
 * load before it can be placed, and appending items never moves the ones already on screen.
 */
const columns = computed(() => {
  const n = columnCount.value
  const colWidth = width.value > 0 ? (width.value - gap.value * (n - 1)) / n : 1
  const gapRatio = width.value > 0 ? gap.value / colWidth : 0
  const heights = new Array<number>(n).fill(0)
  const cols: Entry[][] = Array.from({ length: n }, () => [])
  for (const entry of entries.value) {
    const shortest = heights.indexOf(Math.min(...heights))
    cols[shortest].push(entry)
    heights[shortest] += entry.ratio + gapRatio
  }
  return cols
})
</script>

<template>
  <div ref="el" class="masonry" :style="{ gap: `${gap}px` }" :aria-busy="skeleton || undefined">
    <div v-for="(col, i) in columns" :key="i" class="col" :style="{ gap: `${gap}px` }">
      <template v-for="entry in col" :key="entry.key">
        <WallpaperCard
          v-if="entry.item"
          :item="entry.item"
          :collector-mode="collectorMode"
          :is-collected="collectedIds ? collectedIds.has(entry.item.id) : false"
          @open="emit('open', $event)"
          @collect="emit('collect', $event)"
        />
        <div v-else class="ghost" :style="{ aspectRatio: `1 / ${entry.ratio}` }" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.masonry {
  display: flex;
  align-items: flex-start;
}

.col {
  display: flex;
  flex: 1 1 0;
  min-width: 0;
  flex-direction: column;
}

.ghost {
  width: 100%;
  border-radius: var(--r-tile);
  background: linear-gradient(100deg, var(--surface) 30%, var(--surface-2) 50%, var(--surface) 70%);
  background-size: 300% 100%;
  animation: sweep 1.6s linear infinite;
}

@keyframes sweep {
  from {
    background-position: 100% 0;
  }
  to {
    background-position: 0 0;
  }
}
</style>
