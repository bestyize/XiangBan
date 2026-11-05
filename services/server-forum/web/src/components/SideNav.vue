<script setup lang="ts">
import { computed } from 'vue'
import { categoryDestination, categoryLabel, navigationSection } from '@/lib/communityStructure'
import { SHOW_CHILD_THRESHOLD, useMeta } from '@/stores/meta'

const props = defineProps<{ active?: string }>()
const meta = useMeta()

const activeSection = computed(() => {
  const category = props.active ? meta.bySlug.get(props.active) : undefined
  return category ? navigationSection(category) : undefined
})
function visibleChildren(node: (typeof meta.navigation)[number]) {
  return activeSection.value === node.key ? node.children : node.children.filter(c=>c.topic_count>=SHOW_CHILD_THRESHOLD)
}
</script>

<template>
  <nav class="side" aria-label="分类">
    <RouterLink to="/" class="item top" :class="{ on: active === 'home' }">首页</RouterLink>
    <div v-for="node in meta.navigation" :key="node.key" class="group">
      <RouterLink
        :to="node.href"
        class="item root"
        :class="{ on: activeSection === node.key }"
      >
        <span>{{ node.name }}</span>
        <span class="count">{{ node.topic_count || '' }}</span>
      </RouterLink>
      <RouterLink
        v-for="child in visibleChildren(node)"
        :key="child.id"
        :to="categoryDestination(child,meta.categories)"
        class="item child"
        :class="{ on: active === child.slug }"
      >
        <span>{{ categoryLabel(child) }}</span>
        <span class="count">{{ child.topic_count || '' }}</span>
      </RouterLink>
    </div>
  </nav>
</template>

<style scoped>
.side {
  display: grid;
  gap: 1px;
  padding-bottom: 20px;
}
.group {
  display: grid;
  gap: 1px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--line);
}
.item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: var(--r-sm);
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.item:hover {
  background: var(--surface);
  color: var(--ink);
}
.item.top,
.item.root {
  font-weight: 700;
  color: var(--ink);
  font-size: var(--fs-md);
}
.item.child {
  padding-left: 22px;
}
.item.on {
  background: var(--surface);
  color: var(--ink);
  box-shadow: inset 3px 0 0 var(--brand), 0 0 0 1px var(--line);
}
.count {
  font-family: var(--font-display);
  font-size: 12px;
  color: var(--ink-3);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}
</style>
