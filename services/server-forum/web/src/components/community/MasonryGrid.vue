<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { masonrySpan } from '@/lib/cardPresentation'
const grid = ref<HTMLElement>()
withDefaults(defineProps<{ columns?:2|3 }>(),{columns:3})
let resize: ResizeObserver | undefined, mutation: MutationObserver | undefined, frame = 0
const observed = new Set<HTMLElement>()
function schedule() {
  if (frame) return
  frame = requestAnimationFrame(() => {
    frame = 0
    if (!grid.value) return
    const gap = parseFloat(getComputedStyle(grid.value).rowGap) || 12
    for (const item of observed) {
      const span = `span ${masonrySpan(item.getBoundingClientRect().height, 4, gap)}`
      if (item.style.gridRowEnd !== span) item.style.gridRowEnd = span
    }
    grid.value.dataset.ready = 'true'
  })
}
function observeChildren() {
  const children = new Set(Array.from(grid.value?.children || []) as HTMLElement[])
  for (const item of observed) if (!children.has(item)) { resize?.unobserve(item); observed.delete(item) }
  for (const item of children) if (!observed.has(item)) { observed.add(item); resize?.observe(item) }
  schedule()
}
onMounted(() => {
  if (!grid.value || typeof ResizeObserver === 'undefined') return
  resize = new ResizeObserver(schedule); resize.observe(grid.value)
  mutation = new MutationObserver(observeChildren); mutation.observe(grid.value, { childList: true })
  observeChildren()
})
onBeforeUnmount(() => { resize?.disconnect(); mutation?.disconnect(); cancelAnimationFrame(frame) })
</script>
<template><div ref="grid" class="masonry-grid" :class="{'two-columns':columns===2}"><slot /></div></template>
<style scoped>
.masonry-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;align-items:start;min-width:0}
.masonry-grid[data-ready="true"]{grid-auto-rows:4px}
@media(min-width:960px){.masonry-grid{grid-template-columns:repeat(3,minmax(0,1fr))}}
.masonry-grid.two-columns{grid-template-columns:repeat(2,minmax(0,1fr))}
</style>
